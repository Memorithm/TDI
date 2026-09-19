"""Profiler accounting and failure preservation without changing real policies."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("profile_session", Path(__file__).with_name("profile-forge-session.py"))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class ProfileTests(unittest.TestCase):
    def test_separate_self_and_cumulative(self):
        rows = module.function_rows({("x", 2, "parent"): (1, 1, 0.2, 0.7, {}),
                                     ("x", 3, "child"): (1, 4, 0.5, 0.5, {})})
        self.assertEqual(rows[0]["function"], "child")
        self.assertEqual(rows[1]["cumulative_seconds"], 0.7)
        self.assertEqual(rows[0]["calls"], 4)

    def test_failure_retains_diagnostics_and_propagates(self):
        with tempfile.TemporaryDirectory() as temp:
            output = Path(temp) / "profile"
            with patch.object(module.benchmark, "run", side_effect=RuntimeError("failure")), \
                 patch.object(module.benchmark, "git_revision", return_value="fixture"):
                with self.assertRaisesRegex(RuntimeError, "failure"):
                    module.profile_run(Path("unused"), output)
            payload = json.loads((output / "profile.json").read_text())
            self.assertEqual(payload["status"], "incomplete")
            self.assertIsNone(payload["benchmark_report_identity"])
            self.assertTrue(payload["functions"])
            with self.assertRaises(FileExistsError):
                module.profile_run(Path("unused"), output)

    def test_complete_requires_verified_report(self):
        with tempfile.TemporaryDirectory() as temp:
            output = Path(temp) / "profile"
            with patch.object(module.benchmark, "run", return_value={"identity": "fixture"}) as run, \
                 patch.object(module.benchmark, "verify_report", side_effect=ValueError("invalid")), \
                 patch.object(module.benchmark, "git_revision", return_value="fixture"):
                with self.assertRaisesRegex(ValueError, "invalid"):
                    module.profile_run(Path("unused"), output)
                self.assertEqual(run.call_args.args[2], "session-smoke")
            self.assertEqual(json.loads((output / "profile.json").read_text())["status"], "incomplete")


if __name__ == "__main__":
    unittest.main()
