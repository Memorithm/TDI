# TDI-22.1 — Normative Episode Generator Contract

Status: **part of the TDI-22.1 freeze; non-final only**.

## Shared structural indices

`query_global_index = episode_index * 8 + query_index`.

The frozen bounds ensure `query_global_index <= 255`.

For G1/G2 candidates:

`candidate_geometry_index = (query_global_index + candidate_identity + 1) mod 256`.

Structural index arithmetic is checked and consumes no PRNG words.

## F1 / P1

For each query attempt consume, in order:

1. query `v.x,v.y,v.z` via `half_step`;
2. query `omega.x,omega.y,omega.z` via `half_step`;
3. query `Q.x,Q.y,Q.z` via `quarter_step`;
4. for candidate ids `0..15` ascending: `R` (3 half-step draws), local `M(P)` (3 half-step draws), supplied reference `P` (3 quarter-step draws).

Construct each torsor from `(R,M(P),P)` and choose the evaluator target by frozen direct torsor pairing. Only evaluator-target ambiguity is retryable.

## F2 / P2

For each query attempt consume query `(v,omega)` using six half-step draws, then for candidate ids `0..15` generate `(R,C)` using six half-step draws each. All positions are G0 origin, so `M(P)=C`. Target is the unique maximum of evaluator-owned `v.R + omega.C`.

## F3 / P3, P4, P5

F3 target identity is fixed before nuisance geometry:

1. generate query `(v,omega)` using six half-step draws;
2. generate content `(R,C)` for candidate ids `0..15` using six half-step draws per candidate;
3. choose the unique evaluator target using only `v.R + omega.C` and the frozen tie rule;
4. assign nuisance geometry only after target identity is fixed:
   - P3/G1: query `G1(query_global_index)`, candidate `G1(candidate_geometry_index)`;
   - P4/G2: query `G2(query_global_index)`, candidate `G2(candidate_geometry_index)`;
   - P5/G3: three quarter-step draws for query Q, then three quarter-step draws per candidate reference P in ascending identity;
5. reconstruct each stored local moment as `M(P)=C-P x R`.

Thus changing nuisance reference point cannot change content `(R,C)` or evaluator-owned target identity.

## Derived-bound checks

After every cross product, reconstructed moment, factorized query component or transported moment is formed, each component must be finite and `abs(component) <= 64`. Violation is fail-closed; it is never clamped or silently retried except where the protocol explicitly defines evaluator-target ambiguity retry.

## T1-specific tie

T1 reuses T0 scores after an episode exists. If T0 has a tied maximum under the frozen tolerance, emit `AmbiguousT1Top` for T1 only. Do not regenerate the episode and do not break the tie by identity.
