"""Benchmark counter correctness and predeclared comparison policy boundaries."""
import copy
import json
from pathlib import Path
import tempfile
import unittest

import tdi_engine_benchmark as benchmark
import tdi_experiment_supervisor as durable
from tdi_engine_store import identity


def capacity(cpu_slots=2, cpu_raw="200000 100000", memory_limit=1073741824):
    """Synthetic stable-capacity fixture; no hardware observation is asserted."""
    return {"scope": "synthetic-limit-fixture", "capacity": {"status": "available", "cpu_slots": cpu_slots, "available_memory_bytes": memory_limit // 2},
            "readings": [{"sensor": "/sys/fs/cgroup/test/cpu.max", "unit": "microsecond quota per period", "raw": cpu_raw},
                         {"sensor": "/sys/fs/cgroup/test/memory.max", "unit": "byte", "limit": memory_limit,
                          "current": memory_limit // 2, "available": memory_limit // 2}]}


class BenchmarkTests(unittest.TestCase):
    def test_real_journal_counts_storage_memory_and_reopen(self):
        for n in (4, 32):
            with tempfile.TemporaryDirectory() as root:
                result = benchmark.journal_case(Path(root), n, 128)
            self.assertEqual(2 * n, result['append_work']['hash_calls'])
            self.assertEqual(0, result['append_work']['full_scans'])
            self.assertEqual(1, result['audit_work']['full_scans'])
            self.assertGreater(result['audit_work']['hash_input_bytes'], 0)
            self.assertGreater(result['database_bytes'], 0)
            self.assertGreaterEqual(result['python_peak_bytes'], result['python_retained_bytes'])
            self.assertGreater(result['reopen_audit']['wall_ns'], 0)

    def test_real_catalogue_count_queries_and_serialization(self):
        with tempfile.TemporaryDirectory() as root:
            result = benchmark.catalogue_case(Path(root), 64)
        self.assertGreater(result['populate']['wall_ns'], 0)
        self.assertTrue(result['query_plan'])
        encoded = benchmark.serialization_case(1024)
        self.assertGreater(encoded['bytes_per_operation'], 1024)
        self.assertGreater(encoded['sha256']['wall_ns'], 0)

    def test_warmups_are_excluded_from_descriptive_summaries(self):
        # Comparator-only synthetic numbers; never published as measurements.
        records = [{'phase': phase, 'measurements': {'wall_ns': value}} for phase, value in
                   [('warmup', 999999), ('measured', 10), ('measured', 20), ('measured', 30)]]
        result = benchmark.summarize(records)['/wall_ns']
        self.assertEqual({'median': 20, 'min': 10, 'max': 30, 'repetitions': 3}, result)

    def test_predeclared_policy_rejects_corruption_missing_metrics_and_incompatible_environment(self):
        case = {'kind': 'minimal-worker'}
        environment = {key: 'synthetic-comparator-fixture' for key in ('python', 'platform', 'machine', 'sqlite', 'cpu_affinity', 'hardware')}
        environment['capacity'] = {'capacity': {'status': 'available', 'cpu_slots': 2, 'available_memory_bytes': 100},
                                   'scope': 'synthetic-limit-fixture', 'readings': [{'sensor': '/cpu.max', 'raw': '200000 100000'},
                                       {'sensor': '/memory.max', 'limit': 1000, 'current': 900}]}
        baseline = {'schema': 1, 'kind': 'tdi-engine-benchmark', 'status': 'measured',
                    'plan': {'cases': [case], 'warmup': 1, 'repetitions': 3, 'environment': environment,
                             'files': {name: '0' * 64 for name in benchmark.DEPLOYMENT_MODULES}},
                    'summary': [{'case': case, 'metrics': {'/wall_ns': {'median': 100}}}]}
        baseline['identity'] = identity('tdi-engine-benchmark/v1', baseline)
        policy = {'schema': 1, 'baseline_identity': baseline['identity'], 'reason': 'comparator unit fixture',
                  'limits': [{'case': case, 'metric': '/wall_ns', 'max_ratio': 1.1}]}
        benchmark.validate_baseline(baseline, policy)
        current = copy.deepcopy(baseline)
        current['summary'][0]['metrics']['/wall_ns']['median'] = 111
        self.assertEqual('regression', benchmark.compare(baseline, current, policy)['status'])
        current['summary'][0]['metrics']['/wall_ns']['median'] = 109
        self.assertEqual('within-policy', benchmark.compare(baseline, current, policy)['status'])
        current['plan']['environment']['capacity']['capacity']['available_memory_bytes'] = 200
        current['plan']['environment']['capacity']['readings'][1]['current'] = 800
        self.assertEqual('within-policy', benchmark.compare(baseline, current, policy)['status'])
        for field in ('cpu', 'memory', 'unknown'):
            changed = copy.deepcopy(current)
            capacity = changed['plan']['environment']['capacity']
            if field == 'cpu': capacity['readings'][0]['raw'] = '150000 100000'
            if field == 'memory': capacity['readings'][1]['limit'] = 2000
            if field == 'unknown': capacity['capacity']['status'] = 'unknown'
            self.assertEqual('incompatible', benchmark.compare(baseline, changed, policy)['status'])
        current['plan']['environment']['hardware'] = 'different CPU/filesystem'
        self.assertEqual('incompatible', benchmark.compare(baseline, current, policy)['status'])
        for invalid in (dict(policy, baseline_identity='0' * 64), dict(policy, limits=[]),
                        dict(policy, limits=[{'case': case, 'metric': '/missing_ns', 'max_ratio': 1.1}]),
                        dict(policy, limits=[{'case': case, 'metric': '/wall_ns', 'max_ratio': float('nan')}])):
            with self.assertRaises(durable.ContractError): benchmark.validate_baseline(baseline, invalid)
        corrupted = copy.deepcopy(baseline); corrupted['summary'][0]['metrics']['/wall_ns']['median'] = 99
        with self.assertRaises(durable.ContractError): benchmark.validate_baseline(corrupted, policy)

    def test_committed_q04_evidence_binds_no_history_reread_to_measured_scaling(self):
        path = Path(__file__).resolve().parents[1] / 'docs' / 'engineering' / 'benchmarks' / '2026-09-16-engine-baseline-qualified.json'
        report = json.loads(path.read_text())
        evidence = benchmark.validate_q04_evidence(report)
        self.assertEqual('no-history-reread-with-measured-scaling-observations', evidence['qualification'])
        self.assertEqual(report['identity'], evidence['report_identity'])
        self.assertEqual([64, 4096], [profile['payload_bytes'] for profile in evidence['profiles']])
        self.assertTrue(all(profile['counts'] == [32, 128, 512] for profile in evidence['profiles']))
        self.assertIn('no asymptotic latency class', evidence['limitations'])

    def test_q04_evidence_rejects_tampered_append_work_even_with_recomputed_identity(self):
        path = Path(__file__).resolve().parents[1] / 'docs' / 'engineering' / 'benchmarks' / '2026-09-16-engine-baseline-qualified.json'
        report = json.loads(path.read_text())
        tampered = copy.deepcopy(report)
        row = next(record for record in tampered['records']
                   if record['phase'] == 'measured' and record['case'].get('kind') == 'journal')
        row['measurements']['append_work']['full_scans'] = 1
        tampered['identity'] = identity('tdi-engine-benchmark/v1', {k: v for k, v in tampered.items() if k != 'identity'})
        with self.assertRaises(durable.ContractError):
            benchmark.validate_q04_evidence(tampered)

    def test_process_duration_strings_cpu_and_outer_rss_remain_comparable(self):
        records = [{'phase': 'measured', 'measurements': {'worker_process': {'wall_ns': '100', 'user_cpu_seconds': .001, 'system_cpu_seconds': .002}},
                    'outer_process': {'wall_ns': '200', 'peak_rss_bytes': 4096}}]
        metrics = benchmark.summarize(records)
        self.assertEqual(100, metrics['/worker_process/wall_ns']['median'])
        self.assertEqual(.001, metrics['/worker_process/user_cpu_seconds']['median'])
        self.assertEqual(200, metrics['/whole_case_process/wall_ns']['median'])
        self.assertEqual(4096, metrics['/whole_case_process/peak_rss_bytes']['median'])
        self.assertIn('tdi_engine_cache.py', benchmark.DEPLOYMENT_MODULES)

    def test_process_numeric_encodings_remain_comparable(self):
        records = [{'phase': 'measured', 'measurements': {'worker_process': {
            'wall_ns': value, 'user_cpu_seconds': cpu, 'peak_rss_bytes': rss}}}
            for value, cpu, rss in [('100', .01, 1024), ('120', .02, 2048), ('140', .03, 3072)]]
        summary = benchmark.summarize(records)
        self.assertEqual(120, summary['/worker_process/wall_ns']['median'])
        self.assertAlmostEqual(.02, summary['/worker_process/user_cpu_seconds']['median'])
        self.assertEqual(2048, summary['/worker_process/peak_rss_bytes']['median'])
        self.assertNotEqual(benchmark.stable_capacity(capacity()), benchmark.stable_capacity(capacity(cpu_slots=4, cpu_raw='400000 100000')))
        self.assertNotEqual(benchmark.stable_capacity(capacity()), benchmark.stable_capacity(capacity(memory_limit=2147483648)))


if __name__ == '__main__': unittest.main()
