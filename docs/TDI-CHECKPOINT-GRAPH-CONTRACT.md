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
- which declared step output is a checkpoint and which input may resume from one.

`Memorithm/scirust-hub` owns generic orchestration. TDI does **not** implement another scheduler, retry engine, lease manager, remote worker pool, cancellation service or artifact registry in this lot.

The thin Hub compiler is pinned to the audited Hub source commit `4bf6186841e1ea70ed15cd84faf33de9b48429cd`, `WorkflowSpec` schema version `1`, and workflow model version `1.2.0`. A later Hub contract change requires an explicit TDI adapter/version update rather than silent reinterpretation.

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

The graph is deliberately a **topological declaration**, not a scheduler. Each step may depend only on a step already declared earlier. This gives a simple fail-closed acyclicity rule while leaving actual ready-set scheduling and parallel execution to Hub.

Each step binds:

- stable step key;
- component alias;
- Hub-compatible capability name;
- canonical parameters;
- immutable external artifact inputs and/or outputs of earlier steps;
- declared output labels;
- explicit `after` dependencies;
- timeout in integer milliseconds;
- checkpoint policy: `none` or `exact`.

Graph identity is domain-separated as `tdi-execution-graph/v1`. Each step receives a `tdi-execution-step/v1` identity bound to the root TDI execution-plan identity and its canonical step definition. Any semantic change to a step therefore invalidates an old checkpoint for exact resume.

Graph parameters intentionally reject JSON floating-point values and unsafe integers. Domains needing floating-point parameters must first define an explicit stable string representation, matching the ExperimentSpec numeric policy.

## Hub compilation

`compile_hub_workflow()` produces a Hub `WorkflowSpec/v1` object only. It performs no network call and no execution.

Bindings are explicit:

- `component_alias -> Hub ComponentId`;
- external content SHA-256 -> Hub ArtifactId.

Hub IDs must be canonical lowercase hyphenated UUIDs. Capability names are checked against the audited Hub grammar: one or more dot-separated `[a-z][a-z0-9_]{0,63}` segments. TDI output labels also follow Hub's short printable/no-whitespace boundary.

The compiler emits no retry policy. A missing retry policy means one attempt in the audited Hub model. Distributed leases, fencing tokens, liveness and authoritative publication are a later Hub integration lot and are not inferred from this local graph contract.

## Checked-in fixtures

- `docs/examples/checkpoint-manifest-v1.json`
- `docs/examples/execution-graph-v1.json`

The fixture graph pins the Hub source/model contract and demonstrates `prepare -> evaluate`, with a content-addressed external input and an exact checkpoint output. These are software fixtures, not scientific populations or benchmark evidence.

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

The tests cover canonical identity, state/RNG sensitivity, exact lineage binding, frozen progress budgets, input/RNG drift, graph order/cycle rejection, output references, checkpoint port declarations, language-independent parameters, Hub capability grammar, canonical Hub UUIDs and the thin workflow compiler.

Passing these tests establishes only the software contract. A real distributed TDI↔Hub edge requires a separate versioned capability plus Hub-side tests and fencing/authoritative-publication qualification.
