"""Benchmark counter correctness and predeclared comparison policy boundaries."""
import copy
from pathlib import Path
import tempfile
import unittest

import tdi_engine_benchmark as benchmark
import tdi_experiment_supervisor as durable
from tdi_engine_store import identity


def capacity(cpu_slots=2, cpu_raw="200000 100000", memory_limit=1073741824):
    """Synthetic stable-capacity fixture; timestamps/current usage are intentionally absent."""
    return {"capacity": {"status": "available", "cpu_slots": cpu_slots, "available_memory_bytes": memory_limit // 2},
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
        records = [{'phase': phase, 'measurements': {'wall_ns': value}} for phase, value in
                   [('warmup', 999999), ('measured', 10), ('measured', 20), ('measured', 30)]]
        result = benchmark.summarize(records)['/wall_ns']
        self.assertEqual({'median': 20, 'min': 10, 'max': 30, 'repetitions': 3}, result)

    def test_process_numeric_encodings_remain_comparable(self):
        records = [{'phase': 'measured', 'measurements': {'worker_process': {
            'wall_ns': value, 'user_cpu_seconds': cpu, 'peak_rss_bytes': rss}}}
            for value, cpu, rss in [('100', .01, 1024), ('120', .02, 2048), ('140', .03, 3072)]]
        summary = benchmark.summarize(records)
        self.assertEqual(120, summary['/worker_process/wall_ns']['median'])
        self.assertAlmostEqual(.02, summary['/worker_process/user_cpu_seconds']['median'])
        self.assertEqual(2048, summary['/worker_process/peak_rss_bytes']['median'])

    def test_predeclared_policy_rejects_corruption_missing_metrics_and_incompatible_environment(self):
        case = {'kind': 'minimal-worker'}
        environment = {key: 'synthetic-comparator-fixture' for key in ('python', 'platform', 'machine', 'sqlite', 'cpu_affinity', 'hardware')}
        environment['capacity'] = capacity()
        baseline = {'schema': 1, 'kind': 'tdi-engine-benchmark', 'status': 'measured',
                    'plan': {'cases': [case], 'warmup': 1, 'repetitions': 3, 'environment': environment},
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
        current['plan']['environment']['hardware'] = 'different CPU/filesystem'
        self.assertEqual('incompatible', benchmark.compare(baseline, current, policy)['status'])
        current = copy.deepcopy(baseline)
        current['plan']['environment']['capacity'] = capacity(cpu_slots=4, cpu_raw="400000 100000")
        self.assertEqual('incompatible', benchmark.compare(baseline, current, policy)['status'])
        current = copy.deepcopy(baseline)
        current['plan']['environment']['capacity'] = capacity(memory_limit=2147483648)
        self.assertEqual('incompatible', benchmark.compare(baseline, current, policy)['status'])
        current = copy.deepcopy(baseline)
        current['plan']['environment']['capacity']['capacity']['available_memory_bytes'] -= 4096
        current['plan']['environment']['capacity']['readings'][1]['current'] += 4096
        current['plan']['environment']['capacity']['readings'][1]['available'] -= 4096
        self.assertEqual('within-policy', benchmark.compare(baseline, current, policy)['status'])
        for invalid in (dict(policy, baseline_identity='0' * 64), dict(policy, limits=[]),
                        dict(policy, limits=[{'case': case, 'metric': '/missing_ns', 'max_ratio': 1.1}]),
                        dict(policy, limits=[{'case': case, 'metric': '/wall_ns', 'max_ratio': float('nan')}])):
            with self.assertRaises(durable.ContractError): benchmark.validate_baseline(baseline, invalid)
        corrupted = copy.deepcopy(baseline); corrupted['summary'][0]['metrics']['/wall_ns']['median'] = 99
        with self.assertRaises(durable.ContractError): benchmark.validate_baseline(corrupted, policy)


if __name__ == '__main__': unittest.main()
