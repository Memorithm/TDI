# Operational TDI engine v1

This is the first executable TDI Graph/v1 → Hub → real Rust worker → verified
result path. It adds a durable scientific evidence catalogue, a CLI, a local
viewer, portable evidence bundles and an exact deterministic-data cache. Hub
continues to own scheduling, process execution, registry, leases and CAS.

The shipped reference adapter runs the actual `tdi-ai` `durable_worker` example
and independently checks its finite counter/shift oracle. Its `Verified` result
is a software fixture verdict, not a neural-model or scientific-benefit claim.
Only explicit Development/Validation trusted-software campaigns are accepted.
Existing frozen series and their execution gates are unchanged.

## Installation and a complete runnable example

Linux, Python 3.12, SQLite with local fsync/flock semantics, TDI Rust 1.85+ and
Hub Rust 1.89 are required for the tested profile. The HTTP integration avoids
raising TDI's Rust MSRV. Build dependencies from their lockfiles. Hub transfer
support is pinned in CI to merge `ccdcb99a4573dbefb944af0df713101b100b5f78` (#55).
The older source pins embedded in Graph/G1/G2/G3 describe audited contract
versions; they are not an attestation of the running daemon's source.

From the TDI checkout, with a sibling checkout named `scirust-hub`:

```bash
cargo build --locked -p tdi-ai --example durable_worker
cargo +1.89.0 build --locked --manifest-path ../scirust-hub/Cargo.toml -p scirust-hubd -p scirust-hub-worker
export TDI_HUB_TOKEN=$(python3 -c 'import secrets; print(secrets.token_hex(32))')
SCIRUST_HUB_TOKEN="$TDI_HUB_TOKEN" ../scirust-hub/target/debug/scirust-hubd --listen 127.0.0.1:8477 --data-dir ./hub-data
```

Keep that daemon running; use another terminal with the same token. The token
is an environment value, never part of a URL or CLI argument. For a remote Hub,
select HTTPS and configure its certificate and authorization using Hub's guide.
Plain HTTP requires an explicit loopback IP and `--allow-loopback-http`.

```bash
tdi() { python3 scripts/tdi_engine.py --hub http://127.0.0.1:8477 --allow-loopback-http "$@"; }
tdi doctor
tdi fixture-plan --worker target/debug/examples/durable_worker --trials 2 --output counter-plan.json
tdi validate counter-plan.json
tdi plan counter-plan.json
CAMPAIGN=$(tdi submit counter-plan.json | python3 -c 'import json,sys; print(json.load(sys.stdin)["result"]["campaign"])')
tdi run "$CAMPAIGN"
tdi status --phase completed
tdi inspect "$CAMPAIGN"
tdi events "$CAMPAIGN"
tdi resume "$CAMPAIGN"
tdi export "$CAMPAIGN" counter-evidence.json
tdi verify counter-evidence.json
tdi backup catalogue-backup.sqlite
python3 scripts/tdi_engine.py --catalogue restored.sqlite --hub http://127.0.0.1:8477 --allow-loopback-http restore counter-evidence.json
tdi view --port 8765
```

Open `http://127.0.0.1:8765`. The viewer is optional and read-only, serves actual
catalogue records and has no Hub credentials. CLI status, inspect, compare,
events, backup and archive verification also work offline. `compare LEFT RIGHT`
shows both protocols and never interprets a protocol mismatch as a regression.
The reference test executes these commands, including an independent restored
catalogue and a real loopback HTTP viewer; no pre-existing result file is used.

## Intent, interruption and recovery

The catalogue commits `submitting` before workflow creation and `executing`
before dispatch. Its short SQLite transactions do not hold a process lock
while waiting for Hub; a second client can persist `cancel CAMPAIGN` promptly.
If a mutation response is lost, the campaign remains visibly ambiguous. The
client never repeats the mutation automatically. `resume` reconciles the
existing workflow by GET. It does not authorize a new scientific attempt.

If workflow creation succeeded but its response was lost, inspect the existing
Hub workflow and explicitly use `attach CAMPAIGN WORKFLOW_UUID [--roots FILE]`.
The complete graph, admission pins and root bytes must match. An already
running attached workflow cannot return to a dispatchable state. If no durable
workflow exists, operator reconciliation is required; deleting intent to force
a replay is not a recovery protocol. Hub's own attempt/retry policy remains
authoritative; this client neither issues leases nor publishes result fences.

Successful outputs are checked against the successful attempt, producer run,
declared media/access/JSON policy, portable SHA-256/size, authoritative
publication and exact admission. Partial verified outputs remain visible after
another step fails. Terminal failure survives daemon and client restart.
Technical completion does not infer a beneficial scientific verdict.

All command results are JSON envelopes with schema 1. Exit codes: `0` technical
success/query accepted, `20` failed/cancelled trial or ambiguous transport,
`21` invalid contract, `22` storage failure, `130` interrupted client. A running
phase in a successful query is not a completed campaign. CLI errors also go to
stderr; Hub response bodies and credentials are not copied into diagnostics.

## Portable evidence and cache

Evidence bundles contain the complete declared result inventory of successful
steps, raw bytes, descriptor, source provenance, admission and terminal Hub
snapshot. Hashes and coverage are checked before any restore upload. Supply
`verify/restore --expected-identity SHA256` from a trusted receipt to detect
whole-bundle replacement; internal hashes alone cannot authenticate an author.

Restoration uploads immutable blobs, then atomically imports catalogue evidence.
Original workflow/attempt/provenance identities remain unchanged; new Hub UUIDs
are separate transfer receipts. Imported records are read-only evidence and
cannot run or cancel their historical workflow. They can be queried, backed up
and exported again without the old working directory. A failed upload may leave
unreferenced immutable Hub artifacts; manage those with the Hub retention
policy. This is a result evidence bundle, not an executable backup of input
datasets, toolchains or component binaries.

Cache eligibility must explicitly be `deterministic-data` in the output policy;
use `disabled` for timing samples or unqualified backends. Completed verified
results are indexed by domain, frozen plan, step, entire campaign/policy,
implementation/backend manifest, parameters and immediate input artifact
identities. Manifests must pin all relevant code, RNG, metric and environment
dependencies. A changed manifest or domain cannot reuse an old entry.

```bash
tdi cache-request "$CAMPAIGN" run-0 file:result | python3 -c 'import json,sys; print(json.dumps(json.load(sys.stdin)["result"]))' > cache-request.json
tdi cache-get cache-request.json --allow-exact-reuse
```

Lookup verifies bytes again and returns `cache_hit`, original provenance,
`new_execution: false` and `fresh_timing_measurement: false`. The caller must
authorize reuse; lookup never inserts a fabricated new trial or silently
replaces a scheduled Hub step. Missing or corrupted blobs fail visibly.

## Bounds and trust profile

Transfers and complete bundles are at most 16 MiB; JSON result payloads at most
256 KiB; bundle inventories at most 4096 results; catalogue pages at most 200
records. Graph limits come from the existing pinned Graph/v1 validator. The
viewer emits at most 8 MiB, with 50-row HTML pages and 10-result API pages.
Use `after` pagination; catalogue event cursors are sequence numbers, other
page offsets count records. API errors are versioned JSON.

Store directories and component deployments must be trusted and immutable for
an execution lifetime. The fixture pins the wrapper, Python interpreter and
Rust binary, and verifies the Rust binary before and after execution. It does
not infer Git-source/build reproducibility from a declared SHA. Hub supervises
the process tree; this integration does not turn process supervision into an
OS sandbox or enforce the separate cgroup-v2 quota backend. Hard GPU quotas,
energy measurements and new concrete-model stage authorization are not claimed.

Restricted-reference roots are rejected before downloads. The operational
viewer contains only the explicitly admitted non-final catalogue. Hub inspect
authorization is deployment-wide, not per-artifact scientific clearance; use
a dedicated Hub/catalogue for this profile, never a mixed protected store.
No environment dump, arbitrary diagnostics download, evaluator labels, token
or final-series payload is automatically exported.

## Qualification

```bash
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=scripts python3 -m unittest -v scripts/test_tdi_engine_unit.py scripts/test_tdi_hub_admission_contract.py
TDI_HUBD_BIN="$PWD/../scirust-hub/target/debug/scirust-hubd" TDI_DURABLE_WORKER="$PWD/target/debug/examples/durable_worker" PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=scripts python3 -m unittest -v scripts/test_tdi_engine_integration.py
```

The integration suite requires actual binaries and fails if they are missing.
It covers real Rust calculation plus independent verification, restart without
another attempt, failed-worker persistence, cancellation from a second client,
lost submission response, cache misses under dependency/domain drift, byte and
provenance preserving export/restore, backups and viewer permission checks.
Boundary tests cover corrupted/truncated/oversized transport, redirect refusal,
strict JSON, unsupported origins, partial-file/fsync/disk-full failures,
pre-dispatch persistence failure, restricted input refusal and HTML escaping.
This qualifies the declared software fixture; it is not a production load
benchmark or a distributed hardware qualification.
