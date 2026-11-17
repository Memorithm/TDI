# TDI-22 — Experimental event-driven torsor aggregation

Status: **Development-only software semantics**. No TDI-22 frozen payload,
preregistration, protected/final population, model runtime or ADA/FLAT adoption
is changed by this extension.

## Problem

Test whether a sparse stream of insert/replace/remove events can maintain a
torsor-like aggregate without rescanning all live contributors on every query.

Each source `i` has a stable identity, a finite signed weight `w_i`, and
a frozen TDI-22 `T_i(P_i) = (R_i, M_i(P_i))`. Define

```text
C_i = M_i(P_i) + P_i × R_i
R_total = Σ_i w_i R_i
C_total = Σ_i w_i C_i
M_total(Q) = C_total − Q × R_total
s(Q, twist) = (v + Q × ω) · R_total + ω · C_total
```

The stored pair `(R_total,C_total)` is **not** a probability-normalized
attention output, KV cache or proof that torsors suffice for a learned task.
An event replaces just one contribution, or removes it by source identity.
The total is updated by subtracting the old stored weighted pair and adding
the new one; both finite checks complete before any mutation. Missing deletes,
NaN, infinite weights, and derived overflow fail closed.

## Executable contribution

- `tdi-ai/src/tdi22_event_aggregate.rs`, opt-in experimental API.
- Independent full-replay oracle test of signed weighted dual pairings after
  each event, plus reduction-point invariants, common-origin translation,
  transactional failure, and idempotent updates.
- Per event: constant-size vector arithmetic plus O(log K) keyed lookup,
  with O(K) stored contributions for K live identities; query score uses
  six cached scalar components. This is *reference algorithm* accounting,
  not measured energy, bandwidth, throughput, or an end-to-end model claim.

## Limits and follow-ups

Floating-point event order changes rounding. Comparisons use declared binary64
tolerances; repeated cancellation may accumulate drift. A future independently
registered experiment should compare full recomputation, compensated updates
and periodic rebasing under matched workloads, with TDI-28 accounting only
through an explicit versioned adapter. No existing TDI-25/TDI-28 experimental
result is promoted or reinterpreted here.

Validation (non-final):

```bash
cargo fmt --all -- --check
cargo clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo test -p tdi-ai --features experimental --lib tdi22_event_aggregate
bash scripts/check-tdi22-bootstrap.sh
bash scripts/check-tdi22-freeze.sh
```
