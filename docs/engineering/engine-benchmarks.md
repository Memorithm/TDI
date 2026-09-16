# Measuring the research engine

`scripts/tdi_engine_benchmark.py` runs a bounded suite of actual software
measurements. It records a plan before execution, every warmup and repetition,
child CPU/wall/peak RSS, visible CPU/filesystem/cgroup context, exact source
file and binary hashes, and a final content-addressed report. Each case runs
in a fresh process and temporary catalogue. Incomplete execution retains its
completed records and reports failure; it never produces a successful baseline.

Use Linux/Python 3.12 and the real public counter worker and Hub from the
[operational engine guide](operational-engine.md). No optional Python package
is required. The normal command runs serial cases; concurrency is exercised
only inside explicitly planned Hub campaigns.

```bash
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=scripts python3 scripts/tdi_engine_benchmark.py \
  --hubd ../scirust-hub/target/debug/scirust-hubd \
  --worker target/debug/examples/durable_worker \
  --source-commit "$(git rev-parse HEAD)" \
  --hub-source-commit ccdcb99a4573dbefb944af0df713101b100b5f78 \
  --sizes 32 128 512 --hub-trials 1 4 16 \
  --repetitions 3 --warmup 1 --max-seconds 600 --output engine-baseline
```

The output directory must not exist. Source declarations do not attest a build;
the report separately includes hashes of the actual binaries and source files.
The worker uses only the public non-final counter control and independent
expected score `[2,2,2,2]`. Hub daemons use fresh authenticated loopback stores;
no shared deployment or scientific research data is modified.

| Case | Measurements and scope |
| --- | --- |
| Minimal worker | Actual process launch/completion, CPU, peak RSS and output bytes; includes process overhead. |
| Journal | FULL-synchronous SQLite append with 64/4096-byte result bodies, hash-call/input-byte counts, independent audit, cached read, reopen and Python allocations. |
| Serialization | Canonical JSON encoding, bounded parsing and raw SHA-256 over 1 KiB, 64 KiB and 1 MiB bodies. |
| Catalogue | Actual SQLite insertion and last-page queries over explicitly synthetic inventory; EXPLAIN query plan and database size. These rows assert no executed workflows. |
| Hub | Actual registration, admission, scheduling/execution/collection, artifact transfer, provenance hashing, daemon restart and durable reconciliation. Concurrency one/four uses the same declared trial count. |
| Observability | Actual OTLP plan encoding and catalogue persistence after execution. Remote delivery is explicitly unmeasured; vendor/server latency is not inferred. |

Every raw record distinguishes warmup from measured repetitions. Summaries give
median/min/max and sample count, without confidence or significance claims.
Linux `/proc/self/io` deltas include reading the counters themselves. Physical
`read_bytes` may be zero when caches serve requests. JSON response counts measure
canonical body representations, not HTTP headers/TLS framing. Artifact transfer
counts use the actual downloaded byte lengths.

Journal hashing checks are exact: two new event hashes per completed trial and
zero full scans during append. Audit and reopen remain full integrity checks.
This does not assert constant SQLite/fsync/index cost or constant memory: the
journal retains completed results in memory, and larger bodies cost more.

The reference environment exposes overlay storage with `fsync=volatile`.
Consequently its timing results qualify only that visible software environment;
they do not qualify power-loss durability or predict industrial disk throughput.
No CPU isolation, thermal stability, cold page cache, GPU or energy measurement
is claimed. Run the same plan on the intended deployment before setting an SLO.

## Regression policy

A comparison requires both `--baseline engine-baseline/report.json` and
`--policy regression-policy.json`. Freeze the latter before running the candidate.
Its schema is:

```json
{
  "schema": 1,
  "baseline_identity": "EXACT_BASELINE_REPORT_ID",
  "reason": "Document the workload SLO, noise allowance and why this budget is acceptable",
  "limits": [
    {"case": {"kind": "journal", "count": 512, "payload_bytes": 64},
     "metric": "/write/wall_ns", "max_ratio": 1.2}
  ]
}
```

This is a schema example, not a qualified 20% threshold. Choose limits from an
independently recorded baseline and the deployment's justified budget, before
seeing candidate results. Missing/zero metrics, altered baseline identity,
non-finite ratios and incompatible workload/environment are rejected. A ratio
above the declared limit returns exit 20 with all actual observations retained.
No default latency threshold silently passes the build.

The bounded CI profile uses sizes 16/64, one/four Hub trials, two measured
repetitions and no warmup. It validates executability and exact work counters;
it does not gate noisy latency or replace the richer recorded baseline.

## Initial observations

The September 16 baseline was measured before storage/query optimization.
Its raw report is versioned beside this guide. The journal's measured hashing
volume grows with the new event bytes, with no history scan during append.
The catalogue query uses a full scan and temporary sort on the unindexed
baseline. Hub artifact payloads stay small, while complete admission evidence
is repeated in every stored result. This identifies shared-proof storage and
indexed catalogue queries as concrete optimization targets. Reconstructed
portable evidence must remain byte-for-byte semantically identical after any
such change; a smaller database alone does not prove faster verification.

Baseline identity: `bb8a9d8f86f128b247ff8611360deb1e7000976dc57df8745729082404a4e571`. [Raw plan and repetitions](benchmarks/2026-09-16-engine-baseline.json).


| Workload | Median measurement | Additional observed volume |
| --- | --- | --- |
| Journal 32 trials, 64-byte bodies | 5.868 ms append | 9,484 hashed bytes; no full append scan |
| Journal 128 trials, 64-byte bodies | 21.537 ms append | 38,052 hashed bytes; no full append scan |
| Journal 512 trials, 64-byte bodies | 92.746 ms append | 152,868 hashed bytes; no full append scan |
| Hub 1 trials, concurrency 1 | 166.958 ms execute/collect | 13,574 serialized evidence bytes |
| Hub 4 trials, concurrency 1 | 643.978 ms execute/collect | 109,688 serialized evidence bytes |
| Hub 16 trials, concurrency 1 | 3263.523 ms execute/collect | 1,328,144 serialized evidence bytes |

Each row has one warmup and three measured repetitions. These are scoped
observations on the recorded host, not latency targets or general speed claims.
