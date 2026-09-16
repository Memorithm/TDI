# Paired analysis with the shared SciRust process

The generic engine analyzes explicitly selected Development/Validation evidence.
It does not change the frozen analysis rules of existing scientific series.
An exploratory protocol declares independent units, paired repeats, strata,
comparisons, metric units, missing-data policy, confidence, resampling budget,
seed and multiplicity. A common seed or repeated row is not proof of independence.

SciRust owns numerical resampling and sensitivity primitives. TDI owns protocol
semantics, provenance selection and reports. The integration uses
`scirust-research-stats-json/v1` at SciRust commit
`06c9eaef248c0f40b7d363c16a7b4c1c1c54a5a0` (PR Memorithm/scirust#1452).
This source is pending upstream integration; use the exact pin for qualification.
The installed binary's SHA-256 is recorded separately; this is not a build
attestation. No new numerical Python dependency or TDI Rust MSRV bump is required.

## Protocol and selected observations

For a small software demonstration, prepare this protocol before running the
campaign. Each of two units has one paired repeat, and both arms are evaluated.
The pairing is a demonstration grouping, not a research claim about independent
real-world tasks.

```json
{"schema":1,"purpose":"exploratory-analysis","domain":"Development","name":"counter-comparison","unit_kind":"declared-cluster","units":[{"id":"u0","stratum":"counter","replicates":["r0"]},{"id":"u1","stratum":"counter","replicates":["r0"]}],"comparisons":[{"id":"pair","reference":"reference","candidate":"candidate","metric":"distance","unit":"counter-units"}],"strata":[],"missing":"reject-incomplete","confidence":0.95,"resamples":1000,"seed":"731","multiplicity":"marginal-exploratory"}
```

Save it as `protocol.json`, then validate without reading any result:

```bash
python3 scripts/tdi_engine.py analysis-validate protocol.json
```

Follow the [operational engine guide](operational-engine.md) to run a four-trial
counter fixture. Each result is produced by the actual Rust worker and checked
by the independent verifier. Select `/scores/0` from `run-0` and `run-1` as u0's
reference/candidate, and `run-2` and `run-3` as u1's. Selectors have exactly these
fields (replace the campaign identity with the actual returned identity):

```json
{"unit":"u0","replicate":"r0","arm":"reference","metric":"distance","campaign":"<actual-campaign-identity>","step":"run-0","output":"file:result","pointer":"/scores/0"}
```

Save the four selectors as a JSON list in `selections.json`. The path is an
explicit JSON pointer, including array indices when appropriate; no metric or
dataset discovery occurs. Selectors for failed/unexecuted steps produce visible
nonnumeric observations. A successful step missing verified evidence is an
integrity error. A single source observation cannot be counted twice within a
metric. Restored/cached evidence retains original source identity.

Build SciRust independently, outside TDI's Cargo configuration directory:

```bash
cd ../scirust
git checkout 06c9eaef248c0f40b7d363c16a7b4c1c1c54a5a0
cargo build --locked -p scirust-stats --example research_stats
sha256sum target/debug/examples/research_stats
cd ../TDI
```

The example assumes sibling checkouts named `scirust` and `TDI`. Keeping the
build inside SciRust avoids inheriting TDI's offline Cargo configuration.
Supply the printed binary digest as `STATS_SHA256`:

```bash
python3 scripts/tdi_engine.py --catalogue tdi-campaigns.sqlite analyze \
  --protocol protocol.json --selections selections.json \
  --worker ../scirust/target/debug/examples/research_stats \
  --worker-sha256 "$STATS_SHA256" \
  --source-commit 06c9eaef248c0f40b7d363c16a7b4c1c1c54a5a0 \
  --output analysis.json
```

This software fixture produces four distance values of 2 and a paired mean
effect of 0, with a degenerate [0,0] interval. It is a successful infrastructure
qualification with no scientific benefit verdict. Existing output files are
never overwritten. The `--observations` alternative accepts an explicit
protocol-bound observation document; its provenance references are caller
assertions, whereas `--selections` projects the verified catalogue.

## What the report means

Repeats are paired and averaged *within* each unit before resampling. Complete
units receive equal weight even when their repeat counts differ. Missing,
technical-error and rejected observations are distinct; no zero or mean is
imputed. `reject-incomplete` refuses analysis, while
`exclude-incomplete-unit` excludes the whole paired unit and retains every
reason and all per-arm planned/observed/error denominators. Fewer than two
included units is `insufficient-units`, with a null interval.

`bonferroni-family` adjusts the requested confidence over all declared
comparison × (overall + stratum) intervals, including unavailable strata.
`marginal-exploratory` leaves marginal confidence unchanged and makes no
family-level coverage claim. The percentile bootstrap is approximate, not a
universal finite-sample coverage guarantee. No p-value, equivalence,
non-inferiority, optional stopping or confirmatory authorization is inferred.

Reports preserve protocol and observation identities, source artifact and
provenance references, unit effects, exclusions, bounds, estimator version,
binary/source identities and an explicit `scientific_verdict:not-assessed`.
The effect is candidate minus reference in the declared metric's units. Costs
are explicitly unavailable unless measured/selected by a separate qualified
path; logical counts are not converted to seconds or joules.

Limits: 16 MiB and one million JSON items per selected analysis input, 20000
planned/observed rows, 10000 units, 100 repeats per unit, eight
comparisons, sixteen selected strata, 64 source campaigns, 10 million bootstrap
contributions per request and 20 million for the entire report. The trusted
process client bounds input and each output stream to 1 MiB and wall time to
30 seconds. It runs without inherited credentials and removes its process group
on exit/interruption. This is not hostile-code isolation.

## Executed qualification

```bash
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=scripts python3 -m unittest -v scripts.test_tdi_research_analysis.ProtocolTests
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=scripts python3 -m unittest -v scripts/test_tdi_research_analysis.py scripts/test_tdi_analysis_integration.py
```

The second command requires `TDI_SCIRUST_STATS_BIN`, `TDI_SCIRUST_SOURCE_COMMIT`,
`TDI_HUBD_BIN`, and `TDI_DURABLE_WORKER` to identify actual built executables.
Missing binaries fail qualification. Eight tests pass locally: invalid protocol
and rows, equal unit weighting, exclusions and family confidence, insufficient
data, actual shared methods, and Hub→catalogue→CLI→SciRust report generation.
The last scenario also checks source duplication, cross-domain rejection and
overwrite refusal. SciRust's independent reference suite checks the estimators
against SciPy 1.18.1 and SALib 1.5.2; see its `docs/RESEARCH_STATISTICS.md`.
