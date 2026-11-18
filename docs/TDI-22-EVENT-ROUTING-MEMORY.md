# TDI-22 — Event pulse propagation and matched bilinear memory

**Scope:** opt-in non-final Development code; no frozen TDI-22,
TDI-28, protected final data, deployed model, physical runner, or
production service is modified.

## Implemented mechanism

The merged TDI-22 sparse graph emits `Rising`, `Falling` and `None`
events for the declared reciprocal source links. The new
`tdi22_event_routing_memory.rs` adds a **distinct event-driven state
transition law** on top of those edges:

1. An external source event updates the sparse graph once.
2. Rising links seed both endpoints with distance zero.
3. A bounded breadth-first wave traverses only currently active
   declared links, to at most `max_hops` (0–16) additional hops.
4. Each reached node stores the last accepted event sequence.
5. No rising edge means no wave. Falling edges update link activity
   but do not delete earlier delivery memory. No timer or background
   clock causes state changes.

The per-node output includes the minimum hops from any rising-link
endpoint during that event. This makes a multi-hop message delivery
observable without a fresh input event at the destination.

## Fair comparator

`MatchedEventRoutingProtocol` runs two **independent** edge scoring
implementations: the torsor-source graph from PR #791 and the plain
six-channel bilinear control from PR #792. The two sides have identical
link topology, thresholds, hop budgets, event order and explicit
one-bit-plus-timestamp routing state per node. It compares exact
edge transitions, active edge sets and entire routed node waves.

The same routing law is implemented for BOTH. Consequently any
routing behavior is also accessible to the ordinary bilinear graph.
No distinct expressivity or task success is claimed for torsors.

A second, ablated control uses `max_hops=0`. For the four-node
chain, a rise on link (3,4) reaches node 1 only when `max_hops>=2`.
This is a change of the communication budget, not a demonstration
that the two models have equal runtime or that a torsor offers
something a matched bilinear graph cannot.

Any externally supplied target node and binary label are used
**after** both routing waves are computed. Candidate and plain
bilinear TP/FP/TN/FN counts must agree. The embedded PR #792
comparison also records its independent any-active-link task score;
the new routing report is specifically about node reachability.

## Transactional and computational boundaries

Source events and paired route state are staged by cloning the whole
reference protocol. Derived invalid source events, stale sequences
and contradictions in edge updates are rejected without partially
committing state. The reference caps each sequence at 4096 events
and `max_hops` at 16.

Each wave traverses only its active connected neighborhood, bounded
by declared topology and the hop limit. However complete staging
clones source/edge/adjacency maps on every event, so this is
**not an O(degree) total-cost implementation**. No physical
memory traffic, joules, latency, communication bytes or execution
speed improvement is inferred. Those require separate TDI-28
accounting and physical qualification.

## Deterministic software regression

Eight tests exercise two-hop delivery, `max_hops=0` ablation,
persistence of event memory across falling links, labels with
unchanged predictions, disconnected topology, stale/deleted
source rollback, malformed edge-update rollback and zero-signal
negative controls. They are software semantics only.

## Exact-HEAD validation

```bash
cargo fmt --all -- --check
cargo clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo test -p tdi-ai --features experimental --lib tdi22_event_routing_memory
bash scripts/check-tdi22-bootstrap.sh
bash scripts/check-tdi22-freeze.sh
```

### Next evidence gate

Compare the routing law against a bilinear graph with the SAME
hop budget, activation memory and source encoding on externally
registered tasks requiring delayed information delivery. Measure
quality and resource use separately. Retain parity and negative
results instead of promoting a classical identity as novelty.
