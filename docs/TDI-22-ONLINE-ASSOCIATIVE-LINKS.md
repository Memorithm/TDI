# TDI-22 — Online, budgeted associative source wiring

Status: **non-final Development hypothesis**. This increment implements
real event-driven link creation, replacement and deletion, rather than
replaying a fixed graph. It does not claim a novel learning law, useful
generalization, lower joules or industrial throughput.

## Causal learning policy

Each accepted source-ID event has a strictly increasing sequence number.
An upsert of source i increments an integer support counter for every
other currently retained source j whose last update occurred at most
**W sequence units earlier**. No label, target query or reciprocal
score enters this counter. A source removal deletes all its pair
evidence and any selected incident links. A repeated upsert is itself
an observed event; self-pairs are never counted.

Eligible undirected edges need `support >= min_support`. Every event
reselects the best eligible links, using the published deterministic
ordering `(support descending, last co-observation descending, pair ID
ascending)`. Greedy selection enforces both a global edge budget
and a degree cap per source. Previously wired links may be displaced
immediately when a newer or stronger association wins. Reassignment
takes effect without a background timer. Support saturates at `u16`
maximum instead of overflowing.

The reference caps a run at 4096 events, 64 source IDs, 32 selected
links and 8 links per source. Candidate support storage is therefore
at most 64×63/2=2016 unordered source pairs. Source recency is a
sequence index, **not real time**.

## Strictly matched bilinear control

Newly selected or retained edges score using two separately written
functions: T3 reciprocal `R_i·C_j + R_j·C_i` and ordinary six-channel
`x_i^T J x_j`. Both use the same source events, weighted six channels,
wiring-selection policy, thresholds, one-bit hysteresis state, budgets
and event sequence. Exact score/transition parity is required for
commit; disagreement rejects the entire transaction.

The target pair and binary task label are consulted **only after**
all selections, gate scores and transitions have been calculated.
Separate confusion matrices record the two arms. As demonstrated
by TDI-22 #792, these score forms are algebraically identical;
the expected outcome is equal logical behavior.

## Independent negative controls and rollback

Eight tests cover repeated associations forming an active edge;
bounded rewiring from an old pair to a newer stronger pair;
deletion purging evidence; label and query permutations leaving
topology identical; zero-signal source events learning topology but
not activating it; degree and edge budgets; transactional rollback
on stale sequence, missing source and numerical overflow; and a
zero-edge-budget control that cannot activate any link.

Tests are deterministic software fixtures, not independent held-out
task evidence or a measured physical speedup. The implementation
clones the entire learner state before each event, recomputes the
bounded eligibility ordering and reevaluates selected pair scores.
It is not an O(degree) incremental fast path, and the two Rust
representations are matched in logical budgets **not exact bytes**.

## Boundaries and next integration step

The learner supplies a currently selected graph and link admissions,
evictions, rising and falling events. It does not yet mutate the
previously merged TDI-22 fixed-wiring event routing memory (#793).
Integrating the learned topology with that memory will require an
explicit transactional rewire interface and rules for persistent
node timestamps when links disappear. No frozen preregistration
or TDI-28 physical resource evidence is modified.

Before claims about intelligence, predeclare independent tasks
whose labels depend on useful temporal structure, evaluate
independent split generalization and compare against a six-channel
ordinary bilinear learner with identical association and memory
budget. Preserve negative outcomes.

## Exact commit validation

```bash
cargo fmt --all -- --check
cargo clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo test -p tdi-ai --features experimental --lib tdi22_online_associative_links
bash scripts/check-tdi22-bootstrap.sh
bash scripts/check-tdi22-freeze.sh
```
