# TDI-25 slice 47 — qualified timing, attempt 2 (**non qualifiée**, raw pending)

Status: recorded run, **not qualified**. No qualified timing claim, no
performance claim. Slice 47 remains **timing non qualifié**. Attempts are
numbered and every attempt is recorded, whatever its outcome (no fishing).

| Item | Value |
| --- | --- |
| Attempt | 2 |
| Protocol | `tdi25-qualified-timing-environment-v2` (`docs/TDI-25-QUALIFIED-TIMING-ENVIRONMENT-V2.md`) |
| Operator | the user, on the Dell PowerEdge T430 |
| Machine (as reported) | CPU `Intel Xeon E5-2683 v4`, kernel `6.12.88`, BIOS `2.19.0`, rustc `1.97.1`, governor `performance`, `no_turbo=1`, `taskset 28`, load1 `0.97` |
| Raw results | **raw-pending**: `results/tdi25_s47_timing/attempt-2/results.jsonl` is **not yet in the repository**; the JSON lines were not available on the integration box at record time and are **not reconstructed or invented here** |
| `fingerprint.json` | **raw-pending**: not yet provided; it may be added later next to the raw results without altering them |

The values below are the operator-reported summary of the run. They are
traceability notes, not repository-verified evidence, until the raw
`results.jsonl` and `fingerprint.json` land in a follow-up PR that adds
`results/tdi25_s47_timing/attempt-2/` without altering them.

## Outcome: all 4 cells `non_qualifiee` (frequency)

Attempt 2 passed the V2 load precondition (load1 0.97 at start) but every
cell failed the V2 frequency rule: the APERF/MPERF effective frequency was
about 1.89 GHz versus the fixed `2100000 kHz` ±1 % check. IQR was
excellent (reported 1–11 ns across the cells).

- **Frequency**: APERF/MPERF measures the actual average core frequency
  over the measured window, relative to the invariant TSC/base frequency
  (2.1 GHz on this CPU). With turbo off and the `performance` governor the
  core on this machine runs at about 1.9 GHz (its P1 / guaranteed
  frequency), not at the 2.1 GHz base frequency the V2 rule required.
  The absolute `fixed_frequency_khz = 2100000` check therefore rejects a
  stable, unthrottled run. This motivates the V3 amendment, recorded
  separately; attempt 2 stays judged under V2.
- **IQR**: reported 1–11 ns, well within the 5 % rule in every cell.
- **Load**: load1 0.97 at start, within the ≤ 2 rule.

## Indicative numbers — **NON QUALIFIÉS**

Shown only for traceability, as reported by the operator. They are **not**
repository-verified, are **not** qualified timings, carry no performance
claim and must not be cited as slice-47 results.

| Cell | median (ns / pass of 64 pair scores) |
| --- | ---: |
| development / T6 | 3680 |
| validation / T6 | 3688 |
| development / C6 | 3824 |
| validation / C6 | 3822 |

## Exact operator commands to supply the raw evidence

On the T430, locate the attempt-2 run directory and print the raw files
verbatim (do not edit them), then provide the output for integration:

```bash
ls -dt /home/tdi/tdi25-s47-timing-runs/tdi25-s47-timing-*-attempt-2
cat /home/tdi/tdi25-s47-timing-runs/tdi25-s47-timing-*-attempt-2/results.jsonl
cat /home/tdi/tdi25-s47-timing-runs/tdi25-s47-timing-*-attempt-2/fingerprint.json
cat /home/tdi/tdi25-s47-timing-runs/tdi25-s47-timing-*-attempt-2/run-id.txt
```

Integration (follow-up PR, only from those verbatim lines): create
`results/tdi25_s47_timing/attempt-2/results.jsonl` byte-for-byte, add
`fingerprint.json` if present, add a `build-provenance.txt` in the
attempt-1 format and a `SHA256SUMS`, and update this record to replace
**raw-pending** with the run id, commit/tree, binary SHA-256 and the
per-cell table read from the raw lines. Until then this attempt is
documented as **recorded-but-raw-pending**.

## Recorded degeneracies

- Attempt 2 is judged under V2 and is not re-judged under V3; the cells
  are not rerun under V2.
- The reported medians above are operator-reported only; no `results.jsonl`
  line, quartile, swap counter, SMT-sibling fraction or per-cell effective
  frequency is asserted here because the raw lines are pending.
- `fingerprint.json` is pending; no machine fingerprint beyond the
  operator-reported summary above is recorded so far.
