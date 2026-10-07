# TDI-21 Pascal P5 Validation runner amendment

Status: prospective operational amendment registered after the frozen P5
Development dossier and before any P5 Validation timing.

The original P5 preregistration already declares a 54-cell Validation split at
n=15 and authorizes unchanged Validation after Development gates G1-G6 hold and
Development evidence is frozen. The Development implementation froze only a
Development-only executable wrapper. This amendment does not change the P5
scientific question, geometry, matched arms, generator, timing procedure,
analysis, endpoint definitions, or interpretation. It supplies the missing thin
Validation-only entry point and a Validation plan emitter, both calling the
already-frozen public P5 functions without adding scientific logic.

The original Development evidence remains immutable. Its frozen implementation
manifest must continue to verify byte-for-byte for:

- docs/TDI-21.0-PASCAL-P5-REALIZED-COST-PREREGISTRATION.md
- tdi-ai/src/tdi21_pascal_p4.rs
- tdi-ai/src/tdi21_pascal_p5.rs
- tdi-ai/src/lib.rs
- tdi-ai/tests/tdi21_pascal_p5_preflight.rs
- tdi-ai/examples/tdi21_pascal_p5_plan.rs
- tdi-ai/examples/tdi21_pascal_p5_development.rs

Before Validation timing, the new Validation wrapper, Validation plan emitter,
Validation plan bytes, this amendment, and their SHA-256 values must be frozen
together. The host/toolchain identity and timing procedure must match the
Development freeze. The semantic preflight must cover all 54 Validation cells
with exact matched-arm outputs and zero pairwise-token comparisons.

Any change to an original frozen implementation byte invalidates this bridge
and requires a new prospective study rather than retroactive repair.

No protected/final population is created, opened, inspected, or executed. This
amendment authorizes no confirmatory/final work and no promotion into
SML-GENIUS-owned model primitives, SBG, MOR, Delta-KV, context memory, KV cache,
or model architecture.
