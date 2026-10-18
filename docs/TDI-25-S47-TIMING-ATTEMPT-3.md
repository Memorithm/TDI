# TDI-25 slice 47 — qualified timing, attempt 3 (**non qualifiée**, V2 stale binary, transcription-of-operator-output)

Status: recorded run, **not qualified**. No qualified timing claim, no
performance claim. Slice 47 remains **timing non qualifié** overall, pending
the partially qualified attempt 4 record. Attempts are numbered and every
attempt is recorded, whatever its outcome (no fishing).

| Item | Value |
| --- | --- |
| Attempt | 3 |
| Run id | `tdi25-s47-timing-7fc1a25bb4b8-attempt-3` |
| Operator | the user, on the Dell PowerEdge T430 |
| Protocol judged under | `tdi25-qualified-timing-environment-v2` (stale binary, run before the V3 checkout; **not** re-judged under V3) |
| Commit / tree | `7fc1a25bb4b86b6880e521ef270bce6b0b77ebff` / `96a1e5ad8817925481754a05c2a56ed6b2794731` (tree derived from the named commit in this repository) |
| Binary SHA-256 | **pending**: not provided in the operator transcript supplied for this record; not invented here |
| Raw results | **transcription-of-operator-output, not raw**: `results/tdi25_s47_timing/attempt-3/transcription-of-operator-output.txt`; the 4 JSON lines were not available byte-exact on the integration box and are **not reconstructed or invented here**, so no `results.jsonl` is created for this attempt |
| `fingerprint.json` | **pending**: not yet provided by the user; it may be added later next to the transcription without altering it |
| Environment attested (operator-reported) | governor `performance`, `no_turbo=1` attested; load1 `1.44` |

## Outcome: all 4 cells `non_qualifiee`

All 4 cells were rejected on the V2 `frequency` rule, and one cell
additionally on `smt_sibling_busy` (the transcript supplied for this
record does not identify which cell; that detail is not invented here).

- **Frequency**: judged under the stale V2 binary, i.e. the absolute
  APERF/MPERF check against the fixed 2 100 000 kHz that V3 replaced.
  Attempt 3 is not re-judged under V3.
- **Load**: operator-reported load1 1.44, within the ≤ 2 rule.
- **SMT**: one additional `smt_sibling_busy` rejection (cell not
  identified in the supplied transcript).

## Indicative numbers — **NON QUALIFIÉS**

Shown only for traceability, as reported by the operator (development
then validation, per arm). They are **not** repository-verified raw
evidence, are **not** qualified timings, carry no performance claim and
must not be cited as slice-47 results.

| Cell | median (ns / pass of 64 pair scores) |
| --- | ---: |
| development / T6 | 3985 |
| validation / T6 | 3692 |
| development / C6 | 3827 |
| validation / C6 | 3826 |

## Recorded degeneracies

- Attempt 3 consumed attempt number 3 but was run with the stale V2
  binary before the V3 checkout; it stays judged under V2 and is not
  rerun or re-judged under V3.
- The supplied transcript does not identify which cell carried the
  additional `smt_sibling_busy` rejection, nor per-cell quartiles, swap
  counters, SMT fractions or effective frequencies; none are asserted
  here.
- `fingerprint.json` and the byte-exact `results.jsonl` are pending;
  the file in `results/tdi25_s47_timing/attempt-3/` is explicitly a
  transcription-of-operator-output, not raw evidence.
