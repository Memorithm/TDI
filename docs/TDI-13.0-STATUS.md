# TDI-13.0 Status

Stage: active bootstrap; not frozen.

## Landed on bootstrap branch

- Research scope and explicit prohibition set for Boolean candidate arms.
- Architecture ladder B0-B5 separating attention baselines from non-attention candidates.
- Deterministic Stage-0 Boolean-state and clause primitives.
- Bounded direct-address memory with explicit hit/miss/collision/replacement semantics.
- Exact semantic memory-bit accounting for the Stage-0 reference memory.
- Deterministic resource counters for Boolean operations, routes, and memory events.
- Unit tests for candidate validation, delayed recall, collision handling, distractor rejection, and memory accounting.

## Still required before TDI-13.0 can be frozen

- Integrate the module into `tdi-ai` public/experimental module structure without changing stable semantics.
- Add deterministic task fixtures for keyed Boolean fact recall, copy-after-marker, conjunction retrieval, and distractor rejection.
- Add a baseline adapter interface for B0/B1 that cannot be called from B2-B5 candidate code paths.
- Define exact development budgets: sequence lengths, state widths, memory slots, sample counts, seeds, and rejection thresholds.
- Define a leak-safe development/confirmation split and a future holdout authorization rule.
- Add machine-readable run manifests and exact provenance output.
- Add scaling counters that detect accidental O(N^2) behavior independently of wall-clock timing.
- Decide whether ANF/Zhegalkin terms enter TDI-13.0 or a later TDI-13.x stage.
- Run workspace formatting, clippy, and tests on the final branch head.

## Explicit non-claims

No current TDI-13 artifact establishes that Boolean relational computation is superior to attention, that it scales to language modeling, that it is asymptotically optimal, or that it yields a hardware speedup.
