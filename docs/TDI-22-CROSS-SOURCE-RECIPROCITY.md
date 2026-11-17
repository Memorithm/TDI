# TDI-22 — Cross-source reciprocal interaction hypothesis

**Status: development-only research hypothesis.** The underlying
bilinear expression is the classical screw/Klein reciprocal form.
It is not claimed as a new mathematical law, a new training algorithm,
an attention result or a measured improvement.

## Scientific question

Previous TDI-22 event gates score the sum of source torsors against a
fixed twist. A linear query cannot distinguish two source decompositions
if their six-channel totals are identical. Here the candidate state
depends directly on **relationships among pairs of sources**.

For keyed, signed weighted torsors i, define the resultant R_i and
origin-reduced moment C_i = M_i(P_i) + P_i × R_i. Then define

```text
K(i,j) = R_i · C_j + R_j · C_i
S      = Σ_{i<j} K(i,j)
       = (Σ R_i) · (Σ C_i) − Σ (R_i · C_i)
```

This cross-source score discards all self-pair terms. Unlike
the aggregate-only scalar `(ΣR)·(ΣC)`, it depends on the source
decomposition, not solely on the aggregate six-channel state.

**Counterexample:** two contributions can both sum to R=0 and C=0
but yield S=-2 or S=+2 according to the alignment of each source's
R and C. A one-bit hysteresis observer can therefore produce different
event transitions for configurations that the aggregate-only control
cannot distinguish.

## Geometry and mathematical controls

Under a shared spatial translation d, a physically translated source
point changes C_i to C_i + d × R_i, with the local source moment kept
fixed. The added terms cancel pairwise:

```text
R_i · (d × R_j) + R_j · (d × R_i) = 0
```

Therefore K(i,j), S and event decisions are invariant to a common
translation. Changing each torsor's *reduction point* using the frozen
Varignon transport convention leaves C_i unchanged and preserves S.

This invariant is classical screw-system mathematics. The research
hypothesis concerns whether using it as an autonomous source-source
interaction and state-transition rule can have useful computational
consequences distinct from ordinary scalar-score attention.

## Implementation

`tdi-ai/src/tdi22_cross_source_reciprocity.rs` provides a keyed map
of signed weighted sources, cached total R/C, a cross-source score,
atomic insert/replace/remove, an independent direct O(K²) pair oracle,
a six-channel Neumaier-compensated O(K) source replay, an explicit
transactional numerical drift audit/rebase, and a one-bit hysteresis
observer. The incremental candidate recurrence is

```text
S_after = S_before − K(old_source, all_other_sources)
                    + K(new_source, all_other_sources)
```

All derived calculations are checked for nonfinite arithmetic before
mutating state. The audit compares **both** the independently replayed
cross-source score and the six separately compensated cached R/C channels.
The explicit `rebase()` transactionally replaces all seven values, not
just the scalar score, or preserves the entire original state if a replay
fails. This prevents stale channel totals from corrupting subsequent
source events after a numerical rebase.

A documented cancellation regression inserts R.x=10^16, then R.x=1,
then removes the first contributor. An ordinary incremental accumulator
loses the unit residual. The channel audit detects that drift, and rebase
restores it before the next cross-source interaction is evaluated.
Additional fixtures exercise the C channel and reject overflowing
replays atomically.

The audit/rebase costs O(K²) for independent pair replay plus O(K)
for compensated channel replay. Neither is a normal incremental fast path.
This software treatment is **not** a certified numerical error bound:
incremental binary64 drift remains possible.

## Required falsification

Tests include identical total R/C with opposite cross interactions,
common translation invariance, source reference-point transport,
weighted insert/replace/delete streams versus the independent pair
oracle, nonfinite rejection with complete rollback, single-source
null coupling, and idempotence.

A future registered comparison must add equal-memory controls
for ordinary query pairing, source-count effects and unrestricted
bilinear features, evaluate independent tasks, test adversarial
source encodings and study numerical drift. Any energy analysis
belongs to the separate TDI-28 evidence protocol.

**Limits:** the source map consumes O(K) state, the reference
oracle scans O(K²) unordered pairs, and source-ID provenance is
supplied by the caller. The current formula is symmetric and scalar,
not a neural network, learned attention, general relational memory,
or proof that the laws of mechanics govern cognition.

## Qualification

```bash
cargo fmt --all -- --check
cargo clippy -p tdi-ai --features experimental --all-targets -- -D warnings
cargo test -p tdi-ai --features experimental --lib tdi22_cross_source_reciprocity
bash scripts/check-tdi22-bootstrap.sh
bash scripts/check-tdi22-freeze.sh
```
