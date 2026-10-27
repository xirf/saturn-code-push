import base64
import importlib.util
import json
import pathlib
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('publisher', pathlib.Path(__file__).parents[1] / 'publish-asset-bundle.py')
publisher = importlib.util.module_from_spec(spec)
spec.loader.exec_module(publisher)


class AssetSigningTest(unittest.TestCase):
    def test_published_manifest_signature_binds_identity_and_content(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            key = root / 'key.pem'
            public = root / 'public.pem'
            subprocess.run(['openssl', 'genpkey', '-algorithm', 'ED25519', '-out', str(key)], check=True)
            subprocess.run(['openssl', 'pkey', '-in', str(key), '-pubout', '-out', str(public)], check=True)
            requests = []

            def request(url, method, token, payload, content_type):
                requests.append(payload)
                return {'id': 1, 'upload_url': 'https://example.com/asset-uploads/test'}

            args = ['publisher', '--base-apk', 'base', '--patched-apk', 'patched', '--app-id', 'app', '--patch-id', '99', '--release-version', '1+1', '--patch-number', '3', '--asset-signing-key', str(key), '--output', str(root / 'bundle.zip'), '--publish']
            with patch.object(sys, 'argv', args), patch.dict('os.environ', {'SATURN_API_TOKEN': 'local-test-token'}), patch.object(publisher, 'build_bundle', return_value=(b'zip fixture', {'asset': b'new'})), patch.object(publisher, 'request', side_effect=request):
                publisher.main()
            metadata = json.loads(requests[0])
            message = json.dumps(['saturn-assets-v1', 'app', '1+1', 3, 'android', metadata['sha256'], metadata['size']], separators=(',', ':')).encode()
            (root / 'manifest').write_bytes(message)
            (root / 'signature').write_bytes(base64.b64decode(metadata['signature']))
            command = ['openssl', 'pkeyutl', '-verify', '-pubin', '-inkey', str(public), '-rawin', '-in', str(root / 'manifest'), '-sigfile', str(root / 'signature')]
            self.assertEqual(subprocess.run(command, capture_output=True).returncode, 0)
            (root / 'manifest').write_bytes(message.replace(b'1+1', b'2+1'))
            self.assertNotEqual(subprocess.run(command, capture_output=True).returncode, 0)


if __name__ == '__main__':
    unittest.main()
