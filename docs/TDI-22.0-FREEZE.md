# TDI-22.0 — Stage-0 Freeze Record

Status: **freeze candidate**. This record freezes TDI-22.0 only after it is merged to `main` with all required CI green on its exact head SHA.

## Frozen Stage-0 payload

The following Git blob identities define the exact Stage-0 scientific/software payload currently merged on `main`:

- `docs/TDI-22-PROGRAMME.md` → `6649b39552aa04128fcc39980ffcbb8743be7d1a`
- `docs/TDI-22.0-SCOPE.md` → `47143b190b70cd59b1972c6fbb594ae4456c6e2e`
- `tdi-ai/src/tdi22_torsor.rs` → `419052f499ef83c1909248e12263f5ca846a3b96`
- `tdi-ai/tests/tdi22_torsor_properties.rs` → `42f3246223b39f8fbc69492b4350914c02d257ec`
- `scripts/check-tdi22-bootstrap.sh` → `7e535217eef2b01732a949336a2e347c7617c932`
- `.github/workflows/tdi22-torsor-bootstrap.yml` → `6c53e2fee3023325b649dea29ee0cb7a9c46d918`

These blob identities are content-addressed. Any later change to one of these files is a new scientific/software revision and must not be represented as the frozen TDI-22.0 payload.

## Frozen mathematical convention

TDI-22.0 freezes only the following three-dimensional convention:

```text
M(Q) = M(P) + (P - Q) x R
C    = M(P) + P x R
M(Q) = C - Q x R

s_direct     = v . R + omega . M(Q)
s_factorized = (v + Q x omega) . R + omega . C
```

The direct and factorized pairings are required to agree up to declared binary64 roundoff. This is an algebraic/implementation property only.

## Frozen interpretation boundaries

Stage 0 does **not** establish that torsor attention improves task quality, memory use, computational complexity, latency, bandwidth, throughput, training efficiency, model quality, or generalization. It does not establish novelty.

The future TDI-22 comparison ladder must preserve the matched six-component non-torsor control T4. A T3-vs-T0 result alone cannot be interpreted as evidence for a torsor-specific effect.

## Authorization produced by this freeze

After this record is merged on `main`, its integrity check passes, and the TDI ecosystem roadmap is reread, TDI-22.1 may **preregister** deterministic non-final tasks, matched budgets, metrics, geometry arms, development/validation splits, rejection rules, and the T0–T4 comparison contract.

This freeze does not authorize TDI-22.2+ evaluation, a final/confirmatory population, learned model execution, FLAT-ATTENTION integration, or any hardware-performance claim.
