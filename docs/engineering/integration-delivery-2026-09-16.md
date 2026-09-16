# Integrated research-engine delivery — 2026-09-16

This is the current engineering handoff for the integrated candidate. It
supersedes the coding-frontier/Q19–Q30 observations in the earlier
[qualification audit](industrialization-qualification-audit-2026-09-16.md),
which is retained as a historical snapshot. No historical merge-time evidence
is inferred from later queued reruns. No scientific stage is authorized here.

## Implemented and exercised

| Review partition | Exact source head included | Delivered behavior |
| --- | --- | --- |
| [TDI #385](https://github.com/Memorithm/TDI/pull/385) | `135fc20520fed390d23e78f6374a6a0207a26a4d` | Actual Forge ask/tell and Hub compile/verify/measure; durable checkpoints, costs, baseline/Pareto and ambiguous-submission cancellation. |
| [TDI #387](https://github.com/Memorithm/TDI/pull/387) | `54dff997ede3b948b420ab0c021af2933334ad63` | Actual Hub Morris/Sobol/ablation over declared analytic fixtures; independent SALib/SciPy references and exact workflow/coordinate lineage. |
| [TDI #388](https://github.com/Memorithm/TDI/pull/388) | `9e8ba6bb5e2cad4d6fa1eca6b33b3fb0a371f6da` | Measured journal/serialization/catalogue/Hub benchmarks, capacity compatibility and raw observations; predeclared regression policy. |
| [TDI #389](https://github.com/Memorithm/TDI/pull/389) | `13d46681f8a9f629680eb9c9b77180bf5db8484b` | Schema-4 shared admission proofs, atomic migration, indexed pagination and unchanged portable evidence identity. |
| [TDI #409](https://github.com/Memorithm/TDI/pull/409) | `b9deadb88375cea9d6f67b795bb91fb03f00da44` | Actual campaign/search/DAG consultation, comparison, real observed curves and portable JSON/CSV/HTML plus SVG/PNG/PDF. |
| [TDI #411](https://github.com/Memorithm/TDI/pull/411) | `8424c0dac025b6764b865466ca0b165463abefc2` | Actual FLAT CPU probe, independent f64 oracle, explicit optional NNIS CUDA path and exact upstream unresolved-qualification review. |

These are combined with the current main engine, replay SDK, paired SciRust
analysis, MLflow/OTLP and actual Elastic admission. Integration preserves every
CLI command when merging the overlapping dispatch branches. It also tests
report export directly from all three actual Hub sensitivity campaigns against
the schema-4 catalogue. The complete public-process integration run passed
**70 tests in 42.373 seconds**, with optional figures and real MLflow enabled.
This local run is new evidence; it does not turn queued GitHub checks green.

The final source-bound aggregate invocation on published head
`cabb0b5ab2c6ad2c45121134bbeffa4c822ef028` also passed all 70 tests, with
zero skips/expected failures and unchanged executable hashes. Its
[raw software report](qualification/2026-09-16-public-integration.json) records
the exact runner/test-file hashes, package versions and source declarations.
It explicitly retains `release_qualified: false` and
`hardware_execution_performed: false`. This report is not a build attestation.

The integration adds an always-scheduled aggregate workflow, a no-skip runner,
an operating/recovery guide, source/format compatibility and release procedures,
and a prepared inactive ruleset. Existing numerical, MSRV, scientific-integrity,
optional-feature and real Linux cgroup gates remain in place. The run list is
explicit and contains no protected/final or blocked model suite.

The optional NNIS feature and default FLAT builds both passed Rust 1.89 build
and Clippy locally. Six actual attention tests also passed with each build.
No CUDA device was exposed locally; the CUDA execution path is compiled, not
physically qualified. NNIS's separate CUDA Rust SIMT manifest remains
`unresolved_blocking`; its validator's successful recognition of that state
does not resolve the blocker. Small standard-softmax probes do not authorize a
new attention mechanism, concrete model or production routing change.

## Quantified storage and benchmark evidence

The 16-trial/32-output fixture's stored JSON evidence fell from **1,328,144 to
150,890 bytes (88.6%)** by sharing the exact admission proof. This is stored
JSON payload, not SQLite page size, total campaign memory, CPU complexity or
device speed. Export reconstruction preserves the complete evidence identity.

The corrected engine baseline contains 72 raw records across 18 cases, including
one warmup and three measurements per case. Its plan identity is
`c9f6ba56f4dac4bab374b37c7fac1c547417771c8e36f67c2f5459b92995215e`.
See [raw corrected baseline](benchmarks/2026-09-16-engine-baseline-qualified.json)
and the [benchmark guide](engine-benchmarks.md) for instrumentation/environment
details. “Qualified” in that filename identifies the corrected measurement
manifest; it does not mean that GitHub CI or the industrial release is qualified.
The older pre-cache-manifest files are retained as historical observations and
cannot be silently promoted to compatible timing baselines.

## Exact-head release blockers observed

Historical component-delivery main snapshot: `d8785a04d112e732bf92bd115d23640e7bfb60e7`.
The following counts were fetched on 2026-09-16 after publishing the final
component fixes; they are observations, not predictions of later CI state.

At the 2026-09-16 23:26 UTC consolidation sync, default `main` had advanced to
`bcfa972003907c2528b3664d1b74f13f88b6f892` through PRs #412, #407 and #410.
The consolidated #413 branch was merged forward through that exact main without
conflicts. Current queries for the final heads of those already-merged CI PRs
still include queued/pending workflow runs, so their presence on `main` is not
used here as retroactive exact-head qualification evidence.

At the 2026-09-16 23:31 UTC resync, `main` advanced again through TDI-2.1
PRs #414, #415 and #416 to `0fd3946065fa25e74ee3d3ac0e6d8e93363e620b`;
#413 was merged forward through that state without conflicts. That snapshot had
a Rust 1.97.1 formatting regression in `tdi2_intuition_experience.rs`. PR #419
subsequently landed the canonical formatter result; the duplicate repair #417
was closed without merge. At the current resync, `main` is
`c809ef34e22df7c90843b5946408e45f4ef5ac3d` through #423 and
`cargo +1.97.1 fmt --all -- --check` passes. This removes the inherited format
blocker only; it does not promote any research result or clear the remaining
exact-head/upstream release blockers.

| Repository / PR | Exact head | Observed checks | Consequence |
| --- | --- | --- | --- |
| TDI #409 | `b9deadb88375cea9d6f67b795bb91fb03f00da44` | 57 queued | No final-head qualification. |
| TDI #411 | `8424c0dac025b6764b865466ca0b165463abefc2` | 54 queued | No final-head qualification. |
| [Forge #39](https://github.com/Memorithm/Forge/pull/39) | `0d48a91a5eed6a9ae09a5eacd7c4f18df8bdd333` | 3 queued | Dependency remains candidate. |
| [SciRust #1452](https://github.com/Memorithm/scirust/pull/1452) | `7981120d9b703a5d82796608e7565e3763d8c611` | 7 success, 36 queued, 1 failure | No upstream qualification or final TDI pin. |

The failed SciRust [Dream 7B Thor job](https://github.com/Memorithm/scirust/actions/runs/35153161140/job/104986183960)
reports `CUBLAS_STATUS_NOT_INITIALIZED` at `cublasLtMatmulAlgoGetHeuristic`
during the q-projection. Earlier rotary compatibility changes did not establish
a successful physical run. The failure is retained; no CPU substitution,
tolerance relaxation or model retry has been used to relabel it successful.
Its cause requires evidence from that job's owning hardware/environment.

The current TDI numerical consumer deliberately still identifies tested SciRust
candidate `06c9eaef248c0f40b7d363c16a7b4c1c1c54a5a0`. After upstream exact-head
qualification and merge, all consumers must pin the actual merge SHA and pass
again. Forge has the same final-upstream promotion dependency. The complete
[promotion procedure](release-and-compatibility.md#promotion-procedure) also
requires current reviews, exact TDI-head CI and post-merge verification.

No main merge, industrial release, GPU performance, multi-node load qualification
or production-wide safety claim follows from this delivery record. Main's
branch endpoint reported no active required-check protection; administrative
runner details were unavailable through the connector. Queue cause is unknown.

## Remaining qualification scope

The code and documentation in these review partitions are reviewable now.
To prevent an independent component squash from racing the consolidated route,
#385, #387, #388, #389, #409 and #411 are held as draft review/evidence
partitions while #413 is the active consolidated candidate. Remaining work is
to resolve upstream hardware/CI failures, obtain final reviewed dependency merge
pins, execute the exact-head GitHub gates, merge the consolidated candidate and
verify the resulting main SHA. Physical GPU results,
multi-node fault/load profiles, power-loss durability on a persistent filesystem,
hard VRAM quotas and a mixed-store scientific ACL require their own qualified
profiles; they are not silently included in this trusted non-final product.
