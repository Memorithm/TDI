# TDI-22.0 → TDI-22.1 Implementation Gate

TDI-22.1 preregistration work may begin only after all conditions below are true on `main`:

1. TDI-22.0 bootstrap is merged.
2. `docs/TDI-22.0-FREEZE.md` is merged by a later reviewed change.
3. `scripts/check-tdi22-freeze.sh` passes on `main`.
4. The TDI ecosystem roadmap is reread after the freeze merge.
5. No TDI-22.2+ evaluator, result payload, final dataset, final seed list, FLAT integration, or hardware benchmark exists before TDI-22.1 freezes its comparison protocol.

## Frozen TDI-22.0 invariants

TDI-22.1 must preserve:

- three-dimensional torsor/twist semantics under the declared sign convention;
- `M(Q) = M(P) + (P - Q) x R`;
- `C = M(P) + P x R` and `M(Q) = C - Q x R`;
- direct pairing `v.R + omega.M(Q)`;
- factorized pairing `(v + Q x omega).R + omega.C`;
- deterministic equality of direct/factorized forms within declared binary64 roundoff;
- transport composition/round-trip checks;
- reduction-point invariants used by Stage 0;
- fail-closed rejection of non-finite input/arithmetic;
- explicit separation between an algebraic identity and evidence of attention utility;
- T4 matched six-component non-torsor control as the critical control for torsor-specific effects;
- TDI ownership of scientific semantics and FLAT-ATTENTION ownership of downstream execution semantics.

## TDI-22.1 authorised work

After this gate is satisfied, agents may prepare and freeze a **non-final preregistration only** covering:

- deterministic bounded task families;
- T0 vector reference, T1 torsor-value, T2 hybrid, T3 full-torsor, and T4 matched six-component non-torsor arms;
- exact parameter/state/cache/resource accounting;
- exact geometry families to be compared;
- normalization policy;
- deterministic development/validation split derivation;
- metrics and ordered decision rules;
- rejection/error semantics;
- provenance schema;
- explicit implementation/evaluation stage boundaries.

Small software fixtures needed to validate serialization, accounting, or contract consistency may be added only if they do not execute the scientific arm comparison.

## Still forbidden after TDI-22.1 preregistration authorization

This gate does **not** authorize:

- running T0–T4 comparative evidence as a scientific result;
- tuning arm parameters against a hidden/final population;
- selecting a favorable geometry after seeing confirmatory results;
- dropping T4 because T3 beats T0;
- claiming a torsor-specific effect without the frozen T3-vs-T4 comparison;
- claiming lower KV memory or lower complexity without separate exact accounting;
- claiming latency, throughput, bandwidth, or energy improvement without downstream device evidence;
- modifying FLAT-ATTENTION production routing or Kernel IR;
- any final/confirmatory evaluation before a later explicit authorization.

## Next gate

TDI-22.2 implementation/evaluation remains blocked until TDI-22.1 itself is merged, content-addressed/frozen, and its integrity gate passes. No later stage may weaken TDI-22.1 controls in response to observed results.
