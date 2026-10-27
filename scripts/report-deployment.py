#!/usr/bin/env python3
"""Report a deployment from GitHub Actions using standard-library Python only."""
import argparse
import json
import os
import urllib.error
import urllib.parse
import urllib.request


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--release-id', required=True, type=int)
    parser.add_argument('--destination', required=True)
    parser.add_argument('--status', required=True, choices=['pending', 'running', 'failed', 'uploaded', 'in_review', 'published'])
    parser.add_argument('--build-number', default='')
    parser.add_argument('--source-kind', choices=['commit', 'tag', 'release'])
    args = parser.parse_args()
    required = ['SATURN_URL', 'SATURN_TOKEN', 'SATURN_APP_ID', 'GITHUB_REPOSITORY', 'GITHUB_SHA', 'GITHUB_RUN_ID', 'GITHUB_REF_NAME']
    missing = [name for name in required if not os.environ.get(name)]
    if missing:
        parser.error('Missing environment variables: ' + ', '.join(missing))
    base = os.environ['SATURN_URL'].rstrip('/')
    url = urllib.parse.urlsplit(base)
    if url.scheme != 'https' or not url.netloc or url.username or url.password or url.query or url.fragment:
        parser.error('SATURN_URL must be an HTTPS URL without credentials, query or fragment')
    payload = {
        'release_id': args.release_id,
        'key': f"{os.environ['GITHUB_REPOSITORY']}:{os.environ['GITHUB_RUN_ID']}:{os.environ.get('GITHUB_RUN_ATTEMPT', '1')}:{args.destination}",
        'repository': os.environ['GITHUB_REPOSITORY'],
        'commit': os.environ['GITHUB_SHA'],
        'source_kind': args.source_kind or ('release' if os.environ.get('GITHUB_EVENT_NAME') == 'release' else 'tag' if os.environ.get('GITHUB_REF_TYPE') == 'tag' else 'commit'),
        'source_ref': os.environ['GITHUB_REF_NAME'],
        'workflow_run': int(os.environ['GITHUB_RUN_ID']),
        'destination': args.destination,
        'status': args.status,
        'build_number': args.build_number,
    }
    request = urllib.request.Request(
        f"{base}/api/v1/apps/{urllib.parse.quote(os.environ['SATURN_APP_ID'], safe='')}/deployments",
        data=json.dumps(payload).encode(),
        headers={'Authorization': 'Bearer ' + os.environ['SATURN_TOKEN'], 'Content-Type': 'application/json'},
        method='POST',
    )
    # Never forward the API token through a redirect.
    class NoRedirect(urllib.request.HTTPRedirectHandler):
        def redirect_request(self, req, fp, code, msg, headers, newurl):
            return None
    try:
        with urllib.request.build_opener(NoRedirect).open(request, timeout=30) as response:
            print(f'Deployment reported ({response.status})')
    except urllib.error.HTTPError as error:
        raise SystemExit(f'Deployment reporting failed (HTTP {error.code})') from None
    except urllib.error.URLError:
        raise SystemExit('Deployment reporting failed: could not reach Saturn') from None


if __name__ == '__main__':
    main()
