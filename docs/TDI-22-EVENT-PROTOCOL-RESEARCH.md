# TDI-22 — Non-final independent-label event comparison protocol

Status: **Development-only research infrastructure**. No new frozen
preregistration, task-quality result, final data, external measurement, or
ADA/FLAT-ATTENTION adoption is authorized by this module.

## Purpose

The TDI-22 event aggregate, one-bit hysteretic gate, and impulse memory
already exist. This increment adds a real paired evaluation carrier: a
single event updates the same source aggregate observed by four mechanisms.
An independently supplied binary label is used **only after** all four
predictions have been computed. No label reaches the score or recurrence.

| Arm | Score law | Activation law |
|---|---|---|
| InstantT3 | Transported T3 | Instant hysteresis |
| InstantT4 | Non-transported T4 | Instant hysteresis |
| ImpulseT3 | Transported T3 | Score-delta impulse memory |
| ImpulseT4 | Non-transported T4 | Score-delta impulse memory |

Every arm sees the same keyed insert, replacement, or removal, the same
twist, query point, thresholds, event sequence and independent task target.
Impulse arms additionally share the same retention coefficient.

## Mechanism and negative controls

PairedEventProtocol::push(sequence, event, target) stages the aggregate
and all four observers together, then computes four predictions before
recording TP/FP/TN/FN and rising/falling transitions independently for
each arm. Any failed operation (including non-finite derived score)
rejects the entire event without updating source, observers, counters
or sequence. Duplicate/out-of-order sequence numbers fail closed.

The six-channel T4 control is a width-matched semantic comparator,
**not** matched for scalar arithmetic operations. The independent
target comes from the caller; the reference software cannot establish
that it was acquired without leakage. The evaluator must independently
register where the labels came from. No target can be inferred from the
torseur formula as proof of externally useful intelligence.

Tests include a deliberately arbitrary label sequence, a label-only
permutation proving predictions do not depend on target values,
a no-signal negative control with alternating targets, transactional
rollback of invalid source events and derived overflow, and
common-origin translation parity for the transported T3 arms.

## Complexity and boundaries

Transactionality is implemented by **cloning the entire keyed aggregate
for every reference event**: O(K) memory/work per processed event
for K live contributions. This deliberately sacrifices runtime efficiency
for simple, inspectable fail-closed reference semantics. It is **not**
the O(log K) event-update fast path and has no energy-efficiency claim.
An independent optimized path would require a separate source-level
atomic commit primitive and differential equivalence testing.

Maximum: 4096 events per protocol instance. Counts are checked.
All outcomes are descriptive Development diagnostics only; no inference,
significance test, trained model, physical-energy measurement or
generalization claim is made. Negative and null results must be kept.

Validation:

```bash
cargo fmt --all -- --check
cargo clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo test -p tdi-ai --features experimental --lib tdi22_event_protocol
bash scripts/check-tdi22-bootstrap.sh
bash scripts/check-tdi22-freeze.sh
```
