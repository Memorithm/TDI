from pathlib import Path
import sys
import tempfile
import unittest

SCRIPTS = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPTS))
from tdi_cgroup_delegation import require_inside_delegation, unified_process_path
from tdi_linux_containment import ContainmentError


class DelegationTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name) / "cgroup"
        self.parent = self.root / "tdi"
        self.manager = self.parent / "manager"
        self.manager.mkdir(parents=True)
        self.proc = Path(self.temp.name) / "self.cgroup"

    def test_unified_path_is_strict(self):
        self.assertEqual(unified_process_path("0::/tdi/manager\n"), "/tdi/manager")
        for raw in ("", "1:name:/legacy\n", "0::relative\n",
                    "0::/one\n0::/two\n"):
            with self.assertRaises(ContainmentError):
                unified_process_path(raw)

    def test_supervisor_inside_delegation_is_accepted(self):
        self.proc.write_text("0::/tdi/manager\n")
        current = require_inside_delegation(
            self.parent, cgroup_root=self.root, proc_self_cgroup=self.proc)
        self.assertEqual(current, self.manager)

    def test_supervisor_outside_delegation_is_rejected(self):
        outside = self.root / "other"
        outside.mkdir()
        self.proc.write_text("0::/other\n")
        with self.assertRaisesRegex(ContainmentError, "outside the delegated"):
            require_inside_delegation(
                self.parent, cgroup_root=self.root, proc_self_cgroup=self.proc)


if __name__ == "__main__":
    unittest.main()
