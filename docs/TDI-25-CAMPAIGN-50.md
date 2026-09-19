# TDI-25 — Campaign A: 50 substantive PR slices

Tracker: #392. Primary comparison: **T6 torsor vs C6 chiral**. `G6` is an attribution control only.

This campaign is dependency ordered. A PR number is earned by a concrete research artifact; empty/churn PRs are forbidden. If an earlier gate fails, later slices pause or are revised rather than being generated mechanically.

## Phase A — contract binding and matched comparison semantics (PR 01–10)

| Slice | Deliverable | Gate / definition of done |
| ---: | --- | --- |
| 01 | TDI-25 programme + comparison scaffold | **landed** in #408; reuses TDI-22 and TDI-24 contracts; no copied algebra |
| 02 | Contract/version provenance pin | **landed** in #489; exact TDI-22/TDI-24 semantic contract IDs recorded, validated and rejected on mismatch |
| 03 | G6 generic six-component control | **landed** in #552; versioned finite 6D Euclidean control with matched width and fail-closed arithmetic |
| 04 | Carrier/accounting equivalence | **landed** in #553; explicit score-carrier accounting plus stored T6 reference and query-position geometry |
| 05 | Score-scale contract | **landed** in #554; one finite positive divisor applied identically to T6/C6/G6 |
| 06 | Torsor invariant bridge tests | **landed** in #555; observe upstream invariants before/after transport and verify TDI-25 score invariance |
| 07 | Chiral invariant bridge tests | **landed** in #557; fail-closed M²/J²/MJM and s/m/chi reflection identities through the TDI-25 adapter |
| 08 | Shared masking/normalization reference | **current slice**; identical mask/invalid-row rules across arms |
| 09 | Typed comparison record | family, arm, seed, budget, contract and failure provenance |
| 10 | Stage-A audit/freeze | adversarial review; no unresolved material semantic defect |

## Phase B — balanced task families and split discipline (PR 11–20)

| Slice | Deliverable | Gate / definition of done |
| ---: | --- | --- |
| 11 | Torsor-favorable transport tasks | reduction-point/translation semantics with deterministic oracle |
| 12 | Chiral-favorable reflection tasks | mirrored handedness pairs with deterministic oracle |
| 13 | Mixed geometry tasks | transported relation + parity-sensitive relation both required |
| 14 | Neutral six-component controls | neither torsor nor chirality privileged by target construction |
| 15 | Position-geometry arm registry | linear/helical/learned/external geometry kept explicit |
| 16 | Difficulty strata | bounded deterministic levels independent of model output |
| 17 | Development/Validation split manifest | split identity embedded in every case |
| 18 | Protected-label inference API | inference path cannot read target/oracle label |
| 19 | Seed/case canonicalization + hash | disjoint reproducible populations |
| 20 | Stage-B leakage/balance audit | task-family and split audit green before evaluation |

## Phase C — matched evaluators and statistical protocol (PR 21–30)

| Slice | Deliverable | Gate / definition of done |
| ---: | --- | --- |
| 21 | T6 evaluator | consumes TDI-22 factorized torsor contract |
| 22 | C6 evaluator | consumes TDI-24 chiral contract unchanged |
| 23 | G6 evaluator | same readout/evaluation envelope |
| 24 | Parameter/readout matcher | reject capacity mismatch rather than silently compensate |
| 25 | Optimizer/update-budget matcher | same examples/order/steps/stopping rule where trained |
| 26 | Metric registry | primary family metrics and cross-family summary frozen |
| 27 | Paired uncertainty engine | paired effect/interval computation by seed block |
| 28 | Family-stratified synthesis | pooled summaries cannot hide sign reversals |
| 29 | Typed failure/resource accounting | numerical/task/resource failures retained by arm |
| 30 | Stage-C bounded preflight | Development/Validation only; no protected/final execution |

## Phase D — structure attribution and geometry ablations (PR 31–40)

| Slice | Deliverable | Gate / definition of done |
| ---: | --- | --- |
| 31 | Torsor reduction-point ablation | remove/alter transport structure under matched capacity |
| 32 | Direct vs factorized torsor bridge | numerical equivalence monitored inside comparison harness |
| 33 | Chiral `gamma=0` ablation | remove parity-odd channel |
| 34 | Chiral parity-shuffle control | six values preserved; H+/H- semantics destroyed |
| 35 | Torsor structure-shuffle control | six values preserved; Varignon pairing semantics destroyed |
| 36 | G6 orthogonal-basis control | generic capacity under deterministic basis rotations |
| 37 | Position-geometry ablation | compare declared geometry families without implicit privileged choice |
| 38 | Sequence-length scaling | matched masks, lengths and resource accounting |
| 39 | Data-volume scaling | preregistered sample counts; no adaptive arm-specific allocation |
| 40 | Stage-D attribution audit | determine which structure-specific claims remain admissible |

## Phase E — stress, replication and evidence gate (PR 41–50)

| Slice | Deliverable | Gate / definition of done |
| ---: | --- | --- |
| 41 | Multi-seed replication | frozen paired seed blocks |
| 42 | Input/noise robustness | identical perturbation distributions by arm |
| 43 | Translation/origin stress suite | hard torsor-relevant transformations independent of model output |
| 44 | Mirror/parity stress suite | hard chiral-relevant transformations independent of model output |
| 45 | Mixed adversarial suite | both transformation classes combined under deterministic oracle |
| 46 | Numerical precision study | f64/f32 tolerance/failure comparison |
| 47 | Reference cost study | operations, memory and qualified CPU timing; no production/GPU claim |
| 48 | Integrated non-final campaign | frozen Development/Validation execution of T6/C6/G6 |
| 49 | Statistical synthesis + confirmatory pre-arm | freeze primary T6-vs-C6 rule and guarded manifest; no protected run |
| 50 | Evidence decision artifact | retain positive/equivalent/harmful/inconclusive result and downstream boundary |

## Stop rules

Pause the campaign when any of the following occurs:

- source TDI-22 or TDI-24 contract identity is ambiguous;
- a bridge test fails an exact algebraic requirement;
- T6/C6 capacity or budget matching is not explicit;
- task-family construction leaks labels or privileges one arm outside the declared family purpose;
- split provenance/disjointness is not demonstrable;
- pooled analysis would conceal a family-specific reversal;
- a predecessor has an unresolved material review finding;
- execution would cross a protected/final gate without its dedicated authorization artifact.

## Progress accounting

**7/50 merged** (#408, #489, #552, #553, #554, #555, #557). Slice 08 is current. Only merged, exact-head-qualified substantive PRs increment the campaign counter.
