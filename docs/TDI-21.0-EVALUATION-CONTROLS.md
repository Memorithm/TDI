# TDI-21.0 — Shared development scoring and competence controls

Status: **development-only; not a TDI-21.1 freeze or a TDI-21.2 comparative experiment**.

This continues the causal B2/B3 references merged by #237. Source inspected: `bfc4a5dff418e6b370594b94bf7ac86dbe898c21`. Candidate storage/routing algorithms, historical counterexamples, frozen series and protected holdouts are unchanged.

## Why this increment is necessary

The original fixed example computed `correct_queries == stats.replies`. A generic future mechanism omitting replies could therefore improve its denominator; a zero-reply trace could make that expression vacuously true. The old example ALSO had hard-coded fixture-count assertions, which already prevented a wholly silent run from passing that script. This increment replaces the fragile generic expression rather than claiming those existing assertions were absent.

The new `ScoreCard` counts queries from the evaluator's expectation, not from the candidate output or its self-reported counters. Empty/no-query evaluation cannot be classified as success. A missing output entry or an extra entry is a trace-length error. `Quiet` on a query counts as an omitted reply, and execution errors cannot remove a query from the denominator.

## Disjoint outcomes

| Expected | Observed | Category |
| --- | --- | --- |
| Available value | Same value | ExactValue |
| Absent fact | Explicit Miss | CorrectAbsence |
| Available value | Different value | WrongValue |
| Absent fact | Any value, including zero | FalseHit |
| Available value, including zero | Miss | ForgottenValue |
| Query | Quiet | OmittedReply |
| Query | Execution error | QueryError |
| Non-query | Quiet | CorrectQuiet |
| Non-query | Reply | SpuriousReply |
| Non-query | Execution error | QuietError |

The first seven categories partition all expected queries. Exact accuracy is `(ExactValue + CorrectAbsence) / expected_queries`. The API returns an integer pair, not a rounded percentage. `all_correct` additionally requires a nonzero query denominator and no spurious reply or non-query error. Private counts and checked additions preserve the partition invariant.

Correct absence is not the same as abstaining on a known fact. A no-memory control must not earn credit for forgetting a stored false/zero value. The report keeps value errors, false hits, missing values and missing replies separate.

## Independent oracle and bounded workload

`DevelopmentEpisode::new` validates public events and computes expected answers by scanning original event prefixes in reverse. It does not call the candidate's hash, clause evaluator, memory table or output. Future writes cannot answer earlier queries. Marker-inhibited writes do not supersede older accepted facts. Conjunction requires both operands; a known zero with an absent other operand still yields Miss under the existing declared task semantics.

The candidate runner creates a fresh `BooleanStream` and calls `step(event)` once per event. Oracle expectations never enter that signature. The dictionary control also processes current events only and is not allowed to copy the oracle trace. Its ordered-map implementation is separate from the oracle's reverse-prefix scan.

Development admission bounds are 16,384 events, 4,096 accepted writes (overrides count too), valid payload widths 1..64 and at most 1,000,000 oracle prefix inspections. Invalid events are rejected even when their marker would inhibit a write. These are software safety limits, not a selected scientific task grid. The oracle work counter is evaluator-owned and excluded from candidate work.

## Competence controls, not fake attention baselines

`NoMemory` ignores writes and emits Miss for each query. It can correctly reject an actually absent fact but cannot recall available values.

`ExactDictionary` stores accepted original identifiers and their latest payloads in an ordered map. The episode bound limits the number of entries; it never reads the precomputed answers. This control establishes that the public task is solvable by an ordinary dictionary. It is not B0, B1 or an attention replacement result.

The dictionary reports logical map reads/writes, successful conjunctions, stored facts and the identity-plus-payload lower bound of 128 bits per stored fact. This lower bound deliberately excludes tree nodes, links, allocator and inline object state. Map comparison counts and physical allocation costs are not measured. The control's resources are NOT declared matched to B2/B3. The no-memory control's zero entry payload does not mean zero process memory.

## Common Boolean memory-substrate envelope

For S entries, the existing reference substrates cost:

- B2: `129 * S` semantic bits;
- B3: `129 * S + S / 2` semantic bits, for positive even S.

Each entry has a 64-bit identifier, a stored 64-bit payload and an occupancy bit. B3 additionally needs one round-robin bit per two-entry bucket. A smaller logical payload width does not magically compress the allocated fields.

For a common ceiling B, `fit_slots` chooses `min(floor(B/129), 4096)` for B2 and `2 * min(floor(B/259), 2048)` for B3. Zero feasible entries is an explicit error. Tests cross-check the fit against independent enumeration and check the maximum machine integer without overflow.

At 516 bits, four B2 entries fit; four B3 entries require 518 bits, so only two fit. At 518 bits, both allow four entries, with two unused substrate bits in B2. The evaluator rejects over-budget configurations before constructing a candidate, and also checks payload-width and event-budget compatibility.

This is a COMMON SUBSTRATE CEILING, not equal total model/training/CPU/allocator memory or a claim of matched end-to-end resource consumption. Existing physical object/buffer footprints and separate work counters remain in the output.

## Published development fixtures

The example preserves delay, pair and saturation fixtures and adds mixed absent/zero/override/inhibited-write cases plus the already established B2-favouring replacement counterexample. All four mechanisms receive the same public events. The common B2/B3 substrate ceiling is 518 bits. No training occurs.

These are source-defined regression expectations, to be checked by actual execution rather than presented as independent scientific observations:

| Fixture | Queries | B2 correct | B3 correct | NoMemory correct | ExactDictionary correct |
| --- | ---: | ---: | ---: | ---: | ---: |
| delay | 1 | 1 | 1 | 0 | 1 |
| pair | 2 | 1 | 2 | 0 | 2 |
| saturated | 3 | 1 | 2 | 0 | 3 |
| mixed | 7 | 7 | 7 | 3 | 7 |
| reverse | 1 | 1 | 0 | 0 | 1 |

The dictionary solving these tasks is a control against overstating architectural progress. The reverse fixture keeps B3 non-dominance visible. No uniform improvement is inferred from the pair fixture. No aggregate benchmark score across this hand-picked fixture set is claimed.

## Validation and provenance

The ten evaluation contract tests include all outcome categories, no-query/silent/truncated cases, non-query errors, independent prefix semantics, memory envelopes, capacity failures in both directions, resource admission and exhaustive length-three streams over eight events. Of 8^3 = 512 strings, 5^3 = 125 have no query and are rejected as no-query episodes; 387 are scored. Both Boolean arms and the exact control are cross-checked on this tiny no-eviction domain. This is neither a held-out dataset nor language-model evaluation.

Run from a clean checkout:

```bash
bash scripts/check-tdi21-development.sh
```

The existing script runs old and new regressions in debug and release, records its actual source SHA, Rust metadata and hashes of the evaluator/example/manifests, and compares two complete example outputs byte-for-byte. It checks that tests did not change the source checkout. The source SHA and hashed example bind these fixed input definitions; arbitrary external input provenance would require an additional input-artifact contract. Doctests run in the existing all-feature test gate. CI logs, not source assertions alone, establish executed outcomes.

## Reuse and next work

The shared query denominator, disjoint outcome ledger and explicit resource-envelope check are reusable evaluation ideas for SciRust, BooleanLab and later hybrid attention. Their current API is TDI-event-specific and stays in TDI; promotion requires a separately reviewed versioned interface. No runtime code was copied into FLAT-ATTENTION or ElasticXxx.

The next architectural work remains isolated B0/B1 attention references on common causal inputs, learned/synthesized Boolean encoding/routing and explicit total training/tuning/compute envelopes. TDI-21.1 task/split/decision freezes remain unresolved; no final or protected material is created or accessed by this development increment.
