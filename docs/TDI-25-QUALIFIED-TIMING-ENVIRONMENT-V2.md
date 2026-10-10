# TDI-25 slice 47 — qualified timing environment, V2 amendment (`tdi25-qualified-timing-environment-v2`)

Status: frozen protocol amendment, approved by the user on 2026-10-10 after
attempt 1. It amends `docs/TDI-25-QUALIFIED-TIMING-ENVIRONMENT-V1.md`
(`tdi25-qualified-timing-environment-v1`), which stays in the repository
unchanged, as does the record of attempt 1
(`docs/TDI-25-S47-TIMING-ATTEMPT-1.md`). Attempt 1 remains judged under V1
(non qualifiée). V2 applies to attempts ≥ 2. Slice 47 stays **timing non
qualifié** until a qualified attempt lands. No performance claim.

Manifest: `docs/tdi25-qualified-timing-environment-v2.yaml`. The checked-in
`QUALIFIED_TIMING_ENVIRONMENT` is now the V2 T430 manifest; the harness
accepts exactly that manifest.

## Why amend

Attempt 1 (run `tdi25-s47-timing-8bd8aea`) rejected all 4 cells for two
environmental reasons, while IQR was ≈ 0.4–0.5 % of the median:

1. **Frequency rule unreliable.** `scaling_cur_freq` read 1.57–2.07 GHz
   versus the fixed 2.1 GHz although governor `performance` and
   `no_turbo = 1` were set and attested. Under `intel_pstate`,
   `scaling_cur_freq` is a kernel estimate sampled outside the timed window,
   not a measurement of the CPU frequency during the measured runs. The rule
   could not distinguish a real frequency deviation from estimator noise.
2. **Load.** load1 was 2.41 / 2.22 > 2 on the shared host. The threshold
   was right; the protocol lacked a start precondition and a documented way
   to quiesce the host.

The amendment changes **how** the frequency is measured and adds a start
precondition. It does **not** relax any threshold: frequency tolerance stays
±1 % of 2.1 GHz, load threshold stays 2, IQR rule stays 5 %, 5 + 31 runs.

## Changes from V1

| Rule | V1 | V2 |
| --- | --- | --- |
| Frequency measurement | `scaling_cur_freq` after each run vs `scaling_max_freq` | APERF/MPERF MSR deltas on CPU 28 around the 31 measured runs of each cell: effective = 2 100 000 kHz × ΔAPERF / ΔMPERF |
| Base frequency | `scaling_max_freq` | frozen 2 100 000 kHz, attested against `MSR_PLATFORM_INFO` bits 15:8 (ratio 21) |
| MSR access | — | `/dev/cpu/28/msr` (`msr` module, root); refuse if unreadable (`attestation_msr_unreadable`) |
| Frequency tolerance | ±1 % | ±1 % (unchanged); ΔMPERF = 0 rejects the cell |
| Load at start | per-cell rejection only | additionally a **start precondition**: refuse without measuring if load1 > 2, printing the top CPU consumers |
| Services | — | never stopped by the script; the user follows the checklist below |
| Attempts | output directory chosen freely | numbered attempts (`run <N>`, N ≥ 2), run id `tdi25-s47-timing-<commit12>-attempt-<N>`, refuse if the run directory exists or attempt N is already recorded |

Unchanged: machine, toolchain 1.97.1, pinning (`numactl --membind=0
taskset -c 28`), SMT sibling 60 idle, governor/turbo set-and-restore,
5 warm-up + 31 measured runs per cell, median (primary)/min/IQR with
ranks 8/16/24, per-cell rejection rules (IQR > 5 % median, load1 > 2 at
start or end, swap change, frequency), no rerun, logical memory primary,
`VmHWM` informational, results valid for this machine only.

MPERF counts at the base (TSC) frequency and APERF at the actual frequency,
both only in C0, so their ratio is the average active frequency of CPU 28
over the measured runs — the quantity the V1 rule intended to check.

## No fishing

Attempts are numbered sequentially and **every attempt is recorded** in the
repository (`results/tdi25_s47_timing/attempt-<N>/` and a
`docs/TDI-25-S47-TIMING-ATTEMPT-<N>.md` record), whatever its outcome.
A precondition refusal (load, MSR, fingerprint) happens before any
measurement, writes no results and does not consume an attempt number.
Once measuring has started, the attempt is recorded even if every cell is
rejected. Results are never selected across attempts.

## Pre-run checklist (done by the user, not by the script)

Before attempt 2, on the T430, as root:

1. Look at the top consumers: `ps -eo pid,user,pcpu,etime,comm --sort=-pcpu | head -20`.
2. Pause the heavy workloads, recording what you paused so you can resume it:
   - qemu/libvirt VMs: `virsh list` then `virsh suspend <vm>` (resume:
     `virsh resume <vm>`); for bare qemu processes, `kill -STOP <pid>`
     (resume: `kill -CONT <pid>`);
   - ollama: `systemctl stop ollama` (resume: `systemctl start ollama`);
   - docker/containerd workloads: `docker ps` then `docker pause <container>`
     (resume: `docker unpause <container>`).
3. Wait until `cat /proc/loadavg` shows a 1-min load ≤ 2 (it decays over a
   few minutes).
4. Load the MSR driver if needed: `modprobe msr` (check `ls /dev/cpu/28/msr`).
5. Run the attempt (commands below), then resume what you paused.

## Commands (attempt 2)

```bash
# as user tdi
cd /home/tdi/TDI && git fetch origin && git checkout <merged commit>
bash scripts/tdi25-s47-qualified-timing.sh build
# as root, after the checklist
modprobe msr
cd /home/tdi/TDI && bash scripts/tdi25-s47-qualified-timing.sh run 2
```

Outputs (default parent `/home/tdi/tdi25-s47-timing-runs/`):
`run-id.txt`, `fingerprint.json`, `results.jsonl`, `restore.log`.

## Recorded degeneracies

- APERF/MPERF are read with `pread` on `/dev/cpu/28/msr` from the pinned
  process; the reads bracket the 31 measured runs, so the effective
  frequency is a cell average, not a per-run value.
- `MSR_PLATFORM_INFO` attests the nominal base ratio; it does not prove the
  TSC frequency, which is assumed equal to the base on this CPU (invariant
  TSC, Broadwell-EP).
- The load precondition can be satisfied by waiting; the per-cell load rule
  still applies during the run.
- The host remains shared; quiescing is the user's responsibility and is not
  verified beyond load average and SMT-sibling idleness.
