#!/usr/bin/env python3
"""Probe Saturn with stable simulated installation IDs; never download patches."""
import argparse
import json
import sys
import uuid
from urllib.parse import urlsplit
from urllib.request import Request, build_opener, HTTPRedirectHandler


class NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


def origin(value):
    parts = urlsplit(value)
    if (parts.scheme not in ('https', 'http') or not parts.hostname
            or parts.username or parts.password or parts.path not in ('', '/')
            or parts.query or parts.fragment):
        raise argparse.ArgumentTypeError('server must be an HTTP(S) origin without credentials or a path')
    return value.rstrip('/')


def check(server, payload):
    request = Request(server + '/api/v1/patches/check',
                      data=json.dumps(payload).encode(),
                      headers={'Content-Type': 'application/json'}, method='POST')
    with build_opener(NoRedirect()).open(request, timeout=15) as response:
        raw = response.read(65537)
    if len(raw) > 65536:
        raise ValueError('patch-check response exceeds 64 KiB')
    result = json.loads(raw)
    if not isinstance(result, dict) or type(result.get('patch_available')) is not bool:
        raise ValueError('invalid patch-check response')
    number = None
    if result['patch_available']:
        patch = result.get('patch')
        if not isinstance(patch, dict) or type(patch.get('number')) is not int:
            raise ValueError('available patch has no integer number')
        number = patch['number']
    rolled_back = result.get('rolled_back_patch_numbers')
    if not isinstance(rolled_back, list) or any(type(n) is not int for n in rolled_back):
        raise ValueError('invalid rollback instructions')
    # Never print download capabilities, signatures, or installation identifiers.
    return number, tuple(sorted(rolled_back))


def probe(server, payload, target, installations, cohort, expected=None):
    included = 0
    for index in range(installations):
        identity = str(uuid.uuid5(uuid.NAMESPACE_URL, f'saturn-smoke:{cohort}:{index}'))
        request = dict(payload, client_id=identity)
        first = check(server, request)
        if check(server, request) != first:
            raise ValueError('selection changed for a repeated identity; freeze publishing during the probe')
        included += first[0] == target
    legacy = check(server, payload)
    if expected == 0 and included != 0:
        raise ValueError('paused target was offered')
    if expected == 100 and included != installations:
        raise ValueError('full rollout target was not offered to every simulated installation')
    if expected == 100 and legacy[0] != target:
        raise ValueError('full rollout target was not offered to a client without an ID')
    if expected is not None and expected < 100 and legacy[0] == target:
        raise ValueError('partial rollout target was offered to a client without an ID')
    return {'simulated_installations': installations, 'target_offered': included,
            'observed_percentage': round(included * 100 / installations, 2),
            'stable_repeated_checks': True,
            'without_id_target_offered': legacy[0] == target}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--server', type=origin, required=True)
    parser.add_argument('--app-id', required=True)
    parser.add_argument('--release-version', required=True)
    parser.add_argument('--platform', default='android')
    parser.add_argument('--arch', default='aarch64')
    parser.add_argument('--channel', default='stable')
    parser.add_argument('--target-patch', type=int, required=True, help='patch number, not database ID')
    parser.add_argument('--current-patch', type=int, default=0)
    parser.add_argument('--installations', type=int, default=20)
    parser.add_argument('--cohort', default='community-smoke', help='reuse this name to keep simulated IDs stable')
    parser.add_argument('--expect-percentage', type=int, choices=range(101), metavar='0..100')
    args = parser.parse_args()
    if not 1 <= args.installations <= 1000:
        parser.error('installations must be from 1 to 1000')
    if args.current_patch < 0 or args.target_patch <= args.current_patch:
        parser.error('target-patch must be greater than the nonnegative current-patch')
    payload = {'app_id': args.app_id, 'release_version': args.release_version,
               'platform': args.platform, 'arch': args.arch, 'channel': args.channel,
               'current_patch_number': args.current_patch or None}
    try:
        result = probe(args.server, payload, args.target_patch, args.installations,
                       args.cohort, args.expect_percentage)
    except Exception as error:
        # Avoid dumping request/response content or capability URLs.
        print(f'Rollout probe failed ({type(error).__name__}). Check deployment and rollout state.', file=sys.stderr)
        return 1
    print(json.dumps(result, indent=2))
    return 0


if __name__ == '__main__':
    sys.exit(main())
