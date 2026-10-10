# TDI-25 slice 47 — qualified timing environment (`tdi25-qualified-timing-environment-v1`)

Status: frozen protocol, experimental, non-final. Approved by the user on
2026-10-10 (22:02 Europe/Paris). This document freezes **where and how**
slice 47 may time the T6 and C6 reference scores. It records **no timing
result**: the slice stays **timing non qualifié** until the user's run on the
machine below lands in a separate PR. No scientific, production, GPU or
performance claim; Development/Validation only; holdouts untouched.

Machine manifest: `docs/tdi25-qualified-timing-environment.yaml`. Code:
`T430_QUALIFIED_TIMING_ENVIRONMENT` in `tdi-ai/src/tdi25_eval.rs`
(`QUALIFIED_TIMING_ENVIRONMENT` is exactly that constant). Script:
`scripts/tdi25-s47-qualified-timing.sh`. Binary:
`tdi-ai/examples/tdi25_s47_qualified_timing.rs`.

## Qualified environment

| Item | Frozen value |
| --- | --- |
| System | Dell PowerEdge T430, bare metal, hostname `debian`, shared host |
| BIOS | 2.19.0 (2024-02-22) |
| CPU | 2 × Intel Xeon E5-2683 v4 @ 2.10 GHz, 32 cores / 64 threads, SMT on |
| NUMA | 2 nodes; node 0 = even CPUs; SMT sibling of CPU n is n + 32 |
| Memory | 128 GB DDR4-2133 |
| OS / kernel | Debian 13.4, `6.12.88+deb13-amd64`, mitigations on |
| Toolchain | rustc 1.97.1 (rustup; the observed system rustc 1.98.1 is not accepted) |
| Tools | `taskset`, `numactl` (required); `cpupower`, `chrt` present, unused |

Results are valid for this machine only.

## Protocol

1. **Fingerprint** each run: CPU model, kernel, BIOS, `rustc -Vv` (must be
   1.97.1), git commit and tree (tree must be clean), binary SHA-256,
   original and set governor/turbo, load average, swap counters, kernel
   command line → `<outdir>/fingerprint.json`.
2. **Pinning**: `numactl --membind=0 taskset -c 28`; single core. The SMT
   sibling CPU 60 must be idle (checked and recorded before each cell).
3. **Frequency**: governor `performance` on CPUs 28 and 60,
   `intel_pstate/no_turbo = 1`, original values restored on exit (trap) and
   the restore verified. Root is required; the run refuses if the values
   cannot be set or do not read back.
4. **Cells**: {development, validation} × {T6, C6} on the smoke budget; one
   run = one pass over the bounded matched population (slice-47 harness
   definition). Each cell: **5 warm-up runs, then 31 measured runs**.
5. **Statistics**: median (primary), min, IQR. Quartiles are the order
   statistics of rank 8, 16 and 24 of the 31 sorted samples (no
   interpolation; the `(n+1)p` rule and Tukey's hinges coincide at n = 31).
6. **Rejection** (cell reported `non_qualifiee`, **no rerun**):
   - IQR > 5 % of the median;
   - 1-min load average > 2 at the start or the end of the cell;
   - any change of `/proc/vmstat` `pswpin`/`pswpout` during the cell;
   - CPU 28 frequency deviates from the fixed value.
7. **Memory**: logical bytes from the slice-47 accounting are primary;
   `VmHWM` is informational only.
8. **Operator**: the user runs it. The bot never runs timing.

## Implementation thresholds (to be confirmed by the user)

The protocol leaves these operational details open; the frozen
implementation uses:

- **Frequency deviation**: every `scaling_cur_freq` reading of CPU 28,
  taken after each measured run outside the timed window, must lie within
  ±1 % of `scaling_max_freq` read at the start of the cell.
- **SMT sibling idle**: CPU 60 busy fraction ≤ 1 % over a 1 s `/proc/stat`
  sample immediately before each cell.
- **Quartile convention**: ranks 8/16/24 of 31 as above.
- **Run granularity**: a run is a single population pass (64 pair scores per
  split), i.e. microsecond scale; short runs make IQR rejection more likely.
  This is recorded, not tuned.

Changing any of these is a new protocol version.

## Gate

`run_qualified_reference_timing_cell` refuses with
`timing_environment_mismatch` unless the checked-in environment equals the
T430 manifest, then with `attestation_toolchain`, `attestation_cpu_model`,
`attestation_kernel`, `attestation_bios`, `attestation_pinning`
(`Cpus_allowed_list` ≠ `28`), `attestation_governor` or `attestation_turbo`
unless the running host and process match. CI runners are refused. The
script additionally refuses on hostname, SMT topology, NUMA placement, a
dirty tree, a binary built from another commit or with another rustc, and
an existing `results.jsonl` (no rerun).

## Command (run by the user on the T430)

```bash
# as the normal user, in a clean checkout at the merged commit
rustup toolchain install 1.97.1
bash scripts/tdi25-s47-qualified-timing.sh build
# then, as root
sudo bash scripts/tdi25-s47-qualified-timing.sh run tdi25-s47-timing-<commit>
```

Outputs: `fingerprint.json`, `results.jsonl` (one line per cell with
`cell_status`, `rejection_reasons`, median/min/IQR and the raw samples) and
`restore.log`. Landing those results is a separate, user-approved PR.

## Recorded degeneracies

- The timed unit is a microsecond-scale population pass; the 5 % IQR rule
  may reject cells on a shared host. Rejected cells are reported, not rerun.
- The host is shared; the load-average and SMT-sibling checks only bound,
  not eliminate, interference.
- `scaling_cur_freq` is sampled once per run outside the timed window; it
  cannot prove the frequency inside the window.
- `VmHWM` includes the whole process (population generation, std runtime)
  and is informational only.
