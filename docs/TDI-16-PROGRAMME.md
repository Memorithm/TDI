# TDI-16.x — Event-Triggered Attention

Status: **active Stage 0 bootstrap; not frozen; no confirmatory execution authorised**.

## Conjecture

A bounded recurrent/associative controller can invoke expensive attention or retrieval only when an observable event indicates insufficient internal state, and thereby improve the quality/cost frontier relative to always-on attention and no-attention baselines.

Candidate trigger families include retrieval deficit, novelty, contradiction, uncertainty, state-change magnitude and bounded combinations of these signals.

## Primary null

Any apparent gain is explained by extra compute, oracle leakage or task-specific tuning; after matched resource accounting, event-triggered attention does not improve held-out quality/cost.

## Stage map

- **TDI-16.0** — freeze tasks, trigger observables, event timing, action budget, resource accounting and endpoints.
- **TDI-16.1** — deterministic recurrent/associative reference controller with explicit attention action.
- **TDI-16.2** — compare never/always/static-periodic/event-triggered policies.
- **TDI-16.3** — causal perturbation of trigger observables and false-trigger analysis.
- **TDI-16.4** — architecture/runtime transfer only after a qualified non-final result.

## Required controls

Always-attend, never-attend, periodic attend at matched event count, random triggers, oracle trigger upper bound reported separately, equal compute/memory envelopes, shuffled-trigger controls and held-out task families.

## Decision principle

Support requires an event-triggered policy to improve a frozen Pareto criterion over competent matched-budget baselines on held-out data, while its trigger is available prospectively and does not receive hidden outcome labels.

## Ecosystem boundary

TDI-8 may provide recurrent/associative references. FLAT-ATTENTION may later implement the attention action efficiently. ElasticXxx may consume a qualified policy. None of those engineering components establish the scientific claim.

## Non-claims

TDI-16 does not claim linear-time universal inference, universal trigger observables, or superiority over Transformers.