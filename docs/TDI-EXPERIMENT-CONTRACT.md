# TDI ExperimentSpec and worker-response contracts

Status: **engine software contract; Development/Validation only; no scientific-stage authorization**.

This document describes the schema-3 execution path added by the engine industrialization programme. It does not alter frozen TDI protocols, populations, margins, results, holdout rules or future-entropy gates. Schema 1 and schema 2 remain supported for their qualified scopes.

## Identity layers

TDI intentionally distinguishes five identities:

1. **Question identity** — semantic protocol fields: series/stage, protocol references, hypotheses, metrics, data derivation, randomness declaration and statistical plan.
2. **ExperimentSpec plan identity** — the complete `ExperimentSpec/v1`, including arms, adapter, logical budget, physical constraints, retry and artifact policies.
3. **Execution plan identity** — the complete schema-3 runnable plan, including explicit indices, pinned executable/input artifacts and cgroup profile, bound to the ExperimentSpec plan identity.
4. **Trial / attempt identity** — stable coordinates derived from the execution plan, domain, index, backend and attempt ordinal.
5. **Scientific result identity** — canonical scientific disposition/result plus the worker's content-addressed artifact references, bound to experiment/plan/trial identity. Artifact order is canonicalized by name. Operational progress and measured resource telemetry are deliberately excluded.

A timestamp, measured RSS or progress-cost counter therefore does not change the scientific result identity. Changing an artifact content hash does. Changing a metric unit, protocol reference, generator derivation or statistical plan changes the question identity. Changing only a physical memory bound keeps the question identity but changes both plan identities.

## Canonical numeric policy

`ExperimentSpec/v1` uses canonical sorted UTF-8 JSON for hashing. Numeric identity fields are restricted to the JavaScript-safe integer range (`<= 2^53-1`) so a client cannot silently round a plan identity. Full-width u64 seeds are transported as canonical decimal strings in worker-response/v2.

Schema-3 execution-plan identity and journal binding do not distinguish JSON spellings such as `2` and `2.0` when they describe the same validated timeout. The authoritative `ExperimentSpec/v1` integer `timeout_milliseconds` is substituted into the canonical binding as `<milliseconds>ms` only after the inherited runner timeout has been validated against it. This normalization changes neither the requested timeout nor schema-2 compatibility; it prevents semantically identical cross-language JSON round trips from producing different execution identities.

Worker-response/v2 does **not** permit JSON floating-point values inside the canonical `result` object. A domain needing floating-point scientific output must define an explicit representation (for example a frozen hexadecimal or decimal-string format) before those values participate in canonical identity. This restriction avoids making an experiment identity depend on language-specific float rendering.

## ExperimentSpec/v1

The complete example is [`docs/examples/experiment-spec-v1.json`](examples/experiment-spec-v1.json). Required sections are:

- semantic version, series and stage;
- content-addressed protocol references;
- hypothesis IDs and metric definitions with units/orientation;
- authorized data class, generator identity and partition derivation;
- RNG algorithm, namespace and stream-derivation declaration;
- arms and implementation identities;
- adapter API/implementation, capabilities and reproducibility class;
- logical budgets;
- physical constraints;
- retry policy;
- statistical-plan and missing-observation policy;
- artifact access/retention/export policy;
- dependency identities.

The schema has no implicit scientific defaults. The generic `Development` / `Validation` domains remain an engine namespace and do not replace any series-specific frozen seed derivation.

## Worker-response/v2

A successfully started schema-3 worker receives these identity arguments in addition to `--tdi-seed` and `--tdi-plan-id`:

```text
--tdi-worker-protocol 2
--tdi-experiment-id <sha256>
--tdi-trial-id <sha256>
--tdi-attempt-id <sha256>
--tdi-domain Development|Validation
--tdi-backend-identity linux-cgroup-v2
```

The worker returns exactly one JSON object containing:

- `schema: 2`;
- `execution_status: "completed"` for a process that completed the worker protocol;
- `scientific_disposition: "evaluated" | "rejected"`;
- all caller-owned identities echoed exactly;
- `seed_decimal` as canonical unsigned decimal text;
- bounded progress counters `completed_steps` and `completed_observations`, plus bounded auxiliary cost counters;
- content-addressed artifact references with access class;
- canonical result object or `null`;
- structured error or `null`.

`progress.completed_steps` and `progress.completed_observations` are authoritative counters checked respectively against the frozen `ExperimentSpec.logical_budget.max_steps_per_trial` and `max_observations_per_trial` before a response is accepted. A response exceeding either declared logical budget is a contract failure rather than a completed scientific result. Auxiliary `progress.costs` entries are telemetry and cannot substitute for either authoritative counter. Content-addressed worker artifacts are also part of `scientific_result_id`, so changing bulk scientific output cannot be hidden behind an unchanged small `result` object.

A scientific rejection is not converted into a technical worker failure. An identity mismatch, invalid serialization, over-budget progress or unsupported canonical number is a contract failure.

`tdi-ai/examples/durable_worker_v2.rs` is a deterministic infrastructure fixture that exercises the actual bounded TDI Rust API. It is not a scientific population or performance benchmark.

## Preparing a schema-3 fixture plan

The checked-in fixture protocol and ExperimentSpec are software-only. Verify the protocol content hash before use:

```bash
sha256sum docs/examples/experiment-protocol-fixture.txt
```

Expected fixture hash:

```text
542c27a6d01f11175be39122c3c052753d9512e6abcf5f14960ee146e44dadbc
```

Build the v2 fixture worker, then prepare an immutable plan:

```bash
cargo build --locked -p tdi-ai --example durable_worker_v2
python3 scripts/prepare_tdi_experiment_plan.py \
  --root . \
  --worker target/debug/examples/durable_worker_v2 \
  --artifact Cargo.lock \
  --artifact tdi-ai/examples/durable_worker_v2.rs \
  --index 0 --index 1 --index 2 \
  --domain Development \
  --timeout 5 \
  --output-limit 65536 \
  --containment cgroup-v2 \
  --memory-max-bytes 67108864 \
  --swap-max-bytes 0 \
  --cpu-quota-us 100000 \
  --cpu-period-us 100000 \
  --pids-max 16 \
  --experiment-spec docs/examples/experiment-spec-v1.json \
  --plan /tmp/tdi-engine-contract-plan.json
```

Plan creation refuses to overwrite an existing path. The actual run additionally requires a deliberately delegated cgroup-v2 parent satisfying the Lot-C contract; do not substitute the host cgroup root casually. Use `python3 scripts/tdi_linux_containment.py doctor --parent <delegated-parent>` before execution.

## Resume semantics

The durable journal binds the complete schema-3 plan. Reopening the same plan and journal retains the same trial/attempt/result identities and does not repeat completed trials. A semantically changed ExperimentSpec, changed executable hash, changed index list or changed resource plan yields a different plan binding and is refused for that journal.

An interrupted active attempt is retained as `Interrupted`; it is not silently transformed into a fresh scientific trial. Any later retry feature must honor the explicit `retry` contract and series gate rather than bypassing this lineage.

## RNG streams

`stream_identity()` derives a stable coordinate from question identity, trial identity, namespace and stream name. Reordering scheduling does not change that coordinate. This is **not** an RNG algorithm and does not prove statistical independence; adapters must still declare and qualify their actual RNG implementation.

## Compatibility

- schema 1: historical process-group durable path, unchanged;
- schema 2: cgroup-v2 containment contract from Lot C, unchanged, including its legacy 32-hex attempt identity;
- schema 3: additive ExperimentSpec/v1 + worker-response/v2 path;
- no final domain is introduced;
- no filesystem/network sandbox or hard VRAM quota is implied;
- no Hub lease/fencing behavior is introduced here; that remains a later distributed integration contract.

## Qualification

Non-privileged tests cover canonical identities, semantic/physical drift, timeout-representation normalization, retry policy, JSON-safe integers, u64 seed strings, worker bindings, logical step- and observation-budget enforcement, artifact-backed result identity, float rejection and schema-2 compatibility. The dedicated real-kernel CI job executes a synthetic schema-3 campaign inside the qualified cgroup-v2 boundary, reopens it without repeating the completed trial and verifies that semantic drift is refused.
