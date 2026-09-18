"""Actual FLAT/Hub and NNIS source-validator qualification, without model data."""
import copy
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

import tdi_attention_fixture as attention
import tdi_experiment_supervisor as durable
from tdi_engine_store import EngineStore
from tdi_nnis_qualification_review import review_checkout, _pinned_blob
import test_tdi_engine_integration as operational


class ProbeBoundaryTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.worker = attention.pin(Path(os.environ['TDI_ATTENTION_PROBE']))

    def test_actual_flat_matches_independent_mha_gqa_mqa_causal_and_offset_oracles(self):
        for index in range(6):
            request = attention.fixture_request(index)
            actual = attention.call_probe(self.worker, request, run=True)
            self.assertEqual(0, actual['return_code'], actual['response'])
            evidence = attention.check_observation(request, actual['response'])
            self.assertLess(evidence['max_absolute_output_error'], 3e-6)
            self.assertIsNone(actual['cost_measurements']['gpu_time'])
            changed = copy.deepcopy(actual['response']); changed['output'][0] += 0.01
            with self.assertRaises(durable.ContractError): attention.check_observation(request, changed)

    def test_causal_future_values_cannot_change_first_output(self):
        request = attention.fixture_request(1)
        original = attention.call_probe(self.worker, request, run=True)['response']
        width = request['shape']['head_dim']
        request['v'][width:] = [31] * (len(request['v']) - width)
        altered = attention.call_probe(self.worker, request, run=True)['response']
        self.assertEqual(original['output'][:width], altered['output'][:width])
        self.assertNotEqual(original['output'][width:], altered['output'][width:])

    def test_invalid_semantics_duplicates_nonfinite_shapes_and_no_implicit_cuda(self):
        from tdi_physical_telemetry import measured_process
        request = attention.fixture_request(0)
        variants = []
        for key, value in (('domain', 'Confirmation'), ('dtype', 'bf16'), ('mechanism', 'recurrent'),
                           ('mask', 'boolean'), ('layout', 'BNHD'), ('schema', True), ('scale', 0)):
            variants.append(dict(request, **{key: value}))
        x = copy.deepcopy(request); x['shape']['query_len'] = 33; variants.append(x)
        x = copy.deepcopy(request); x['q'].pop(); variants.append(x)
        x = copy.deepcopy(request); x['q'][0] = 33; variants.append(x)
        x = copy.deepcopy(request); x['shape']['q_heads'] = 3; x['shape']['kv_heads'] = 2; variants.append(x)
        x = attention.fixture_request(3); x['backend'] = 'nnis-cuda-fused'; variants.append(x)
        for value in variants:
            result = attention.call_probe(self.worker, value)
            self.assertNotEqual(0, result['return_code']); self.assertEqual('rejected', result['response']['status'])
        for raw in (b'{"schema":1,' + json.dumps(request).encode()[1:], json.dumps(request).replace('0.25', '1e999').encode()):
            status, out, _, _ = measured_process([self.worker['path'], 'validate'], input_bytes=raw)
            self.assertNotEqual(0, status); self.assertEqual('rejected', json.loads(out)['status'])
        oversized = subprocess.run([self.worker['path'], 'validate'], input=b' ' * (1024 * 1024 + 1), capture_output=True, timeout=5)
        self.assertEqual(21, oversized.returncode); self.assertIn('1 MiB', json.loads(oversized.stdout)['error'])
        for index in range(6):
            nnis = attention.fixture_request(index, 'nnis-cuda-fused')
            self.assertEqual('validated', attention.call_probe(self.worker, nnis)['response']['status'])
            rejected = attention.call_probe(self.worker, nnis, run=True)
            self.assertNotEqual(0, rejected['return_code']); self.assertIn('allow-cuda', rejected['response']['error'])
        with self.assertRaises(durable.ContractError): attention.pin(self.worker['path'], '0' * 64)

    def test_actual_nnis_owned_validator_keeps_unresolved_status_and_rejects_blob_drift(self):
        checkout = Path(os.environ['TDI_NNIS_CHECKOUT'])
        report = review_checkout(checkout)
        self.assertEqual('unresolved_blocking', report['status'])
        self.assertFalse(report['qualification_resolved']); self.assertFalse(report['hardware_execution_performed'])
        self.assertIn('fail-closed and unresolved', report['validator_stdout'])
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory); (root / 'changed').write_text('{}')
            with self.assertRaises(durable.ContractError): _pinned_blob(root, 'changed', '0' * 40)
            (root / 'link').symlink_to(root / 'changed')
            with self.assertRaises(durable.ContractError): _pinned_blob(root, 'link', '0' * 40)
        with self.assertRaises(durable.ContractError): review_checkout(Path(__file__).parents[1])


class AttentionHubTests(unittest.TestCase):
    setUp = operational.OperationalIntegrationTests.setUp
    start_hub = operational.OperationalIntegrationTests.start_hub
    stop_hub = operational.OperationalIntegrationTests.stop_hub
    cli = operational.OperationalIntegrationTests.cli

    @classmethod
    def setUpClass(cls):
        cls.hubd = Path(os.environ['TDI_HUBD_BIN']).resolve(strict=True)
        cls.worker = Path(os.environ['TDI_ATTENTION_PROBE']).resolve(strict=True)

    def test_actual_public_operator_dag_archive_and_no_implicit_device_plan(self):
        path = self.root / 'attention.json'
        self.cli('attention-fixture-plan', '--worker', self.worker, '--trials', 6, '--output', path)
        campaign = self.cli('submit', path)['campaign']
        self.assertEqual('completed', self.cli('run', campaign)['phase'])
        with EngineStore(self.catalogue, readonly=True) as store:
            rows = store.results(campaign)
            self.assertEqual(12, len(rows))
            for row in rows:
                self.assertFalse(row['evidence']['cache_eligible'])
                result = row['evidence']['json']
                if result['status'] == 'Verified': self.assertLess(result['evidence']['max_absolute_output_error'], 3e-6)
                else: self.assertIsNone(result['execution']['response']['device'])
        self.stop_hub(); self.start_hub()
        self.assertEqual('completed', self.cli('resume', campaign)['phase'])
        receipt = self.cli('export', campaign, self.root / 'bundle.json')
        self.assertEqual(12, receipt['members'])
        self.cli('verify', self.root / 'bundle.json', '--expected-identity', receipt['identity'])
        self.cli('attention-fixture-plan', '--worker', self.worker, '--backend', 'nnis-cuda-fused',
                 '--output', self.root / 'nnis.json', expected_code=durable.EXIT_CONTRACT)
        self.assertFalse((self.root / 'nnis.json').exists())

    def test_real_launch_failure_retains_attempt_cost_without_verified_result(self):
        worker = self.root / 'unlaunchable-worker'
        shutil.copyfile(self.worker, worker); worker.chmod(0o600)
        path = self.root / 'failed.json'
        self.cli('attention-fixture-plan', '--worker', worker, '--trials', 1, '--output', path)
        campaign = self.cli('submit', path)['campaign']
        self.assertEqual('failed', self.cli('run', campaign, expected_code=durable.EXIT_TRIAL_FAILURE)['phase'])
        with EngineStore(self.catalogue, readonly=True) as store:
            rows = store.results(campaign); self.assertEqual(1, len(rows))
            execution = rows[0]['evidence']['json']['execution']
            self.assertIsNone(execution['response'])
            self.assertEqual('process-launch-failed', execution['cost_measurements']['technical_failure'])
            self.assertIsNone(execution['cost_measurements']['peak_rss_bytes'])


if __name__ == '__main__': unittest.main()
