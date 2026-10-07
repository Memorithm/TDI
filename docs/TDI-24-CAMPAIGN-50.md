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
| 34 | parity-shuffle control | **landed** in #713, exact head `17407c30e4d132b8d68af72798d9a15193400d10`, merge `8aef82dfda23baa9104178b1ff8e7dcd7652e2fa`; capacity preserved, H+/H- structure destroyed reproducibly: one fixed carrier-slot permutation drawn from the reused Slice-18 registered seed (no new seed material), block-preserving/swapping draws rejected, `P^T M P != ±M` and `P^T J P != ±J` checked exactly, weights/capacity unchanged, direct-product multiset preserved bit-for-bit |
| 35 | fixed-M sensitivity | **landed** in #718, exact head `b1af49b0e831472baaa47f097781c76dc8851513`, merge `2fb389b27150d875cbac95f64df1a08e9158a32a`; alternative fixed mirror bases under frozen rule: all `C(6,3) = 20` choices of the parity-even sector (ascending, lexicographic, canonical first, none selected), exact algebra `M'^2 = I`, `J'^T = -J'`, `J'^2 = -I`, `M' J' M' = -J'` per basis, canonical basis reproduces Stage-C C6 bit-for-bit, complementary bases give exactly opposite `chi`, C6 weights/capacity unchanged; see `docs/TDI-24-FIXED-M-SENSITIVITY-V1.md` |
| 36 | structure-preserving learned basis prototype | **landed** in #721, exact head `b40da215d4a39e5cd843713af2cb5b3bf0e81511`, merge `ee49a5e63b6402fd8169b10da543640f9b2e17c2`; only orthogonal/constrained transforms that preserve algebra: `O(theta)` = product of the `C(6,2) = 15` Givens rotations (orthogonal by construction, non-orthogonal transforms rejected), effective `O^T M O`, `O^T J O` checked for `M'^2 = I`, `J'^T = -J'`, `J'^2 = -I`, `M' J' M' = -J'` within declared tolerance `1e-12`; four declared probes (identity, gauge `diag(R, R)`, sector mixing, generic), none trained or selected; identity reproduces Stage-C C6 bit-for-bit, gauge probe score-invariant; C6 weights/capacity unchanged; see `docs/TDI-24-LEARNED-BASIS-PROTOTYPE-V1.md` |
| 37 | head-sharing ablation | **landed** in #724, exact head `e13f41eb85fd6a6d37fb5e43c37ce6576cd76233`, merge `9895cd9ee7cec4adff0dfdde862eda7d985e8731`; shared vs per-head chiral structure under matched capacity: two-head C6 score (mean of head scores, head order), shared arm uses the canonical basis on both heads, per-head arm uses slice-35 fixed mirror bases of rank `[0, 1]` (declared rule, not tuned); identical C6 weights and zero-trainable capacity on both arms; shared arm reproduces single-head Stage-C C6 bit-for-bit, per-head arm equals the mean of slice-35 basis scores bit-for-bit; see `docs/TDI-24-HEAD-SHARING-ABLATION-V1.md` |
| 38 | width scaling | **landed** in #727, exact head `59655ad0f54d01205e4e7ef90c9d5337ff4426df`, merge `0eb49ea22d750c8a542aaad7fc2aa4f56087c84a`; preregistered matched widths; no adaptive cherry-picking: all even carrier widths `2n`, `n = 1..=3` (`[2, 4, 6]`, derived from the carrier width, all reported), carrier restricted to the first `n` slots of each sector; C6 `(1, 0, 1)` and its direct-only arm `(1, 0, 0)` (weights fixed per arm across widths) on identical restricted carriers with matched zero-trainable capacity at every width; full width reproduces Stage-C C6 bit-for-bit; see `docs/TDI-24-WIDTH-SCALING-V1.md` |
| 39 | sequence-length scaling | **landed** in #730, exact head `36ff40db13d42c2a13e932194dacb94721eced48`, merge `c9d935cd77e3b0b49bb27b5e3528cbc60514b764`; preregistered lengths `[2, 4, 8]` (all reported, no length selection) under both shared mask policies (full, causal) through the shared `tdi24_attention` normalizer; C6 `(1, 0, 1)` and direct-only `(1, 0, 0)` on identical non-overlapping case windows with matched zero-trainable capacity; exact cost accounting (score evaluations `L^2` per window, normalizer calls, masked entries, unwindowed remainder) and row-sum tolerance `1e-12`; see `docs/TDI-24-SEQUENCE-LENGTH-SCALING-V1.md` |
| 40 | Stage-D attribution audit | **current stacked slice**; identify which claims remain admissible after ablations: every Phase-C/D slice (30 to 39) regenerated and validated on the bounded Development/Validation budget, flags read from each report, per-entry FNV-1a digest of the regenerated evidence; the only admissible claim class is software semantics, scientific attribution stays inadmissible (no trained, preregistered Stage-D run); see `docs/TDI-24-STAGE-D-ATTRIBUTION-AUDIT-V1.md` |

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

**39/50 merged** (#406, #471, #487, #492, #548, #549, #550, #551, #556, #559, #562, #565, #567, #569, #601, #602, #604, #608, #609, #610, #611, #615, #655, #657, #659, #667, #671, #677, #687, #698, #702, #705, #707, #713, #718, #721, #724, #727, #730). Slice 40 is next. Only merged, exact-head-qualified substantive PRs increment the campaign counter.
