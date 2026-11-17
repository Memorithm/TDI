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

## Numerical drift audit and explicit compensated rebase

The default `apply` path remains an O(log K) source update with constant-size
floating-point arithmetic. After many inserts, replacements and deletions,
non-associative binary64 cancellation can cause incremental totals to diverge
from a fresh sum of the active contributions. This is not a torsor-specific
identity failure; it is a numerical consequence of summation order.

`audit_rebase()` deterministically traverses the live source-id map, uses
six separate Neumaier-compensated accumulators, and reports the observed
maximum absolute disagreement in the resultant and origin-moment channels.
It does **not** modify state and is an O(K) operation.

`rebase()` performs that audit and atomically replaces both cached channels
only if the complete recomputation and diagnostic checks succeed. It preserves
all contributor identities. Neither the reported differences nor the
compensated results are rigorously certified numerical error bounds.

A deterministic cancellation control adds `1e16`, `1`, and `-1e16`
in that event order. Simple binary64 incremental accumulation loses the
unit residual, while ordered Neumaier replay recovers it. Additional tests
cover no-mutation audits, repeated rebases, reduction-point consistency,
pairing parity and empty-state behavior. These are software fixtures, not
energy/performance findings.

**Research cost:** an audit or rebase explicitly scans O(K) stored entries
and uses more arithmetic than the normal update path. Before integrating
any automatic cadence, compare numerical drift, task quality, memory traffic
and physical cost under preregistered workloads and preserve negative results.

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
