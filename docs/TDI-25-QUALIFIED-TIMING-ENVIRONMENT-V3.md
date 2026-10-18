# TDI-25 slice 47 — qualified timing environment, V3 amendment (`tdi25-qualified-timing-environment-v3`)

Status: frozen protocol amendment, approved by the user on 2026-10-18 after
attempt 2. It amends `docs/TDI-25-QUALIFIED-TIMING-ENVIRONMENT-V2.md`
(`tdi25-qualified-timing-environment-v2`), which stays in the repository
unchanged, as do V1 and the records of attempts 1 and 2. Attempt 1 remains
judged under V1, attempt 2 under V2 (both **non qualifiée**). V3 applies to
attempts ≥ 3 **on this machine only**. Slice 47 stays **timing non
qualifié** until a qualified attempt lands. No performance claim.

Manifest: `docs/tdi25-qualified-timing-environment-v3.yaml`. The checked-in
`QUALIFIED_TIMING_ENVIRONMENT` is now the V3 T430 manifest; the harness
accepts exactly that manifest.

## Why amend

Attempt 2 (T430, governor `performance`, `no_turbo=1`, `taskset 28`,
load1 0.97) failed **only** the V2 frequency rule in all 4 cells, with
excellent IQR (1–11 ns) and stable medians (T6 3680/3688 ns,
C6 3824/3822 ns per 64-case pass, operator-reported; raw pending):

1. **APERF/MPERF semantics.** `IA32_MPERF` counts at the invariant
   TSC/base frequency and `IA32_APERF` at the actual core frequency, both
   only in C0. Their ratio is the average **actual** frequency over the
   measured window, expressed relative to the 2.1 GHz base. It is not a
   promise that the core runs **at** the base frequency.
2. **Attempt-2 evidence.** The effective APERF/MPERF frequency was about
   1.89 GHz. With turbo off and the `performance` governor this machine
   runs stably at about 1.9 GHz (its P1 / guaranteed frequency), below
   the 2.1 GHz TSC base. The V2 absolute check
   `fixed_frequency_khz = 2100000 kHz ±1 %` therefore rejects a stable,
   unthrottled run — it confuses "frequency is stable and not throttled"
   with "frequency equals the TSC base", which APERF/MPERF does not
   measure.

## Changes from V2

| Rule | V2 | V3 |
| --- | --- | --- |
| Frequency measurement | APERF/MPERF MSR deltas on CPU 28 around the 31 measured runs of each cell: effective = 2 100 000 kHz × ΔAPERF / ΔMPERF | unchanged |
| Frequency verdict | each cell: \|effective − 2 100 000\| ≤ 1 % of 2 100 000 | **replaced**: each cell: effective within **±1 % of the median effective frequency across the 4 cells of the run** AND effective ≥ **0.85 × 2 100 000 = 1 785 000 kHz** (anti-throttling floor); the per-cell ratio `effective / median_effective` and the median are recorded in `results.jsonl` |
| Gate logic | `summarize_timing_cell` (per cell) | per-cell floor in `summarize_timing_cell`; cross-cell median gate in `cross_cell_frequency_gate` (Rust, unit-tested incl. negative cases) mirrored by the measurement script, which writes the final `cell_status` |

Scope: **this machine only** (Dell PowerEdge T430, Xeon E5-2683 v4).
The floor and the median rule are not portable frequency claims.

Unchanged: machine, toolchain 1.97.1, pinning (`numactl --membind=0
taskset -c 28`), SMT sibling 60 busy ≤ 1 %, governor/turbo set-and-restore,
load ≤ 2 at start (precondition) and per cell, swap counters stable,
5 warm-up + 31 measured runs per cell, median (primary)/min/IQR with
ranks 8/16/24 and IQR ≤ 5 % of the median, no rerun, logical memory
primary, `VmHWM` informational, attempts numbered and all recorded.

Median definition: over the present (non-missing) effective frequencies
of the run; for 4 values the mean of the two middle values. A missing
effective frequency (ΔMPERF = 0) fails its cell and is excluded from the
median. The script records `frequency_median_effective_khz` and
`frequency_ratio_to_median` on every result line.

## No fishing

Attempts are numbered sequentially and **every attempt is recorded** in
the repository (`results/tdi25_s47_timing/attempt-<N>/` and a
`docs/TDI-25-S47-TIMING-ATTEMPT-<N>.md` record), whatever its outcome.
A precondition refusal (load, MSR, fingerprint) happens before any
measurement, writes no results and does not consume an attempt number.
Once measuring has started, the attempt is recorded even if every cell
is rejected. Results are never selected across attempts.

## Pre-run checklist (done by the user, not by the script)

Same as V2: pause heavy workloads (qemu/libvirt, ollama, docker — the
script never stops services), wait for load1 ≤ 2, `modprobe msr`.

## Commands (attempt 3)

```bash
# as user tdi, on the T430, after this V3 PR is merged:
cd /home/tdi/TDI && git fetch origin && git checkout <merged V3 commit>
bash scripts/tdi25-s47-qualified-timing.sh build
# as root, after the checklist:
modprobe msr
cd /home/tdi/TDI && bash scripts/tdi25-s47-qualified-timing.sh run 3
# then resume what you paused, and provide verbatim:
cat /home/tdi/tdi25-s47-timing-runs/tdi25-s47-timing-*-attempt-3/results.jsonl
cat /home/tdi/tdi25-s47-timing-runs/tdi25-s47-timing-*-attempt-3/fingerprint.json
```

Outputs (default parent `/home/tdi/tdi25-s47-timing-runs/`):
`run-id.txt`, `fingerprint.json`, `results.jsonl`, `restore.log`.
The operator never runs attempt 3 from the integration box; it is run by
the user on the T430 only.

## Recorded degeneracies

- APERF/MPERF bracket the 31 measured runs, so each effective frequency
  is a cell average, not a per-run value; the cross-cell median inherits
  that averaging.
- A run in which all 4 cells throttle **together** by the same amount
  passes the median-consistency rule; only the 1 785 000 kHz floor
  guards that case, by design (it is an anti-throttling floor, not a
  nominal-frequency claim).
- `MSR_PLATFORM_INFO` attests the nominal base ratio (21); the TSC
  frequency is assumed equal to the base on this CPU (invariant TSC,
  Broadwell-EP), unchanged from V2.
- The host remains shared; quiescing is the user's responsibility and is
  not verified beyond load average and SMT-sibling idleness.
