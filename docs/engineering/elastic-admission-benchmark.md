# Real Elastic admission comparison

This non-final software benchmark reuses TDI's counter/shift Rust worker,
scirust-hub's actual scheduler and ElasticXxx's `admit-capacity` controller.
It does not implement a second resource planner in Python or introduce Go.
It is a measurement prerequisite, not an announced performance improvement.

## Fixed protocol

The default matrix has requested widths 1, 2, 4, 8, eight identical trials
(each run has a dependent independent verification step), four repetitions and
two arms: fixed-width Hub versus actual Elastic admission. Arm order alternates
by repetition. Width order is fixed, not randomized; host drift remains a limit.
Each case starts a fresh isolated Hub and SQLite catalogue. No cached execution
is counted as new work. Elastic may reduce the width; the workload cannot change.
This is **one graph at a time**, not a global multi-campaign capacity allocator.

The manifest is written before execution. It retains the complete schedule,
declared per-trial memory envelope, source declarations, binary hashes and
deployment script hashes. Source declarations are not build attestations.
An existing output directory is rejected. Failures keep prior cases, the local
catalogue, diagnostic data and Hub state; no complete report is published on
exceptions. Admission refusals remain in the report and yield a nonzero exit.

The primary wall-time window includes submit/admission, execution and verified
collection. Registration and Hub startup, export/verification, clean restart
and reconciliation have separate clocks. Controller process costs and capacity
observations are retained. Python CPU clocks do not include Hub/worker CPU.
Total concurrent peak RAM and energy are **not measured** and must not be inferred
from the Elastic controller child's RSS. No power-loss experiment is performed.

The benchmark compares all result payloads, checks that only concurrency changes,
exports and verifies every bundle, and restarts the Hub to ensure reconciliation
does not change snapshots or results. This checks clean restart, not crash recovery.
Summary p95 uses nearest rank on completed runs only and is descriptive (with four
repetitions it is merely the maximum). Refusals are not included as fast latency
samples; their timed admission windows remain in the successful-work throughput
denominator. All attempt counts and actual widths are published.

## Execute

Build the pinned dependencies as in `tdi-resource-admission.yml`, then run from TDI:

```sh
python3 scripts/tdi_elastic_benchmark.py \
  --hubd ../integration-hub/target/debug/scirust-hubd \
  --worker target/debug/examples/durable_worker \
  --elastic ../integration-elastic/target/debug/elastic-cli \
  --hub-source ccdcb99a4573dbefb944af0df713101b100b5f78 \
  --elastic-source f5af4f3129100f4bb52d47d1adfc202347d0b098 \
  --output elastic-admission-benchmark
```

Recheck the matrix, bundle bytes/provenance, cross-arm payloads and summaries
without executing any workload:

```sh
python3 scripts/tdi_elastic_benchmark.py --verify elastic-admission-benchmark
```

This verifies internal consistency, not authenticity of wall-clock measurements.

Use an isolated local deployment with enough memory for the fixed arm. The
64 MiB/trial and 128 MiB reserve defaults are admission envelopes, not hard
quotas or workload allocations. The tiny counter work primarily measures
orchestration and verification, not a representative ML workload. Elastic can
add overhead without changing width on an unconstrained host. A favorable
result would not establish better search quality, universal speedup, distributed
admission safety or performance under sustained competing campaigns.

CI runs a smaller 1/2-width, two-trial, two-repetition matrix using the same
real binaries. It asserts correctness and restart behavior, never a noisy
speed threshold. Artifacts are retained even when the job fails.

## Development measurements, 2026-09-19

The initial 32-case debug execution at `559fe421a7f8522e2da6830307ce16d7b1c8b0c5`
is preliminary: it included an extra caller-side executable hash inside the
timed admission window. `f6ff031451095b11c3017e98ee3651360a779c47` moves this
setup outside timing; the consumer's before/after identity checks are unchanged.
Do not pool preliminary and corrected timing samples.

Before executing corrected measurements, the planned conditions are: all three
Rust executables built with `cargo +1.89.0 build --release --locked`; the full
default matrix first with inherited CPU affinity, then restricted to the first
two allowed CPUs using `taskset`. Child Hub and worker processes inherit the
same affinity. This restricts allowed CPUs but does not isolate them from other
host activity. Both modes run sequentially, not simultaneously. All cases and
both conditions must be retained regardless of performance. The two-CPU case
tests real width reduction rather than a forged capacity observation.

### Observed results

Both corrected release matrices completed: **64 graph executions, 512 Rust
counter trials and 512 independent verification steps**. Every exported bundle
passes verification; payload identities agree across arms within each matrix;
all 64 clean restarts preserve snapshots and results. These are software
qualification observations, not independent scientific tasks or ML evidence.

Median submit/admission + execute + collect wall time, milliseconds (four
repetitions per cell, no confidence interval):

| Requested width | Inherited affinity, fixed | Inherited affinity, Elastic | Two CPUs, fixed | Two CPUs, Elastic |
| --- | ---: | ---: | ---: | ---: |
| 1 | 1255.85 | 1270.24 | 1208.69 | 1196.66 |
| 2 | 793.72 | 841.32 | 766.86 | 789.10 |
| 4 | 579.30 | 564.04 | 763.56 | 753.89 |
| 8 | 456.76 | 456.99 | 776.19 | 758.17 |

With inherited affinity, Elastic retained widths 1/2/4/8. With affinity limited
to CPUs 0 and 1 (the first two allowed CPUs), Elastic admitted widths 1/2/2/2;
the fixed arm retained 1/2/4/8. At requested width eight on two CPUs, observed
throughput was 10.55 versus 10.30 verified trials/second in the timed windows.
Four repetitions on a shared host do **not** establish a reliable speedup.
They demonstrate actual width reduction and comparable observed throughput on
this small fixture. No memory savings or generic performance benefit is inferred.
The native controller is still only an admission decision; this does not solve
cross-campaign accounting of capacity or adaptive resizing of a running graph.

The executed measurement code is `f6ff031451095b11c3017e98ee3651360a779c47`;
the condition declaration was published at
`11f37b77c7989c63acb945875f4addf8eb3d45d4` before corrected execution. A subsequent
change moves report verification before final publication and adds a fault
regression; it does not change the measured execution path. All archives below
are retained, including the preliminary debug run, and reverified by CI.

| Archive in `benchmarks/` | SHA-256 |
| --- | --- |
| `2026-09-19-elastic-preliminary-debug.tar.gz` | `1c136c3b2573c1d9e8297662a34839f278c9936189765aad7d254ec8ca3f74e4` |
| `2026-09-19-elastic-release.tar.gz` | `8e87e25f9b8f516ff90592ab5f632d22f6f31e88b049a7afa2ad4c085a91d022` |
| `2026-09-19-elastic-release-two-cpu.tar.gz` | `2905f4dbc4c540a136418756633d8cfda4f8c0a68961650a722abdd631f9c748` |

Corrected report identities: inherited affinity
`e22fc53dc469d4e4b6bfedc49e3ed546d0b35d795459db363a3e3b13f74549c1`;
two CPUs `160767b8a90b6cb64ea8e32ebaaa929b070e0813eb18210d8075acbc84504f6c`.
Archives contain manifests, reports, per-case records and verified bundles;
they intentionally omit Hub databases/logs and are not deployment backups.
