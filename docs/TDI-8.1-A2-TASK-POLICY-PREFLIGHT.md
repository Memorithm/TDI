# TDI-8.1 — A2 task-routing policy preflight

Status: **bounded software qualification only; not H8-A evidence; no TDI-8.2 surface**

## Why this tranche exists

PR #117 qualified concrete A0/A1 symbolic adapters and deliberately stopped before A2/A3. That boundary is correct: `A2Reference::step` requires a memory `read_key` and optional `write_key` for every recurrent event, while the architecture-neutral `SymbolicTaskAdapter` exposes only the task stimulus. The mapping between those two interfaces is architecture semantics, not generic plumbing.

This tranche makes one A2 mapping explicit, deterministic and independently auditable before any experimental dimensions, budgets or populations are selected.

## Candidate A2 routing

For each symbolic event, the candidate policy is:

| Event | Recurrent input | A2 read key | A2 write key |
| --- | --- | --- | --- |
| association `(key,value)` | qualified association frame | logical association key | same association key |
| T2 payload `value` | qualified payload frame | domain-separated call-order payload key | same payload key |
| distractor `token` | qualified distractor frame | per-instance safe distractor key | none |
| association query `key` | qualified query frame | matching logical association key | none |
| payload query `position` | qualified query frame | matching call-order payload key | none |

The underlying `A2Reference` retains its already-reviewed lookup-before-write semantics. This tranche does not alter the table, recurrence or fusion implementation.

### Association and payload writes

A2 stores the post-fusion recurrent state already defined by `A2Reference`; this policy does not invent a second payload representation. For each write event the read key and write key are identical. Under the frozen T1/T3 generators, association keys are unique within one instance; under T2, payload positions are unique. A direct-mapped collision with a different resident key therefore remains a `CollisionMiss` before deterministic replacement rather than fusing the wrong payload.

### Payload call order and atomicity

`SymbolicTaskAdapter::payload` does not receive generator-side source position. The policy reconstructs T2 positions only from successful chronological payload calls using `PayloadKeyCursor`.

Payload routing is prospective: the policy returns both the route and a proposed next routing state. The concrete adapter commits that next state only after `A2Reference::step` succeeds. A technical recurrent/memory failure cannot silently advance the logical payload position.

### Distractor route

The distractor read key is selected runner-side by the already-qualified `distractor_read_key_for_instance` helper. It is guaranteed to differ from every logical write key in that immutable task instance and is never written.

The key is routing metadata, not a recurrent input feature. Exact targets, association source indices and T3 generator collision classes are not used to construct recurrent inputs or memory keys.

## Independent physical-collision check

The preflight does not trust the adapter's own counters alone. Before execution it runs the PR #112 `audit_associative_projection` oracle against the same immutable task and concrete direct-mapped layout/projection. During execution the adapter independently records:

- number of writes;
- physical replacement collisions;
- query hits;
- query collision misses;
- query empty reads.

The preflight requires all five observed quantities to match the independent projection audit on bounded T2 and T3 software fixtures. Generator-side T3 collision classes remain separate metadata and are never substituted for physical table collisions.

## Readout and invalid outputs

Queries use the target-blind exact recurrent readout candidate qualified by PR #114. Per PR #115:

- canonical state output -> `TaskPrediction::Symbol`;
- finite non-canonical state output -> `TaskPrediction::Invalid`;
- `Invalid` stays in the declared query denominator and counts as failure;
- true encoding/readout/reference faults remain technical adapter errors.

The software fixtures deliberately make no A2 retrieval-quality claim. They verify event routing, denominator preservation and agreement with the physical projection oracle only.

## Accounting

The preflight calls the existing `A2Reference::memory_accounting` after execution. Recurrent state, associative payload, associative metadata, temporary working storage and static parameters therefore remain accounted by the merged reference implementation.

No memory quantity is converted into a runtime, bandwidth, energy or asymptotic claim.

## A3 remains unselected

This A2 policy does not select an A3 VSA task policy. `A3Reference::store_vsa(key,payload)` remains deliberately separate from `A3Reference::step`, and the merged reference assigns the decision of when and what to store to the later bounded evaluator.

A3 therefore requires its own explicit candidate defining at minimum:

- which task events write VSA state;
- the role/read key used for each event type;
- which finite vector is stored;
- ordering relative to integrated VSA read/fusion/A2 step;
- atomic failure semantics;
- exact operation and memory accounting implications.

None of those choices is inferred from this A2 policy.

## Non-freezes

This tranche does not freeze:

- recurrent dimensions or parameter values;
- A2 slot count, projection seed or fusion gain;
- recurrent readout coordinates;
- matched A1/A2/A3 dynamic budget;
- numeric short/medium/long horizons;
- train/development/validation/final seed ranges;
- population or sample counts;
- late-retrieval deficit construction;
- paired-bootstrap replicate count/seed or final interval promotion;
- intervention sites or recovery descriptors;
- closed final rejection taxonomy;
- any A3 VSA task policy.

It accesses no TDI-7.2 payload, computes no H8-A/H8-B verdict and creates no TDI-8.2 runner, token, seed range, holdout or result surface.
