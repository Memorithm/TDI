# TDI-25 matched primary evaluator gate

Status: **negative capability evidence / implementation prerequisite**.

This gate records the evaluator paths that are reachable from sealed,
non-test constructors on the default-branch TDI-25 API. It does not alter the
preregistered four-family protocol and does not create a scientific result.

## Current exact capability matrix

| Family | T6 primary path | C6 primary path | Complete |
|---|---|---|---|
| Torsor-favorable | sealed torsor-transport evaluator | absent | no |
| Chiral-favorable | absent | sealed chiral-reflection evaluator | no |
| Mixed | sealed mixed torsor view | sealed mixed chiral view | yes |
| Neutral | absent | absent | no |

G6 has a sealed Neutral path, but remains a secondary attribution control. It
cannot satisfy either missing T6 or C6 primary cell.

## Consequence

Four-family primary synthesis must fail closed while this matrix is
incomplete. Green tests for synthesis over synthetic fixtures do not make the
production evaluator matrix complete.

The implementation in
`experimental::tdi25_matched_matrix::require_complete_primary_matrix`
returns the first missing primary path deterministically. It admits only the
four paths already reachable from sealed production evaluators.

## Required next freeze

Before filling any missing cell, a later versioned change must define:

1. one inference-visible population per family that both T6 and C6 consume;
2. one common target/scoring contract per family;
3. explicit carrier materialisation with no silent conversion;
4. identical metadata visibility and bounded budgets for both arms;
5. canonical split, family, seed block, case and input identities;
6. negative tests for arm swaps, family relabeling, population mismatch,
   asymmetric metadata, stale provenance and synthetic production outcomes.

Existing TDI-22/TDI-24 carrier and scoring semantics remain unchanged.
Development/Validation only. Protected/final and holdout surfaces remain
forbidden. No accuracy, superiority, novelty or performance claim is made.

Tracking: [#691](https://github.com/Memorithm/TDI/issues/691) and blocked
candidate [#689](https://github.com/Memorithm/TDI/pull/689).
