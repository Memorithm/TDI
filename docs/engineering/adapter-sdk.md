# Replay adapter SDK and actual library qualification

`tdi_ai::adapter_sdk::ReplayCodec` extends the existing `ReplayAdapter` instead
of introducing another executor. It adds a complete canonical binary codec,
absolute logical progress, a reproducibility class, units, precision, RNG and
structural limits. Generation and intervention produce owned checkpoints;
`run_paired` still owns paired advancement, caller-supplied scoring and sinks.
Hub still owns process execution, dependency scheduling and publication fences.

The three adapters in `tdi_bench::engine_adapters` exercise real library state plus distinct replay surfaces:

| Adapter | Library/state surface | Independent check | Declared reproduction |
| --- | --- | --- | --- |
| `FiniteCycle` | `tdi_core::TableSystem` | Modular arithmetic for a four-state cycle | Exact for the same build/target |
| `FiniteBranchRng` | `tdi_core::TableSystem` + checkpointed deterministic LCG state | Independent integer LCG + sorted-successor oracle | Exact for the same build/target |
| `JacobiSweep` | `tdi_operator::GreenBands` + mutable cached diagonal | Analytic inverse of a positive two-row matrix | Binary64, absolute tolerance `2e-15` in this fixture |

These public fixed fixtures use no frozen research populations. The Jacobi
adapter accepts generic 1..16 row matrices and reports positive-pivot failures;
its Hub qualification recipe deliberately fixes the independently checkable
two-row matrix. Success is software evidence, not confirmation of a hypothesis,
a theorem about asymptotics, a hardware result or a general numerical guarantee.

## Implement an adapter

1. Implement `ReplayAdapter` with a complete value checkpoint, including mutable
   caches and all RNG state. `fork` must not share mutable backend state. Declare
   logical advancement units and supported noise-stream semantics.
2. Implement `ReplayCodec` with bounded decoding. Reject version, shape,
   truncation, trailing data, invalid numeric values and incompatible immutable
   configuration before constructing a branch. Preserve the source on failure.
3. Give independent oracles and freeze any numerical tolerance before running a
   comparison. Do not change an existing protocol because a backend needs a
   different dtype, precision, batching, stochastic or device policy.
4. Run `check_codec_conformance` and `check_replay_conformance` on valid ordered
   contexts. Codec conformance round-trips the fresh source and every supplied
   post-advance checkpoint so an initially empty cache/RNG cannot hide omitted
   runtime state. Add backend-specific malformed checkpoint, cancellation, failure,
   independent-branch and RNG/cache tests. The generic checks are bounded
   diagnostics, not proof that an opaque device session is independent.
5. Bind serialized artifacts to graph/plan, trial, code and input identities at
   deployment. The codec encodes backend state; it does not authorize a campaign.

`FiniteCycle::flipped` and `JacobiSweep::shifted` demonstrate independent
intervention checkpoints. An adapter step uses an absolute `StepContext.depth`;
after restore the next depth is `progress() + 1`. A worker request is bounded to
64 total advances. Encoded backend checkpoints are at most 10 or 512 bytes;
these structural limits are not physical RSS measurements or hostile-code
isolation. Cancellation/deadlines are cooperative between bounded steps.

For paired execution after restore, explicitly prepare
`RelativeReplay::new(&factory, &decoded_reference)` and pass that factory plus
both decoded checkpoints to the existing `run_paired`. It translates relative
run depths to the common restored origin and rejects mismatched branch origins
before scoring. Cancellation callbacks, sink depths and reports remain relative
to the new run; `origin()` supplies the absolute offset. A dedicated regression
runs both actual libraries through this paired path and compares their suffix
observations to uninterrupted execution. Passing advanced absolute-depth
adapters directly to the original relative-depth `run_paired` is unsupported.

## Execute both libraries through Hub

Build Hub in a sibling checkout, outside TDI's offline Cargo configuration:

```sh
cd ../scirust-hub
git checkout ccdcb99a4573dbefb944af0df713101b100b5f78
cargo build --locked -p scirust-hubd -p scirust-hub-worker
cd ../TDI
cargo build --locked -p tdi-bench --example engine_adapter_worker
export TDI_HUB_TOKEN=your-existing-local-token
python3 scripts/tdi_engine.py --hub http://127.0.0.1:8477 --allow-loopback-http \
  library-fixture-plan --worker "$PWD/target/debug/examples/engine_adapter_worker" \
  --adapter jacobi --domain Validation --trials 2 --output jacobi-plan.json
python3 scripts/tdi_engine.py validate jacobi-plan.json
python3 scripts/tdi_engine.py --hub http://127.0.0.1:8477 --allow-loopback-http submit jacobi-plan.json
```

Start the configured local Hub service first, then pass the returned `campaign`
to `run`, `inspect`, `resume` and `export` as described in
[operational-engine.md](operational-engine.md). Use `--adapter finite` or `--adapter branch-rng` for the
finite-state fixtures. Registration hashes the actual trusted Rust executable,
Python interpreter and wrapper and binds their identities into the plan. The
operator must keep this deployment immutable; hashes do not constitute an
attestation or a sandbox against a malicious local writer.

Each trial is a four-step DAG: two-step prefix, restored two-step suffix, fresh
four-step run, independent verifier. The verifier checks all observations,
checkpoint shape/state, exact replay equality, plan/seed/adapter identities and
hashes of its actual inputs. Jacobi uses an analytic formula separate from the
library algorithm. The worker's checkpoint envelope includes both plan ID and
full u64 seed; wrong plan, seed or backend is rejected. Both halves run in
distinct processes; the Hub transfers the real checkpoint artifact.

Run the automated qualification (mandatory binaries, no substituted output):

```sh
cargo test --locked -p tdi-bench --test engine_adapters
cargo test --locked -p tdi-ai --test adapter_sdk_codec --test experiment_contract --test bounded_migration
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=scripts \
TDI_HUBD_BIN="$PWD/../scirust-hub/target/debug/scirust-hubd" \
TDI_LIBRARY_WORKER="$PWD/target/debug/examples/engine_adapter_worker" \
python3 -m unittest -v scripts/test_tdi_library_integration.py
```

This covers actual Hub dependency/artifact handling, independent formulas,
separate-process replay, wrong plan/seed/backend, malformed checkpoints, horizon
exhaustion, Hub restart and verified bundle export. The `branch-rng` fixture adds
a complete post-advance RNG checkpoint checked by a separately implemented Python
integer oracle; Jacobi independently exercises mutable cached numerical state.
Rust tests cover zero horizon, cancellation before launch, metric/sink failure,
failure atomicity, parallel owned branches and stochastic/cache conformance.
No FLAT/NNIS/GPU or protected model execution is implied by these capabilities.
