# TDI-AI Linux containment and recovery

This document describes the non-final Linux resource boundary introduced for TDI Development/Validation workers. It does not authorize a scientific stage, does not make cgroup v2 a filesystem/network sandbox, and does not claim a GPU VRAM quota.

## Architecture

Schema-1 durable execution from PR #250 remains unchanged. Schema-2 plans use an additive path:

- `tdi_linux_contract.py`: validates the schema-2 execution contract and derives plan/attempt identities;
- `tdi_linux_runner.py`: reuses the qualified durable Journal and bounded-output supervisor;
- `tdi_linux_campaign.py`: CLI and exit-code mapping;
- `tdi_linux_containment.py`: cgroup-v2 limits, diagnostics and recovery owner;
- `tdi_cgroup_exec.py`: attaches the exact child to `cgroup.procs` before `exec`.

The durable journal stores a compatibility envelope containing the complete immutable schema-2 plan. Thus a resource-profile change changes the journal binding and prevents accidental resume, while the qualified schema-1 Start/Finish event form remains intact.

## Resource profile

A schema-2 plan binds this versioned structure:

```json
{
  "schema": 1,
  "memory_max_bytes": 536870912,
  "swap_max_bytes": 0,
  "cpu_quota_us": 100000,
  "cpu_period_us": 100000,
  "pids_max": 64,
  "trust": "trusted",
  "gpu_required": false,
  "gpu_memory_max_bytes": null
}
```

`memory.max`, `memory.swap.max`, `cpu.max` and `pids.max` are written then read back. `memory.oom.group=1` is enabled. Evidence records effective limits and available counters from `memory.*`, `cpu.stat`, `pids.*` and `cgroup.events`.

`trust=untrusted` fails closed because cgroups are not a complete sandbox. `gpu_memory_max_bytes` also fails closed: device visibility is not a hard VRAM quota. `gpu_required=true` only checks that an NVIDIA device is visible.

## Process tree and recovery

Each trial receives a deterministic `attempt_id` derived from full schema-2 plan identity, trial index and attempt ordinal. A dedicated `tdi-<attempt_id>` cgroup is created and verified before durable Start. The launcher joins that cgroup before `exec`.

A detached recovery helper watches the exact supervisor process through `pidfd`. If that process dies, the helper invokes `cgroup.kill` and writes an fsync-backed recovery marker. On restart, an unfinished attempt is reconciled before TDI records `Interrupted`; it is not silently converted into a fresh scientific trial. Normal completion, cancellation and timeout also clean the complete cgroup tree.

## Doctor

Read-only capability diagnosis launches no scientific work:

```bash
python3 scripts/tdi_linux_containment.py doctor
```

or for a delegated parent:

```bash
python3 scripts/tdi_linux_containment.py doctor --parent /sys/fs/cgroup/tdi
```

The JSON output reports cgroup-v2/controllers, parent writability, `cgroup.kill`, `pidfd`, NVIDIA visibility and the presence of `unshare`. It explicitly reports `sandbox_qualified: false` and `gpu_memory_quota_qualified: false`.

## Prepare and run

Only explicitly selected artifacts are pinned:

```bash
python3 scripts/prepare_tdi_experiment_plan.py \
  --root ./fixture --worker worker --artifact input.json \
  --index 0 --index 1 --domain Development \
  --timeout 30 --output-limit 1048576 \
  --containment cgroup-v2 \
  --memory-max-bytes 536870912 --swap-max-bytes 0 \
  --cpu-quota-us 100000 --cpu-period-us 100000 --pids-max 64 \
  --plan ./plan.json
```

The delegated cgroup parent is deployment state, not scientific identity:

```bash
python3 scripts/tdi_linux_campaign.py \
  --root ./fixture --plan ./plan.json --journal ./campaign.sqlite \
  --cgroup-parent /sys/fs/cgroup/tdi
```

An unavailable delegated parent or controller fails before worker launch; the contained CLI uses capability exit code `23`.

## Qualification

Standard tests exercise parsing, fail-closed behavior, exact cgroup writes/readbacks, attempt identity, pidfd recovery and campaign resume. These mocks do not prove kernel enforcement.

`scripts/test_tdi_linux_containment_privileged.py` requires `TDI_CGROUP_TEST_PARENT` and exercises real kernel memory pressure, `pids.max`, CPU throttling, tree kill and sibling isolation. The dedicated GitHub workflow provisions a disposable parent on a compatible runner. A skipped privileged test is not Q07/Q08 evidence.

Kernel semantics are based on the primary Linux cgroup-v2 documentation: <https://docs.kernel.org/admin-guide/cgroup-v2.html> (`cpu.max`, `pids.max`, `memory.swap.max`, `cgroup.kill`).

## Known limits

- no qualified hostile-code filesystem/network sandbox yet;
- no hard NVIDIA VRAM quota claim;
- a writable delegated cgroup-v2 parent with `cpu`, `memory`, `pids` enabled is required;
- local recovery does not replace distributed Hub lease/fencing semantics;
- Debian 12 has not yet been qualified by an executed installation recipe, so no Debian-specific success claim is made.
