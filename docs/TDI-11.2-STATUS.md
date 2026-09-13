# TDI-11.2 Status

Status: **ACTIVE PRE-ARM — MODEL EXECUTION NOT AUTHORIZED**

## Current state

TDI-11.1 has integrated the deterministic non-final controlled-world reference foundation through PR #157. TDI-11.2 is now preparing the contract required to instrument prospective precursor signals on a concrete local/open model.

This stage is deliberately split so that repository preparation cannot silently become model experimentation.

## Current authorized work

At this status, authorized work is limited to:

- TDI-11.2 protocol and gate documents;
- machine-readable pre-arm state;
- integrity/CI checks;
- non-executing adapter/schema scaffolding;
- non-executing model-observation rejection vocabulary + provenance scaffolding
  (candidates only; does not pin freeze fields);
- deterministic tests proving that unresolved model-observation fields keep execution blocked.

## Current blocking fields

Concrete model execution remains blocked until the repository freezes exact values for:

- model artifact;
- adapter version;
- tokenizer/template;
- decoding/generation configuration;
- prompt serializer;
- observation registry;
- observation event timing;
- prospective eligibility rule;
- Development/Validation population derivation;
- resource accounting;
- typed rejection;
- provenance.

The machine-readable source of this pre-arm state is `docs/tdi11.2-prearm.yaml`.
The content-addressed unresolved ledger (not a pin) is `docs/TDI-11.2-UNRESOLVED-LEDGER.md`
with sidecar `docs/tdi11.2-model-observation-freeze.sha256`.
Always-on readiness: `scripts/check-tdi11.2-readiness.sh` (workflow `tdi11.2-readiness.yml`).

## Forbidden state transitions

The following are invalid at this stage:

- invoking a concrete model from a TDI-11.2 runner;
- adding a runnable TDI-11.2 model-experiment surface while any blocking field remains unresolved;
- producing a final seed list, final dataset or final result;
- selecting an observation vector after seeing validation/final outcomes and presenting it as preregistered;
- using hidden world truth or evaluator labels as model-facing signals.

## Next transition

The next transition is a separately reviewed TDI-11.2 model/observation freeze. That transition must pin exact non-final Development/Validation identities and settings, content-address the frozen contract, pass its integrity gate on the exact PR head, merge to `main`, and update the agent roadmap separately.

Only then may a concrete local/open model instrumentation runner be implemented or executed.

## Agent advance boundary

Autonomous agent advances may strengthen integrity/CI/docs/scaffolding that keep
execution blocked. They must **not** invent model/adapter/tokenizer pins or set
`model_execution_authorized` / `final_execution_authorized` true while any of
the 12 freeze fields remain `unresolved_blocking` (orchestrator holdout / no
AUTO_MERGE of freeze 11.2: #186). The existing pre-arm and freeze-schema gates
already enforce this fail-closed posture.

Fail-closed reminder (unchanged authorizations): both execution flags remain
**false** until a separately reviewed freeze pins all twelve blocking fields.
Post-#204 scout found no model/adapter/tokenizer identifier that can be pinned;
**0/12 pinned** (STATUS↔freeze JSON pin-count cross-check in readiness). All
12 fields stay `unresolved_blocking`. The readiness gate fail-closes if
`model_execution_authorized` is true while any field is `unresolved_blocking`,
if any of the 12 fields is invented as pinned, if STATUS↔JSON pin counts
diverge, if the unresolved-ledger digest drifts, or if a forbidden surface
appears (#199 floors + #202-style cross-check parity with 8.1/9.1).

Machine-readable companions (CI-verified; not pins):

- non-executing rejection + provenance scaffolding (candidate only; **not a pin**):
  `docs/TDI-11.2-MODEL-OBSERVATION-REJECTIONS.md`,
  `tdi-ai/src/hallucination_model_observation_rejections.rs`,
  `scripts/check-tdi11.2-model-observation-rejections.sh`
- blocker evidence-class inventory: `docs/tdi11.2-blocker-evidence-classes.json`
- freeze progress summary: `docs/tdi-freeze-progress-summary.json` (includes
  verified ledger digest `dec8bd504a903b0125fdb88f66821940d286358eec6d59ee1a194fc6c63e9421`)

Orthogonal TDI-10.x through TDI-10.19 / TDI-12.0 Stage-0 must not invent those
pins or contact TDI-7.2 / TDI-8.2 / TDI-9.2 surfaces. TDI-10.19 reverse/cross
family composition does not pin typed_rejection_contract / provenance_contract.
Post-#212 diversification: rejection/provenance scaffolding remains
non-authorizing; pins stay **0/12**.

