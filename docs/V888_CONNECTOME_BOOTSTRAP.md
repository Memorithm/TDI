# TDI-22.x — BANC V888 sparse recurrent dynamics bootstrap

Status: Stage-0 bootstrap. No confirmatory run is authorized by this document.

## Independence from frozen programmes

TDI-22 is a new research line. It must not alter, read or repurpose protected final/confirmatory surfaces from TDI-7, TDI-8, TDI-9, TDI-11, TDI-12 or TDI-21.

TDI-8 remains the ASSR/ASSR-H programme. TDI-22 studies topology and sparse recurrent dynamics. Cross-series comparisons may be added only through a later explicit protocol.

## Source constraint

Only FlyWire BANC v888 is in scope.

Codex currently identifies BANC v888 as Female Adult Fly Brain and Nerve Cord, snapshot 2026-05-20, with 158,262 neurons and 3,037,361 aggregated connections.

Sources:
- https://codex.flywire.ai/?dataset=banc
- https://codex.flywire.ai/faq

Raw BANC data must not be committed to TDI. TDI consumes deterministic descriptor/topology artifacts through a versioned SciRust interface.

## Primary question

Do V888-derived structural properties provide predictive or computational value beyond competent sparse-graph controls under matched resource budgets?

TDI-22 must be able to conclude no.

## Architecture/control ladder

Provisional development arms:

- C0: dense recurrent control where computationally feasible;
- C1: random sparse graph at matched node/edge budget;
- C2: in/out-degree matched graph;
- C3: degree + reciprocity matched graph;
- C4: degree + reciprocity + modularity matched graph;
- C5: V888-derived structural-prior graph;
- C6: C5 plus bounded associative memory;
- C7: C5 plus event-driven execution;
- C8: trainable sparse graph initialized from the V888 prior;
- C9: trainable sparse graph initialized from a matched random control.

These names are provisional until TDI-22.0 freezes them.

## Stage sequence

### TDI-22.0 — scope/preregistration bootstrap

Freeze:
- source artifact identity;
- descriptor panel;
- topology generators;
- task families;
- resource budgets;
- allowed observables;
- intervention families;
- statistical decision rule;
- split/seed discipline;
- final-holdout authorization mechanism.

No final data may be generated before freeze.

### TDI-22.1 — deterministic evaluator

Implement Rust evaluator using SciRust primitives:
- identical task streams across arms;
- exact node/edge/memory accounting;
- deterministic state traces;
- event counters;
- task outputs;
- failure/timeout semantics.

### TDI-22.2 — topology identifiability controls

Before outcome tests, verify that the declared descriptors actually distinguish:
- random sparse;
- degree-matched;
- higher-order matched;
- V888-derived arms.

If V888 becomes indistinguishable under the descriptor panel, revise the scientific question before confirmatory work.

### TDI-22.3 — intervention-conditioned trajectories

Run development-only interventions:
- random node silencing;
- random edge removal;
- targeted high-degree node silencing;
- inter-module cut;
- input perturbation;
- state perturbation.

Measure recovery trajectories without assuming that recovery is beneficial.

### TDI-22.4 — predictive descriptor test

Ask whether early trajectory descriptors predict later task deficit beyond:
- degree;
- centrality;
- component size;
- event count;
- activity density;
- baseline task confidence/score.

Use out-of-sample evaluation.

### TDI-22.5 — event-driven versus fixed-step semantics

Scientific and systems separation:
- first establish semantic parity on the same dynamics;
- then measure resource differences;
- never interpret a faster scheduler as a better architecture.

### TDI-22.6 — ASSR bridge

Only after TDI-22 internal controls:
- compare topology-free recurrent state;
- ASSR;
- connectomic sparse recurrence;
- ASSR + connectomic sparse recurrence.

This is a new cross-series development experiment and must not modify TDI-8.2.

### TDI-22.7 — SML-GENIUS bridge

Evaluate frozen SML/CSP candidates using TDI-owned tasks and counters. TDI may reject promotion even if the SML repository reports internal success.

### TDI-22.8 — FLAT hybrid bridge

Compare:
- dense FLAT attention;
- conventional sparse FLAT;
- V888-derived sparse pre-routing + FLAT;
- recurrent graph + restricted FLAT;
- non-attention SML candidate.

Budget matching must state which dimensions are equal and which are not.

### TDI-22.9 — robustness/OOD

Test changed:
- sequence horizon;
- activity density;
- topology size;
- input noise;
- memory budget;
- perturbation strength.

Do not call this biological generalization.

### TDI-22.10 — confirmatory gate

A future confirmatory stage requires a separate human-authorized freeze artifact. This bootstrap does not authorize execution.

## Mandatory outputs

Every run records:
- topology fingerprint;
- source/control arm;
- nodes/edges;
- active nodes/edges per step/cycle;
- memory bits;
- recurrent/event steps;
- task result;
- intervention identity;
- seed;
- exact commit and dependency SHAs.

## Promotion rules

A positive V888 result is insufficient for product promotion unless it survives the strongest matched controls. Reusable primitives go to SciRust; topology search goes to Forge; runtime mechanisms go to NNIS; resource policies go to ElasticXxx; SML model decisions stay in SML-GENIUS.
