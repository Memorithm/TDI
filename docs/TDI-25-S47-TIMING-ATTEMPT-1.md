# TDI-25 slice 47 — qualified timing, attempt 1 (**non qualifiée**)

Status: recorded run, **not qualified**. No qualified timing claim, no
performance claim. Slice 47 remains **timing non qualifié**. Attempts are
numbered and every attempt is recorded, whatever its outcome (no fishing).

| Item | Value |
| --- | --- |
| Attempt | 1 |
| Run id | `tdi25-s47-timing-8bd8aea` |
| Operator | the user, on the Dell PowerEdge T430 (2026-10-10, ~23:18 Europe/Paris) |
| Protocol | `tdi25-qualified-timing-environment-v1` (`docs/TDI-25-QUALIFIED-TIMING-ENVIRONMENT-V1.md`) |
| Commit / tree | `8bd8aeacec96f05f64508a509faa2b45b147eb6a` / `daffdc043bb2af3117716818d8abc009a0ad23d5` |
| Binary SHA-256 | `c87d237f79891f872ae4e8443fd3da467730a08412348eb556fa340751be1a89` |
| Toolchain | rustc 1.97.1 (8bab26f4f 2026-07-14), LLVM 22.1.6 |
| Restore | `restored governor=schedutil/schedutil no_turbo=0` (verified by the script) |
| Raw results | `results/tdi25_s47_timing/attempt-1/results.jsonl` (SHA-256 `ab2db3cd203dcc29054616127fff7475ac73967f918deab8d60180f651cf2f40`) |
| Build provenance | `results/tdi25_s47_timing/attempt-1/build-provenance.txt` |
| `fingerprint.json` | **pending**: not yet provided by the user; it may be added later next to the raw results without altering them |

The harness attested CPU `Intel(R) Xeon(R) CPU E5-2683 v4 @ 2.10GHz`,
kernel `6.12.88+deb13-amd64`, BIOS 2.19.0, `Cpus_allowed_list` = 28,
governor `performance` on CPUs 28 and 60, `no_turbo` = 1. Swap counters were
unchanged in every cell.

## Outcome: all 4 cells `non_qualifiee`

| Cell | Rejection reasons | load1 start / end | SMT sibling busy | IQR / median | `scaling_cur_freq` (kHz) |
| --- | --- | --- | ---: | ---: | ---: |
| development / T6 | load_start, load_end, frequency | 2.41 / 2.41 | 0.0000 | 0.51 % | 2 048 054 |
| development / C6 | load_start, load_end, frequency | 2.41 / 2.41 | 0.0000 | 0.39 % | 1 567 895 |
| validation / T6 | load_start, load_end, frequency | 2.22 / 2.22 | 0.0100 | 0.45 % | 2 069 770 |
| validation / C6 | load_start, load_end, frequency | 2.22 / 2.22 | 0.0000 | 0.39 % | 1 986 263 |

- **Load**: the 1-min load average exceeded the frozen threshold 2 at the
  start and the end of every cell (shared host).
- **Frequency**: every post-run `scaling_cur_freq` reading was outside ±1 %
  of the fixed 2 100 000 kHz, although governor `performance` and
  `no_turbo = 1` were set and attested. Under `intel_pstate`,
  `scaling_cur_freq` is a kernel estimate, not a measurement of the timed
  window; the reading is not a reliable frequency check. This motivates the
  V2 amendment (APERF/MPERF), recorded separately; attempt 1 stays judged
  under V1.
- **IQR**: within the 5 % rule in every cell.

## Indicative numbers — **NON QUALIFIÉS**

Shown only for traceability. They are **not** qualified timings, carry no
performance claim and must not be cited as slice-47 results.

| Cell | median (ns / pass of 64 pair scores) | min | Q1 | Q3 | IQR |
| --- | ---: | ---: | ---: | ---: | ---: |
| development / T6 | 3739 | 3686 | 3730 | 3749 | 19 |
| development / C6 | 3863 | 3831 | 3858 | 3873 | 15 |
| validation / T6 | 3737 | 3702 | 3728 | 3745 | 17 |
| validation / C6 | 3883 | 3824 | 3877 | 3892 | 15 |

Peak RSS (`VmHWM`, informational only): 2248–2368 kB.

## Recorded degeneracies

- Load average and frequency rejections are environmental; they are not
  relaxed after the fact and the cells are not rerun under V1.
- `load1` is sampled per cell around microsecond-scale cells, so start and
  end readings are identical in practice.
- `fingerprint.json` is pending; the attestation embedded in every
  `results.jsonl` line is the only machine fingerprint recorded so far.
