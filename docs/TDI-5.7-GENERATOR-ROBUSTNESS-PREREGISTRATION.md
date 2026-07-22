# TDI-5.7 — Generator Robustness: Does the Overlap Signal Survive a Family of Exact Generators?

## Preregistration — DRAFT for design review

> **Status: DRAFT.** Not frozen. This design is presented for review before
> any freeze or build. Once accepted, the constants marked *(finalize at
> freeze)* are fixed, the v57 evaluator is derived, manifests/CI/tests are
> committed, and only then is the design frozen. Freezing does not authorize a
> run; the real experiment is a deliberate one-time human action under the
> exact confirmation token (Section 16). The authoring agent never runs
> `--full`.

## 1. Experimental status and provenance

TDI-5.7 is a new confirmatory experiment derived from the completed and merged
TDI-5.6 result. TDI-5.1 → TDI-5.6 remain frozen and untouched; the frozen
ancestor test and CI pin the full chain (12 hashes, TDI-5.1 → TDI-5.6).

Every TDI-5.x result to date uses a **single generator**: the successor set of
each state is drawn by `splitmix64` uniformly over all non-empty successor
subsets (widths 3 and 4). Robustness to *generator changes* is therefore the
single most-repeated open limitation (every prior Section 20). A skeptic can
still argue the overlap signal — even after surviving the exact contraction
descriptors (5.5) and the exact spectral moments (5.6) — is an artifact of
that one candidate distribution.

TDI-5.7 confronts this directly. It asks whether the **primary TDI-5.6 finding**
— the marginal value of `{O_1, O_2}` beyond the full exact descriptor set
(baseline + Dobrushin δ, δ̄ + spectral moments s₂, s₃; i.e. the SKT-vs-SK
comparison) — **replicates across a family of structurally distinct exact
generators**, not just the inherited base generator. If it does, the signal is
a property of branching dynamics broadly, not of one candidate distribution.
If it does not, the earlier results are scoped to the base generator and TDI's
generality is bounded.

## 2. Research questions

Within the frozen candidate machinery and the frozen exact descriptor set:

1. does the SKT-minus-SK marginal value of `{O_1, O_2}` classify **Beneficial
   across every generator family** at the focal horizons U₃ and U₆
   (**replication**, criterion TDI-5.7A)?
2. how **heterogeneous** is the effect size across families — is the aggregate
   relative-MSE reduction tightly clustered above the 2% margin, or does it
   swing widely / cross into Equivalence for some family (**heterogeneity**,
   criterion TDI-5.7B)?
3. does a model fitted on one family **transfer** to another family at the same
   width — does the overlap signal's calibration generalize across generators,
   or is it generator-specific (**cross-generator transfer**, criterion
   TDI-5.7C, descriptive)?

TDI-5.7 does **not** re-open the exact-descriptor questions (5.5/5.6), the
persistence confound (5.5B), nonlinear families, or the literal spectral gap —
those are settled or deferred (TDI-6, roadmap). It changes **only** the
generator, holding every feature, layout, model and criterion machine of 5.6
fixed.

## 3. Relationship to the frozen ancestors

**Inherited unchanged** from TDI-5.2…5.6 (frozen; not re-derived): the entire
exact candidate analysis, observation geometry, target geometry
`U_h = -log2(1 - O_h)`, the 13 structural/entropic baseline variables, the two
early overlaps O₁, O₂, the two exact contraction descriptors δ, δ̄, the two
exact spectral moments s₂, s₃, the layouts **CK / SK / SKT**, ridge
`lambda = 1.0`, the paired + stratified-aggregate bootstrap, the four-way ±2%
classifier, and the `tdi-core` exact primitives (unchanged).

**New in TDI-5.7** (the only substantive addition): a **family of exact
generators** — several distinct, deterministic successor-mask construction
rules (Section 5) — and the three replication/heterogeneity/transfer criteria
that compare the *same* SKT-vs-SK signal *across* those generators.

## 4. Design notes and confirmatory integrity

### 4.1 Why change only the generator

Holding the entire 5.6 measurement apparatus fixed and varying only the
candidate distribution isolates exactly one factor — the generator — so any
change in the SKT-vs-SK classification is attributable to the generator and
nothing else. This is the clean confound design applied to the generality
question.

### 4.2 What "a family of generators" means, exactly

The inherited generator is one **mask-generation rule**: for each of the
`2^width` states, `mask = splitmix64(seed-chain) % (2^{2^width} − 1) + 1`, a
uniform draw over all non-empty successor subsets. A generator *family member*
is a **different deterministic rule** for producing each state's non-empty
successor set, using only the frozen `tdi-core` primitives (`State`,
`TableSystem`, `Action::Noop`) to assemble the system. Only the rule that
chooses successor sets differs; everything downstream (exact analysis,
features, models, bootstrap, classifier) is identical.

### 4.3 Single width family, exact throughout

To isolate the generator from the width confound, TDI-5.7's primary criteria
hold width fixed (the base width-3 + width-4 in-distribution composition of
5.6) and vary the generator rule. Cross-width generality remains a separate
question (roadmap TDI-5.8). Every generator family member is exact and
deterministic; no floating-point candidate construction is introduced.

## 5. The generator family *(finalize the exact set + parameters at freeze)*

Each family member is a deterministic successor-mask rule seeded by the
candidate seed, producing a non-empty successor set for every state. The
proposed family (K = 4):

| Family | Rule | Intended structural contrast |
|---|---|---|
| **F0 — base** | `mask` uniform over all non-empty subsets (inherited rule, unchanged). | The 5.6 generator; the reference member. |
| **F1 — sparse** | draw a successor-set size `d` from a distribution favoring **small** `d` (e.g. `d` geometric-truncated on `[1, d_max^-]`), then choose `d` distinct successors by a seeded permutation. | Low branching → weaker mixing, larger spectral moments; stresses whether the signal needs high branching. |
| **F2 — dense** | draw `d` favoring **large** `d` (near `2^width`). | High branching → fast mixing, small spectral gap; stresses the low-contraction regime. |
| **F3 — local** | successors restricted to states within **Hamming distance ≤ 1** of the source (plus the source), a seeded non-empty subset thereof. | Structured locality (not IID masks) → a qualitatively different topology and kernel spectrum. |

All four are exact, deterministic, width-parametric, and assemble a valid
total Noop kernel (every state has ≥ 1 successor). The precise size
distributions and the F3 neighborhood definition are **finalized at freeze**;
they are chosen a priori (no tuning on any observed classification).

**Design question for review:** K = 4 as above, or a different/expanded family
(e.g. add a "reversible-biased" member, or a member with self-loops
suppressed)? The heterogeneity criterion is more informative with more
structurally-distinct members, at linear cost in run time.

## 6. Feature layouts

Unchanged from TDI-5.6 Section 6: **CK** (baseline + δ + δ̄, 15), **SK** (CK +
s₂ + s₃, 17), **SKT** (SK + O₁ + O₂, 19). The confirmatory comparison is
**SKT vs SK** (the overlaps' marginal value beyond the full exact descriptor
set), computed independently *within each generator family*.

## 7. Populations *(finalize counts + seed layout at freeze)*

For **each** generator family, generate the same in-distribution populations as
5.6 (width-3 + width-4 training and holdout), across a small number of fresh,
pairwise-disjoint seed blocks. To bound total run time across K families, the
proposed layout is **2 seed blocks per family** (vs 3 in 5.6), giving
K × 2 × 40,000 records (K = 4 → 320,000). Per-family models are fitted on that
family's training populations and every criterion evaluated on that family's
holdout populations. **No OOD populations.**

- Fresh seed blocks, disjoint from the TDI-5.6 blocks J/K/L and all earlier
  blocks; disjoint *across* families. Proposed base offsets ≥ 1.4e9, spaced so
  every family's blocks are pairwise disjoint (finalize at freeze; the
  evaluator verifies disjointness at runtime).
- Fresh bootstrap seeds, `0x5444493537…` ("TDI57"), disjoint from every prior
  bootstrap seed.

**Design question for review:** 2 blocks/family (leaner, ~320k records for
K=4) or keep 3 blocks/family (heavier, ~480k, closer to per-experiment
statistical power)?

## 8. Criteria

### TDI-5.7A — replication across generators (primary)

For **each** family Fᵢ, compute the SKT-vs-SK four-way classification at the
focal horizons U₃ and U₆ (identical machinery to 5.6A). TDI-5.7A is the
conjunction: **Beneficial at U₃ and U₆ for every family** is the preregistered
"robust" outcome. Any family classifying Equivalent/Harmful at a focal horizon
is reported as a **located non-replication** (which family, which horizon) —
an informative negative, not a hidden failure.

### TDI-5.7B — effect-size heterogeneity

Report, across families, the aggregate relative-MSE reduction of SKT over SK at
each focal horizon: its **minimum, maximum, and range**, and whether **all K**
exceed the 2% margin. A tight cluster well above 2% is strong generality; a
wide swing (even if all Beneficial) is a preregistered caveat on effect-size
transportability.

### TDI-5.7C — cross-generator transfer (descriptive)

For an ordered pair of families (A → B) fixed a priori (e.g. F0 → F1), fit the
SK and SKT models on A's training population and evaluate SKT-vs-SK on B's
holdout. Report the standardized-U R² and the four-way classification.
Descriptive only: it distinguishes "the signal exists in each generator" (5.7A)
from "the *same fitted* signal transports across generators."

## 9. Everything else

Metrics (Section 11), standardized-U primacy (12), determinism (18), required
raw output (17 — now per family), reproduction requirements (19), and
interpretation boundaries (20) are inherited from TDI-5.6 with per-family
reporting. Guard token: `TDI57_CONFIRM_FULL_RUN=I_ACCEPT_THE_TDI57_FREEZE_RULE`.

## 10. Interpretation boundaries

A TDI-5.7 result establishes the (non)replication and effect-size stability of
the `{O_1,O_2}`-beyond-exact-descriptors signal **across the specific
preregistered generator family of Section 5**, at fixed width, within the
frozen exact machinery. It does **not** establish: robustness to generators
outside that family; cross-width invariance (TDI-5.8); control against
non-exact descriptors or model families (TDI-6); causal effects; or external
validity. The TDI-5.7A/B/C summaries may not be rewritten after observing the
result.

## 11. Open design questions for the reviewer

1. **Family set (Section 5):** accept F0–F3, or adjust/expand?
2. **Blocks per family (Section 7):** 2 (leaner) or 3 (heavier, more power)?
3. **Transfer pair (Section 8, 5.7C):** F0 → F1, or a different / multiple
   ordered pairs?
4. Anything you want added as a fourth criterion (e.g. a per-family diagnostic
   of how much δ, δ̄, s₂, s₃ themselves move across families, contextualizing
   how demanding the SK baseline is in each)?
