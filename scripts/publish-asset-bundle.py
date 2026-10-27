#!/usr/bin/env python3
"""Publish changed Flutter assets from Android APK or AAB build artifacts."""

import argparse
import base64
import subprocess
import tempfile
import hashlib
import io
import json
import os
import urllib.request
import urllib.parse
import zipfile


PREFIXES = ('assets/flutter_assets/', 'base/assets/flutter_assets/')


def assets(apk):
    with zipfile.ZipFile(apk) as archive:
        result = {}
        for name in archive.namelist():
            for prefix in PREFIXES:
                if name.startswith(prefix) and not name.endswith('/'):
                    result[name[len(prefix):]] = archive.read(name)
        return result


def build_bundle(base_apk, patched_apk):
    base = assets(base_apk)
    patched = assets(patched_apk)
    if 'AssetManifest.bin' not in patched:
        raise ValueError('Patched APK has no AssetManifest.bin')
    changed = {name: data for name, data in patched.items()
               if name == 'AssetManifest.bin' or base.get(name) != data}
    output = io.BytesIO()
    with zipfile.ZipFile(output, 'w', compression=zipfile.ZIP_DEFLATED,
                         compresslevel=9) as archive:
        for name, data in sorted(changed.items()):
            archive.writestr(name, data)
    return output.getvalue(), changed


def request(url, method, token, payload, content_type):
    headers = {'Content-Type': content_type}
    if token:
        headers['Authorization'] = 'Bearer ' + token
    req = urllib.request.Request(url, data=payload, headers=headers, method=method)
    with urllib.request.urlopen(req, timeout=120) as response:
        return json.load(response)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--base-apk', required=True, help='Baseline APK or AAB')
    parser.add_argument('--patched-apk', required=True, help='Patched APK or AAB')
    parser.add_argument('--app-id', required=True)
    parser.add_argument('--patch-id', required=True, type=int)
    parser.add_argument('--server', default='http://127.0.0.1:3000')
    parser.add_argument('--output', default='saturn-assets.zip')
    parser.add_argument('--asset-signing-key', help='Separate Ed25519 PEM private key; public key must be pinned in the app baseline')
    parser.add_argument('--release-version')
    parser.add_argument('--patch-number', type=int)
    parser.add_argument('--publish', action='store_true')
    parser.add_argument('--allow-manifest-only', action='store_true',
                        help='Allow a bundle with no changed asset file (for deletions)')
    args = parser.parse_args()
    payload, changed = build_bundle(args.base_apk, args.patched_apk)
    with open(args.output, 'wb') as file:
        file.write(payload)
    digest = hashlib.sha256(payload).hexdigest()
    print(json.dumps({'output': args.output, 'files': sorted(changed),
                      'size': len(payload), 'sha256': digest}, indent=2))
    if not args.publish:
        return
    if set(changed) == {'AssetManifest.bin'} and not args.allow_manifest_only:
        raise SystemExit('No changed asset file found. Check that the patched build is the new AAB/APK, or pass --allow-manifest-only for deletions.')
    token = os.environ.get('SATURN_API_TOKEN')
    if not token:
        raise SystemExit('Set SATURN_API_TOKEN before --publish')
    endpoint = (args.server.rstrip('/') + '/api/v1/apps/' + args.app_id +
                '/patches/' + str(args.patch_id) + '/asset-bundle')
    if not args.asset_signing_key or not args.release_version or not args.patch_number or args.patch_number < 1:
        raise SystemExit('--publish requires --asset-signing-key, --release-version and positive --patch-number')
    manifest = json.dumps(['saturn-assets-v1', args.app_id, args.release_version,
                           args.patch_number, 'android', digest, len(payload)],
                          ensure_ascii=False, separators=(',', ':')).encode()
    with tempfile.TemporaryDirectory() as directory:
        message = os.path.join(directory, 'manifest')
        with open(message, 'wb') as file:
            file.write(manifest)
        signature = subprocess.check_output(['openssl', 'pkeyutl', '-sign', '-rawin',
                                            '-inkey', args.asset_signing_key, '-in', message])
    if len(signature) != 64:
        raise SystemExit('Expected a 64-byte Ed25519 signature')
    metadata = json.dumps({'platform': 'android', 'sha256': digest,
                           'size': len(payload), 'signature': base64.b64encode(signature).decode()}).encode()
    registered = request(endpoint, 'POST', token, metadata, 'application/json')
    upload_url = registered['upload_url']
    # The public URL may point through a tunnel. Upload to the chosen local
    # server while retaining the unguessable capability path.
    upload_path = urllib.parse.urlparse(upload_url).path
    request(args.server.rstrip('/') + upload_path, 'POST', None, payload,
            'application/octet-stream')
    print('Published asset bundle id', registered['id'])


if __name__ == '__main__':
    main()
