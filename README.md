# TDI — Dynamic Information Theory

**TDI is a deterministic Rust research programme for testing which structural, dynamical and representational properties retain predictive information under controlled interventions.**

The repository is organized as a sequence of falsifiable research benches. Each bench owns its question, controls, numerical policy, evidence boundary and reproducibility contract. Positive, negative, equivalent and inconclusive outcomes are retained.

> Start here for execution: [Research engine guide](docs/engineering/research-engine.md).  
> Scientific status is defined by the canonical programme and preregistration files under [`docs/`](docs/), not by README prose.

## Method

TDI follows a fail-closed research workflow:

```text
question
  -> preregistration
  -> deterministic reference semantics
  -> bounded Development / Validation
  -> frozen evidence boundary
  -> gated confirmation when authorised
  -> retained result + provenance
```

The core rules are simple:

- controls are defined before confirmatory evidence;
- final/protected populations are isolated from ordinary development;
- exact and floating-point claims are clearly separated;
- resource accounting is explicit when architectures are compared;
- numerical evidence is never promoted to proof;
- failed hypotheses and counterexamples remain part of the programme.

## Research benches

| Research family | Series | Core question | Status |
| --- | --- | --- | --- |
| **Finite-state dynamics & recovery geometry** | TDI-1 → TDI-6.8 | Which recovery descriptors predict later dynamical deficit, and which effects survive stronger structural, spectral and transfer controls? | Historical core established; TDI-6.8 is the transportable-ordering extension |
| **Attention recovery** | [TDI-7.x](docs/TDI-7.0-ATTENTION-RECOVERY-PREREGISTRATION.md) | Do early intervention-conditioned recovery descriptors add predictive value beyond static attention diagnostics? | Active |
| **Recurrent associative architectures** | [TDI-8.x](docs/TDI-8-PROGRAMME.md) | Under matched memory budgets, what is gained by recurrence, associative memory and a bounded VSA workspace? | Active bounded evaluator programme |
| **Adaptive inference dynamics** | [TDI-9.x](docs/TDI-9-PROGRAMME.md) | When should an inference process continue, stop, verify or recover under an explicit compute envelope? | Active |
| **Operators, resolvents & ordinal transport** | [TDI-10.x](docs/TDI-10-PROGRAMME.md), [TDI-12.x](docs/TDI-12-PROGRAMME.md) | Which operator identities hold exactly, and does ordinal structure transport more robustly than absolute calibration? | Active |
| **Hallucination dynamics & control** | [TDI-11.x](docs/TDI-11-PROGRAMME.md) | Can prospective trajectory signals support bounded verification, recovery or abstention before unsupported generation is emitted? | Active pre-arm / Development |
| **Boolean & formal representation systems** | [TDI-21.x](docs/TDI-21-PROGRAMME.md), [TDI-22.x](docs/TDI-22-PROGRAMME.md), [TDI-23.x](docs/TDI-23-PROGRAMME.md) | Can Boolean routing, torsor geometry and typed categorical rewrites reproduce useful structure while preserving explicit semantics and budgets? | Active Stage-0 / Development |
| **Matched representation campaigns** | [TDI-24.x](docs/TDI-24-PROGRAMME.md), [TDI-25.x](docs/TDI-25-PROGRAMME.md) | Under matched capacity and evaluation budgets, how do vector, chiral and torsor representations differ? | Active |
| **Sparse recurrent topology** | [TDI-26.x](docs/V888_CONNECTOME_BOOTSTRAP.md) | Does V888-derived sparse structure add predictive or computational value beyond progressively stronger matched graph controls? | Stage-0 |
| **Latent concept geometry** | [TDI-27.x](docs/TDI-27-PROGRAMME.md) | After removing known subspaces, is the remaining latent innovation stable, causal and sequentially independent? | Active Development |

## Mathematical atlas

These equations are orientation landmarks, not substitutes for the frozen protocols.

### Dynamics and recovery — TDI-1 → TDI-6.8

TDI-5.1 uses overlap-derived deficit geometry:

$$
U_h=-\log_2(1-O_h),
\qquad
x_{\mathrm{TDI}}=(O_1,\,O_2,\,O_2-O_1).
$$

The question is not whether a descriptor correlates with failure, but whether it adds predictive information beyond competent controls and survives held-out evaluation.

### Attention recovery — TDI-7

The reference attention semantic may be written

$$
A(Q,K,V)=\operatorname{softmax}\!\left(\frac{QK^\top}{\sqrt d}\right)V,
$$

while the primary incremental-value statistic is

$$
r_{\mathrm{MSE}}
=
\frac{\mathrm{MSE}_{B0}-\mathrm{MSE}_{B1}}
     {\mathrm{MSE}_{B0}}.
$$

Here (B0) is the frozen static/task baseline and (B1) adds early TDI recovery descriptors.

### Recurrent associative architectures — TDI-8

The primary architecture contrasts are evaluated under a matched dynamic-memory budget:

$$
M_{A1}=M_{A2}=M_{A3},
$$

with cell-level relative mean-deficit reduction

$$
R=\frac{B-C}{B}.
$$

The budget equality is part of the scientific contract; metadata and working storage are accounted explicitly.

### Adaptive inference and Boolean policy synthesis — TDI-9

The generic control object is

$$
p_t=P(\mathrm{observation}_t),
\qquad
a_t=F_{\mathrm{bool}}(p_t),
$$

where the policy can select only actions allowed by the declared C0/C1/C2/C3 resource envelope.

### Hallucination dynamics and control — TDI-11

TDI-11 separates detection, localization, diagnosis, intervention and decision. Its bounded action vocabulary includes `CONTINUE`, `VERIFY`, `BACKTRACK` / `RECOVER`, `EMIT` and `ABSTAIN`; controller evidence must remain prospective and must not receive hidden evaluator truth.

### Operator and ordinal research — TDI-10 / TDI-12

For (0<\alpha_k\le1), the exact product/remainder line includes

$$
\sum_k(1-\alpha_k)=\infty
\quad\Longleftrightarrow\quad
\prod_k \alpha_k \to 0,
$$

while ordinal transport is represented by rank-based quantities such as

$$
\rho_S
=
\operatorname{corr}
\bigl(\operatorname{rank}(x),\operatorname{rank}(y)\bigr).
$$

TDI-10 distinguishes exact identities, proved statements, formal asymptotics, numerical evidence, conjectures and refutations.

### Boolean, torsor, categorical and chiral representations — TDI-21 → TDI-25

A Boolean ANF / Zhegalkin routing function is represented over (mathbb F_2) as

$$
f(x)
=
\bigoplus_{S} a_S
\prod_{i\in S}x_i.
$$

The TDI-22 torsor pairing is

$$
s_T
=
(v+Q\times\omega)\cdot R
+
\omega\cdot C.
$$

The TDI-23 coordinate reduction is

$$
R(f)=E_K^\dagger f E_H,
$$

and the TDI-24 chiral odd channel is

$$
\chi(q,k)=q^\top Jk,
\qquad
\chi(Mq,Mk)=-\chi(q,k).
$$

These are representation identities and experimental objects, not superiority claims.

### Sparse recurrent topology — TDI-26

The programme compares the V888-derived graph against progressively stronger matched controls:

$$
G_{\mathrm{V888}}
\quad\text{vs}\quad
G_{\mathrm{random}},
G_{\mathrm{degree}},
G_{\mathrm{reciprocity}},
G_{\mathrm{modularity}}.
$$

Only effects that survive the strongest declared matched controls are candidates for promotion.

### Latent concept geometry — TDI-27

For positive and control populations (P) and (C),

$$
v=\mu_P-\mu_C,
\qquad
r=(I-P_U)v,
$$

with innovation energy

$$
I=\frac{\lVert r\rVert^2}{\lVert v\rVert^2},
$$

and nonlinear interaction residual

$$
J(a,b)=E(a+b)-E(a)-E(b).
$$

TDI-27 tests whether residual geometry is stable and causally specific; a large residual by itself is not a mechanistic conclusion.

## Current frontier

The current `main` line is advancing **TDI-27.1 resampling and null calibration**. Independent positive/control bootstrap streams are now part of the Development substrate, while confirmatory and final TDI-27 execution remain disabled.

Parallel active work continues in the matched representation programmes (TDI-24/TDI-25), sparse recurrent topology (TDI-26), and the existing attention, memory, inference, operator and formal-representation lines.

For exact stage status, use the programme documents linked above and the repository issues/PRs rather than this summary.

## Repository map

| Path | Purpose |
| --- | --- |
| [`tdi-core/`](tdi-core/) | exact finite-state and structural primitives |
| [`tdi-ai/`](tdi-ai/) | attention, memory, adaptive-inference and experimental architecture primitives |
| [`tdi-operator/`](tdi-operator/) | generic operator, resolvent, Green-function and ordinal primitives |
| [`tdi-bench/`](tdi-bench/) | deterministic evaluators, research benches and executable experiments |
| [`docs/`](docs/) | programmes, preregistrations, audits, status records and scientific reports |
| [`results/`](results/) | retained deterministic outputs and evidence artefacts |
| [`scripts/`](scripts/) | integrity, reproduction and bounded research workflows |

## Reproduce the development surface

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Individual benches may impose additional frozen manifests, scripts, confirmation guards or provenance requirements. Those controls are part of the scientific protocol and must not be bypassed.

For the TDI-27 Development bench:

```bash
bash scripts/check-tdi27-bootstrap.sh
cargo run --locked -p tdi-bench --bin tdi27_concept_geometry
```

## Evidence boundary

TDI does **not** claim a universal law of intelligence, universal architecture superiority, guaranteed hardware speedups, proprietary-model reverse engineering, or a universal solution to hallucination.

A result supports only the bounded claim tested by its declared protocol. Promotion into SciRust, Forge, NNIS, ElasticXxx, FLAT-ATTENTION or another Memorithm project requires a separate scientific or engineering contract.

## License

TDI is **dual-licensed**; see [`LICENSING.md`](LICENSING.md).

- Noncommercial and personal use: [PolyForm Noncommercial License 1.0.0](LICENSE.md).
- Commercial use: separate commercial licence from the copyright holder.

Copyright 2026 Tarek Zekriti.
