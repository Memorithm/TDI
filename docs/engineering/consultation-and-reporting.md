# Research consultation and report exports

The local viewer reads the same catalogue as the CLI. It requires no Hub token,
does not start workers and does not refresh a workflow. A displayed state is the
latest stored observation; `resume` is the explicit CLI reconciliation action.
Use an administrative catalogue containing only records you may inspect. Keep
its file permissions separate from a candidate-generation account.

## Open the catalogue

After the [operational quickstart](operational-engine.md) creates a catalogue:

```sh
python3 scripts/tdi_engine.py --catalogue tdi-campaigns.sqlite --format pretty status
python3 scripts/tdi_engine.py --catalogue tdi-campaigns.sqlite view --port 8765
```

Open `http://127.0.0.1:8765`. Campaign filters use an exact phase and an optional
Development/Validation domain. Campaign details show counts from the stored
snapshot, declared dependencies, individual attempts, errors and verified outputs.
The dependency diagram uses both `after` and data edges, has at most four nodes
per row and is omitted above 32 steps. Step/output tables page at 50 records.
Unknown observations remain “not observed”; no inferred queue position is shown.

Compare takes two explicit campaign IDs. A shared root protocol is reported as a
declared identity match, not proof of exchangeable measurements. Statistical
comparisons require the separately declared [paired analysis](paired-analysis.md).

Search views show the stored Forge projection: candidates, incorrect/failed
states, measured metrics with their original units, baseline qualification,
Pareto identifiers, active attempt and cancellation cleanup diagnostics. The
`charged_ms` field is reserved non-refundable stage budget, not observed latency.
Each stage links to its actual campaign. No search is replayed by opening a page.

## Select observed curves

The executable counter tutorial produces the actual `/scores` array in `run-0`.
Replace `CAMPAIGN_ID` below with the campaign printed by `submit`:

```json
[{"campaign":"CAMPAIGN_ID","step":"run-0","output":"file:result",
  "label":"Finite counter scores","x_pointer":null,"y_pointer":"/scores",
  "x_unit":"step","y_unit":"points","x_semantics":"logical-index"}]
```

Save this as `series-selection.json`, then use a new output directory:

```sh
python3 scripts/tdi_engine.py --catalogue tdi-campaigns.sqlite series-export \
  --selections series-selection.json --output observed-series
python3 scripts/tdi_engine.py --catalogue tdi-campaigns.sqlite view \
  --report observed-series/report.json
```

Selectors are explicit: 1–16 successful declared outputs from terminal campaigns
in one common non-final domain, and 1–4096 points per series. An absent x pointer
creates only a labeled logical index with unit `step`. Physical time requires an
actual x array and the `physical-time` declaration; arbitrary observed coordinates
use `declared-coordinate`. Units are caller declarations retained in the report.
Null y values create gaps, and no interpolation or imputation is performed.
Out-of-order x coordinates, absent/scalar pointers, nonfinite values and mixed
domains are rejected. A curve does not establish causality.

## Export tables, uncertainty and figures

HTML/JSON/CSV use Python's standard library. The optional Python 3.12 figure
profile is pinned separately, so installing plotting packages does not affect
the minimal engine:

```sh
python3 -m venv .venv-report
.venv-report/bin/python -m pip install -r scripts/requirements-reporting.txt
.venv-report/bin/python scripts/tdi_engine.py report-export \
  observed-series/report.json report-with-figures --figures
.venv-report/bin/python scripts/tdi_engine.py --catalogue tdi-campaigns.sqlite view \
  --report observed-series/report.json --figures
```

`report-export` also accepts `tdi-paired-analysis` and `tdi-sensitivity-analysis`
reports. Paired plots retain candidate-minus-reference estimates, declared units,
percentile intervals, exclusions and denominators. Different units get separate
figures. Insufficient units never become a zero effect. Sensitivity plots retain
Morris, Sobol or ablation point estimates, including negative estimates; no
unavailable confidence interval is invented. Morris effects are per normalized
factor range, while Sobol indices are dimensionless.

Each new export contains `report.json`, `data.csv`, `index.html`, optional
SVG/PNG/PDF figures, and `manifest.json` written last. The manifest records file
SHA-256, lengths, units, report identity, renderer identity and figure dependencies.
HTML exposes the original provenance, scientific limitations and available costs;
costs not selected by an analysis remain unavailable. CSV escapes textual formula
prefixes while preserving numeric negative values. JSON is the full original
report; the HTML table previews at most 500 rows. A pre-existing destination is
refused; an interrupted export has no completion manifest. Verify member hashes
before consuming a directory as complete. Pixel-identical figures across platforms
are not promised. A self-consistent report hash is not a build attestation or an
independent verification of caller-supplied measurements.

## API and access boundaries

GET routes are `/api/campaigns`, `/api/searches`, `/api/report`, `/report.csv` and
`/figure`. Query parameters are allowlisted per route; pagination is bounded;
invalid API requests receive a versioned JSON error envelope. Only reports
explicitly selected at server startup can be fetched, by content identity. No
HTTP path can select a local file. Report contents are frozen in memory at startup.
Limits are 8 MiB per report/response, eight reports and 16 MiB of report content;
optional PNGs have a 32 MiB aggregate budget. No scripts or remote resources are
loaded. Host validation, output escaping, CSP and a read-only SQLite connection
apply to all successful views. Non-GET control operations are unsupported.

## Executed qualification

On Python 3.12/Linux with actual pinned Hub, counter, Forge and SciRust binaries,
19 tests pass: five report/HTTP boundary tests, one full actual Forge search,
nine engine boundary tests and four operational integrations. The optional run
also renders PNG/SVG/PDF, verifies file hashes, checks the image endpoint, preserves
null gaps and exports real SciRust intervals over an explicitly synthetic public
presentation fixture. The minimal profile runs the same suite without figures.
`tdi-research-reporting.yml` builds exact dependency revisions and runs both
profiles. CI qualification remains attached to its exact tested revision.
