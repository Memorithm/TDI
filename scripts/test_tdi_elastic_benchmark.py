"""Methodology regressions; real binary qualification is a separate CI step."""
import tempfile
import json
from pathlib import Path
from types import SimpleNamespace
import unittest
from unittest.mock import patch

import tdi_elastic_benchmark as bench
import tdi_experiment_supervisor as durable


class ElasticBenchmarkTests(unittest.TestCase):
    def test_balanced_fixed_schedule_and_bounds(self):
        cases = bench.schedule([1, 2, 4, 8], 4)
        self.assertEqual(32, len(cases))
        self.assertEqual([(0, 1, "fixed"), (0, 1, "elastic")], cases[:2])
        self.assertEqual([(1, 1, "elastic"), (1, 1, "fixed")], cases[8:10])
        for widths, repeats in [([], 2), ([1, 1], 2), ([True], 2), ([9], 2), ([1], 0), ([1], 21)]:
            with self.subTest(widths=widths, repeats=repeats), self.assertRaises(durable.ContractError):
                bench.schedule(widths, repeats)

    def test_refusals_are_not_fast_successes(self):
        records = [{"requested_width": 2, "arm": "elastic", "status": "completed", "actual_width": 1,
                    "trials": 8, "submit_execute_collect": {"wall_ns": 2_000_000_000}},
                   {"requested_width": 2, "arm": "elastic", "status": "rejected",
                    "trials": 8, "submit_execute_collect": {"wall_ns": 1_000_000_000}}]
        result = bench.summarize(records)[0]
        self.assertEqual(2, result["attempted"])
        self.assertEqual(1, result["failed_or_rejected"])
        self.assertEqual(2_000_000_000, result["median_wall_ns"])
        self.assertAlmostEqual(8 / 3, result["successful_trials_per_second_in_timed_windows"])
        rejected = bench.summarize(records[1:])[0]
        self.assertIsNone(rejected["median_wall_ns"])
        self.assertIsNone(rejected["p95_wall_ns_nearest_rank"])
        self.assertEqual(0, rejected["successful_trials_per_second_in_timed_windows"])

    def test_payloads_bind_every_step_without_completion_order(self):
        rows = [{"step": mode + "-0", "output": "file:result", "evidence": {"json": {"mode": mode}}}
                for mode in ("run", "verify")]
        self.assertEqual(bench.payload_identity(rows, 1), bench.payload_identity(rows[::-1], 1))
        for incomplete in (rows[:1], rows + rows[:1], [rows[0], rows[0]]):
            with self.assertRaises(durable.ContractError):
                bench.payload_identity(incomplete, 1)
        changed = [{**r, "evidence": {"json": {"changed": True}}} for r in rows]
        self.assertNotEqual(bench.payload_identity(rows, 1), bench.payload_identity(changed, 1))

    def test_failure_preserves_case_and_manifest_without_complete_report(self):
        with tempfile.TemporaryDirectory() as directory:
            binary = Path(__file__).resolve()
            args = SimpleNamespace(widths=[1], repeats=1, trials=1, memory_bytes_per_trial=1,
                                   reserve_memory_bytes=0, elastic_source="a" * 40, hub_source="b" * 40,
                                   hubd=binary, worker=binary, elastic=binary, output=Path(directory) / "out")
            with patch.object(bench, "run_case", side_effect=RuntimeError("synthetic qualification fault")):
                with self.assertRaisesRegex(RuntimeError, "qualification fault"):
                    bench.run(args)
            self.assertTrue((args.output / "manifest.json").is_file())
            self.assertTrue((args.output / "case-000.json").is_file())
            self.assertFalse((args.output / "report.json").exists())
            with self.assertRaises(FileExistsError):
                bench.run(args)

    def test_verifier_rejects_rehashed_misleading_summary(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = {"schedule": [[0, 1, "elastic"]], "trials": 1}
            record = {"repeat": 0, "requested_width": 1, "arm": "elastic", "trials": 1,
                      "status": "rejected", "submit_execute_collect": {"wall_ns": 100}}
            report = {"manifest_identity": bench.identity("tdi-elastic-benchmark-manifest/v1", manifest),
                      "records": [record], "status": "contains-rejections", "summary": bench.summarize([record])}
            report["identity"] = bench.identity("tdi-elastic-benchmark-report/v1", report)
            for name, value in (("manifest", manifest), ("case-000", record), ("report", report)):
                (root / (name + ".json")).write_text(json.dumps(value))
            self.assertEqual(report["identity"], bench.verify_report(root))
            report["summary"][0]["completed"] = 1
            report.pop("identity")
            report["identity"] = bench.identity("tdi-elastic-benchmark-report/v1", report)
            (root / "report.json").write_text(json.dumps(report))
            with self.assertRaisesRegex(durable.ContractError, "summary"):
                bench.verify_report(root)

    def test_report_is_not_published_when_final_verification_fails(self):
        with tempfile.TemporaryDirectory() as directory:
            binary = Path(__file__).resolve()
            args = SimpleNamespace(widths=[1], repeats=1, trials=1, memory_bytes_per_trial=1,
                                   reserve_memory_bytes=0, elastic_source="a" * 40, hub_source="b" * 40,
                                   hubd=binary, worker=binary, elastic=binary, output=Path(directory) / "out")
            def rejected(root, **kwargs):
                return {"arm": kwargs["arm"], "requested_width": 1, "trials": 1, "status": "rejected",
                        "submit_execute_collect": {"wall_ns": 100}}
            with patch.object(bench, "run_case", side_effect=rejected), patch.object(
                    bench, "verify_report", side_effect=durable.ContractError("synthetic verification failure")):
                with self.assertRaisesRegex(durable.ContractError, "verification failure"):
                    bench.run(args)
            self.assertTrue((args.output / "case-001.json").exists())
            self.assertFalse((args.output / "report.json").exists())


if __name__ == "__main__":
    unittest.main()
