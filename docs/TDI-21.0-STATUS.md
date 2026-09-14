# TDI-21.0 Status

Status: **active bootstrap; not frozen; development evidence only**.

## Implemented on the bootstrap branch

- Canonical TDI-21 programme and candidate prohibition set.
- Architecture ladder B0-B5 with B0/B1 explicitly separated as attention baselines.
- Experimental-only Rust module exposed through `tdi-ai::experimental::tdi21`.
- Fixed-width Boolean state and literal/conjunction evaluation.
- Deterministic Boolean route activation.
- Bounded direct-address memory with explicit hit, miss, collision, and replacement semantics.
- Exact Stage-0 semantic memory-bit accounting.
- Boolean operation, route, memory, and pairwise-comparison counters.
- Pairwise-comparison fail-closed evidence guard for B2-B5.
- Algebraic normal form / Zhegalkin evaluation over Boolean assignments.
- Deterministic delayed-bit-recall fixture.
- Unit tests for prohibition validation, direct recall, collision behavior, distractor rejection, ANF truth tables, semantic memory accounting, route determinism, and pairwise-work rejection.

## Not yet established

The bootstrap does not yet establish:

- that B2/B3 match a competent attention baseline on a frozen task suite;
- that Boolean routing scales better in real wall-clock or device measurements;
- that the current direct-address memory is sufficient for long-horizon or language tasks;
- that ANF/Zhegalkin composition improves the candidate frontier;
- that B4/B5 training or search is feasible;
- that any FLAT-ATTENTION hybrid should be implemented.

## Required before TDI-21.1 freeze

- Complete deterministic fixtures for keyed Boolean fact recall, copy-after-marker, two-fact conjunction retrieval, and distractor rejection.
- Define exact sequence-length, state-width, memory-slot, task-count, and development-seed grids.
- Define B0/B1 baseline adapters without making them callable from B2-B5 candidate code paths.
- Define matched resource envelopes and separate native counters for unlike mechanisms.
- Add machine-readable run provenance tied to exact git SHA.
- Define development, validation, and future untouched holdout boundaries.
- Define explicit acceptance/rejection criteria before comparative runs.
- Run formatting, Clippy, and workspace tests on the final branch head.

## Downstream boundary

FLAT-ATTENTION hybridization remains downstream. A future elastic engine may compare Boolean-only, softmax-only, and Boolean-pre-route-to-restricted-softmax modes only after TDI-21 produces reproducible evidence for a Boolean candidate.
