# TDI-22 — Exact six-channel bilinear negative control for sparse graph

**Status: Development-only falsification instrument.** This is not a
confirmatory protocol, novelty claim, or a benefit/performance result.

## Algebraic null hypothesis

The existing TDI-22 sparse reciprocal edge graph uses

```text
K(i,j) = R_i · C_j + R_j · C_i
x_i    = (R_ix, R_iy, R_iz, C_ix, C_iy, C_iz)
J      = [0  I3]
         [I3 0 ]
K(i,j) = x_i^T J x_j
```

The classical Klein reciprocity law is **exactly** a fixed six-channel
bilinear transformation on ordinary real vectors. It is not outside
the expressive capacity of six-channel bilinear neural or algorithmic
layers. Consequently, comparing sparse torsor edges against only a
single aggregate scalar or a dense attention implementation does not
establish any fundamentally different representational capacity.

## Implemented independent control

`tdi22_bilinear_control.rs` builds an ordinary six-channel reference
from the same labelled source events. Its graph uses the **same
undirected wiring**, one score and activation bit per edge, identical
hysteresis thresholds, event ordering and six scalar source channels.
It does *not* call the candidate graph's scoring function. Rather,
it computes the independent dot-product form using six-element arrays.

`MatchedBilinearProtocol` runs both implementations transactionally
on one source event and compares every affected edge score, activation
transition and currently active edge. Any discrepancy or invalid
intermediate operation fails closed. External binary task labels are
introduced only after both predictions; TP/FP/TN/FN are recorded
separately for each arm and expected to be **equal**.

The ordinary control recomputes incident edges by scanning its whole
edge map O(E); its logical per-source and per-edge state matches the
candidate but physical allocation footprints differ (candidate also
stores adjacency). The paired protocol clones both graphs at each
event and is O(N+E) or greater. **It is not a latency, throughput,
communication or energy benchmark.** This project already owns
resource-cost hypotheses and evidence contracts in TDI-28.

## Required negative cases

Seven software tests check independent algebraic identity, complete
edge-by-edge trace parity on signed events, cancellation where the
aggregate score loses local relations, unchanged predictions under
label permutations, strictly incident edge updates, no-signal input,
and rollback on overflow/stale events.

On this matched fixed bilinear control, identical predictions are
**expected**. This is a meaningful falsification of any claim that
the Klein pairing alone gives a new type of expressive information.
A truly distinct candidate must differ in **update law, state
evolution, routing, event semantics or trained inductive bias** and
must be compared against an ordinary graph granted the same resources.

Scientific conclusions remain conditional: the controls are software
identities and toy trace tests, not a trained-model or out-of-sample
generalization study. No TDI frozen final surface is touched.

## Exact SHA qualification

```bash
cargo fmt --all -- --check
cargo clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo test -p tdi-ai --features experimental --lib tdi22_bilinear_control
bash scripts/check-tdi22-bootstrap.sh
bash scripts/check-tdi22-freeze.sh
```
