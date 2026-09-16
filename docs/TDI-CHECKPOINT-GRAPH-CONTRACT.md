# TDI checkpoint and execution-graph contracts

Status: **engine software contract; Development/Validation infrastructure only; no scientific-stage authorization**.

This Lot-E contract extends the qualified ExperimentSpec/worker-response foundation with exact checkpoint lineage and a declarative multi-step graph. It does not change any frozen TDI protocol, holdout, future-entropy rule, scientific verdict, or protected/final execution boundary.

## Ownership boundary

TDI owns the scientific/execution meaning that must survive a restart:

- experiment, plan, trial and step identity;
- immutable input identities;
- causal progress and frozen logical budgets;
- RNG stream coordinates and counters;
- adapter/backend identity;
- content-addressed checkpoint state;
- exact Hub component/capability pins required by a step;
- which declared step output is a checkpoint and which input may resume from one.

`Memorithm/scirust-hub` owns generic orchestration. TDI does **not** implement another scheduler, retry engine, lease manager, remote worker pool, cancellation service or artifact registry in this lot.

The Hub bridge is pinned to audited Hub source commit `4bf6186841e1ea70ed15cd84faf33de9b48429cd`, `WorkflowSpec` schema version `1`, and workflow model version `1.2.0`. A later Hub contract change requires an explicit TDI adapter/version update rather than silent reinterpretation.

## CheckpointManifest/v1

A checkpoint manifest binds:

- `experiment_id`, `plan_id`, `trial_id`;
- graph `step_key` and content-derived `step_identity`;
- adapter and backend identity;
- the full-width seed as canonical decimal text;
- checkpoint ordinal;
- the frozen `max_steps` and `max_observations` budget values for the owning step/trial;
- completed steps and completed observations;
- named RNG stream identities plus canonical decimal counters;
- named immutable input artifact SHA-256 identities;
- one content-addressed state artifact (`sha256`, media type, byte length).

`checkpoint_identity()` hashes the complete canonical manifest under the domain `tdi-checkpoint/v1`. Named input/RNG sets are sorted by name before hashing, so list order is not semantic. Frozen budgets are part of the identity; changing them creates a different checkpoint identity. State bytes are not embedded in the manifest: their SHA-256 is authoritative.

### Exact resume

`validate_resume()` is fail-closed. It accepts a checkpoint only when all caller-owned lineage is identical:

- experiment, plan, trial, step and step-definition identity;
- adapter/backend identity;
- seed;
- frozen step/observation budgets;
- immutable input artifact names and hashes;
- RNG stream names and stream identities.

The manifest is invalid if recorded progress exceeds its own embedded frozen budgets. Resume also requires those embedded budget values to equal the plan-derived `max_steps_per_trial` / `max_observations_per_trial` supplied by the owning engine boundary; a wider caller limit cannot silently reinterpret an existing checkpoint. RNG counters and checkpoint state may advance between checkpoints, but changing a stream identity is not resume.

An accepted checkpoint is **not** permission to retry, resume a protected/final run, or bypass a series gate. Authorization remains external to this software contract.

## ExecutionGraph/v1

The graph is deliberately a **topological declaration**, not a scheduler. Each step may depend only on a step already declared earlier. This gives a fail-closed acyclicity rule while leaving actual ready-set scheduling and parallel execution to Hub.

Each step binds:

- stable step key;
- component alias;
- exact Hub `ComponentId`;
- exact component version;
- exact content digest of the component manifest expected by TDI;
- Hub-compatible capability name;
- exact capability contract version;
- canonical parameters;
- immutable external artifact inputs and/or outputs of earlier steps;
- declared output labels;
- explicit `after` dependencies;
- timeout in integer milliseconds;
- checkpoint policy: `none` or `exact`.

One component alias may not resolve to different component ID/version/manifest pins in the same graph. Graph identity is domain-separated as `tdi-execution-graph/v1`. Each step receives a `tdi-execution-step/v1` identity bound to the root TDI execution-plan identity and its complete canonical step definition. Changing component ID, version, manifest digest or capability contract version therefore changes the step identity and invalidates an old checkpoint for exact resume.

Graph parameters reject JSON floating-point values and unsafe integers. For the audited Hub v1/default model, Graph/v1 also enforces the relevant structural admission limits before preview compilation: at most 32 inputs, canonical serialized parameters at most 16 KiB, timeout at most 3,600,000 ms, Hub input-name grammar, at most 1024 graph steps and concurrency at most 64. These are the defaults at the pinned Hub source commit, not a claim about every future/deployment-specific Hub configuration.

## Hub structural preview — not execution authorization

The audited Hub `WorkflowSpec/v1` carries a `ComponentId`, but normal workflow submission resolves the latest registered manifest for that component. It does not atomically carry/enforce the component version + manifest digest pins required for TDI exact reproducibility. Therefore Lot E does **not** claim that a raw WorkflowSpec/v1 is an executable exact TDI plan.

`compile_hub_workflow_preview()` returns a versioned envelope containing:

- `execution_authorized: false`;
- a Hub `WorkflowSpec/v1`-shaped structural preview;
- exact component alias/ComponentId/version/manifest-digest pins;
- exact per-step capability contract-version pins;
- the pinned Hub source/model contract.

The `ComponentId` is part of Graph/v1 itself; it is not supplied later through an alias binding that could change without changing step identity. External content SHA-256 values are separately mapped to Hub `ArtifactId` values for preview materialization.

Hub IDs must be canonical lowercase hyphenated UUIDs. Capability names are checked against the audited Hub grammar: one or more dot-separated `[a-z][a-z0-9_]{0,63}` segments. Versions intentionally match the pinned Hub `Version::parse` grammar, which is semver-shaped rather than full SemVer: three numeric core components and an optional non-empty ASCII prerelease containing only alphanumeric, `.` or `-`. TDI does not impose stricter SemVer rules that would reject a version the pinned Hub itself accepts. Workflow names match Hub's 1..128-byte and no-NUL boundary. Output labels reject whitespace and all Unicode `Cc` control characters, matching Hub's `char::is_control()` boundary.

Even after the structural limits above are checked, the preview **must not be submitted as authoritative TDI execution**. Current WorkflowSpec/v1 cannot atomically enforce the component-version/manifest/capability pins, and an actual Hub instance may be configured with limits that differ from the audited defaults. A later executable edge must query/validate the authoritative Hub registry and limits before submission.

The external TDI SHA-256 -> Hub ArtifactId mapping is likewise a caller-supplied structural binding, not an attestation that Hub's stored bytes or its domain-separated `ContentDigest` equal the TDI digest namespace. The real edge must verify authoritative Hub artifact metadata/bytes before execution.

The preview emits no retry policy. Distributed leases, fencing tokens, liveness and authoritative publication are later Hub integration work and are not inferred from this local graph contract.

## Checked-in fixtures

- `docs/examples/checkpoint-manifest-v1.json`
- `docs/examples/execution-graph-v1.json`

The fixture graph pins the Hub source/model contract, ComponentId, component version/manifest digest and capability contract versions, and demonstrates `prepare -> evaluate`, with a content-addressed external input and an exact checkpoint output. The checkpoint fixture also carries its frozen step/observation budgets. These are software fixtures, not scientific populations or benchmark evidence.

## Qualification

Non-privileged qualification is:

```bash
PYTHONPATH=scripts python3 -m py_compile \
  scripts/tdi_checkpoint_contract.py \
  scripts/tdi_execution_graph.py
PYTHONPATH=scripts python3 -m unittest \
  scripts/test_tdi_checkpoint_contract.py \
  scripts/test_tdi_execution_graph.py -v
```

The current targeted suite contains 20 tests: 6 checkpoint tests and 14 graph tests. It covers canonical checkpoint identity, state/RNG/budget sensitivity, exact lineage binding, embedded frozen progress budgets, caller-budget reinterpretation rejection, input/RNG drift, graph order/cycle rejection, output references, checkpoint ports, language-independent parameters, exact Hub source pinning, component-ID/version/manifest/capability sensitivity, alias-pin consistency, audited Hub input/parameter/timeout bounds, canonical UUIDs, workflow-name NUL rejection, pinned Hub prerelease-version behavior, Unicode control rejection and the non-executable structural workflow preview.

Passing these tests establishes only the software contract. A real distributed TDI↔Hub edge requires a separate versioned capability plus authoritative component/artifact verification, exact deployed Hub-limit checks, Hub-side tests, fencing and authoritative-publication qualification.
