# TDI-22 — Sparse event-only reciprocal edge graph

Status: **Development-only mechanistic candidate**. This is an explicit
interaction topology, not a Transformer replacement, a trained model,
a novel mathematical law, or a measured hardware/energy improvement.

## Distinct hypothesis

Global aggregate score can cancel even when individual source-source
relationships are strongly positive or negative. Instead of collapsing
all sources into a single scalar, a sparse *declared* graph keeps
one hysteretic activation state for each wired source pair.

For each declared undirected edge `(i,j)`, evaluate the classical
reciprocal torsor form:

```text
K(i,j) = R_i · C_j + R_j · C_i
C_i = M_i(P_i) + P_i × R_i
```

The score law is the classical symmetric Klein pairing. The research
question is whether a **topologically sparse, event-activated source
network** enables a useful kind of relational memory independent of
global-score aggregation. We make no novelty or benefit claim for
the underlying pairing.

## Implemented operations

`SparseReciprocalEdgeGraph::new(wiring, lower, upper)` validates a
fixed undirected topology before admitting any source. It rejects
duplicate/reversed edges, self-links, excessive fanout and invalid
thresholds. The reference bounds are 4096 edges and 256 neighbors
per source. All thresholds require `0 <= lower < upper`, so absent
endpoints (score zero) cannot activate a link and removing an active
endpoint produces a falling edge.

`apply(TorsorEvent)` stages only edges adjacent to the modified
source, computes their score and local rising/falling/retention
transitions, validates all numerical results and commits changes
atomically. An unchanged upsert has no further effects. Removing a
missing source fails closed. A disconnected source does not touch
any edge.

Per event with degree d and N sources/E declared edges, indexing
work is O(d·(log N + log E)), with O(d) staged output. This is
an algorithmic reference estimate, **not** a timed measurement, a
cache-miss measurement, or a joule reduction. The fixed BTree
containers carry nontrivial memory and allocation overhead.

`verify_all_edges()` explicitly recomputes every declared pair as an
independent O(E) reference diagnostic; it does not run in `apply()`.
State remains keyed by exact contributor IDs and declared edges;
source-ID provenance and wiring are caller-supplied.

## Mechanistic falsification

The three-source fixture contains one resultant-only source and two
opposing moment-only sources. Their **global reciprocal score is zero**,
but the two sparse edges score +2 and -2, so exactly one edge rises.
That establishes a representation difference from a scalar global
aggregate, not an externally useful cognitive benefit.

Seven tests cover this cancellation example, degree-local updates,
independent edge retention, replacement/deletion/idempotence, shared
spatial translation invariance, atomic failure on derived overflow,
invalid wiring/fanout and isolated-source behavior.

## Boundaries and next registered comparisons

A properly matched comparator must include **ordinary six-channel
bilinear pairwise features with identical wiring, state bits and
thresholds**, not only the aggregate scalar. Because the present
pairing can be implemented as basic dot products, its advantages
must not be attributed to an intrinsically new torsor law.

A task-quality experiment must define labels outside the graph's
scoring rule, use predeclared controls and preserve null results.
TDI-28 is the existing separate framework for physical energy
evidence. This module does not modify any frozen TDI-22 contract
or protected experimental population.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo test -p tdi-ai --features experimental --lib tdi22_sparse_reciprocal_graph
bash scripts/check-tdi22-bootstrap.sh
bash scripts/check-tdi22-freeze.sh
```
