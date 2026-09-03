# TDI-9.x — Autonomous Adaptive Inference Dynamics Research Programme

TDI-9.x is the distinct TDI scientific line for adaptive inference dynamics.
It studies independently defined mechanisms that decide how much computation to
spend and when to continue, verify, backtrack/recover, or stop.

TDI-9 is motivated by observable adaptive-inference behavior in modern reasoning
systems. It does not claim knowledge of any proprietary model architecture.

## Stage map

| Stage | Purpose | Status |
| --- | --- | --- |
| TDI-9.0 | Freeze adaptive-inference questions, policy ladder, evidence rules and autonomous-confirmation contract | active |
| TDI-9.1 | Build bounded deterministic policy/evaluator stack and run agent-driven development/validation | blocked until TDI-9.0 is merged and frozen |
| TDI-9.2 | Execute autonomous sealed confirmation from future-derived entropy | future, does not yet exist |
| TDI-9.3+ | Ablation, transfer, tool interleaving, routing and ecosystem extensions | conditional on evidence |

## Policy ladder

- **C0 — fixed compute:** every instance receives the same frozen compute allowance and fixed stopping schedule.
- **C1 — static preallocation:** compute allowance may depend on task class or other information available before inference begins, but not on trajectory observations.
- **C2 — adaptive stopping:** a bounded policy observes only allowed trajectory signals and chooses `CONTINUE` or `STOP`.
- **C3 — adaptive verification/recovery:** C2 plus explicit `VERIFY` and `BACKTRACK`/`RECOVER` actions under the same declared maximum-compute envelope.

Primary scientific contrasts are C2 versus C0 and C3 versus C2. C1 is a
competent static-allocation control that distinguishes true trajectory adaptation
from simply knowing a task class in advance.

## Agent-first research model

TDI-9 is designed for autonomous AI research agents. Agents may propose bounded
mechanisms, implement software oracles, run development and validation campaigns,
falsify candidates, open and repair PRs, and promote only evidence-qualified
mechanisms.

No human confirmation token is part of TDI-9. Anti-leakage is instead enforced
by:

1. immutable preregistration and evaluator manifests;
2. disjoint generation/development/validation/final domains;
3. no final seed list or final dataset before the final entropy event;
4. a frozen deterministic derivation from a future public randomness value or equivalent immutable future-derived entropy;
5. machine-verifiable provenance and CI gates;
6. a prohibition on selecting, retrying or replacing the future entropy value based on results.

## TDI-9.1 boundary

TDI-9.1 may search policies and observables only on non-final development and
validation surfaces. It must freeze before TDI-9.2:

- concrete task battery and difficulty strata;
- maximum compute envelope and exact operation accounting;
- memory/accounting rules;
- C0/C1/C2/C3 deterministic reference semantics;
- allowed policy observation schema;
- verifier semantics and backtracking semantics;
- quality/compute decision margins and statistical procedure;
- generator and non-final seed domains;
- final entropy source identity, future target event/round and seed derivation algorithm;
- typed rejection taxonomy;
- result/provenance schema.

## TDI-9.2 autonomy boundary

TDI-9.2 must not exist as a runnable confirmation surface during TDI-9.0.
During TDI-9.1, final-evaluation code may be prepared only after the complete
final derivation contract is frozen and only if it cannot know the final entropy
value in advance.

Once the preregistered future entropy becomes available, an agent/CI path may
derive the final seeds and execute exactly the frozen evaluator. No discretionary
policy, metric, task, seed, retry or entropy-source change is allowed after the
entropy reveal.

## Ecosystem sequence

1. TDI-9.0 preregistration and bootstrap;
2. TDI-9.1 deterministic reference evaluator and agent-driven falsification;
3. TDI-9.2 autonomous sealed confirmation if TDI-9.1 reaches its readiness gate;
4. ADA may propose semantic mechanisms and observables, but final material never enters candidate generation;
5. Forge may search policies only after a leak-safe TDI-9 search contract exists;
6. ITD may contribute versioned structural trajectory diagnostics through explicit adapters;
7. reusable primitives may move to SciRust;
8. NNIS and ElasticXxx may receive evidence-qualified policies for real execution/runtime work.

## Mandatory isolation

TDI-9 must not read, generate, modify or reinterpret TDI-7.2 protected payloads.
TDI-9 must not create, access or reinterpret a TDI-8.2 surface. TDI-8 and TDI-9
may coexist because they answer different scientific questions and maintain
separate final-evaluation lineage.

## Scientific interpretation rule

A positive TDI-9 result would establish only the bounded tested claim. It would
not establish that a named commercial model uses the same mechanism, that the
policy generalizes to language modelling, that tool use is solved, or that a
particular GPU/runtime implementation is faster. Those require separate direct
evidence.
