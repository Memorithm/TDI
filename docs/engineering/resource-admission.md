# Local resource admission and physical telemetry

`run-local-admitted` is an optional execution profile for a local Hub deployment
whose process workers share this client's host, CPU affinity and cgroup scope.
It samples real capacity, asks the separately built Elastic controller to admit
work, and lowers only the graph's `max_concurrency`. The graph's root plan,
seeds, step parameters, component pins, inputs, outputs and scientific domain
remain byte-for-byte unchanged after canonicalization. Hub remains the scheduler.

This is a new executable contract using `elastic admit-capacity`, separate from
the older non-executing `elastic.hub.run` partner description. That description's
authority flags remain false; they are not repurposed as execution permissions.
The numerical controller and actual permit actuation live in ElasticXxx PR #95.

## Capacity and measurement semantics

`python3 scripts/tdi_engine.py capacity` reports actual Linux `MemAvailable`, CPU
affinity, and every visible unified-cgroup ancestor's `memory.max`,
`memory.current` and `cpu.max`. Byte and quota/period units are explicit.
Only full CPU slots are admitted. Unknown layouts/sensors fail closed; no
missing limit is interpreted optimistically inside a delegated subtree. Limits
hidden outside a cgroup namespace are not observable and are outside this scope.

Readings include sensor version, local boot/environment identity, monotonic and
Unix sample times and sampling duration. The admission freshness deadline uses
the local monotonic clock. GPU memory, synchronized GPU time and energy remain
absent without qualified sensors. A measured capacity is not a RAM reservation,
a promise against another process consuming memory, or a hard process quota.
Use the separately qualified cgroup execution profile for hard resource limits.

`measured_process` records a particular child's `wait4` user/system CPU seconds,
Linux peak RSS in bytes, wall/launch nanoseconds, output sizes and exit status.
Wall time includes launch, I/O and up to 5 ms completion polling latency. RSS is
the child's high-water measurement, not a sum of concurrent descendants. File
output budgets are polled and bounded on capture; they are not kernel-enforced
file quotas. Timeout/output failures stay explicit. This helper executes trusted
pinned software, not hostile code.

## Execute an admitted campaign

Build the actual Hub and TDI counter worker using the sibling-checkout commands
in [operational-engine.md](operational-engine.md). Build Elastic separately:

```sh
cd ../ElasticXxx
# Check out the exact qualified source pinned in tdi-resource-admission.yml.
cargo build --locked -p elastic-cli
cd ../TDI
python3 scripts/tdi_engine.py capacity
```

Create a policy before execution. `memory_bytes_per_trial` is the operator's
declared envelope, not a memory value estimated from an outcome:

```json
{"max_concurrency":2,"memory_bytes_per_trial":67108864,"reserve_memory_bytes":134217728,"max_age_milliseconds":10000}
```

Use an existing prepared non-final plan, a running local process Hub and its
existing `TDI_HUB_TOKEN`. Supply the built binary's actual SHA-256 and the exact
source commit (a source declaration is not a build attestation):

```sh
python3 scripts/tdi_engine.py --hub http://127.0.0.1:8477 --allow-loopback-http \
  run-local-admitted plan.json --resource-policy policy.json \
  --elastic-worker "$PWD/../ElasticXxx/target/debug/elastic-cli" \
  --worker-sha256 "$ELASTIC_BINARY_SHA256" --source-commit "$ELASTIC_SOURCE_COMMIT"
```

The profile checks a loopback Hub and exact registered process components with
local program paths. The operator must ensure this deployment actually shares
the measured resource scope: loopback alone is not node attestation. Containers
with different cgroups and remote worker placement need their own node-specific
contract and are not qualified by this command.

Elastic computes the minimum of the requested maximum, available CPU slots and
available bytes after reserve divided by per-trial bytes. It applies the width
through its existing `Runtime::cycle_attempt` and real `TransactionalConcurrency`
ledger, checks the result, and returns the authoritative decision. TDI then
submits the graph at that width and verifies the exact width in the Hub snapshot.
The CLI's temporary Elastic ledger does not itself schedule TDI work: Hub enforces
the returned width. The Rust controller can also be embedded with its shared
permit handle around actual tasks. Neither route changes a scientific arm.

## Failure and recovery

Every attempted decision retains the real snapshot, complete Elastic request/
response, source/binary identity, cause and measured process cost as an immutable
catalogue event. Unexpected Elastic output retains a protocol failure and output
hash, never raw unselected logs. Use `events CAMPAIGN` with pagination to export
these versioned records; ordinary scientific bundle exports retain their existing
schema. A denied admission leaves a prepared campaign with no workflow or results
and returns exit 20. Contract errors before launch return exit 21.

The controller is bound to the original plan identity and this locally sampled
executor environment through required, separate CLI arguments. A request for a
different plan or environment is rejected before permit actuation. If process
launch fails (permissions, format or loader), admission retains the launch
attempt duration and errno. Child exit status, CPU use and RSS remain absent;
the campaign has a durable `resource-admission` failure event and no Hub work.

The link from original plan to admitted graph is durable before any Hub mutation.
That dispatch binding includes the policy, Elastic binary/source identities and
a content identity of the supplied root-artifact bindings, so a crash before Hub
submission cannot be retried with different roots. Resource admission is never
retrofit onto a campaign already dispatched through ordinary `submit`/`run`; such
a campaign is rejected unless a prior `resource-dispatch` event proves that its
Hub lifecycle belongs to this admission path.

Retrying the same admitted command reconciles an already-dispatched workflow. If
dispatch has not started, it samples fresh capacity and must re-admit the exact
same width and root bindings; it cannot change an already submitted graph. Stale
readings after Hub submission leave an admitted workflow unexecuted. Changed
resource policy, binary/source identity, root bindings, or insufficient capacity
for an existing width stops execution and requires an explicit cancel/replan.
Ambiguous submission uses the existing attach/inspect contract and never creates
another workflow automatically.

Qualification executes the real Elastic binary and Hub, reduces a three-trial
counter graph to width one, verifies all other canonical fields unchanged,
checks identical snapshots on retry, rejects a real impossible memory envelope
without creating work, and rejects retroactive admission of an already-submitted
ordinary campaign. Tests also execute a failing process, timeout, output-budget
failure and wrong binary identity. No GPU/energy claim is made.
