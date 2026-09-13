# TDI-12.x — Ordinal Universality of Operator Responses

Status: **active Stage 0 bootstrap; not frozen; no confirmatory execution authorised**.

## Conjecture

For declared families of finite Jacobi/tridiagonal operators, absolute response calibration may fail to transport while an ordinal relation between response observables remains substantially more stable across dimension, coefficient family and perturbation regime.

The programme tests whether rank/order information is a more transportable object than absolute calibration for resolvent, Green-function, cavity and perturbation-response observables.

## Primary null

After competent normalization and uncertainty accounting, ordinal transport is no more stable out of family than absolute calibration.

## Stage map

- **TDI-12.0** — Stage-0 bootstrap: declare freeze-template fields (populations, response observables, ranking metric, ties, normalization, split discipline) without pinning scientific values; land exact ordinal ranking primitives and candidate Green-band response extractors over generic TDI-10 primitives; keep confirmatory execution flags false.
- **TDI-12.1** — deterministic exact/synthetic operator evaluator using generic TDI-10 primitives only (authorized only after TDI-12.0 bootstrap integrity passes and any required freeze fields for that evaluator slice are explicitly resolved).
- **TDI-12.2** — development/validation comparison of calibration transport vs ordinal transport.
- **TDI-12.3** — perturbation and counterexample search.
- **TDI-12.4** — separately gated confirmatory population if earlier stages justify it.

## Required controls

Identity ordering, dimension-only ordering, spectral-gap and norm baselines, monotone rescalings, shuffled-family controls, tie-heavy adversarial cases and explicitly generated counterexamples.

## Decision principle

Support requires materially stronger held-out ordinal stability than calibrated-value transport under the frozen metric, without deriving the ordering from hidden target labels. Negative and equivalent outcomes are publishable results.

## Ecosystem boundary

TDI-10 supplies generic operator/resolvent primitives. RiemannBench may provide non-final source populations only after Riemann-specific semantics are stripped. No TDI-12 result implies anything about the Riemann hypothesis.

## Holdout / execution boundary

TDI-12 Stage 0 must not contact TDI-7.2, TDI-8.2, or TDI-9.2 surfaces. Confirmatory and final execution flags in the Stage-0 freeze template remain `false`. Merging Stage-0 code does **not** invent freeze pins for TDI-8.1 / TDI-9.1 / TDI-11.2.

## Non-claims

TDI-12 does not currently claim a universal ordering law, asymptotic theorem, cross-domain universality, or proof of any operator-theoretic conjecture.
