# Release qualification and compatibility

## Candidate profile

This integration is not a released industrial qualification. Its source pins
are explicit, including still-open upstream candidates. Local software passes,
merged code, binary hashes and caller-declared source SHAs are separate facts.
The [dated record](integration-delivery-2026-09-16.md) records their current status.

| Component | Revision consumed by the integration |
| --- | --- |
| TDI | Exact PR head in CI; exact `github.sha` for a main push. |
| scirust-hub | `ccdcb99a4573dbefb944af0df713101b100b5f78`, merged #55. |
| SciRust | `06c9eaef248c0f40b7d363c16a7b4c1c1c54a5a0`, candidate primitive source; #1452 final qualification pending. |
| Forge | `0d48a91a5eed6a9ae09a5eacd7c4f18df8bdd333`, candidate #39. |
| ElasticXxx | `f8e10b1a1d05d6c22c0f56fed87252bc65d4e66a`, final source head of merged #95; current full qualification must be collected separately. |
| FLAT-ATTENTION | `1d5ac64cc87c5dd526e04527c3bb4b78ba0add33`, public CPU reference. |
| NNIS | `5436736002834dd6dd7d5ace8c1c044b47aed18f`, optional CUDA adapter and exact upstream validator. |

The complete public software suite was executed on Linux x86_64, Python 3.12
and Rust 1.89.0. The local kernel was 6.18.44; CPU EPYC 9V74 with eight visible
CPUs. Optional packages are locked in `requirements-research-qualification.txt`
(MLflow 3.16.0, SALib 1.5.2, SciPy 1.18.1, NumPy 2.5.3, Matplotlib 3.11.2 and
OpenTelemetry 1.44.0). Separate smaller sensitivity/reporting locks remain
available. The minimal engine uses the Python standard library. Debian 12 was
not executed and is not a validated installation recipe.

TDI's declared Rust 1.85 MSRV gate remains independent of the Rust 1.89 Hub and
isolated attention-probe workspace. NNIS's own declared MSRV is not raised by
the standalone consumer. Existing default/MSRV/feature gates are retained.
Default FLAT and optional NNIS-feature builds and Clippy passed locally. NNIS
GPU execution, GPU timing/VRAM/energy, multi-node failure/load, hostile-code
sandboxing and power-loss durability on the local volatile overlay filesystem
are not qualified by this profile.

## Reproduce the aggregate software gate

The exact build, checkout and environment recipe is in
[`tdi-research-integration.yml`](../../.github/workflows/tdi-research-integration.yml).
Check out each dependency at the table's SHA into the workflow's sibling paths,
build with `--locked` in each repository's own Cargo configuration, create a
Python 3.12 virtual environment and install the qualification lockfile. The
workflow uses no private datasets, model checkpoints, GPU devices or deployment
credentials. Cargo's attention workspace fetches its exact FLAT/NNIS revisions.

The aggregate runner requires these explicit environment variables:

| Variable | File or value |
| --- | --- |
| `TDI_HUBD_BIN` | Actual `scirust-hubd`; built `scirust-hub-worker` alongside it. |
| `TDI_DURABLE_WORKER` | TDI `durable_worker` example. |
| `TDI_LIBRARY_WORKER` | TDI `engine_adapter_worker` example. |
| `TDI_SEARCH_WORKER` | TDI `finite_search_worker` example. |
| `TDI_SCIRUST_STATS_BIN` / `TDI_SCIRUST_SOURCE_COMMIT` | Actual `research_stats` example and table's source SHA. |
| `TDI_FORGE_WORKER` / `TDI_FORGE_SOURCE_COMMIT` | Actual `scientific_search` example and table's source SHA. |
| `TDI_SEARCH_SOURCE_COMMIT` | Explicit 40-character TDI source revision declaration. |
| `TDI_ELASTIC_BIN` / `TDI_ELASTIC_SOURCE_COMMIT` | Actual `elastic-cli` and table's source SHA. |
| `TDI_ATTENTION_PROBE` | Default CPU build of the standalone attention probe. |
| `TDI_NNIS_CHECKOUT` | Exact NNIS checkout in the table. |
| `TDI_MLFLOW_BIN` | The locked virtual environment's actual `mlflow` executable. |
| `TDI_REPORT_FIGURES` | `1` (optional figures are mandatory in this aggregate profile). |

With those paths set, from TDI:

```bash
.venv-research/bin/python scripts/check_tdi_research_integration.py --report research-integration.json
```

The report filename must be new; no overwrite is performed. The runner selects
14 explicit public modules, checks the pinned Python packages and executable
availability, and runs at least 70 tests. Missing imports, binaries, source
declarations or optional dependencies fail. Fewer collected/executed tests,
skips, expected failures and executable-byte changes also fail. The report
records runtime versions and hashes without dumping the environment or tokens.
An empty/interrupted report is not successful evidence. Hashes do not establish
source/build provenance, nor authenticate the report's author.

`Research integration qualification` is always scheduled and explicitly needs
`Research public software` to succeed. Its `always()` job refuses skip, cancel,
absence and failure; it does not replace repository/MSRV/cgroup or hardware
gates. Only the fixed public test list executes. Source-level scientific gates
and protected-stage rules remain authoritative. The actual workflow must pass
on GitHub before its CI behavior is considered qualified.

## Persisted formats

| Surface | Compatibility contract |
| --- | --- |
| Legacy durable journal | Non-destructive migration and full audit; original evidence retained. |
| Catalogue schema 1 | Original campaigns/events/results; read-only access supported. |
| Catalogue schema 2 | Durable bounded external export queue; atomic migration from 1. |
| Catalogue schema 3 | Durable search checkpoint/attempt mapping; atomic migration from 1/2. |
| Catalogue schema 4 | Shared exact admission proofs and catalogue indexes; byte-preserving old rows, mixed old/new results, atomic migration. |
| Future/unknown catalogue | Refused before create/write; old writers must not open new versions. |
| Portable evidence | Complete descriptors/provenance/results plus trusted optional receipt; restored evidence retains original lineage and cannot dispatch historical work. |
| CLI and HTTP API | Versioned schema-1 envelopes, additive named endpoints; query success does not imply completion. |
| Checkpoint/graph/artifact contracts | Existing versioned manifests retain their own audited source pins; these are contract identities, not running-daemon build attestations. |
| Search/sensitivity/report files | Explicit schema and identity validation; not silently upgraded into a different scientific protocol. |

Run migration/corruption/export/replay suites on every change to these formats.
Back up before any writer upgrade and use the [rollback procedure](operations.md).
The aggregate gate consumes the real Rust examples as external processes,
exercising the SDK outside the library crate.

## Promotion procedure

1. Resolve every material review on the exact upstream candidates. Collect all
   applicable checks, including absent/queued/cancelled/skipped states and full
   pagination; do not infer success from a non-red summary.
2. Qualify and merge SciRust #1452 and Forge #39 in their own repositories. Record
   exact final head and merge SHA. Diagnose the SciRust hardware failure on its
   owning runner; a software stats pass cannot waive it.
3. Correct **all** TDI consumers/workflows/runner declarations to the resulting
   upstream merge SHAs. Rebuild and run the affected numerical/reference and
   integrated suites. Candidate-only pins cannot be relabelled final.
4. Review the consolidated TDI tree, including overlapping CLI dispatch and the
   schema-4/report/sensitivity paths. The component PRs #385/#387/#388/#389/#409/
   #411 are review partitions; choose one integration route and do not replay
   conflicting squash merges. Retain links and authorship evidence.
5. Require every applicable exact-head TDI check and material review resolution.
   Existing Rust/MSRV/format/Clippy, preregistration, contract, actual integration,
   optional-feature and real cgroup gates remain required for their scope.
6. Merge without bypass. Record merge SHA, then re-run the integration and
   applicable regression gates on that exact main SHA. Preserve reports and raw
   benchmark identities. A PR-head pass is not a post-merge pass.
7. Publish a release only with a version, source/lockfile inventory, compatibility
   and migration notes, tested profiles and explicit remaining limitations.
   Device or production claims require their own direct evidence.

The benchmark gate separates correctness from optional large measurements.
Timing regression requires a predeclared compatible baseline and threshold;
three local observations or different filesystems cannot establish a portable
performance guarantee. No new scheduled external automation is introduced.

## Protection policy prepared for administration

[`research-main-ruleset.json`](research-main-ruleset.json) defines a proposed
main-branch ruleset: no bypass actors, no deletion/force push, pull requests with
resolved reviews and one independent approval, and always-scheduled Rust plus
aggregate integration contexts. It is deliberately stored with enforcement
`disabled`; it has **not** been installed or used to change existing protection.
Activation requires observing the exact context names on GitHub, validating
reviewer availability and retaining any stronger existing repository/organization
rules. Additional path-specific gates remain mandatory in the release procedure;
they are not made globally required while their workflows can be absent.

The branch API at the recorded snapshot reported main unprotected with no
required contexts. Runner administration could not be inspected through the
connector; queued jobs do not prove a runner-capacity or billing diagnosis.
The outstanding CI and upstream failures remain release blockers.

The definition follows GitHub's [ruleset REST schema](https://docs.github.com/en/rest/repos/rules#create-a-repository-ruleset).
The aggregation explicitly checks predecessor results because a conditional
skip alone is not qualification; see [job conditions](https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/control-jobs-with-conditions).
