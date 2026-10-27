import contextlib
import importlib.util
import io
import json
import os
from pathlib import Path
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('report_deployment', Path(__file__).parents[1] / 'report-deployment.py')
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class DeploymentReportingTest(unittest.TestCase):
    def report(self, *arguments):
        captured = []
        class Response:
            status = 200
            def __enter__(self): return self
            def __exit__(self, *args): return False
        class Opener:
            def open(self, request, timeout):
                captured.append(json.loads(request.data))
                return Response()
        variables = {'SATURN_URL': 'https://updates.example.com', 'SATURN_TOKEN': 'test-secret-never-log', 'SATURN_APP_ID': 'app', 'GITHUB_REPOSITORY': 'owner/repo', 'GITHUB_SHA': 'a' * 40, 'GITHUB_RUN_ID': '42', 'GITHUB_RUN_ATTEMPT': '1', 'GITHUB_REF_NAME': 'v1', 'GITHUB_EVENT_NAME': 'release'}
        output = io.StringIO()
        with patch.dict(os.environ, variables), patch('sys.argv', ['report-deployment.py', '--release-id', '2', '--destination', 'production', '--status', 'uploaded', *arguments]), patch.object(module.urllib.request, 'build_opener', return_value=Opener()), contextlib.redirect_stdout(output):
            module.main()
        self.assertNotIn(variables['SATURN_TOKEN'], output.getvalue())
        return captured[0]

    def test_github_release_trigger_is_independent_of_saturn_patch_target(self):
        release = self.report()
        patch_report = self.report('--patch-id', '3')
        self.assertIsNone(release['patch_id'])
        self.assertEqual(patch_report['release_id'], 2)
        self.assertEqual(patch_report['patch_id'], 3)
        self.assertEqual(patch_report['source_kind'], 'release')
        self.assertEqual(patch_report['workflow_attempt'], 1)
        self.assertNotEqual(release['key'], patch_report['key'])

    def test_invalid_patch_id_is_rejected_before_network_access(self):
        with patch('sys.argv', ['report-deployment.py', '--release-id', '2', '--patch-id', '-1', '--destination', 'production', '--status', 'uploaded']), patch.object(module.urllib.request, 'build_opener') as network, contextlib.redirect_stderr(io.StringIO()):
            with self.assertRaises(SystemExit): module.main()
            network.assert_not_called()
