# TDI AI: development experiment hardening

This engineering change addresses the audit of the reusable `tdi-ai` core.
It builds on main `b4151ad`, which already contains the TDI-11 controlled-world
oracle, non-final generator, exact evaluator, prospective observation timeline,
leakage-safe observation adapter and the TDI-11.2 pre-arm gate.

## Changes and evidence

| Audit issue | Implementation | Qualification |
| --- | --- | --- |
| Clone does not imply isolated state/cache/RNG | `ReplayAdapter`, independently forked paired sessions, explicit stream identities, deterministic bridge | Independent fixture, deliberately shared `Rc<Cell<_>>` counterexample, noise coupling tests |
| Unbounded loops/retention and opaque failure | Validated step/point limits, fallible reservation, stride/no-retention collection, streaming sink, cancellation, depth/branch/stage diagnostics | Limit, capacity, cancellation, metric/backend/sink failure tests |
| Ambiguous raw profile horizon and invalid values | Additive `ValidatedProfile`, positive increasing depths, distinct observation count/last depth, explicit finite/unit-interval score validation | Ordering, sparse depths, NaN and domain tests |
| Snapshots can accept overflowing totals | Complete dynamic plus static accounting validation at snapshot construction | Component-legal overflow rejection tests |
| Non-query A2 updates require a synthetic lookup key | `step_without_read`, optional write and no lookup | Resident-memory independence and rejected-input atomicity |
| TDI-9 failures discard consumed effort | Diagnostic evaluator and last accounted progress in typed rejection records | Compatibility equality and decision-limit cost tests |
| Experimental code awkward to consume externally | `experimental` Cargo feature with explicitly unstable namespaced TDI-9 APIs | External integration tests, example, all-feature CI/MSRV |
| Weak experiment identity | Required provenance fields and unambiguous canonical encoding; policy float parameters use binary64 bits | Order-independent encoding, parameter distinction and invalid identity tests |
| Rejected trials disappear or get rerun during resume | Bounded ordered campaign, immutable result prefix, generator/seed/evaluator rejection retention, same-plan in-memory resume | Paused vs uninterrupted equality; plan drift and wrong generator seed rejection |
| Logical storage confused with hardware evidence | Separate optional `PhysicalTelemetry` | No automatic conversion or performance inference |
| Missing TDI-11 foundation in original audit snapshot | Reuse merged TDI-11 modules and their existing qualification gates | TDI-11 bootstrap and existing crate tests |

## Use

From a clean committed checkout:

```bash
cargo test -p tdi-ai --all-features
bash scripts/check-tdi-ai-experiments.sh
cargo run -p tdi-ai --features experimental --example development_campaign -- "$(git rev-parse HEAD)"
```

The example uses three deliberately small software fixture seeds. Its output is
engineering evidence, not a scientific population, effect estimate or promotion.
`CampaignPlan::canonical_record` binds source/dependency/backend/generator/metric/
accounting declarations, exact policies, resource envelope, decision limit and
ordered domain/indices. Outcomes follow that same policy order. Technical
rejections are distinct from completed task-quality outcomes.

## Contracts and remaining boundaries

* Fork independence remains an adapter obligation. Conformance fixtures can find
  violations but cannot prove arbitrary opaque backends independent. Checkpoints
  must be value snapshots; pointer equality is insufficient. RNG state, caches,
  threading and nondeterministic kernels require backend-specific qualification.
* Noise stream IDs are not RNG implementations. Paired and deterministic modes
  both use stream zero; the backend determines whether noise actually exists.
* Cancellation is cooperative between steps (paired runner) or complete paired
  trials (campaign). Blocking calls require backend deadlines. These limits do
  not bound arbitrary allocations inside user adapters, generators or sinks.
* The campaign resume API is in-memory. It neither promises crash-safe disk
  checkpoints nor restores half-completed solver sessions. Callers can persist
  the canonical plan and records, but importing resumable records is not exposed.
* Manifest fields are caller declarations, not cryptographic attestations. Supply
  exact artifact identities and verify them outside this library. A valid commit
  string alone does not establish a clean checkout. The generic Development /
  Validation namespace does not replace any frozen series-specific derivation.
* Progress records report observed accounting, including charged policy work
  before a failed action. They do not invent costs for operations the reference
  meter rejected before committing, nor convert logical work to physical time.
* `step_without_read` is an additive primitive. Existing frozen reference paths
  preserve their semantics. Integrating a different transition into an evaluator
  requires its own operation-accounting and causal qualification.
* Hardware telemetry is optional and externally measured. No GPU/Jetson speedup,
  memory saving, model-quality improvement or external-repository integration is
  established by these deterministic tests.
* TDI-11.2 concrete model runner implementation/execution is blocked by the
  merged pre-arm gate until the exact model/observation freeze is complete.
  This patch does not select a model or resolve scientific blockers by fiat.
* No historical or final-evaluation material is accessed or generated. Existing
  frozen protocols, outcomes and scientific promotion criteria remain binding.

The feature facade is an engineering development surface, not stabilization of
the qualification-only TDI-9 APIs. Default builds do not expose these modules.
