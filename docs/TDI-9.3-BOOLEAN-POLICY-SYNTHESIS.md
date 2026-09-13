# TDI-9.3 — Boolean Policy Synthesis

Status: **ACTIVE DESIGN / NON-FINAL** (TDI-9.3.0 representation calibration partially landed)

## Purpose

TDI-9.3 studies whether adaptive-inference decisions can be represented and searched as explicit Boolean policies over a frozen, leakage-safe observation boundary.

The series is a TDI-9 extension. It does not replace TDI-9.1, does not create a TDI-9.2 confirmation surface, and must not consume final or protected material.

Its generic object is

```text
p_t = P(observation_t)
a_t = F_bool(p_t)
```

where `P` converts only allowed current/past trajectory information into Boolean predicates and `F_bool` maps those predicates to an action admitted by the relevant TDI policy arm.

For a binary C2 decision, the minimal form is

```text
STOP_t = F_stop(p_t)
CONTINUE_t = NOT STOP_t
```

For C3, an ordered or otherwise explicitly resolved family of Boolean rules may select among

```text
CONTINUE
VERIFY
BACKTRACK / RECOVER
STOP
```

subject to the existing TDI-9 action vocabulary and resource envelope.

## Relationship to TDI-9.1

TDI-9.3 is **not** a TDI-9.1 freeze-resolution vehicle.

| Surface | Owner | May pin TDI-9.1 freeze fields? | May authorize TDI-9.2? |
| --- | --- | --- | --- |
| TDI-9.1 configuration freeze / readiness | TDI-9.1 | yes, only with Dev/Val evidence | no (flag stays false) |
| TDI-9.3 Boolean IR / experimental feature | TDI-9.3 design | **no** | **no** |

In particular:

- merging or extending TDI-9.3 Boolean representation/search does **not** pin
  `permitted_observation_vector` or any other unresolved 9.1 field;
- Boolean IR and `PolicyObservation` types are representation surfaces, not an
  experimental observation-vector pin;
- TDI-9.3 remains **ACTIVE DESIGN / NON-FINAL** until its own stage ladder
  (9.3.0+) is separately gated; it never silently upgrades 9.1
  `scientific_status`.

## Motivation from the current reference implementation

The present non-final C2 reference policy already has an explicit Boolean structure:

```text
STOP = enough_steps
       AND (
           terminal
           OR (
               residual_small
               AND delta_small
               AND margin_large
           )
       )
```

TDI-9.3 generalizes the representation and controlled search of such rules. It does not assume that a searched rule is better than the hand-specified reference.

## Relationship to BooleanLab BL-14

BooleanLab BL-14 and TDI-9.3 share reusable Boolean-policy machinery but answer different scientific questions.

- **BooleanLab BL-14** studies Boolean equations as a control plane for sparsity and develops generic Boolean-rule synthesis/evaluation methods.
- **TDI-9.3** studies whether Boolean rules over TDI trajectory observables improve adaptive-inference decision trade-offs.
- **SciRust** is the intended promotion target for generic, evidence-qualified Boolean-expression, synthesis, equivalence or complexity primitives that are not TDI-specific.

TDI must not silently import BL-14 empirical claims. Only generic mechanisms or explicitly qualified policies may cross the boundary.

## Experimental ladder

### TDI-9.3.0 — Representation calibration

Goal: verify that the Boolean IR can exactly reproduce declared hand-written reference rules and account for predicate reads, logical operations and expression depth deterministically.

**Landed (exact / engineering, non-pinning):** the experimental module now exposes
`reference_c2_hand_stop`, `reference_c2_stop_expression`, and
`reference_c2_stop_policy`, and CI-covered unit tests exhaustively compare the IR
against the hand Boolean on all `2^5` predicate vectors. Complexity for the
reference C2 STOP shape remains exact (`5` predicate reads, `4` logical ops,
depth `4`). Fail-closed missing-predicate / forbidden-action behavior remains
covered.

Acceptance criteria:

- exact output equivalence on exhaustive bounded predicate tables for the target rule (**met for the documented C2 STOP shape**);
- fail-closed behavior for missing predicates or forbidden actions (**met**);
- deterministic complexity accounting (**met for the reference C2 STOP shape**);
- no access to final-evaluation data (**met**; experimental feature only).

TDI-9.3.0 calibration does **not** freeze observation-to-predicate mappings,
thresholds, or `permitted_observation_vector`, and does **not** authorize
TDI-9.1 pins or TDI-9.2.

### TDI-9.3.1 — C2 Boolean stopping-rule search

Question: under a frozen observation-to-predicate mapping and matched compute envelope, can a searched Boolean `CONTINUE/STOP` rule improve the quality/compute frontier relative to the hand-written C2 reference and simpler Boolean baselines?

Required baselines include:

- current hand-written C2 structure;
- single-predicate stopping rules;
- conjunction-only and disjunction-only bounded grammars where feasible;
- density/complexity-matched random Boolean rules as a search-control baseline.

### TDI-9.3.2 — C3 multi-action Boolean policy search

Question: can an explicit Boolean rule set choose `CONTINUE`, `VERIFY`, `BACKTRACK/RECOVER` or `STOP` under the same frozen C3 observation and resource boundary with a non-dominated quality/resource trade-off?

The experiment must define conflict resolution exactly. Ordered first-match rules are allowed, but rule order is part of the policy identity and search space.

### TDI-9.3.3 — Rule complexity and stability

Question: how do literal count, predicate reads, logical-operation count, depth and rule-set size affect transfer, robustness and decision stability across allowed non-final task strata?

A more accurate but substantially larger rule is not automatically preferred.

### TDI-9.3.4 — Relational and temporal predicates

Question: after the base predicate contract is frozen, do explicitly defined relational or temporal predicates improve policy decisions beyond independent scalar-threshold predicates?

Examples may include trend, persistence, agreement/disagreement or checkpoint relations, but none are admitted until separately declared and leakage-reviewed.

### TDI-9.3.5 — Search-method comparison

Question: under matched candidate-evaluation budgets, how do bounded exhaustive search, SAT/MaxSAT, CEGIS, evolutionary search or Forge-style policy search compare in the quality/complexity frontier they recover?

The search method is not allowed to inspect final TDI-9.2 material.

## Multi-objective evaluation

TDI-9.3 retains a vector of outcomes rather than collapsing evidence prematurely:

```text
J = (
    task_quality,
    compute_used,
    verifier_cost,
    recovery_cost,
    policy_predicate_reads,
    policy_logical_ops,
    policy_depth,
    policy_size,
    decision_stability
)
```

Any scalar search objective must freeze its weights before evaluation. Reporting must preserve the underlying components.

## Information boundary

TDI-9.3 does not define new privileged observations. Candidate Boolean predicates may be constructed only from fields already authorized by the relevant TDI-9 stage or by a later explicit freeze.

In particular, policies must never receive:

- evaluator-owned target labels;
- future trajectory values;
- alternative-arm outcomes;
- final seed identities or future entropy before its authorized reveal;
- TDI-7.2 protected material;
- TDI-8.2 protected material.

## Implementation boundary

The first code lives only behind `tdi-ai`'s `experimental` feature.

The initial module provides:

- deterministic Boolean expression evaluation;
- ordered Boolean action rules;
- TDI policy-arm action validation;
- exact reference counts for predicate reads, logical operations and expression depth;
- fail-closed handling of malformed predicate vectors;
- TDI-9.3.0 representation-calibration fixtures for the hand-written C2 STOP
  Boolean (`reference_c2_*`), including exhaustive truth-table equivalence tests.

It deliberately does **not** yet provide:

- a chosen TDI-9 predicate schema or observation-vector pin;
- thresholds derived from evidence;
- a search algorithm;
- integration into the C2/C3 reference evaluator;
- a confirmatory runner;
- access to TDI-9.2 final derivation or evidence.

Those additions require the corresponding TDI stage gates.

## Promotion rule

A TDI-9.3 candidate may be promoted only after:

1. the relevant observation-to-predicate boundary is frozen;
2. development and validation search remain disjoint from final confirmation;
3. matched reference policies are evaluated under the same resource envelope;
4. policy complexity and decision overhead are charged explicitly;
5. negative, equivalent and harmful candidates are retained;
6. any future final test follows the already-declared TDI-9 autonomous confirmation discipline.

A positive TDI-9.3 result would support only the bounded claim that a tested Boolean policy improved a declared adaptive-inference trade-off in the tested regime. It would not establish a universal reasoning law or imply that Boolean policies are always preferable.
