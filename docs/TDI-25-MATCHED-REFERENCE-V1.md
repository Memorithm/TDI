# TDI-25 matched reference v1

Status: prospective bounded Development/Validation software protocol for the
PR #689 prerequisite; specified before implementing or running this reference.
This is a new, explicitly versioned population. It does not relabel the legacy
Phase-B cases or retroactively qualify their arm-specific oracle records.
The four-family T6/C6 primary comparison and the upstream TDI-22/TDI-24 algebra
contracts are unchanged. No confirmatory or final execution is authorized.

## Identities and population

Population/materialisation contract: `tdi25-matched-reference-population-v1`.
Evaluator contract: `tdi25-matched-reference-evaluator-v1`.
Target contracts, respectively: `tdi25-matched-target-torsor-v1`,
`tdi25-matched-target-chiral-v1`, `tdi25-matched-target-mixed-v1`, and
`tdi25-matched-target-neutral-v1`.

Only the typed Development and Validation splits exist. Each family has at
most 64 seed blocks, numbered 0..63, with 2..64 cases admitted per run. Case
indices are 0..n-1; the registered local seed is `64 * block + case_index`.
The existing `mix_registered_seed(SeedDomain::from_split(split), family, seed)`
provides the domain separation. From that seed (zero replaced with one), the
fixed xorshift64* recurrence is `s ^= s >> 12; s ^= s << 25; s ^= s >> 27`,
then wrapping multiplication by `2685821657736338717`; each drawn scalar is
`((output >> 61) as i64 - 4) / 2`, in {-2,-1.5,-1,-0.5,0,0.5,1,1.5}.
Draw query[6], key[6], key_position[3], query_position[3], in that order.
Chiral and Neutral populations set both positions to zero after drawing.
No rejection sampling, result-conditioned retries, filtering or tuning exists.
Case generation is prefix-stable and independent of the requested run size.

## Equal inference surface and explicit materialisation

Both scoring functions receive the exact same immutable numeric input: six
query scalars, six key scalars and six geometry scalars. Family, split, block,
case identity and target remain evaluator-owned metadata, absent from the
scoring input type. No arm receives the other arm's result.

T6 interprets query as `(v, omega)` and key as `(R, M(P))`, with the explicit
stored reduction point P and query point Q. It calls the existing TDI-25
`torsor_arm_score`, which delegates to the TDI-22 factorized dual pairing.
C6 interprets the identical six query/key scalars as `(x+, x-)` and calls
`chiral_arm_score` with fixed `(alpha,beta,gamma)=(1,0,1)` for **every family**.
C6 receives the same positions but its declared upstream score does not use
those coordinates. No hidden geometric conversion or family-specific weight
selection is permitted. Neither arm combines the two representations.

## One target per case, shared by both arms

Write `D = query dot key`, `X = query^T J key`, and
`T = v dot R + omega dot (M(P) + (P-Q) cross R)`.
The evaluator computes the target before either candidate is scored, through
the upstream direct torsor pairing and primitive chiral observables, not by
reading or selecting candidate outputs.

| Family | Common scalar target |
|---|---|
| TorsorFavorable | T |
| ChiralFavorable | D + X |
| Mixed | (T + D + X) / 2 |
| Neutral | sum_i (query_i - key_i)^2 |

Mixed is a joint task target, not a hybrid candidate architecture. Neutral
uses a generic squared-distance target rather than choosing either arm's
native bilinear score. The favorable targets are favorable by construction;
passing them is software evidence, not a discovery of architectural quality.
Both arms use exactly the same finite scalar tolerance:
`abs(score-target) <= 1e-12 * (1 + max(abs(score), abs(target)))`.

## Budget and provenance

Both arms have zero trainable parameters, zero updates, one scalar score per
case, the same order, cases, precision (f64) and tolerance. The common visible
payload is 18 f64 values (144 bytes), excluding evaluator metadata, allocations
and retained reports. Score carriers remain six-dimensional. T6's source
carrier accounting includes its reference point and query geometry; equal
score width is not a claim of equal operation count, resident memory, runtime
or hardware cost. Those costs are not measured by this software slice.

Each emitted outcome seals source arm, split, family, block, case index,
common-target identity and a lossless canonical encoding of all input f64
bits plus versioned provenance. The encoding is an identity, not a
cryptographic attestation. Constructors for correctness bits remain private.
Legacy v1 evaluator adapters continue to seal `shared_target_contract=None`.
The new registry cannot upgrade these legacy outcomes. G6 remains secondary.

## Validation and interpretation

Public-API integration tests must execute both production scorers on every
family, both splits, and replicated full blocks; test both synthesis entry
points; retain per-block effects; and reject missing families, arm swaps,
family relabeling, stale targets, population mismatch and duplicates.
Private synthetic unit outcomes are not production evidence.

The statistical machinery retains its existing within-block and cluster
interval conventions. This deterministic bounded reference does not establish
random sampling, empirical interval coverage, final confirmation or
out-of-population validity. No weights, targets, budgets or population rules
may be changed after observing these runs without another explicit version.
No protected/final material, frozen historical holdout, TDI-24 implementation,
production routing or downstream performance claim is part of this change.
