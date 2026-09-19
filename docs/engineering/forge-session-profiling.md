# Session profiling and bounded JSON shape fast path

Public Development diagnostics only; no change to optimizer policies, baseline,
budgets, fsync, command receipts, checkpoint identities or protected TDI stages.

## Implementation

`profile-forge-session.py` executes the existing `session-smoke` workload through
cProfile, retains ordinary benchmark evidence and emits plain JSON function
counts, self time and cumulative time. It verifies a completed report before
marking profiling complete. Exceptions propagate, partial diagnostics remain,
and existing output directories are rejected. It never loads profiler pickles.

Cumulative times overlap; do not sum them. Python caller profiling includes IO
and waiting, not just CPU, and does not expose Rust internal function timings.
Instrumented elapsed times are not uninstrumented speed comparisons.

The JSON shape walk now accounts for object keys in place instead of allocating
and later popping a `(key, depth)` stack tuple for every key. `json.loads` supplies
string keys. Keys still count as items, occupy the next depth and are checked by
UTF-8 byte length. Values retain the iterative bounded walk. Empty objects remain
valid at depth zero. Where several limits fail, which limit is reported first
can change; acceptance/rejection, not error precedence, is the contract.

Duplicate-key, nonfinite-number, byte-length and UTF-8 decoder guards are unchanged.
The change does not skip validation or introduce a trusted-input bypass.

## Local diagnostic evidence

Base checkout: `1f130894f510bf4315d3070ce06f2ac34d116b99`; Forge rebuilt from
`83c8c572a692016fb786e661c47a249112b6cadd`; Optuna 5.0.0 exact lock.
These are pre-publication local diagnostics: the after run includes the JSON
patch, and manifests retain individual source hashes. They are not release
qualification or measurements of a later default branch.

One original smoke profile completed 2,304 evaluations. Its 7.608 s instrumented
total included 2.664 s cumulative `atomic_json`, 2.329 s `_exchange`, and 0.959 s
`strict_json`. The categories overlap. This supports inspecting persistence and
validation, not attributing all residual cost to fsync or Rust proposals.

Two uninstrumented smoke campaigns (2,304 evaluations each) were verified and all
144 parameter/loss/incumbent trajectories matched. Before report identity:
`3c42c4d982facb13adc0b8385e9e8d7bc9238050be3d278965f37a633a120478`.
After report identity:
`f283d4ba6cdd0bd24228c912df68190c87d96646729f116a67c17c710a7378e4`.
No overall speed claim is inferred from these two runs.

Retained raw evidence: [profile](benchmarks/2026-09-19-json-shape-profile.json.gz),
[before](benchmarks/2026-09-19-json-shape-before.json.gz),
[after](benchmarks/2026-09-19-json-shape-after.json.gz). The before/after reports
can be decompressed and checked with `tdi_optuna_benchmark.py --verify-report`.

The initial alternating microbenchmark used 22 real session records, 30 repeats
per round and ten rounds: median paired reference/candidate shape-walk ratio
1.3624. This is only the shape walk, not decoding or complete adapter speed.
`benchmark-json-shape.py` reproduces that schedule and emits all timings and
input hashes. CI records diagnostics without unstable performance thresholds.

Local tests: 35 passed, one existing supervisor test skipped because its separate
Rust worker was not built. The three new shape tests cover 8,320 differential
data/limit combinations, Unicode byte boundaries, item/depth boundaries and
decoder rejection. Three profiler tests cover accounting, failure preservation
and refusal to classify an unverified report as complete. Actual Forge session,
crash recovery, corruption, bounds, pidfd/fallback and high-descriptor checks pass.

## Reproduce

Build the pinned Forge release binary and install the comparison requirements as
in [the session protocol](forge-session-comparison.md), then run:

```bash
PYTHONPATH=scripts python3 -m unittest scripts/test_json_shape_fastpath.py scripts/test_forge_session_profile.py
.venv-optuna/bin/python scripts/profile-forge-session.py --forge-worker ../integration-forge/target/release/examples/scientific_search --output session-diagnostic
.venv-optuna/bin/python scripts/benchmark-json-shape.py --session-directory session-diagnostic/experiment/shifted-bowl/0/forge-tpe-session
```

Next: measure persistence, initialization and closing separately under equal
durability guarantees. Do not remove fsync or verification to manufacture a win
against Optuna. Search-policy improvements require a separate prospective
comparison; these diagnostics do not select a policy or modify frozen evidence.
