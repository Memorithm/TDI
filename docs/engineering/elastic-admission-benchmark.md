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
