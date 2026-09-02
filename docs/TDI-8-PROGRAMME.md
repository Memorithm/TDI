# TDI-8.x — Recurrent Associative Architecture Research Programme

TDI-8.x is the distinct TDI scientific line for bounded alternative
recurrent/associative architecture experiments.

The authoritative TDI-8.0 design is
[`TDI-8.0-ASSR-PREREGISTRATION.md`](TDI-8.0-ASSR-PREREGISTRATION.md).
Current bounded implementation status is tracked in
[`TDI-8.1-STATUS.md`](TDI-8.1-STATUS.md).

## Stage map

| Stage | Purpose | Status |
| --- | --- | --- |
| TDI-8.0 | Freeze falsifiable ASSR / ASSR-H questions and controls | complete — merged and frozen by PR #88 |
| TDI-8.1 | Build bounded deterministic A0/A1/A2/A3 evaluator | active — reference foundation merged by PR #89 |
| TDI-8.2 | Execute untouched confirmatory holdout | future human-only, does not exist and is not authorized |
| TDI-8.3+ | Ablation, robustness, transfer and ecosystem extensions | conditional on evidence |

## Reference architecture ladder

- **A0:** competent attention-like full-history reference.
- **A1:** bounded recurrent-state-only reference.
- **A2:** A1 plus explicit bounded associative memory; working label ASSR.
- **A3:** A2 plus a bounded VSA/holographic workspace paid from the same dynamic-memory budget; working label ASSR-H.

The scientific primary contrasts are A2 versus A1 and A3 versus A2. A0 is a
contextual full-history reference rather than the primary matched-budget arm.

## Current TDI-8.1 implementation boundary

The merged TDI-8.1 foundation establishes typed architecture identities and
exact component-wise memory accounting. It is not yet the complete evaluator.
The next bounded implementation work is deterministic associative-memory
reference semantics with explicit addressing, write/read, collision and
replacement behavior plus exact metadata accounting and oracle tests.

Concrete experimental dimensions, memory budgets, horizon values, non-final
seed ranges, sample counts, paired interval configuration and typed rejection
policy remain TDI-8.1 decisions that must be frozen before any TDI-8.2 surface
can exist.

## Mandatory separation from TDI-7

TDI-8 work must not read, generate, reuse or modify TDI-7.2 final-holdout data,
seeds, execution tokens or authorization state. TDI-7.2 remains separate and
must not be touched by TDI-8 development.

## Downstream ecosystem sequence

The intended promotion order is:

1. TDI-8.0 preregistration — complete and frozen;
2. TDI-8.1 deterministic reference correctness and bounded evidence — active;
3. TDI-8.2 human-only confirmatory result only if separately armed and explicitly authorized;
4. Forge search only after a leak-safe domain contract exists;
5. reusable general primitives may move to SciRust;
6. NNIS may implement validated mechanisms for NVIDIA hardware;
7. a dedicated ASSR repository is considered only if the semantics become
   stable enough to deserve independent product ownership.

DA-LUC-like KV representation work remains primarily owned by SLHAv2 rather
than TDI-8.

## Scientific interpretation rule

The architecture names are working labels. Neither a positive bounded result
nor a fast implementation is by itself evidence of architectural novelty,
universal asymptotic superiority, language-model quality, constant total
memory, cognitive transparency or hardware speedup.
