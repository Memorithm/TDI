# TDI-25 slice 47 — qualified timing, attempt 4 (**partiellement qualifiée**, transcription-of-operator-output)

Status: recorded run, **first partially-qualified V3 run**. Three cells
are `qualifiee`; one cell is `non_qualifiee` solely on SMT. This is not
a complete qualified timing result and carries no general performance
claim. Slice 47 is therefore recorded as timing partially qualified on
this machine only (see the bounded conclusion below). Attempts are
numbered and every attempt is recorded, whatever its outcome (no
fishing).

| Item | Value |
| --- | --- |
| Attempt | 4 |
| Run id | `tdi25-s47-timing-0a2eb708f85c-attempt-4` |
| Operator | the user, on the Dell PowerEdge T430 |
| Protocol | `tdi25-qualified-timing-environment-v3` (`docs/TDI-25-QUALIFIED-TIMING-ENVIRONMENT-V3.md`) |
| Commit / tree | `0a2eb708f85cf0569c9e59d6e7a34520c8e0ccfe` / `e435f31fb92cc3967399a090a0368770dc765d15` (tree derived from the named commit in this repository) |
| Binary SHA-256 | `e9e7b04c...` (operator-provided prefix only; full value **pending**, not invented here) |
| Raw results | **transcription-of-operator-output, not raw**: `results/tdi25_s47_timing/attempt-4/transcription-of-operator-output.txt`; the 4 JSON lines were not available byte-exact on the integration box and are **not reconstructed or invented here**, so no `results.jsonl` is created for this attempt |
| `fingerprint.json` | **pending**: not yet provided by the user; it may be added later next to the transcription without altering it |
| Cross-cell frequency | `frequency_median_effective_khz` = 1890544 (operator-reported, V3 cross-cell median) |

## Outcome: 3 cells `qualifiee`, 1 cell `non_qualifiee` (SMT only)

| Cell | Status | Rejection reasons | median (ns / pass of 64 pair scores) |
| --- | --- | --- | ---: |
| development / T6 | `qualifiee` | — | 3618 |
| validation / T6 | `qualifiee` | — | 3593 |
| validation / C6 | `qualifiee` | — | 3826 |
| development / C6 | `non_qualifiee` | `smt_sibling_busy` (5 %) | 3826 (indicative only, **NON QUALIFIÉ** for this cell) |

- **SMT**: development / C6 was rejected solely because the SMT sibling
  busy fraction was 5 %, above the frozen ≤ 1 % rule. Its 3826 ns
  median is indicative only.
- **Frequency**: the run is judged under V3 (cross-cell APERF/MPERF
  median 1890544 kHz, ±1 % consistency and ≥ 1 785 000 kHz floor);
  no per-cell effective frequency is asserted here because the raw
  lines are pending.

## Bounded conclusion — qualified cells only

On the validation split, both arms are `qualifiee`: T6 3593 ns versus
C6 3826 ns per 64-case pass, i.e. T6 about 3.6 µs and C6 about
3.83 µs, a difference of about +6 % for C6 relative to T6
((3826 − 3593) / 3593 ≈ 6.5 %, reported as about +6 %). T6 is also
`qualifiee` on development (3618 ns).

Scope: **this T430 only**, smoke budget of 64 cases, slice-47
reference-cost harness only. This is not a production, GPU or general
performance claim, and the rejected development / C6 cell must not be
cited as qualified.

## Recorded degeneracies

- Attempt 4 is partially qualified, not a complete 4-cell qualified
  run; results are never selected across attempts and the rejected
  cell is not rerun as part of this attempt.
- Per-cell quartiles, swap counters, per-cell SMT fractions (other
  than the reported 5 % for development / C6), per-cell effective
  frequencies and ratios to the median are not asserted here because
  the byte-exact raw lines are pending.
- `fingerprint.json` and the full binary SHA-256 are pending; the
  file in `results/tdi25_s47_timing/attempt-4/` is explicitly a
  transcription-of-operator-output, not raw evidence.
