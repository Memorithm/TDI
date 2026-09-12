# TDI-8.1 status

- Scientific series: TDI-8.x
- Stage: TDI-8.1 bounded deterministic reference evaluator
- Status: **active** — reference arms A0/A1/A2/A3, symbolic generators, leakage-safe execution/encoding/readout, concrete adapters (including qualified A3), exact semantic operation accounting, typed rejection provenance, and a fail-closed **configuration freeze registry** are merged on `main`. All **17** scientific freeze fields remain `unresolved_blocking`; no experimental values have been selected yet.
- Parent programme issue: #87
- TDI-8.0 parent merge: `24d41eb7e5d72fc3b5eec9b6434930b10c1f241f`
- Frozen TDI-8.0 preregistration blob: `fe80e7053d89824a77ef6790794f6930d1b424e2`
- Configuration freeze contract: `docs/tdi8.1-configuration-freeze.json` (schema `tdi8.1-configuration-freeze-v1`, PR #176)
- Final holdout: does **not** exist
- Confirmatory runner: does **not** exist
- Human confirmation token: does **not** exist
- TDI-8.2 seed range / result surface: does **not** exist
- `tdi8_2_execution_authorized`: **false** (hard)
- TDI-7.2 interaction: **forbidden**

## What is merged (infrastructure only — not H8-A/H8-B evidence)

### Reference arms and memory primitives

- A0 competent deterministic full-history contextual reference (PR #103)
- A1 bounded recurrent-state-only reference (PR #94 lineage)
- A2 recurrent + deterministic associative memory (PRs #91/#94/#125/#169)
- A3 A2 + bounded VSA/holographic workspace with separated VSA/A2 read routing (PRs #96/#97/#127)
- Cross-mechanism atomic A2+VSA store primitive (PRs #132/#136)
- Exact matched dynamic-memory accounting contracts for A1/A2/A3; A0 cumulative history reported separately (PR #108 hardening)

### Tasks, encoding, execution boundary

- Symbolic T1/T2/T3 generators with horizon labels and fail-closed guards (PR #104)
- Leakage-safe symbolic execution contract; targets/source indices/T3 collision classes stay evaluator-owned (PR #110)
- `TaskPrediction::Invalid` remains in the quality denominator (PR #115)
- Leakage-safe binary64 encoding + exact target-blind readout (PRs #112/#114/#167)

### Concrete adapters (qualified software oracles)

- A0/A1 concrete adapters (PR #117) extracted to reusable `tdi-ai` (PR #168)
- Transactional A2 adapter (PR #125) extracted (PR #169)
- Bounded A3 symbolic task/VSA adapter policy qualified (PR #138) and extracted (PR #172)
  - association/payload writes via atomic `step_skip_vsa_and_store`
  - distractors: VSA `Skip`, neutral non-query A2 read, no VSA store
  - queries: keyed VSA + logical A2 keys, never write
  - fixture dual-path oracle only — **not** a freeze of dimensions/gains/seeds/budgets

### Decision, uncertainty, provenance, accounting

- Frozen nine-cell primary decision rules transcribed to `tdi-bench::decision_v8` (PR #106)
- Paired-resampling foundation (PR #109) + conservative percentile interval **candidate** (PR #113) — interval method/replicates/seed/degenerate policy still unresolved in the freeze registry
- Exact semantic operation accounting + observed accumulation for A0/A1/A2/A3 (PRs #160/#166/#173/#174)
- Typed symbolic rejection provenance with stable codes (PRs #162/#175)

### Configuration freeze contract (non-executing)

PR #176 added `docs/tdi8.1-configuration-freeze.json`:

- every registered scientific choice starts as `unresolved_blocking` with `value: null`
- `scientific_status` cannot become `frozen_nonfinal` until all registered choices are explicit
- TDI-8.2 authorization / holdout / confirmatory runner / human token remain hard-false
- validator + CI reject accidental TDI-8.2 seed/result/confirmation surfaces

This contract **does not select** dimensions, seeds, budgets, horizons, or sample counts.

## Remaining TDI-8.1 work (ordered)

1. Using only admissible **Development/Validation** (non-holdout) evidence, propose and review concrete values for the 17 freeze fields in `docs/tdi8.1-configuration-freeze.json`, including at least:
   - recurrent dimension/parameters and readout coordinates
   - A2 capacity/projection
   - A3 VSA width/role seed/fusion and event store/read/cleanup policy
   - matched dynamic-memory budget
   - Short/Medium/Long numeric horizons
   - late-retrieval deficit, intervention sites/recovery observable, closed rejection taxonomy
   - paired interval method + replicate count + seed + degenerate-replicate policy
   - development / validation / final population domains and sample counts (final domain rules only — **no** final seed list or result payload)
2. Content-address and CI-qualify the completed non-final freeze on its exact head before treating it as authoritative.
3. Add a final TDI-8.1 readiness/integrity gate proving every experimental choice is frozen and that **no** TDI-8.2 executable, seed/result payload, confirmation token, or authorization surface exists.
4. Keep `docs/TDI-8.1-STATUS.md` synchronized with merges (this file).

Do **not** invent freeze values without reviewable Dev/Val evidence. Do **not** authorize model/holdout execution from numbering alone.

## Holdout boundary

TDI-8.2 remains future **human-only** and is **not** authorized by this status file. Autonomous agents must never supply or infer a confirmation token and must never initiate a confirmatory run.

TDI-8 work must not read, generate, reuse, or modify TDI-7.2 final-holdout data, seeds, runner authorization state, or confirmation surface.
