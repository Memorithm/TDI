"""Validate cgroup-v2 delegation containment for a TDI supervisor process."""
from pathlib import Path

from tdi_linux_containment import ContainmentError

CGROUP_ROOT = Path("/sys/fs/cgroup")
PROC_SELF_CGROUP = Path("/proc/self/cgroup")


def unified_process_path(raw: str) -> str:
    """Return the unified cgroup-v2 path from ``/proc/<pid>/cgroup`` text."""
    matches = []
    for line in raw.splitlines():
        parts = line.split(":", 2)
        if len(parts) == 3 and parts[0] == "0" and parts[1] == "":
            matches.append(parts[2])
    if len(matches) != 1 or not matches[0].startswith("/"):
        raise ContainmentError("cannot determine a unique unified cgroup-v2 process path")
    return matches[0]


def require_inside_delegation(cgroup_parent, *, cgroup_root=CGROUP_ROOT,
                              proc_self_cgroup=PROC_SELF_CGROUP):
    """Require this supervisor to live at or below the delegated parent.

    Linux cgroup-v2 migration permission is constrained by the common ancestor
    of the source and destination cgroups. A process outside the delegated
    subtree can therefore be unable to move its own children into an otherwise
    writable delegated cgroup. Failing here prevents a durable trial Start from
    being recorded for a deployment that cannot attach its worker.
    """
    root = Path(cgroup_root).resolve(strict=True)
    parent = Path(cgroup_parent).resolve(strict=True)
    if not parent.is_relative_to(root):
        raise ContainmentError("cgroup parent is outside the unified cgroup-v2 mount")
    try:
        raw = Path(proc_self_cgroup).read_text(encoding="ascii")
    except (OSError, UnicodeError) as error:
        raise ContainmentError(f"cannot read supervisor cgroup membership: {error}") from error
    relative = unified_process_path(raw).lstrip("/")
    current = (root / relative).resolve(strict=True)
    if current != parent and parent not in current.parents:
        raise ContainmentError(
            "supervisor process is outside the delegated cgroup subtree; "
            "launch it inside the delegation before starting workers"
        )
    return current
