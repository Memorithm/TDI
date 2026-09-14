# TDI-21.0 — Causal Boolean stream references

Status: **development-only software reference; not frozen; no confirmatory execution**.

This increment follows the [bootstrap audit](TDI-21.0-AUDIT-20260914.md). It implements a genuine event sequence rather than calling an immediate write/read helper and naming it long-delay recall. It is still a hand-written reference memory, not learned semantic addressing or a trained model.

## Candidate contract

`BooleanStream::step(Event)` receives exactly the current event. Its signature has no expected answer, oracle object, future event sequence, similarity scores or callback capable of substituting an attention engine.

Events are `Write { key, payload, marker }`, `Recall { key }`, `Conjunction { left, right }` and `Ignore`. Full identities are 64-bit. Payload width is declared in 1..64. Marker bit 0 enables a write and marker bit 1 inhibits it; any higher marker bit is invalid. An inhibited write still consumes a sequence event and a clause evaluation but does not overwrite memory.

A reply distinguishes `Hit(BooleanState(0))` from `Miss`. Conjunction is the packed Boolean AND of two available payloads; an absent operand yields `Miss`, not a guessed false value. This intentionally does not infer partial answers from one known false operand.

Malformed input and an exhausted event budget return a typed error without mutating memory or counters. Resource bounds are checked before allocation: 1..4096 entries, payload width 1..64 and 1..1,000,000 events. Two-way mode requires an even entry count. Construction is fallible; ordinary steps and reset allocate no new buffers. Reset clears memory, replacement state and counters and starts an independent episode. Initialization/reset cost is O(slots), not hidden in a per-event constant-time claim.

## B2 and B3

B2 uses one slot at the direct bucket address. B3 uses two slots in one addressed bucket. Both use the versioned invertible route from the audit; finite bucket reduction still causes capacity collisions. Each occupied slot is checked using the full identifier, separately counted from token-pair similarity.

B3 first updates an existing matching identifier, otherwise fills the first empty slot. A full bucket evicts its round-robin victim and flips the one-bit replacement cursor. Reads and in-place updates do not change that cursor. A memory access inspects at most two slots; a conjunction performs two reads and inspects at most four. There is no scan over prior sequence events.

`memory_collisions` in this runtime means a full addressed bucket requiring eviction, not every pair of identifiers that share a bucket. `slot_probes` counts inspected slots including empty slots; `tag_equality_checks` counts occupied-slot comparisons. Address derivations, literal evaluations and instrumented word operations retain separate categories. These are declared semantic work categories, not CPU instructions, FLOPs, bandwidth or energy measurements. Input validation and evaluator execution are not inferred from them.

## Memory accounting

Each entry's semantic payload is 64 identifier bits + 64 stored Boolean bits + 1 occupancy bit, even when the logical payload width is smaller. B3 additionally uses one replacement bit per bucket. With four entries, the memory substrates therefore contain 516 semantic bits for B2 and 518 for B3. These are not matched total architecture budgets.

`MemoryFootprint` also exposes inline object bytes and actual reserved-buffer capacity bytes, using Rust's target-dependent layouts. This covers the reference object's configuration/counters/vector metadata and buffers, but excludes allocator bookkeeping, temporary execution stack, evaluator oracle, input/output traces, executable code and other process memory. Never relabel this as peak RSS, VRAM or total model state.

## Development validation

`tdi21_stream_contract` checks causal prefixes, absent versus stored-zero values, updates, independent episodes, a 4,096-event delay, marker rejection, error atomicity, capacity/width/budget validation, direct-memory differential behavior, and all 7^4 = 2,401 length-four streams per mode over a seven-event alphabet. These 4,802 bounded software cases use an evaluator-owned ordered dictionary oracle, not the candidate's route, clauses or storage. They are development enumerations, not unseen validation or a proof over arbitrary language tasks.

The `tdi21_development` executable publishes three deliberately small fixtures for both modes. The expected software behavior is fixed in its source before execution:

- delay: one stored zero survives 4,096 `Ignore` events in both modes;
- pair: identifiers 1 and 5 collide in B2 with four entries; B2 loses the first, while the two-way bucket can retain both;
- saturated: identifiers 1, 5 and 9 exceed one B3 bucket's two-entry capacity; B2 retains one and B3 retains two, while the independent oracle still remembers all three.

These deliberately constructed outcomes illustrate capacity, not attention superiority. They do not compare equal bit budgets or trained architectures. Failed retrievals are retained in the output rather than replaced by hidden fallbacks.

Run from a clean checkout:

```bash
bash scripts/check-tdi21-development.sh
```

The script binds source SHA to the checked-out revision, reports rustc metadata and source hashes, runs audit/stream tests in debug and release, executes the development example twice and compares exact output. The new CI workflow checks out the PR's exact head with read-only permissions. Existing format/Clippy/MSRV/workspace gates remain unchanged and applicable. CI logs, not the presence of assertions in source, determine whether validation passed.

The example uses public fixed fixtures and no random sampling or training. Its evidence v2 `development_seed=0` is a fixed sentinel, not a claim of an independent random population. `state_width_bits` there denotes declared logical payload width; full identity and allocated storage widths are separately documented here. The source SHA is syntax-checked by the formatter and independently supplied by the clean-checkout script, not authenticated by a free-form string alone.

## Remaining scientific work

B0/B1 attention adapters, exact-dictionary/no-memory competence controls, learned logical encoding/routing, matched training and total memory/compute envelopes, held-out task families, causal ablations, scaling and physical benchmarks remain unimplemented. Do not freeze TDI-21.1 by merely populating template fields. No final holdout is read or authorized here.

The future FLAT-ATTENTION hybrid must be evaluated separately: Boolean-only, softmax-only or selective cooperation, including switching cost and the cost of retaining or reconstructing numerical KV state. BooleanLab/SciRust promotion requires a generic, versioned, independently qualified interface rather than copying this task-specific prototype.
