# TDI-28.0 — Energy-aware Boolean event algebra (Stage-0)

Status: **experimental reference only; no frozen preregistration, physical energy result, confirmatory execution or model promotion**.

## Scientific question

Can an event-driven Boolean system represent resource cost *within* its
compositional operations (approach A), and can a separate resource-aware
decision calculus choose whether to evaluate, reuse or defer an operation
(approach B) while preserving required semantics? Are any gains preserved
when every cost, including bookkeeping, memory and idle power, is accounted
for on the same hardware?

These approaches are complementary, not two descriptions of one method.

## Stage-0 reference implementation

The standalone Rust oracle is
`tdi-ai/src/tdi28_energy_algebra.rs`, opt-in behind the
`tdi-ai/experimental` feature:

- **A — attributed operation**:
  `(gate, inputs, revisions, previous_state) -> (output, work_vector)`.
  Every required or optional event is evaluated; operation cost remains a
  *vector of reference event counts*, rather than an invented energy value.
- **B — energy-governed transition**:
  `(demand, revisions, cache) -> EVALUATE | REUSE | DEFER`.
  REUSE requires an exact match of both dependency revisions **and** gate
  identity. DEFER is permissible only for an optional observation. Required
  outputs cannot be silently suppressed.
- **A+B**: run B, attribute its actual reference work using A's same work
  accounting. This is the intended combined candidate.

At Stage 0 both are *software semantics*, not a calibrated physical model.
The input reader is a lazy callback: the reference oracle can test that
REUSE and DEFER do not request value reads through that callback. It does
not claim that a CPU issues no cache-line loads or that a compiler produces
a specific instruction sequence.

## Four experimental arms (future Development protocol)

| Arm | Policy | Cost visibility | Role |
| --- | --- | --- | --- |
| C0 | Always evaluate | Operations and outcomes only | Semantic reference |
| C1 | Always evaluate | A: complete declared work vector | Attribution control |
| C2 | Reuse/defer if admissible | Same work vector | B: decision-control candidate |
| C3 | Reuse/defer if admissible | A+B composed into selection/score | Joint candidate |

C0 and C1 produce identical outputs and execute identically: their distinction
is **measurement/selection semantics**, not runtime speed. C2 and C3 must
provide identical mandatory outputs to C0 under truthful revision stamps;
C3 differs from C2 only when a later, separately frozen decision rule
uses pre-observable, calibrated cost information. Stage 0 does **not** yet
implement cost-based optimization or an adaptive learned controller.

## Measurement protocol to freeze before results

1. **Correctness first.** Freeze Boolean function, dependency-tracking
   invariants, event traces, which outputs are mandatory, and permitted
   deferral horizon. Reject or report missed deadlines and stale outputs;
   do not score them as saved energy.
2. **Matched inputs and budgets.** Replay the same event sequence and
   initial state through all arms; separately report active events,
   gate evaluations, input/revision/cache reads, bit transitions,
   deferred/pending events, event dispatches, and idle ticks.
3. **Model costs carefully.** Use the full work vector first. A weighted
   scalar is a *proxy* with explicit versioned positive coefficients, not
   joules. Calibrate coefficients outside the optimization split on the
   target CPU, with separate reference readings for idle baseline,
   memory movement and communication. If a physical sensor is absent,
   label energy **unmeasured**.
4. **Strong negative controls.** Change cache-to-compute cost ratios;
   vary event density, revision-change rate, cache locality, deferral
   deadlines and idle duration; test adversarially expensive cache probes.
   Preserve the possibility that policy B is worse than A.
5. **No hindsight.** B's decision sees only current demand, declared
   revisions and valid cached identities. No future events, expected target
   labels, output oracle or alternative-arm outcome may enter its policy.
6. **Separate observables.** Report required-output exactness, missed
   deadlines, retained deferred work, logical reads, actual memory bytes,
   elapsed CPU/wall time, power integration and measurement uncertainty
   separately. Do not infer energy from work counts alone.
7. **Development/Validation then confirmation.** Candidate traces and
   scoring coefficients must be frozen before Validation; a protected
   final/confirmatory population requires a later independently reviewed
   gate and separate authorization.

An additive compositional work vector can be an exact property of the
chosen *reference accounting*, but not automatically of physical power
consumption. Real energy may include non-additive effects from caches,
concurrency, background tasks and voltage/frequency scaling. Consequently,
any measured hardware effect must be independently evaluated.

## Research lineage, boundaries and possible reuse

- **TDI-26 (V888 sparse recurrent dynamics)** is the future primary
  integration bench for matched fixed-step/event-driven comparisons,
  but its Stage-0 freeze and TDI-26.1 gate are not yet complete.
  This Stage-0 line does not amend TDI-26's source, matched control ladder
  or protected evaluation boundary.
- **TDI-9** concerns adaptive inference actions. TDI-28 does not reinterpret
  its frozen policy or final entropy contracts.
- **TDI-21** concerns Boolean relational alternatives to attention. TDI-28
  does not claim an attention replacement.
- **SML-GENIUS / V888-BOOLEEN / FLAT-ATTENTION** are downstream candidates
  for versioned adapters only after separate qualification.
- **RemoteOps** remains the only remote hardware control plane; this PR
  does not provision runners or run physical watt measurements.
- No new NVIDIA dependency, non-Rust runtime, final seed set, production
  integration, consciousness claim or model-quality claim is authorized.

## Falsifiable outcomes

A null result is expected if revision/cache probes offset skipped input reads,
if deferral merely displaces the same work, or if physical idle/DRAM costs
dominate. A rejection is required if the policy suppresses mandatory outputs,
relies on revision identities that can silently alias changed values, or
fails to disclose reference accounting overhead. Retain null, negative,
harmful and inconclusive findings without renumbering.

## Stage-0 reproducibility

```bash
cargo test -p tdi-ai --features experimental --lib tdi28_energy_algebra --locked
cargo clippy -p tdi-ai --features experimental --all-targets --locked -- -D warnings
cargo fmt --all -- --check
```

The current Stage-0 tests exercise conditional input-reader invocation,
stale cache handling, mandatory-vs-optional output semantics, compositional
overflow, idle cost and a counterexample where expensive cache reads reverse
the expected preference. They are qualification tests, **not an energy
experiment or scientific verdict**.


## Stage-0 extension: bounded event streams and deferred-work debt

The next implementable primitive is `tdi-ai/src/tdi28_trace.rs` (experimental
facade only). It replays an **identical, caller-supplied, bounded** stream
through the always-evaluate and energy-governed references. This is
Development-only oracle scaffolding, not a frozen experimental campaign.

Each event declares logical tick, gate, both input revision stamps, input
values known only to the offline evaluator / lazy evaluation callback, and
mandatory-versus-optional demand. The offline preflight rejects a revision
stamp that yields conflicting values, non-increasing event ticks, an invalid
terminal horizon, and unbounded event populations (maximum 4096 events).
The policy never receives expected answers, future events, or alternative-arm
outcomes. Values in preflight are evaluator-owned and **not** counted as online
policy reads; physical benchmark protocols must separately measure their real
provenance and overhead.

Every returned trace includes independent per-arm work vectors, output
observations, decisions, an explicit deadline-free optional work ledger, and
idle ticks (initial gaps, inter-event gaps, trailing idle time). The ledger
distinguishes:
- **deferred requests**: optional outputs absent at their original event;
- **resolved requests**: deferred work later satisfied for the same gate and
  dependency identity;
- **superseded requests**: optional outputs were never produced because a
  newer gate/dependency identity replaced them;
- **outstanding requests**: still awaiting resolution or supersession.

An unsettled trace **cannot** return a scalar proxy-cost verdict. A settled
trace retains the superseded count for joint quality/coverage review; this
does not silently equate lost optional output to preserved utility.
This first slice has no mandatory deadlines for optional work. A later,
separately frozen experiment must declare bounded deferral horizons and
evaluate deadline misses before interpreting energy savings.

### Overhead accounting correction

The Stage-0 `Work` vector now separately records
`cache_validation_comparisons` (one composite cache-validity predicate
for an event with an existing cache in the governed arm). The
always-evaluate arm does not need to execute this predicate; it still
counts a cache probe if previous output is consulted to track transitions.
This explicitly admits negative controls in which the validation overhead
outweighs skipped gate evaluations. The new counter is a logical work unit,
not a calibrated memory access or an instruction count.

### Qualification and remaining gaps

```bash
cargo fmt --all -- --check
cargo test -p tdi-ai --features experimental --lib tdi28_ --locked
cargo clippy -p tdi-ai --features experimental --all-targets --locked -- -D warnings
```

Future work still needed: an online cost-based C3 policy with preobservable
calibrated coefficients, memory-write and inter-node communication
instrumentation, explicit optional output deadlines and utility, a
preregistered Development/Validation split, then independently controlled
RemoteOps physical measurements. This slice reports no watts, joules,
real-model quality gains, or evidence of consciousness.

## Stage-0 C3 budgeted control (experimental-only extension)

`tdi-ai/src/tdi28_c3_policy.rs` introduces a third, **prospective**
reference policy beside the always-evaluate and unconditional-reuse arms.
This is not a scientific C3 superiority result or a calibrated hardware
energy optimizer.

A caller supplies a versioned `ProxyBudget` with a nonzero cost-model
revision, explicit nonnegative integer weights for all reference-work classes,
and a per-event upper bound. The planner receives only the current tick,
gate identity, input revision stamps, optional cached output, and obligation
status; actual input values are read lazily **only** after an EVALUATE decision.

- **EVALUATE** is chosen if no reusable state exists, or when the *declared*
  cost of validating a cache is not lower than recomputation. A conservative
  allowance for a possible output transition is included in its budget.
- **REUSE** is chosen only if cache validity is worth testing *and* both gate
  and dependency revision stamps match. Validation overhead is counted.
- **DEFER** is possible only for optional work before its declared absolute
  logical deadline when the preferred action exceeds the immediate budget.
  Deferral itself is priced, not free. Required/due work that cannot fit its
  declared resource budget is **rejected**, not silently omitted.

The `DeadlineLedger` holds bounded pending obligations (at most 4096).
Its caller must deliver explicit wakeups; it does not promise an autonomous
operating-system timer. A wakeup later than the pending deadline fails closed.
A deferred obligation may be registered only using a sealed C3 execution
record proving a matching DEFER decision; it is resolved only by supplying
another actual C3 execution with a produced output, matching
gate/revision identity, and a logical tick equal to the ledger's current
wakeup. A previously completed output cannot be replayed as new evidence.
The ledger rejects mismatches, missed deadlines and counter overflow rather
than fabricating success.
An explicit cancellation is recorded separately from success.
Outstanding obligations block any settled-episode interpretation.

### Limits of this slice

- The weights are synthetic scenarios: `proxy_cost` is **not** a joule,
  physical power or hardware instruction count.
- The reference policy only constrains per-event work, **not** aggregate
  memory traffic, wall time, process scheduling or global power.
- C3 does not yet choose a learned policy; any later adaptive calibration
  requires a separate preregistered Development/Validation protocol.
- The ledger must be wired to a scheduler with real deadlines before any
  production claim. Its interface is deliberately independent of TDI-26's
  unfrozen topology/stream protocol.
- All protected/confirmatory surfaces remain unavailable.

Qualification:

```bash
cargo fmt --all -- --check
cargo test -p tdi-ai --features experimental --lib tdi28_ --locked
cargo clippy -p tdi-ai --features experimental --all-targets --locked -- -D warnings
```


## Stage-0 four-arm matched event comparator

`tdi-ai/src/tdi28_four_arm.rs` implements deterministic, bounded
Development-only matched replay through **C0, C1, C2 and C3**. This closes
the reference-comparison gap between the existing paired oracle (C0/C2)
and the separate C3 policy. It does **not** constitute frozen validation,
measured electricity savings or a trained cognitive mechanism.

- **C0**: unconditional Boolean evaluation, reference outputs.
- **C1**: the **same execution and reference work** as C0 with explicit
  attribution. There is deliberately no fabricated runtime difference.
- **C2**: fixed prospective reuse/defer policy from `tdi28_energy_algebra`
  with optional debt ledger from `tdi28_trace`.
- **C3**: cost-guided prospective selection using one declared cost model,
  a per-event ceiling and absolute optional deadlines. Its ledger requires
  matching execution proof and rejects missed logical deadlines.

All four arms see identical gate inputs, current revisions, demand,
identities, logical ticks, declared cost weights, and per-event ceilings.
The reference preflight rejects inconsistent revision aliases before any
candidate replay. Offline evaluation callbacks are *not* passed as
online-policy knowledge; no future inputs, target values, alternative-arm
outputs, or score improvements enter C3 selection.

### Strict quality parity gate

The comparator verifies each required C3 output and any delivered optional
C3 output against C0. Its `strict_quality_parity` additionally requires:

1. every optional result was present **at its original event**, in both C2
   and C3; a deferred output subsequently resolved is not retroactively
   counted as present;
2. all C2 and C3 pending obligations are empty and none was cancelled;
3. every event's delivered value exactly equals the C0 value.

`strict_proxy_scores` evaluates only with the original coefficient vector
stored inside the report: it does not accept post-hoc outcome-dependent
reweighting. It returns **None** rather than present a cost
comparison as equal-utility evidence if any of these conditions fail.
The raw diagnostic work vectors and per-event records remain available,
including negative results. A later task-utility/coverage metric may
permit non-strict evaluation, but must be specified and frozen separately.

### Shared budgets, explicit violations and incomplete accounting

Every event declares one common ceiling. **C0/C1/C2 are diagnostic
unconstrained baselines**: any individual event exceeding this ceiling is
reported as a violation rather than being hidden or retroactively modified.
C3 rejects an operation it cannot admit, including overdue mandatory work.
If C3 cannot complete a mandatory event, no positive four-arm report exists.

Logical idle ticks are charged symmetrically including the initial interval,
inter-event gaps, and the declared tail to the horizon. All vector
components and weighted proxy scores use checked arithmetic. Important
limitations: validation/preflight, scheduler, ledger bookkeeping, actual
memory bytes/communication traffic, host runtime and power-supply losses
are not yet physically metered. C1 is an accounting contrast, not a
distinct executable optimization. Results must not be labelled watts
or joules.

### Stage-0 deterministic controls

The new Rust tests cover strict equal-output comparison on unchanged and
changed inputs, high cache-validation overhead (C3 chooses recomputation
where C2 reuses), optional debt resolved later **without** retroactive
equal-quality claims, pending and missed deadlines, required budget rejection,
invalid deadline shape, revision consistency and per-event ceiling
exceedances. Future campaigns must freeze split assignments,
task-equivalence criteria, deadline/utility requirements, hardware
resource observations and provenance **before** confirmation.

## Stage-0 memory and communication transfer algebra (synthetic contract)

`tdi-ai/src/tdi28_traffic.rs` adds a declared transfer accounting vector
with independent components:

```text
Traffic = (logical_memory_read_bytes,
           logical_memory_write_bytes,
           logical_communication_bytes,
           logical_message_count)
```

These are **declared logical API-level quantities** derived from the
work vector, not CPU cache-line counts, DRAM controller statistics,
socket traffic, packet captures, actual power, or physical joules.

An explicit nonzero-versioned `TransferProfile` declares assumed byte
volumes per value/revision/cache read, cache validation, output write,
state transition, deferral queue write, and dispatch event, plus assumed
message sizes and counts for evaluation, cache validation, dispatch, and
deferral. All transfer dimensions are separate from Boolean gate counts.
Intra-core/local communication can be represented by explicit zeros;
this is a *scenario*, not a measurement.

`Traffic::checked_add` defines checked vector composition. A separate
`TransferPrices` assigns integer **proxy** costs to memory read bytes,
write bytes, communication bytes and messages. A nonzero profile version
is not proof of calibration. The proxy scalar is additive by definition;
real hardware energy can be non-additive due to memory hierarchies,
clocking, voltage scaling, batching and interconnect contention.

`replay_four_with_traffic` binds the synthetic scenario **before**
replaying the same four arms. `TrafficReport` retains its original
matched replay and frozen transfer contract, per-event transfer vectors,
per-arm aggregate vectors, and score derivation. To prevent silent
post-result optimization, the public strict score accessor accepts no
replacement weights. It checks equal per-event output coverage and
unsettled optional debts exactly as the four-arm comparator does. Raw
memory/communication dimensions remain visible for failed/negative/
inconclusive comparisons, without claiming equal task utility.

### Counterexamples and limits

The Development test suite includes a case where C2 avoids Boolean
operations but its declared **remote cache-validation communication**
cost dominates. Thus an apparent Work-only advantage reverses when
the same predeclared transfer scenario is considered. C3 currently
selects based on its original logical Work proxy and **does not**
claim to optimize this newly declared traffic cost. A future
traffic-aware policy must price all preobservable transfers *inside*
its decision and re-run matched correctness, debt, and budget gates.

The transfer profile excludes runtime allocations, scheduler wakeups,
ledger traversal, offline revision consistency validation, memory
hierarchy effects, communication retries, and unobserved background
power. Additional empirical counters/physical metering via RemoteOps
are required before discussing actual energy consumed. Do not label
these values "energy saved", "measured bytes", "DRAM reads" or "watts".

### Qualification gate

```bash
cargo fmt --all -- --check
cargo test -p tdi-ai --features experimental --lib tdi28_ --locked
cargo clippy -p tdi-ai --features experimental --all-targets --locked -- -D warnings
```

The test set covers C0/C1 exact parity, matched C0/C2/C3 outputs,
communication-vs-compute ranking reversal, local communication control,
optional result coverage, vector associativity and arithmetic overflow.
No TDI-26 topology/confirmatory integration is authorized by this slice.

## Stage-0 traffic-governed C3: prospective memory/communication pricing

The new opt-in Rust entry point `replay_four_traffic_governed` now binds
a single versioned `TrafficContract` and its immutable work model
**before** replaying event data, then compiles the linear transfer
accounting into a synthetic per-operation budget:

```text
effective_weight(operation)
  = logical_weight(operation)
  + transfer_price(declared_traffic(one operation))
```

Its C3 selector uses the combined weights to choose EVALUATE, REUSE or
DEFER, including memory read/write assumptions, declared communication
bytes and message counts. Cache-validation traffic is included in the
prospective cost of reuse; mandatory work over budget fails closed.
The compilation uses checked u128 math and rejects per-operation prices
that cannot be represented in the C3 evaluator's u64 coefficient model.
A unit-work conservation test checks that this compilation agrees exactly
with independently composed Work+Traffic accounting.

`replay_four_with_traffic` remains the **legacy Work-only decision**
control: it prices the same traffic only after all actions. The report's
`TrafficBudgetMode` distinguishes the two treatments explicitly. The
four-arm comparison retains C0/C1/C2 policies unchanged under an identical
declared combined ceiling and emits budget exceedance diagnostics for
unconstrained baselines. For a traffic-governed C3, the underlying
`FourArmReport::cost_model` represents the combined selection budget;
`TrafficReport::strict_total_proxy_scores` deliberately recomputes using
the **original** Work coefficients plus the declared traffic once, avoiding
double-charging. Equal-output, full-coverage and no-debt requirements remain
necessary before a comparable scalar score is returned.

The tests include a remote cache-validation counterexample:
a Work-only C3 may reuse an apparently cheap cache entry, whereas a
traffic-priced C3 instead recomputes because a declared remote validation
message is too expensive. They also reject invalid contract revisions,
unrepresentable coefficients and required work that cannot satisfy
the **total** budget. These are transparent scenario counterexamples,
not measured performance, calibrated joules, real memory traffic, or
evidence of industrial deployment.

The transfer algebra is *linear by contract*; measured memory hierarchy
effects, batching, queueing, scheduling, physical energy and communication
contention may not be additive. A subsequent study must freeze target
hardware, instrumented traffic, power-meter provenance and workload
partitioning under RemoteOps prior to any real-energy claim.

## Stage-0 physical-meter evidence intake (qualification scaffold)

The opt-in `tdi-ai/src/tdi28_meter_evidence.rs` module is a
**measurement-evidence validator**, not a device reader, physical
calibration, or confirmatory analysis. It accepts caller-supplied,
non-wrapping cumulative microjoule readings with strictly increasing
monotonic timestamps. A provenance record includes declared sensor ID,
domain (CPU package or whole system), source type, resolution, and
nonzero SHA-256 digests of host configuration, binary, workload and
isolation manifest. These are **unverified assertions by the submitter**;
the Rust API does not authenticate them, prove sensor installation, or
control any machine. Real collection must occur via RemoteOps only.

Each individual C0 or C3 trial includes a meter window for work,
**two idle windows bracketing that work**, and an observed quality record.
Counter reset, wraparound, zero-duration windows, non-bracketing idle
windows, and context mismatch are rejected rather than silently
estimated. Each idle window provides a distinct estimated baseline
contribution to the work interval. The *range* of the two
idle-corrected values is retained, including negative endpoints caused
by idle overcorrection. This is a sensitivity interval for drift, **not**
a statistical confidence interval or a precisely measured dynamic
energy component.

The matched C0/C3 comparison requires at least four pairs, no more
than 128, unique pair/run IDs, balanced AB/BA ordering, and actual
chronological consistency with the declared order of the samples and
the uninterrupted cumulative meter epoch. This ordering check spans
**both the trials within a pair and the boundaries between successive
pairs**; overlapping intervals or a counter reset between pairs fail
closed, even if each individual pair looks valid. The supplied pair
sequence is chronological; pair IDs are uniqueness labels, not sortable
timestamps. All pairs use one declared
source, domain, configuration, workload, binary and isolation manifest
and a single explicitly identified Development **or** Validation split.
Data from those splits cannot be pooled by this API.

Qualification additionally requires exact Stage-0 reference parity,
and self-reported *per-run* required/optional correctness, deadline and
outstanding-work counters matching the reference expectations. The API
cannot independently prove the actual program produced those outputs:
that requires authenticated run artifacts and separate physical audit.
Unqualified trials must not be presented as equal-quality experiments.

Results report every paired C0-minus-C3 idle-adjusted microjoule
interval, sign-ambiguous pairs, positive/negative pairs and a
descriptive central-rank interval. A positive interval means only that
the **submitted** energy counters show C3 below C0 under both idle
brackets. It is not proof of a statistically significant or causal
energy saving; no confidence interval, model selection, attribution
to particular Boolean operations, automatic tuning, or calibration
coefficient is inferred from this scaffold.

### Remaining work before hardware qualification

1. Establish a versioned measurement manifest, sensor-domain
   availability and resolution for a named RemoteOps runner; ensure
   dedicated quiet time without altering productive host services.
2. Implement a RemoteOps-only, **read-only** meter collector that
   preserves raw readings and provenance, detects wrap/reset, and
   records idle-before/active/idle-after intervals and actual output
   quality on each arm. Do not invent sensor values.
3. Freeze matched workloads, AB/BA allocation, measurement environment,
   development/validation split and acceptance thresholds *before*
   any physical observations are used for parameter tuning.
4. Review idle drift, shared-host interference, power supply boundary
   and measurement uncertainty; retain null/harmful/inconclusive runs.
5. Only then consider calibrated coefficients or conditional
   downstream adapters to V888/SML-GENIUS.

The new unit tests use fabricated counter fixtures solely to prove
input-rejection, arithmetic, ordering and coverage semantics. They do
not imply that RemoteOps collected joules or that any model is more
energy efficient. Protected TDI-26 and historical final-holdout
surfaces remain unaffected.
