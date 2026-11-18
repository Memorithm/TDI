# TDI-22 — Dynamic learned wiring with persistent event propagation

**Status:** experimental, non-final software integration. This work
connects the two already merged development mechanisms: online source
association (#794) and event pulse routing (#793). It is neither a
new mathematical law nor a trained-model result.

## Mechanism

Each accepted source event updates `MatchedAssociativeLearner`, which
learns and changes its undirected topology from recent co-observations.
Both torsor and independent ordinary six-channel bilinear scorers
must agree on the selected links, active links and activation edges.

`EventRoutingMemory::observe_rewired` receives the complete selected
topology and its active subset. It validates canonical source pairs,
duplicates, fanout and edge limits, then atomically updates adjacency
and active-edge state. Newly active connections seed a pulse through
currently active links, with the already established bounded
breadth-first traversal and hop budget.

An evicted active edge is immediately removed from paths and counted
as a falling transition, **without emitting a new pulse**. Historical
`last_reached_at(node)` sequence stamps survive link deletion and
reassignment. Inactive links never forward a pulse. Nodes can be
reached without having a new external source update themselves.

The old fixed-wiring `EventRoutingMemory::observe` interface remains
unchanged; all new behavior is opt-in.

## Controls and causal ordering

`MatchedLearnedPulseNetwork` stages the whole learner and two
routing memories before any mutation. The source learner receives
a fixed sentinel target pair `(u64::MAX,u64::MAX)` and a constant
false diagnostic label because its existing API requires these
inputs; they do not affect learning or activation, and its diagnostic
confusion matrix is ignored here. The **actual** external target
node and independent task label are evaluated only after both
routing memories have produced complete traces.

Candidate and control have identical source features, association
window, support thresholds, edge and fanout budget, link activation
states, hop budget and persistent delivery stamps. The control uses
the separately implemented six-channel bilinear scorer in #794.
Both routing arms intentionally reuse the *same* pulse-law code.
An exact discrepancy in learned transitions, selected topology,
pulse delivery or confusion counters aborts the entire update.

This is a matched *logical budget*, not a matched physical footprint
or independent routing algorithm. Source to route communication is
via selected/active link lists and not an optimized dirty-edge delta.

## Regression gates

Nine deterministic tests cover first learned rising-link delivery,
absence of repeat pulses, eviction with retained historical delivery,
deterministic budget replacement with matching bilinear traces,
a two-hop delivery that follows retained paths after dynamic rewiring,
zero-hop ablation, task-label/query leakage controls, zero-score
negative control, malformed rewiring and stale-event rollback,
and atomic rollback for an invalid source event.

The source learner is bounded by 64 source IDs, 32 learned links
and 4096 source events; hop budget 0–16. The pulse code stages
whole maps and validates the entire selected topology each event,
so its actual per-event cost is dominated by snapshot/reselection.
No claim of O(degree) total execution, faster inference, lower RAM
traffic, lower joules or superiority to ordinary graph networks
is made. TDI-28 is the separate physical evidence path.

## Scientific falsification boundary

The six-channel bilinear control reproduces both selection and
routing traces. This is expected from the algebraic identity
established in #792: the classical reciprocal pairing is bilinear.
Any genuinely distinct future hypothesis must alter the learned
state law, capacity, objectives or resource tradeoffs and be tested
against a matched ordinary recurrent/graph control on labels
independent of the candidate.

Frozen TDI-22 preregistrations and protected final populations are
not touched; development evidence is not scientific confirmation.

## Exact SHA qualification

```bash
cargo fmt --all -- --check
cargo clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo test -p tdi-ai --features experimental --lib tdi22_learned_pulse_network
cargo test -p tdi-ai --features experimental --lib tdi22_event_routing_memory
bash scripts/check-tdi22-bootstrap.sh
bash scripts/check-tdi22-freeze.sh
```
