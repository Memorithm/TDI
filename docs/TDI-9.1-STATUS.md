# TDI-9.1 status

- Scientific series: TDI-9.x
- Stage: TDI-9.1 bounded autonomous adaptive-inference evaluator
- Status: active — policy/accounting foundation, deterministic P1/P2/P3 task generation, solver/verifier/checkpoint/replay execution and bounded C0/C1/C2/C3 reference policies are merged; the next frontier is complete non-final evaluator integration plus agent-search-safe policy qualification
- Parent TDI-9.0 merge: `bb13c59aa91e3e5e2e6a480f4ae12adfe168221b` (PR #122)
- TDI-9.1 policy/accounting foundation merge: PR #123
- TDI-9.1 deterministic P1/P2/P3 task generator merge: PR #126
- TDI-9.1 deterministic reference execution merge: `6a4b7decdb044166ee3a6193108fed423913499e` (PR #128)
- TDI-9.1 bounded C0/C1/C2/C3 reference policy merge: `d5ba650e2f0abdadbe44856698a8e44d33117cb0` (PR #129)
- Frozen TDI-9.0 preregistration blob: `babad0a4e309e67e57820281a0f31284ba1e5da0`
- TDI-9.2 runner: does not exist
- TDI-9.2 final seed list: does not exist
- TDI-9.2 final dataset: does not exist
- TDI-9.2 result payload: does not exist
- Human confirmation token: intentionally absent from TDI-9
- TDI-7.2 interaction: forbidden
- TDI-8.2 interaction: forbidden

## Merged policy and resource-accounting foundation

PR #123 defines the infrastructure required by the frozen TDI-9.0 policy ladder:

- `C0FixedCompute`, `C1StaticPreallocation`, `C2AdaptiveStopping`, and `C3VerificationRecovery` identities;
- frozen `CONTINUE`, `VERIFY`, `BACKTRACK`, `STOP` action vocabulary;
- arm-level action legality, with verifier/backtracking restricted to C3;
- a deterministic `PolicyObservation` carrier containing only current/past trajectory summaries;
- C3-only verifier signal and checkpoint metadata;
- strictly positive common maximum compute/memory envelopes;
- explicit solver, verifier, policy-decision, checkpoint and replay operation accounting;
- simultaneous persistent, policy, checkpoint and temporary-peak memory accounting;
- atomic fail-closed rejection before resource-envelope overflow.

This foundation is infrastructure only and freezes no primary difficulty values, policy thresholds, non-final populations or final entropy.

## Merged deterministic P1/P2/P3 task generation

PR #126 implements the three frozen mechanistic task families behind a typed boundary between policy-visible task input and evaluator-only metadata:

- P1 staged evidence accumulation with caller-supplied non-final decisive-prefix configuration;
- P2 bounded GF(2) unit-lower-triangular parity systems with one exact satisfying vector;
- P3 recoverable deceptive fork with local decoy evidence, a later ordinary contradiction and recovery evidence;
- `PolicyTask` is separated from `EvaluatorRecord`, which retains the seed, hidden stratum, exact target and construction oracle;
- Shallow / Intermediate / Deep strata remain evaluator-only;
- generation is deterministic and domain-separated with checked event counts and fallible allocation.

The merged task-generator fixtures are software qualification only. They do not freeze concrete primary-cell difficulty values, seed populations or resource envelopes.

## Merged deterministic solver/verifier/checkpoint/replay execution

PR #128 adds the bounded deterministic execution layer between task generation and policy choice:

- deterministic P1 sequential evidence solver;
- deterministic P2 unit-lower-triangular GF(2) solver;
- P3 branch commitment under local evidence;
- explicit independent verifier with no evaluator-target input;
- one C3-only paid P3 checkpoint at the public ChoicePoint;
- verifier-gated BACKTRACK semantics retaining only the refuted live branch;
- canonical checkpoint store/restore byte accounting;
- replay operation accounting after restoration;
- atomic resource rejection across solver/checkpoint/meter state;
- current/past-only `PolicyObservation` production;
- policy-decision accounting hooks without embedding policy-choice logic;
- post-STOP evaluation through a distinct `StoppedCandidate` type.

The qualification fixture establishes the intended behavioral contrast: a C2-style P3 forward path can remain committed to the decoy, while C3 can recover only through paid VERIFY -> violated signal -> BACKTRACK -> replay. This is a software-oracle contrast, not H9-B evidence.

The live executor receives only `PolicyTask`. Evaluator target access is structurally restricted to post-STOP evaluation. P1/P3 verification uses only observed evidence/events; P2 verification may scan the public constraint system only through the explicit paid VERIFY action.

## Merged bounded C0/C1/C2/C3 reference policies

PR #129 adds the bounded reference policy layer above the execution engine:

- C0 uses a fixed schedule and a step-index-only runtime decision API;
- C1 may perform task-family-only pre-inference planning, then executes a step-index-only static plan;
- C2 adapts only from current/past leakage-safe `PolicyObservation` and is restricted to CONTINUE/STOP;
- C3 may additionally consume the authorized verifier signal and checkpoint availability to choose VERIFY/BACKTRACK;
- every policy decision has explicit logical operation and policy-state memory charges;
- C2/C3 evaluate their declared predicates without short-circuiting so reference policy charges are path-invariant;
- invalid C3-only observation fields on lower arms fail closed;
- the P3 software fixture requires paid C3 VERIFY/BACKTRACK/replay recovery while the corresponding C2 forward path remains on the decoy.

The logical reference charges are accounting conventions for this bounded evaluator, not CPU-instruction or wall-clock claims. PR #129 does not freeze primary schedules, adaptive thresholds, difficulty values, common resource envelopes, split populations or final entropy.

## Remaining TDI-9.1 work

1. compose task generation, execution and C0/C1/C2/C3 policies into one architecture-neutral non-final evaluator with exact per-run provenance, actual compute/memory accounting and typed rejection records;
2. define an agent-search-safe policy mutation/evaluation contract for development/validation only, with executed evidence authoritative over proposals and no path to evaluator-only metadata;
3. implement deterministic paired primary-cell evidence records and the frozen quality-first H9-A/H9-B classifier/aggregation plumbing;
4. use bounded non-final evidence to freeze concrete P1/P2/P3 difficulty parameters, C0/C1 schedules, C2/C3 thresholds, permitted observation vector and common max-compute/max-memory envelopes;
5. freeze one paired family-wise uncertainty implementation and a closed rejection taxonomy consistent with TDI-9.0;
6. establish reproducible checkpoint/provenance/result schemas for non-final qualification and policy-search lineage;
7. freeze the future public-entropy source, event, encoding and deterministic final-seed derivation contract before that public value is knowable;
8. prove fail-closed that no TDI-9.2 final seed list, dataset, executable or result payload exists before the future-entropy gate.

No item in this status file authorizes TDI-9.2 execution.
