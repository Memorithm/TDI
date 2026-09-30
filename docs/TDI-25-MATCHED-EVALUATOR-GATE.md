# TDI-25 matched primary evaluator gate

Status: **versioned matched reference implemented; legacy negative capability retained**.

This gate records the evaluator paths that are reachable from sealed,
non-test constructors on the default-branch TDI-25 API. It does not alter the
preregistered four-family protocol and does not create a scientific result.

## Legacy v1 capability matrix (unchanged)

| Family | T6 primary path | C6 primary path | Complete |
|---|---|---|---|
| Torsor-favorable | sealed torsor-transport evaluator | absent | no |
| Chiral-favorable | absent | sealed chiral-reflection evaluator | no |
| Mixed | sealed mixed torsor view; arm-specific target | sealed mixed chiral view; arm-specific target | no |
| Neutral | absent | absent | no |

The two Mixed paths are individually sealed, but T6 scores
`expected_torsor_score` while C6 scores `expected_chiral_score`. Those
arm-specific oracle fields are not one common target/scoring contract, so
Mixed also remains incomplete.

G6 has a sealed Neutral path, but remains a secondary attribution control. It
cannot satisfy either missing T6 or C6 primary cell.

## Legacy consequence

Legacy four-family primary synthesis must fail closed while its matrix is
incomplete. Green tests for synthesis over synthetic fixtures do not make the
production evaluator matrix complete.

The historical view `legacy_matched_family_paths` retains these four paths
and their absent shared targets. `require_complete_primary_matrix` now checks
the separately versioned matched reference below. It does not upgrade legacy
evaluator records; paired admission still refuses their absent target.

## Required protocol before adding matched paths

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

## Slice 28 paired-admission repair

A matching input digest is necessary but not a shared target. The v1
evaluator adapters now retain an explicitly absent common-target contract
in the integrity-bound revealed outcome. Neither changing the capability
matrix later nor relabeling a family can upgrade these legacy records.

The paired engine rejects them with `missing_common_target_contract`
before computing a primary difference or interval. Any later target must
be identical across arms and registered by the matched evaluator matrix.
Single-block and replicated family synthesis use the same check. G6's
separate, non-primary attribution summary remains available.

`tdi-ai/tests/tdi25_production_pairing.rs` exercises the public API against
genuine sealed Development/Validation evaluator runs. The library in
that integration target is compiled without `cfg(test)`. These are
negative admission regressions, not a complete production matrix or
a scientific result. Private synthetic unit fixtures remain restricted
to testing the statistical machinery. That admission-only repair did not resolve the missing paths. The subsequent
matched-reference protocol and implementation below provide those paths.
Campaign progress is recorded separately in TDI-25-CAMPAIGN-50.md.

## Versioned matched reference

`docs/TDI-25-MATCHED-REFERENCE-V1.md` was committed before implementation
and execution. It specifies the new population, common targets, explicit
carrier materialisation, identical inference visibility, budgets and
provenance required above. No legacy task or target is silently changed.

Matrix contract `tdi25-matched-evaluator-matrix-v2` registers eight paths:
both T6 and C6 for TorsorFavorable, ChiralFavorable, Mixed and Neutral.
`tdi25_eval::matched_reference::MatchedPrimaryRun::evaluate` runs both
fixed upstream scorers against one evaluator-owned target per input.
The production constructor has no caller-supplied correctness bit,
target, arbitrary callback or family relabeling surface.

`tdi-ai/tests/tdi25_matched_reference.rs` exercises both public synthesis
entry points, all families, both non-final splits and two full 64-case
blocks per family. Independent arithmetic checks pin actual scores and
common-target correctness. Negative tests preserve missing-family,
swapped-arm, duplicate, population-mismatch and mutation rejection.
Legacy production-pairing tests continue to require rejection.

This completes a bounded software path, not a confirmatory campaign,
an architecture-quality result, random-sampling guarantee or measured
operation/memory/runtime comparison. Favorable targets are favorable
by construction. The four-family requirement is unchanged.
