# Sensitivity and ablation

The optional TDI sensitivity CLI creates reproducible Development/Validation
plans, executes explicitly selected public analytic controls through the normal
Hub workflow API, and analyzes complete verified catalogue outputs using the
actual SciRust process. It grants no model or final-population authorization.

Install the optional profile with Python 3.12:

```bash
python3 -m venv .sensitivity-venv
.sensitivity-venv/bin/python -m pip install -r scripts/requirements-sensitivity.txt
```

The tested sampling profile is SALib 1.5.2, NumPy 2.5.3 and SciPy 1.18.1.
Their versions and the sampled function's source hash are recorded. The plan
contains all coordinates, row ordinals, units, generator options and an identity.
Validation regenerates the full design; use its original pinned code/environment
to resume it. A declared source hash is not a build attestation.

## Protocol

The following creates an eight-trajectory Morris protocol over three declared
dimensionless factors. It creates no observations and overwrites no file.

```bash
.sensitivity-venv/bin/python - <<'PY'
import json
p = {
    "schema": 1, "purpose": "exploratory-sensitivity", "domain": "Development",
    "name": "additive-control", "method": "morris", "output_unit": "dimensionless",
    "samples": 8, "seed": "0", "max_evaluations": 32,
    "response": {"kind": "public-analytic/v1", "function": "additive"},
    "missing": "reject-incomplete",
    "assumptions": ["deterministic-response", "independent-uniform-factors"],
    "factors": [{"name": f"x{i}", "unit": "dimensionless", "lower": 0, "upper": 1,
                 "baseline": None, "intervention": None} for i in range(3)]
}
with open("sensitivity-protocol.json", "x") as f:
    json.dump(p, f, allow_nan=False)
PY
.sensitivity-venv/bin/python scripts/tdi_engine.py sensitivity-plan \
  --protocol sensitivity-protocol.json --output sensitivity-plan.json
```

All factors must have finite positive ranges. The seed is a decimal u64 string.
Limits are 1–8 factors, 4,096 evaluations, 16,384 coordinate scalars and a declared
`max_evaluations` that covers the complete design. Budget violations fail before
sampling. No sampler may discard observations, optimize its seed after results,
or reinterpret a failed batch as zero output.

The response function is frozen in the protocol before sampling. Analytic plans
also pin the evaluator, its support files and Python binary. Collection
reconstructs the exact expected component manifest, parameters and workflow;
registration checks the manifest digest against the actual Hub. Editing a
prepared workflow's coordinates, function, component or policy makes its results
incompatible with the original plan, even when echoed row IDs remain unchanged.

| Method | Design | Output and limitations |
| --- | --- | --- |
| `morris` | 2–128 complete trajectories; four levels; no optimized subset; `N*(D+1)` points | SciRust unscaled elementary effects: `mu`, `mu_star`, sample `sigma`. Coordinates are normalized, so effects refer to each factor's full declared range. |
| `sobol` | Base N is a power of two from 16–2048, subject to total bounds; scrambled LMS+shift, zero skipped points; `N*(D+2)` points | SciRust Saltelli 2010 first and total indices, centered outputs, A/B population variance. Dimensionless; estimates remain unclipped. No second-order indices or intervals. |
| `ablation` | `samples=1`, `seed="0"`; one baseline and one explicit intervention per factor | Descriptive intervention-minus-baseline differences. Every factor requires distinct in-range `baseline` and `intervention`; no implicit “off” value. No causal/general benefit verdict. |

For ablation, declare assumptions exactly as
`["deterministic-response", "declared-one-factor-interventions"]`. For Sobol and
Morris, use the assumptions shown in the example. Independence and a deterministic
response must hold for the research use; the engine cannot infer them from rows.
Stochastic models need a separately declared replication/noise design first.

The implementation follows the official
[SALib Morris sampler](https://salib.readthedocs.io/en/latest/api/SALib.sample.morris.html)
and [modern Sobol sampler](https://salib.readthedocs.io/en/latest/api/SALib.sample.html#SALib.sample.sobol.sample).
It does not use the deprecated Saltelli sampler. SciRust consumes the complete
Morris trajectories or A, B and the D A-with-B-column output arrays. Sampling
points with equal coordinates keep distinct row identities and evaluations.

## Real Hub execution

Use the [operational engine](operational-engine.md) setup with an authenticated
explicit Hub. The examples use its permitted loopback development deployment.
Preparing registers a pinned local analytic component but submits no workflow.

```bash
.sensitivity-venv/bin/python scripts/tdi_engine.py --hub http://127.0.0.1:8477 \
  --allow-loopback-http sensitivity-fixture-plan --plan sensitivity-plan.json \
  --function additive --output sensitivity-campaign.json
.sensitivity-venv/bin/python scripts/tdi_engine.py --hub http://127.0.0.1:8477 \
  --allow-loopback-http submit sensitivity-campaign.json
```

Pass the returned `campaign` ID to ordinary `run`, `inspect`, `resume`, `attach`,
`cancel`, `export` and `verify` commands. TDI does not implement another scheduler.
The graph has at most 128 batches, 32 points per batch, concurrency at most four
and a ten-second timeout per batch. Cache is disabled. Every output records
the plan identity, row IDs, actual response and scoped batch CPU/wall/process-RSS
measurements. Startup, imports and hash checks are excluded from the batch timer;
GPU and energy measurements are absent.

The analytic choices are `additive`, sum of `(factor_index+1)*value`, and
`ishigami`, the public three-factor function with constants 7 and 0.1. The latter
requires three factors bounded within [-pi, pi]. Both require dimensionless
units. Neither choice is a concrete TDI research-series model.

## Analyze and preserve evidence

Build the real SciRust `research_stats` example as described in the
[paired-analysis guide](paired-analysis.md). The current CI dependency is
SciRust candidate `06c9eaef248c0f40b7d363c16a7b4c1c1c54a5a0`; promotion remains
blocked until upstream #1452 is fully qualified and the final merge is repinned.

```bash
.sensitivity-venv/bin/python scripts/tdi_engine.py sensitivity-analyze \
  --plan sensitivity-plan.json --campaign CAMPAIGN_ID \
  --worker ../scirust/target/debug/examples/research_stats \
  --worker-sha256 "$(sha256sum ../scirust/target/debug/examples/research_stats | cut -d ' ' -f 1)" \
  --source-commit 06c9eaef248c0f40b7d363c16a7b4c1c1c54a5a0 \
  --output sensitivity-report.json
```

Use the literal returned ID in place of `CAMPAIGN_ID`. Ablation requires no
SciRust executable. A generic consumer can select exact batches with
`--selections selectors.json`, an array of `{campaign,step,output}` records,
or supply a complete observation envelope using `--observations`. External
non-executing plans may declare `response` as `{kind: "declared-external/v1",
name, implementation_sha256, configuration_sha256}` (the latter two are exact
SHA-256 strings). Their catalogue collector requires a separately qualified
adapter; this release accepts their explicit observation envelope only. Direct
observation provenance is explicitly a caller assertion. The test module shows
that envelope and exercises it against independent analytic references.

Catalogue extraction accepts terminal non-final sources only. It orders rows by
the original plan, rejects duplicate/scalar reuse, mismatched plan or units,
non-finite outputs, missing observations and incomplete batches. Batch completion
order is immaterial. A failed analysis leaves all source campaign diagnostics and
costs intact. Existing output files are never overwritten. Reports carry complete
denominators, source references, costs, implementation identity and
`scientific_verdict: not-assessed`; confidence intervals are explicitly absent.

## Executed qualification

`scripts/test_tdi_sensitivity.py` requires the actual SciRust binary and Hub daemon
and uses the optional Python profile. Six test methods cover three real CLI/Hub
campaigns, Morris and Sobol differences against official SALib analyzers on the
Ishigami control (tolerance 5e-12), analytic additive shares (absolute tolerance
0.015 at N=512), declared ablation effects, deterministic regeneration, incomplete
and permuted inputs, duplicate sources, invalid domains/seeds/units/budgets and
file overwrite refusal. Two additionally executed, deliberately altered Hub
workflows prove that substituted coordinates and a changed function are rejected
before analysis. Numerical agreement is not a universal estimator proof.

```bash
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=scripts \
TDI_HUBD_BIN="$PWD/../scirust-hub/target/debug/scirust-hubd" \
TDI_DURABLE_WORKER="$PWD/target/debug/examples/durable_worker" \
TDI_SCIRUST_STATS_BIN="$PWD/../scirust/target/debug/examples/research_stats" \
TDI_SCIRUST_SOURCE_COMMIT=06c9eaef248c0f40b7d363c16a7b4c1c1c54a5a0 \
.sensitivity-venv/bin/python -m unittest -v scripts/test_tdi_sensitivity.py
```
