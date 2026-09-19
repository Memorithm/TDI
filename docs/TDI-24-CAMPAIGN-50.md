# TDI-24 — Campaign A: 50 substantive PR slices

Tracker: #391. Primary comparison: **V6 vector control vs C6 chiral candidate**.

This is a dependency-ordered research campaign, not a requirement to manufacture 50 empty pull requests. A slice counts only when it produces a concrete reviewed artifact. Later slices may be stopped, revised or marked inapplicable if earlier evidence invalidates their premise; the original slice identity remains in the ledger.

## Phase A — exact semantics and controls (PR 01–10)

| Slice | Deliverable | Gate / definition of done |
| ---: | --- | --- |
| 01 | Stage-0 programme + shared `M/J` algebra scaffold | **landed** in #406; exact tests for `M²=I`, `J²=-I`, `MJM=-J`, parity sign; experimental API only |
| 02 | Arithmetic hardening | **landed** in #471; finite-input checks, product/accumulator/weighted overflow rejection, deterministic algebra property fixtures |
| 03 | Matched V6 vector reference | **landed** in #487; distinct finite 6D vector carrier, direct score only, matched width/arithmetic, no mirror/chiral API or hidden extra state |
| 04 | Channel decomposition contract | **landed** in #492; expose `s`, `m`, `chi` separately with stable IDs, reflection parity and provenance/version tags |
| 05 | R/L enantiomorphic score pair | **landed** in #548; versioned R/L pair with provenance and exact reflection swap |
| 06 | Even/odd attention recombination contract | **landed** in #549; checked mean/contrast parity decomposition and R/L reconstruction |
| 07 | Deterministic normalizer reference | **landed** in #550; stable f64 masked softmax shared by V6/C6, fail-closed invalid rows |
| 08 | Causal/non-causal masking reference | **landed** in #551; shared full/causal mask builder and identical normalization path across arms |
| 09 | Operation + storage accounting | **current slice**; source-level V6/C6 arithmetic and fail-closed validity-predicate counts plus carrier and mask/normalizer logical storage accounting |
| 10 | Stage-A audit/freeze | adversarial review; no unresolved P0/P1 semantic defects |

## Phase B — task populations and leakage discipline (PR 11–20)

| Slice | Deliverable | Gate / definition of done |
| ---: | --- | --- |
| 11 | Reflection-discriminative generator | deterministic paired handedness tasks |
| 12 | Reflection-nuisance generator | mirrored pairs with invariant target |
| 13 | Direction/reversal generator | ordered-relational tasks without target leakage |
| 14 | Non-chiral negative-control generator | parity carries no target information |
| 15 | Difficulty strata | deterministic bounded levels independent of model output |
| 16 | Split manifest | Development/Validation identities embedded in every case |
| 17 | Protected-label API | inference callback cannot access expected target |
| 18 | Seed registry + disjointness tests | no overlap across declared domains |
| 19 | Dataset canonicalization/hash | stable canonical record and digest |
| 20 | Stage-B data audit/freeze | leakage/adversarial audit green before training/evaluation |

## Phase C — matched training/evaluation machinery (PR 21–30)

| Slice | Deliverable | Gate / definition of done |
| ---: | --- | --- |
| 21 | V6 evaluator | deterministic non-final Development/Validation path |
| 22 | C6 evaluator | same evaluator contract and readout budget |
| 23 | Parameter-count matcher | reject unmatched trainable-capacity configurations |
| 24 | Initialization matcher | paired deterministic initialization policy |
| 25 | Optimizer/update-budget contract | same examples, ordering, steps and stopping rule |
| 26 | Metric registry | primary metric + paired secondary diagnostics frozen |
| 27 | Paired uncertainty engine | confidence intervals/effect summaries without label leakage |
| 28 | Failure taxonomy | typed invalid/numerical/resource/task failures retained |
| 29 | Provenance envelope | code/config/data/seed/toolchain identity per run |
| 30 | Stage-C preflight | bounded smoke campaign; zero protected/final access |

## Phase D — attribution ablations (PR 31–40)

| Slice | Deliverable | Gate / definition of done |
| ---: | --- | --- |
| 31 | `gamma=0` ablation | parity-odd channel removed, all else fixed |
| 32 | `beta=0` ablation | mirror-even extra channel removed |
| 33 | direct-only collapse | C6 path numerically matches V6 score semantics |
| 34 | parity-shuffle control | capacity preserved, H+/H- structure destroyed reproducibly |
| 35 | fixed-M sensitivity | alternative fixed mirror bases under frozen rule |
| 36 | structure-preserving learned basis prototype | only orthogonal/constrained transforms that preserve algebra |
| 37 | head-sharing ablation | shared vs per-head chiral structure under matched capacity |
| 38 | width scaling | preregistered matched widths; no adaptive cherry-picking |
| 39 | sequence-length scaling | matched lengths/masks and bounded cost accounting |
| 40 | Stage-D attribution audit | identify which claims remain admissible after ablations |

## Phase E — robustness, replication and evidence gate (PR 41–50)

| Slice | Deliverable | Gate / definition of done |
| ---: | --- | --- |
| 41 | Multi-seed replication | frozen seed blocks and paired summaries |
| 42 | Input-noise robustness | declared perturbation families; paired arms |
| 43 | Reflection adversarial set | hard mirrored pairs generated independently of model outputs |
| 44 | Numerical precision study | f64/f32 comparison with explicit tolerance and failure accounting |
| 45 | Gradient/stability study | finite gradients, norm/variance diagnostics for trained arms |
| 46 | CPU reference cost study | measured software reference only; no GPU/production claim |
| 47 | Non-final integrated campaign | Development/Validation execution under frozen slices 01–46 |
| 48 | Statistical synthesis + preregistration freeze | freeze admissible primary contrast and rejection rules |
| 49 | Confirmatory pre-arm | content-addressed manifest/guard; **does not execute protected/final data** |
| 50 | Evidence decision artifact | record positive/null/harmful/inconclusive outcome and any downstream recommendation boundary |

## Stop rules

The campaign pauses rather than forcing a later PR when:

- an exact algebra identity fails;
- V6/C6 matching cannot be made explicit;
- a task leaks its protected target;
- Development/Validation domains are not provenance-disjoint;
- a statistical primary contrast was not frozen before the corresponding evaluation;
- a required predecessor has an unresolved material review finding;
- an execution would cross a protected/final gate without the dedicated authorization artifact.

## Progress accounting

**8/50 merged** (#406, #471, #487, #492, #548, #549, #550, #551). Slice 09 (#556) is the current candidate. Only merged, exact-head-qualified substantive PRs increment the campaign counter.
