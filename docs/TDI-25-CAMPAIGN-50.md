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
| 08 | Shared masking/normalization reference | **landed** in #561; T6/C6/G6 call the same TDI-24 mask/normalizer path |
| 09 | Typed comparison record | **landed** in #563; family, arm, seed block, case, budget, contracts and retained failure provenance |
| 10 | Stage-A audit/freeze | **landed** in #566; cross-contract audit and non-authorizing freeze manifest |

## Phase B — balanced task families and split discipline (PR 11–20)

| Slice | Deliverable | Gate / definition of done |
| ---: | --- | --- |
| 11 | Torsor-favorable transport tasks | **landed** in #568; two reductions of one physical torsor share one separated deterministic score oracle |
| 12 | Chiral-favorable reflection tasks | **landed** in #603; mirrored handedness pairs with deterministic oracle |
| 13 | Mixed geometry tasks | **landed** in #605; transported relation + parity-sensitive relation both required |
| 14 | Neutral six-component controls | **landed** in #606; neither torsor nor chirality privileged by target construction |
| 15 | Position-geometry arm registry | **landed** in #607; linear/helical/learned/external geometry kept explicit |
| 16 | Difficulty strata | **landed** in #612; bounded deterministic levels independent of model output |
| 17 | Development/Validation split manifest | **landed** in #654; split identity embedded in every case |
| 18 | Protected-label inference API | **landed** in #656; inference path cannot read target/oracle label |
| 19 | Seed/case canonicalization + hash | **landed** in #658; disjoint reproducible populations |
| 20 | Stage-B leakage/balance audit | **landed** in #660; task-family and split audit green before evaluation |

## Phase C — matched evaluators and statistical protocol (PR 21–30)

| Slice | Deliverable | Gate / definition of done |
| ---: | --- | --- |
| 21 | T6 evaluator | **landed** in #661, exact head `6646886c7e749bf158714984ac4de92eec3228cb`, merge `8dfc513382f77308042f01ad2571cd1844c27d64`; consumes TDI-22 factorized torsor contract through a bounded non-final envelope |
| 22 | C6 evaluator | **landed** in #663, exact head `25ff8e020a0c784d3a52d021fb13d9fcc986702f`, merge `ae501e9b3941a293c3cb18f3450c3c077b25f48f`; consumes TDI-24 chiral contract unchanged |
| 23 | G6 evaluator | **landed** in #666, exact head `3403b8768d24ac1543b75120d15a09dd5b58c50e`, merge `d6c8226025de1dbe4848f83d8092c316066f97b1`; same readout/evaluation envelope as T6/C6 |
| 24 | Parameter/readout matcher | **landed** in #669, exact head `69f5ec9c27ceeadd047fd196193131964b7a0023`, merge `32c02cb7625dae788dc5d44d4449447a408f0589`; reject capacity mismatch rather than silently compensate |
| 25 | Optimizer/update-budget matcher | **landed** in #673, exact head `d171ee9815813704b7214f5c812b2e6e864b6116`, merge `efb03b8c3e37bcb537952bdca2ce545765da29c8`; same examples/order/steps/stopping rule where trained |
| 26 | Metric registry | **landed** in #679, exact head `c21c4463bb23eb8f078d1ec51612cd04def112d2`, merge `62b015cf3752835bda2f4f476f118118ef4208b6`; primary family metrics and cross-family summary frozen |
| 27 | Paired uncertainty engine | **current candidate** in #683; paired effect/interval computation by seed block |
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

**26/50 merged** (#408, #489, #552, #553, #554, #555, #557, #561, #563, #566, #568, #603, #605, #606, #607, #612, #654, #656, #658, #660, #661, #663, #666, #669, #673, #679). Slice 27 is next. Only merged, exact-head-qualified substantive PRs increment the campaign counter.
