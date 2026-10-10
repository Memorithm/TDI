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
| 27 | Paired uncertainty engine | **landed** in #683, exact head `9bf33373630ea7b7bdc338da359c9eb1fcc845d9`, merge `ff7de14f6f3968f2fb5679e96d962a5f651d43b4`; paired effect/interval computation by seed block |
| 28 | Family-stratified synthesis | **landed** in #689, exact head `e34411b6b6e2e23bf36374bdd07368c2b9bf3121`, merge `14102d8745b79d41aca821a37c81c90077ea77d2`; pooled summaries cannot hide sign reversals |
| 29 | Typed failure/resource accounting | **landed** in #699, exact head `b2b26ff0ea5d141eae5d4611d8191c25e0a35047`, merge `a8d82700d00b70a7e20eea5104b09b0a7d8ee1c5`; numerical/task/resource failures retained by arm |
| 30 | Stage-C bounded preflight | **landed** in #701, exact head `d32d6a3985ae76ad5db272078de69893817efd88`, merge `4a58ff8c5aaf244d75b4c228decc9e5003359e07`; Development/Validation only; no protected/final execution |

## Phase D — structure attribution and geometry ablations (PR 31–40)

| Slice | Deliverable | Gate / definition of done |
| ---: | --- | --- |
| 31 | Torsor reduction-point ablation | **landed** in #704, exact head `66265e1ec9627a78e5ce17a658eec900469d784b`, merge `c18189fa5c50d9c224044b5368071947b9a9db89`; remove/alter transport structure under matched capacity |
| 32 | Direct vs factorized torsor bridge | **landed** in #708, exact head `9a899209a16ba2ac3cd745fa1e601901e66c2caa`, merge `c73f98a9ed69f842ee2d49cdc8b24b5fafc42f7a`; numerical equivalence monitored inside comparison harness |
| 33 | Chiral `gamma=0` ablation | **landed** in #714, exact head `da2a98ae9ef596b2dc0fb5dbd79764e829609e5c`, merge `87b5f02fa053d415b3c9f5b2d5041d4fe9c58b41`; remove parity-odd channel: matched C6 weights `(1, 0, 1)` with only `gamma` zeroed at matched capacity on the bounded matched population; exact `reference = ablated + gamma*chi` and closed enantiomorphic split per case; ChiralFavorable target equals the reference score, recorded as a degeneracy |
| 34 | Chiral parity-shuffle control | **landed** in #717, exact head `bbc415bb53475b3c59d0fa57d2daf2bc8f507a2f`, merge `e95b16f378e826638f2fdba5e4d90328d8a643d9`; six values preserved; H+/H- semantics destroyed: query/key carrier slots relabelled by the unchanged TDI-24 slice-34 parity shuffle drawn from the already-registered seed `(split domain, ChiralFavorable, 0)` (no new seed material), block-preserving/swapping draws rejected, `P^T M P != ±M` and `P^T J P != ±J` checked exactly, matched weights `(1, 0, 1)` and capacity unchanged, six-value and direct-product multisets preserved bit-for-bit; see `docs/TDI-25-CHIRAL-PARITY-SHUFFLE-CONTROL-V1.md` |
| 35 | Torsor structure-shuffle control | **landed** in #722, exact head `0a94994806a3877ed9590f77d131e4901faa7502`, merge `e49e905f133f26824d03cc20e4089aba01d6c8a1`; six values preserved; Varignon pairing semantics destroyed: twist `(v | omega)` and torsor `(R | M(P))` carrier slots relabelled by the unchanged TDI-24 slice-34 shuffle drawn from the already-registered seed `(split domain, TorsorFavorable, 0)` (no new seed material), block-preserving/swapping draws rejected, reduction and query points unchanged, six-value and untransported coordinate-product multisets preserved bit-for-bit, T6 capacity unchanged, transport term `omega.((P - Q) x R)` recorded on both sides; see `docs/TDI-25-TORSOR-STRUCTURE-SHUFFLE-CONTROL-V1.md` |
| 36 | G6 orthogonal-basis control | **landed** in #725, exact head `ffa6cae574eaec88a2711363178ff131683cfbf1`, merge `790b888a203733ee963865162ee7d382924652fb`; generic capacity under deterministic basis rotations: the four declared TDI-24 slice-36 orthogonal probes (identity, gauge `diag(R, R)`, sector mixing, generic; consumed unchanged, no new parameters) rotate query/key scalars; G6 score basis-invariant within `1e-12` relative, identity bit-exact, matched C6 on the rotated pair recorded for contrast, G6 capacity unchanged; see `docs/TDI-25-G6-ORTHOGONAL-BASIS-CONTROL-V1.md` |
| 37 | Position-geometry ablation | **landed** in #728, exact head `f1b89404af364e0cd74ed3edb19db8103a951b8c`, merge `cb4c98d7f88c6ffaefcca687b8fe2d58ae4d8125`; compare declared geometry families without implicit privileged choice: T6 on the bounded matched population under the generated geometry and every internal arm of the frozen `tdi25-position-geometry-arm-v1` registry (linear, helical, learned table; `P` at index `2c`, `Q` at `2c+1`), all reported, none selected; six query/key scalars and T6 capacity unchanged; generated arm reproduces matched T6 bit-for-bit; `External` excluded (needs caller input), recorded; see `docs/TDI-25-POSITION-GEOMETRY-ABLATION-V1.md` |
| 38 | Sequence-length scaling | **landed** in #731, exact head `a718c117a8bef8b6229823510f9150d43cf761eb`, merge `4cb438183cd8b0d4ad0e39f6b450c4fbb837419f`; matched masks, lengths and resource accounting: T6 and C6 on identical non-overlapping windows of the bounded matched population at preregistered lengths `[2, 4, 8]` (all reported, none selected) under both shared TDI-24 mask policies (full, causal) through `normalize_arm_row`; row `i` scores query `i` against every key of the window and the diagonal reproduces the matched primary scores bit-for-bit; exact accounting of score evaluations (`L^2` per window), normalizer calls, masked entries and unwindowed remainder, identical across arms; row-sum tolerance `1e-12`; see `docs/TDI-25-SEQUENCE-LENGTH-SCALING-V1.md` |
| 39 | Data-volume scaling | **landed** in #733, exact head `beacb91291326106c0483a5abad582ef7f2f30ea`, merge `b4280a341cd8771a17879d5c0e0fa8e46b046ec8`; preregistered sample counts; no adaptive arm-specific allocation: T6 and C6 on identical nested prefixes of the bounded matched population at per-block counts `[8, 16, 32]` (all reported, none selected), same seed blocks; smaller volumes verified to be exact prefixes of the largest; identical per-arm allocation and reference capacities; see `docs/TDI-25-DATA-VOLUME-SCALING-V1.md` |
| 40 | Stage-D attribution audit | **landed** in #736, exact head `6077f5d7412878717a1503e1db21376fbc7db389`, merge `15de338b190359004e8afa5b8aa57612caae0ef5`; determine which structure-specific claims remain admissible: every Phase-C/D slice (30 to 39) regenerated and validated on the bounded Development/Validation budget, flags read from each report (Stage-C preflight has no training path), per-entry FNV-1a digest of the regenerated evidence; the only admissible claim class is software semantics, structure-specific scientific attribution stays inadmissible; see `docs/TDI-25-STAGE-D-ATTRIBUTION-AUDIT-V1.md` |

## Phase E — stress, replication and evidence gate (PR 41–50)

| Slice | Deliverable | Gate / definition of done |
| ---: | --- | --- |
| 41 | Multi-seed replication | **landed** in #739, exact head `c0d31dcb233bfaa8d9cab636f69dc52b308ef16c`, merge `325766691cd0e1ecfde954f28b3f811e5a15005b`; frozen paired seed blocks: T6 and C6 on eight frozen seed blocks `[0..8)` of the bounded matched population (all families, same cases for both arms, pairing checked by canonical case digest, blocks disjoint by digest), paired discordance (T6-only/C6-only), pooled counts and descriptive block tallies; see `docs/TDI-25-MULTI-SEED-REPLICATION-V1.md` |
| 42 | Input/noise robustness | **landed** in #742, exact head `85c83e47cd48ef1be40dfe52c066e03fa784c24a`, merge `c491c891c162c97afc8ae5a760f7609f712ab5a9`; identical perturbation distributions by arm: declared deterministic perturbation families (isotropic on all 18 shared scalars, carrier-only, position-only) at declared absolute amplitudes `[1e-3, 1e-2, 1e-1]` (all reported, none selected), seed derived from the contract pin, T6 and C6 score the identical perturbed matched input on the bounded matched population; clean/perturbed matches against the evaluator-recomputed common target, match flips and label-free max score change per cell; clean scores reproduce the matched primary bit-for-bit; zero flips recorded as a degeneracy; see `docs/TDI-25-INPUT-NOISE-ROBUSTNESS-V1.md` |
| 43 | Translation/origin stress suite | **landed** in #746, exact head `1e66e1dff65315ef1b6aa96227a863863b313b96`, merge `c04fa07ad6815126276b31b6fdf03ccf013f0250`; hard torsor-relevant transformations independent of model output: rigid origin shift (both reduction points move by `d`), key re-reduction through the upstream `Torsor3::transport` and query re-reduction `v' = v + omega x d`, each leaving the physical pairing invariant, at declared offsets `|d|` in `[1, 1e3, 1e6]` (all reported, none selected) along a unit direction drawn from the contract seed per case; T6 and C6 score the identical transformed matched input on the bounded matched population; clean/stressed matches against the evaluator-recomputed common target, flips, label-free max score change and max target change per cell; C6 origin-shift invariance checked bit-for-bit; see `docs/TDI-25-TRANSLATION-ORIGIN-STRESS-V1.md` |
| 44 | Mirror/parity stress suite | **landed** in #749, exact head `0dc02135d48e1eae640bcb19e617a9381fbf8ea5`, merge `660a684f6f19b30c1f6c875c65542d3667c17885`; hard chiral-relevant transformations independent of model output: simultaneous mirror `(Mq, Mk)` (direct pairing invariant, parity-odd pairing negated), complex structure `(Jq, Jk)` (both pairings invariant) and key-only mirror `(q, Mk)`, using the upstream TDI-24 involutions on the two six-carriers with reduction points unchanged; T6 and C6 score the identical transformed matched input on the bounded matched population; clean/stressed matches against the evaluator-recomputed common target, flips, label-free max score change and max target change per cell; C6 complex-structure invariance checked bit-for-bit and C6 chiral-favorable matches required under every transformation; see `docs/TDI-25-MIRROR-PARITY-STRESS-V1.md` |
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

**44/50 merged** (#408, #489, #552, #553, #554, #555, #557, #561, #563, #566, #568, #603, #605, #606, #607, #612, #654, #656, #658, #660, #661, #663, #666, #669, #673, #679, #683, #689, #699, #701, #704, #708, #714, #717, #722, #725, #728, #731, #733, #736, #739, #742, #746, #749). Slice 45 is next. Only merged, exact-head-qualified substantive PRs increment the campaign counter.
