import argparse
import importlib.util
from pathlib import Path
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('rollout_probe', Path(__file__).parents[1] / 'check-rollout.py')
probe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)


class RolloutProbeTest(unittest.TestCase):
    def test_simulated_identities_are_reused_across_checks_and_runs(self):
        calls = []
        def check(server, payload):
            calls.append(payload.copy())
            return (2 if payload.get('client_id') else 1), ()
        with patch.object(probe, 'check', side_effect=check):
            result = probe.probe('https://example.com', {}, 2, 3, 'demo')
            first = calls.copy()
            calls.clear()
            probe.probe('https://example.com', {}, 2, 3, 'demo')
        self.assertEqual(first, calls)
        self.assertEqual(calls[0]['client_id'], calls[1]['client_id'])
        self.assertNotEqual(calls[0]['client_id'], calls[2]['client_id'])
        self.assertNotIn('client_id', calls[-1])
        self.assertEqual(result['target_offered'], 3)

    def test_changing_selection_fails(self):
        with patch.object(probe, 'check', side_effect=[(2, ()), (1, ())]):
            with self.assertRaisesRegex(ValueError, 'selection changed'):
                probe.probe('https://example.com', {}, 2, 1, 'demo')

    def test_paused_and_full_rollout_assertions(self):
        for expected, number in [(0, 2), (100, 1)]:
            with patch.object(probe, 'check', return_value=(number, ())):
                with self.assertRaises(ValueError):
                    probe.probe('https://example.com', {}, 2, 1, 'demo', expected)

    def test_legacy_client_must_not_receive_partial_rollout(self):
        with patch.object(probe, 'check', return_value=(2, ())):
            with self.assertRaisesRegex(ValueError, 'without an ID'):
                probe.probe('https://example.com', {}, 2, 1, 'demo', 25)

    def test_origin_validation(self):
        self.assertEqual(probe.origin('https://example.com/'), 'https://example.com')
        for value in ['file:///tmp/a', 'https://user:pass@example.com', 'https://example.com/path']:
            with self.assertRaises(argparse.ArgumentTypeError):
                probe.origin(value)

    def test_http_transport_sends_native_body_and_omits_sensitive_output(self):
        import io
        import json
        body = json.dumps({'patch_available': True, 'patch': {
            'number': 2, 'download_url': 'https://example.com/objects/secret'
        }, 'rolled_back_patch_numbers': [4, 3]}).encode()
        response = io.BytesIO(body)
        with patch.object(probe, 'build_opener') as opener:
            opener.return_value.open.return_value = response
            result = probe.check('https://example.com', {'client_id': 'private-id', 'current_patch_number': 1})
            request = opener.return_value.open.call_args.args[0]
        self.assertEqual(request.full_url, 'https://example.com/api/v1/patches/check')
        self.assertEqual(request.get_method(), 'POST')
        self.assertEqual(json.loads(request.data)['client_id'], 'private-id')
        self.assertEqual(result, (2, (3, 4)))
        self.assertNotIn('secret', str(result))

    def test_malformed_or_oversized_responses_fail(self):
        import io
        import json
        for raw in [b'x' * 65537, b'[]', b'{}', json.dumps({
                'patch_available': True, 'patch': {}, 'rolled_back_patch_numbers': []
        }).encode()]:
            with patch.object(probe, 'build_opener') as opener:
                opener.return_value.open.return_value = io.BytesIO(raw)
                with self.assertRaises(ValueError):
                    probe.check('https://example.com', {})
