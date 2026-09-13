<p align="center">
  <img src="docs/assets/tdi-research-roadmap.svg" alt="TDI research programme" width="100%">
</p>

# TDI — Dynamic Information Theory

**A deterministic Rust research programme studying which structural and dynamical descriptions of a system retain predictive information beyond simple scalar summaries.**

TDI develops falsifiable experiments, preregistered decision rules, deterministic evaluators and reproducible evidence. The programme started with exact finite-state dynamics and has expanded into attention / memory, bounded recurrent-associative architectures, adaptive inference dynamics, generic operator / resolvent research, hallucination dynamics / control, and ordinal universality of operator responses.

> The illustration above is a visual introduction. The status table below is the authoritative research map for the repository.

## Why TDI matters

TDI is not a collection of benchmark claims. It is a sequence of controlled research programmes designed to answer narrower questions with stronger evidence.

The project has already established a useful pattern: simple summaries can miss predictive structure, stronger controls can absorb apparent effects, transfer can preserve ordering while losing calibration, and a failed preregistered criterion can still reveal a concrete mechanism worth testing next. Positive, negative, equivalent and inconclusive outcomes are all retained.

The result is a growing body of **code + preregistration + evaluator + provenance + result artefacts**, rather than a narrative reconstructed after the experiment.

## Research map at a glance

| Series | Research line | Repository status | What it contributes |
| --- | --- | --- | --- |
| **TDI-1** | **Deterministic systems** | ✅ Completed | Shows that equal Shannon block entropy need not imply equal perturbation recovery, while also showing that the tested TDI-1 signal is subsumed by a stronger orbital baseline. |
| **TDI-2** | **Branching systems** | ✅ Completed | Preregistered continuous branching evaluation; predictive signal is measurable, but universal cross-width calibration is not established. |
| **TDI-3** | **Inter-width evaluation** | ✅ Completed | The preregistered inter-width criteria fail; the experiment isolates a width-sensitive signal rather than a universal representation. |
| **TDI-4** | **Target geometry** | ✅ Completed | The preregistered two-head protocol fails formally, while the continuous deficit geometry remains strongly informative in the tested regime. |
| **TDI-5.x** | **Exact confirmatory battery** | ✅ Completed line | Exact-rational, SHA-frozen, confirmation-gated experiments covering overlap ablation, nonlinear sufficiency, spectral controls, generator robustness and cross-width behaviour. |
| **TDI-6.1–6.7** | **Non-exact frontier** | ✅ Completed line | Extends the controls to literal spectral gap, nonlinear models, information decomposition, causal probes and cross-generator transfer diagnostics under declared floating-point discipline. |
| **TDI-6.8** | **Transportable Ordering Across Generator Families** | 🟠 Preregistered / research line | Tests whether rank ordering — rather than absolute calibration — transports across fresh generator-family populations under a frozen per-block criterion. |
| **TDI-7.x** | **Attention / Memory research programme** | 🟠 Active | Moves the intervention-conditioned recovery question into deterministic attention / memory tasks, then studies heterogeneity, long-horizon joint information and successor semantic questions. |
| **TDI-8.x** | **Recurrent Associative Architecture Research Programme** | 🟠 Active | Matched-budget A0/A1/A2/A3 programme comparing full-history reference, bounded recurrent state, associative memory, and a bounded VSA / holographic workspace. |
| **TDI-9.x** | **Autonomous Adaptive Inference Dynamics Research Programme** | 🟠 Active | C0/C1/C2/C3 policy ladder plus **TDI-9.3 Boolean Policy Synthesis**, which represents and later searches explicit Boolean action policies over frozen leakage-safe trajectory predicates. |
| **TDI-10.x** | **Operator / Resolvent Research** | 🟠 Active | Generic Jacobi / tridiagonal operator work on shifted resolvents, Schur cavities, Green functions, finite transport and explicitly classified asymptotic claims. |
| **TDI-11.x** | **Hallucination Dynamics & Control Research Programme** | 🟠 Active bootstrap | Studies supported vs unsupported generation in fully specified worlds, measurable precursors, causal perturbations, risk estimation and bounded adaptive verification / recovery / abstention. |
| **TDI-12.x** | **Ordinal Universality of Operator Responses** | 🟠 Active Stage-0 bootstrap | Tests whether ordinal/rank relations among resolvent and Green response observables transport more stably than absolute calibration across dimension, coefficient family and perturbation regime. |

## Current research frontier

The repository no longer stops at TDI-6.x.

### TDI-7.x — Attention / Memory

The programme begins with [`TDI-7.0 — Attention recovery preregistration`](docs/TDI-7.0-ATTENTION-RECOVERY-PREREGISTRATION.md): deterministic associative-recall and copy tasks test whether early intervention-conditioned recovery descriptors predict later retrieval deficit beyond competent static attention diagnostics.

Later TDI-7 work must preserve the distinction between a **confirmatory result**, a **frozen protocol**, an **identifiability blocker**, and an **unauthorised final holdout**. In particular, the current TDI-7.4 identifiability finding is a blocker for the present H-AI-3 realisation, not a confirmatory negative result.

### TDI-8.x — Recurrent Associative Architecture Research Programme

[`TDI-8.x`](docs/TDI-8-PROGRAMME.md) is the bounded alternative recurrent / associative architecture line.

- **A0** — competent attention-like full-history reference;
- **A1** — bounded recurrent-state-only reference;
- **A2** — A1 + explicit bounded associative memory (working label: ASSR);
- **A3** — A2 + bounded VSA / holographic workspace paid from the same dynamic-memory budget (working label: ASSR-H).

TDI-8.0 is frozen. TDI-8.1 builds the deterministic evaluator. TDI-8.2 remains a future human-only confirmatory holdout and is not authorised by ordinary development or CI.

### TDI-9.x — Autonomous Adaptive Inference Dynamics Research Programme

[`TDI-9.x`](docs/TDI-9-PROGRAMME.md) studies how much computation to spend and when to continue, stop, verify or recover.

Its policy ladder deliberately separates **fixed compute**, **static preallocation**, **adaptive stopping**, and **adaptive verification / recovery**. The final confirmation design uses a frozen derivation from future public entropy rather than a discretionary post-hoc seed choice.

**TDI-9.3 — Boolean Policy Synthesis** is now the explicit Boolean-policy extension of that line. It starts from the observation that the current hand-written C2 stopping policy is already a Boolean composition of trajectory predicates, then generalizes this into a deterministic Boolean-expression IR and a future bounded search programme. The initial implementation remains non-final and available only through the `tdi-ai` `experimental` feature; TDI-9.3.0 adds exhaustive exact truth-table calibration for the hand-written C2 STOP Boolean shape and the documented C3 ordered multi-action state machine, plus exact synthesis-search envelopes, fail-closed IR mutation, C2↔C3 joint ABSENT/`BASE_STOP` projection invariants, and complexity Pareto-dominance lemmas, without pinning TDI-9.1 fields or altering C2/C3 reference semantics or creating a TDI-9.2 final-evaluation surface. See [`docs/TDI-9.3-BOOLEAN-POLICY-SYNTHESIS.md`](docs/TDI-9.3-BOOLEAN-POLICY-SYNTHESIS.md).

### TDI-10.x — Operator / Resolvent Research

[`TDI-10.x`](docs/TDI-10-PROGRAMME.md) is a scientifically autonomous generic operator line for real symmetric tridiagonal / Jacobi operators.

The latest `main` state includes TDI-10.5 (**REFUTED** pointwise-subunit decay), TDI-10.6 (**EXACT** uniform geometric bound), TDI-10.7 (**EXACT** divergent-remainder decay lemma), TDI-10.8 (**EXACT** witness trichotomy for Types S/U/D), TDI-10.9 (**EXACT** harmonic remainder-rate lemma + closed-form witness calculus), TDI-10.10 (**EXACT** remainder / product / log-sum equivalence under `0 < alpha_k <= 1`), and TDI-10.11 (**EXACT** quantitative product / exponential bounds companion to 10.10), and TDI-10.12 (**EXACT** Cesàro / mean-remainder exponential rate; **REFUTED** necessity of positive Cesàro limit for product → 0), and TDI-10.13 (**EXACT** named operator families `F_S`/`F_U`/`F_D` realizing the trichotomy through cavity transport; **REFUTED** that every cavity family forces decay; subunit-product arc 10.5–10.12 closed, operator-family chapter open), and TDI-10.14 (**EXACT** contraction ↔ Type-U ρ bridge with dual-path `ρ^n` identity; **REFUTED** that every frozen Toeplitz symbol yields Type D), and TDI-10.15 (**EXACT** `F_U` then `F_D`/`F_S` family composition with closed concatenation products; **REFUTED** that a Type-S suffix erases Type-U prefix decay uniformly in `m`), and TDI-10.16 (**EXACT** κ(a,b) domain/monotonicity + matched TDI-10.3 factorization → Type-U ρ; **REFUTED** that κ → 0 as `a ↓ 2|b|+`; **REFUTED** that larger diagonal alone forces smaller κ without fixing `|b|`), and TDI-10.17 (**EXACT** family↔10.4 affine-unrolling link with constant-drift `F_U`/`F_D` closed `B_n`; **REFUTED** that Type-U product decay alone forces cavity-error → 0 under nonzero constant drift), and TDI-10.18 (**EXACT** operator-family finite hypothesis checklist: Item D via TDI-10.9 `1-alpha_k ≥ L/k` eventually + Item T = TDI-10.2 on family steps; **REFUTED** that informal `alpha_k → 1` meets Item D; **REFUTED** that a finite-window harmonic bound alone forces product → 0). TDI-10 explicitly labels its evidence as **EXACT**, **PROVED UNDER DECLARED ASSUMPTIONS**, **FORMAL ASYMPTOTIC**, **NUMERICAL EVIDENCE**, **CONJECTURE**, or **REFUTED**. Numerical evidence is never silently promoted to proof.

### TDI-11.x — Hallucination Dynamics & Control Research Programme

[`TDI-11.x`](docs/TDI-11-PROGRAMME.md) studies hallucination as an inference phenomenon rather than only as a benchmark score. The programme begins from fully specified deterministic worlds so the evaluator can separate facts explicitly available to a model, facts derivable from those inputs, hidden-but-true facts, contradictions, nonexistent entities and genuinely unsupported assertions.

TDI-11 deliberately separates **detection**, **localization**, **diagnosis**, **intervention** and **decision**. Its working controller label is **HAC — Hallucination Adaptive Controller**, with candidate bounded actions such as `CONTINUE`, `VERIFY`, `BACKTRACK`, `RECOVER`, `EMIT` and `ABSTAIN`. HAC is a working label, not a novelty or universality claim.

TDI-11.0 is an active bootstrap/scope stage and is **not frozen**. TDI-11.1 evaluator implementation is not authorised until an explicit preregistration and implementation gate freeze the taxonomy, controlled-world oracle, allowed observables, interventions, resource accounting, split discipline, metrics and provenance rules. See [`TDI-11.0 — Hallucination Dynamics Scope and Stage Gate`](docs/TDI-11.0-HALLUCINATION-DYNAMICS-SCOPE.md).

### TDI-12.x — Ordinal Universality of Operator Responses

[`TDI-12.x`](docs/TDI-12-PROGRAMME.md) asks whether rank/order information among finite Jacobi / resolvent / Green response observables is a more transportable object than absolute calibration.

TDI-12.0 is an **active Stage-0 bootstrap** and is **not frozen**. It lands exact average-rank / Spearman / Kendall τ-b primitives, candidate Green-band response extractors with EXACT `GreenBands` wiring tests, Stage-0 controls (identity, dimension-only, monotone affine, full-tie fail-closed, reverse-order, coefficient-norm / Gershgorin-margin, deterministic shuffle), a machine-readable freeze template whose scientific fields remain `unresolved_blocking` with confirmatory/final execution flags **false**, and a fail-closed freeze-template validator that refuses invented pins. See [`docs/TDI-12.0-SCOPE.md`](docs/TDI-12.0-SCOPE.md) and [`docs/TDI-12.0-STATUS.md`](docs/TDI-12.0-STATUS.md).

Stage 0 does not authorize confirmatory populations, does not invent TDI-8.1 / TDI-9.1 / TDI-11.2 freeze pins, and does not contact TDI-7.2 / TDI-8.2 / TDI-9.2 surfaces.


## What the completed programme has taught us

Across the historical finite-state campaign, the strongest supported conclusion is deliberately narrower than a grand theory claim.

Within the tested synthetic branching families, early intervention-conditioned distribution overlap contains predictive information beyond entropy/topology controls, exact contraction descriptors, exact spectral moments, the literal spectral gap, ε-mixing time, and the tested degree-2 interaction model. The effect replicates across multiple generator families and widths.

At the same time, **transportable calibration fails** across widths and generators, effect size is not universal, and two label-free calibration repairs were refuted. This is not hidden as a weakness: it is part of the scientific result and directly motivates the transport/order and successor programmes.

## Evidence discipline

TDI uses a fail-closed research workflow:

1. **Preregister** the question, controls, split discipline, metric and decision rule.
2. **Freeze** the scientific artefacts and their hashes before confirmatory evidence.
3. **Implement** deterministic reference semantics and bounded validation tests.
4. **Keep final material isolated** from ordinary development surfaces.
5. **Execute once under the declared gate** when a final run is authorised.
6. **Publish the result even when it fails**, including provenance, limitations and counterexamples.

TDI-5.x uses exact-rational, bit-reproducible computation. TDI-6.x and later non-exact work must declare numerical policies and evidence boundaries explicitly.

## Explore the evidence

- [`docs/`](docs/) — preregistrations, scientific reports, status documents, audits and limitations.
- [`results/`](results/) — captured deterministic reference outputs and result artefacts.
- [`scripts/`](scripts/) — reproduction commands and integrity checks.
- [`tdi-core/`](tdi-core/) — finite-state dynamics and structural primitives.
- [`tdi-bench/`](tdi-bench/) — evaluators, scans, models and deterministic statistical procedures.
- [`tdi-operator/`](tdi-operator/) — generic operator / resolvent primitives for the TDI-10 line.

## Reproducibility

Standard development validation:

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Confirmatory experiments may impose additional frozen scripts, manifests, exact tokens or future-entropy rules. Those gates are part of the scientific protocol and must not be bypassed by CI or development automation.

## Non-claims

TDI does **not** claim a universal law of intelligence, AGI, a world model, proprietary-model reverse engineering, universal asymptotic superiority, guaranteed hardware speedups, or a universal solution to hallucination.

Each result supports only the bounded claim that its frozen experiment actually tests. Promotion into SciRust, Forge, NNIS, ElasticXxx, FLAT-ATTENTION or another Memorithm project requires a separate engineering or scientific contract.

## License

TDI is **dual-licensed**; see [`LICENSING.md`](LICENSING.md).

- Noncommercial and personal use: [PolyForm Noncommercial License 1.0.0](LICENSE.md).
- Commercial use: separate commercial licence from the copyright holder.

Copyright 2026 Tarek Zekriti.
