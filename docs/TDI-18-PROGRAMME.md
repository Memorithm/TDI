# TDI-18.x — Elastic Verification Thresholds

Status: **active Stage 0 bootstrap; not frozen; no confirmatory execution authorised**.

## Conjecture

Under a finite compute/memory budget and a prospectively available risk state, a threshold-structured policy over actions such as CONTINUE, VERIFY, RECOVER and ABSTAIN can be optimal or near-optimal for a declared utility/risk objective over a meaningful class of inference environments.

## Primary null

Threshold structure gives no held-out advantage over competent non-threshold policies once policy capacity, tuning budget and action costs are matched.

## Stage map

- **TDI-18.0** — freeze state variable(s), action set, cost model, utility/risk endpoint and policy-capacity accounting.
- **TDI-18.1** — deterministic finite-state environments where the dynamic-programming optimum is computable.
- **TDI-18.2** — compare optimal, threshold, fixed, random and capacity-matched learned policies.
- **TDI-18.3** — perturb monotonicity assumptions and search for counterexamples to threshold optimality.
- **TDI-18.4** — non-final transfer to TDI-9/TDI-11-compatible inference traces if justified.

## Required controls

Fixed always/never verification, random action policies, capacity-matched trees/piecewise policies, exact dynamic-programming oracle in small environments, cost misspecification and calibration perturbations.

## Decision principle

A threshold claim requires an explicitly defined environment class and either a proof under declared assumptions or reproducible empirical near-optimality against the exact/strong policy baseline. Failure outside that class must be reported.

## Ecosystem boundary

TDI-9 supplies the generic adaptive-inference vocabulary; TDI-11 may later supply qualified hallucination-risk signals. ElasticXxx is a downstream runtime target only after independent qualification.

## Non-claims

TDI-18 does not claim universal threshold optimality, universal risk calibration, or production safety.