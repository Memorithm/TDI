# Optional external tracking and telemetry

TDI can export committed non-final evidence to an existing MLflow experiment
or an OTLP/HTTP JSON collector. Scientific execution has no dependency on either
service. A delivery error returns CLI code `23`, records a failed export, and
preserves the original campaign phase, results, attempts and provenance.

The implementation follows [MLflow's REST API](https://mlflow.org/docs/latest/api_reference/rest-api.html)
and [OTLP/HTTP JSON](https://opentelemetry.io/docs/specs/otlp/). Qualification uses
an actual MLflow 3.16.0 server and the official OpenTelemetry protobuf 1.44.0
schemas. Runtime clients use only the Python standard library. The fully pinned
optional Python 3.12 test environment is
`scripts/requirements-research-qualification.txt`; it also includes the numerical
reference packages used by research qualification. None are imported by the
minimal CLI.

## Export a completed campaign

Reuse `CAMPAIGN` and the catalogue from the operational quickstart. The default
MLflow experiment `0` must exist on the chosen local server. To use a remote
server, choose an HTTPS origin and set `TDI_EXPORT_TOKEN` in the environment.
It is distinct from the Hub token and is never saved in the catalogue.

```bash
python3 scripts/tdi_engine.py --allow-loopback-http export-mlflow "$CAMPAIGN" --endpoint http://127.0.0.1:5000 --experiment-id 0
python3 scripts/tdi_engine.py --allow-loopback-http export-otlp "$CAMPAIGN" --endpoint http://127.0.0.1:4318
python3 scripts/tdi_engine.py exports
python3 scripts/tdi_engine.py inspect-export "$EXPORT_ID"
```

MLflow receives explicit domain/protocol parameters, counts of verified outputs
and observed steps/attempts, and artifact/provenance identities. Optional scalar
metrics require `--metrics FILE`, an explicit selection with a unit:

```json
[
  {"key":"quality","step":"evaluate","output":"file:result","field":"quality","unit":"distance"}
]
```

This is a schema example: the selected field must actually exist in that
campaign's verified output. Missing fields, arrays, strings, booleans, nonfinite
values and integers outside exact binary64 range are rejected before delivery.
Units are exported as `tdi.unit.KEY` tags. TDI does not search result JSON for
possible metrics or export raw results, logs, labels or environment variables.
The built-in counter fixture exports its real technical counts; it has no scalar
`quality` field and correctly rejects the example selection.

OTLP exports three signals. Metrics are counts for the selected durable campaign
with only domain/state dimensions; source identities are trace exemplars. The
span measures the catalogue's wall-clock lifecycle, including queue and user
wait. Logs contain an allowlisted lifecycle state, timestamp, sequence and trace
identity. They omit worker payloads and free-form diagnostics. These exports do
not supply worker CPU/GPU duration, energy, hardware memory or new timing
measurements from a cache hit. A collector's partial-success rejection is an
export failure, even with HTTP 200.

## Delivery recovery and bounds

Export plans are content-addressed, immutable and at most 1 MiB. The local queue
admits at most 32 unfinished exports. Each plan allows at most 200 result
references, 100 selected metrics and 1000 lifecycle events. MLflow batches use
at most 100 items per field. HTTP responses are bounded, redirects are refused,
TLS certificate validation remains enabled and mutations are never retried
automatically.

Repeating an already sent plan returns its original receipt. Failed delivery is
visible in `exports` and the campaign event history. Explicit retry can repeat
remote metrics/telemetry, so external delivery is not claimed to be exactly
once. It never creates a new scientific trial.

```bash
python3 scripts/tdi_engine.py --allow-loopback-http retry-export "$EXPORT_ID"
python3 scripts/tdi_engine.py --allow-loopback-http reconcile-export "$EXPORT_ID" --run-id "$EXISTING_MLFLOW_RUN_ID"
```

MLflow retry reuses an existing run with the exact export identity tag and
experiment. If run creation succeeded but its response was lost, inspect MLflow
for `tdi.export_id`, then supply that run id. Reconciliation performs GET only,
checks every planned metric (including timestamp and step), parameter, tag and
run start/end time and final status, then records
complete or partial delivery. A local delivery lock prevents concurrent senders
and reconcilers without blocking campaign cancellation. An interrupted OTLP
delivery stays visibly unfinished. After inspecting the collector, an explicit
`retry-export` can resend its immutable plan once the lock proves no local
sender is active. This may duplicate already accepted signals; an unfinished
receipt never implies all three signals arrived.

## Catalogue compatibility and qualification

Opening a v1 catalogue for writing performs a transactional v1→v2 migration that
adds delivery state. Existing evidence tables and records remain unchanged;
read-only v1 queries still work. The frozen SQL fixture verifies byte-preserving
migration, rollback after injected DDL failure and successful subsequent recovery.
Backups include delivery state. Result bundles continue to preserve scientific
evidence independently of external tracking services.

```bash
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=scripts python3 -m unittest -v scripts/test_tdi_observability_unit.py
python3 -m venv .qualification-venv
.qualification-venv/bin/pip install -r scripts/requirements-research-qualification.txt
TDI_HUBD_BIN="$PWD/../scirust-hub/target/debug/scirust-hubd" TDI_DURABLE_WORKER="$PWD/target/debug/examples/durable_worker" TDI_MLFLOW_BIN="$PWD/.qualification-venv/bin/mlflow" PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=scripts .qualification-venv/bin/python -m unittest -v scripts/test_tdi_observability_integration.py
```

The tests run the actual Rust/Hub fixture, start MLflow on loopback with SQLite,
verify server-side values and provenance, inject a delivery failure, retry the
same run, and reconcile a lost final acknowledgement. OTLP requests are checked
against the official protobuf message types with the specified hex-ID JSON
mapping; collector partial failure and retry preserve the original results.
No remote cloud account, model, protected dataset or physical resource claim is
needed for these software qualifications.
