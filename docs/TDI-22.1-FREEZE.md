# TDI-22.1 — Frozen Non-Final Evaluator Contract

Status: **frozen candidate; authorises only bounded non-final TDI-22.2 development/validation after merge and green CI**.

This file resolves the execution-blocking fields declared by `docs/TDI-22.1-PREREGISTRATION.md`. It does not authorize final/confirmatory execution, learned geometry, FLAT-ATTENTION changes, or hardware claims.

## 1. Arm set

The first TDI-22.2 evaluator implements exactly **T0, T1, T3 and T4**. **T2 is explicitly deferred.** The critical torsor-specific contrast remains `T3 - T4`.

## 2. Primary cells

1. `P1 = F1 × G3_SUPPLIED`.
2. `P2 = F2 × G0_ORIGIN`.
3. `P3 = F3 × G1_LINEAR`.
4. `P4 = F3 × G2_HELIX`.
5. `P5 = F3 × G3_SUPPLIED`.

`F1 × G0_ORIGIN` is secondary only. No secondary cell may replace P1–P5.

## 3. Population bounds

Per primary cell:

- development episodes: `16`;
- validation episodes: `32`;
- queries per episode: `8`;
- candidates per query: `16`;
- exactly one evaluator-owned target per query;
- maximum structural geometry index: `255`.

Primary query counts per arm are therefore 640 development and 1280 validation.

## 4. Numeric domain

Reference arithmetic is deterministic IEEE-754 binary64 with fixed iteration order.

- generated content components are exact multiples of `0.5` in `[-8,+8]`;
- evaluator-supplied G3 coordinates are exact quarter-integers in `[-2,+2]`;
- every input and every derived vector component must have absolute value `<= 64`.

No saturation or clamping is permitted. Any actual bound violation is a typed rejection.

## 5. Frozen geometry

`G0_ORIGIN`: `P(i)=(0,0,0)`.

`G1_LINEAR`: for `0 <= i <= 255`, `P(i)=(i/64,0,0)`.

`G2_HELIX`: let `k=i mod 8`, use exact XY lookup
`[(1,0),(1,1),(0,1),(-1,1),(-1,0),(-1,-1),(0,-1),(1,-1)]`, and `z=i/64`.

`G3_SUPPLIED`: evaluator-generated exact quarter-integers in `[-2,+2]`.

All divisions by 64 are exact in binary64. No transcendental function is used.

## 6. Split domains and generation

Development domain: `0x5444493232444556` (`TDI22DEV`).
Validation domain: `0x544449323256414c` (`TDI22VAL`).

Exact stream packing, SplitMix64 transition, scalar mapping, retry semantics and family-specific draw ordering are frozen in `TDI-22.1-SEED-CONTRACT.md` and `TDI-22.1-GENERATOR-CONTRACT.md`.

## 7. Targets

F1: unique maximum of the frozen direct torsor pairing.

F2: unique maximum of evaluator-owned `v.R + omega.C` at G0.

F3: unique maximum of evaluator-owned `v.R + omega.C` **before nuisance geometry is assigned**. Local `M(P)` is then reconstructed from frozen content `(R,C)` according to the generator contract.

Evaluator-target ambiguity may retry at most 32 times using the same forward-only episode stream. Exhaustion is `RetryBudgetExhausted`.

## 8. Floating tolerance

`tol(a,b) = 512 * EPSILON * max(1, abs(a), abs(b))`.

Scores are tied iff `abs(a-b) <= tol(a,b)`. Value reconstruction uses the same component-wise tolerance. This tolerance may not be widened after observing outcomes.

## 9. T1 value rule and tie policy

T1 uses T0 scoring and a one-hot top-1 readout. If T0 has one unique maximum under the frozen tolerance, that candidate receives weight 1 and all others receive 0; the returned value is its `(R,C)`.

If T0 has a tied maximum, the T1 record is **fail-closed rejected** as `AmbiguousT1Top`. The evaluator must not choose the lowest identity, perturb scores, regenerate the episode, or silently break the tie. Other arm records for the same evaluator-owned query remain valid when otherwise admissible. This rule is frozen specifically so F1 target uniqueness under T3 does not imply an unstated T0/T1 tie-break.

## 10. Resource accounting

Every arm record exposes all preregistered resource fields separately:

- `query_bits`;
- `key_bits`;
- `value_bits`;
- `position_bits`;
- `dynamic_state_bits` (including candidate-retained cache/state not already represented by the per-record key/value/position payload fields);
- `static_parameter_bits` (constants/tables required by the candidate semantics);
- `temporary_slots`;
- `add_count`;
- `mul_count`;
- `cross_count`;
- `dot_lane_count`;
- `comparison_count`.

No field may be omitted because its value is zero. T3 query-side `Q x omega` is counted once per query, not per candidate. Evaluator-oracle work is recorded separately from candidate work and is not charged to an arm.

## 11. Provenance

Schema version is exactly `tdi22-eval-record-v1`. Exact field order, delimiters, enum tokens, booleans, rejection tokens and vector/scalar encodings are frozen by `TDI-22.1-RECORD-CONTRACT.md`.

## 12. Authorization

After this freeze is merged and all TDI-22 gates pass on `main`, agents may implement and execute bounded **non-final** TDI-22.2 development/validation for T0/T1/T3/T4.

Still forbidden: T2 execution, learned geometry, final/confirmatory material, result-conditioned protocol tuning, FLAT-ATTENTION runtime changes, and asymptotic/KV/latency/bandwidth/energy/hardware superiority claims.
