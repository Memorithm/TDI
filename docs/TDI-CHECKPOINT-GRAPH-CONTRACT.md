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
- exact component/capability contract pins required by a step;
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
- completed steps and completed observations;
- named RNG stream identities plus canonical decimal counters;
- named immutable input artifact SHA-256 identities;
- one content-addressed state artifact (`sha256`, media type, byte length).

`checkpoint_identity()` hashes the complete canonical manifest under the domain `tdi-checkpoint/v1`. Named input/RNG sets are sorted by name before hashing, so list order is not semantic. State bytes are not embedded in the manifest: their SHA-256 is authoritative.

### Exact resume

`validate_resume()` is fail-closed. It accepts a checkpoint only when all caller-owned lineage is identical:

- experiment, plan, trial, step and step-definition identity;
- adapter/backend identity;
- seed;
- immutable input artifact names and hashes;
- RNG stream names and stream identities.

Recorded progress may not exceed the frozen `max_steps_per_trial` or `max_observations_per_trial` supplied by the owning ExperimentSpec. RNG counters and checkpoint state may advance between checkpoints, but changing a stream identity is not resume.

An accepted checkpoint is **not** permission to retry, resume a protected/final run, or bypass a series gate. Authorization remains external to this software contract.

## ExecutionGraph/v1

The graph is deliberately a **topological declaration**, not a scheduler. Each step may depend only on a step already declared earlier. This gives a fail-closed acyclicity rule while leaving actual ready-set scheduling and parallel execution to Hub.

Each step binds:

- stable step key;
- component alias;
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

One component alias may not resolve to different component version/manifest pins in the same graph. Graph identity is domain-separated as `tdi-execution-graph/v1`. Each step receives a `tdi-execution-step/v1` identity bound to the root TDI execution-plan identity and its complete canonical step definition. Changing component version, component manifest digest or capability contract version therefore changes the step identity and invalidates an old checkpoint for exact resume.

Graph parameters intentionally reject JSON floating-point values and unsafe integers. Domains needing floating-point parameters must first define an explicit stable string representation, matching the ExperimentSpec numeric policy.

## Hub structural preview — not execution authorization

The audited Hub `WorkflowSpec/v1` carries a `ComponentId`, but normal workflow submission resolves the latest registered manifest for that component. It does not atomically carry/enforce the component version + manifest digest pins required for TDI exact reproducibility. Therefore Lot E does **not** claim that a raw WorkflowSpec/v1 is an executable exact TDI plan.

`compile_hub_workflow_preview()` returns a versioned envelope containing:

- `execution_authorized: false`;
- a Hub `WorkflowSpec/v1`-shaped structural preview;
- exact component alias -> Hub ComponentId/version/manifest-digest pins;
- exact per-step capability contract-version pins;
- the pinned Hub source/model contract.

Bindings are explicit:

- `component_alias -> {component_id, component_version, manifest_digest}`;
- external content SHA-256 -> Hub ArtifactId.

Hub IDs must be canonical lowercase hyphenated UUIDs. Capability names are checked against the audited Hub grammar: one or more dot-separated `[a-z][a-z0-9_]{0,63}` segments. Versions follow the audited Hub version grammar. TDI output labels also follow Hub's short printable/no-whitespace boundary.

A mismatched component version or manifest digest is rejected before the preview is returned. However, because current WorkflowSpec/v1 cannot enforce those pins atomically during normal submission, the preview **must not be submitted as authoritative TDI execution**. A later versioned TDI↔Hub capability edge must enforce component/manifest/capability pins inside Hub before setting an execution authorization boundary.

The preview also does **not** attest deployment admission. The pinned Hub model applies runtime `Limits` to `RunSpec` values (including serialized parameter size, input count and timeout), while Graph/v1 is a TDI semantic contract with its own bounds. A later executable edge must validate every compiled step against the exact Hub limits in force. Consequently this Lot-E preview is not described as a Hub-admissible workflow merely because its JSON shape matches `WorkflowSpec/v1`.

The external TDI SHA-256 -> Hub ArtifactId mapping is likewise a caller-supplied structural binding, not an attestation that Hub's stored bytes or its domain-separated `ContentDigest` equal the TDI digest namespace. The real edge must verify authoritative Hub artifact metadata/bytes before execution.

The preview emits no retry policy. Distributed leases, fencing tokens, liveness and authoritative publication are later Hub integration work and are not inferred from this local graph contract.

## Checked-in fixtures

- `docs/examples/checkpoint-manifest-v1.json`
- `docs/examples/execution-graph-v1.json`

The fixture graph pins the Hub source/model contract, component version/manifest digest and capability contract versions, and demonstrates `prepare -> evaluate`, with a content-addressed external input and an exact checkpoint output. These are software fixtures, not scientific populations or benchmark evidence.

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

The tests cover canonical checkpoint identity, state/RNG sensitivity, exact lineage binding, frozen progress budgets, input/RNG drift, graph order/cycle rejection, output references, checkpoint port declarations, language-independent parameters, Hub source/capability/version grammar, component-version-sensitive step identities, component binding mismatch, canonical Hub UUIDs and the non-executable structural workflow preview.

Passing these tests establishes only the software contract. A real distributed TDI↔Hub edge requires a separate versioned capability plus authoritative component/artifact verification, exact Hub admission-limit checks, Hub-side tests, fencing and authoritative-publication qualification.
