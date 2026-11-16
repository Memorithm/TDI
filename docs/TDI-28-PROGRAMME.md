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
