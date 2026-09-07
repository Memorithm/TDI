# TDI-8.1 semantic operation accounting

Status: **bounded deterministic software-accounting qualification only — not runtime evidence and not H8-A/H8-B evidence**.

Tracks #159 and the TDI-8 programme issue #87. The concrete A3 symbolic adapter was already qualified and merged in PR #138 before this accounting layer was introduced.

## Purpose

TDI-8.0 requires quality, memory and deterministic operation-count evidence to remain distinct. The existing TDI-8.1 stack already has exact metadata-inclusive memory accounting. This tranche adds a deterministic operation vocabulary for the already-merged reference semantics before concrete experimental dimensions, budgets, horizons and populations are frozen.

The counts are **reference-semantic work units**. They are not CPU instructions, FLOPs, wall-clock time, energy, bandwidth, cache traffic, GPU occupancy or a claim about any accelerator.

Validation scans, fallible allocation and Rust container bookkeeping are deliberately outside the architecture-semantic count. A later NNIS/device study must measure hardware behavior separately.

## Declared operation classes

`ReferenceOperationAccounting` records checked `u128` counts for:

- recurrent multiply-accumulate terms;
- hard-tanh activation terms;
- associative address projections;
- associative payload fusion terms;
- associative payload stores;
- VSA bind terms;
- VSA bundle additions;
- VSA unbind terms;
- VSA-to-input fusion terms;
- A0 history distance-coordinate terms;
- A0 history selection comparisons;
- A0 append scalar stores.

The reported total is the exact checked sum of those declared classes. It must not be re-labelled as a hardware-independent FLOP count.

## Exact mapping to merged reference code

### A0

For layout key width `K`, value width `V` and a successful read over `H > 0` history items:

- append: `K + V` history scalar stores;
- read distance work: `H × K` distance-coordinate terms;
- read selection: `H` comparisons because the merged loop evaluates `distance <= best_distance` once for every item, including the first.

The readout vector allocation/copy path is not counted as semantic compute in this vocabulary.

### A1

For recurrent input width `I` and state width `S`, one merged fixed-order recurrent step executes:

- `S × (S + I)` recurrent multiply-accumulate terms;
- `S` hard-tanh activations.

This transcribes the two inner loops and the final activation in `BoundedRecurrentCore::compute_next`.

### A2

Every A2 step contains the A1 work plus one associative read address projection.

If the observed `A2StepReport` is a hit:

- add `S` associative payload fusions;
- add `S` hard-tanh activations for the post-retrieval fusion.

If the report contains a write:

- add one associative address projection;
- add `S` associative payload stores.

The accounting therefore depends on the actual deterministic read/write outcome rather than on a guessed task label.

### A3 routed read

A3 `Skip` adds no VSA work to the observed A2 step.

A3 keyed read adds, for VSA/input width `I`:

- `I` VSA unbind terms;
- `I` VSA-to-input fusion terms.

### A3 atomic skip-and-store

The qualified A3 adapter uses `step_skip_vsa_and_store` on write events. VSA preparation adds, for width `I`:

- `I` VSA bind terms;
- `I` VSA bundle terms.

The prepared-state commit performs no numeric calculation and therefore adds no semantic work unit. The subsequent A2 work is counted from its actual `A2StepReport`.

## Overflow and invalid-state policy

All derived products, component accumulation and totals use checked `u128` arithmetic. Overflow fails closed. A0 read accounting rejects `H = 0`, matching the merged A0 reference where a successful content read requires non-empty history.

## Qualification boundary

This tranche does not freeze or select:

- recurrent dimensions or parameter values;
- associative capacity, projection seed or fusion gain;
- VSA width, role seed, fusion gain or final multi-item policy;
- matched dynamic-memory budget;
- Short/Medium/Long numeric horizons;
- train/development/validation/final populations or sample counts;
- late-retrieval deficit or intervention/recovery semantics;
- final paired interval configuration;
- TDI-8.2 seeds, runner, result payload or authorization surface.

It also produces no Transformer/Mamba superiority, O(N), constant-memory, latency, GPU, bandwidth or energy claim.

## Reproducibility gate

Run:

```bash
bash scripts/check-tdi8.1-operation-accounting.sh
```

The dedicated gate compiles/tests the qualification-local accounting module through the bounded preflight binary, executes the deterministic fixture and rechecks TDI-8.2 absence through the parent bootstrap chain.

The module is intentionally qualification-local in this tranche, following the same preflight pattern used earlier in TDI-8.1. Promotion into the stable `tdi-ai` public module surface should occur only when the evaluator/provenance integration consumes the reviewed vocabulary without changing its semantics.
