# Durable development execution

This follow-up implements five engineering improvements without changing frozen
scientific semantics or authorizing model/final execution.

| Point | Surface | Acceptance evidence |
| --- | --- | --- |
| Persistence | Linux Python supervisor with SQLite FULL synchronous append-only journal and single-writer lock | Restart equivalence, real process crash, corruption, incomplete trial and writer exclusion tests |
| Verified provenance | SHA-256 of explicitly selected artifacts, canonical versioned plan, pre/post-trial verification and worker response binding | Changed worker/plan and wrong response identity rejected |
| Adapter qualification | Both execution orders, complete checkpoint comparisons, source and sibling mutation checks | RNG replay, shared cache counterexample, injected fork/checkpoint/step failures |
| Deadline/isolation | Separate worker process group, wall-clock deadline, bounded combined stdout/stderr, cooperative cancellation and cleanup | Timeout, output flooding, cancellation and worker failure tests |
| Bounded migration | `tdi_ai::bounded_recovery::analyze_bounded` | Equality with the historical API for qualified pure callbacks, zero horizon, one intervention, partial failure |

## Scope and guarantees

The Rust library continues to own dynamics and evaluation. The supervisor is a
standard-library-only Linux operational client, not a replacement scientific
evaluator, model adapter, scheduler, search engine or hardware runtime.

`CampaignPlan` remains an in-memory typed API. Durable execution is a separate
per-trial worker protocol: a worker receives `--tdi-seed` and `--tdi-plan-id` and
returns a single JSON object with matching integer `seed`, string `plan_id` and
`status` equal to `Evaluated` or `Rejected`. Additional result fields are preserved.
The supplied `durable_worker` example exercises the real bounded Rust API on a
deterministic counter fixture. Actual TDI series workers require separate
qualification and must respect the merged series gate.

No final domain is available. No model is selected, downloaded or executed. The
generic Development/Validation namespace is not a frozen series seed protocol.

Each trial's Start is committed before spawning; Finish is committed after output
and artifact verification. Resume never executes an already recorded trial again.
An unmatched Start becomes an explicit Interrupted rejection, not a silent retry
or a fabricated result. Thus completed-trial-boundary interruption is equivalent
to continuous execution; a crash during an active trial has an intentionally
different recorded outcome. Exactly-once arbitrary external effects are not
promised. Workers should be side-effect-free outside their returned output.

SQLite integrity checking and a SHA-256 chain reject corrupt records and invalid
transitions. They are not tamper-proof attestation: an attacker able to rewrite
the whole journal, or a cleanly deleted suffix, requires an external trusted
checkpoint to detect. Use a local filesystem with reliable fsync and flock.

Artifacts are an explicit allowlist: pin the executable, relevant source files,
lockfile, configuration and every external input. The tool verifies listed bytes,
not the completeness of the operator's list, the compiler's provenance, all
dynamic system libraries or a Git commit's existence. Source, runtime and input
directories must remain trusted and immutable while running; pre/post hashing
cannot rule out a malicious modify-and-restore race. The tool never scans or
collects protected historical datasets automatically.

Process groups separate failures and allow cleanup on normal supervisor paths.
This is not a security sandbox: filesystem/network access is not restricted,
escaped descendants require an external containment mechanism, and sudden
supervisor SIGKILL can leave workers alive. Recovery records the uncertain trial
without rerunning it; operators must ensure old workers are terminated before
resuming after a supervisor-only crash. Wall-clock/output bounds are not RAM,
CPU, GPU or energy bounds. Real hardware evidence still requires measurement.

The historical Rust API is unchanged. The new migration API requires independent
Clone semantics and side-effect-free dynamics/observables; it is not a universal
replacement for callbacks whose observation order changes external state.

## Runnable fixture (Linux, Python 3.11+, Rust 1.85+)

From the repository root, build and freeze an explicit fixture plan:

```bash
cargo build --locked -p tdi-ai --example durable_worker
python3 scripts/prepare_tdi_experiment_plan.py --root . \
  --worker target/debug/examples/durable_worker \
  --artifact Cargo.lock --artifact tdi-ai/src/experiment.rs \
  --artifact tdi-ai/src/bounded_recovery.rs --artifact tdi-ai/examples/durable_worker.rs \
  --index 0 --index 1 --index 2 --domain Development \
  --timeout 10 --output-limit 65536 --plan /tmp/tdi-development-plan.json
python3 scripts/tdi_experiment_supervisor.py --root . \
  --plan /tmp/tdi-development-plan.json --journal /tmp/tdi-development.sqlite
```

Repeating the last command reads the same completed records, without executing
the workers again. Creating a new plan refuses to overwrite an existing file.
Never use the same journal for a different plan. These paths are example outputs,
not durable storage recommendations for valuable experiments.

Validation:

```bash
cargo test --locked -p tdi-ai --all-features
TDI_DURABLE_WORKER=target/debug/examples/durable_worker \
  python3 -m unittest discover -s scripts -p test_tdi_experiment_supervisor.py -v
```

Potential reuse: the supervisor/worker boundary can serve other deterministic
Memorithm experiments without moving TDI scientific semantics. No other
repository is modified or claimed qualified by this change.
