# TDI-19.x — Approximate Submodularity of Scientific Discovery

Status: **active Stage 0 bootstrap; not frozen; no confirmatory execution authorised**.

## Conjecture

For a declared class of scientific-search problems with partially redundant experiments, marginal information gain exhibits sufficiently strong diminishing returns that a greedy `EIG / cost` planner approaches the performance of substantially more expensive non-myopic planners.

## Primary null

Greedy performance is explained by benchmark simplicity, estimator bias or horizon truncation; on held-out problem families it is not reliably competitive with stronger planning baselines.

## Stage map

- **TDI-19.0** — freeze synthetic discovery environments, prior/likelihood families, EIG estimator, costs, horizon and stopping criteria.
- **TDI-19.1** — exact small environments where adaptive submodularity violations and planner regret can be computed.
- **TDI-19.2** — compare greedy, random, grid, one-step lookahead, finite-horizon lookahead and oracle policies.
- **TDI-19.3** — search automatically for counterexamples and characterize regimes where diminishing returns fail.
- **TDI-19.4** — non-final transfer to SciRust/SDE planner traces if earlier stages justify it.

## Required controls

Exact EIG where available, estimator-noise sensitivity, equal experiment budgets, horizon-matched planning, prior misspecification, redundant/non-redundant experiment families and adversarial synergy cases.

## Decision principle

Support requires a declared approximation/regret criterion on held-out environments and explicit measurement of submodularity violations. Similar final accuracy without lower cost or lower regret is not support.

## Ecosystem boundary

SciRust/SDE may supply planner implementations and provenance contracts, but TDI-19 owns the experimental scientific claim. SDE's own planned EIG benchmark is motivation, not evidence.

## Non-claims

TDI-19 does not claim adaptive submodularity universally, optimality of greedy science planning, or that EIG is always the correct scientific objective.