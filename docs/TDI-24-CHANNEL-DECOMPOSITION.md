# TDI-24 Slice 04 — Primitive channel decomposition contract

Status: bounded Stage-A semantic surface; non-confirmatory.

The TDI-24 chiral carrier exposes three primitive scalar channels before any weighting or normalization:

- `s(q,k) = q^T k` — channel ID `S`, even under simultaneous reflection;
- `m(q,k) = q^T M k` — channel ID `M`, even under simultaneous reflection;
- `chi(q,k) = q^T J k` — channel ID `Chi`, odd under simultaneous reflection.

The algebra remains `tdi24-mirror-coupled-chiral-v1`. The tagged decomposition surface is versioned independently as `tdi24-channel-decomposition-v1`.

`decomposed_observables()` returns each scalar separately with its channel ID, declared reflection parity, algebra-contract ID and decomposition-contract ID. It does not compute a weighted score, normalize, mask, aggregate channels or select an outcome.

The separate version tag is intentional: later changes to experimental metadata can be reviewed without pretending that the underlying `M/J` algebra changed. Conversely, a future algebra revision must receive its own algebra-contract identity.

## Scientific boundary

This tranche adds semantic provenance to existing primitive observables. It does not train a model, execute Development/Validation or protected populations, establish utility, or compare V6 and C6 performance.
