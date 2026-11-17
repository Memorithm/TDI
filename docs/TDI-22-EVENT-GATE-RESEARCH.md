# TDI-22 — Event-observed torsor hysteresis (T3/T4)

Status: experimental **Development-only** software mechanism. This is not
a change to the frozen TDI-22.0 algebra, preregistration, protected/final
population, production model, or TDI-25 scientific verdict.

## Question

Can a transport-dependent torsor pairing induce different *state transitions*
than its matched six-channel non-transported control when both observe the
same event sequence, source memory, twist, point, and hysteresis budget?

## Candidate and control

| Arm | Score |
|---|---|
| T3 | (v + Q × omega) · R + omega · C |
| T4 | v · R + omega · C |

Here R and C are the current unnormalized totals from the already merged
EventTorsorAggregate. Both arms use one active-state bit, common finite
thresholds lower < upper and the same stream of inserted/replaced/deleted
source contributions. An inactive observer rises when score >= upper;
an active one falls when score <= lower; otherwise it holds its state.

The observer does not apply the event, does not consume oracle answers, and
does not suppress computations. Each explicit observation recomputes a score.
The T4 control has the same six stored channels but is not matched on exact
floating-point operation count; that must be separately accounted for.

## Bounded deterministic software controls

- T3/T4 observe the identical sequence of inserts, replacements and removal.
- Explicit expected scores and one-bit rising/falling/retention are checked.
- Common translation of P and Q preserves T3 score and activation; the
  ordinary T4 score can differ because it lacks query transport.
- Changing only the source torsor reduction point preserves either score.
- Invalid thresholds/positions and non-finite derived score fail closed;
  a score error cannot mutate the hysteresis state.

The translation fixture is an algebraic discrimination, **not** a task win.
Only a separate, registered matched comparison could evaluate usefulness.

## Limits

This is not a physical/biological torsor or proof that positions inferred
for language tokens carry mechanical meaning. No learned geometry, causal
identifiability, latency, joules, or model quality is measured. The gate
stores one bit but scores are always evaluated. TDI-28 cost accounting
requires an explicit versioned adapter and a matched workload before claims
about energy savings. ADA or FLAT adoption is NOT authorized.

Validation command:

```bash
cargo fmt --all -- --check
cargo clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo test -p tdi-ai --features experimental --lib tdi22_event_gate
bash scripts/check-tdi22-bootstrap.sh
bash scripts/check-tdi22-freeze.sh
```
