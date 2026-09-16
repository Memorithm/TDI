# Research engine: start here

The engine runs explicitly admitted Development/Validation research through
Hub, preserves evidence across failures, and provides bounded search, analysis
and consultation. The integrated implementation is a **release candidate**.
The [qualification matrix](industrialization-status.md#qualification-matrix-progress)
and [delivery record](integration-delivery-2026-09-16.md) distinguish completed
software work from pending CI, upstream dependencies and hardware evidence.

## First campaign

Follow the [local operational tutorial](operational-engine.md#installation-and-a-complete-runnable-example).
It builds a real Rust counter worker, starts an authenticated local Hub, submits
a two-trial run/verify graph, inspects it, restarts/reconciles, and exports and
restores its evidence. Use a new directory and new plan/archive names. `validate`
checks the contract; `plan` previews the graph; neither command authorizes an
otherwise blocked scientific stage. A successful fixture result means the
independent counter oracle agrees. It says nothing about a research hypothesis.

The CLI defaults to versioned JSON. Add `--format pretty` before the command for
indented JSON. A technical exit of zero does not mean that a running campaign
has finished or that a scientific hypothesis has been accepted. Inspection
retains failed, incomplete, rejected and unavailable states.

## Choose the next task

| Task | Executable guide | Evidence and scope |
| --- | --- | --- |
| Add a backend, branch or checkpoint | [Replay adapter SDK](adapter-sdk.md) | Actual finite-state and Jacobi libraries; complete state, cache and RNG isolation; independent replay checks. |
| Admit resources before execution | [Elastic resource admission](resource-admission.md) | Actual Linux capacity readings and Elastic permits; only graph width changes within the frozen protocol. |
| Search a finite candidate space | [Scientific search](scientific-search.md) | Forge ask/tell, Hub compile/verify/measure, bounded budgets, checkpoint and cancellation recovery. |
| Compare paired outcomes | [Paired analysis](paired-analysis.md) | Predeclared units, exclusions, effect, bootstrap interval and Holm family; insufficient units remain explicit. |
| Measure sensitivity or ablation | [Sensitivity](sensitivity.md) | Morris/Sobol/ablation plans and actual public analytic Hub fixtures; source and plan identities retained. |
| Inspect a campaign or publish figures | [Consultation and reporting](consultation-and-reporting.md) | Read-only local pages/API, DAG, search costs, real observed curves, CSV/HTML/JSON and optional SVG/PNG/PDF. |
| Export tracking or telemetry | [MLflow/OTLP](observability-exports.md) | Optional explicit export; bounded durable delivery queue; external failures do not erase source results. |
| Inspect physical and storage costs | [Engine benchmarks](engine-benchmarks.md), [shared evidence](shared-evidence-storage.md) | Raw observations, warmups, units and compatibility rules; byte deduplication does not imply a timing speedup. |
| Compare public attention operators | [FLAT/NNIS probe](../../integrations/attention-probe/README.md) | Real FLAT CPU semantics and independent oracle; optional NNIS CUDA compiled, physical execution unqualified. |
| Operate or release the engine | [Operations](operations.md), [release and compatibility](release-and-compatibility.md) | Backups, migrations, incidents, exact dependency pins, software/hardware gates and release conditions. |

## Ownership and deployment

| Owner | Responsibility |
| --- | --- |
| TDI | Scientific protocol, domain separation, result admissibility, provenance and interpretation. |
| scirust-hub | Generic scheduling, workers, registry, leases, fencing, transport, CAS and authoritative publication. |
| Forge | Candidate generation, proposal budget, search checkpoint, baseline/Pareto mechanics. |
| SciRust | General numerical/statistical primitives. |
| ElasticXxx | Resource observations, admission and physical resource-control semantics. |
| FLAT-ATTENTION / NNIS | Attention semantics and backend-specific NVIDIA execution/qualification. |

The supported operational profile uses trusted immutable component deployments,
a dedicated non-final Hub/catalogue and explicit local authentication. A remote
Hub requires HTTPS and its own correctly provisioned worker deployments. The
shipped executable-file fixtures require loopback because their pins refer to
local files; changing the URL cannot deploy those binaries remotely. The
separate cgroup-v2 supervisor has its own qualified Linux quota/cleanup profile.
Hub process supervision alone does not inherit its cgroup guarantees.

## Reproducibility and interpretation

Freeze the protocol, independent experimental unit, arm/stratum, input hashes,
seed/RNG algorithm and state, adapter, dependencies, precision and budgets before
execution. A checkpoint binds all relevant state and progress to that identity.
Changing the plan, seed, backend or checkpoint lineage refuses a resume.

Logical determinism means the same logical operations and decisions under the
declared contract. Bitwise replay additionally requires the qualified backend,
precision, implementation and environment. A shared seed alone promises neither
bitwise floating-point equality nor the same timing, device behavior or energy.
Use the declared reproducibility class in the SDK; record unavailable physical
measurements as unavailable. Cache reuse preserves original provenance and is
never a new trial or a fresh timing observation.

Analysis aggregates at the predeclared independent unit, not at the number of
correlated trajectory observations. Missing pairs and exclusions remain in the
inventory; no imputation or selective retry is implicit. A negative effect,
wide interval, unresolved outcome or failed candidate is valid evidence. Holm
adjustment uses the predeclared family, including unavailable members according
to the declared policy. Do not read a display interval as a new test of a
different hypothesis or silently clip negative Sobol estimates.

## Vocabulary

| Term | Meaning in this engine |
| --- | --- |
| Protocol | Frozen question, population, interventions, budgets, metrics and decision rules. |
| Campaign | Durable execution intent and evidence for one admitted plan. |
| Trial | One planned realization with explicit identity and seed. |
| Attempt | A concrete execution of work; a failed attempt remains distinct from a planned trial. |
| Observation | A declared measurement or logical record within a trial. |
| Metric | A quantity with a definition, unit and aggregation rule. |
| Artifact | Immutable bytes with identity, media type, access policy and provenance. |
| Checkpoint | Complete resumable state bound to plan, adapter, input, RNG and progress. |
| Lease | Hub-owned time-bounded permission for an execution attempt. |
| Fencing | Hub rejection of publication from a superseded or expired attempt. |
| Determinism | Repetition property within a specifically declared logical or numerical scope. |
| Experimental unit | Independent unit that contributes weight to statistical inference. |
| Effect | The explicitly defined contrast between arms at those units. |
| Interval | Uncertainty computed by a declared method with denominator and confidence level. |
| Verdict | A scientific classification under frozen rules, distinct from technical completion. |

Ordinary examples, CI, adapters and optimization never authorize protected
holdouts, final material or a blocked concrete-model stage.
