# TDI/Forge versus Optuna: executable comparison

This is a **public Development-only engineering benchmark**, not a confirmatory
TDI series or a claim of general superiority. TDI supplies the common evaluator,
oracle, budget accounting and evidence. Candidate proposals come from the actual
Forge Rust bridge or actual Optuna 5.0.0. No alternative Forge algorithm is
reimplemented in Python and no Optuna result is simulated.

## What the first profile can answer

At the same number of objective evaluations, which strategy finds better points
in the same finite categorical domain? How often does each revisit a point?
What wall time does each **complete adapter path** consume on this machine?

| Arm | Implementation | Replacement / adaptation |
| --- | --- | --- |
| `forge-grid` | Forge `scientific_search`, `grid` | Lexicographic, without replacement; non-adaptive. |
| `forge-random` | Same Rust binary, `random-without-replacement` | Seeded random ordering after the mandatory baseline; non-adaptive. |
| `optuna-random` | Optuna `RandomSampler` | With replacement; duplicate evaluations are charged and retained. |
| `optuna-tpe` | Optuna `TPESampler` | Categorical TPE, 10 startup trials, 24 EI candidates, `multivariate=False`, `constant_liar=False`. |

Forge source is `28067ab0aa1d52a2260d9bb2bf35a346547a292a`. Its bridge currently
offers grid/random proposal strategies, not TPE. This benchmark does not measure
all Forge algorithms or all Optuna configurations. Future optimizer changes need
new revisions and comparison identities; never silently rewrite old observations.

## Fixed comparison contract

- Three exact integer functions: shifted quadratic bowl, coupled quadratic
  ridge, and quartic double well. Each uses `x,y in {-8,...,7}`: 256 legal points.
- The evaluator and separately expanded polynomial oracle agree over the entire
  grid before running. Exhaustive oracle work establishes the grid minimum and
  maximum for scoring; those values are **not supplied to either proposer** and
  are outside the common optimization budget. This is public software checking,
  not an independent hidden population or ML validation dataset.
- Every arm starts at `(-8,-8)`, because Forge requires that baseline. Optuna
  receives it via `enqueue_trial`; it counts as evaluation one for all arms.
- `smoke`: 2 seeds × 3 tasks × 4 arms × 8 evaluations = 192 evaluations. It checks
  plumbing, not TPE learning: this budget is below its startup threshold.
- `development`: seeds 0–19 × 3 tasks × 4 arms × 32 evaluations = 7,680
  evaluations. This allows TPE's adaptive phase. Public seed numbers are fixed
  before execution. The same numeric seed does not imply identical random draws
  across Rust/rand and Optuna/NumPy.
- All evaluations are serial. No pruning, cache, batching or automatic retry.
  A duplicate still consumes an evaluation. The same compilation/configuration,
  independent verification and objective evaluation path is used for all arms.
- Arm order rotates by task index plus seed. This mitigates simple order effects
  without establishing CPU isolation or removing thermal/load noise.
- Primary outcome: final best loss minus exact finite-grid minimum. Secondary:
  mean normalized incumbent regret across **all** evaluations, including baseline
  and duplicates; normalization uses the same grid range for every arm.
- Report each task separately. Seeds are optimizer repetitions on fixed tasks,
  not independent task populations. Deterministic grid repeats are identical in
  quality. Medians are descriptive; no p-value, superiority, tie-equivalence or
  population confidence interval is inferred.

The manifest records the complete contract, packages, source-file hashes,
declared Git revisions, actual Forge/Python binary hashes and runtime environment
before the first proposal. Declared revisions/hashes are not build attestations.
Forge checkpoints persist before acting on stage permits. Every arm saves its
stage evidence and trajectory. Final completion requires every task/seed/arm and
every budget slot; a failure keeps partial files and creates `failure.json`, not
a successful report. A partial directory must not be reused for a fresh run.

## Timing boundaries

Forge uses the existing TDI process client, hashes its executable before/after
each call, and replays its bounded checkpoint. Use the release build below;
the debug build has a different binary size and execution cost.
Optuna runs in-process with its in-memory study. Both save stage evidence, but
their control and persistence mechanisms differ. `observed_adapter_ns` includes
setup, control calls, verification, evaluation and durable per-arm writes.
Per-trial time excludes writing its final event row; stage-body time excludes
control calls and disk publication. Whole-process startup, the initial exhaustive
oracle and final aggregation are outside per-arm timing. These are not native
optimizer microbenchmarks or end-to-end Hub campaign timings.

The benchmark does not rank frameworks by speed, fault tolerance, RSS, GPU,
multi-objective performance or pruning. No Hub worker or model is executed.
Correctness checks on public polynomials do not establish that generated native
code is safe or scientifically correct.

## Run on the qualified Python 3.12 / Rust 1.89 profile

From TDI, use a separately pinned Forge checkout in a new sibling directory:

```bash
git clone https://github.com/Memorithm/Forge.git ../forge-optuna-reference
git -C ../forge-optuna-reference checkout --detach 28067ab0aa1d52a2260d9bb2bf35a346547a292a
```

Build from Forge's directory so TDI's offline Cargo settings do not apply:

```bash
(cd ../forge-optuna-reference && cargo +1.89.0 build --release --locked -p forge-bridge --example scientific_search)
python3.12 -m venv .venv-optuna
.venv-optuna/bin/python -m pip install -r scripts/requirements-optuna-benchmark.txt
.venv-optuna/bin/python -m pip check
PYTHONPATH=scripts .venv-optuna/bin/python -m unittest -v scripts/test_tdi_optuna_benchmark.py
.venv-optuna/bin/python scripts/tdi_optuna_benchmark.py \
  --forge-worker ../forge-optuna-reference/target/release/examples/scientific_search \
  --profile smoke --max-seconds 300 --output optuna-smoke
.venv-optuna/bin/python scripts/tdi_optuna_benchmark.py --verify-report optuna-smoke/report.json
```

For the exploratory development comparison, use a **new** output directory:

```bash
.venv-optuna/bin/python scripts/tdi_optuna_benchmark.py \
  --forge-worker ../forge-optuna-reference/target/release/examples/scientific_search \
  --profile development --max-seconds 1800 --output optuna-development
.venv-optuna/bin/python scripts/tdi_optuna_benchmark.py --verify-report optuna-development/report.json
```

`report.json` embeds the manifest and raw trajectories. Its verifier checks
content identities and recomputes scores, incumbents, duplicates, complete budgets
and summaries. Self-consistent hashes do not authenticate the producer or prove
execution. Keep CI logs/raw stage files as separate execution evidence. Debian 12
installation and physical-device profiles require their own validation.

## Next comparison stages

1. Add representative public numerical/ML problems, including conditional and
   continuous spaces, only after both adapters expose identical domains. Preserve
   these first finite-domain results and keep development separate from later
   unseen evaluation instances.
2. Introduce adaptive Forge proposals behind its versioned bridge, then compare
   against the unchanged Optuna baseline. Generic search algorithms belong in
   Forge, numerical estimators in SciRust, and comparison semantics in TDI.
3. Compare pruning separately with matched training-step/resource budgets;
   equal trial counts alone are insufficient when trials stop early.
4. Compare multi-objective Pareto quality with frozen objective directions,
   constraints and reference points; do not reuse this single-objective score.
5. Compare deployment/recovery using equivalent storage and worker arrangements,
   before drawing infrastructure-performance conclusions.

References consulted 2026-09-18: Optuna's official [ask/tell interface](https://optuna.readthedocs.io/en/stable/tutorial/20_recipes/009_ask_and_tell.html),
[TPE sampler](https://optuna.readthedocs.io/en/stable/reference/samplers/generated/optuna.samplers.TPESampler.html)
and [sampling/pruning overview](https://optuna.readthedocs.io/en/stable/tutorial/10_key_features/003_efficient_optimization_algorithms.html).
The installed package lock, rather than a moving documentation alias, identifies
the implementation actually executed.

## First observed development run

The [2026-09-18 evidence record](benchmarks/2026-09-18-optuna-development.md)
contains the actual 7,680-evaluation result on source
`2c185982f1d62cd73b5dcf59c5c0f487b0165c3b`, its full raw report and median
incumbent curves. It is exploratory evidence under the exact finite categorical
protocol above; it does not establish a general optimizer ranking.


## Adaptive extension (separate public Development profile)

`adaptive-smoke` uses two seeds and sixteen evaluations per arm, so the adaptive
phase is exercised. `adaptive-development` uses twenty seeds (0..19), thirty-two
evaluations per arm and six arms: Forge random, native categorical TPE, SciRust GP
with Forge categorical acquisition, Optuna random, independent TPE and multivariate
TPE. The three historical tasks remain; shifted absolute loss, rotated valley and
categorical interaction add different public landscapes. Six tasks × twenty seeds
× six arms × thirty-two evaluations = **23,040 evaluations**. Both new Forge
strategies are fixed before the first execution. Constants, source pins, tasks,
budgets and comparators are recorded in the protocol and manifest.

This is Development, not an independent test population or a protected TDI
research series. The earlier public results were known during implementation;
this extension cannot establish general superiority. Neither additional tasks nor
seeds are dropped on unfavorable outcomes. No tuning of constants occurs after
results in this version. All comparators get the same categorical domain and
initial baseline. No ordinal geometry, oracle minima, alternative-arm results or
cached objective values enter either Forge model. Optuna's independent TPE is
retained for historical continuity; explicit multivariate TPE is also included,
reflecting Optuna 5's single-objective default. Both use ten startup trials and
24 EI candidates, with no pruning/concurrency and `constant_liar=False`.

The GP reuses `scirust-gp` at `146575107005c24a47682dcaa08c4cd9464d1cc3` through a
pinned Cargo dependency. SciRust owns Cholesky/posterior arithmetic; Forge owns the
categorical kernel, normalization, acquisition and verification-gated feedback.
The ANEE audit and reasons for selecting this primitive are recorded in Forge's
`docs/SCIRUST_SEARCH_AUDIT.md`. Source declarations and binary hashes remain
provenance, not build attestations. Legacy smoke/development protocol and retained
reports keep their original Forge pin and are verified unchanged.
