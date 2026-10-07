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
| 09 | Operation + storage accounting | **landed** in #556; source-level V6/C6 arithmetic and fail-closed validity-predicate counts plus carrier and mask/normalizer logical storage accounting |
| 10 | Stage-A audit/freeze | **landed** in #559; cross-contract differential/adversarial audit and fail-closed manifest |

## Phase B — task populations and leakage discipline (PR 11–20)

| Slice | Deliverable | Gate / definition of done |
| ---: | --- | --- |
| 11 | Reflection-discriminative generator | **landed** in #562; deterministic exact mirrored pairs with opposite handedness labels |
| 12 | Reflection-nuisance generator | **landed** in #565; family-namespaced exact mirrored pairs with one invariant even-sector target |
| 13 | Direction/reversal generator | **landed** in #567; deterministic query/key order reversal with opposite direction oracle and antisymmetric chi |
| 14 | Non-chiral negative-control generator | **landed** in #569; opposite targets share identical odd sectors within nuisance strata |
| 15 | Difficulty strata | **landed** in #601; deterministic bounded levels (0..=3) with fail-closed range checks, independent of model output |
| 16 | Split manifest | **landed** in #602; Development/Validation identities embedded in every case |
| 17 | Protected-label API | **landed** in #604; inference callback cannot access expected target |
| 18 | Seed registry + disjointness tests | **landed** in #608; no overlap across declared domains |
| 19 | Dataset canonicalization/hash | **landed** in #609; stable canonical record and digest |
| 20 | Stage-B data audit/freeze | **landed** in #610; leakage/adversarial audit green before training/evaluation |

## Phase C — matched training/evaluation machinery (PR 21–30)

| Slice | Deliverable | Gate / definition of done |
| ---: | --- | --- |
| 21 | V6 evaluator | **landed** in #611; deterministic non-final Development/Validation path |
| 22 | C6 evaluator | **landed** in #615; same evaluator contract and readout budget |
| 23 | Parameter-count matcher | **landed** in #655; reject unmatched trainable-capacity configurations |
| 24 | Initialization matcher | **landed** in #657; paired deterministic initialization policy |
| 25 | Optimizer/update-budget contract | **landed** in #659, exact head `922ffe5c1dabed26f8662b0be319466ef020e8a9`, merge `5f4bd6c51e81adb4ef831b3caf3aeb0db62ff498`; same examples, ordering, steps and stopping rule |
| 26 | Metric registry | **landed** in #667, exact head `9d98deba309da78fb5f57930c980ea14041af434`, merge `277896b34d7a0f76af0d176ec4c9356a5b4e68cc`; primary metric + paired secondary diagnostics frozen |
| 27 | Paired uncertainty engine | **landed** in #671, exact head `7c640354626de37400bf0f86655e6c0e21284d39`, merge `c547fa5f7f12e001f4437c3bb054d75b239099a7`; confidence intervals/effect summaries without label leakage |
| 28 | Failure taxonomy | **landed** in #677, exact head `c8bfb4aa82e6bbf9b0767ad0a3e73f93a98b6b85`, merge `b0f5c76c04fd2e67d22cad8a20aecbe825122d9d`; typed invalid/numerical/resource/task failures retained |
| 29 | Provenance envelope | **landed** in #687, exact head `cd098d81ccbedce5f0a75a9216b4d72350729f14`, merge `11245ad9e4ba93cd0580a82e1a78aa2d11760a34`; code/config/data/seed/toolchain identity per run |
| 30 | Stage-C preflight | **landed** in #698, exact head `586de6338a486bb9cbef3183f2be8faf2450d75f`, merge `60bb25afa271079181df17ca6a4f1201b291c5b5`; bounded smoke campaign; zero protected/final access |

## Phase D — attribution ablations (PR 31–40)

| Slice | Deliverable | Gate / definition of done |
| ---: | --- | --- |
| 31 | `gamma=0` ablation | **landed** in #702, exact head `56ff53b183e10f052c1d62b97c03f4991252b147`, merge `2d59303a73c39100c52b872b7f6ad61a7a1e2504`; parity-odd channel removed, all else fixed |
| 32 | `beta=0` ablation | **landed** in #705, exact head `a3b4a8e986baf4b37bc6421ab14c55d28556af41`, merge `dcbd11751aebcdc4de3882e84ea7504d81abe936`; mirror-even extra channel removed; matched C6 reference already has `beta=0`, so recorded explicitly as the identity |
| 33 | direct-only collapse | **landed** in #707, exact head `e775f1fbfc7dde86124886a8acbe6383e7ded303`, merge `3f6fd02aa68222110384405cf230e6c049fb8767`; C6 path numerically matches V6 score semantics (bit-for-bit, declared tolerance `0.0`); matched C6 reference already has `beta=0`, so the collapse shares its weights with the slice-31 `gamma=0` ablation, recorded explicitly |
| 34 | parity-shuffle control | **current stacked slice**; capacity preserved, H+/H- structure destroyed reproducibly: one fixed carrier-slot permutation drawn from the reused Slice-18 registered seed (no new seed material), block-preserving/swapping draws rejected, `P^T M P != ±M` and `P^T J P != ±J` checked exactly, weights/capacity unchanged, direct-product multiset preserved bit-for-bit |
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

**33/50 merged** (#406, #471, #487, #492, #548, #549, #550, #551, #556, #559, #562, #565, #567, #569, #601, #602, #604, #608, #609, #610, #611, #615, #655, #657, #659, #667, #671, #677, #687, #698, #702, #705, #707). Slice 34 is next. Only merged, exact-head-qualified substantive PRs increment the campaign counter.
