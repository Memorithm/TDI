# TDI-21 Pascal P2 Development preflight evidence

Status: retained non-final negative implementation/protocol evidence.

This record does not contain confirmatory or final evidence. No protected/final
population was opened or generated, and no P2 Validation scientific cell was
executed.

## Bound inputs

Execution source commit: `2d4fc0e4e254804725418ef383294665912f1723`

P2 preregistration SHA-256:
`5cf4a3e87e324657b69d82ef58eba5aac15004b1bcdec5a4bac0c7a76c42d5f2`

Preflight source SHA-256:
`0c216dd43d308b9e13d8b4edf457a8fb799472da37c8b4e397c7556d1262a5dc`

Development qualification script SHA-256:
`622b6327f7b4843d04438b95fa8a6fc433f492b8eec6d702020abb4ca4c28014`

Command:

```text
cargo test --manifest-path tdi-ai/Cargo.toml --locked --test tdi21_pascal_p2_preflight -- --nocapture
```

## Exact Development result

The frozen P2 Development grid contains 108 cells. The inherited P1 affine
generator formula is viable for 72 cells and blocked for 36 cells.

Observed deterministic summary:

```text
PASCAL_P2_DEV_PREFLIGHT,total=108,eligible=72,blocked=36,reason=inherited_generator_duplicate_masks
```

All three preflight tests passed. The pass means the blocker was reproduced and
retained exactly; it does not mean P2 passed its scientific gates.

The blocked region is exactly `n=9` at densities `d=32` and `d=256`,
across all preregistered schedules, query loads and reuse values. With the
inherited P1 formula,

```text
mask = 1 + ((257 * gate + 73 * term) mod (2^n - 1))
```

for `n=9`, `2^9 - 1 = 511 = 7 * 73`. Therefore the term sequence has period
7 and cannot provide 32 or 256 unique masks. Changing that generator after this
observation would change the preregistered design.

## Additional protocol ambiguity retained

P2 preregisters signed contrasts using `work(DIRECT_ANF_QUERY)`,
`work(PASCAL_ZETA_BANK)`, and `work(GENERIC_FULL_MATERIALIZE)`, but it does
not define the exact `work(...)` accounting function. The existing P1 harness
records primitive counters, but selecting a new aggregation after observing P2
would be post-hoc. No signed P2 contrast is therefore invented in this record.

## Decision

P2 cannot proceed as a valid full Development/Validation execution under its
current frozen specification. The 36 blocked Development cells and the undefined
work aggregation are retained as negative implementation/protocol evidence.

No blocked cell is rerun under a modified generator and presented as P2. Any
corrected generator or work accounting must be introduced under a new
prospective protocol version before execution. SML-GENIUS remains the owner of
model-side Pascal primitives; this finding authorizes no SBG, MOR, Delta-KV,
context-memory or model-architecture promotion.
