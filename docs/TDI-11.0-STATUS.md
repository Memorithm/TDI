# TDI-11.0 Status

Status: **FROZEN DESIGN CANDIDATE — merge and bootstrap validation required**

## Current state

TDI-11.x was introduced by PR #150 and is tracked by issue #151.

This stage now has a candidate preregistration and implementation gate that freeze the first controlled-world scientific contract before TDI-11.1 reference evaluator work.

Primary surfaces:

- `docs/TDI-11-PROGRAMME.md`;
- `docs/TDI-11.0-HALLUCINATION-DYNAMICS-SCOPE.md`;
- `docs/TDI-11.0-HALLUCINATION-DYNAMICS-PREREGISTRATION.md`;
- `docs/TDI-11.0-HALLUCINATION-DYNAMICS-PREREGISTRATION.gitblob`;
- `docs/TDI-11.0-IMPLEMENTATION-GATE.md`;
- `scripts/check-tdi11-bootstrap.sh`.

## Frozen candidate decisions

The candidate preregistration fixes the following before implementation evidence exists:

1. primary phenomenon is **unsupported generation under declared evidence**, not generic factuality;
2. primary scoring is exact and evaluator-owned, not LLM-judged;
3. primary outputs use the deterministic `ASSERT ...` / `ABSTAIN` grammar;
4. atomic labels preserve explicit support, derived support, hidden-but-unsupported truth, contradiction, nonexistence, indeterminacy, and abstention;
5. F1/F2/F3/F4 cover explicit support, derived support, insufficient visible evidence, and contradiction/override stress;
6. each family has Shallow/Intermediate/Deep strata, giving 12 primary cells;
7. H11-A is prospective and cannot be satisfied by a post-hoc detector;
8. B0/B1/B2/B3 separate fixed inference, static verification, risk gating, and adaptive recovery;
9. coverage and useful-task performance must be protected before unsupported-risk reduction can count as beneficial control;
10. hidden world truth is never a controller/verifier input.

## Authorization state

Until this candidate is merged and `scripts/check-tdi11-bootstrap.sh` passes on `main`, TDI-11.1 remains **not authorized**.

After those conditions hold, TDI-11.1 is authorized only for non-final controlled-world generator/oracle, parser/scorer, provenance, accounting, timing, fixtures, and development/validation evaluator implementation.

No final TDI-11 evaluation is authorized by TDI-11.0.

## Next engineering slice after freeze

The first TDI-11.1 implementation slice is:

`World schema -> visible/hidden partition -> deterministic closure -> ASSERT/ABSTAIN parser -> exact support label -> rejection/provenance record`

The slice is complete only when independently constructed fixtures cover every atomic label and every F1/F2/F3/F4 family without exposing hidden truth through the policy-facing representation.
