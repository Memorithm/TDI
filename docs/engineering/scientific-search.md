# Durable scientific parameter search

TDI can drive the actual Forge ask/tell process and execute each authorized
stage through the local Hub. The first qualified consumer is a public four-state
cycle: `table` uses `tdi_core::TableSystem`, `modular` computes its closed form,
and `incorrect` is an intentional negative control. This is an infrastructure
fixture, independent of the frozen TDI research populations and model gates.

Forge owns the finite search space, proposal order, prerequisites, retries and
Pareto selection. TDI owns the independent oracle and measurements. Hub owns
processes, workflow identities, artifact transfer and authoritative publication.
The catalogue links each Forge attempt to exactly one Hub workflow. Compiled
configuration and verification outputs become verified root artifacts of the
next workflow, preserving their actual CAS identities and provenance.

## Run the public fixture

Use Rust 1.89 or the repository's qualified toolchain, Python 3.12 and Linux with
`wait4`, `/proc` and a trusted local Hub. Build the sibling repositories outside
TDI's Cargo configuration:

```sh
cd ../Forge
git checkout 0d48a91a5eed6a9ae09a5eacd7c4f18df8bdd333
cargo build --locked -p forge-bridge --example scientific_search
cd ../scirust-hub
git checkout ccdcb99a4573dbefb944af0df713101b100b5f78
cargo build --locked -p scirust-hubd -p scirust-hub-worker
cd ../TDI
cargo build --locked -p tdi-bench --example finite_search_worker
```

Start the configured Hub with the same trusted local deployment described in
[operational-engine.md](operational-engine.md). Keep its existing token in
`TDI_HUB_TOKEN`. Then create a search:

```sh
python3 scripts/tdi_engine.py --hub http://127.0.0.1:8477 --allow-loopback-http \
  search-fixture --worker "$PWD/target/debug/examples/finite_search_worker" \
  --tdi-source-commit "$(git rev-parse HEAD)" \
  --forge-worker "$PWD/../Forge/target/debug/examples/scientific_search" \
  --forge-sha256 "$(sha256sum ../Forge/target/debug/examples/scientific_search | cut -d ' ' -f 1)" \
  --forge-source-commit 0d48a91a5eed6a9ae09a5eacd7c4f18df8bdd333 \
  --strategy grid --seed 18446744073709551615
```

The returned `result.id` is the search identity. Preparation validates the actual
Forge process and persists the contract; it executes no candidate. Pass that
identity to these commands (replace `SEARCH_ID`):

```sh
python3 scripts/tdi_engine.py --hub http://127.0.0.1:8477 --allow-loopback-http search-run SEARCH_ID --max-stages 1
python3 scripts/tdi_engine.py search-inspect SEARCH_ID
python3 scripts/tdi_engine.py --hub http://127.0.0.1:8477 --allow-loopback-http search-resume SEARCH_ID
python3 scripts/tdi_engine.py searches --limit 20
python3 scripts/tdi_engine.py backup search-backup.sqlite
```

The explicit first-stage pause demonstrates recovery without changing the search
population. A full successful fixture has three proposals, eight executed stages,
two measured variants and one incorrect variant with absent metrics. Timing/RSS
values and the Pareto set depend on the actual machine; there is no fixed winner
or expected speedup. `scientific_verdict` remains `not-assessed`.

Use `search-cancel SEARCH_ID` with the same Hub options for cancellation. It
persists `cancel-requested` before cancelling mapped workflows. A transactional check
prevents a cancelled search from starting an already admitted stage. A prepared
workflow can now be cancelled before any Hub submission. In-flight execution
still requires the Hub's cleanup acknowledgement. Repeating cancellation can
finish cleanup after a lost reply; it does not reopen the search. Unknown
submissions remain `cancel-requested` with explicit `pending_cleanup` campaign
identities until their existing workflows are attached and cancellation is
repeated. The state becomes `cancelled` only after all mapped work is terminal.
Neither a pending nor completed cancellation can dispatch work.

## What is measured

The compile stage explicitly records `precompiled-configuration`: it verifies and
materializes the chosen configuration for the already built worker. It does not
claim to compile native code per candidate, and prior Cargo build time is outside
this declared search profile. The generic Forge protocol also represents actual
native compilation for a separately qualified executor. Arbitrary generated
source, models, remote placement and hostile code are not enabled here.

The verifier launches real worker processes for four initial states and six
horizons, checking every output against independent Python modular arithmetic.
Candidate subprocesses receive only implementation, initial state and horizon;
they never receive oracle answers, verifier records, Hub credentials or fitness.
Malformed/failed processes are technical failures, separate from well-formed
incorrect answers. The former pause for explicit retry and retain measured costs;
the latter are terminal incorrect candidates and never enter measurement.

After successful verification, one warmup and three measured repetitions execute
the same declared state/horizon. The external evaluator reports:

- median **process wall time**, in nanoseconds, including launch and result I/O;
- maximum observed child **peak RSS** across measured repetitions, in bytes;
- all raw measured costs, warmup labels, CPU user/system time, process exit state,
  sensor identity, clock and 5 ms completion polling granularity.

These metrics are process measurements, not isolated operation latency. They are
not suitable for claiming sub-millisecond kernel speedups. Missing accelerator
time, GPU memory and energy remain absent. Verification and measurement share
the exact declared local environment identity. Logical state size is not physical
RSS. The immutable binary, evaluator, Python and telemetry source identities are
bound to the search; declared Git revisions are not build attestations.

## Budgets, permissions and recovery

This consumer fixes three candidates, twelve stage attempts, two attempts per
stage and 15 seconds per attempt. Every begin reserves the full 15 seconds, with
no refund after success, failure, interruption or retry. Observed overshoot is
additionally charged and prevents selection. Unobserved failure durations stay
absent, with explicit missing-cost counts; they do not become zero-cost retries.
Forge control-process costs are recorded separately in search events. Queueing
and Hub admission/transfer costs belong to the linked workflow timeline, outside
the declared stage execution reservation.

The complete Forge checkpoint commits before Hub submission. The attempt-to-Hub
mapping commits before the first workflow mutation. On restart, TDI replays the
pinned Forge binary and compares the derived projection, then reconciles the
same mapped workflow. Completed measurements are not rerun or counted as fresh
replications. A lost `submit` reply remains `submission-unknown`: use the existing
`attach` command with the exact workflow and root bindings from `search-inspect`
before resuming. Never create a replacement workflow to hide uncertainty.

All stage artifacts use Validation access and disabled scientific caching.
Administrative inspection is for the trusted local operator. Only the separate
`generation_view` may be passed to a proposer; it omits verification and final
sources. This fixture has no final source at all. Parameter search does not grant
access to any research series' protected data or final execution surface.

Exit codes follow the operational CLI: 0 for success or explicit bounded pause,
20 for a failed/unknown execution, resource stop or cancellation, 21 for invalid
contract and 22 for storage failure. Search inspection includes `last_transition`,
`execution_error`, stage/workflow IDs and root bindings, so paused failures are
distinguishable from an intentional pause.

## Storage and qualification

Catalogue schema 3 introduced searches, their events and attempt mappings. Schema
4 additionally interns repeated admission proofs and indexes catalogue queries.
Writers
atomically migrate deployed schemas 1 and 2; read-only access preserves those
versions. Legacy evidence/export receipt bytes are unchanged. Migration failure
rolls back DDL and version together. Old writers reject schema 4; upgrade them
before sharing the catalogue. SQLite backup includes search state. Export the
linked stage campaigns with the existing `export` command to retain their actual
Hub artifacts as portable bundles; a catalogue backup alone does not copy Hub CAS.

Run the actual qualification with all binaries explicitly supplied:

```sh
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=scripts \
TDI_HUBD_BIN="$PWD/../scirust-hub/target/debug/scirust-hubd" \
TDI_SEARCH_WORKER="$PWD/target/debug/examples/finite_search_worker" \
TDI_FORGE_WORKER="$PWD/../Forge/target/debug/examples/scientific_search" \
TDI_FORGE_SOURCE_COMMIT=0d48a91a5eed6a9ae09a5eacd7c4f18df8bdd333 \
TDI_SEARCH_SOURCE_COMMIT="$(git rev-parse HEAD)" \
python3 -m unittest -v scripts/test_tdi_forge_integration.py
```

The suite covers real Forge/Hub/process execution, independent incorrect-candidate
exclusion, restart, tell persistence failure after actual completion, unchanged
Hub run identity on resume, cancellation before/after admission, corrupted
projection rejection, launch failure accounting, explicit paid retry, and v2
migration/rollback/backup. This is software evidence, not model-quality or hardware
optimization qualification.
