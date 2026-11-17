# TDI-22 — Experimental event-only impulse interaction memory

Status: **non-final Development software hypothesis**. Does not change
the frozen TDI-22.0 representation, the TDI-22.1 arm protocol, any protected
population, or FLAT-ATTENTION/ADA production execution.

## Candidate event law

For the same event-updated torsor aggregate `(R,C)` and fixed twist
`(v,omega)` at `Q`, calculate a scalar score after every declared event:

- T3 transported: `(v + Q × omega) · R + omega · C`
- T4 matched-width control: `v · R + omega · C`

For either arm, let `s[n]` be the score at event n and `u[n]` the
one-scalar impulse memory. With predeclared retention `0 <= λ <= 1`:

```text
Δs[n] = s[n] − s[n−1]
u[n]  = λ × u[n−1] + Δs[n]
```

The initial `s[-1]` is sampled from the aggregate at construction;
initial memory is 0. Only accepted, explicit `observe_event(sequence, aggregate)`
calls advance this recurrence. There is **no time-based decay without an
event**. For both arms the same finite `lower < upper` thresholds apply:
inactive rises at `u >= upper`, active falls at `u <= lower`, otherwise
the one-bit state remains unchanged.

The hypothesis is that *changes in transported interactions* might be
usable state-transition inputs independent of instantaneous score. This
recurrence is proposed, not mathematically discovered by ADA or confirmed
to benefit a trained model. Its physical impulse analogy is not an
identity between physical and cognitive quantities.

## Software controls and fail-closed behavior

- A signed insert/replace/remove stream produces T3 and T4 event edges
  with identical retention, threshold and state budgets.
- Different event histories ending in the same static aggregate may
  produce different memory values; this is a path-dependence control.
- A common origin translation of source points and query points
  preserves the T3 score, difference, memory and edge sequence.
- Binding to a preloaded aggregate avoids an artificial initial pulse.
- NaN/infinite/out-of-range retention, stale or duplicate event sequence,
  non-finite score difference, or non-finite memory are rejected without
  partially changing the observer state.

The `sequence` is supplied by the caller and must strictly increase.
This module does not validate whether source events were delivered,
whether timestamps are trusted, or whether external dependencies changed.
Arithmetic uses binary64 and can accumulate drift; TDI-22's explicit
compensated rebase remains separate and is not invoked automatically.

## Resource and scientific boundary

Each observer stores two scalar values (memory and last score), one
active bit, one sequence stamp and fixed parameters. Both arms use the
same update law; arithmetic counts of T3/T4 scoring differ. This version
**still evaluates a full score on every delivered event**. There is no
claim of less memory traffic, lower wall-clock time, reduced joules,
better attention or learned task success. TDI-28 accounting would
require an independently versioned adapter and controlled workloads.

Validation:

```bash
cargo fmt --all -- --check
cargo clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo test -p tdi-ai --features experimental --lib tdi22_event_impulse
bash scripts/check-tdi22-bootstrap.sh
bash scripts/check-tdi22-freeze.sh
```
