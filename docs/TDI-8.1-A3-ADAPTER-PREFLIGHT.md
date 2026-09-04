# TDI-8.1 A3 symbolic adapter preflight

Status: **bounded software-oracle qualification only — not H8-B evidence and not a final TDI-8.1 configuration freeze**.

Tracks #137 and the frozen TDI-8 programme issue #87. The cross-mechanism transaction required by this policy was qualified separately in #136.

## Policy boundary

The adapter receives only the existing `SymbolicTaskAdapter` arguments and passes them through the merged leakage-safe `LosslessTaskEncoder`. Exact targets, generator source indices and T3 collision-class annotations remain evaluator-owned and cannot enter the A3 input frame.

The bounded candidate policy is:

- association events encode the visible key/value pair, advance A2 using the instance-scoped neutral non-query read key, and atomically store the same encoded event in VSA while writing A2 under `association_memory_key(key_code)`;
- payload events encode only the visible value, derive their logical key from the adapter-owned `PayloadKeyCursor`, then atomically use that same logical key for A2 and VSA storage;
- the payload cursor is committed only after the combined A2+VSA transaction succeeds;
- distractors advance A2 with `A3VsaReadRoute::Skip`, the neutral non-query A2 read key and no write;
- association queries use the association logical key for both keyed VSA retrieval and the A2 lookup, with no write;
- payload queries use the payload logical key for both keyed VSA retrieval and the A2 lookup, with no write;
- no VSA cleanup, similarity search, candidate vocabulary or target-conditioned decoder is introduced.

Write events deliberately use the merged `step_skip_vsa_and_store` primitive. The prospective VSA bundle is therefore prepared before A2 mutation and committed only after a successful A2 step. This preserves the qualified cross-mechanism atomicity and the existing A3 temporary-memory peak.

## Dual-path software oracle

The executable uses fixture-only parameters designed to prove that the preflight does not silently neutralize either A2 or VSA:

- recurrent input width is the existing lossless minimum;
- recurrent state width is four coordinates, split into two association-readout limbs and two payload-readout limbs;
- association symbol limbs are mapped into the association state coordinates with weight `0.5`;
- payload symbol limbs are mapped into the payload state coordinates with weight `0.5`;
- A2 associative fusion gain is `1.0`;
- A3 VSA fusion gain is `1.0`.

For a single stored item, keyed VSA retrieval contributes one exact half-symbol to the designated readout coordinates and the resident A2 associative payload contributes the other exact half-symbol. Because the exact-u64 limbs are finite binary fractions, the equal half contributions reconstruct the canonical limbs exactly in this software fixture.

The main preflight executes a T2 instance with one payload item and a positive distractor delay. It therefore exercises payload storage, distractor VSA-skip behavior, delayed logical-key query routing, keyed VSA retrieval, A2 memory lookup and exact symbolic readout in one evaluator-owned execution.

Additional tests exercise an isolated association round trip, prove distractors and queries leave the VSA workspace unchanged, and force an A2 rejection after VSA preparation to verify that the payload cursor and VSA persistent state remain uncommitted.

## What this does not establish

This preflight does **not** freeze:

- recurrent or VSA experimental dimensions;
- associative table capacity or projection seed;
- associative or VSA fusion gains;
- VSA role seed;
- horizons, seed populations or sample counts;
- T1/T3 multi-item VSA superposition/cleanup behavior;
- matched A1/A2/A3 experimental budgets;
- late-retrieval deficit definitions or uncertainty intervals;
- intervention sites or ablation semantics;
- an H8-B verdict.

In particular, the exact single-item dual-path oracle is a software qualification of routing and arithmetic, not evidence that raw superposition without cleanup is an adequate final multi-item A3 policy.

## Holdout boundary

- TDI-8.2 executable: absent;
- TDI-8.2 seed/result surface: absent;
- final holdout: does not exist;
- H8-B evidence: not produced by this preflight.
