//! TDI-25 Phase-C matched evaluation machinery.
//!
//! Slice 21 introduces the shared bounded, non-final evaluator envelope and
//! the T6 arm. Slice 22 adds the matched C6 arm that reuses the TDI-24 chiral
//! contract unchanged under the same envelope and readout budget. Slice 23
//! adds the matched G6 attribution-control arm that scores sealed Neutral
//! Generic6 cases through `generic_arm_score` / `GENERIC6_CONTRACT` under the
//! same envelope. MixedGeometryInput does not expose Generic6 fields, so G6
//! does not invent a mixed scoring path. Slice 24 adds an explicit
//! parameter/readout matcher that accepts only capacity-matched arm pairs on
//! the declared readout surface (trainable/update budget, readout scalars,
//! matched query/score carrier widths from `carrier_accounting`) and rejects
//! mismatches fail-closed without silently compensating. Slice 25 adds an
//! optimizer/update-budget matcher that requires identical examples count,
//! ordering identity, update steps, and stopping rule across paired arms so
//! later trained paths cannot silently diverge; Phase-C references remain
//! non-trained (`updates=0`, `ExhaustExamples`). Slice 26 freezes the Stage-C
//! metric registry under `tdi25-metric-registry-v1`: closed per-family primary
//! score-match-vs-oracle ids (TorsorFavorable/ChiralFavorable/Mixed/Neutral),
//! one cross-family summary id, and an ordered closed secondary-diagnostic set.
//! Primaries name family-level case-paired accuracy against each family's
//! sealed oracle (TDI-24 `paired_task_accuracy` analogue); they do not require
//! every arm to score every family, invent a shared Mixed scalar, or authorize
//! protected/final runs. Fail-closed on drift/empty/dupes/invention. Evaluator
//! open paths require the pinned registry. Slice 27 adds the paired
//! uncertainty engine under `tdi25-paired-uncertainty-v1`: deterministic
//! Wilson and Hoeffding/cluster-aware intervals for the primary T6-vs-C6
//! contrast by family-tagged seed block, with optional G6 attribution
//! contrasts retained only as secondary controls. Consumes already-revealed
//! match bits only (no ProtectedLabel surface). Fail-closed on empty pairs,
//! length/identity mismatch, non-finite statistics, registry drift, and
//! protected/final splits. Family + seed-block tagging keeps later stratified
//! synthesis (slice 28) from hiding reversals. Slice 28 adds family-stratified
//! synthesis under `tdi25-family-stratified-synthesis-v1`: per-family signed
//! effect / outcome class (positive/null/harmful/inconclusive) from paired
//! effect summaries, with an optional pooled summary only when no family sign
//! reversal is present versus the pooled sign; otherwise `sign_reversal_present`
//! is set and a clean pooled win cannot be claimed. Fail-closed on missing
//! required families, registry/contract drift, and protected/final. Deterministic;
//! no RNG; no ProtectedLabel. Slice 29 adds typed failure/resource accounting
//! under `tdi25-failure-resource-accounting-v1`: a closed
//! numerical/task/resource/invalid taxonomy over `EvalError`, a bounded
//! `FailureResourceLedger` that retains every failure by arm and counts
//! matched readout resources only through evaluator-backed paths, and
//! `require_complete_primary_accounting`, which refuses a clean T6-vs-C6
//! summary when the primaries attempted different cases or retained failures.
//! Protected/final errors are never retained as a class. Slice 30 adds the
//! Stage-C bounded preflight under `tdi25-stage-c-preflight-v1`: a bounded
//! smoke run that exercises the complete matched evaluator matrix, matched
//! primary blocks for every required family, the failure/resource ledger, the
//! paired uncertainty engine, and family-stratified synthesis end to end on
//! Development/Validation only. It refuses to report when any primary failure
//! is retained, never emits a scientific claim, and records zero protected/final
//! access. Slice 31 opens Phase D with the torsor reduction-point ablation under
//! `tdi25-torsor-reduction-point-ablation-v1`: on the same bounded matched
//! population, T6 is compared with an ablation that keeps every input scalar
//! and the matched capacity but drops the Varignon transport, pairing the stored
//! moment `M(P)` as if reduced at the query point. Each case carries the
//! removed transport term `omega.((P - Q) x R)` and is checked against the
//! monitored identity `reference = ablated + transport_term`; cases whose
//! reduction point already equals the query point must carry an exactly zero
//! term. The report records match counts only; attribution is reserved for the
//! Stage-D audit. Slice 32 adds the direct vs factorized torsor bridge monitor
//! under `tdi25-direct-vs-factorized-bridge-v1`: on the same bounded matched
//! population, every T6 case is scored through the unchanged factorized TDI-22
//! pairing `(v + Q x omega).R + omega.C` and through the direct pairing
//! `v.R + omega.M(Q)`, and the residual between the two forms is monitored
//! under the shared v1 tolerance. Slice 33 adds the chiral `gamma=0` ablation
//! under `tdi25-chiral-gamma-zero-ablation-v1`: on the same bounded matched
//! population, the matched C6 weights keep `alpha`/`beta` and zero only the
//! parity-odd coefficient; the exact identity `reference = ablated + gamma*chi`
//! and the closed right/left enantiomorphic split are checked per case at
//! matched capacity. Slice 34 adds the chiral parity-shuffle control under
//! `tdi25-chiral-parity-shuffle-control-v1`: on the same bounded matched
//! population, query and key carrier slots are relabelled by the unchanged
//! TDI-24 slice-34 parity shuffle, drawn from an already-registered TDI-25
//! seed (no new seed material). The six query and six key values, the
//! direct-product multiset, the matched weights and the capacity are
//! preserved bit-for-bit, while the shuffle provably breaks `M` and `J`, so
//! the `H+`/`H-` semantics are destroyed. Slice 35 adds the torsor
//! structure-shuffle control under `tdi25-torsor-structure-shuffle-control-v1`:
//! on the same bounded matched population, the twist and torsor carrier slots
//! are relabelled by the same unchanged TDI-24 slice-34 shuffle, drawn from
//! the already-registered seed `(split domain, TorsorFavorable, 0)`. The six
//! query and six key values, both geometry points, the untransported
//! coordinate-product multiset and the T6 capacity are preserved bit-for-bit,
//! while linear/angular blocks are provably mixed, so the Varignon pairing
//! semantics are destroyed. Slice 36 adds the G6 orthogonal-basis control
//! under `tdi25-g6-orthogonal-basis-control-v1`: on the same bounded matched
//! population, query and key scalars are rotated by each of the four declared
//! TDI-24 slice-36 orthogonal probes (consumed unchanged; no new parameters).
//! The generic G6 score is invariant within the upstream tolerance, the
//! identity probe is bit-exact, and the matched C6 score on the rotated pair
//! is recorded for contrast. Slice 37 adds the position-geometry ablation
//! under `tdi25-position-geometry-ablation-v1`: on the same bounded matched
//! population, T6 is scored with the generated geometry and with every
//! internal arm of the frozen position-geometry registry (linear, helical,
//! learned table), all reported, none privileged or selected. Slice 38 adds
//! sequence-length scaling under `tdi25-sequence-length-scaling-v1`: T6 and C6
//! on identical non-overlapping windows of preregistered lengths `[2, 4, 8]`
//! under both shared mask policies, with exact resource accounting. Slice 39
//! adds data-volume scaling under `tdi25-data-volume-scaling-v1`: T6 and C6
//! on identical nested prefixes of preregistered per-block counts
//! `[8, 16, 32]`, never allocated per arm. Slice 40 adds the Stage-D
//! attribution audit: slices 30 to 39 are regenerated and validated, and the
//! only admissible claim class is software semantics; structure-specific
//! scientific attribution stays inadmissible. Slice 41 (Phase E) adds
//! multi-seed replication: T6 and C6 on eight frozen paired seed blocks of
//! the matched population, with paired discordance, pooled counts and
//! descriptive block tallies. Slice 42 adds input/noise robustness under
//! `tdi25-input-noise-robustness-v1`: declared deterministic perturbation
//! families (isotropic, carrier-only, position-only) at declared amplitudes,
//! applied identically to T6 and C6. Slice 43 adds the translation/origin
//! stress suite under `tdi25-translation-origin-stress-v1`: declared
//! torsor-relevant transformations (rigid origin shift, key re-reduction,
//! query re-reduction) at declared offsets `[1, 1e3, 1e6]`, drawn from the
//! contract seed independently of any score and applied identically to T6
//! and C6. All arms consume sealed
//! Development/Validation cases, keep task oracles outside inference
//! callbacks, and reject split or contract drift. They do not train, access
//! protected/final data, or authorize a scientific claim.

/// Versioned common-population T6/C6 reference; non-final only.
pub mod matched_reference {
    include!("tdi25_matched_reference.rs");
}

pub use matched_reference::{
    ChiralGammaZeroAblationCase, ChiralParityShuffleCase, ReductionPointAblationCase,
    TorsorBridgeCase, chiral_gamma_zero_weights, chiral_parity_odd_channel,
    direct_torsor_bridge_score, evaluate_chiral_gamma_zero_ablation,
    evaluate_chiral_parity_shuffle_control, evaluate_reduction_point_ablation,
    evaluate_torsor_bridge_equivalence, gamma_zero_chiral_score, parity_shuffled_chiral_score,
    reduction_point_transport_term, untransported_torsor_score,
};
pub use matched_reference::{
    G6OrthogonalBasisCase, evaluate_g6_orthogonal_basis_control, g6_rotation_invariant,
    rotated_generic_and_chiral_scores,
};
pub use matched_reference::{
    INPUT_NOISE_AMPLITUDES, INPUT_NOISE_FAMILIES, InputNoiseCell, InputNoiseFamily,
    evaluate_input_noise_robustness, perturb_matched_input,
};
pub use matched_reference::{
    ORIGIN_STRESS_OFFSETS, ORIGIN_STRESS_TRANSFORMS, OriginStressCell, OriginStressTransform,
    evaluate_translation_origin_stress, origin_stress_direction, stress_matched_input,
};
pub use matched_reference::{
    POSITION_GEOMETRY_ABLATION_ARMS, PositionGeometryCase, evaluate_position_geometry_ablation,
    position_geometry_ablation_points,
};
pub use matched_reference::{
    SEQUENCE_SCALING_ARMS, SEQUENCE_SCALING_LENGTHS, SEQUENCE_SCALING_POLICIES,
    SEQUENCE_SCALING_ROW_SUM_TOLERANCE, SequenceScalingCell, evaluate_sequence_length_scaling,
};
pub use matched_reference::{
    TorsorStructureShuffleCase, evaluate_torsor_structure_shuffle_control,
    structure_shuffled_torsor_score,
};

use core::fmt;

use super::tdi22_torsor::TORSOR_CONTRACT;
use super::tdi24_attention::{MASKING_CONTRACT, MaskPolicy, NORMALIZER_CONTRACT};
use super::tdi24_chiral::{CHIRAL_CONTRACT, ChiralScoreWeights};
use super::tdi24_eval::{
    LEARNED_BASIS_PROBE_COUNT, LEARNED_BASIS_PROTOTYPE_CONTRACT, LearnedBasisProbe,
    PARITY_SHUFFLE_CONTROL_CONTRACT, ParityShuffle, learned_basis_probes, parity_shuffle_from_seed,
};
use super::tdi25_matched_matrix::{
    MATCHED_EVALUATOR_MATRIX_CONTRACT, matched_family_paths, require_complete_primary_matrix,
};
use super::tdi25_tasks::{
    CHIRAL_REFLECTION_TASK_CONTRACT, ChiralReflectionInput, ChiralReflectionOracle, DataSplit,
    LabeledCase, MIXED_GEOMETRY_TASK_CONTRACT, MixedGeometryInput, MixedGeometryOracle,
    NEUTRAL_CONTROL_TASK_CONTRACT, NeutralControlInput, NeutralControlOracle,
    PROTECTED_LABEL_CONTRACT, TORSOR_TRANSPORT_TASK_CONTRACT, TorsorTransportInput,
    TorsorTransportOracle, canonicalize_chiral_reflection_input, canonicalize_mixed_geometry_input,
    canonicalize_neutral_control_input, canonicalize_torsor_transport_input,
    chiral_reflection_pair_in_split, mixed_geometry_pair_in_split, neutral_control_pair_in_split,
    run_inference_callback, torsor_transport_pair_in_split,
};
use super::tdi25_tasks::{POSITION_GEOMETRY_ARM_CONTRACT, PositionGeometryArm};
use super::tdi25_tasks::{RegisteredSeed, SeedDomain, register_seed};
use super::tdi25_torsor_chiral::{
    ComparisonArm, GENERIC6_CONTRACT, PINNED_SOURCE_CONTRACTS, TaskFamily, Tdi25Error,
    carrier_accounting, chiral_arm_score, generic_arm_score, torsor_arm_score,
    validate_source_contracts,
};

/// Shared evaluator envelope for all later TDI-25 arms.
pub const EVALUATOR_ENVELOPE_CONTRACT: &str = "tdi25-evaluator-envelope-v1";

/// Versioned T6 evaluator contract.
pub const T6_EVALUATOR_CONTRACT: &str = "tdi25-t6-evaluator-v1";

/// Versioned C6 evaluator contract.
pub const C6_EVALUATOR_CONTRACT: &str = "tdi25-c6-evaluator-v1";

/// Versioned G6 evaluator contract.
pub const G6_EVALUATOR_CONTRACT: &str = "tdi25-g6-evaluator-v1";

/// Matched readout-budget contract shared by Phase-C evaluator arms.
pub const READOUT_BUDGET_CONTRACT: &str = "tdi25-readout-budget-v1";

/// Versioned parameter/readout capacity matcher contract.
pub const PARAMETER_READOUT_MATCHER_CONTRACT: &str = "tdi25-parameter-readout-matcher-v1";

/// Versioned optimizer/update-budget matcher contract.
pub const OPTIMIZER_UPDATE_BUDGET_MATCHER_CONTRACT: &str =
    "tdi25-optimizer-update-budget-matcher-v1";

/// Versioned Stage-C metric-registry contract.
pub const METRIC_REGISTRY_CONTRACT: &str = "tdi25-metric-registry-v1";

/// Versioned paired uncertainty engine contract.
pub const PAIRED_UNCERTAINTY_CONTRACT: &str = "tdi25-paired-uncertainty-v1";

/// Versioned family-stratified synthesis contract.
pub const FAMILY_STRATIFIED_SYNTHESIS_CONTRACT: &str = "tdi25-family-stratified-synthesis-v1";

/// Versioned typed failure/resource accounting contract (slice 29).
pub const FAILURE_RESOURCE_ACCOUNTING_CONTRACT: &str = "tdi25-failure-resource-accounting-v1";

/// Canonical accounted arms: T6/C6 primaries plus the G6 attribution control.
/// Stage-C bounded preflight contract pin (slice 30).
pub const STAGE_C_PREFLIGHT_CONTRACT: &str = "tdi25-stage-c-preflight-v1";

/// Phase-D torsor reduction-point ablation contract pin (slice 31).
pub const TORSOR_REDUCTION_POINT_ABLATION_CONTRACT: &str =
    "tdi25-torsor-reduction-point-ablation-v1";

/// Phase-D chiral `gamma=0` ablation contract pin (slice 33).
pub const CHIRAL_GAMMA_ZERO_ABLATION_CONTRACT: &str = "tdi25-chiral-gamma-zero-ablation-v1";

/// Phase-D chiral parity-shuffle control contract pin (slice 34).
pub const CHIRAL_PARITY_SHUFFLE_CONTROL_CONTRACT: &str = "tdi25-chiral-parity-shuffle-control-v1";

/// Phase-D torsor structure-shuffle control contract pin (slice 35).
pub const TORSOR_STRUCTURE_SHUFFLE_CONTROL_CONTRACT: &str =
    "tdi25-torsor-structure-shuffle-control-v1";

/// Phase-D G6 orthogonal-basis control contract pin (slice 36).
pub const G6_ORTHOGONAL_BASIS_CONTROL_CONTRACT: &str = "tdi25-g6-orthogonal-basis-control-v1";

/// Phase-D position-geometry ablation contract pin (slice 37).
pub const POSITION_GEOMETRY_ABLATION_CONTRACT: &str = "tdi25-position-geometry-ablation-v1";

/// Phase-D sequence-length scaling contract pin (slice 38).
pub const SEQUENCE_LENGTH_SCALING_CONTRACT: &str = "tdi25-sequence-length-scaling-v1";

/// Phase-D data-volume scaling contract pin (slice 39).
pub const DATA_VOLUME_SCALING_CONTRACT: &str = "tdi25-data-volume-scaling-v1";

/// Phase-E multi-seed replication contract pin (slice 41).
pub const MULTI_SEED_REPLICATION_CONTRACT: &str = "tdi25-multi-seed-replication-v1";

/// Relative tolerance of the monitored transport identity; identical to the
/// matched-reference v1 scalar tolerance shared by every arm.
pub const TRANSPORT_IDENTITY_RELATIVE_TOLERANCE: f64 = 1e-12;

/// Phase-D direct vs factorized torsor bridge contract pin (slice 32).
pub const DIRECT_VS_FACTORIZED_BRIDGE_CONTRACT: &str = "tdi25-direct-vs-factorized-bridge-v1";

/// Relative tolerance of the monitored bridge equivalence; reuses the
/// matched-reference v1 scalar tolerance shared by every arm (no new value).
pub const BRIDGE_EQUIVALENCE_RELATIVE_TOLERANCE: f64 = TRANSPORT_IDENTITY_RELATIVE_TOLERANCE;

/// Maximum seed blocks per family in one Stage-C preflight.
pub const MAX_PREFLIGHT_SEED_BLOCKS: u64 = 2;

/// Maximum matched cases per `(family, seed_block)` in one Stage-C preflight.
pub const MAX_PREFLIGHT_CASES_PER_BLOCK: u64 = 16;

pub const ACCOUNTED_ARMS: &[ComparisonArm] =
    &[ComparisonArm::T6, ComparisonArm::C6, ComparisonArm::G6];

/// Maximum retained failures per arm in one accounting ledger.
pub const MAX_FAILURES_PER_ARM: u64 = MAX_CASES_PER_RUN;

/// Maximum accounted attempts per arm: one full block per seed block bound.
pub const MAX_ACCOUNTED_CASES_PER_ARM: u64 =
    MAX_CASES_PER_RUN * MAX_SEED_BLOCKS_PER_SYNTHESIS as u64;

/// Canonical ordered families required for a complete Stage-C stratified synthesis.
pub const REQUIRED_SYNTHESIS_FAMILIES: &[TaskFamily] = &[
    TaskFamily::TorsorFavorable,
    TaskFamily::ChiralFavorable,
    TaskFamily::Mixed,
    TaskFamily::Neutral,
];

/// Nominal two-sided confidence level for Stage-C paired uncertainty summaries.
pub const PAIRED_UNCERTAINTY_LEVEL: f64 = 0.95;

/// Φ^{-1}(0.975) critical value for two-sided 95% Wilson intervals.
/// Deterministic constant; no RNG and no runtime table lookup.
pub const NORMAL_CRITICAL_Z_95: f64 = 1.959_963_984_540_054;

/// ln(2 / 0.05), used by deterministic two-sided 95% Hoeffding bounds.
/// Kept as a literal so the uncertainty surface does not depend on runtime logs.
pub const HOEFFDING_LOG_40: f64 = 3.688_879_454_113_936_3;

/// Primary family metric: torsor-favorable paired accuracy / score-match vs oracle.
pub const PRIMARY_FAMILY_TORSOR_FAVORABLE: &str = "family_paired_task_accuracy_torsor_favorable";

/// Primary family metric: chiral-favorable paired accuracy / score-match vs oracle.
pub const PRIMARY_FAMILY_CHIRAL_FAVORABLE: &str = "family_paired_task_accuracy_chiral_favorable";

/// Primary family metric: mixed-geometry paired accuracy / score-match vs oracle.
pub const PRIMARY_FAMILY_MIXED: &str = "family_paired_task_accuracy_mixed";

/// Primary family metric: neutral-control paired accuracy / score-match vs oracle.
pub const PRIMARY_FAMILY_NEUTRAL: &str = "family_paired_task_accuracy_neutral";

/// Cross-family summary metric id (pooled view; cannot erase family reversals).
pub const CROSS_FAMILY_PAIRED_SUMMARY: &str = "cross_family_paired_summary";

/// Secondary diagnostic: paired task outcome difference (T6 vs C6).
pub const SECONDARY_PAIRED_OUTCOME_DIFFERENCE: &str = "paired_outcome_difference";

/// Secondary diagnostic: torsor transport / reduction-point identity error.
pub const SECONDARY_TORSOR_TRANSPORT_REDUCTION_POINT_IDENTITY_ERROR: &str =
    "torsor_transport_reduction_point_identity_error";

/// Secondary diagnostic: chiral mirror-swap / parity identity error.
pub const SECONDARY_CHIRAL_MIRROR_SWAP_PARITY_IDENTITY_ERROR: &str =
    "chiral_mirror_swap_parity_identity_error";

/// Secondary diagnostic: G6 attribution contrast versus T6/C6.
pub const SECONDARY_G6_ATTRIBUTION_CONTRAST: &str = "g6_attribution_contrast";

/// Secondary diagnostic: calibration / stability where scores are exposed.
pub const SECONDARY_CALIBRATION_STABILITY: &str = "calibration_stability";

/// Secondary diagnostic: operation / memory / runtime under a qualified env only.
pub const SECONDARY_OP_COUNT_MEMORY_RUNTIME: &str = "op_count_memory_runtime";

/// Maximum primary family metrics admitted in one frozen registry.
pub const MAX_PRIMARY_FAMILY_METRICS: usize = 4;

/// Maximum secondary diagnostics admitted in one frozen registry.
pub const MAX_SECONDARY_DIAGNOSTICS: usize = 6;

/// Shared ordering identity for non-trained Phase-C optimizer budgets.
pub const NON_TRAINED_ORDERING_ID: u64 = 0;

/// Maximum cases admitted to one non-final evaluator run.
pub const MAX_CASES_PER_RUN: u64 = 64;

/// Maximum seed blocks admitted to one bounded family synthesis.
pub const MAX_SEED_BLOCKS_PER_SYNTHESIS: usize = 64;

/// Maximum retained scalar fields per case (score plus oracle-match flag).
pub const MAX_READOUT_SCALARS_PER_CASE: u64 = 2;

/// Slice-21 is a non-trained reference path.
pub const NON_TRAINED_UPDATE_BUDGET: u64 = 0;

/// Fixed evaluator readout budget, to be reused unchanged by C6 and G6.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReadoutBudget {
    pub max_cases: u64,
    pub max_readout_scalars_per_case: u64,
    pub updates: u64,
    pub contract: &'static str,
}

impl ReadoutBudget {
    /// Canonical matched non-trained budget.
    #[must_use]
    pub const fn matched_non_trained() -> Self {
        Self {
            max_cases: MAX_CASES_PER_RUN,
            max_readout_scalars_per_case: MAX_READOUT_SCALARS_PER_CASE,
            updates: NON_TRAINED_UPDATE_BUDGET,
            contract: READOUT_BUDGET_CONTRACT,
        }
    }
}

/// Per-arm parameter and readout capacity descriptor.
///
/// Non-trained Phase-C reference arms declare zero trainable parameters and
/// zero updates. Carrier widths come from `carrier_accounting`: the matched
/// readout surface is the shared query/score pairing width; T6 may still
/// declare extra key/external geometry without silently padding other arms.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParameterReadoutCapacity {
    /// Arm whose capacity is declared.
    pub arm: ComparisonArm,
    /// Exact trainable parameter count (0 on the non-trained Stage-C path).
    pub trainable_parameters: u64,
    /// Optimizer/update count retained in the shared readout budget.
    pub updates: u64,
    /// Maximum cases admitted to one run.
    pub max_cases: u64,
    /// Maximum readout scalars retained per case.
    pub max_readout_scalars_per_case: u64,
    /// Matched score-pairing query width from `carrier_accounting`.
    pub query_components: u64,
    /// Matched score-pairing score width from `carrier_accounting`.
    pub score_components: u64,
    /// Declared key width from `carrier_accounting` (T6 may differ).
    pub key_components: u64,
    /// Declared external geometry width from `carrier_accounting`.
    pub external_geometry_components: u64,
    /// Matcher contract pin.
    pub matcher_contract: &'static str,
    /// Shared readout-budget contract pin.
    pub budget_contract: &'static str,
}

impl ParameterReadoutCapacity {
    /// Build a non-trained reference capacity from the arm's carrier accounting
    /// and the shared readout budget.
    #[must_use]
    pub const fn reference(arm: ComparisonArm) -> Self {
        let accounting = carrier_accounting(arm);
        let budget = ReadoutBudget::matched_non_trained();
        Self {
            arm,
            trainable_parameters: 0,
            updates: budget.updates,
            max_cases: budget.max_cases,
            max_readout_scalars_per_case: budget.max_readout_scalars_per_case,
            query_components: accounting.query_components as u64,
            score_components: accounting.score_components as u64,
            key_components: accounting.key_components as u64,
            external_geometry_components: accounting.external_geometry_components as u64,
            matcher_contract: PARAMETER_READOUT_MATCHER_CONTRACT,
            budget_contract: READOUT_BUDGET_CONTRACT,
        }
    }

    /// Canonical non-trained T6 reference capacity.
    #[must_use]
    pub const fn reference_t6() -> Self {
        Self::reference(ComparisonArm::T6)
    }

    /// Canonical non-trained C6 reference capacity.
    #[must_use]
    pub const fn reference_c6() -> Self {
        Self::reference(ComparisonArm::C6)
    }

    /// Canonical non-trained G6 reference capacity.
    #[must_use]
    pub const fn reference_g6() -> Self {
        Self::reference(ComparisonArm::G6)
    }
}

/// Witness that two arm parameter/readout capacities are matched.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MatchedParameterReadout {
    pub trainable_parameters: u64,
    pub updates: u64,
    pub max_cases: u64,
    pub max_readout_scalars_per_case: u64,
    pub query_components: u64,
    pub score_components: u64,
    pub left_arm: ComparisonArm,
    pub right_arm: ComparisonArm,
    pub matcher_contract: &'static str,
    pub budget_contract: &'static str,
}

fn validate_capacity_budget(capacity: ParameterReadoutCapacity) -> Result<(), EvalError> {
    if capacity.updates != NON_TRAINED_UPDATE_BUDGET
        || capacity.max_cases == 0
        || capacity.max_cases > MAX_CASES_PER_RUN
        || capacity.max_readout_scalars_per_case < 2
        || capacity.max_readout_scalars_per_case > MAX_READOUT_SCALARS_PER_CASE
    {
        return Err(EvalError::InvalidBudget);
    }
    Ok(())
}

fn validate_capacity_carrier(capacity: ParameterReadoutCapacity) -> Result<(), EvalError> {
    let accounting = carrier_accounting(capacity.arm);
    if capacity.query_components != accounting.query_components as u64
        || capacity.score_components != accounting.score_components as u64
        || capacity.key_components != accounting.key_components as u64
        || capacity.external_geometry_components != accounting.external_geometry_components as u64
    {
        return Err(EvalError::ContractMismatch(
            "parameter_readout_carrier_accounting",
        ));
    }
    Ok(())
}

/// Accept only matched parameter/readout configurations; reject mismatches
/// fail-closed without silently padding or compensating.
///
/// Attribution pairs (T6↔G6, C6↔G6, T6↔C6) must agree on trainable/update
/// counts, readout budget fields, and the matched query/score carrier widths.
/// T6's extra key/external geometry remains declared via `carrier_accounting`
/// and is not used to pad C6/G6.
pub fn match_parameter_readouts(
    left: ParameterReadoutCapacity,
    right: ParameterReadoutCapacity,
) -> Result<MatchedParameterReadout, EvalError> {
    if left.matcher_contract != PARAMETER_READOUT_MATCHER_CONTRACT
        || right.matcher_contract != PARAMETER_READOUT_MATCHER_CONTRACT
    {
        return Err(EvalError::ContractMismatch(
            "parameter_readout_matcher_contract",
        ));
    }
    if left.budget_contract != READOUT_BUDGET_CONTRACT
        || right.budget_contract != READOUT_BUDGET_CONTRACT
    {
        return Err(EvalError::ContractMismatch("budget_contract"));
    }
    validate_capacity_budget(left)?;
    validate_capacity_budget(right)?;
    validate_capacity_carrier(left)?;
    validate_capacity_carrier(right)?;
    if left.arm == right.arm {
        return Err(EvalError::ContractMismatch(
            "parameter_readout_distinct_arms",
        ));
    }
    if left.trainable_parameters != right.trainable_parameters
        || left.updates != right.updates
        || left.max_cases != right.max_cases
        || left.max_readout_scalars_per_case != right.max_readout_scalars_per_case
        || left.query_components != right.query_components
        || left.score_components != right.score_components
    {
        return Err(EvalError::ParameterReadoutMismatch {
            left_arm: left.arm,
            right_arm: right.arm,
            left_trainable: left.trainable_parameters,
            right_trainable: right.trainable_parameters,
            left_updates: left.updates,
            right_updates: right.updates,
            left_query_components: left.query_components,
            right_query_components: right.query_components,
        });
    }
    Ok(MatchedParameterReadout {
        trainable_parameters: left.trainable_parameters,
        updates: left.updates,
        max_cases: left.max_cases,
        max_readout_scalars_per_case: left.max_readout_scalars_per_case,
        query_components: left.query_components,
        score_components: left.score_components,
        left_arm: left.arm,
        right_arm: right.arm,
        matcher_contract: PARAMETER_READOUT_MATCHER_CONTRACT,
        budget_contract: READOUT_BUDGET_CONTRACT,
    })
}

/// Declared stopping rule for a Phase-C optimizer/update budget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoppingRule {
    /// Non-trained path: stop after the declared example budget (zero updates).
    ExhaustExamples,
    /// Trained path: stop after the declared fixed update count.
    FixedUpdates,
}

impl StoppingRule {
    /// Stable lowercase label for manifests and audits.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExhaustExamples => "exhaust_examples",
            Self::FixedUpdates => "fixed_updates",
        }
    }
}

/// Explicit optimizer/update budget for one Phase-C arm.
///
/// Paired T6/C6/G6 attribution configurations must publish identical examples,
/// ordering identity, update steps and stopping rule rather than silently
/// diverging. Phase-C reference arms remain non-trained (`examples=0`,
/// `updates=0`, `ExhaustExamples`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OptimizerUpdateBudget {
    /// Arm whose budget is declared.
    pub arm: ComparisonArm,
    /// Number of examples admitted under this budget (0 on non-trained refs).
    pub examples: u64,
    /// Parameter-update steps (zero on the non-trained reference path).
    pub updates: u64,
    /// Shared deterministic ordering identity across paired arms.
    pub ordering_id: u64,
    /// Stopping rule applied identically to both arms.
    pub stopping: StoppingRule,
    /// Matcher contract pin.
    pub matcher_contract: &'static str,
}

impl OptimizerUpdateBudget {
    /// Build a non-trained matched reference budget for one arm.
    #[must_use]
    pub const fn matched_non_trained(arm: ComparisonArm) -> Self {
        Self {
            arm,
            examples: 0,
            updates: NON_TRAINED_UPDATE_BUDGET,
            ordering_id: NON_TRAINED_ORDERING_ID,
            stopping: StoppingRule::ExhaustExamples,
            matcher_contract: OPTIMIZER_UPDATE_BUDGET_MATCHER_CONTRACT,
        }
    }

    /// Canonical non-trained T6 reference budget.
    #[must_use]
    pub const fn reference_t6() -> Self {
        Self::matched_non_trained(ComparisonArm::T6)
    }

    /// Canonical non-trained C6 reference budget.
    #[must_use]
    pub const fn reference_c6() -> Self {
        Self::matched_non_trained(ComparisonArm::C6)
    }

    /// Canonical non-trained G6 reference budget.
    #[must_use]
    pub const fn reference_g6() -> Self {
        Self::matched_non_trained(ComparisonArm::G6)
    }
}

/// Witness that two optimizer/update budgets are matched under Slice 25.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MatchedOptimizerUpdateBudget {
    pub examples: u64,
    pub updates: u64,
    pub ordering_id: u64,
    pub stopping: StoppingRule,
    pub left_arm: ComparisonArm,
    pub right_arm: ComparisonArm,
    pub matcher_contract: &'static str,
}

fn validate_optimizer_update_budget(budget: OptimizerUpdateBudget) -> Result<(), EvalError> {
    match budget.stopping {
        StoppingRule::ExhaustExamples => {
            // Non-trained / exhaust path: zero updates; examples may be 0
            // (non-trained ref) or a sealed case count.
            if budget.updates != NON_TRAINED_UPDATE_BUDGET {
                return Err(EvalError::InvalidBudget);
            }
        }
        StoppingRule::FixedUpdates => {
            // Trained path: require a positive update count and at least one
            // example so a matched witness cannot admit an unusable budget.
            if budget.updates == 0 || budget.examples == 0 {
                return Err(EvalError::InvalidBudget);
            }
        }
    }
    Ok(())
}

/// Accept only matched optimizer/update budgets; reject unpaired examples,
/// ordering, steps or stopping rules fail-closed without silent compensation.
///
/// Attribution pairs (T6↔G6, C6↔G6, T6↔C6) must agree on examples count,
/// ordering identity, update steps, and stopping rule. Each arm's budget is
/// validated against stopping-rule invariants before comparison so equal
/// malformed trained budgets cannot produce a matched witness. Later trained
/// paths must still pass through this matcher so mismatched budgets cannot
/// diverge silently.
pub fn match_optimizer_update_budgets(
    left: OptimizerUpdateBudget,
    right: OptimizerUpdateBudget,
) -> Result<MatchedOptimizerUpdateBudget, EvalError> {
    if left.matcher_contract != OPTIMIZER_UPDATE_BUDGET_MATCHER_CONTRACT
        || right.matcher_contract != OPTIMIZER_UPDATE_BUDGET_MATCHER_CONTRACT
    {
        return Err(EvalError::ContractMismatch(
            "optimizer_update_budget_matcher_contract",
        ));
    }
    validate_optimizer_update_budget(left)?;
    validate_optimizer_update_budget(right)?;
    if left.arm == right.arm {
        return Err(EvalError::ContractMismatch(
            "optimizer_update_budget_distinct_arms",
        ));
    }
    if left.examples != right.examples
        || left.updates != right.updates
        || left.ordering_id != right.ordering_id
        || left.stopping != right.stopping
    {
        return Err(EvalError::OptimizerUpdateBudgetMismatch {
            left_arm: left.arm,
            right_arm: right.arm,
            left_examples: left.examples,
            right_examples: right.examples,
            left_updates: left.updates,
            right_updates: right.updates,
        });
    }
    Ok(MatchedOptimizerUpdateBudget {
        examples: left.examples,
        updates: left.updates,
        ordering_id: left.ordering_id,
        stopping: left.stopping,
        left_arm: left.arm,
        right_arm: right.arm,
        matcher_contract: OPTIMIZER_UPDATE_BUDGET_MATCHER_CONTRACT,
    })
}

/// Closed-set primary family quality metric for Stage-C evaluation.
///
/// One id per task family: case-paired accuracy / score-match rate versus that
/// family's sealed oracle on Development/Validation only (TDI-24
/// `paired_task_accuracy` analogue). Freezing the id does not require every
/// comparison arm to score every family, invent a shared Mixed scalar across
/// torsor/chiral components, or authorize protected/final evaluation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PrimaryFamilyMetricId {
    /// TorsorFavorable family paired task accuracy.
    TorsorFavorable,
    /// ChiralFavorable family paired task accuracy.
    ChiralFavorable,
    /// Mixed family paired task accuracy.
    Mixed,
    /// Neutral family paired task accuracy.
    Neutral,
}

impl PrimaryFamilyMetricId {
    /// Stable lowercase metric id.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TorsorFavorable => PRIMARY_FAMILY_TORSOR_FAVORABLE,
            Self::ChiralFavorable => PRIMARY_FAMILY_CHIRAL_FAVORABLE,
            Self::Mixed => PRIMARY_FAMILY_MIXED,
            Self::Neutral => PRIMARY_FAMILY_NEUTRAL,
        }
    }

    /// Task family this primary metric reports.
    #[must_use]
    pub const fn family(self) -> TaskFamily {
        match self {
            Self::TorsorFavorable => TaskFamily::TorsorFavorable,
            Self::ChiralFavorable => TaskFamily::ChiralFavorable,
            Self::Mixed => TaskFamily::Mixed,
            Self::Neutral => TaskFamily::Neutral,
        }
    }

    /// Canonical ordered Stage-C primary family metric set.
    #[must_use]
    pub const fn admitted_set() -> &'static [Self] {
        PINNED_PRIMARY_FAMILY_METRICS
    }
}

/// Parse a primary family metric id from the closed Stage-C set.
pub fn parse_primary_family_metric_id(label: &str) -> Result<PrimaryFamilyMetricId, EvalError> {
    match label {
        PRIMARY_FAMILY_TORSOR_FAVORABLE => Ok(PrimaryFamilyMetricId::TorsorFavorable),
        PRIMARY_FAMILY_CHIRAL_FAVORABLE => Ok(PrimaryFamilyMetricId::ChiralFavorable),
        PRIMARY_FAMILY_MIXED => Ok(PrimaryFamilyMetricId::Mixed),
        PRIMARY_FAMILY_NEUTRAL => Ok(PrimaryFamilyMetricId::Neutral),
        "" => Err(EvalError::MetricRegistryInvalid {
            reason: "empty_primary",
        }),
        _ => Err(EvalError::MetricRegistryInvalid {
            reason: "unknown_primary",
        }),
    }
}

/// Canonical ordered primary family metrics frozen for Stage-C.
pub const PINNED_PRIMARY_FAMILY_METRICS: &[PrimaryFamilyMetricId] = &[
    PrimaryFamilyMetricId::TorsorFavorable,
    PrimaryFamilyMetricId::ChiralFavorable,
    PrimaryFamilyMetricId::Mixed,
    PrimaryFamilyMetricId::Neutral,
];

/// Closed-set cross-family summary metric for Stage-C evaluation.
///
/// A pooled view only; family-stratified synthesis must still surface
/// family-specific sign reversals (slice 28).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CrossFamilySummaryMetricId {
    /// Cross-family paired summary over the four primary family metrics.
    CrossFamilyPairedSummary,
}

impl CrossFamilySummaryMetricId {
    /// Stable lowercase metric id.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CrossFamilyPairedSummary => CROSS_FAMILY_PAIRED_SUMMARY,
        }
    }
}

/// Parse a cross-family summary metric id from the closed Stage-C set.
pub fn parse_cross_family_summary_metric_id(
    label: &str,
) -> Result<CrossFamilySummaryMetricId, EvalError> {
    match label {
        CROSS_FAMILY_PAIRED_SUMMARY => Ok(CrossFamilySummaryMetricId::CrossFamilyPairedSummary),
        "" => Err(EvalError::MetricRegistryInvalid {
            reason: "empty_cross_family_summary",
        }),
        _ => Err(EvalError::MetricRegistryInvalid {
            reason: "unknown_cross_family_summary",
        }),
    }
}

/// Closed-set secondary diagnostics admitted for Stage-C under the frozen registry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SecondaryDiagnosticId {
    /// Paired task outcome difference across matched arms (T6 vs C6).
    PairedOutcomeDifference,
    /// Torsor transport / reduction-point identity error.
    TorsorTransportReductionPointIdentityError,
    /// Chiral mirror-swap / parity identity error.
    ChiralMirrorSwapParityIdentityError,
    /// G6 attribution contrast versus the primary T6/C6 pair.
    G6AttributionContrast,
    /// Calibration / stability diagnostics when predictions expose scores.
    CalibrationStability,
    /// Operation count, memory, and runtime under an explicitly qualified environment.
    OpCountMemoryRuntime,
}

impl SecondaryDiagnosticId {
    /// Stable lowercase diagnostic id.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PairedOutcomeDifference => SECONDARY_PAIRED_OUTCOME_DIFFERENCE,
            Self::TorsorTransportReductionPointIdentityError => {
                SECONDARY_TORSOR_TRANSPORT_REDUCTION_POINT_IDENTITY_ERROR
            }
            Self::ChiralMirrorSwapParityIdentityError => {
                SECONDARY_CHIRAL_MIRROR_SWAP_PARITY_IDENTITY_ERROR
            }
            Self::G6AttributionContrast => SECONDARY_G6_ATTRIBUTION_CONTRAST,
            Self::CalibrationStability => SECONDARY_CALIBRATION_STABILITY,
            Self::OpCountMemoryRuntime => SECONDARY_OP_COUNT_MEMORY_RUNTIME,
        }
    }

    /// Canonical ordered Stage-C secondary diagnostic set.
    #[must_use]
    pub const fn admitted_set() -> &'static [Self] {
        PINNED_SECONDARY_DIAGNOSTICS
    }
}

/// Parse a secondary diagnostic id from the closed Stage-C set.
pub fn parse_secondary_diagnostic_id(label: &str) -> Result<SecondaryDiagnosticId, EvalError> {
    match label {
        SECONDARY_PAIRED_OUTCOME_DIFFERENCE => Ok(SecondaryDiagnosticId::PairedOutcomeDifference),
        SECONDARY_TORSOR_TRANSPORT_REDUCTION_POINT_IDENTITY_ERROR => {
            Ok(SecondaryDiagnosticId::TorsorTransportReductionPointIdentityError)
        }
        SECONDARY_CHIRAL_MIRROR_SWAP_PARITY_IDENTITY_ERROR => {
            Ok(SecondaryDiagnosticId::ChiralMirrorSwapParityIdentityError)
        }
        SECONDARY_G6_ATTRIBUTION_CONTRAST => Ok(SecondaryDiagnosticId::G6AttributionContrast),
        SECONDARY_CALIBRATION_STABILITY => Ok(SecondaryDiagnosticId::CalibrationStability),
        SECONDARY_OP_COUNT_MEMORY_RUNTIME => Ok(SecondaryDiagnosticId::OpCountMemoryRuntime),
        _ => Err(EvalError::MetricRegistryInvalid {
            reason: "unknown_secondary",
        }),
    }
}

/// Canonical ordered secondary diagnostics frozen for Stage-C.
pub const PINNED_SECONDARY_DIAGNOSTICS: &[SecondaryDiagnosticId] = &[
    SecondaryDiagnosticId::PairedOutcomeDifference,
    SecondaryDiagnosticId::TorsorTransportReductionPointIdentityError,
    SecondaryDiagnosticId::ChiralMirrorSwapParityIdentityError,
    SecondaryDiagnosticId::G6AttributionContrast,
    SecondaryDiagnosticId::CalibrationStability,
    SecondaryDiagnosticId::OpCountMemoryRuntime,
];

/// Frozen Stage-C metric registry: ordered primary family metrics, one
/// cross-family summary, and ordered secondaries.
///
/// Experimental and non-final only; never authorises protected/final evaluation
/// or a scientific superiority claim. The cross-family summary id is frozen for
/// later stratified synthesis (slice 28); it does not erase family-specific
/// reversals or invent observations an arm cannot produce.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MetricRegistry {
    /// Ordered primary family quality metrics frozen before evaluation.
    pub primary_family_metrics: &'static [PrimaryFamilyMetricId],
    /// Cross-family summary metric frozen before evaluation.
    pub cross_family_summary: CrossFamilySummaryMetricId,
    /// Ordered closed-set secondary diagnostics admitted for Stage-C.
    pub secondaries: &'static [SecondaryDiagnosticId],
    /// Registry contract pin.
    pub registry_contract: &'static str,
    /// Must remain true: registry is experimental and non-final only.
    pub experimental_non_final: bool,
}

impl MetricRegistry {
    /// Canonical frozen Stage-C metric registry under the versioned contract.
    #[must_use]
    pub const fn pinned() -> Self {
        Self {
            primary_family_metrics: PINNED_PRIMARY_FAMILY_METRICS,
            cross_family_summary: CrossFamilySummaryMetricId::CrossFamilyPairedSummary,
            secondaries: PINNED_SECONDARY_DIAGNOSTICS,
            registry_contract: METRIC_REGISTRY_CONTRACT,
            experimental_non_final: true,
        }
    }

    /// Reject any registry that is not the exact pinned Stage-C set.
    ///
    /// TDI-25 `DataSplit` already admits only Development/Validation, so the
    /// split gate is type-enforced; this still validates the registry itself.
    pub fn admit_split(self, _split: DataSplit) -> Result<(), EvalError> {
        validate_metric_registry(&self)
    }
}

/// Freeze a metric registry under the versioned contract; fail-closed on invalid sets.
pub fn freeze_metric_registry(
    primary_family_metrics: &'static [PrimaryFamilyMetricId],
    cross_family_summary: Option<CrossFamilySummaryMetricId>,
    secondaries: &'static [SecondaryDiagnosticId],
) -> Result<MetricRegistry, EvalError> {
    if primary_family_metrics.is_empty() {
        return Err(EvalError::MetricRegistryInvalid {
            reason: "empty_primary",
        });
    }
    let Some(cross_family_summary) = cross_family_summary else {
        return Err(EvalError::MetricRegistryInvalid {
            reason: "empty_cross_family_summary",
        });
    };
    let registry = MetricRegistry {
        primary_family_metrics,
        cross_family_summary,
        secondaries,
        registry_contract: METRIC_REGISTRY_CONTRACT,
        experimental_non_final: true,
    };
    validate_metric_registry(&registry)?;
    Ok(registry)
}

/// Validate a metric registry: reject contract drift, empty primary, empty
/// cross-family summary, duplicate secondaries, oversized sets, invented ids,
/// non-experimental finals, and any primary/secondary sequence that is not the
/// complete canonical ordered registry.
pub fn validate_metric_registry(registry: &MetricRegistry) -> Result<(), EvalError> {
    if registry.registry_contract != METRIC_REGISTRY_CONTRACT {
        return Err(EvalError::ContractMismatch("metric_registry_contract"));
    }
    if !registry.experimental_non_final {
        return Err(EvalError::MetricRegistryInvalid {
            reason: "experimental_non_final_required",
        });
    }
    if registry.primary_family_metrics.is_empty() {
        return Err(EvalError::MetricRegistryInvalid {
            reason: "empty_primary",
        });
    }
    if registry.primary_family_metrics.len() > MAX_PRIMARY_FAMILY_METRICS {
        return Err(EvalError::MetricRegistryInvalid {
            reason: "too_many_primary_family_metrics",
        });
    }
    for (index, primary) in registry.primary_family_metrics.iter().enumerate() {
        parse_primary_family_metric_id(primary.as_str())?;
        for prior in &registry.primary_family_metrics[..index] {
            if prior == primary {
                return Err(EvalError::MetricRegistryInvalid {
                    reason: "duplicate_primary",
                });
            }
        }
        if !PINNED_PRIMARY_FAMILY_METRICS.contains(primary) {
            return Err(EvalError::MetricRegistryInvalid {
                reason: "unknown_primary",
            });
        }
    }
    if registry.primary_family_metrics != PINNED_PRIMARY_FAMILY_METRICS {
        return Err(EvalError::MetricRegistryInvalid {
            reason: "primary_sequence_mismatch",
        });
    }
    parse_cross_family_summary_metric_id(registry.cross_family_summary.as_str())?;
    if registry.cross_family_summary != CrossFamilySummaryMetricId::CrossFamilyPairedSummary {
        return Err(EvalError::MetricRegistryInvalid {
            reason: "unknown_cross_family_summary",
        });
    }
    if registry.secondaries.len() > MAX_SECONDARY_DIAGNOSTICS {
        return Err(EvalError::MetricRegistryInvalid {
            reason: "too_many_secondaries",
        });
    }
    for (index, secondary) in registry.secondaries.iter().enumerate() {
        parse_secondary_diagnostic_id(secondary.as_str())?;
        for prior in &registry.secondaries[..index] {
            if prior == secondary {
                return Err(EvalError::MetricRegistryInvalid {
                    reason: "duplicate_secondary",
                });
            }
        }
        if !PINNED_SECONDARY_DIAGNOSTICS.contains(secondary) {
            return Err(EvalError::MetricRegistryInvalid {
                reason: "unknown_secondary",
            });
        }
    }
    if registry.secondaries != PINNED_SECONDARY_DIAGNOSTICS {
        return Err(EvalError::MetricRegistryInvalid {
            reason: "secondary_sequence_mismatch",
        });
    }
    Ok(())
}

/// Immutable configuration for one Phase-C evaluator run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EvaluatorConfig {
    pub split: DataSplit,
    pub budget: ReadoutBudget,
    pub envelope_contract: &'static str,
    pub arm_contract: &'static str,
    /// Validated, exact Stage-C metric registry.
    pub metric_registry: MetricRegistry,
}

impl EvaluatorConfig {
    /// Construct a Development/Validation-only T6 configuration.
    #[must_use]
    pub const fn t6(split: DataSplit) -> Self {
        Self {
            split,
            budget: ReadoutBudget::matched_non_trained(),
            envelope_contract: EVALUATOR_ENVELOPE_CONTRACT,
            arm_contract: T6_EVALUATOR_CONTRACT,
            metric_registry: MetricRegistry::pinned(),
        }
    }

    /// Construct a Development/Validation-only C6 configuration.
    #[must_use]
    pub const fn c6(split: DataSplit) -> Self {
        Self {
            split,
            budget: ReadoutBudget::matched_non_trained(),
            envelope_contract: EVALUATOR_ENVELOPE_CONTRACT,
            arm_contract: C6_EVALUATOR_CONTRACT,
            metric_registry: MetricRegistry::pinned(),
        }
    }

    /// Construct a Development/Validation-only G6 configuration.
    #[must_use]
    pub const fn g6(split: DataSplit) -> Self {
        Self {
            split,
            budget: ReadoutBudget::matched_non_trained(),
            envelope_contract: EVALUATOR_ENVELOPE_CONTRACT,
            arm_contract: G6_EVALUATOR_CONTRACT,
            metric_registry: MetricRegistry::pinned(),
        }
    }
}

/// Outcome retained for one T6 case.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct T6Outcome {
    pub score: f64,
    pub matches_oracle: bool,
}

/// Immutable non-final T6 evaluation record.
#[derive(Clone, Debug, PartialEq)]
pub struct T6EvalRecord {
    split: DataSplit,
    family: TaskFamily,
    case_id: u64,
    /// Retained seed-block / pair identity from the sealed oracle.
    seed_block: u64,
    outcome: T6Outcome,
    canonical_digest: String,
    envelope_contract: &'static str,
    arm_contract: &'static str,
    budget_contract: &'static str,
    metric_registry_contract: &'static str,
    source_torsor_contract: &'static str,
    label_contract: &'static str,
}

/// Bounded accumulator for one deterministic non-final T6 run.
#[derive(Clone, Debug, PartialEq)]
pub struct T6EvaluatorRun {
    config: EvaluatorConfig,
    records: Vec<T6EvalRecord>,
}

impl T6EvaluatorRun {
    /// Open a run only when every contract pin and budget is valid.
    pub fn open(config: EvaluatorConfig) -> Result<Self, EvalError> {
        config.metric_registry.admit_split(config.split)?;
        if config.envelope_contract != EVALUATOR_ENVELOPE_CONTRACT {
            return Err(EvalError::ContractMismatch("envelope_contract"));
        }
        if config.arm_contract != T6_EVALUATOR_CONTRACT {
            return Err(EvalError::ContractMismatch("arm_contract"));
        }
        if config.budget.contract != READOUT_BUDGET_CONTRACT {
            return Err(EvalError::ContractMismatch("budget_contract"));
        }
        if config.budget.max_cases == 0
            || config.budget.max_cases > MAX_CASES_PER_RUN
            || config.budget.max_readout_scalars_per_case < 2
            || config.budget.max_readout_scalars_per_case > MAX_READOUT_SCALARS_PER_CASE
            || config.budget.updates != NON_TRAINED_UPDATE_BUDGET
        {
            return Err(EvalError::InvalidBudget);
        }
        let sources = validate_source_contracts().map_err(EvalError::Bridge)?;
        if sources.torsor != TORSOR_CONTRACT || sources.torsor != PINNED_SOURCE_CONTRACTS.torsor {
            return Err(EvalError::ContractMismatch("source_torsor_contract"));
        }
        Ok(Self {
            config,
            records: Vec::new(),
        })
    }

    /// Borrow emitted records in admission order.
    #[must_use]
    pub fn records(&self) -> &[T6EvalRecord] {
        &self.records
    }

    /// Evaluate a sealed torsor-favorable case via the factorized TDI-22 path.
    pub fn evaluate_torsor_transport(
        &mut self,
        case: &LabeledCase<TorsorTransportInput, TorsorTransportOracle>,
    ) -> Result<&T6EvalRecord, EvalError> {
        self.reserve_case(case.inference_input().split)?;
        let input = case.inference_input();
        let oracle = case.protected_label().reveal_for_evaluation();
        if case.inference_input().generator_contract != TORSOR_TRANSPORT_TASK_CONTRACT
            || oracle.generator_contract != TORSOR_TRANSPORT_TASK_CONTRACT
            || input.task_family != TaskFamily::TorsorFavorable
            || case.label_contract() != PROTECTED_LABEL_CONTRACT
            || !is_canonical_torsor_case(input, oracle)
        {
            return Err(EvalError::ContractMismatch("torsor_transport_case"));
        }
        let score =
            run_inference_callback(case, score_torsor_transport).map_err(EvalError::Bridge)?;
        let record = T6EvalRecord {
            split: input.split,
            family: input.task_family,
            case_id: input.case_id,
            seed_block: oracle.pair_id,
            outcome: T6Outcome {
                score,
                matches_oracle: approximately_equal(score, oracle.expected_score),
            },
            canonical_digest: canonicalize_torsor_transport_input(input).digest,
            envelope_contract: EVALUATOR_ENVELOPE_CONTRACT,
            arm_contract: T6_EVALUATOR_CONTRACT,
            budget_contract: READOUT_BUDGET_CONTRACT,
            metric_registry_contract: self.config.metric_registry.registry_contract,
            source_torsor_contract: TORSOR_CONTRACT,
            label_contract: PROTECTED_LABEL_CONTRACT,
        };
        self.records.push(record);
        Ok(self.records.last().expect("record was just pushed"))
    }

    /// Evaluate the torsor component of a sealed mixed-geometry case.
    pub fn evaluate_mixed(
        &mut self,
        case: &LabeledCase<MixedGeometryInput, MixedGeometryOracle>,
    ) -> Result<&T6EvalRecord, EvalError> {
        self.reserve_case(case.inference_input().split)?;
        let input = case.inference_input();
        let oracle = case.protected_label().reveal_for_evaluation();
        if input.generator_contract != MIXED_GEOMETRY_TASK_CONTRACT
            || oracle.generator_contract != MIXED_GEOMETRY_TASK_CONTRACT
            || input.task_family != TaskFamily::Mixed
            || case.label_contract() != PROTECTED_LABEL_CONTRACT
            || !is_canonical_mixed_case(input, oracle)
        {
            return Err(EvalError::ContractMismatch("mixed_case"));
        }
        let score = run_inference_callback(case, score_mixed_torsor).map_err(EvalError::Bridge)?;
        let record = T6EvalRecord {
            split: input.split,
            family: input.task_family,
            case_id: input.case_id,
            seed_block: oracle.pair_id,
            outcome: T6Outcome {
                score,
                matches_oracle: approximately_equal(score, oracle.expected_torsor_score),
            },
            canonical_digest: canonicalize_mixed_geometry_input(input).digest,
            envelope_contract: EVALUATOR_ENVELOPE_CONTRACT,
            arm_contract: T6_EVALUATOR_CONTRACT,
            budget_contract: READOUT_BUDGET_CONTRACT,
            metric_registry_contract: self.config.metric_registry.registry_contract,
            source_torsor_contract: TORSOR_CONTRACT,
            label_contract: PROTECTED_LABEL_CONTRACT,
        };
        self.records.push(record);
        Ok(self.records.last().expect("record was just pushed"))
    }

    fn reserve_case(&self, split: DataSplit) -> Result<(), EvalError> {
        if split != self.config.split {
            return Err(EvalError::SplitMismatch {
                expected: self.config.split,
                actual: split,
            });
        }
        if self.records.len() as u64 >= self.config.budget.max_cases {
            return Err(EvalError::CaseBudgetExceeded);
        }
        Ok(())
    }
}

/// Outcome retained for one C6 case.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct C6Outcome {
    pub score: f64,
    pub matches_oracle: bool,
}

/// Immutable non-final C6 evaluation record.
#[derive(Clone, Debug, PartialEq)]
pub struct C6EvalRecord {
    split: DataSplit,
    family: TaskFamily,
    case_id: u64,
    /// Retained seed-block / pair identity from the sealed oracle.
    seed_block: u64,
    outcome: C6Outcome,
    canonical_digest: String,
    envelope_contract: &'static str,
    arm_contract: &'static str,
    budget_contract: &'static str,
    metric_registry_contract: &'static str,
    source_chiral_contract: &'static str,
    label_contract: &'static str,
}

/// Bounded accumulator for one deterministic non-final C6 run.
#[derive(Clone, Debug, PartialEq)]
pub struct C6EvaluatorRun {
    config: EvaluatorConfig,
    records: Vec<C6EvalRecord>,
}

impl C6EvaluatorRun {
    /// Open a run only when every contract pin and budget is valid.
    pub fn open(config: EvaluatorConfig) -> Result<Self, EvalError> {
        config.metric_registry.admit_split(config.split)?;
        if config.envelope_contract != EVALUATOR_ENVELOPE_CONTRACT {
            return Err(EvalError::ContractMismatch("envelope_contract"));
        }
        if config.arm_contract != C6_EVALUATOR_CONTRACT {
            return Err(EvalError::ContractMismatch("arm_contract"));
        }
        if config.budget.contract != READOUT_BUDGET_CONTRACT {
            return Err(EvalError::ContractMismatch("budget_contract"));
        }
        if config.budget.max_cases == 0
            || config.budget.max_cases > MAX_CASES_PER_RUN
            || config.budget.max_readout_scalars_per_case < 2
            || config.budget.max_readout_scalars_per_case > MAX_READOUT_SCALARS_PER_CASE
            || config.budget.updates != NON_TRAINED_UPDATE_BUDGET
        {
            return Err(EvalError::InvalidBudget);
        }
        let sources = validate_source_contracts().map_err(EvalError::Bridge)?;
        if sources.chiral != CHIRAL_CONTRACT || sources.chiral != PINNED_SOURCE_CONTRACTS.chiral {
            return Err(EvalError::ContractMismatch("source_chiral_contract"));
        }
        Ok(Self {
            config,
            records: Vec::new(),
        })
    }

    /// Borrow emitted records in admission order.
    #[must_use]
    pub fn records(&self) -> &[C6EvalRecord] {
        &self.records
    }

    /// Evaluate a sealed chiral-favorable case via the TDI-24 chiral contract.
    pub fn evaluate_chiral_reflection(
        &mut self,
        case: &LabeledCase<ChiralReflectionInput, ChiralReflectionOracle>,
    ) -> Result<&C6EvalRecord, EvalError> {
        self.reserve_case(case.inference_input().split)?;
        let input = case.inference_input();
        let oracle = case.protected_label().reveal_for_evaluation();
        if case.inference_input().generator_contract != CHIRAL_REFLECTION_TASK_CONTRACT
            || oracle.generator_contract != CHIRAL_REFLECTION_TASK_CONTRACT
            || input.task_family != TaskFamily::ChiralFavorable
            || case.label_contract() != PROTECTED_LABEL_CONTRACT
            || !is_canonical_chiral_case(input, oracle)
        {
            return Err(EvalError::ContractMismatch("chiral_reflection_case"));
        }
        let score =
            run_inference_callback(case, score_chiral_reflection).map_err(EvalError::Bridge)?;
        let record = C6EvalRecord {
            split: input.split,
            family: input.task_family,
            case_id: input.case_id,
            seed_block: oracle.pair_id,
            outcome: C6Outcome {
                score,
                matches_oracle: approximately_equal(score, oracle.expected_score),
            },
            canonical_digest: canonicalize_chiral_reflection_input(input).digest,
            envelope_contract: EVALUATOR_ENVELOPE_CONTRACT,
            arm_contract: C6_EVALUATOR_CONTRACT,
            budget_contract: READOUT_BUDGET_CONTRACT,
            metric_registry_contract: self.config.metric_registry.registry_contract,
            source_chiral_contract: CHIRAL_CONTRACT,
            label_contract: PROTECTED_LABEL_CONTRACT,
        };
        self.records.push(record);
        Ok(self.records.last().expect("record was just pushed"))
    }

    /// Evaluate the chiral component of a sealed mixed-geometry case.
    pub fn evaluate_mixed(
        &mut self,
        case: &LabeledCase<MixedGeometryInput, MixedGeometryOracle>,
    ) -> Result<&C6EvalRecord, EvalError> {
        self.reserve_case(case.inference_input().split)?;
        let input = case.inference_input();
        let oracle = case.protected_label().reveal_for_evaluation();
        if input.generator_contract != MIXED_GEOMETRY_TASK_CONTRACT
            || oracle.generator_contract != MIXED_GEOMETRY_TASK_CONTRACT
            || input.task_family != TaskFamily::Mixed
            || case.label_contract() != PROTECTED_LABEL_CONTRACT
            || !is_canonical_mixed_case(input, oracle)
        {
            return Err(EvalError::ContractMismatch("mixed_case"));
        }
        let score = run_inference_callback(case, score_mixed_chiral).map_err(EvalError::Bridge)?;
        let record = C6EvalRecord {
            split: input.split,
            family: input.task_family,
            case_id: input.case_id,
            seed_block: oracle.pair_id,
            outcome: C6Outcome {
                score,
                matches_oracle: approximately_equal(score, oracle.expected_chiral_score),
            },
            canonical_digest: canonicalize_mixed_geometry_input(input).digest,
            envelope_contract: EVALUATOR_ENVELOPE_CONTRACT,
            arm_contract: C6_EVALUATOR_CONTRACT,
            budget_contract: READOUT_BUDGET_CONTRACT,
            metric_registry_contract: self.config.metric_registry.registry_contract,
            source_chiral_contract: CHIRAL_CONTRACT,
            label_contract: PROTECTED_LABEL_CONTRACT,
        };
        self.records.push(record);
        Ok(self.records.last().expect("record was just pushed"))
    }

    fn reserve_case(&self, split: DataSplit) -> Result<(), EvalError> {
        if split != self.config.split {
            return Err(EvalError::SplitMismatch {
                expected: self.config.split,
                actual: split,
            });
        }
        if self.records.len() as u64 >= self.config.budget.max_cases {
            return Err(EvalError::CaseBudgetExceeded);
        }
        Ok(())
    }
}

/// Outcome retained for one G6 case.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct G6Outcome {
    pub score: f64,
    pub matches_oracle: bool,
}

/// Immutable non-final G6 evaluation record.
#[derive(Clone, Debug, PartialEq)]
pub struct G6EvalRecord {
    split: DataSplit,
    family: TaskFamily,
    case_id: u64,
    /// Retained seed-block / pair identity from the sealed oracle.
    seed_block: u64,
    outcome: G6Outcome,
    canonical_digest: String,
    envelope_contract: &'static str,
    arm_contract: &'static str,
    budget_contract: &'static str,
    metric_registry_contract: &'static str,
    source_generic_contract: &'static str,
    label_contract: &'static str,
}

/// Bounded accumulator for one deterministic non-final G6 run.
#[derive(Clone, Debug, PartialEq)]
pub struct G6EvaluatorRun {
    config: EvaluatorConfig,
    records: Vec<G6EvalRecord>,
}

impl G6EvaluatorRun {
    /// Open a run only when every contract pin and budget is valid.
    pub fn open(config: EvaluatorConfig) -> Result<Self, EvalError> {
        config.metric_registry.admit_split(config.split)?;
        if config.envelope_contract != EVALUATOR_ENVELOPE_CONTRACT {
            return Err(EvalError::ContractMismatch("envelope_contract"));
        }
        if config.arm_contract != G6_EVALUATOR_CONTRACT {
            return Err(EvalError::ContractMismatch("arm_contract"));
        }
        if config.budget.contract != READOUT_BUDGET_CONTRACT {
            return Err(EvalError::ContractMismatch("budget_contract"));
        }
        if config.budget.max_cases == 0
            || config.budget.max_cases > MAX_CASES_PER_RUN
            || config.budget.max_readout_scalars_per_case < 2
            || config.budget.max_readout_scalars_per_case > MAX_READOUT_SCALARS_PER_CASE
            || config.budget.updates != NON_TRAINED_UPDATE_BUDGET
        {
            return Err(EvalError::InvalidBudget);
        }
        // Keep the shared T6/C6 source-pin health check, then pin G6's own
        // GENERIC6_CONTRACT. MixedGeometryInput exposes no Generic6 fields, so
        // this arm does not invent a mixed scoring path.
        let _sources = validate_source_contracts().map_err(EvalError::Bridge)?;
        if GENERIC6_CONTRACT != "tdi25-generic6-control-v1" {
            return Err(EvalError::ContractMismatch("source_generic_contract"));
        }
        Ok(Self {
            config,
            records: Vec::new(),
        })
    }

    /// Borrow emitted records in admission order.
    #[must_use]
    pub fn records(&self) -> &[G6EvalRecord] {
        &self.records
    }

    /// Evaluate a sealed neutral Generic6 control case via GENERIC6_CONTRACT.
    pub fn evaluate_neutral_control(
        &mut self,
        case: &LabeledCase<NeutralControlInput, NeutralControlOracle>,
    ) -> Result<&G6EvalRecord, EvalError> {
        self.reserve_case(case.inference_input().split)?;
        let input = case.inference_input();
        let oracle = case.protected_label().reveal_for_evaluation();
        if case.inference_input().generator_contract != NEUTRAL_CONTROL_TASK_CONTRACT
            || oracle.generator_contract != NEUTRAL_CONTROL_TASK_CONTRACT
            || input.task_family != TaskFamily::Neutral
            || case.label_contract() != PROTECTED_LABEL_CONTRACT
            || !is_canonical_neutral_case(input, oracle)
        {
            return Err(EvalError::ContractMismatch("neutral_control_case"));
        }
        let score =
            run_inference_callback(case, score_neutral_control).map_err(EvalError::Bridge)?;
        let record = G6EvalRecord {
            split: input.split,
            family: input.task_family,
            case_id: input.case_id,
            seed_block: oracle.pair_id,
            outcome: G6Outcome {
                score,
                matches_oracle: approximately_equal(score, oracle.expected_score),
            },
            canonical_digest: canonicalize_neutral_control_input(input).digest,
            envelope_contract: EVALUATOR_ENVELOPE_CONTRACT,
            arm_contract: G6_EVALUATOR_CONTRACT,
            budget_contract: READOUT_BUDGET_CONTRACT,
            metric_registry_contract: self.config.metric_registry.registry_contract,
            source_generic_contract: GENERIC6_CONTRACT,
            label_contract: PROTECTED_LABEL_CONTRACT,
        };
        self.records.push(record);
        Ok(self.records.last().expect("record was just pushed"))
    }

    fn reserve_case(&self, split: DataSplit) -> Result<(), EvalError> {
        if split != self.config.split {
            return Err(EvalError::SplitMismatch {
                expected: self.config.split,
                actual: split,
            });
        }
        if self.records.len() as u64 >= self.config.budget.max_cases {
            return Err(EvalError::CaseBudgetExceeded);
        }
        Ok(())
    }
}

fn is_canonical_torsor_case(input: &TorsorTransportInput, oracle: &TorsorTransportOracle) -> bool {
    let Ok(pair) = torsor_transport_pair_in_split(oracle.pair_id, input.split) else {
        return false;
    };
    oracle == &pair.oracle && (input == &pair.original || input == &pair.transported)
}

fn is_canonical_mixed_case(input: &MixedGeometryInput, oracle: &MixedGeometryOracle) -> bool {
    let Ok(pair) = mixed_geometry_pair_in_split(oracle.pair_id, input.split) else {
        return false;
    };
    (input == &pair.base && oracle == &pair.base_oracle)
        || (input == &pair.transformed && oracle == &pair.transformed_oracle)
}

fn is_canonical_chiral_case(
    input: &ChiralReflectionInput,
    oracle: &ChiralReflectionOracle,
) -> bool {
    let Ok(pair) = chiral_reflection_pair_in_split(oracle.pair_id, input.split) else {
        return false;
    };
    (input == &pair.right && oracle == &pair.right_oracle)
        || (input == &pair.left && oracle == &pair.left_oracle)
}

fn is_canonical_neutral_case(input: &NeutralControlInput, oracle: &NeutralControlOracle) -> bool {
    let Ok(pair) = neutral_control_pair_in_split(oracle.pair_id, input.split) else {
        return false;
    };
    (input == &pair.class_a && oracle == &pair.class_a_oracle)
        || (input == &pair.class_b && oracle == &pair.class_b_oracle)
}

/// Inference-only score callback for torsor-favorable inputs.
pub fn score_torsor_transport(input: &TorsorTransportInput) -> Result<f64, Tdi25Error> {
    torsor_arm_score(input.query, input.key, input.query_position)
}

/// Inference-only T6 score callback for mixed inputs.
pub fn score_mixed_torsor(input: &MixedGeometryInput) -> Result<f64, Tdi25Error> {
    torsor_arm_score(input.torsor_query, input.torsor_key, input.query_position)
}

/// Inference-only score callback for chiral-favorable inputs.
pub fn score_chiral_reflection(input: &ChiralReflectionInput) -> Result<f64, Tdi25Error> {
    chiral_arm_score(input.query, input.key, input.weights)
}

/// Inference-only C6 score callback for mixed inputs.
pub fn score_mixed_chiral(input: &MixedGeometryInput) -> Result<f64, Tdi25Error> {
    chiral_arm_score(input.chiral_query, input.chiral_key, input.weights)
}

/// Inference-only score callback for neutral Generic6 control inputs.
pub fn score_neutral_control(input: &NeutralControlInput) -> Result<f64, Tdi25Error> {
    generic_arm_score(input.query, input.key)
}

fn approximately_equal(left: f64, right: f64) -> bool {
    let scale = 1.0_f64.max(left.abs()).max(right.abs());
    (left - right).abs() <= 128.0 * f64::EPSILON * scale
}

/// Interval construction method retained on every Stage-C uncertainty summary.
///
/// Choice (documented, deterministic, no RNG):
/// - [`UncertaintyMethod::WilsonScore`] for single-arm accuracy rates when
///   callers provide independent revealed match bits within one seed block.
/// - [`UncertaintyMethod::BoundedHoeffdingPairedDifference`] for direct paired
///   T6/C6 differences in [-1, 1]. The finite-sample bound remains non-degenerate
///   at all-tie and all-win boundaries.
/// - The cluster-aware variants treat each seed block as one independent unit
///   (optionally further namespaced by family) and use observed cluster sizes
///   in the Hoeffding range term. This preserves related cases inside a seed
///   block as one cluster for later stratified synthesis.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UncertaintyMethod {
    /// Wilson score interval on an independent single-arm match-oracle rate.
    WilsonScore,
    /// Bounded Hoeffding interval on independent paired differences.
    BoundedHoeffdingPairedDifference,
    /// Cluster-aware Hoeffding interval on a Bernoulli mean.
    ClusterHoeffdingBernoulliMean,
    /// Cluster-aware Hoeffding interval on paired differences.
    ClusterHoeffdingPairedDifference,
}

impl UncertaintyMethod {
    /// Stable lowercase method id.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::WilsonScore => "wilson_score",
            Self::BoundedHoeffdingPairedDifference => "bounded_hoeffding_paired_difference",
            Self::ClusterHoeffdingBernoulliMean => "cluster_hoeffding_bernoulli_mean",
            Self::ClusterHoeffdingPairedDifference => "cluster_hoeffding_paired_difference",
        }
    }
}

/// Closed confidence interval with retained method and nominal level.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConfidenceInterval {
    pub lower: f64,
    pub upper: f64,
    pub confidence: f64,
    pub method: UncertaintyMethod,
}

impl ConfidenceInterval {
    fn checked(
        lower: f64,
        upper: f64,
        confidence: f64,
        method: UncertaintyMethod,
    ) -> Result<Self, EvalError> {
        if !lower.is_finite() || !upper.is_finite() || !confidence.is_finite() {
            return Err(EvalError::PairedUncertaintyInvalid {
                reason: "non_finite",
            });
        }
        if lower > upper {
            return Err(EvalError::PairedUncertaintyInvalid {
                reason: "non_finite",
            });
        }
        Ok(Self {
            lower,
            upper,
            confidence,
            method,
        })
    }
}

/// Already-revealed, family-tagged match bit admitted into the uncertainty surface.
///
/// Construct only from evaluator-retained `matches_oracle` flags (or synthetic
/// test vectors of the same shape). There is intentionally no constructor from
/// [`super::tdi25_tasks::ProtectedLabel`], raw sealed targets, or leak oracles
/// beyond what T6/C6/G6 eval records already retained. `seed_block` is the
/// paired seed-block / group identity used for stratified synthesis.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RevealedMatchOutcome {
    /// Sealed source arm; absent only for synthetic module-test vectors.
    arm: Option<ComparisonArm>,
    /// Evaluator-owned common target, never inferred from a family label.
    /// All existing v1 evaluator records have arm-specific targets only.
    shared_target_contract: Option<&'static str>,
    /// Non-final split retained with the revealed bit.
    pub split: DataSplit,
    /// Task family this outcome belongs to.
    pub family: TaskFamily,
    /// Paired seed-block / group identity retained from evaluator provenance.
    pub seed_block: u64,
    /// Case identity within the seed block.
    pub case_id: u64,
    /// Canonical case identity retained across evaluator arms. Kept private so
    /// callers cannot relabel record-derived evidence after revelation.
    canonical_digest: String,
    /// Whether the arm score matched the sealed oracle on the evaluation path.
    pub matches_oracle: bool,
    /// Private snapshot sealing every evidence-bearing field at construction.
    integrity: RevealedMatchOutcomeIntegrity,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RevealedMatchOutcomeIntegrity {
    arm: Option<ComparisonArm>,
    shared_target_contract: Option<&'static str>,
    split: DataSplit,
    family: TaskFamily,
    seed_block: u64,
    case_id: u64,
    canonical_digest: String,
    matches_oracle: bool,
}

impl RevealedMatchOutcomeIntegrity {
    fn new(
        arm: Option<ComparisonArm>,
        split: DataSplit,
        family: TaskFamily,
        seed_block: u64,
        case_id: u64,
        canonical_digest: &str,
        matches_oracle: bool,
    ) -> Self {
        Self {
            arm,
            shared_target_contract: None,
            split,
            family,
            seed_block,
            case_id,
            canonical_digest: canonical_digest.to_owned(),
            matches_oracle,
        }
    }

    fn matches(&self, outcome: &RevealedMatchOutcome) -> bool {
        self.arm == outcome.arm
            && self.shared_target_contract == outcome.shared_target_contract
            && self.split == outcome.split
            && self.family == outcome.family
            && self.seed_block == outcome.seed_block
            && self.case_id == outcome.case_id
            && self.canonical_digest == outcome.canonical_digest
            && self.matches_oracle == outcome.matches_oracle
    }
}

impl RevealedMatchOutcome {
    /// Build a synthetic match-oracle bit for module tests only.
    ///
    /// Production evidence must originate from retained evaluator records;
    /// this constructor is deliberately absent from non-test builds.
    #[cfg(test)]
    #[must_use]
    fn from_matches_oracle(
        split: DataSplit,
        family: TaskFamily,
        seed_block: u64,
        case_id: u64,
        matches_oracle: bool,
    ) -> Self {
        let canonical_digest = format!("synthetic:{seed_block}:{case_id}");
        let mut integrity = RevealedMatchOutcomeIntegrity::new(
            None,
            split,
            family,
            seed_block,
            case_id,
            &canonical_digest,
            matches_oracle,
        );
        integrity.shared_target_contract = Some("tdi25-unit-test-shared-target");
        Self {
            arm: None,
            shared_target_contract: integrity.shared_target_contract,
            split,
            family,
            seed_block,
            case_id,
            canonical_digest,
            matches_oracle,
            integrity,
        }
    }

    fn from_evaluator_record(
        arm: ComparisonArm,
        split: DataSplit,
        family: TaskFamily,
        seed_block: u64,
        case_id: u64,
        canonical_digest: String,
        matches_oracle: bool,
    ) -> Self {
        let integrity = RevealedMatchOutcomeIntegrity::new(
            Some(arm),
            split,
            family,
            seed_block,
            case_id,
            &canonical_digest,
            matches_oracle,
        );
        Self {
            arm: Some(arm),
            // V1 record correctness is relative to an arm-specific oracle.
            // A future matrix entry cannot retroactively requalify it.
            shared_target_contract: None,
            split,
            family,
            seed_block,
            case_id,
            canonical_digest,
            matches_oracle,
            integrity,
        }
    }
}

/// Map a task family onto its frozen primary family metric id.
#[must_use]
pub const fn primary_family_metric_for(family: TaskFamily) -> PrimaryFamilyMetricId {
    match family {
        TaskFamily::TorsorFavorable => PrimaryFamilyMetricId::TorsorFavorable,
        TaskFamily::ChiralFavorable => PrimaryFamilyMetricId::ChiralFavorable,
        TaskFamily::Mixed => PrimaryFamilyMetricId::Mixed,
        TaskFamily::Neutral => PrimaryFamilyMetricId::Neutral,
    }
}

/// Paired T6/C6 effect summary for one family seed block, bound to the frozen registry.
///
/// Experimental and non-final only; never authorises protected/final evaluation
/// or scientific claims. G6 is intentionally absent from this primary surface.
#[derive(Clone, Debug, PartialEq)]
pub struct PairedEffectSummary {
    pub split: DataSplit,
    pub family: TaskFamily,
    /// Canonical first block, retained for single-block API compatibility.
    pub seed_block: u64,
    /// Complete sorted block provenance, bound into the integrity snapshot.
    /// Multi-block summaries use cluster-aware CIs.
    pub seed_blocks: Vec<u64>,
    pub n_pairs: u64,
    pub t6_accuracy: f64,
    pub c6_accuracy: f64,
    /// Mean of per-pair differences `I(C6)-I(T6)`; equals `c6_accuracy - t6_accuracy`.
    pub paired_difference_mean: f64,
    pub paired_difference_ci: ConfidenceInterval,
    pub t6_accuracy_ci: ConfidenceInterval,
    pub c6_accuracy_ci: ConfidenceInterval,
    pub primary_family_metric: PrimaryFamilyMetricId,
    pub cross_family_summary: CrossFamilySummaryMetricId,
    pub secondary_paired_outcome_difference: SecondaryDiagnosticId,
    pub uncertainty_contract: &'static str,
    pub metric_registry_contract: &'static str,
    pub experimental_non_final: bool,
    /// Private snapshot of every evidence-bearing field at construction.
    ///
    /// Public fields remain readable for reporting, but a cloned summary cannot
    /// be mutated into stronger evidence and then accepted by synthesis.
    integrity: Option<PairedEffectSummaryIntegrity>,
}

#[derive(Clone, Debug, PartialEq)]
struct PairedEffectSummaryIntegrity {
    split: DataSplit,
    family: TaskFamily,
    seed_block: u64,
    seed_blocks: Vec<u64>,
    n_pairs: u64,
    t6_accuracy: f64,
    c6_accuracy: f64,
    paired_difference_mean: f64,
    paired_difference_ci: ConfidenceInterval,
    t6_accuracy_ci: ConfidenceInterval,
    c6_accuracy_ci: ConfidenceInterval,
    primary_family_metric: PrimaryFamilyMetricId,
    cross_family_summary: CrossFamilySummaryMetricId,
    secondary_paired_outcome_difference: SecondaryDiagnosticId,
    uncertainty_contract: &'static str,
    metric_registry_contract: &'static str,
    experimental_non_final: bool,
}

impl PairedEffectSummaryIntegrity {
    fn from_summary(summary: &PairedEffectSummary) -> Self {
        Self {
            split: summary.split,
            family: summary.family,
            seed_block: summary.seed_block,
            seed_blocks: summary.seed_blocks.clone(),
            n_pairs: summary.n_pairs,
            t6_accuracy: summary.t6_accuracy,
            c6_accuracy: summary.c6_accuracy,
            paired_difference_mean: summary.paired_difference_mean,
            paired_difference_ci: summary.paired_difference_ci,
            t6_accuracy_ci: summary.t6_accuracy_ci,
            c6_accuracy_ci: summary.c6_accuracy_ci,
            primary_family_metric: summary.primary_family_metric,
            cross_family_summary: summary.cross_family_summary,
            secondary_paired_outcome_difference: summary.secondary_paired_outcome_difference,
            uncertainty_contract: summary.uncertainty_contract,
            metric_registry_contract: summary.metric_registry_contract,
            experimental_non_final: summary.experimental_non_final,
        }
    }
}

/// Optional G6 attribution control retained as a secondary only.
///
/// Reports Neutral-family G6 accuracy/CIs by seed block under
/// [`SecondaryDiagnosticId::G6AttributionContrast`]. Reachable from real
/// [`G6EvaluatorRun`] outputs. Never elevates G6 to a primary T6/C6 hypothesis.
#[derive(Clone, Debug, PartialEq)]
pub struct G6AttributionContrastSummary {
    pub split: DataSplit,
    pub family: TaskFamily,
    pub seed_block: u64,
    pub n_cases: u64,
    pub g6_accuracy: f64,
    pub g6_accuracy_ci: ConfidenceInterval,
    pub secondary_g6_attribution_contrast: SecondaryDiagnosticId,
    pub uncertainty_contract: &'static str,
    pub metric_registry_contract: &'static str,
    pub experimental_non_final: bool,
}

/// Wilson score interval for `successes` in `n` Bernoulli trials at 95%.
fn wilson_score_interval(successes: u64, n: u64) -> Result<ConfidenceInterval, EvalError> {
    if n == 0 {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "empty_pairs",
        });
    }
    let n_f = n as f64;
    let p = successes as f64 / n_f;
    let z = NORMAL_CRITICAL_Z_95;
    let z2 = z * z;
    let denom = 1.0 + z2 / n_f;
    if !denom.is_finite() || denom == 0.0 {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "non_finite",
        });
    }
    let center = (p + z2 / (2.0 * n_f)) / denom;
    let inner = p * (1.0 - p) / n_f + z2 / (4.0 * n_f * n_f);
    if !inner.is_finite() || inner < 0.0 {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "non_finite",
        });
    }
    let margin = z * inner.sqrt() / denom;
    ConfidenceInterval::checked(
        center - margin,
        center + margin,
        PAIRED_UNCERTAINTY_LEVEL,
        UncertaintyMethod::WilsonScore,
    )
}

/// Distribution-free Hoeffding interval on a bounded mean.
fn bounded_hoeffding_mean_interval(
    mean: f64,
    n: usize,
    sum_squared_cluster_sizes: u64,
    lower_bound: f64,
    upper_bound: f64,
    method: UncertaintyMethod,
) -> Result<ConfidenceInterval, EvalError> {
    if n < 2 {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "insufficient_pairs",
        });
    }
    if !mean.is_finite() || lower_bound >= upper_bound {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "non_finite",
        });
    }
    let n_u64 = n as u64;
    if sum_squared_cluster_sizes < n_u64 || sum_squared_cluster_sizes > n_u64.saturating_mul(n_u64)
    {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "invalid_cluster_sizes",
        });
    }
    let n_f = n as f64;
    let range_width = upper_bound - lower_bound;
    let margin = range_width
        * ((sum_squared_cluster_sizes as f64 * HOEFFDING_LOG_40) / (2.0 * n_f * n_f)).sqrt();
    if !margin.is_finite() {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "non_finite",
        });
    }
    ConfidenceInterval::checked(
        (mean - margin).max(lower_bound),
        (mean + margin).min(upper_bound),
        PAIRED_UNCERTAINTY_LEVEL,
        method,
    )
}

/// Bounded finite-sample CI on the mean of paired outcome differences.
fn bounded_hoeffding_paired_difference_ci(
    differences: &[f64],
    sum_squared_cluster_sizes: u64,
    cluster_aware: bool,
) -> Result<(f64, ConfidenceInterval), EvalError> {
    if differences.is_empty() {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "empty_pairs",
        });
    }
    if differences.len() < 2 {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "insufficient_pairs",
        });
    }
    let n_f = differences.len() as f64;
    let mean = differences.iter().sum::<f64>() / n_f;
    let method = if cluster_aware {
        UncertaintyMethod::ClusterHoeffdingPairedDifference
    } else {
        UncertaintyMethod::BoundedHoeffdingPairedDifference
    };
    let ci = bounded_hoeffding_mean_interval(
        mean,
        differences.len(),
        sum_squared_cluster_sizes,
        -1.0,
        1.0,
        method,
    )?;
    Ok((mean, ci))
}

/// Bind a registry to the frozen Stage-C pin required by the uncertainty engine.
fn require_pinned_metric_registry(registry: &MetricRegistry) -> Result<(), EvalError> {
    validate_metric_registry(registry)?;
    if *registry != MetricRegistry::pinned() {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "registry_not_pinned",
        });
    }
    if !registry
        .secondaries
        .contains(&SecondaryDiagnosticId::PairedOutcomeDifference)
    {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "secondary_paired_outcome_missing",
        });
    }
    if !registry
        .secondaries
        .contains(&SecondaryDiagnosticId::G6AttributionContrast)
    {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "secondary_g6_attribution_missing",
        });
    }
    Ok(())
}

fn require_revealed_match_outcome_integrity(
    outcome: &RevealedMatchOutcome,
) -> Result<(), EvalError> {
    if !outcome.integrity.matches(outcome) {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "outcome_integrity_mismatch",
        });
    }
    Ok(())
}

fn require_revealed_outcome_arm(
    expected: ComparisonArm,
    outcomes: &[RevealedMatchOutcome],
) -> Result<(), EvalError> {
    for outcome in outcomes {
        if outcome.arm == Some(expected) {
            continue;
        }
        #[cfg(test)]
        if outcome.arm.is_none() {
            continue;
        }
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "arm_mismatch",
        });
    }
    Ok(())
}

fn require_split_family_seed_block_outcomes(
    split: DataSplit,
    family: TaskFamily,
    seed_block: u64,
    outcomes: &[RevealedMatchOutcome],
) -> Result<(), EvalError> {
    for outcome in outcomes {
        require_revealed_match_outcome_integrity(outcome)?;
        if outcome.split != split {
            return Err(EvalError::PairedUncertaintyInvalid {
                reason: "split_mismatch",
            });
        }
        if outcome.family != family {
            return Err(EvalError::PairedUncertaintyInvalid {
                reason: "family_mismatch",
            });
        }
        if outcome.seed_block != seed_block {
            return Err(EvalError::PairedUncertaintyInvalid {
                reason: "seed_block_mismatch",
            });
        }
    }
    Ok(())
}

fn require_split_family_outcomes(
    split: DataSplit,
    family: TaskFamily,
    outcomes: &[RevealedMatchOutcome],
) -> Result<(), EvalError> {
    for outcome in outcomes {
        require_revealed_match_outcome_integrity(outcome)?;
        if outcome.split != split {
            return Err(EvalError::PairedUncertaintyInvalid {
                reason: "split_mismatch",
            });
        }
        if outcome.family != family {
            return Err(EvalError::PairedUncertaintyInvalid {
                reason: "family_mismatch",
            });
        }
    }
    Ok(())
}

fn require_pair_identity_alignment(
    left: &[RevealedMatchOutcome],
    right: &[RevealedMatchOutcome],
) -> Result<(), EvalError> {
    if left.len() != right.len() {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "length_mismatch",
        });
    }
    for (index, (a, b)) in left.iter().zip(right.iter()).enumerate() {
        require_revealed_match_outcome_integrity(a)?;
        require_revealed_match_outcome_integrity(b)?;
        if a.split != b.split
            || a.family != b.family
            || a.seed_block != b.seed_block
            || a.case_id != b.case_id
            || a.canonical_digest != b.canonical_digest
        {
            return Err(EvalError::PairedUncertaintyInvalid {
                reason: "pair_identity_mismatch",
            });
        }
        if left[..index].iter().any(|prior| {
            prior.split == a.split
                && prior.family == a.family
                && prior.seed_block == a.seed_block
                && prior.case_id == a.case_id
        }) {
            return Err(EvalError::PairedUncertaintyInvalid {
                reason: "duplicate_pair_identity",
            });
        }
    }
    Ok(())
}

/// Admit only a common target retained by both outcome constructors.
/// The capability matrix is necessary but cannot upgrade legacy evidence.
/// Synthetic targets exist only in private module-test constructors.
fn require_shared_paired_target(
    family: TaskFamily,
    t6_matches: &[RevealedMatchOutcome],
    c6_matches: &[RevealedMatchOutcome],
) -> Result<(), EvalError> {
    for (t6, c6) in t6_matches.iter().zip(c6_matches) {
        let (Some(t6_target), Some(c6_target)) =
            (t6.shared_target_contract, c6.shared_target_contract)
        else {
            return Err(EvalError::PairedUncertaintyInvalid {
                reason: "missing_common_target_contract",
            });
        };
        if t6_target != c6_target {
            return Err(EvalError::PairedUncertaintyInvalid {
                reason: "common_target_contract_mismatch",
            });
        }
        if (t6.arm.is_some() || c6.arm.is_some())
            && matched_family_paths(family).shared_target_contract() != Some(t6_target)
        {
            return Err(EvalError::PairedUncertaintyInvalid {
                reason: "unregistered_common_target_contract",
            });
        }
    }
    Ok(())
}

/// Return sum_g n_g^2 for seed-block clustering, namespaced by family.
#[allow(dead_code)]
fn sum_squared_seed_block_sizes(outcomes: &[RevealedMatchOutcome]) -> Result<u64, EvalError> {
    let mut groups: Vec<(TaskFamily, u64, u64)> = Vec::with_capacity(outcomes.len());
    for outcome in outcomes {
        if let Some((_, _, size)) = groups.iter_mut().find(|(family, seed_block, _)| {
            *family == outcome.family && *seed_block == outcome.seed_block
        }) {
            *size += 1;
        } else {
            groups.push((outcome.family, outcome.seed_block, 1));
        }
    }
    groups.into_iter().try_fold(0_u64, |sum, (_, _, size)| {
        size.checked_mul(size)
            .and_then(|square| sum.checked_add(square))
            .ok_or(EvalError::PairedUncertaintyInvalid {
                reason: "invalid_cluster_sizes",
            })
    })
}

fn summarize_paired_uncertainty_inner(
    split: DataSplit,
    family: TaskFamily,
    seed_block: u64,
    t6_matches: &[RevealedMatchOutcome],
    c6_matches: &[RevealedMatchOutcome],
    registry: &MetricRegistry,
    cluster_sum_squares: Option<u64>,
) -> Result<PairedEffectSummary, EvalError> {
    validate_non_final_split(split)?;
    require_pinned_metric_registry(registry)?;
    if t6_matches.is_empty() || c6_matches.is_empty() {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "empty_pairs",
        });
    }
    let cluster_aware = cluster_sum_squares.is_some();
    if cluster_aware {
        require_split_family_outcomes(split, family, t6_matches)?;
        require_split_family_outcomes(split, family, c6_matches)?;
    } else {
        require_split_family_seed_block_outcomes(split, family, seed_block, t6_matches)?;
        require_split_family_seed_block_outcomes(split, family, seed_block, c6_matches)?;
    }
    require_revealed_outcome_arm(ComparisonArm::T6, t6_matches)?;
    require_revealed_outcome_arm(ComparisonArm::C6, c6_matches)?;
    require_pair_identity_alignment(t6_matches, c6_matches)?;
    require_shared_paired_target(family, t6_matches, c6_matches)?;

    let mut seed_blocks = Vec::new();
    let mut seed_block_sizes: Vec<(u64, u64)> = Vec::new();
    for outcome in t6_matches {
        if !seed_blocks.contains(&outcome.seed_block) {
            seed_blocks.push(outcome.seed_block);
        }
        if let Some((_, size)) = seed_block_sizes
            .iter_mut()
            .find(|(seed_block, _)| *seed_block == outcome.seed_block)
        {
            *size += 1;
        } else {
            seed_block_sizes.push((outcome.seed_block, 1));
        }
    }
    seed_blocks.sort_unstable();
    if seed_blocks.first().copied() != Some(seed_block)
        || (!cluster_aware && seed_blocks.len() != 1)
    {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "seed_block_mismatch",
        });
    }
    if seed_blocks.len() > MAX_SEED_BLOCKS_PER_SYNTHESIS {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "too_many_seed_blocks",
        });
    }
    if seed_block_sizes
        .iter()
        .any(|(_, size)| *size > MAX_CASES_PER_RUN)
    {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "too_many_pairs",
        });
    }

    if t6_matches.len() < 2 {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "insufficient_pairs",
        });
    }
    let max_pairs = (MAX_CASES_PER_RUN as usize)
        .checked_mul(seed_blocks.len())
        .ok_or(EvalError::PairedUncertaintyInvalid {
            reason: "too_many_pairs",
        })?;
    if t6_matches.len() > max_pairs {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "too_many_pairs",
        });
    }

    let n = t6_matches.len();
    let mut t6_successes = 0_u64;
    let mut c6_successes = 0_u64;
    let mut differences = Vec::with_capacity(n);
    for (t6, c6) in t6_matches.iter().zip(c6_matches.iter()) {
        let t = u64::from(t6.matches_oracle);
        let c = u64::from(c6.matches_oracle);
        t6_successes += t;
        c6_successes += c;
        differences.push(c as f64 - t as f64);
    }

    let n_f = n as f64;
    let t6_accuracy = t6_successes as f64 / n_f;
    let c6_accuracy = c6_successes as f64 / n_f;
    if !t6_accuracy.is_finite() || !c6_accuracy.is_finite() {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "non_finite",
        });
    }

    let sum_squared_cluster_sizes = cluster_sum_squares.unwrap_or(n as u64);
    let (t6_accuracy_ci, c6_accuracy_ci) = if cluster_aware {
        (
            bounded_hoeffding_mean_interval(
                t6_accuracy,
                n,
                sum_squared_cluster_sizes,
                0.0,
                1.0,
                UncertaintyMethod::ClusterHoeffdingBernoulliMean,
            )?,
            bounded_hoeffding_mean_interval(
                c6_accuracy,
                n,
                sum_squared_cluster_sizes,
                0.0,
                1.0,
                UncertaintyMethod::ClusterHoeffdingBernoulliMean,
            )?,
        )
    } else {
        (
            wilson_score_interval(t6_successes, n as u64)?,
            wilson_score_interval(c6_successes, n as u64)?,
        )
    };
    let (paired_difference_mean, paired_difference_ci) = bounded_hoeffding_paired_difference_ci(
        &differences,
        sum_squared_cluster_sizes,
        cluster_aware,
    )?;

    let expected_gap = c6_accuracy - t6_accuracy;
    if !paired_difference_mean.is_finite() || (paired_difference_mean - expected_gap).abs() > 1e-12
    {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "non_finite",
        });
    }

    let mut summary = PairedEffectSummary {
        split,
        family,
        seed_block,
        seed_blocks,
        n_pairs: n as u64,
        t6_accuracy,
        c6_accuracy,
        paired_difference_mean,
        paired_difference_ci,
        t6_accuracy_ci,
        c6_accuracy_ci,
        primary_family_metric: primary_family_metric_for(family),
        cross_family_summary: registry.cross_family_summary,
        secondary_paired_outcome_difference: SecondaryDiagnosticId::PairedOutcomeDifference,
        uncertainty_contract: PAIRED_UNCERTAINTY_CONTRACT,
        metric_registry_contract: registry.registry_contract,
        experimental_non_final: true,
        integrity: None,
    };
    summary.integrity = Some(PairedEffectSummaryIntegrity::from_summary(&summary));
    Ok(summary)
}

/// Summarise paired T6/C6 match outcomes for one family seed block.
///
/// Primary hypothesis surface only (T6 vs C6). Consumes already-revealed
/// [`RevealedMatchOutcome`] bits. Rejects empty pairs, length/identity mismatch,
/// family or seed-block drift, non-finite statistics, registry pin drift, and
/// protected/final splits. Deterministic; no RNG.
pub fn summarize_paired_uncertainty_by_seed_block(
    split: DataSplit,
    family: TaskFamily,
    seed_block: u64,
    t6_matches: &[RevealedMatchOutcome],
    c6_matches: &[RevealedMatchOutcome],
    registry: &MetricRegistry,
) -> Result<PairedEffectSummary, EvalError> {
    summarize_paired_uncertainty_inner(
        split, family, seed_block, t6_matches, c6_matches, registry, None,
    )
}

fn summarize_g6_control_inner(
    split: DataSplit,
    seed_block: u64,
    g6_matches: &[RevealedMatchOutcome],
    registry: &MetricRegistry,
) -> Result<G6AttributionContrastSummary, EvalError> {
    validate_non_final_split(split)?;
    require_pinned_metric_registry(registry)?;
    if g6_matches.is_empty() {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "empty_pairs",
        });
    }
    // Real G6 evaluator emits Neutral only; bind the secondary control to that family.
    require_split_family_seed_block_outcomes(split, TaskFamily::Neutral, seed_block, g6_matches)?;
    require_revealed_outcome_arm(ComparisonArm::G6, g6_matches)?;
    if g6_matches.len() < 2 {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "insufficient_pairs",
        });
    }
    if g6_matches.len() > MAX_CASES_PER_RUN as usize {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "too_many_pairs",
        });
    }
    let n = g6_matches.len();
    let g6_successes = g6_matches
        .iter()
        .map(|outcome| u64::from(outcome.matches_oracle))
        .sum::<u64>();
    let g6_accuracy = g6_successes as f64 / n as f64;
    if !g6_accuracy.is_finite() {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "non_finite",
        });
    }
    let g6_accuracy_ci = wilson_score_interval(g6_successes, n as u64)?;
    Ok(G6AttributionContrastSummary {
        split,
        family: TaskFamily::Neutral,
        seed_block,
        n_cases: n as u64,
        g6_accuracy,
        g6_accuracy_ci,
        secondary_g6_attribution_contrast: SecondaryDiagnosticId::G6AttributionContrast,
        uncertainty_contract: PAIRED_UNCERTAINTY_CONTRACT,
        metric_registry_contract: registry.registry_contract,
        experimental_non_final: true,
    })
}

/// Summarise Neutral-family G6 attribution control accuracy for one seed block.
///
/// Secondary control only — reachable from real [`G6EvaluatorRun`] Neutral
/// records. Does not pair against T6/C6 (those arms do not emit Neutral) and
/// never promotes G6 to a primary hypothesis.
pub fn summarize_g6_attribution_contrast_by_seed_block(
    split: DataSplit,
    seed_block: u64,
    g6_matches: &[RevealedMatchOutcome],
    registry: &MetricRegistry,
) -> Result<G6AttributionContrastSummary, EvalError> {
    summarize_g6_control_inner(split, seed_block, g6_matches, registry)
}

/// Extract revealed match bits from T6 evaluator records for one split.
///
/// Seed-block identity is taken from retained opaque [`T6EvalRecord`] values
/// (oracle `pair_id`). Does not accept [`super::tdi25_tasks::ProtectedLabel`].
pub fn revealed_matches_from_t6_records(
    records: &[T6EvalRecord],
    expected_split: DataSplit,
) -> Result<Vec<RevealedMatchOutcome>, EvalError> {
    validate_non_final_split(expected_split)?;
    if records.is_empty() {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "empty_pairs",
        });
    }
    if records.len() > MAX_CASES_PER_RUN as usize {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "too_many_pairs",
        });
    }
    let mut out = Vec::with_capacity(records.len());
    for record in records {
        if record.split != expected_split {
            return Err(EvalError::SplitMismatch {
                expected: expected_split,
                actual: record.split,
            });
        }
        if record.metric_registry_contract != METRIC_REGISTRY_CONTRACT {
            return Err(EvalError::ContractMismatch("metric_registry_contract"));
        }
        if record.envelope_contract != EVALUATOR_ENVELOPE_CONTRACT {
            return Err(EvalError::ContractMismatch("envelope_contract"));
        }
        if record.arm_contract != T6_EVALUATOR_CONTRACT {
            return Err(EvalError::ContractMismatch("arm_contract"));
        }
        if record.budget_contract != READOUT_BUDGET_CONTRACT {
            return Err(EvalError::ContractMismatch("budget_contract"));
        }
        if record.source_torsor_contract != TORSOR_CONTRACT {
            return Err(EvalError::ContractMismatch("source_torsor_contract"));
        }
        if record.label_contract != PROTECTED_LABEL_CONTRACT {
            return Err(EvalError::ContractMismatch("label_contract"));
        }
        out.push(RevealedMatchOutcome::from_evaluator_record(
            ComparisonArm::T6,
            record.split,
            record.family,
            record.seed_block,
            record.case_id,
            record.canonical_digest.clone(),
            record.outcome.matches_oracle,
        ));
    }
    Ok(out)
}

/// Extract revealed match bits from C6 evaluator records for one split.
pub fn revealed_matches_from_c6_records(
    records: &[C6EvalRecord],
    expected_split: DataSplit,
) -> Result<Vec<RevealedMatchOutcome>, EvalError> {
    validate_non_final_split(expected_split)?;
    if records.is_empty() {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "empty_pairs",
        });
    }
    if records.len() > MAX_CASES_PER_RUN as usize {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "too_many_pairs",
        });
    }
    let mut out = Vec::with_capacity(records.len());
    for record in records {
        if record.split != expected_split {
            return Err(EvalError::SplitMismatch {
                expected: expected_split,
                actual: record.split,
            });
        }
        if record.metric_registry_contract != METRIC_REGISTRY_CONTRACT {
            return Err(EvalError::ContractMismatch("metric_registry_contract"));
        }
        if record.envelope_contract != EVALUATOR_ENVELOPE_CONTRACT {
            return Err(EvalError::ContractMismatch("envelope_contract"));
        }
        if record.arm_contract != C6_EVALUATOR_CONTRACT {
            return Err(EvalError::ContractMismatch("arm_contract"));
        }
        if record.budget_contract != READOUT_BUDGET_CONTRACT {
            return Err(EvalError::ContractMismatch("budget_contract"));
        }
        if record.source_chiral_contract != CHIRAL_CONTRACT {
            return Err(EvalError::ContractMismatch("source_chiral_contract"));
        }
        if record.label_contract != PROTECTED_LABEL_CONTRACT {
            return Err(EvalError::ContractMismatch("label_contract"));
        }
        out.push(RevealedMatchOutcome::from_evaluator_record(
            ComparisonArm::C6,
            record.split,
            record.family,
            record.seed_block,
            record.case_id,
            record.canonical_digest.clone(),
            record.outcome.matches_oracle,
        ));
    }
    Ok(out)
}

/// Extract revealed match bits from G6 evaluator records for one split.
pub fn revealed_matches_from_g6_records(
    records: &[G6EvalRecord],
    expected_split: DataSplit,
) -> Result<Vec<RevealedMatchOutcome>, EvalError> {
    validate_non_final_split(expected_split)?;
    if records.is_empty() {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "empty_pairs",
        });
    }
    if records.len() > MAX_CASES_PER_RUN as usize {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "too_many_pairs",
        });
    }
    let mut out = Vec::with_capacity(records.len());
    for record in records {
        if record.split != expected_split {
            return Err(EvalError::SplitMismatch {
                expected: expected_split,
                actual: record.split,
            });
        }
        if record.metric_registry_contract != METRIC_REGISTRY_CONTRACT {
            return Err(EvalError::ContractMismatch("metric_registry_contract"));
        }
        if record.envelope_contract != EVALUATOR_ENVELOPE_CONTRACT {
            return Err(EvalError::ContractMismatch("envelope_contract"));
        }
        if record.arm_contract != G6_EVALUATOR_CONTRACT {
            return Err(EvalError::ContractMismatch("arm_contract"));
        }
        if record.budget_contract != READOUT_BUDGET_CONTRACT {
            return Err(EvalError::ContractMismatch("budget_contract"));
        }
        if record.source_generic_contract != GENERIC6_CONTRACT {
            return Err(EvalError::ContractMismatch("source_generic_contract"));
        }
        if record.label_contract != PROTECTED_LABEL_CONTRACT {
            return Err(EvalError::ContractMismatch("label_contract"));
        }
        out.push(RevealedMatchOutcome::from_evaluator_record(
            ComparisonArm::G6,
            record.split,
            record.family,
            record.seed_block,
            record.case_id,
            record.canonical_digest.clone(),
            record.outcome.matches_oracle,
        ));
    }
    Ok(out)
}

/// Summarise paired T6/C6 records for one retained family seed block.
///
/// Records must be equal-length and identity-aligned: matching `family`,
/// `case_id`, `seed_block`, and `canonical_digest`. Seed-block identity is
/// taken from retained evaluator provenance (`oracle.pair_id`), not caller
/// relabeling. Uses within-block independent Wilson / Hoeffding intervals
/// (informative at finite n). Cluster-aware multi-block synthesis is deferred
/// to slice 28 so a single-block summary never substitutes `n²` into the
/// Hoeffding margin. Existing Mixed T6/C6 records share input identities, but
/// score different arm-specific targets. They are rejected with
/// `missing_common_target_contract`, not admitted as primary evidence.
/// A later versioned common-target evaluator is required for every family.
pub fn summarize_paired_uncertainty_from_records(
    split: DataSplit,
    family: TaskFamily,
    seed_block: u64,
    t6_records: &[T6EvalRecord],
    c6_records: &[C6EvalRecord],
    registry: &MetricRegistry,
) -> Result<PairedEffectSummary, EvalError> {
    if t6_records.len() != c6_records.len() {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "length_mismatch",
        });
    }
    if t6_records.len() > MAX_CASES_PER_RUN as usize {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "too_many_pairs",
        });
    }
    for (index, (left, right)) in t6_records.iter().zip(c6_records.iter()).enumerate() {
        if left.family != family || right.family != family {
            return Err(EvalError::PairedUncertaintyInvalid {
                reason: "family_mismatch",
            });
        }
        if left.seed_block != seed_block || right.seed_block != seed_block {
            return Err(EvalError::PairedUncertaintyInvalid {
                reason: "seed_block_mismatch",
            });
        }
        if left.family != right.family
            || left.case_id != right.case_id
            || left.seed_block != right.seed_block
            || left.canonical_digest != right.canonical_digest
        {
            return Err(EvalError::PairedUncertaintyInvalid {
                reason: "pair_identity_mismatch",
            });
        }
        if t6_records[..index].iter().any(|prior| {
            prior.family == left.family
                && prior.case_id == left.case_id
                && prior.seed_block == left.seed_block
                && prior.canonical_digest == left.canonical_digest
        }) {
            return Err(EvalError::PairedUncertaintyInvalid {
                reason: "duplicate_pair_identity",
            });
        }
    }
    let t6 = revealed_matches_from_t6_records(t6_records, split)?;
    let c6 = revealed_matches_from_c6_records(c6_records, split)?;
    // Independent within-block intervals — never cluster a solitary seed block as n².
    summarize_paired_uncertainty_inner(split, family, seed_block, &t6, &c6, registry, None)
}

/// Signed effect class retained on family-stratified synthesis reports.
///
/// Classification uses the retained paired-difference CI:
/// - [`SignedEffectClass::Positive`] when the interval lies entirely above zero
/// - [`SignedEffectClass::Harmful`] when the interval lies entirely below zero
/// - [`SignedEffectClass::Inconclusive`] when the interval contains zero
/// - [`SignedEffectClass::Null`] reserved for a future explicit equivalence/null test
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SignedEffectClass {
    /// Paired difference favors C6 (CI entirely above zero).
    Positive,
    /// Explicit null/equivalence result from a dedicated test (not emitted by this classifier).
    Null,
    /// Paired difference favors T6 / harms a C6-favoring claim (CI entirely below zero).
    Harmful,
    /// Finite but not classifiable under the closed Stage-C rules.
    Inconclusive,
}

impl SignedEffectClass {
    /// Stable lowercase class id.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Positive => "positive",
            Self::Null => "null",
            Self::Harmful => "harmful",
            Self::Inconclusive => "inconclusive",
        }
    }
}

/// Algebraic sign of a paired difference mean, used for reversal detection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EffectSign {
    /// Strictly positive mean (`C6 - T6 > 0`).
    Positive,
    /// Strictly negative mean (`C6 - T6 < 0`).
    Negative,
    /// Exact zero mean.
    Zero,
}

impl EffectSign {
    /// Stable lowercase sign id.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Positive => "positive",
            Self::Negative => "negative",
            Self::Zero => "zero",
        }
    }
}

/// Per-family signed effect retained by stratified synthesis.
#[derive(Clone, Debug, PartialEq)]
pub struct FamilySignedEffect {
    pub family: TaskFamily,
    pub seed_block: u64,
    pub seed_blocks: Vec<u64>,
    pub n_pairs: u64,
    pub paired_difference_mean: f64,
    pub paired_difference_ci: ConfidenceInterval,
    pub effect_sign: EffectSign,
    pub outcome_class: SignedEffectClass,
    pub primary_family_metric: PrimaryFamilyMetricId,
}

/// Optional pooled summary admitted only when no family sign reversal is present.
#[derive(Clone, Debug, PartialEq)]
pub struct PooledSynthesisSummary {
    pub n_pairs: u64,
    pub paired_difference_mean: f64,
    pub effect_sign: EffectSign,
    pub outcome_class: SignedEffectClass,
    pub cross_family_summary: CrossFamilySummaryMetricId,
}

/// Family-stratified synthesis report. Pooled wins cannot hide family reversals.
#[derive(Clone, Debug, PartialEq)]
pub struct FamilyStratifiedSynthesisReport {
    pub split: DataSplit,
    /// Cluster-aware aggregate effect for each required task family.
    pub family_effects: Vec<FamilySignedEffect>,
    /// Individually classified `(family, seed_block)` effects retained before
    /// any cluster-aware family aggregation.
    pub seed_block_effects: Vec<FamilySignedEffect>,
    /// True when at least one family is Positive and another is Negative.
    pub sign_reversal_present: bool,
    /// True when one family contains both positive and negative seed blocks.
    pub seed_block_sign_reversal_present: bool,
    /// Present only when neither family nor seed-block sign reversal exists.
    pub pooled_summary: Option<PooledSynthesisSummary>,
    /// True only when a pooled Positive claim is admitted without either kind
    /// of reversal.
    pub claims_clean_pooled_win: bool,
    pub synthesis_contract: &'static str,
    pub uncertainty_contract: &'static str,
    pub metric_registry_contract: &'static str,
    pub cross_family_summary: CrossFamilySummaryMetricId,
    pub experimental_non_final: bool,
}

/// Map a finite paired-difference mean onto its algebraic sign.
pub fn effect_sign_from_mean(mean: f64) -> Result<EffectSign, EvalError> {
    if !mean.is_finite() {
        return Err(EvalError::FamilyStratifiedSynthesisInvalid {
            reason: "non_finite",
        });
    }
    if mean > 0.0 {
        Ok(EffectSign::Positive)
    } else if mean < 0.0 {
        Ok(EffectSign::Negative)
    } else {
        Ok(EffectSign::Zero)
    }
}

/// Internally classify an engine-derived paired-difference CI.
///
/// This remains private because a bare mean and interval do not carry the
/// sufficient statistics needed to prove that the engine derived the bounds.
fn classify_signed_effect(
    mean: f64,
    ci: ConfidenceInterval,
) -> Result<SignedEffectClass, EvalError> {
    require_admissible_confidence_interval(
        ci,
        mean,
        &[
            UncertaintyMethod::BoundedHoeffdingPairedDifference,
            UncertaintyMethod::ClusterHoeffdingPairedDifference,
        ],
    )?;
    if !(-1.0..=1.0).contains(&mean)
        || !(-1.0..=1.0).contains(&ci.lower)
        || !(-1.0..=1.0).contains(&ci.upper)
    {
        return Err(EvalError::FamilyStratifiedSynthesisInvalid {
            reason: "paired_difference_out_of_bounds",
        });
    }
    if ci.lower > 0.0 {
        Ok(SignedEffectClass::Positive)
    } else if ci.upper < 0.0 {
        Ok(SignedEffectClass::Harmful)
    } else {
        // A zero-crossing interval establishes neither direction nor equivalence.
        Ok(SignedEffectClass::Inconclusive)
    }
}

fn require_admissible_confidence_interval(
    ci: ConfidenceInterval,
    mean: f64,
    admitted_methods: &[UncertaintyMethod],
) -> Result<(), EvalError> {
    if !mean.is_finite()
        || !ci.lower.is_finite()
        || !ci.upper.is_finite()
        || !ci.confidence.is_finite()
    {
        return Err(EvalError::FamilyStratifiedSynthesisInvalid {
            reason: "non_finite",
        });
    }
    if ci.lower > ci.upper {
        return Err(EvalError::FamilyStratifiedSynthesisInvalid {
            reason: "non_finite",
        });
    }
    if (ci.confidence - PAIRED_UNCERTAINTY_LEVEL).abs() > 1e-12 {
        return Err(EvalError::FamilyStratifiedSynthesisInvalid {
            reason: "confidence_level_mismatch",
        });
    }
    if !admitted_methods.contains(&ci.method) {
        return Err(EvalError::FamilyStratifiedSynthesisInvalid {
            reason: "uncertainty_method_mismatch",
        });
    }
    if mean < ci.lower - 1e-12 || mean > ci.upper + 1e-12 {
        return Err(EvalError::FamilyStratifiedSynthesisInvalid {
            reason: "mean_outside_interval",
        });
    }
    Ok(())
}

fn family_signed_effect_from_summary(
    summary: &PairedEffectSummary,
) -> Result<FamilySignedEffect, EvalError> {
    let expected_integrity = PairedEffectSummaryIntegrity::from_summary(summary);
    if summary.integrity.as_ref() != Some(&expected_integrity) {
        return Err(EvalError::FamilyStratifiedSynthesisInvalid {
            reason: "summary_integrity_mismatch",
        });
    }
    if !summary.experimental_non_final {
        return Err(EvalError::FamilyStratifiedSynthesisInvalid {
            reason: "experimental_non_final_required",
        });
    }
    if summary.seed_blocks.is_empty()
        || summary.seed_blocks[0] != summary.seed_block
        || summary
            .seed_blocks
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
    {
        return Err(EvalError::FamilyStratifiedSynthesisInvalid {
            reason: "seed_block_mismatch",
        });
    }
    if summary.uncertainty_contract != PAIRED_UNCERTAINTY_CONTRACT {
        return Err(EvalError::FamilyStratifiedSynthesisInvalid {
            reason: "uncertainty_contract_mismatch",
        });
    }
    if summary.metric_registry_contract != METRIC_REGISTRY_CONTRACT {
        return Err(EvalError::FamilyStratifiedSynthesisInvalid {
            reason: "metric_registry_contract_mismatch",
        });
    }
    if summary.primary_family_metric != primary_family_metric_for(summary.family) {
        return Err(EvalError::FamilyStratifiedSynthesisInvalid {
            reason: "primary_family_metric_mismatch",
        });
    }
    if summary.cross_family_summary != CrossFamilySummaryMetricId::CrossFamilyPairedSummary {
        return Err(EvalError::FamilyStratifiedSynthesisInvalid {
            reason: "cross_family_summary_mismatch",
        });
    }
    if summary.secondary_paired_outcome_difference != SecondaryDiagnosticId::PairedOutcomeDifference
    {
        return Err(EvalError::FamilyStratifiedSynthesisInvalid {
            reason: "secondary_paired_outcome_mismatch",
        });
    }
    if summary.n_pairs < 2 {
        return Err(EvalError::FamilyStratifiedSynthesisInvalid {
            reason: "insufficient_pairs",
        });
    }
    if summary.seed_blocks.len() > MAX_SEED_BLOCKS_PER_SYNTHESIS {
        return Err(EvalError::FamilyStratifiedSynthesisInvalid {
            reason: "too_many_seed_blocks",
        });
    }
    let max_pairs = MAX_CASES_PER_RUN
        .checked_mul(summary.seed_blocks.len() as u64)
        .ok_or(EvalError::FamilyStratifiedSynthesisInvalid {
            reason: "too_many_pairs",
        })?;
    if summary.n_pairs > max_pairs {
        return Err(EvalError::FamilyStratifiedSynthesisInvalid {
            reason: "too_many_pairs",
        });
    }
    if !summary.t6_accuracy.is_finite()
        || !summary.c6_accuracy.is_finite()
        || !summary.paired_difference_mean.is_finite()
    {
        return Err(EvalError::FamilyStratifiedSynthesisInvalid {
            reason: "non_finite",
        });
    }
    if !(0.0..=1.0).contains(&summary.t6_accuracy) || !(0.0..=1.0).contains(&summary.c6_accuracy) {
        return Err(EvalError::FamilyStratifiedSynthesisInvalid {
            reason: "accuracy_out_of_bounds",
        });
    }
    let expected_gap = summary.c6_accuracy - summary.t6_accuracy;
    if (summary.paired_difference_mean - expected_gap).abs() > 1e-12 {
        return Err(EvalError::FamilyStratifiedSynthesisInvalid {
            reason: "paired_difference_inconsistent",
        });
    }
    if summary.paired_difference_mean < -1.0 - 1e-12 || summary.paired_difference_mean > 1.0 + 1e-12
    {
        return Err(EvalError::FamilyStratifiedSynthesisInvalid {
            reason: "paired_difference_out_of_bounds",
        });
    }

    require_admissible_confidence_interval(
        summary.paired_difference_ci,
        summary.paired_difference_mean,
        &[
            UncertaintyMethod::BoundedHoeffdingPairedDifference,
            UncertaintyMethod::ClusterHoeffdingPairedDifference,
        ],
    )?;
    require_admissible_confidence_interval(
        summary.t6_accuracy_ci,
        summary.t6_accuracy,
        &[
            UncertaintyMethod::WilsonScore,
            UncertaintyMethod::ClusterHoeffdingBernoulliMean,
        ],
    )?;
    require_admissible_confidence_interval(
        summary.c6_accuracy_ci,
        summary.c6_accuracy,
        &[
            UncertaintyMethod::WilsonScore,
            UncertaintyMethod::ClusterHoeffdingBernoulliMean,
        ],
    )?;

    let outcome_class =
        classify_signed_effect(summary.paired_difference_mean, summary.paired_difference_ci)?;
    Ok(FamilySignedEffect {
        family: summary.family,
        seed_block: summary.seed_block,
        seed_blocks: summary.seed_blocks.clone(),
        n_pairs: summary.n_pairs,
        paired_difference_mean: summary.paired_difference_mean,
        paired_difference_ci: summary.paired_difference_ci,
        effect_sign: effect_sign_from_mean(summary.paired_difference_mean)?,
        outcome_class,
        primary_family_metric: summary.primary_family_metric,
    })
}

fn detect_sign_reversal(effects: &[FamilySignedEffect]) -> bool {
    let mut saw_positive = false;
    let mut saw_negative = false;
    for effect in effects {
        match effect.effect_sign {
            EffectSign::Positive => saw_positive = true,
            EffectSign::Negative => saw_negative = true,
            EffectSign::Zero => {}
        }
    }
    saw_positive && saw_negative
}

fn detect_seed_block_sign_reversal(effects: &[FamilySignedEffect]) -> bool {
    REQUIRED_SYNTHESIS_FAMILIES.iter().any(|family| {
        let family_effects = effects.iter().filter(|effect| effect.family == *family);
        let has_positive = family_effects
            .clone()
            .any(|effect| effect.effect_sign == EffectSign::Positive);
        let has_negative = family_effects
            .clone()
            .any(|effect| effect.effect_sign == EffectSign::Negative);
        has_positive && has_negative
    })
}

fn pooled_summary_from_effects(
    effects: &[FamilySignedEffect],
    cross_family_summary: CrossFamilySummaryMetricId,
) -> Result<PooledSynthesisSummary, EvalError> {
    if effects.is_empty() {
        return Err(EvalError::FamilyStratifiedSynthesisInvalid {
            reason: "empty_families",
        });
    }
    let mut total_pairs = 0_u64;
    let mut weighted = 0.0_f64;
    for effect in effects {
        if effect.n_pairs == 0 {
            return Err(EvalError::FamilyStratifiedSynthesisInvalid {
                reason: "empty_pairs",
            });
        }
        total_pairs = total_pairs.checked_add(effect.n_pairs).ok_or(
            EvalError::FamilyStratifiedSynthesisInvalid {
                reason: "non_finite",
            },
        )?;
        weighted += effect.paired_difference_mean * (effect.n_pairs as f64);
    }
    let mean = weighted / (total_pairs as f64);
    if !mean.is_finite() {
        return Err(EvalError::FamilyStratifiedSynthesisInvalid {
            reason: "non_finite",
        });
    }
    // A pooled directional class is admitted only when every family interval
    // supports that same direction. A positive weighted mean alone is not
    // evidence of a clean pooled win.
    let all_positive = effects
        .iter()
        .all(|effect| effect.outcome_class == SignedEffectClass::Positive);
    let all_harmful = effects
        .iter()
        .all(|effect| effect.outcome_class == SignedEffectClass::Harmful);
    let outcome_class = if all_positive {
        SignedEffectClass::Positive
    } else if all_harmful {
        SignedEffectClass::Harmful
    } else {
        SignedEffectClass::Inconclusive
    };
    Ok(PooledSynthesisSummary {
        n_pairs: total_pairs,
        paired_difference_mean: mean,
        effect_sign: effect_sign_from_mean(mean)?,
        outcome_class,
        cross_family_summary,
    })
}

/// Synthesise family-stratified signed effects from per-family paired summaries.
///
/// Requires Development/Validation, the pinned metric registry, and every
/// family in [`REQUIRED_SYNTHESIS_FAMILIES`] exactly once. A pooled summary is
/// emitted only when no Positive/Negative family sign reversal is present;
/// otherwise `sign_reversal_present` is set and `claims_clean_pooled_win` stays
/// false so a pooled win cannot hide a family-specific reversal.
pub fn synthesize_family_stratified_effects(
    split: DataSplit,
    family_summaries: &[PairedEffectSummary],
    registry: &MetricRegistry,
) -> Result<FamilyStratifiedSynthesisReport, EvalError> {
    validate_non_final_split(split)?;
    require_pinned_metric_registry(registry)?;
    if family_summaries.is_empty() {
        return Err(EvalError::FamilyStratifiedSynthesisInvalid {
            reason: "empty_families",
        });
    }

    let mut effects = Vec::with_capacity(family_summaries.len());
    for summary in family_summaries {
        if summary.split != split {
            return Err(EvalError::FamilyStratifiedSynthesisInvalid {
                reason: "split_mismatch",
            });
        }
        if effects
            .iter()
            .any(|prior: &FamilySignedEffect| prior.family == summary.family)
        {
            return Err(EvalError::FamilyStratifiedSynthesisInvalid {
                reason: "duplicate_family",
            });
        }
        effects.push(family_signed_effect_from_summary(summary)?);
    }

    // Emit families in the canonical Stage-C order when present.
    effects.sort_by_key(|effect| {
        REQUIRED_SYNTHESIS_FAMILIES
            .iter()
            .position(|family| *family == effect.family)
            .unwrap_or(REQUIRED_SYNTHESIS_FAMILIES.len())
    });

    for required in REQUIRED_SYNTHESIS_FAMILIES {
        if !effects.iter().any(|effect| effect.family == *required) {
            return Err(EvalError::FamilyStratifiedSynthesisInvalid {
                reason: "missing_family",
            });
        }
    }
    if effects.len() != REQUIRED_SYNTHESIS_FAMILIES.len() {
        return Err(EvalError::FamilyStratifiedSynthesisInvalid {
            reason: "missing_family",
        });
    }

    let seed_block_effects: Vec<FamilySignedEffect> = effects
        .iter()
        .filter(|effect| effect.seed_blocks.len() == 1)
        .cloned()
        .collect();
    let sign_reversal_present = detect_sign_reversal(&effects);
    let seed_block_sign_reversal_present = detect_seed_block_sign_reversal(&seed_block_effects);
    let cross_family_summary = registry.cross_family_summary;
    let pooled_summary = if sign_reversal_present || seed_block_sign_reversal_present {
        None
    } else {
        Some(pooled_summary_from_effects(&effects, cross_family_summary)?)
    };
    let claims_clean_pooled_win = matches!(
        pooled_summary.as_ref().map(|pooled| pooled.outcome_class),
        Some(SignedEffectClass::Positive)
    );

    Ok(FamilyStratifiedSynthesisReport {
        split,
        family_effects: effects,
        seed_block_effects,
        sign_reversal_present,
        seed_block_sign_reversal_present,
        pooled_summary,
        claims_clean_pooled_win,
        synthesis_contract: FAMILY_STRATIFIED_SYNTHESIS_CONTRACT,
        uncertainty_contract: PAIRED_UNCERTAINTY_CONTRACT,
        metric_registry_contract: registry.registry_contract,
        cross_family_summary,
        experimental_non_final: true,
    })
}

/// Group revealed T6/C6 match outcomes by family and seed block, aggregate each
/// family with cluster-aware intervals, then run stratified synthesis.
///
/// All outcomes must share `split`. A single block retains the within-block
/// Wilson/Hoeffding path; multiple blocks use `sum_g n_g^2` cluster bounds.
/// Does not accept [`super::tdi25_tasks::ProtectedLabel`].
pub fn synthesize_family_stratified_from_revealed_outcomes(
    split: DataSplit,
    t6_matches: &[RevealedMatchOutcome],
    c6_matches: &[RevealedMatchOutcome],
    registry: &MetricRegistry,
) -> Result<FamilyStratifiedSynthesisReport, EvalError> {
    validate_non_final_split(split)?;
    require_pinned_metric_registry(registry)?;
    if t6_matches.is_empty() || c6_matches.is_empty() {
        return Err(EvalError::FamilyStratifiedSynthesisInvalid {
            reason: "empty_pairs",
        });
    }
    require_pair_identity_alignment(t6_matches, c6_matches)?;

    let mut families: Vec<TaskFamily> = Vec::new();
    for outcome in t6_matches {
        if outcome.split != split {
            return Err(EvalError::FamilyStratifiedSynthesisInvalid {
                reason: "split_mismatch",
            });
        }
        if !families.contains(&outcome.family) {
            families.push(outcome.family);
        }
    }

    let mut summaries = Vec::with_capacity(families.len());
    let mut seed_block_summaries = Vec::new();
    for family in families {
        let t6_family: Vec<RevealedMatchOutcome> = t6_matches
            .iter()
            .filter(|outcome| outcome.family == family)
            .cloned()
            .collect();
        let c6_family: Vec<RevealedMatchOutcome> = c6_matches
            .iter()
            .filter(|outcome| outcome.family == family)
            .cloned()
            .collect();
        if t6_family.is_empty() {
            return Err(EvalError::FamilyStratifiedSynthesisInvalid {
                reason: "empty_pairs",
            });
        }
        let mut seed_blocks = Vec::new();
        for outcome in &t6_family {
            if !seed_blocks.contains(&outcome.seed_block) {
                seed_blocks.push(outcome.seed_block);
            }
        }
        seed_blocks.sort_unstable();
        for block in &seed_blocks {
            let t6_block: Vec<RevealedMatchOutcome> = t6_family
                .iter()
                .filter(|outcome| outcome.seed_block == *block)
                .cloned()
                .collect();
            let c6_block: Vec<RevealedMatchOutcome> = c6_family
                .iter()
                .filter(|outcome| outcome.seed_block == *block)
                .cloned()
                .collect();
            seed_block_summaries.push(
                summarize_paired_uncertainty_by_seed_block(
                    split, family, *block, &t6_block, &c6_block, registry,
                )
                .map_err(|err| match err {
                    EvalError::PairedUncertaintyInvalid { reason } => {
                        EvalError::FamilyStratifiedSynthesisInvalid { reason }
                    }
                    other => other,
                })?,
            );
        }
        let seed_block = seed_blocks[0];
        let cluster_sum_squares = if seed_blocks.len() > 1 {
            Some(sum_squared_seed_block_sizes(&t6_family)?)
        } else {
            None
        };
        let summary = summarize_paired_uncertainty_inner(
            split,
            family,
            seed_block,
            &t6_family,
            &c6_family,
            registry,
            cluster_sum_squares,
        )
        .map_err(|err| match err {
            EvalError::PairedUncertaintyInvalid { reason } => {
                EvalError::FamilyStratifiedSynthesisInvalid { reason }
            }
            other => other,
        })?;
        summaries.push(summary);
    }

    let mut report = synthesize_family_stratified_effects(split, &summaries, registry)?;
    report.seed_block_effects = seed_block_summaries
        .iter()
        .map(family_signed_effect_from_summary)
        .collect::<Result<Vec<_>, _>>()?;
    report.seed_block_effects.sort_by_key(|effect| {
        (
            REQUIRED_SYNTHESIS_FAMILIES
                .iter()
                .position(|family| *family == effect.family)
                .unwrap_or(REQUIRED_SYNTHESIS_FAMILIES.len()),
            effect.seed_block,
        )
    });
    report.seed_block_sign_reversal_present =
        detect_seed_block_sign_reversal(&report.seed_block_effects);
    if report.seed_block_sign_reversal_present {
        report.pooled_summary = None;
        report.claims_clean_pooled_win = false;
    }
    Ok(report)
}

/// Closed retained failure class for TDI-25 Stage-C accounting by arm.
///
/// Mirrors the campaign gate wording (numerical / task / resource) plus an
/// explicit `invalid` class for configuration/contract rejections. Unknown
/// labels fail closed. Experimental and non-final only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailureClass {
    /// Contract, budget, registry, matcher or other invalid configuration/input.
    Invalid,
    /// Upstream torsor/chiral/generic/normalizer arithmetic rejected a value.
    Numerical,
    /// Case budget exhausted.
    Resource,
    /// Task case was non-canonical, mislabeled or drawn from the wrong split.
    Task,
}

impl FailureClass {
    /// Stable lowercase failure-class token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Invalid => "invalid",
            Self::Numerical => "numerical",
            Self::Resource => "resource",
            Self::Task => "task",
        }
    }
}

/// Parse a failure-class label; unknown tokens fail closed.
pub fn parse_failure_class(label: &str) -> Result<FailureClass, EvalError> {
    match label {
        "invalid" => Ok(FailureClass::Invalid),
        "numerical" => Ok(FailureClass::Numerical),
        "resource" => Ok(FailureClass::Resource),
        "task" => Ok(FailureClass::Task),
        "" => Err(EvalError::FailureResourceAccountingInvalid {
            reason: "empty_class",
        }),
        _ => Err(EvalError::FailureResourceAccountingInvalid {
            reason: "unknown_class",
        }),
    }
}

const fn classify_bridge_error(error: &Tdi25Error) -> FailureClass {
    match error {
        Tdi25Error::Torsor(_)
        | Tdi25Error::Chiral(_)
        | Tdi25Error::Normalizer(_)
        | Tdi25Error::NonFiniteGeneric
        | Tdi25Error::NonFiniteScalar { .. }
        | Tdi25Error::InvalidScoreScale => FailureClass::Numerical,
        Tdi25Error::InvalidComparisonRecord { .. }
        | Tdi25Error::ChiralInvariantViolation { .. }
        | Tdi25Error::SourceContractMismatch { .. } => FailureClass::Invalid,
    }
}

/// Classify an [`EvalError`] into the closed [`FailureClass`] set.
///
/// Every admitted error maps to exactly one class. Protected/final split
/// errors are never retained as a class: they stay hard rejections so this
/// accounting surface cannot authorise protected/final evaluation.
pub fn classify_eval_error(error: &EvalError) -> Result<FailureClass, EvalError> {
    match error {
        EvalError::ProtectedOrFinalSplit => Err(EvalError::ProtectedOrFinalSplit),
        EvalError::Bridge(bridge) => Ok(classify_bridge_error(bridge)),
        EvalError::CaseBudgetExceeded => Ok(FailureClass::Resource),
        EvalError::SplitMismatch { .. } => Ok(FailureClass::Task),
        EvalError::ContractMismatch(
            "torsor_transport_case"
            | "mixed_case"
            | "chiral_reflection_case"
            | "neutral_control_case",
        ) => Ok(FailureClass::Task),
        EvalError::PairedUncertaintyInvalid {
            reason: "non_finite",
        } => Ok(FailureClass::Numerical),
        EvalError::ContractMismatch(_)
        | EvalError::InvalidBudget
        | EvalError::ParameterReadoutMismatch { .. }
        | EvalError::OptimizerUpdateBudgetMismatch { .. }
        | EvalError::MetricRegistryInvalid { .. }
        | EvalError::PairedUncertaintyInvalid { .. }
        | EvalError::FamilyStratifiedSynthesisInvalid { .. }
        | EvalError::FailureResourceAccountingInvalid { .. }
        | EvalError::StageCPreflightInvalid { .. }
        | EvalError::ReductionPointAblationInvalid { .. }
        | EvalError::TorsorBridgeEquivalenceInvalid { .. }
        | EvalError::ChiralGammaZeroAblationInvalid { .. }
        | EvalError::ChiralParityShuffleControlInvalid { .. }
        | EvalError::TorsorStructureShuffleControlInvalid { .. }
        | EvalError::G6OrthogonalBasisControlInvalid { .. }
        | EvalError::PositionGeometryAblationInvalid { .. }
        | EvalError::SequenceLengthScalingInvalid { .. }
        | EvalError::DataVolumeScalingInvalid { .. }
        | EvalError::StageDAttributionAuditInvalid { .. }
        | EvalError::MultiSeedReplicationInvalid { .. }
        | EvalError::InputNoiseRobustnessInvalid { .. }
        | EvalError::TranslationOriginStressInvalid { .. } => Ok(FailureClass::Invalid),
    }
}

/// Stable message code for one [`EvalError`] variant.
#[must_use]
pub const fn eval_error_message_code(error: &EvalError) -> &'static str {
    match error {
        EvalError::ContractMismatch(_) => "contract_mismatch",
        EvalError::InvalidBudget => "invalid_budget",
        EvalError::SplitMismatch { .. } => "split_mismatch",
        EvalError::CaseBudgetExceeded => "case_budget_exceeded",
        EvalError::Bridge(Tdi25Error::Torsor(_)) => "bridge_torsor",
        EvalError::Bridge(Tdi25Error::Chiral(_)) => "bridge_chiral",
        EvalError::Bridge(Tdi25Error::Normalizer(_)) => "bridge_normalizer",
        EvalError::Bridge(Tdi25Error::NonFiniteGeneric) => "bridge_non_finite_generic",
        EvalError::Bridge(Tdi25Error::NonFiniteScalar { .. }) => "bridge_non_finite_scalar",
        EvalError::Bridge(Tdi25Error::InvalidScoreScale) => "bridge_invalid_score_scale",
        EvalError::Bridge(Tdi25Error::InvalidComparisonRecord { .. }) => {
            "bridge_invalid_comparison_record"
        }
        EvalError::Bridge(Tdi25Error::ChiralInvariantViolation { .. }) => {
            "bridge_chiral_invariant_violation"
        }
        EvalError::Bridge(Tdi25Error::SourceContractMismatch { .. }) => {
            "bridge_source_contract_mismatch"
        }
        EvalError::ParameterReadoutMismatch { .. } => "parameter_readout_mismatch",
        EvalError::OptimizerUpdateBudgetMismatch { .. } => "optimizer_update_budget_mismatch",
        EvalError::MetricRegistryInvalid { .. } => "metric_registry_invalid",
        EvalError::ProtectedOrFinalSplit => "protected_or_final_split",
        EvalError::PairedUncertaintyInvalid { .. } => "paired_uncertainty_invalid",
        EvalError::FamilyStratifiedSynthesisInvalid { .. } => "family_stratified_synthesis_invalid",
        EvalError::FailureResourceAccountingInvalid { .. } => "failure_resource_accounting_invalid",
        EvalError::StageCPreflightInvalid { .. } => "stage_c_preflight_invalid",
        EvalError::ReductionPointAblationInvalid { .. } => "reduction_point_ablation_invalid",
        EvalError::TorsorBridgeEquivalenceInvalid { .. } => "torsor_bridge_equivalence_invalid",
        EvalError::ChiralGammaZeroAblationInvalid { .. } => "chiral_gamma_zero_ablation_invalid",
        EvalError::ChiralParityShuffleControlInvalid { .. } => {
            "chiral_parity_shuffle_control_invalid"
        }
        EvalError::TorsorStructureShuffleControlInvalid { .. } => {
            "torsor_structure_shuffle_control_invalid"
        }
        EvalError::G6OrthogonalBasisControlInvalid { .. } => "g6_orthogonal_basis_control_invalid",
        EvalError::PositionGeometryAblationInvalid { .. } => "position_geometry_ablation_invalid",
        EvalError::SequenceLengthScalingInvalid { .. } => "sequence_length_scaling_invalid",
        EvalError::DataVolumeScalingInvalid { .. } => "data_volume_scaling_invalid",
        EvalError::StageDAttributionAuditInvalid { .. } => "stage_d_attribution_audit_invalid",
        EvalError::MultiSeedReplicationInvalid { .. } => "multi_seed_replication_invalid",
        EvalError::InputNoiseRobustnessInvalid { .. } => "input_noise_robustness_invalid",
        EvalError::TranslationOriginStressInvalid { .. } => "translation_origin_stress_invalid",
    }
}

/// One retained typed failure attributed to exactly one arm.
///
/// Never silently discarded once classified. `case_id` is `None` only for
/// atomic matched-reference block failures, which are retained on both
/// primary arms.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArmFailureRecord {
    pub class: FailureClass,
    pub arm: ComparisonArm,
    pub split: DataSplit,
    pub family: TaskFamily,
    pub case_id: Option<u64>,
    pub message_code: &'static str,
    pub accounting_contract: &'static str,
}

/// Validate a retained failure record against the closed accounting pin.
pub fn validate_arm_failure_record(record: &ArmFailureRecord) -> Result<(), EvalError> {
    validate_non_final_split(record.split)?;
    if record.accounting_contract != FAILURE_RESOURCE_ACCOUNTING_CONTRACT {
        return Err(EvalError::FailureResourceAccountingInvalid {
            reason: "contract_drift",
        });
    }
    if parse_failure_class(record.class.as_str())? != record.class {
        return Err(EvalError::FailureResourceAccountingInvalid {
            reason: "unknown_class",
        });
    }
    if record.message_code.is_empty() {
        return Err(EvalError::FailureResourceAccountingInvalid {
            reason: "empty_message_code",
        });
    }
    Ok(())
}

/// Per-arm resource and failure account for one bounded non-final ledger.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArmResourceAccount {
    pub arm: ComparisonArm,
    /// Matched readout budget shared by every accounted arm.
    pub budget: ReadoutBudget,
    /// Cases that produced an evaluator-minted score.
    pub scored_cases: u64,
    /// Readout scalars retained (score + oracle-match flag per scored case).
    pub readout_scalars_used: u64,
    pub numerical_failures: u64,
    pub task_failures: u64,
    pub resource_failures: u64,
    pub invalid_failures: u64,
    /// Attempts (scored + retained failures) by family, in
    /// [`REQUIRED_SYNTHESIS_FAMILIES`] order.
    pub attempts_by_family: [u64; 4],
}

impl ArmResourceAccount {
    const fn open(arm: ComparisonArm) -> Self {
        Self {
            arm,
            budget: ReadoutBudget::matched_non_trained(),
            scored_cases: 0,
            readout_scalars_used: 0,
            numerical_failures: 0,
            task_failures: 0,
            resource_failures: 0,
            invalid_failures: 0,
            attempts_by_family: [0; 4],
        }
    }

    /// Total retained failures across every class.
    #[must_use]
    pub const fn retained_failures(&self) -> u64 {
        self.numerical_failures
            + self.task_failures
            + self.resource_failures
            + self.invalid_failures
    }

    /// Scored cases plus retained failures.
    #[must_use]
    pub const fn attempted_cases(&self) -> u64 {
        self.scored_cases + self.retained_failures()
    }

    /// Retained failures of one class.
    #[must_use]
    pub const fn failures_of(&self, class: FailureClass) -> u64 {
        match class {
            FailureClass::Invalid => self.invalid_failures,
            FailureClass::Numerical => self.numerical_failures,
            FailureClass::Resource => self.resource_failures,
            FailureClass::Task => self.task_failures,
        }
    }
}

const fn accounted_arm_index(arm: ComparisonArm) -> usize {
    match arm {
        ComparisonArm::T6 => 0,
        ComparisonArm::C6 => 1,
        ComparisonArm::G6 => 2,
    }
}

const fn synthesis_family_index(family: TaskFamily) -> usize {
    match family {
        TaskFamily::TorsorFavorable => 0,
        TaskFamily::ChiralFavorable => 1,
        TaskFamily::Mixed => 2,
        TaskFamily::Neutral => 3,
    }
}

/// Bounded ledger retaining numerical/task/resource/invalid failures by arm and
/// the matched readout resources each arm consumed.
///
/// Scores can only be counted through the evaluator-backed `account_*`
/// methods; there is no public "record a score" surface. Development/
/// Validation only; protected/final errors stay hard rejections.
#[derive(Clone, Debug, PartialEq)]
pub struct FailureResourceLedger {
    split: DataSplit,
    accounting_contract: &'static str,
    accounts: [ArmResourceAccount; 3],
    failures: Vec<ArmFailureRecord>,
}

impl FailureResourceLedger {
    /// Open an empty bounded ledger for one non-final split.
    pub fn open(split: DataSplit) -> Result<Self, EvalError> {
        validate_non_final_split(split)?;
        Ok(Self {
            split,
            accounting_contract: FAILURE_RESOURCE_ACCOUNTING_CONTRACT,
            accounts: [
                ArmResourceAccount::open(ComparisonArm::T6),
                ArmResourceAccount::open(ComparisonArm::C6),
                ArmResourceAccount::open(ComparisonArm::G6),
            ],
            failures: Vec::new(),
        })
    }

    /// Split this ledger was opened under.
    #[must_use]
    pub const fn split(&self) -> DataSplit {
        self.split
    }

    /// Accounting contract pin.
    #[must_use]
    pub const fn accounting_contract(&self) -> &'static str {
        self.accounting_contract
    }

    /// Borrow one arm's account.
    #[must_use]
    pub const fn account(&self, arm: ComparisonArm) -> &ArmResourceAccount {
        &self.accounts[accounted_arm_index(arm)]
    }

    /// Borrow retained failures in admission order.
    #[must_use]
    pub fn failures(&self) -> &[ArmFailureRecord] {
        &self.failures
    }

    fn require_capacity(&self, arm: ComparisonArm) -> Result<(), EvalError> {
        if self.accounting_contract != FAILURE_RESOURCE_ACCOUNTING_CONTRACT {
            return Err(EvalError::FailureResourceAccountingInvalid {
                reason: "contract_drift",
            });
        }
        if self.account(arm).attempted_cases() >= MAX_ACCOUNTED_CASES_PER_ARM {
            return Err(EvalError::CaseBudgetExceeded);
        }
        Ok(())
    }

    fn record_scored(&mut self, arm: ComparisonArm, family: TaskFamily) -> Result<(), EvalError> {
        self.require_capacity(arm)?;
        let account = &mut self.accounts[accounted_arm_index(arm)];
        account.scored_cases += 1;
        account.readout_scalars_used += MAX_READOUT_SCALARS_PER_CASE;
        account.attempts_by_family[synthesis_family_index(family)] += 1;
        Ok(())
    }

    /// Classify and retain one evaluator error against `arm`; never silently drops.
    ///
    /// Protected/final errors are returned unchanged and never retained.
    pub fn retain_failure(
        &mut self,
        arm: ComparisonArm,
        family: TaskFamily,
        case_id: Option<u64>,
        error: &EvalError,
    ) -> Result<&ArmFailureRecord, EvalError> {
        let class = classify_eval_error(error)?;
        self.require_capacity(arm)?;
        if self.account(arm).retained_failures() >= MAX_FAILURES_PER_ARM {
            return Err(EvalError::CaseBudgetExceeded);
        }
        let record = ArmFailureRecord {
            class,
            arm,
            split: self.split,
            family,
            case_id,
            message_code: eval_error_message_code(error),
            accounting_contract: FAILURE_RESOURCE_ACCOUNTING_CONTRACT,
        };
        validate_arm_failure_record(&record)?;
        let account = &mut self.accounts[accounted_arm_index(arm)];
        match class {
            FailureClass::Invalid => account.invalid_failures += 1,
            FailureClass::Numerical => account.numerical_failures += 1,
            FailureClass::Resource => account.resource_failures += 1,
            FailureClass::Task => account.task_failures += 1,
        }
        account.attempts_by_family[synthesis_family_index(family)] += 1;
        self.failures.push(record);
        Ok(self.failures.last().expect("failure was just retained"))
    }

    fn require_run_split(&self, run_split: DataSplit) -> Result<(), EvalError> {
        if run_split != self.split {
            return Err(EvalError::FailureResourceAccountingInvalid {
                reason: "run_split_mismatch",
            });
        }
        Ok(())
    }

    fn settle<R>(
        &mut self,
        arm: ComparisonArm,
        family: TaskFamily,
        case_id: u64,
        result: Result<R, EvalError>,
    ) -> Result<Option<R>, EvalError> {
        match result {
            Ok(outcome) => {
                self.record_scored(arm, family)?;
                Ok(Some(outcome))
            }
            Err(error) => {
                self.retain_failure(arm, family, Some(case_id), &error)?;
                Ok(None)
            }
        }
    }

    /// Evaluate one torsor-favorable case on T6, accounting score or failure.
    pub fn account_t6_torsor_transport(
        &mut self,
        run: &mut T6EvaluatorRun,
        case: &LabeledCase<TorsorTransportInput, TorsorTransportOracle>,
    ) -> Result<Option<T6Outcome>, EvalError> {
        self.require_run_split(run.config.split)?;
        let input = case.inference_input();
        let result = run
            .evaluate_torsor_transport(case)
            .map(|record| record.outcome);
        self.settle(ComparisonArm::T6, input.task_family, input.case_id, result)
    }

    /// Evaluate one mixed case on T6, accounting score or failure.
    pub fn account_t6_mixed(
        &mut self,
        run: &mut T6EvaluatorRun,
        case: &LabeledCase<MixedGeometryInput, MixedGeometryOracle>,
    ) -> Result<Option<T6Outcome>, EvalError> {
        self.require_run_split(run.config.split)?;
        let input = case.inference_input();
        let result = run.evaluate_mixed(case).map(|record| record.outcome);
        self.settle(ComparisonArm::T6, input.task_family, input.case_id, result)
    }

    /// Evaluate one chiral-favorable case on C6, accounting score or failure.
    pub fn account_c6_chiral_reflection(
        &mut self,
        run: &mut C6EvaluatorRun,
        case: &LabeledCase<ChiralReflectionInput, ChiralReflectionOracle>,
    ) -> Result<Option<C6Outcome>, EvalError> {
        self.require_run_split(run.config.split)?;
        let input = case.inference_input();
        let result = run
            .evaluate_chiral_reflection(case)
            .map(|record| record.outcome);
        self.settle(ComparisonArm::C6, input.task_family, input.case_id, result)
    }

    /// Evaluate one mixed case on C6, accounting score or failure.
    pub fn account_c6_mixed(
        &mut self,
        run: &mut C6EvaluatorRun,
        case: &LabeledCase<MixedGeometryInput, MixedGeometryOracle>,
    ) -> Result<Option<C6Outcome>, EvalError> {
        self.require_run_split(run.config.split)?;
        let input = case.inference_input();
        let result = run.evaluate_mixed(case).map(|record| record.outcome);
        self.settle(ComparisonArm::C6, input.task_family, input.case_id, result)
    }

    /// Evaluate one neutral control case on G6, accounting score or failure.
    pub fn account_g6_neutral_control(
        &mut self,
        run: &mut G6EvaluatorRun,
        case: &LabeledCase<NeutralControlInput, NeutralControlOracle>,
    ) -> Result<Option<G6Outcome>, EvalError> {
        self.require_run_split(run.config.split)?;
        let input = case.inference_input();
        let result = run
            .evaluate_neutral_control(case)
            .map(|record| record.outcome);
        self.settle(ComparisonArm::G6, input.task_family, input.case_id, result)
    }

    /// Execute one matched-reference block and account both primary arms.
    ///
    /// The matched block is atomic: a rejection is retained once on **both**
    /// T6 and C6 (`case_id = None`), so neither arm can silently omit it.
    pub fn account_matched_primary_block(
        &mut self,
        family: TaskFamily,
        seed_block: u64,
        n_cases: u64,
    ) -> Result<Option<matched_reference::MatchedPrimaryRun>, EvalError> {
        match matched_reference::MatchedPrimaryRun::evaluate(
            self.split, family, seed_block, n_cases,
        ) {
            Ok(run) => {
                let cases = run.t6_outcomes().len() as u64;
                if cases != run.c6_outcomes().len() as u64 || run.split() != self.split {
                    return Err(EvalError::FailureResourceAccountingInvalid {
                        reason: "matched_block_mismatch",
                    });
                }
                for arm in [ComparisonArm::T6, ComparisonArm::C6] {
                    let account = self.account(arm);
                    if account.attempted_cases() + cases > MAX_ACCOUNTED_CASES_PER_ARM {
                        return Err(EvalError::CaseBudgetExceeded);
                    }
                }
                for _ in 0..cases {
                    self.record_scored(ComparisonArm::T6, family)?;
                    self.record_scored(ComparisonArm::C6, family)?;
                }
                Ok(Some(run))
            }
            Err(EvalError::ProtectedOrFinalSplit) => Err(EvalError::ProtectedOrFinalSplit),
            Err(error) => {
                self.retain_failure(ComparisonArm::T6, family, None, &error)?;
                self.retain_failure(ComparisonArm::C6, family, None, &error)?;
                Ok(None)
            }
        }
    }

    /// Snapshot the ledger into a validated report.
    pub fn report(&self) -> Result<FailureResourceReport, EvalError> {
        let t6 = self.account(ComparisonArm::T6);
        let c6 = self.account(ComparisonArm::C6);
        let report = FailureResourceReport {
            split: self.split,
            accounting_contract: self.accounting_contract,
            arms: self.accounts.to_vec(),
            failures: self.failures.clone(),
            primary_attempts_matched: t6.attempts_by_family == c6.attempts_by_family,
            primary_failure_free: t6.retained_failures() == 0 && c6.retained_failures() == 0,
            experimental_non_final: true,
        };
        validate_failure_resource_report(&report)?;
        Ok(report)
    }
}

/// Immutable per-arm failure/resource report for one non-final split.
#[derive(Clone, Debug, PartialEq)]
pub struct FailureResourceReport {
    pub split: DataSplit,
    pub accounting_contract: &'static str,
    /// T6, C6, G6 accounts in that canonical order.
    pub arms: Vec<ArmResourceAccount>,
    /// Every retained failure, attributed to one arm.
    pub failures: Vec<ArmFailureRecord>,
    /// T6 and C6 attempted identical case counts in every family.
    pub primary_attempts_matched: bool,
    /// Neither primary arm retained a failure.
    pub primary_failure_free: bool,
    /// Must remain true: experimental and non-final only.
    pub experimental_non_final: bool,
}

/// Validate a report: contract pin, canonical arm order, matched budgets,
/// readout accounting, and per-class counts reconciled against retained records.
pub fn validate_failure_resource_report(report: &FailureResourceReport) -> Result<(), EvalError> {
    validate_non_final_split(report.split)?;
    if report.accounting_contract != FAILURE_RESOURCE_ACCOUNTING_CONTRACT {
        return Err(EvalError::FailureResourceAccountingInvalid {
            reason: "contract_drift",
        });
    }
    if !report.experimental_non_final {
        return Err(EvalError::FailureResourceAccountingInvalid {
            reason: "experimental_non_final_required",
        });
    }
    let arms: Vec<ComparisonArm> = report.arms.iter().map(|account| account.arm).collect();
    if arms != ACCOUNTED_ARMS {
        return Err(EvalError::FailureResourceAccountingInvalid {
            reason: "arm_order_mismatch",
        });
    }
    for record in &report.failures {
        validate_arm_failure_record(record)?;
        if record.split != report.split {
            return Err(EvalError::SplitMismatch {
                expected: report.split,
                actual: record.split,
            });
        }
    }
    for account in &report.arms {
        if account.budget != ReadoutBudget::matched_non_trained() {
            return Err(EvalError::FailureResourceAccountingInvalid {
                reason: "unmatched_arm_budget",
            });
        }
        if account.readout_scalars_used != account.scored_cases * MAX_READOUT_SCALARS_PER_CASE {
            return Err(EvalError::FailureResourceAccountingInvalid {
                reason: "readout_accounting_mismatch",
            });
        }
        if account.attempted_cases() > MAX_ACCOUNTED_CASES_PER_ARM
            || account.retained_failures() > MAX_FAILURES_PER_ARM
        {
            return Err(EvalError::FailureResourceAccountingInvalid {
                reason: "accounting_capacity_exceeded",
            });
        }
        if account.attempts_by_family.iter().sum::<u64>() != account.attempted_cases() {
            return Err(EvalError::FailureResourceAccountingInvalid {
                reason: "family_attempts_mismatch",
            });
        }
        for class in [
            FailureClass::Invalid,
            FailureClass::Numerical,
            FailureClass::Resource,
            FailureClass::Task,
        ] {
            let retained = report
                .failures
                .iter()
                .filter(|record| record.arm == account.arm && record.class == class)
                .count() as u64;
            if retained != account.failures_of(class) {
                return Err(EvalError::FailureResourceAccountingInvalid {
                    reason: "failure_not_retained",
                });
            }
        }
    }
    let t6 = &report.arms[accounted_arm_index(ComparisonArm::T6)];
    let c6 = &report.arms[accounted_arm_index(ComparisonArm::C6)];
    if report.primary_attempts_matched != (t6.attempts_by_family == c6.attempts_by_family)
        || report.primary_failure_free
            != (t6.retained_failures() == 0 && c6.retained_failures() == 0)
    {
        return Err(EvalError::FailureResourceAccountingInvalid {
            reason: "primary_flag_mismatch",
        });
    }
    Ok(())
}

/// Require complete primary accounting before any clean T6-vs-C6 summary.
///
/// Fails closed when T6/C6 attempted different case counts in any family or
/// when either primary arm retained a failure — so a pooled or stratified
/// summary cannot silently drop failed cases from one arm.
pub fn require_complete_primary_accounting(
    report: &FailureResourceReport,
) -> Result<(), EvalError> {
    validate_failure_resource_report(report)?;
    if !report.primary_attempts_matched {
        return Err(EvalError::FailureResourceAccountingInvalid {
            reason: "unmatched_primary_family_attempts",
        });
    }
    if !report.primary_failure_free {
        return Err(EvalError::FailureResourceAccountingInvalid {
            reason: "primary_failures_retained",
        });
    }
    Ok(())
}

/// Bounded Stage-C preflight budget: seed blocks per family and matched cases
/// per block. Both are strictly bounded; this is a smoke run, not a campaign.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StageCPreflightBudget {
    pub seed_blocks: u64,
    pub cases_per_block: u64,
}

impl StageCPreflightBudget {
    /// Smallest admissible smoke budget: two seed blocks of eight cases.
    #[must_use]
    pub const fn smoke() -> Self {
        Self {
            seed_blocks: MAX_PREFLIGHT_SEED_BLOCKS,
            cases_per_block: 8,
        }
    }

    /// Reject empty, single-case, or over-budget preflights fail-closed.
    pub const fn validate(self) -> Result<(), EvalError> {
        if self.seed_blocks == 0 || self.seed_blocks > MAX_PREFLIGHT_SEED_BLOCKS {
            return Err(EvalError::StageCPreflightInvalid {
                reason: "seed_block_budget",
            });
        }
        if self.cases_per_block < 2 || self.cases_per_block > MAX_PREFLIGHT_CASES_PER_BLOCK {
            return Err(EvalError::StageCPreflightInvalid {
                reason: "case_budget",
            });
        }
        Ok(())
    }

    /// Matched cases scored per primary arm across all required families.
    #[must_use]
    pub const fn cases_per_arm(self) -> u64 {
        REQUIRED_SYNTHESIS_FAMILIES.len() as u64 * self.seed_blocks * self.cases_per_block
    }
}

/// Immutable Stage-C bounded preflight report (Development/Validation only).
#[derive(Clone, Debug, PartialEq)]
pub struct StageCPreflightReport {
    pub split: DataSplit,
    pub preflight_contract: &'static str,
    pub matrix_contract: &'static str,
    pub budget: StageCPreflightBudget,
    /// Matched T6/C6 pairs revealed and consumed by the synthesis.
    pub matched_pairs: u64,
    pub accounting: FailureResourceReport,
    pub synthesis: FamilyStratifiedSynthesisReport,
    /// Must remain false: the preflight never opens protected/final data.
    pub protected_or_final_access: bool,
    /// Must remain false: a smoke run carries no scientific claim.
    pub scientific_claim: bool,
    /// Must remain true.
    pub experimental_non_final: bool,
}

/// Run one bounded Stage-C preflight over every required family.
///
/// Composes slices 21-29 end to end: the complete matched evaluator matrix,
/// matched primary blocks accounted by arm, paired uncertainty by seed block,
/// and family-stratified synthesis. Any retained primary failure aborts the
/// preflight fail-closed rather than producing a partial report.
pub fn run_stage_c_preflight(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<StageCPreflightReport, EvalError> {
    validate_non_final_split(split)?;
    budget.validate()?;
    require_complete_primary_matrix().map_err(|_| EvalError::StageCPreflightInvalid {
        reason: "incomplete_primary_matrix",
    })?;
    let mut ledger = FailureResourceLedger::open(split)?;
    let mut t6_matches = Vec::new();
    let mut c6_matches = Vec::new();
    for family in REQUIRED_SYNTHESIS_FAMILIES {
        for seed_block in 0..budget.seed_blocks {
            if let Some(run) =
                ledger.account_matched_primary_block(*family, seed_block, budget.cases_per_block)?
            {
                t6_matches.extend_from_slice(run.t6_outcomes());
                c6_matches.extend_from_slice(run.c6_outcomes());
            }
        }
    }
    let accounting = ledger.report()?;
    require_complete_primary_accounting(&accounting)?;
    let synthesis = synthesize_family_stratified_from_revealed_outcomes(
        split,
        &t6_matches,
        &c6_matches,
        &MetricRegistry::pinned(),
    )?;
    let report = StageCPreflightReport {
        split,
        preflight_contract: STAGE_C_PREFLIGHT_CONTRACT,
        matrix_contract: MATCHED_EVALUATOR_MATRIX_CONTRACT,
        budget,
        matched_pairs: t6_matches.len() as u64,
        accounting,
        synthesis,
        protected_or_final_access: false,
        scientific_claim: false,
        experimental_non_final: true,
    };
    validate_stage_c_preflight_report(&report)?;
    Ok(report)
}

/// Parse a split label first, so protected/final labels never start a run.
pub fn run_stage_c_preflight_for_label(
    label: &str,
    budget: StageCPreflightBudget,
) -> Result<StageCPreflightReport, EvalError> {
    run_stage_c_preflight(parse_non_final_split(label)?, budget)
}

/// Validate a preflight report: pins, bounded budget, split agreement,
/// complete primary accounting, full family/seed-block coverage, and the
/// zero-access / no-claim invariants.
pub fn validate_stage_c_preflight_report(report: &StageCPreflightReport) -> Result<(), EvalError> {
    validate_non_final_split(report.split)?;
    let invalid = |reason| Err(EvalError::StageCPreflightInvalid { reason });
    if report.preflight_contract != STAGE_C_PREFLIGHT_CONTRACT {
        return invalid("preflight_contract");
    }
    if report.matrix_contract != MATCHED_EVALUATOR_MATRIX_CONTRACT {
        return invalid("matrix_contract");
    }
    report.budget.validate()?;
    if report.accounting.split != report.split || report.synthesis.split != report.split {
        return invalid("split_mismatch");
    }
    require_complete_primary_accounting(&report.accounting)?;
    let expected = report.budget.cases_per_arm();
    if report.matched_pairs != expected {
        return invalid("matched_pairs_mismatch");
    }
    for arm in [ComparisonArm::T6, ComparisonArm::C6] {
        let account = &report.accounting.arms[accounted_arm_index(arm)];
        if account.scored_cases != expected || account.attempted_cases() != expected {
            return invalid("scored_cases_mismatch");
        }
    }
    let families = REQUIRED_SYNTHESIS_FAMILIES.len();
    if report.synthesis.family_effects.len() != families
        || report.synthesis.seed_block_effects.len() as u64
            != families as u64 * report.budget.seed_blocks
    {
        return invalid("synthesis_coverage");
    }
    if report.synthesis.synthesis_contract != FAMILY_STRATIFIED_SYNTHESIS_CONTRACT {
        return invalid("synthesis_contract");
    }
    if report.protected_or_final_access {
        return Err(EvalError::StageCPreflightInvalid {
            reason: "protected_or_final_access",
        });
    }
    if report.scientific_claim {
        return Err(EvalError::StageCPreflightInvalid {
            reason: "scientific_claim",
        });
    }
    if !report.experimental_non_final || !report.synthesis.experimental_non_final {
        return invalid("experimental_non_final");
    }
    Ok(())
}

/// Per-family match counts for the T6 reference and its reduction-point ablation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReductionPointFamilySummary {
    pub family: TaskFamily,
    pub n_cases: u64,
    pub reference_matches: u64,
    pub ablated_matches: u64,
    /// Cases whose removed transport term is non-zero.
    pub transport_active_cases: u64,
}

/// Immutable torsor reduction-point ablation report on one non-final split.
#[derive(Clone, Debug, PartialEq)]
pub struct ReductionPointAblationReport {
    pub ablation_contract: &'static str,
    pub population_contract: &'static str,
    pub split: DataSplit,
    pub budget: StageCPreflightBudget,
    /// Identical capacity on both sides: the ablation removes no parameter
    /// and no input scalar, only the transport structure.
    pub reference_capacity: ParameterReadoutCapacity,
    pub ablated_capacity: ParameterReadoutCapacity,
    pub cases: Vec<ReductionPointAblationCase>,
    pub families: Vec<ReductionPointFamilySummary>,
    /// Must remain false.
    pub protected_or_final_access: bool,
    /// Must remain false: both sides are the non-trained reference.
    pub training_executed: bool,
    /// Must remain false: attribution is decided only by the Stage-D audit.
    pub scientific_claim: bool,
    /// Must remain true.
    pub experimental_non_final: bool,
}

const fn reduction_point_invalid(reason: &'static str) -> EvalError {
    EvalError::ReductionPointAblationInvalid { reason }
}

/// Monitored identity `reference = ablated + transport_term` under the shared
/// matched-reference tolerance.
#[must_use]
pub fn transport_identity_holds(reference: f64, ablated: f64, transport_term: f64) -> bool {
    let reconstructed = ablated + transport_term;
    reference.is_finite()
        && reconstructed.is_finite()
        && (reference - reconstructed).abs()
            <= TRANSPORT_IDENTITY_RELATIVE_TOLERANCE
                * (1.0 + reference.abs().max(reconstructed.abs()))
}

fn check_reduction_point_case(case: &ReductionPointAblationCase) -> Result<(), EvalError> {
    if !transport_identity_holds(
        case.reference_score,
        case.ablated_score,
        case.transport_term,
    ) {
        return Err(reduction_point_invalid("transport_residual"));
    }
    // A coincident reduction point removes nothing: the term is exactly zero.
    if case.reduction_points_coincide && case.transport_term != 0.0 {
        return Err(reduction_point_invalid("coincident_point_transport"));
    }
    Ok(())
}

fn summarize_reduction_point_families(
    cases: &[ReductionPointAblationCase],
) -> Vec<ReductionPointFamilySummary> {
    REQUIRED_SYNTHESIS_FAMILIES
        .iter()
        .map(|family| {
            let members = cases.iter().filter(|case| case.family == *family);
            ReductionPointFamilySummary {
                family: *family,
                n_cases: members.clone().count() as u64,
                reference_matches: members
                    .clone()
                    .filter(|c| c.reference_matches_target)
                    .count() as u64,
                ablated_matches: members.clone().filter(|c| c.ablated_matches_target).count()
                    as u64,
                transport_active_cases: members.filter(|c| c.transport_term != 0.0).count() as u64,
            }
        })
        .collect()
}

/// Run the torsor reduction-point ablation on the bounded matched population.
///
/// Same families, seed blocks, cases, split and capacity as the Stage-C
/// preflight; only the T6 transport structure is removed. Any scoring, drift
/// or identity failure aborts fail-closed; nothing is silently dropped.
pub fn run_torsor_reduction_point_ablation(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<ReductionPointAblationReport, EvalError> {
    validate_non_final_split(split)?;
    budget.validate()?;
    let mut cases = Vec::new();
    for family in REQUIRED_SYNTHESIS_FAMILIES {
        for seed_block in 0..budget.seed_blocks {
            cases.extend(evaluate_reduction_point_ablation(
                split,
                *family,
                seed_block,
                budget.cases_per_block,
            )?);
        }
    }
    let families = summarize_reduction_point_families(&cases);
    let report = ReductionPointAblationReport {
        ablation_contract: TORSOR_REDUCTION_POINT_ABLATION_CONTRACT,
        population_contract: matched_reference::MATCHED_POPULATION_CONTRACT,
        split,
        budget,
        reference_capacity: ParameterReadoutCapacity::reference_t6(),
        ablated_capacity: ParameterReadoutCapacity::reference_t6(),
        cases,
        families,
        protected_or_final_access: false,
        training_executed: false,
        scientific_claim: false,
        experimental_non_final: true,
    };
    validate_torsor_reduction_point_ablation_report(&report)?;
    Ok(report)
}

/// Parse a split label first; protected/final labels never generate a case.
pub fn run_torsor_reduction_point_ablation_for_label(
    split_label: &str,
    budget: StageCPreflightBudget,
) -> Result<ReductionPointAblationReport, EvalError> {
    run_torsor_reduction_point_ablation(parse_non_final_split(split_label)?, budget)
}

/// Validate a reduction-point ablation report: pins, matched capacity,
/// bounded coverage in canonical order, the monitored per-case transport
/// identity, recomputed family counts, and the no-access / no-training /
/// no-claim flags.
pub fn validate_torsor_reduction_point_ablation_report(
    report: &ReductionPointAblationReport,
) -> Result<(), EvalError> {
    validate_non_final_split(report.split)?;
    if report.ablation_contract != TORSOR_REDUCTION_POINT_ABLATION_CONTRACT {
        return Err(reduction_point_invalid("contract_drift"));
    }
    if report.population_contract != matched_reference::MATCHED_POPULATION_CONTRACT {
        return Err(reduction_point_invalid("population_drift"));
    }
    report.budget.validate()?;
    if report.reference_capacity != report.ablated_capacity
        || report.reference_capacity != ParameterReadoutCapacity::reference_t6()
    {
        return Err(reduction_point_invalid("capacity_mismatch"));
    }
    let expected = report.budget.cases_per_arm();
    if report.cases.len() as u64 != expected {
        return Err(reduction_point_invalid("case_count"));
    }
    let mut position = 0usize;
    for family in REQUIRED_SYNTHESIS_FAMILIES {
        for seed_block in 0..report.budget.seed_blocks {
            for case_id in 0..report.budget.cases_per_block {
                let case = &report.cases[position];
                if case.family != *family
                    || case.seed_block != seed_block
                    || case.case_id != case_id
                {
                    return Err(reduction_point_invalid("case_order"));
                }
                check_reduction_point_case(case)?;
                position += 1;
            }
        }
    }
    if report.families != summarize_reduction_point_families(&report.cases) {
        return Err(reduction_point_invalid("family_summary"));
    }
    if report.protected_or_final_access {
        return Err(reduction_point_invalid("protected_or_final_access"));
    }
    if report.training_executed {
        return Err(reduction_point_invalid("training_executed"));
    }
    if report.scientific_claim {
        return Err(reduction_point_invalid("scientific_claim"));
    }
    if !report.experimental_non_final {
        return Err(reduction_point_invalid("experimental_non_final"));
    }
    Ok(())
}

/// Per-family counts for the direct vs factorized torsor bridge monitor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TorsorBridgeFamilySummary {
    pub family: TaskFamily,
    pub n_cases: u64,
    pub factorized_matches: u64,
    pub direct_matches: u64,
    /// Cases whose two bridge forms agree bit for bit.
    pub bit_identical_cases: u64,
    /// Largest monitored `|factorized - direct|` in the family.
    pub max_abs_residual: f64,
}

/// Immutable direct vs factorized torsor bridge report on one non-final split.
#[derive(Clone, Debug, PartialEq)]
pub struct TorsorBridgeEquivalenceReport {
    pub bridge_contract: &'static str,
    pub population_contract: &'static str,
    pub torsor_contract: &'static str,
    pub split: DataSplit,
    pub budget: StageCPreflightBudget,
    /// Declared monitor tolerance; must equal
    /// [`BRIDGE_EQUIVALENCE_RELATIVE_TOLERANCE`].
    pub relative_tolerance: f64,
    /// Identical capacity on both sides: the two forms are one T6 score.
    pub factorized_capacity: ParameterReadoutCapacity,
    pub direct_capacity: ParameterReadoutCapacity,
    pub cases: Vec<TorsorBridgeCase>,
    pub families: Vec<TorsorBridgeFamilySummary>,
    /// Must remain false.
    pub protected_or_final_access: bool,
    /// Must remain false: both sides are the non-trained reference.
    pub training_executed: bool,
    /// Must remain false: attribution is decided only by the Stage-D audit.
    pub scientific_claim: bool,
    /// Must remain true.
    pub experimental_non_final: bool,
}

const fn bridge_invalid(reason: &'static str) -> EvalError {
    EvalError::TorsorBridgeEquivalenceInvalid { reason }
}

/// Monitored equivalence `factorized = direct` under the shared
/// matched-reference v1 relative tolerance.
#[must_use]
pub fn bridge_equivalence_holds(factorized: f64, direct: f64) -> bool {
    factorized.is_finite()
        && direct.is_finite()
        && (factorized - direct).abs()
            <= BRIDGE_EQUIVALENCE_RELATIVE_TOLERANCE * (1.0 + factorized.abs().max(direct.abs()))
}

fn check_torsor_bridge_case(case: &TorsorBridgeCase) -> Result<(), EvalError> {
    if !bridge_equivalence_holds(case.factorized_score, case.direct_score) {
        return Err(bridge_invalid("bridge_residual"));
    }
    if case.residual.to_bits() != (case.factorized_score - case.direct_score).to_bits() {
        return Err(bridge_invalid("residual_drift"));
    }
    if case.bit_identical != (case.factorized_score.to_bits() == case.direct_score.to_bits()) {
        return Err(bridge_invalid("bit_identity_drift"));
    }
    // Identical scores against one common target must share their match bit.
    if case.bit_identical && case.factorized_matches_target != case.direct_matches_target {
        return Err(bridge_invalid("match_bit_drift"));
    }
    Ok(())
}

fn summarize_torsor_bridge_families(cases: &[TorsorBridgeCase]) -> Vec<TorsorBridgeFamilySummary> {
    REQUIRED_SYNTHESIS_FAMILIES
        .iter()
        .map(|family| {
            let members = cases.iter().filter(|case| case.family == *family);
            TorsorBridgeFamilySummary {
                family: *family,
                n_cases: members.clone().count() as u64,
                factorized_matches: members
                    .clone()
                    .filter(|c| c.factorized_matches_target)
                    .count() as u64,
                direct_matches: members.clone().filter(|c| c.direct_matches_target).count() as u64,
                bit_identical_cases: members.clone().filter(|c| c.bit_identical).count() as u64,
                max_abs_residual: members.fold(0.0, |max: f64, c| max.max(c.residual.abs())),
            }
        })
        .collect()
}

/// Run the direct vs factorized torsor bridge monitor on the bounded matched
/// population.
///
/// Same families, seed blocks, cases, split and capacity as the Stage-C
/// preflight; each T6 case is scored through both upstream TDI-22 forms and
/// their residual is monitored. Any scoring, drift or equivalence failure
/// aborts fail-closed; nothing is silently dropped.
pub fn run_direct_vs_factorized_bridge(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<TorsorBridgeEquivalenceReport, EvalError> {
    validate_non_final_split(split)?;
    budget.validate()?;
    let mut cases = Vec::new();
    for family in REQUIRED_SYNTHESIS_FAMILIES {
        for seed_block in 0..budget.seed_blocks {
            cases.extend(evaluate_torsor_bridge_equivalence(
                split,
                *family,
                seed_block,
                budget.cases_per_block,
            )?);
        }
    }
    let families = summarize_torsor_bridge_families(&cases);
    let report = TorsorBridgeEquivalenceReport {
        bridge_contract: DIRECT_VS_FACTORIZED_BRIDGE_CONTRACT,
        population_contract: matched_reference::MATCHED_POPULATION_CONTRACT,
        torsor_contract: TORSOR_CONTRACT,
        split,
        budget,
        relative_tolerance: BRIDGE_EQUIVALENCE_RELATIVE_TOLERANCE,
        factorized_capacity: ParameterReadoutCapacity::reference_t6(),
        direct_capacity: ParameterReadoutCapacity::reference_t6(),
        cases,
        families,
        protected_or_final_access: false,
        training_executed: false,
        scientific_claim: false,
        experimental_non_final: true,
    };
    validate_direct_vs_factorized_bridge_report(&report)?;
    Ok(report)
}

/// Parse a split label first; protected/final labels never generate a case.
pub fn run_direct_vs_factorized_bridge_for_label(
    split_label: &str,
    budget: StageCPreflightBudget,
) -> Result<TorsorBridgeEquivalenceReport, EvalError> {
    run_direct_vs_factorized_bridge(parse_non_final_split(split_label)?, budget)
}

/// Validate a direct vs factorized bridge report: pins, declared tolerance,
/// matched capacity, bounded coverage in canonical order, the monitored
/// per-case residual, recomputed family summaries, and the no-access /
/// no-training / no-claim flags.
pub fn validate_direct_vs_factorized_bridge_report(
    report: &TorsorBridgeEquivalenceReport,
) -> Result<(), EvalError> {
    validate_non_final_split(report.split)?;
    if report.bridge_contract != DIRECT_VS_FACTORIZED_BRIDGE_CONTRACT {
        return Err(bridge_invalid("contract_drift"));
    }
    if report.population_contract != matched_reference::MATCHED_POPULATION_CONTRACT {
        return Err(bridge_invalid("population_drift"));
    }
    if report.torsor_contract != TORSOR_CONTRACT {
        return Err(bridge_invalid("torsor_contract_drift"));
    }
    if report.relative_tolerance.to_bits() != BRIDGE_EQUIVALENCE_RELATIVE_TOLERANCE.to_bits() {
        return Err(bridge_invalid("tolerance_drift"));
    }
    report.budget.validate()?;
    if report.factorized_capacity != report.direct_capacity
        || report.factorized_capacity != ParameterReadoutCapacity::reference_t6()
    {
        return Err(bridge_invalid("capacity_mismatch"));
    }
    let expected = report.budget.cases_per_arm();
    if report.cases.len() as u64 != expected {
        return Err(bridge_invalid("case_count"));
    }
    let mut position = 0usize;
    for family in REQUIRED_SYNTHESIS_FAMILIES {
        for seed_block in 0..report.budget.seed_blocks {
            for case_id in 0..report.budget.cases_per_block {
                let case = &report.cases[position];
                if case.family != *family
                    || case.seed_block != seed_block
                    || case.case_id != case_id
                {
                    return Err(bridge_invalid("case_order"));
                }
                check_torsor_bridge_case(case)?;
                position += 1;
            }
        }
    }
    if report.families != summarize_torsor_bridge_families(&report.cases) {
        return Err(bridge_invalid("family_summary"));
    }
    if report.protected_or_final_access {
        return Err(bridge_invalid("protected_or_final_access"));
    }
    if report.training_executed {
        return Err(bridge_invalid("training_executed"));
    }
    if report.scientific_claim {
        return Err(bridge_invalid("scientific_claim"));
    }
    if !report.experimental_non_final {
        return Err(bridge_invalid("experimental_non_final"));
    }
    Ok(())
}

/// Per-family match counts for the C6 reference and its `gamma=0` ablation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChiralGammaZeroFamilySummary {
    pub family: TaskFamily,
    pub n_cases: u64,
    pub reference_matches: u64,
    pub ablated_matches: u64,
    /// Cases whose removed parity-odd observable is non-zero.
    pub parity_odd_active_cases: u64,
}

/// Immutable chiral `gamma=0` ablation report on one non-final split.
#[derive(Clone, Debug, PartialEq)]
pub struct ChiralGammaZeroAblationReport {
    pub ablation_contract: &'static str,
    pub population_contract: &'static str,
    pub chiral_contract: &'static str,
    pub split: DataSplit,
    pub budget: StageCPreflightBudget,
    pub reference_weights: ChiralScoreWeights,
    pub ablated_weights: ChiralScoreWeights,
    /// Identical capacity on both sides: the ablation removes no parameter
    /// and no input scalar, only the parity-odd coefficient.
    pub reference_capacity: ParameterReadoutCapacity,
    pub ablated_capacity: ParameterReadoutCapacity,
    pub cases: Vec<ChiralGammaZeroAblationCase>,
    pub families: Vec<ChiralGammaZeroFamilySummary>,
    /// Must remain false.
    pub protected_or_final_access: bool,
    /// Must remain false: both sides are the non-trained reference.
    pub training_executed: bool,
    /// Must remain false: attribution is decided only by the Stage-D audit.
    pub scientific_claim: bool,
    /// Must remain true.
    pub experimental_non_final: bool,
}

const fn chiral_gamma_zero_invalid(reason: &'static str) -> EvalError {
    EvalError::ChiralGammaZeroAblationInvalid { reason }
}

fn check_chiral_gamma_zero_case(
    case: &ChiralGammaZeroAblationCase,
    reference_weights: ChiralScoreWeights,
) -> Result<(), EvalError> {
    // Exact: both sides share the identical alpha*s + beta*m accumulation, so
    // the parity-odd term is the only difference between the two scores.
    if case.reference_score
        != case.ablated_score + reference_weights.gamma * case.parity_odd_channel
    {
        return Err(chiral_gamma_zero_invalid("parity_odd_residual"));
    }
    if !case.enantiomorphic_split_closed {
        return Err(chiral_gamma_zero_invalid("enantiomorphic_split"));
    }
    // A vanishing parity-odd observable removes nothing.
    if case.parity_odd_channel == 0.0
        && (case.reference_score != case.ablated_score
            || case.reference_matches_target != case.ablated_matches_target)
    {
        return Err(chiral_gamma_zero_invalid("inactive_channel_drift"));
    }
    Ok(())
}

fn summarize_chiral_gamma_zero_families(
    cases: &[ChiralGammaZeroAblationCase],
) -> Vec<ChiralGammaZeroFamilySummary> {
    REQUIRED_SYNTHESIS_FAMILIES
        .iter()
        .map(|family| {
            let members = cases.iter().filter(|case| case.family == *family);
            ChiralGammaZeroFamilySummary {
                family: *family,
                n_cases: members.clone().count() as u64,
                reference_matches: members
                    .clone()
                    .filter(|c| c.reference_matches_target)
                    .count() as u64,
                ablated_matches: members.clone().filter(|c| c.ablated_matches_target).count()
                    as u64,
                parity_odd_active_cases: members.filter(|c| c.parity_odd_channel != 0.0).count()
                    as u64,
            }
        })
        .collect()
}

/// Run the chiral `gamma=0` ablation on the bounded matched population.
///
/// Same families, seed blocks, cases, split and capacity as the Stage-C
/// preflight; only the C6 parity-odd coefficient is zeroed. Any scoring,
/// drift or identity failure aborts fail-closed; nothing is silently dropped.
pub fn run_chiral_gamma_zero_ablation(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<ChiralGammaZeroAblationReport, EvalError> {
    validate_non_final_split(split)?;
    budget.validate()?;
    let mut cases = Vec::new();
    for family in REQUIRED_SYNTHESIS_FAMILIES {
        for seed_block in 0..budget.seed_blocks {
            cases.extend(evaluate_chiral_gamma_zero_ablation(
                split,
                *family,
                seed_block,
                budget.cases_per_block,
            )?);
        }
    }
    let families = summarize_chiral_gamma_zero_families(&cases);
    let report = ChiralGammaZeroAblationReport {
        ablation_contract: CHIRAL_GAMMA_ZERO_ABLATION_CONTRACT,
        population_contract: matched_reference::MATCHED_POPULATION_CONTRACT,
        chiral_contract: CHIRAL_CONTRACT,
        split,
        budget,
        reference_weights: matched_reference::MATCHED_CHIRAL_WEIGHTS,
        ablated_weights: chiral_gamma_zero_weights(matched_reference::MATCHED_CHIRAL_WEIGHTS),
        reference_capacity: ParameterReadoutCapacity::reference_c6(),
        ablated_capacity: ParameterReadoutCapacity::reference_c6(),
        cases,
        families,
        protected_or_final_access: false,
        training_executed: false,
        scientific_claim: false,
        experimental_non_final: true,
    };
    validate_chiral_gamma_zero_ablation_report(&report)?;
    Ok(report)
}

/// Parse a split label first; protected/final labels never generate a case.
pub fn run_chiral_gamma_zero_ablation_for_label(
    split_label: &str,
    budget: StageCPreflightBudget,
) -> Result<ChiralGammaZeroAblationReport, EvalError> {
    run_chiral_gamma_zero_ablation(parse_non_final_split(split_label)?, budget)
}

/// Validate a chiral `gamma=0` ablation report: pins, all-else-fixed weights,
/// matched capacity, bounded coverage in canonical order, the exact per-case
/// parity-odd residual and closed enantiomorphic split, recomputed family
/// counts, and the no-access / no-training / no-claim flags.
pub fn validate_chiral_gamma_zero_ablation_report(
    report: &ChiralGammaZeroAblationReport,
) -> Result<(), EvalError> {
    validate_non_final_split(report.split)?;
    if report.ablation_contract != CHIRAL_GAMMA_ZERO_ABLATION_CONTRACT {
        return Err(chiral_gamma_zero_invalid("contract_drift"));
    }
    if report.population_contract != matched_reference::MATCHED_POPULATION_CONTRACT {
        return Err(chiral_gamma_zero_invalid("population_drift"));
    }
    if report.chiral_contract != CHIRAL_CONTRACT {
        return Err(chiral_gamma_zero_invalid("chiral_contract_drift"));
    }
    if report.reference_weights != matched_reference::MATCHED_CHIRAL_WEIGHTS {
        return Err(chiral_gamma_zero_invalid("reference_weights_drift"));
    }
    if report.ablated_weights != chiral_gamma_zero_weights(report.reference_weights) {
        return Err(chiral_gamma_zero_invalid("ablated_weights_drift"));
    }
    report.budget.validate()?;
    if report.reference_capacity != report.ablated_capacity
        || report.reference_capacity != ParameterReadoutCapacity::reference_c6()
    {
        return Err(chiral_gamma_zero_invalid("capacity_mismatch"));
    }
    let expected = report.budget.cases_per_arm();
    if report.cases.len() as u64 != expected {
        return Err(chiral_gamma_zero_invalid("case_count"));
    }
    let mut position = 0usize;
    for family in REQUIRED_SYNTHESIS_FAMILIES {
        for seed_block in 0..report.budget.seed_blocks {
            for case_id in 0..report.budget.cases_per_block {
                let case = &report.cases[position];
                if case.family != *family
                    || case.seed_block != seed_block
                    || case.case_id != case_id
                {
                    return Err(chiral_gamma_zero_invalid("case_order"));
                }
                check_chiral_gamma_zero_case(case, report.reference_weights)?;
                position += 1;
            }
        }
    }
    if report.families != summarize_chiral_gamma_zero_families(&report.cases) {
        return Err(chiral_gamma_zero_invalid("family_summary"));
    }
    if report.protected_or_final_access {
        return Err(chiral_gamma_zero_invalid("protected_or_final_access"));
    }
    if report.training_executed {
        return Err(chiral_gamma_zero_invalid("training_executed"));
    }
    if report.scientific_claim {
        return Err(chiral_gamma_zero_invalid("scientific_claim"));
    }
    if !report.experimental_non_final {
        return Err(chiral_gamma_zero_invalid("experimental_non_final"));
    }
    // Stored evidence is never trusted: every case is regenerated from the
    // canonical matched population `(split, family, seed_block, case_id)` and
    // its evaluator-owned target, and compared exactly.
    let mut regenerated = Vec::with_capacity(report.cases.len());
    for family in REQUIRED_SYNTHESIS_FAMILIES {
        for seed_block in 0..report.budget.seed_blocks {
            regenerated.extend(evaluate_chiral_gamma_zero_ablation(
                report.split,
                *family,
                seed_block,
                report.budget.cases_per_block,
            )?);
        }
    }
    if regenerated != report.cases {
        return Err(chiral_gamma_zero_invalid("case_evidence_drift"));
    }
    Ok(())
}

/// Per-family match counts for the C6 reference and its parity-shuffle control.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChiralParityShuffleFamilySummary {
    pub family: TaskFamily,
    pub n_cases: u64,
    pub reference_matches: u64,
    pub shuffled_matches: u64,
    /// Cases whose parity-odd observable changed under the shuffle.
    pub parity_odd_changed: u64,
}

/// Immutable chiral parity-shuffle control report on one non-final split.
#[derive(Clone, Debug, PartialEq)]
pub struct ChiralParityShuffleControlReport {
    pub control_contract: &'static str,
    pub population_contract: &'static str,
    pub chiral_contract: &'static str,
    /// Upstream TDI-24 slice-34 shuffle contract, consumed unchanged.
    pub shuffle_contract: &'static str,
    pub split: DataSplit,
    pub budget: StageCPreflightBudget,
    /// Reused registered seed `(split domain, ChiralFavorable, 0)`: the seed of
    /// the first ChiralFavorable case of the matched population; no new seed
    /// material.
    pub registered_seed: RegisteredSeed,
    pub shuffle: ParityShuffle,
    /// Identical weights on both sides: the control changes no coefficient.
    pub reference_weights: ChiralScoreWeights,
    pub shuffled_weights: ChiralScoreWeights,
    /// Identical capacity on both sides: the shuffle is a fixed relabelling
    /// with zero trainable parameters.
    pub reference_capacity: ParameterReadoutCapacity,
    pub shuffled_capacity: ParameterReadoutCapacity,
    pub cases: Vec<ChiralParityShuffleCase>,
    pub families: Vec<ChiralParityShuffleFamilySummary>,
    /// Must remain false.
    pub protected_or_final_access: bool,
    /// Must remain false: both sides are the non-trained reference.
    pub training_executed: bool,
    /// Must remain false: attribution is decided only by the Stage-D audit.
    pub scientific_claim: bool,
    /// Must remain true.
    pub experimental_non_final: bool,
}

const fn chiral_parity_shuffle_invalid(reason: &'static str) -> EvalError {
    EvalError::ChiralParityShuffleControlInvalid { reason }
}

/// Registered seed reused by the parity-shuffle control (no new seed material).
#[must_use]
pub fn chiral_parity_shuffle_registered_seed(split: DataSplit) -> RegisteredSeed {
    register_seed(
        SeedDomain::from_split(split),
        TaskFamily::ChiralFavorable,
        0,
    )
}

fn chiral_parity_shuffle_for_split(split: DataSplit) -> Result<ParityShuffle, EvalError> {
    parity_shuffle_from_seed(chiral_parity_shuffle_registered_seed(split).mixed_seed).map_err(
        |error| match error {
            super::tdi24_eval::EvalError::ParityShuffleControlInvalid { reason } => {
                chiral_parity_shuffle_invalid(reason)
            }
            _ => chiral_parity_shuffle_invalid("upstream_shuffle_invalid"),
        },
    )
}

fn summarize_chiral_parity_shuffle_families(
    cases: &[ChiralParityShuffleCase],
) -> Vec<ChiralParityShuffleFamilySummary> {
    REQUIRED_SYNTHESIS_FAMILIES
        .iter()
        .map(|family| {
            let members = cases.iter().filter(|case| case.family == *family);
            ChiralParityShuffleFamilySummary {
                family: *family,
                n_cases: members.clone().count() as u64,
                reference_matches: members
                    .clone()
                    .filter(|c| c.reference_matches_target)
                    .count() as u64,
                shuffled_matches: members
                    .clone()
                    .filter(|c| c.shuffled_matches_target)
                    .count() as u64,
                parity_odd_changed: members
                    .filter(|c| c.reference_parity_odd.to_bits() != c.shuffled_parity_odd.to_bits())
                    .count() as u64,
            }
        })
        .collect()
}

fn collect_chiral_parity_shuffle_cases(
    split: DataSplit,
    budget: StageCPreflightBudget,
    shuffle: &ParityShuffle,
) -> Result<Vec<ChiralParityShuffleCase>, EvalError> {
    let mut cases = Vec::new();
    for family in REQUIRED_SYNTHESIS_FAMILIES {
        for seed_block in 0..budget.seed_blocks {
            cases.extend(evaluate_chiral_parity_shuffle_control(
                split,
                *family,
                seed_block,
                budget.cases_per_block,
                shuffle,
            )?);
        }
    }
    Ok(cases)
}

/// Run the chiral parity-shuffle control on the bounded matched population.
///
/// Same families, seed blocks, cases, split, weights and capacity as the
/// Stage-C preflight; only the carrier slots are relabelled by the unchanged
/// TDI-24 slice-34 shuffle drawn from a reused registered seed. Any scoring,
/// drift or identity failure aborts fail-closed; nothing is silently dropped.
pub fn run_chiral_parity_shuffle_control(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<ChiralParityShuffleControlReport, EvalError> {
    validate_non_final_split(split)?;
    budget.validate()?;
    let shuffle = chiral_parity_shuffle_for_split(split)?;
    let cases = collect_chiral_parity_shuffle_cases(split, budget, &shuffle)?;
    let families = summarize_chiral_parity_shuffle_families(&cases);
    let report = ChiralParityShuffleControlReport {
        control_contract: CHIRAL_PARITY_SHUFFLE_CONTROL_CONTRACT,
        population_contract: matched_reference::MATCHED_POPULATION_CONTRACT,
        chiral_contract: CHIRAL_CONTRACT,
        shuffle_contract: PARITY_SHUFFLE_CONTROL_CONTRACT,
        split,
        budget,
        registered_seed: chiral_parity_shuffle_registered_seed(split),
        shuffle,
        reference_weights: matched_reference::MATCHED_CHIRAL_WEIGHTS,
        shuffled_weights: matched_reference::MATCHED_CHIRAL_WEIGHTS,
        reference_capacity: ParameterReadoutCapacity::reference_c6(),
        shuffled_capacity: ParameterReadoutCapacity::reference_c6(),
        cases,
        families,
        protected_or_final_access: false,
        training_executed: false,
        scientific_claim: false,
        experimental_non_final: true,
    };
    validate_chiral_parity_shuffle_control_report(&report)?;
    Ok(report)
}

/// Parse a split label first; protected/final labels never generate a case.
pub fn run_chiral_parity_shuffle_control_for_label(
    split_label: &str,
    budget: StageCPreflightBudget,
) -> Result<ChiralParityShuffleControlReport, EvalError> {
    run_chiral_parity_shuffle_control(parse_non_final_split(split_label)?, budget)
}

/// Validate a chiral parity-shuffle control report: pins, reused seed and
/// reproducible structure-destroying shuffle, identical weights and capacity,
/// bounded coverage in canonical order, preserved six values and
/// direct-product multiset, recomputed family counts, the no-access /
/// no-training / no-claim flags, and regenerated per-case evidence.
pub fn validate_chiral_parity_shuffle_control_report(
    report: &ChiralParityShuffleControlReport,
) -> Result<(), EvalError> {
    validate_non_final_split(report.split)?;
    if report.control_contract != CHIRAL_PARITY_SHUFFLE_CONTROL_CONTRACT {
        return Err(chiral_parity_shuffle_invalid("contract_drift"));
    }
    if report.population_contract != matched_reference::MATCHED_POPULATION_CONTRACT {
        return Err(chiral_parity_shuffle_invalid("population_drift"));
    }
    if report.chiral_contract != CHIRAL_CONTRACT {
        return Err(chiral_parity_shuffle_invalid("chiral_contract_drift"));
    }
    if report.shuffle_contract != PARITY_SHUFFLE_CONTROL_CONTRACT {
        return Err(chiral_parity_shuffle_invalid("shuffle_contract_drift"));
    }
    if report.registered_seed != chiral_parity_shuffle_registered_seed(report.split) {
        return Err(chiral_parity_shuffle_invalid("seed_drift"));
    }
    super::tdi24_eval::validate_parity_shuffle(&report.shuffle).map_err(|error| match error {
        super::tdi24_eval::EvalError::ParityShuffleControlInvalid { reason } => {
            chiral_parity_shuffle_invalid(reason)
        }
        _ => chiral_parity_shuffle_invalid("upstream_shuffle_invalid"),
    })?;
    if report.shuffle != chiral_parity_shuffle_for_split(report.split)? {
        return Err(chiral_parity_shuffle_invalid("shuffle_not_reproducible"));
    }
    if report.reference_weights != matched_reference::MATCHED_CHIRAL_WEIGHTS {
        return Err(chiral_parity_shuffle_invalid("reference_weights_drift"));
    }
    if report.shuffled_weights != report.reference_weights {
        return Err(chiral_parity_shuffle_invalid("shuffled_weights_drift"));
    }
    report.budget.validate()?;
    if report.reference_capacity != report.shuffled_capacity
        || report.reference_capacity != ParameterReadoutCapacity::reference_c6()
    {
        return Err(chiral_parity_shuffle_invalid("capacity_mismatch"));
    }
    let expected = report.budget.cases_per_arm();
    if report.cases.len() as u64 != expected {
        return Err(chiral_parity_shuffle_invalid("case_count"));
    }
    let mut position = 0usize;
    for family in REQUIRED_SYNTHESIS_FAMILIES {
        for seed_block in 0..report.budget.seed_blocks {
            for case_id in 0..report.budget.cases_per_block {
                let case = &report.cases[position];
                if case.family != *family
                    || case.seed_block != seed_block
                    || case.case_id != case_id
                {
                    return Err(chiral_parity_shuffle_invalid("case_order"));
                }
                if !case.six_values_preserved {
                    return Err(chiral_parity_shuffle_invalid("six_values_drift"));
                }
                if !case.direct_products_preserved {
                    return Err(chiral_parity_shuffle_invalid(
                        "direct_product_multiset_drift",
                    ));
                }
                position += 1;
            }
        }
    }
    if report.families != summarize_chiral_parity_shuffle_families(&report.cases) {
        return Err(chiral_parity_shuffle_invalid("family_summary"));
    }
    if report.protected_or_final_access {
        return Err(chiral_parity_shuffle_invalid("protected_or_final_access"));
    }
    if report.training_executed {
        return Err(chiral_parity_shuffle_invalid("training_executed"));
    }
    if report.scientific_claim {
        return Err(chiral_parity_shuffle_invalid("scientific_claim"));
    }
    if !report.experimental_non_final {
        return Err(chiral_parity_shuffle_invalid("experimental_non_final"));
    }
    // Stored evidence is never trusted: every case is regenerated from the
    // canonical matched population `(split, family, seed_block, case_id)`, its
    // evaluator-owned target and the reproducible shuffle, and compared exactly.
    let regenerated =
        collect_chiral_parity_shuffle_cases(report.split, report.budget, &report.shuffle)?;
    if regenerated != report.cases {
        return Err(chiral_parity_shuffle_invalid("case_evidence_drift"));
    }
    Ok(())
}

/// Per-family match counts for the T6 reference and its structure-shuffle control.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TorsorStructureShuffleFamilySummary {
    pub family: TaskFamily,
    pub n_cases: u64,
    pub reference_matches: u64,
    pub shuffled_matches: u64,
    /// Cases whose Varignon transport term changed under the shuffle.
    pub transport_changed: u64,
}

/// Immutable torsor structure-shuffle control report on one non-final split.
#[derive(Clone, Debug, PartialEq)]
pub struct TorsorStructureShuffleControlReport {
    pub control_contract: &'static str,
    pub population_contract: &'static str,
    pub torsor_contract: &'static str,
    /// Upstream TDI-24 slice-34 shuffle contract, consumed unchanged.
    pub shuffle_contract: &'static str,
    pub split: DataSplit,
    pub budget: StageCPreflightBudget,
    /// Reused registered seed `(split domain, TorsorFavorable, 0)`: the seed of
    /// the first TorsorFavorable case of the matched population; no new seed
    /// material.
    pub registered_seed: RegisteredSeed,
    pub shuffle: ParityShuffle,
    /// Identical capacity on both sides: the shuffle is a fixed relabelling
    /// with zero trainable parameters.
    pub reference_capacity: ParameterReadoutCapacity,
    pub shuffled_capacity: ParameterReadoutCapacity,
    pub cases: Vec<TorsorStructureShuffleCase>,
    pub families: Vec<TorsorStructureShuffleFamilySummary>,
    /// Must remain false.
    pub protected_or_final_access: bool,
    /// Must remain false: both sides are the non-trained reference.
    pub training_executed: bool,
    /// Must remain false: attribution is decided only by the Stage-D audit.
    pub scientific_claim: bool,
    /// Must remain true.
    pub experimental_non_final: bool,
}

const fn torsor_structure_shuffle_invalid(reason: &'static str) -> EvalError {
    EvalError::TorsorStructureShuffleControlInvalid { reason }
}

/// Registered seed reused by the structure-shuffle control (no new seed material).
#[must_use]
pub fn torsor_structure_shuffle_registered_seed(split: DataSplit) -> RegisteredSeed {
    register_seed(
        SeedDomain::from_split(split),
        TaskFamily::TorsorFavorable,
        0,
    )
}

fn torsor_structure_shuffle_for_split(split: DataSplit) -> Result<ParityShuffle, EvalError> {
    parity_shuffle_from_seed(torsor_structure_shuffle_registered_seed(split).mixed_seed).map_err(
        |error| match error {
            super::tdi24_eval::EvalError::ParityShuffleControlInvalid { reason } => {
                torsor_structure_shuffle_invalid(reason)
            }
            _ => torsor_structure_shuffle_invalid("upstream_shuffle_invalid"),
        },
    )
}

fn summarize_torsor_structure_shuffle_families(
    cases: &[TorsorStructureShuffleCase],
) -> Vec<TorsorStructureShuffleFamilySummary> {
    REQUIRED_SYNTHESIS_FAMILIES
        .iter()
        .map(|family| {
            let members = cases.iter().filter(|case| case.family == *family);
            TorsorStructureShuffleFamilySummary {
                family: *family,
                n_cases: members.clone().count() as u64,
                reference_matches: members
                    .clone()
                    .filter(|c| c.reference_matches_target)
                    .count() as u64,
                shuffled_matches: members
                    .clone()
                    .filter(|c| c.shuffled_matches_target)
                    .count() as u64,
                transport_changed: members
                    .filter(|c| {
                        // Numeric change only: a signed-zero flip is not a change.
                        (c.reference_transport_term - c.shuffled_transport_term).abs() > 0.0
                    })
                    .count() as u64,
            }
        })
        .collect()
}

fn collect_torsor_structure_shuffle_cases(
    split: DataSplit,
    budget: StageCPreflightBudget,
    shuffle: &ParityShuffle,
) -> Result<Vec<TorsorStructureShuffleCase>, EvalError> {
    let mut cases = Vec::new();
    for family in REQUIRED_SYNTHESIS_FAMILIES {
        for seed_block in 0..budget.seed_blocks {
            cases.extend(evaluate_torsor_structure_shuffle_control(
                split,
                *family,
                seed_block,
                budget.cases_per_block,
                shuffle,
            )?);
        }
    }
    Ok(cases)
}

/// Run the torsor structure-shuffle control on the bounded matched population.
///
/// Same families, seed blocks, cases, split, and capacity as the
/// Stage-C preflight; only the carrier slots are relabelled by the unchanged
/// TDI-24 slice-34 shuffle drawn from a reused registered seed. Any scoring,
/// drift or identity failure aborts fail-closed; nothing is silently dropped.
pub fn run_torsor_structure_shuffle_control(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<TorsorStructureShuffleControlReport, EvalError> {
    validate_non_final_split(split)?;
    budget.validate()?;
    let shuffle = torsor_structure_shuffle_for_split(split)?;
    let cases = collect_torsor_structure_shuffle_cases(split, budget, &shuffle)?;
    let families = summarize_torsor_structure_shuffle_families(&cases);
    let report = TorsorStructureShuffleControlReport {
        control_contract: TORSOR_STRUCTURE_SHUFFLE_CONTROL_CONTRACT,
        population_contract: matched_reference::MATCHED_POPULATION_CONTRACT,
        torsor_contract: TORSOR_CONTRACT,
        shuffle_contract: PARITY_SHUFFLE_CONTROL_CONTRACT,
        split,
        budget,
        registered_seed: torsor_structure_shuffle_registered_seed(split),
        shuffle,
        reference_capacity: ParameterReadoutCapacity::reference_t6(),
        shuffled_capacity: ParameterReadoutCapacity::reference_t6(),
        cases,
        families,
        protected_or_final_access: false,
        training_executed: false,
        scientific_claim: false,
        experimental_non_final: true,
    };
    validate_torsor_structure_shuffle_control_report(&report)?;
    Ok(report)
}

/// Parse a split label first; protected/final labels never generate a case.
pub fn run_torsor_structure_shuffle_control_for_label(
    split_label: &str,
    budget: StageCPreflightBudget,
) -> Result<TorsorStructureShuffleControlReport, EvalError> {
    run_torsor_structure_shuffle_control(parse_non_final_split(split_label)?, budget)
}

/// Validate a torsor structure-shuffle control report: pins, reused seed and
/// reproducible structure-destroying shuffle, identical and capacity,
/// bounded coverage in canonical order, preserved six values and
/// direct-product multiset, recomputed family counts, the no-access /
/// no-training / no-claim flags, and regenerated per-case evidence.
pub fn validate_torsor_structure_shuffle_control_report(
    report: &TorsorStructureShuffleControlReport,
) -> Result<(), EvalError> {
    validate_non_final_split(report.split)?;
    if report.control_contract != TORSOR_STRUCTURE_SHUFFLE_CONTROL_CONTRACT {
        return Err(torsor_structure_shuffle_invalid("contract_drift"));
    }
    if report.population_contract != matched_reference::MATCHED_POPULATION_CONTRACT {
        return Err(torsor_structure_shuffle_invalid("population_drift"));
    }
    if report.torsor_contract != TORSOR_CONTRACT {
        return Err(torsor_structure_shuffle_invalid("torsor_contract_drift"));
    }
    if report.shuffle_contract != PARITY_SHUFFLE_CONTROL_CONTRACT {
        return Err(torsor_structure_shuffle_invalid("shuffle_contract_drift"));
    }
    if report.registered_seed != torsor_structure_shuffle_registered_seed(report.split) {
        return Err(torsor_structure_shuffle_invalid("seed_drift"));
    }
    super::tdi24_eval::validate_parity_shuffle(&report.shuffle).map_err(|error| match error {
        super::tdi24_eval::EvalError::ParityShuffleControlInvalid { reason } => {
            torsor_structure_shuffle_invalid(reason)
        }
        _ => torsor_structure_shuffle_invalid("upstream_shuffle_invalid"),
    })?;
    if report.shuffle != torsor_structure_shuffle_for_split(report.split)? {
        return Err(torsor_structure_shuffle_invalid("shuffle_not_reproducible"));
    }
    report.budget.validate()?;
    if report.reference_capacity != report.shuffled_capacity
        || report.reference_capacity != ParameterReadoutCapacity::reference_t6()
    {
        return Err(torsor_structure_shuffle_invalid("capacity_mismatch"));
    }
    let expected = report.budget.cases_per_arm();
    if report.cases.len() as u64 != expected {
        return Err(torsor_structure_shuffle_invalid("case_count"));
    }
    let mut position = 0usize;
    for family in REQUIRED_SYNTHESIS_FAMILIES {
        for seed_block in 0..report.budget.seed_blocks {
            for case_id in 0..report.budget.cases_per_block {
                let case = &report.cases[position];
                if case.family != *family
                    || case.seed_block != seed_block
                    || case.case_id != case_id
                {
                    return Err(torsor_structure_shuffle_invalid("case_order"));
                }
                if !case.six_values_preserved {
                    return Err(torsor_structure_shuffle_invalid("six_values_drift"));
                }
                if !case.untransported_products_preserved {
                    return Err(torsor_structure_shuffle_invalid(
                        "untransported_product_multiset_drift",
                    ));
                }
                position += 1;
            }
        }
    }
    if report.families != summarize_torsor_structure_shuffle_families(&report.cases) {
        return Err(torsor_structure_shuffle_invalid("family_summary"));
    }
    if report.protected_or_final_access {
        return Err(torsor_structure_shuffle_invalid(
            "protected_or_final_access",
        ));
    }
    if report.training_executed {
        return Err(torsor_structure_shuffle_invalid("training_executed"));
    }
    if report.scientific_claim {
        return Err(torsor_structure_shuffle_invalid("scientific_claim"));
    }
    if !report.experimental_non_final {
        return Err(torsor_structure_shuffle_invalid("experimental_non_final"));
    }
    // Stored evidence is never trusted: every case is regenerated from the
    // canonical matched population `(split, family, seed_block, case_id)`, its
    // evaluator-owned target and the reproducible shuffle, and compared exactly.
    let regenerated =
        collect_torsor_structure_shuffle_cases(report.split, report.budget, &report.shuffle)?;
    if regenerated != report.cases {
        return Err(torsor_structure_shuffle_invalid("case_evidence_drift"));
    }
    Ok(())
}

/// Per-probe, per-family counts for the G6 orthogonal-basis control.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct G6OrthogonalBasisFamilySummary {
    pub probe_index: usize,
    pub family: TaskFamily,
    pub n_cases: u64,
    pub g6_reference_matches: u64,
    pub g6_rotated_matches: u64,
    /// Cases whose matched C6 score changed beyond the upstream tolerance.
    pub c6_changed: u64,
}

/// Immutable G6 orthogonal-basis control report on one non-final split.
#[derive(Clone, Debug, PartialEq)]
pub struct G6OrthogonalBasisControlReport {
    pub control_contract: &'static str,
    pub population_contract: &'static str,
    pub generic_contract: &'static str,
    /// Upstream TDI-24 slice-36 probe contract, consumed unchanged.
    pub probe_contract: &'static str,
    pub split: DataSplit,
    pub budget: StageCPreflightBudget,
    pub probes: Vec<LearnedBasisProbe>,
    /// Identical capacity before and after rotation.
    pub reference_capacity: ParameterReadoutCapacity,
    pub rotated_capacity: ParameterReadoutCapacity,
    /// Case-major, probe-minor order.
    pub cases: Vec<G6OrthogonalBasisCase>,
    pub summaries: Vec<G6OrthogonalBasisFamilySummary>,
    /// Must remain false.
    pub protected_or_final_access: bool,
    /// Must remain false.
    pub training_executed: bool,
    /// Must remain false.
    pub scientific_claim: bool,
    /// Must remain true.
    pub experimental_non_final: bool,
}

const fn g6_orthogonal_basis_invalid(reason: &'static str) -> EvalError {
    EvalError::G6OrthogonalBasisControlInvalid { reason }
}

fn summarize_g6_orthogonal_basis(
    cases: &[G6OrthogonalBasisCase],
) -> Vec<G6OrthogonalBasisFamilySummary> {
    let mut summaries = Vec::new();
    for probe_index in 0..LEARNED_BASIS_PROBE_COUNT {
        for family in REQUIRED_SYNTHESIS_FAMILIES {
            let members = cases
                .iter()
                .filter(|c| c.probe_index == probe_index && c.family == *family);
            summaries.push(G6OrthogonalBasisFamilySummary {
                probe_index,
                family: *family,
                n_cases: members.clone().count() as u64,
                g6_reference_matches: members
                    .clone()
                    .filter(|c| c.g6_reference_matches_target)
                    .count() as u64,
                g6_rotated_matches: members
                    .clone()
                    .filter(|c| c.g6_rotated_matches_target)
                    .count() as u64,
                c6_changed: members
                    .filter(|c| !g6_rotation_invariant(c.c6_reference_score, c.c6_rotated_score))
                    .count() as u64,
            });
        }
    }
    summaries
}

fn collect_g6_orthogonal_basis_cases(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<Vec<G6OrthogonalBasisCase>, EvalError> {
    let mut cases = Vec::new();
    for family in REQUIRED_SYNTHESIS_FAMILIES {
        for seed_block in 0..budget.seed_blocks {
            cases.extend(evaluate_g6_orthogonal_basis_control(
                split,
                *family,
                seed_block,
                budget.cases_per_block,
            )?);
        }
    }
    Ok(cases)
}

/// Run the G6 orthogonal-basis control on the bounded matched population.
pub fn run_g6_orthogonal_basis_control(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<G6OrthogonalBasisControlReport, EvalError> {
    validate_non_final_split(split)?;
    budget.validate()?;
    let cases = collect_g6_orthogonal_basis_cases(split, budget)?;
    let summaries = summarize_g6_orthogonal_basis(&cases);
    let report = G6OrthogonalBasisControlReport {
        control_contract: G6_ORTHOGONAL_BASIS_CONTROL_CONTRACT,
        population_contract: matched_reference::MATCHED_POPULATION_CONTRACT,
        generic_contract: GENERIC6_CONTRACT,
        probe_contract: LEARNED_BASIS_PROTOTYPE_CONTRACT,
        split,
        budget,
        probes: learned_basis_probes().to_vec(),
        reference_capacity: ParameterReadoutCapacity::reference_g6(),
        rotated_capacity: ParameterReadoutCapacity::reference_g6(),
        cases,
        summaries,
        protected_or_final_access: false,
        training_executed: false,
        scientific_claim: false,
        experimental_non_final: true,
    };
    validate_g6_orthogonal_basis_control_report(&report)?;
    Ok(report)
}

/// Parse a split label first; protected/final labels never generate a case.
pub fn run_g6_orthogonal_basis_control_for_label(
    split_label: &str,
    budget: StageCPreflightBudget,
) -> Result<G6OrthogonalBasisControlReport, EvalError> {
    run_g6_orthogonal_basis_control(parse_non_final_split(split_label)?, budget)
}

/// Validate a G6 orthogonal-basis control report: pins, the unchanged
/// upstream probe set, identical capacity, canonical case-major/probe-minor
/// order, identity bit-exactness and G6 invariance per case, recomputed
/// summaries, flags and regenerated evidence.
pub fn validate_g6_orthogonal_basis_control_report(
    report: &G6OrthogonalBasisControlReport,
) -> Result<(), EvalError> {
    validate_non_final_split(report.split)?;
    if report.control_contract != G6_ORTHOGONAL_BASIS_CONTROL_CONTRACT {
        return Err(g6_orthogonal_basis_invalid("contract_drift"));
    }
    if report.population_contract != matched_reference::MATCHED_POPULATION_CONTRACT {
        return Err(g6_orthogonal_basis_invalid("population_drift"));
    }
    if report.generic_contract != GENERIC6_CONTRACT {
        return Err(g6_orthogonal_basis_invalid("generic_contract_drift"));
    }
    if report.probe_contract != LEARNED_BASIS_PROTOTYPE_CONTRACT {
        return Err(g6_orthogonal_basis_invalid("probe_contract_drift"));
    }
    if report.probes != learned_basis_probes().to_vec() {
        return Err(g6_orthogonal_basis_invalid("probe_set_drift"));
    }
    report.budget.validate()?;
    if report.reference_capacity != ParameterReadoutCapacity::reference_g6()
        || report.rotated_capacity != report.reference_capacity
    {
        return Err(g6_orthogonal_basis_invalid("capacity_mismatch"));
    }
    let expected = report.budget.cases_per_arm() * LEARNED_BASIS_PROBE_COUNT as u64;
    if report.cases.len() as u64 != expected {
        return Err(g6_orthogonal_basis_invalid("case_count"));
    }
    let mut position = 0usize;
    for family in REQUIRED_SYNTHESIS_FAMILIES {
        for seed_block in 0..report.budget.seed_blocks {
            for case_id in 0..report.budget.cases_per_block {
                for probe_index in 0..LEARNED_BASIS_PROBE_COUNT {
                    let case = &report.cases[position];
                    if case.family != *family
                        || case.seed_block != seed_block
                        || case.case_id != case_id
                        || case.probe_index != probe_index
                    {
                        return Err(g6_orthogonal_basis_invalid("case_order"));
                    }
                    if probe_index == 0
                        && (case.g6_rotated_score.to_bits() != case.g6_reference_score.to_bits()
                            || case.c6_rotated_score.to_bits() != case.c6_reference_score.to_bits())
                    {
                        return Err(g6_orthogonal_basis_invalid("identity_probe_drift"));
                    }
                    if !g6_rotation_invariant(case.g6_reference_score, case.g6_rotated_score) {
                        return Err(g6_orthogonal_basis_invalid("g6_rotation_invariance_drift"));
                    }
                    position += 1;
                }
            }
        }
    }
    if report.summaries != summarize_g6_orthogonal_basis(&report.cases) {
        return Err(g6_orthogonal_basis_invalid("family_summary"));
    }
    if report.protected_or_final_access {
        return Err(g6_orthogonal_basis_invalid("protected_or_final_access"));
    }
    if report.training_executed {
        return Err(g6_orthogonal_basis_invalid("training_executed"));
    }
    if report.scientific_claim {
        return Err(g6_orthogonal_basis_invalid("scientific_claim"));
    }
    if !report.experimental_non_final {
        return Err(g6_orthogonal_basis_invalid("experimental_non_final"));
    }
    if collect_g6_orthogonal_basis_cases(report.split, report.budget)? != report.cases {
        return Err(g6_orthogonal_basis_invalid("case_evidence_drift"));
    }
    Ok(())
}

/// Per-arm, per-family counts for the position-geometry ablation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PositionGeometryFamilySummary {
    pub arm_index: usize,
    pub family: TaskFamily,
    pub n_cases: u64,
    pub matches: u64,
    /// Cases with a non-zero Varignon transport term under this arm.
    pub nonzero_transport: u64,
}

/// Immutable position-geometry ablation report on one non-final split.
#[derive(Clone, Debug, PartialEq)]
pub struct PositionGeometryAblationReport {
    pub ablation_contract: &'static str,
    pub population_contract: &'static str,
    pub geometry_contract: &'static str,
    pub torsor_contract: &'static str,
    pub split: DataSplit,
    pub budget: StageCPreflightBudget,
    pub arms: Vec<Option<PositionGeometryArm>>,
    /// Identical T6 capacity under every arm.
    pub capacity: ParameterReadoutCapacity,
    /// Case-major, arm-minor order.
    pub cases: Vec<PositionGeometryCase>,
    pub summaries: Vec<PositionGeometryFamilySummary>,
    /// Must remain false.
    pub protected_or_final_access: bool,
    /// Must remain false.
    pub training_executed: bool,
    /// Must remain false: no geometry is selected.
    pub scientific_claim: bool,
    /// Must remain true.
    pub experimental_non_final: bool,
}

const fn position_geometry_invalid(reason: &'static str) -> EvalError {
    EvalError::PositionGeometryAblationInvalid { reason }
}

fn summarize_position_geometry(
    cases: &[PositionGeometryCase],
) -> Vec<PositionGeometryFamilySummary> {
    let mut summaries = Vec::new();
    for arm_index in 0..POSITION_GEOMETRY_ABLATION_ARMS.len() {
        for family in REQUIRED_SYNTHESIS_FAMILIES {
            let members = cases
                .iter()
                .filter(|c| c.arm_index == arm_index && c.family == *family);
            summaries.push(PositionGeometryFamilySummary {
                arm_index,
                family: *family,
                n_cases: members.clone().count() as u64,
                matches: members.clone().filter(|c| c.matches_target).count() as u64,
                nonzero_transport: members.filter(|c| c.transport_term.abs() > 0.0).count() as u64,
            });
        }
    }
    summaries
}

fn collect_position_geometry_cases(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<Vec<PositionGeometryCase>, EvalError> {
    let mut cases = Vec::new();
    for family in REQUIRED_SYNTHESIS_FAMILIES {
        for seed_block in 0..budget.seed_blocks {
            cases.extend(evaluate_position_geometry_ablation(
                split,
                *family,
                seed_block,
                budget.cases_per_block,
            )?);
        }
    }
    Ok(cases)
}

/// Run the position-geometry ablation on the bounded matched population.
pub fn run_position_geometry_ablation(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<PositionGeometryAblationReport, EvalError> {
    validate_non_final_split(split)?;
    budget.validate()?;
    let cases = collect_position_geometry_cases(split, budget)?;
    let summaries = summarize_position_geometry(&cases);
    let report = PositionGeometryAblationReport {
        ablation_contract: POSITION_GEOMETRY_ABLATION_CONTRACT,
        population_contract: matched_reference::MATCHED_POPULATION_CONTRACT,
        geometry_contract: POSITION_GEOMETRY_ARM_CONTRACT,
        torsor_contract: TORSOR_CONTRACT,
        split,
        budget,
        arms: POSITION_GEOMETRY_ABLATION_ARMS.to_vec(),
        capacity: ParameterReadoutCapacity::reference_t6(),
        cases,
        summaries,
        protected_or_final_access: false,
        training_executed: false,
        scientific_claim: false,
        experimental_non_final: true,
    };
    validate_position_geometry_ablation_report(&report)?;
    Ok(report)
}

/// Parse a split label first; protected/final labels never generate a case.
pub fn run_position_geometry_ablation_for_label(
    split_label: &str,
    budget: StageCPreflightBudget,
) -> Result<PositionGeometryAblationReport, EvalError> {
    run_position_geometry_ablation(parse_non_final_split(split_label)?, budget)
}

/// Validate a position-geometry ablation report: pins, the complete arm set,
/// capacity, canonical order, registry-consistent positions, recomputed
/// summaries, flags and regenerated evidence.
pub fn validate_position_geometry_ablation_report(
    report: &PositionGeometryAblationReport,
) -> Result<(), EvalError> {
    validate_non_final_split(report.split)?;
    if report.ablation_contract != POSITION_GEOMETRY_ABLATION_CONTRACT {
        return Err(position_geometry_invalid("contract_drift"));
    }
    if report.population_contract != matched_reference::MATCHED_POPULATION_CONTRACT {
        return Err(position_geometry_invalid("population_drift"));
    }
    if report.geometry_contract != POSITION_GEOMETRY_ARM_CONTRACT {
        return Err(position_geometry_invalid("geometry_contract_drift"));
    }
    if report.torsor_contract != TORSOR_CONTRACT {
        return Err(position_geometry_invalid("torsor_contract_drift"));
    }
    if report.arms != POSITION_GEOMETRY_ABLATION_ARMS.to_vec() {
        return Err(position_geometry_invalid("arm_set_drift"));
    }
    report.budget.validate()?;
    if report.capacity != ParameterReadoutCapacity::reference_t6() {
        return Err(position_geometry_invalid("capacity_mismatch"));
    }
    let arms = POSITION_GEOMETRY_ABLATION_ARMS.len();
    if report.cases.len() as u64 != report.budget.cases_per_arm() * arms as u64 {
        return Err(position_geometry_invalid("case_count"));
    }
    let mut position = 0usize;
    for family in REQUIRED_SYNTHESIS_FAMILIES {
        for seed_block in 0..report.budget.seed_blocks {
            for case_id in 0..report.budget.cases_per_block {
                for arm_index in 0..arms {
                    let case = &report.cases[position];
                    if case.family != *family
                        || case.seed_block != seed_block
                        || case.case_id != case_id
                        || case.arm_index != arm_index
                    {
                        return Err(position_geometry_invalid("case_order"));
                    }
                    position += 1;
                }
            }
        }
    }
    if report.summaries != summarize_position_geometry(&report.cases) {
        return Err(position_geometry_invalid("family_summary"));
    }
    if report.protected_or_final_access {
        return Err(position_geometry_invalid("protected_or_final_access"));
    }
    if report.training_executed {
        return Err(position_geometry_invalid("training_executed"));
    }
    if report.scientific_claim {
        return Err(position_geometry_invalid("scientific_claim"));
    }
    if !report.experimental_non_final {
        return Err(position_geometry_invalid("experimental_non_final"));
    }
    if collect_position_geometry_cases(report.split, report.budget)? != report.cases {
        return Err(position_geometry_invalid("case_evidence_drift"));
    }
    Ok(())
}

/// Immutable sequence-length scaling report on one non-final split.
#[derive(Clone, Debug, PartialEq)]
pub struct SequenceLengthScalingReport {
    pub scaling_contract: &'static str,
    pub population_contract: &'static str,
    pub normalizer_contract: &'static str,
    pub masking_contract: &'static str,
    pub split: DataSplit,
    pub budget: StageCPreflightBudget,
    pub lengths: Vec<usize>,
    /// Identical T6 and C6 capacity at every length.
    pub t6_capacity: ParameterReadoutCapacity,
    pub c6_capacity: ParameterReadoutCapacity,
    /// Family-major, seed-block, then length, policy, arm.
    pub cells: Vec<SequenceScalingCell>,
    /// Must remain false.
    pub protected_or_final_access: bool,
    /// Must remain false.
    pub training_executed: bool,
    /// Must remain false: no length is selected.
    pub scientific_claim: bool,
    /// Must remain true.
    pub experimental_non_final: bool,
}

const fn sequence_length_invalid(reason: &'static str) -> EvalError {
    EvalError::SequenceLengthScalingInvalid { reason }
}

fn collect_sequence_length_cells(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<Vec<SequenceScalingCell>, EvalError> {
    let mut cells = Vec::new();
    for family in REQUIRED_SYNTHESIS_FAMILIES {
        for seed_block in 0..budget.seed_blocks {
            cells.extend(evaluate_sequence_length_scaling(
                split,
                *family,
                seed_block,
                budget.cases_per_block,
            )?);
        }
    }
    Ok(cells)
}

/// Run the sequence-length scaling study on the bounded matched population.
pub fn run_sequence_length_scaling(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<SequenceLengthScalingReport, EvalError> {
    validate_non_final_split(split)?;
    budget.validate()?;
    let cells = collect_sequence_length_cells(split, budget)?;
    let report = SequenceLengthScalingReport {
        scaling_contract: SEQUENCE_LENGTH_SCALING_CONTRACT,
        population_contract: matched_reference::MATCHED_POPULATION_CONTRACT,
        normalizer_contract: NORMALIZER_CONTRACT,
        masking_contract: MASKING_CONTRACT,
        split,
        budget,
        lengths: SEQUENCE_SCALING_LENGTHS.to_vec(),
        t6_capacity: ParameterReadoutCapacity::reference_t6(),
        c6_capacity: ParameterReadoutCapacity::reference_c6(),
        cells,
        protected_or_final_access: false,
        training_executed: false,
        scientific_claim: false,
        experimental_non_final: true,
    };
    validate_sequence_length_scaling_report(&report)?;
    Ok(report)
}

/// Parse a split label first; protected/final labels never generate a case.
pub fn run_sequence_length_scaling_for_label(
    split_label: &str,
    budget: StageCPreflightBudget,
) -> Result<SequenceLengthScalingReport, EvalError> {
    run_sequence_length_scaling(parse_non_final_split(split_label)?, budget)
}

/// Validate a sequence-length scaling report: pins, the complete length set,
/// matched capacity, canonical order, exact cost accounting, row sums, flags
/// and regenerated evidence.
pub fn validate_sequence_length_scaling_report(
    report: &SequenceLengthScalingReport,
) -> Result<(), EvalError> {
    validate_non_final_split(report.split)?;
    if report.scaling_contract != SEQUENCE_LENGTH_SCALING_CONTRACT {
        return Err(sequence_length_invalid("contract_drift"));
    }
    if report.population_contract != matched_reference::MATCHED_POPULATION_CONTRACT {
        return Err(sequence_length_invalid("population_drift"));
    }
    if report.normalizer_contract != NORMALIZER_CONTRACT
        || report.masking_contract != MASKING_CONTRACT
    {
        return Err(sequence_length_invalid("normalizer_contract_drift"));
    }
    if report.lengths != SEQUENCE_SCALING_LENGTHS.to_vec() {
        return Err(sequence_length_invalid("length_set_drift"));
    }
    report.budget.validate()?;
    if report.t6_capacity != ParameterReadoutCapacity::reference_t6()
        || report.c6_capacity != ParameterReadoutCapacity::reference_c6()
    {
        return Err(sequence_length_invalid("capacity_mismatch"));
    }
    let per_block = SEQUENCE_SCALING_LENGTHS.len()
        * SEQUENCE_SCALING_POLICIES.len()
        * SEQUENCE_SCALING_ARMS.len();
    let expected = REQUIRED_SYNTHESIS_FAMILIES.len() as u64 * report.budget.seed_blocks;
    if report.cells.len() as u64 != expected * per_block as u64 {
        return Err(sequence_length_invalid("cell_count"));
    }
    let n = report.budget.cases_per_block;
    let mut position = 0usize;
    for family in REQUIRED_SYNTHESIS_FAMILIES {
        for seed_block in 0..report.budget.seed_blocks {
            for length in SEQUENCE_SCALING_LENGTHS {
                for policy in SEQUENCE_SCALING_POLICIES {
                    for arm in SEQUENCE_SCALING_ARMS {
                        let cell = &report.cells[position];
                        position += 1;
                        if cell.family != *family
                            || cell.seed_block != seed_block
                            || cell.length != length
                            || cell.policy != policy
                            || cell.arm != arm
                        {
                            return Err(sequence_length_invalid("cell_order"));
                        }
                        let l = length as u64;
                        let masked = match policy {
                            MaskPolicy::Causal => cell.windows * l * (l - 1) / 2,
                            _ => 0,
                        };
                        if cell.windows * l + cell.unwindowed_cases != n
                            || cell.unwindowed_cases >= l
                            || cell.score_evaluations != cell.windows * l * l
                            || cell.normalizer_calls != cell.windows * l
                            || cell.masked_entries != masked
                            || cell.self_retrieval_rows > cell.normalizer_calls
                        {
                            return Err(sequence_length_invalid("cost_accounting_drift"));
                        }
                        if cell.max_row_sum_error.is_nan()
                            || cell.max_row_sum_error > SEQUENCE_SCALING_ROW_SUM_TOLERANCE
                        {
                            return Err(sequence_length_invalid("row_sum_drift"));
                        }
                    }
                }
            }
        }
    }
    if report.protected_or_final_access {
        return Err(sequence_length_invalid("protected_or_final_access"));
    }
    if report.training_executed {
        return Err(sequence_length_invalid("training_executed"));
    }
    if report.scientific_claim {
        return Err(sequence_length_invalid("scientific_claim"));
    }
    if !report.experimental_non_final {
        return Err(sequence_length_invalid("experimental_non_final"));
    }
    if collect_sequence_length_cells(report.split, report.budget)? != report.cells {
        return Err(sequence_length_invalid("case_evidence_drift"));
    }
    Ok(())
}

/// Preregistered per-block sample counts of the data-volume scaling study
/// (slice 39). Every count is reported; none is selected. Both arms always
/// receive the same count: no adaptive, arm-specific allocation.
pub const DATA_VOLUME_COUNTS: [u64; 3] = [8, 16, 32];

/// Per (count, family, arm) match counts on the bounded matched population.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DataVolumeCell {
    pub cases_per_block: u64,
    pub family: TaskFamily,
    pub arm: ComparisonArm,
    /// `seed_blocks * cases_per_block`, identical for T6 and C6.
    pub n_cases: u64,
    pub matches: u64,
}

/// Immutable data-volume scaling report on one non-final split.
#[derive(Clone, Debug, PartialEq)]
pub struct DataVolumeScalingReport {
    pub scaling_contract: &'static str,
    pub population_contract: &'static str,
    pub split: DataSplit,
    /// Only `seed_blocks` is consumed; counts come from the registry.
    pub budget: StageCPreflightBudget,
    pub counts: Vec<u64>,
    pub t6_capacity: ParameterReadoutCapacity,
    pub c6_capacity: ParameterReadoutCapacity,
    /// Count-major, then family, then arm (T6, C6).
    pub cells: Vec<DataVolumeCell>,
    /// Must remain false.
    pub protected_or_final_access: bool,
    /// Must remain false.
    pub training_executed: bool,
    /// Must remain false: no count is selected.
    pub scientific_claim: bool,
    /// Must remain true.
    pub experimental_non_final: bool,
}

const fn data_volume_invalid(reason: &'static str) -> EvalError {
    EvalError::DataVolumeScalingInvalid { reason }
}

fn collect_data_volume_cells(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<Vec<DataVolumeCell>, EvalError> {
    let largest = DATA_VOLUME_COUNTS[DATA_VOLUME_COUNTS.len() - 1];
    let mut cells = Vec::new();
    for count in DATA_VOLUME_COUNTS {
        for family in REQUIRED_SYNTHESIS_FAMILIES {
            let (mut t6, mut c6) = (0u64, 0u64);
            for seed_block in 0..budget.seed_blocks {
                let run = matched_reference::MatchedPrimaryRun::evaluate(
                    split, *family, seed_block, count,
                )?;
                // Smaller volumes are exact prefixes of the largest one.
                let full = matched_reference::MatchedPrimaryRun::evaluate(
                    split, *family, seed_block, largest,
                )?;
                if run.t6_outcomes() != &full.t6_outcomes()[..count as usize]
                    || run.c6_outcomes() != &full.c6_outcomes()[..count as usize]
                {
                    return Err(data_volume_invalid("nesting_drift"));
                }
                t6 += run
                    .t6_outcomes()
                    .iter()
                    .filter(|o| o.matches_oracle)
                    .count() as u64;
                c6 += run
                    .c6_outcomes()
                    .iter()
                    .filter(|o| o.matches_oracle)
                    .count() as u64;
            }
            let n_cases = budget.seed_blocks * count;
            for (arm, matches) in [(ComparisonArm::T6, t6), (ComparisonArm::C6, c6)] {
                cells.push(DataVolumeCell {
                    cases_per_block: count,
                    family: *family,
                    arm,
                    n_cases,
                    matches,
                });
            }
        }
    }
    Ok(cells)
}

/// Run the data-volume scaling study on the bounded matched population.
pub fn run_data_volume_scaling(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<DataVolumeScalingReport, EvalError> {
    validate_non_final_split(split)?;
    budget.validate()?;
    let report = DataVolumeScalingReport {
        scaling_contract: DATA_VOLUME_SCALING_CONTRACT,
        population_contract: matched_reference::MATCHED_POPULATION_CONTRACT,
        split,
        budget,
        counts: DATA_VOLUME_COUNTS.to_vec(),
        t6_capacity: ParameterReadoutCapacity::reference_t6(),
        c6_capacity: ParameterReadoutCapacity::reference_c6(),
        cells: collect_data_volume_cells(split, budget)?,
        protected_or_final_access: false,
        training_executed: false,
        scientific_claim: false,
        experimental_non_final: true,
    };
    validate_data_volume_scaling_report(&report)?;
    Ok(report)
}

/// Parse a split label first; protected/final labels never generate a case.
pub fn run_data_volume_scaling_for_label(
    split_label: &str,
    budget: StageCPreflightBudget,
) -> Result<DataVolumeScalingReport, EvalError> {
    run_data_volume_scaling(parse_non_final_split(split_label)?, budget)
}

/// Validate a data-volume scaling report: pins, the complete count set,
/// matched capacity, canonical order, identical per-arm allocation, flags
/// and regenerated evidence.
pub fn validate_data_volume_scaling_report(
    report: &DataVolumeScalingReport,
) -> Result<(), EvalError> {
    validate_non_final_split(report.split)?;
    if report.scaling_contract != DATA_VOLUME_SCALING_CONTRACT {
        return Err(data_volume_invalid("contract_drift"));
    }
    if report.population_contract != matched_reference::MATCHED_POPULATION_CONTRACT {
        return Err(data_volume_invalid("population_drift"));
    }
    if report.counts != DATA_VOLUME_COUNTS.to_vec() {
        return Err(data_volume_invalid("count_set_drift"));
    }
    report.budget.validate()?;
    if report.t6_capacity != ParameterReadoutCapacity::reference_t6()
        || report.c6_capacity != ParameterReadoutCapacity::reference_c6()
    {
        return Err(data_volume_invalid("capacity_mismatch"));
    }
    let expected = DATA_VOLUME_COUNTS.len() * REQUIRED_SYNTHESIS_FAMILIES.len() * 2;
    if report.cells.len() != expected {
        return Err(data_volume_invalid("cell_count"));
    }
    let mut position = 0usize;
    for count in DATA_VOLUME_COUNTS {
        for family in REQUIRED_SYNTHESIS_FAMILIES {
            for arm in [ComparisonArm::T6, ComparisonArm::C6] {
                let cell = &report.cells[position];
                position += 1;
                if cell.cases_per_block != count || cell.family != *family || cell.arm != arm {
                    return Err(data_volume_invalid("cell_order"));
                }
                if cell.n_cases != report.budget.seed_blocks * count || cell.matches > cell.n_cases
                {
                    return Err(data_volume_invalid("allocation_drift"));
                }
            }
        }
    }
    if report.protected_or_final_access {
        return Err(data_volume_invalid("protected_or_final_access"));
    }
    if report.training_executed {
        return Err(data_volume_invalid("training_executed"));
    }
    if report.scientific_claim {
        return Err(data_volume_invalid("scientific_claim"));
    }
    if !report.experimental_non_final {
        return Err(data_volume_invalid("experimental_non_final"));
    }
    if collect_data_volume_cells(report.split, report.budget)? != report.cells {
        return Err(data_volume_invalid("case_evidence_drift"));
    }
    Ok(())
}

/// Phase-D Stage-D attribution audit contract pin (slice 40).
pub const STAGE_D_ATTRIBUTION_AUDIT_CONTRACT: &str = "tdi25-stage-d-attribution-audit-v1";

/// Audited Phase-C/D slices in campaign order (slices 30 through 39) with
/// their frozen contract pins. Nothing outside this registry is audited.
pub const ATTRIBUTION_AUDITED_SLICES: [(u8, &str); 10] = [
    (30, STAGE_C_PREFLIGHT_CONTRACT),
    (31, TORSOR_REDUCTION_POINT_ABLATION_CONTRACT),
    (32, DIRECT_VS_FACTORIZED_BRIDGE_CONTRACT),
    (33, CHIRAL_GAMMA_ZERO_ABLATION_CONTRACT),
    (34, CHIRAL_PARITY_SHUFFLE_CONTROL_CONTRACT),
    (35, TORSOR_STRUCTURE_SHUFFLE_CONTROL_CONTRACT),
    (36, G6_ORTHOGONAL_BASIS_CONTROL_CONTRACT),
    (37, POSITION_GEOMETRY_ABLATION_CONTRACT),
    (38, SEQUENCE_LENGTH_SCALING_CONTRACT),
    (39, DATA_VOLUME_SCALING_CONTRACT),
];

/// Claim class an audited slice may support after the ablations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttributionClaim {
    /// The validated report supports only software-semantics statements
    /// (contracts, accounting, fail-closed behavior) on Development/Validation.
    SoftwareSemanticsOnly,
    /// Withheld: the report failed validation or set a forbidden flag.
    Withheld,
}

/// One audited slice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttributionEntry {
    pub slice: u8,
    pub contract: &'static str,
    /// The upstream validator accepted the regenerated report.
    pub validated: bool,
    /// FNV-1a 64 digest of the regenerated report's `Debug` rendering; binds
    /// the entry to the exact split, budget and evidence.
    pub evidence_digest: u64,
    /// Flags read from the report itself.
    pub protected_or_final_access: bool,
    pub training_executed: bool,
    pub scientific_claim: bool,
    pub experimental_non_final: bool,
    pub admissible: AttributionClaim,
}

/// Immutable Stage-D attribution audit on one non-final split.
#[derive(Clone, Debug, PartialEq)]
pub struct StageDAttributionAuditReport {
    pub audit_contract: &'static str,
    pub split: DataSplit,
    pub budget: StageCPreflightBudget,
    pub entries: Vec<AttributionEntry>,
    /// Must remain false: no scientific attribution is admissible without a
    /// trained, preregistered Stage-D run, which this campaign never executes.
    pub scientific_attribution_admissible: bool,
    /// Must remain false.
    pub protected_or_final_access: bool,
    /// Must remain false.
    pub training_executed: bool,
    /// Must remain false.
    pub scientific_claim: bool,
    /// Must remain true.
    pub experimental_non_final: bool,
}

const fn attribution_invalid(reason: &'static str) -> EvalError {
    EvalError::StageDAttributionAuditInvalid { reason }
}

fn attribution_digest(rendering: &str) -> u64 {
    rendering.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
    })
}

const fn admissible_claim(entry: &AttributionEntry) -> AttributionClaim {
    if entry.validated
        && !entry.protected_or_final_access
        && !entry.training_executed
        && !entry.scientific_claim
        && entry.experimental_non_final
    {
        AttributionClaim::SoftwareSemanticsOnly
    } else {
        AttributionClaim::Withheld
    }
}

fn collect_attribution_entries(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<Vec<AttributionEntry>, EvalError> {
    macro_rules! audit {
        ($slice:expr, $contract:expr, $report:expr) => {{
            let report = $report;
            let training = report.training_executed;
            audit!(@entry $slice, $contract, report, training)
        }};
        (@untrained $slice:expr, $contract:expr, $report:expr) => {{
            let report = $report;
            // The Stage-C preflight report has no training path at all.
            audit!(@entry $slice, $contract, report, false)
        }};
        (@entry $slice:expr, $contract:expr, $report:expr, $training:expr) => {{
            let report = $report;
            let mut entry = AttributionEntry {
                slice: $slice,
                contract: $contract,
                validated: true,
                evidence_digest: attribution_digest(&format!("{report:?}")),
                protected_or_final_access: report.protected_or_final_access,
                training_executed: $training,
                scientific_claim: report.scientific_claim,
                experimental_non_final: report.experimental_non_final,
                admissible: AttributionClaim::Withheld,
            };
            entry.admissible = admissible_claim(&entry);
            entry
        }};
    }
    // Every runner validates its own report before returning it; any
    // rejection aborts the audit fail-closed.
    Ok(vec![
        audit!(@untrained 30, STAGE_C_PREFLIGHT_CONTRACT, run_stage_c_preflight(split, budget)?),
        audit!(
            31,
            TORSOR_REDUCTION_POINT_ABLATION_CONTRACT,
            run_torsor_reduction_point_ablation(split, budget)?
        ),
        audit!(
            32,
            DIRECT_VS_FACTORIZED_BRIDGE_CONTRACT,
            run_direct_vs_factorized_bridge(split, budget)?
        ),
        audit!(
            33,
            CHIRAL_GAMMA_ZERO_ABLATION_CONTRACT,
            run_chiral_gamma_zero_ablation(split, budget)?
        ),
        audit!(
            34,
            CHIRAL_PARITY_SHUFFLE_CONTROL_CONTRACT,
            run_chiral_parity_shuffle_control(split, budget)?
        ),
        audit!(
            35,
            TORSOR_STRUCTURE_SHUFFLE_CONTROL_CONTRACT,
            run_torsor_structure_shuffle_control(split, budget)?
        ),
        audit!(
            36,
            G6_ORTHOGONAL_BASIS_CONTROL_CONTRACT,
            run_g6_orthogonal_basis_control(split, budget)?
        ),
        audit!(
            37,
            POSITION_GEOMETRY_ABLATION_CONTRACT,
            run_position_geometry_ablation(split, budget)?
        ),
        audit!(
            38,
            SEQUENCE_LENGTH_SCALING_CONTRACT,
            run_sequence_length_scaling(split, budget)?
        ),
        audit!(
            39,
            DATA_VOLUME_SCALING_CONTRACT,
            run_data_volume_scaling(split, budget)?
        ),
    ])
}

/// Run the Stage-D attribution audit over every audited slice.
pub fn run_stage_d_attribution_audit(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<StageDAttributionAuditReport, EvalError> {
    validate_non_final_split(split)?;
    budget.validate()?;
    let report = StageDAttributionAuditReport {
        audit_contract: STAGE_D_ATTRIBUTION_AUDIT_CONTRACT,
        split,
        budget,
        entries: collect_attribution_entries(split, budget)?,
        scientific_attribution_admissible: false,
        protected_or_final_access: false,
        training_executed: false,
        scientific_claim: false,
        experimental_non_final: true,
    };
    validate_stage_d_attribution_audit_report(&report)?;
    Ok(report)
}

/// Parse a split label first; protected/final labels never generate a case.
pub fn run_stage_d_attribution_audit_for_label(
    split_label: &str,
    budget: StageCPreflightBudget,
) -> Result<StageDAttributionAuditReport, EvalError> {
    run_stage_d_attribution_audit(parse_non_final_split(split_label)?, budget)
}

/// Validate a Stage-D attribution audit: pin, the complete audited registry
/// in order, per-entry admissibility derived from flags, no withheld entry,
/// no scientific attribution, flags and regenerated evidence.
pub fn validate_stage_d_attribution_audit_report(
    report: &StageDAttributionAuditReport,
) -> Result<(), EvalError> {
    validate_non_final_split(report.split)?;
    if report.audit_contract != STAGE_D_ATTRIBUTION_AUDIT_CONTRACT {
        return Err(attribution_invalid("contract_drift"));
    }
    report.budget.validate()?;
    if report.entries.len() != ATTRIBUTION_AUDITED_SLICES.len() {
        return Err(attribution_invalid("registry_drift"));
    }
    for (entry, (slice, contract)) in report.entries.iter().zip(ATTRIBUTION_AUDITED_SLICES) {
        if entry.slice != slice || entry.contract != contract {
            return Err(attribution_invalid("registry_drift"));
        }
        if entry.admissible != admissible_claim(entry) {
            return Err(attribution_invalid("admissibility_drift"));
        }
        if entry.admissible != AttributionClaim::SoftwareSemanticsOnly {
            return Err(attribution_invalid("withheld_entry"));
        }
    }
    if report.scientific_attribution_admissible {
        return Err(attribution_invalid("scientific_attribution_admissible"));
    }
    if report.protected_or_final_access {
        return Err(attribution_invalid("protected_or_final_access"));
    }
    if report.training_executed {
        return Err(attribution_invalid("training_executed"));
    }
    if report.scientific_claim {
        return Err(attribution_invalid("scientific_claim"));
    }
    if !report.experimental_non_final {
        return Err(attribution_invalid("experimental_non_final"));
    }
    if collect_attribution_entries(report.split, report.budget)? != report.entries {
        return Err(attribution_invalid("case_evidence_drift"));
    }
    Ok(())
}

/// Frozen paired seed blocks of the multi-seed replication (slice 41). Both
/// arms always score the same blocks and cases; nothing is selected.
pub const REPLICATION_SEED_BLOCKS: [u64; 8] = [0, 1, 2, 3, 4, 5, 6, 7];

/// Paired per (seed block, family) counts on the matched population.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReplicationBlockCell {
    pub seed_block: u64,
    pub family: TaskFamily,
    pub n_cases: u64,
    pub t6_matches: u64,
    pub c6_matches: u64,
    /// Paired discordance: T6 right and C6 wrong, and the reverse.
    pub t6_only: u64,
    pub c6_only: u64,
}

/// Immutable multi-seed replication report on one non-final split.
#[derive(Clone, Debug, PartialEq)]
pub struct MultiSeedReplicationReport {
    pub replication_contract: &'static str,
    pub population_contract: &'static str,
    pub split: DataSplit,
    /// Only `cases_per_block` is consumed; blocks come from the registry.
    pub budget: StageCPreflightBudget,
    pub seed_blocks: Vec<u64>,
    /// Block-major, then family.
    pub cells: Vec<ReplicationBlockCell>,
    pub pooled_t6_matches: u64,
    pub pooled_c6_matches: u64,
    pub pooled_t6_only: u64,
    pub pooled_c6_only: u64,
    /// Blocks (pooled over families) with C6 ahead, T6 ahead, tied.
    /// Descriptive only.
    pub blocks_c6_ahead: u64,
    pub blocks_t6_ahead: u64,
    pub blocks_tied: u64,
    /// Must remain false.
    pub protected_or_final_access: bool,
    /// Must remain false.
    pub training_executed: bool,
    /// Must remain false.
    pub scientific_claim: bool,
    /// Must remain true.
    pub experimental_non_final: bool,
}

const fn replication_invalid(reason: &'static str) -> EvalError {
    EvalError::MultiSeedReplicationInvalid { reason }
}

fn collect_replication_cells(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<Vec<ReplicationBlockCell>, EvalError> {
    let mut cells = Vec::new();
    let mut digests = Vec::new();
    for seed_block in REPLICATION_SEED_BLOCKS {
        for family in REQUIRED_SYNTHESIS_FAMILIES {
            let run = matched_reference::MatchedPrimaryRun::evaluate(
                split,
                *family,
                seed_block,
                budget.cases_per_block,
            )?;
            let mut cell = ReplicationBlockCell {
                seed_block,
                family: *family,
                n_cases: run.t6_outcomes().len() as u64,
                t6_matches: 0,
                c6_matches: 0,
                t6_only: 0,
                c6_only: 0,
            };
            for (t6, c6) in run.t6_outcomes().iter().zip(run.c6_outcomes()) {
                // Paired: both arms carry the same canonical case identity.
                if t6.canonical_digest != c6.canonical_digest || t6.case_id != c6.case_id {
                    return Err(replication_invalid("pairing_drift"));
                }
                digests.push(t6.canonical_digest.clone());
                cell.t6_matches += u64::from(t6.matches_oracle);
                cell.c6_matches += u64::from(c6.matches_oracle);
                cell.t6_only += u64::from(t6.matches_oracle && !c6.matches_oracle);
                cell.c6_only += u64::from(c6.matches_oracle && !t6.matches_oracle);
            }
            cells.push(cell);
        }
    }
    let total = digests.len();
    digests.sort_unstable();
    digests.dedup();
    if digests.len() != total {
        return Err(replication_invalid("block_overlap"));
    }
    Ok(cells)
}

fn block_tallies(cells: &[ReplicationBlockCell]) -> (u64, u64, u64) {
    let (mut ahead, mut behind, mut tied) = (0, 0, 0);
    for seed_block in REPLICATION_SEED_BLOCKS {
        let members = cells.iter().filter(|c| c.seed_block == seed_block);
        let t6: u64 = members.clone().map(|c| c.t6_matches).sum();
        let c6: u64 = members.map(|c| c.c6_matches).sum();
        match c6.cmp(&t6) {
            core::cmp::Ordering::Greater => ahead += 1,
            core::cmp::Ordering::Less => behind += 1,
            core::cmp::Ordering::Equal => tied += 1,
        }
    }
    (ahead, behind, tied)
}

/// Run the multi-seed replication over every frozen paired seed block.
pub fn run_multi_seed_replication(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<MultiSeedReplicationReport, EvalError> {
    validate_non_final_split(split)?;
    budget.validate()?;
    let cells = collect_replication_cells(split, budget)?;
    let (ahead, behind, tied) = block_tallies(&cells);
    let report = MultiSeedReplicationReport {
        replication_contract: MULTI_SEED_REPLICATION_CONTRACT,
        population_contract: matched_reference::MATCHED_POPULATION_CONTRACT,
        split,
        budget,
        seed_blocks: REPLICATION_SEED_BLOCKS.to_vec(),
        pooled_t6_matches: cells.iter().map(|c| c.t6_matches).sum(),
        pooled_c6_matches: cells.iter().map(|c| c.c6_matches).sum(),
        pooled_t6_only: cells.iter().map(|c| c.t6_only).sum(),
        pooled_c6_only: cells.iter().map(|c| c.c6_only).sum(),
        blocks_c6_ahead: ahead,
        blocks_t6_ahead: behind,
        blocks_tied: tied,
        cells,
        protected_or_final_access: false,
        training_executed: false,
        scientific_claim: false,
        experimental_non_final: true,
    };
    validate_multi_seed_replication_report(&report)?;
    Ok(report)
}

/// Parse a split label first; protected/final labels never generate a case.
pub fn run_multi_seed_replication_for_label(
    split_label: &str,
    budget: StageCPreflightBudget,
) -> Result<MultiSeedReplicationReport, EvalError> {
    run_multi_seed_replication(parse_non_final_split(split_label)?, budget)
}

/// Validate a multi-seed replication report: pins, the frozen block set,
/// canonical order, per-cell paired consistency, pooled sums, tallies,
/// flags and regenerated evidence.
pub fn validate_multi_seed_replication_report(
    report: &MultiSeedReplicationReport,
) -> Result<(), EvalError> {
    validate_non_final_split(report.split)?;
    if report.replication_contract != MULTI_SEED_REPLICATION_CONTRACT {
        return Err(replication_invalid("contract_drift"));
    }
    if report.population_contract != matched_reference::MATCHED_POPULATION_CONTRACT {
        return Err(replication_invalid("population_drift"));
    }
    report.budget.validate()?;
    if report.seed_blocks != REPLICATION_SEED_BLOCKS.to_vec() {
        return Err(replication_invalid("block_set_drift"));
    }
    if report.cells.len() != REPLICATION_SEED_BLOCKS.len() * REQUIRED_SYNTHESIS_FAMILIES.len() {
        return Err(replication_invalid("cell_count"));
    }
    let mut position = 0usize;
    for seed_block in REPLICATION_SEED_BLOCKS {
        for family in REQUIRED_SYNTHESIS_FAMILIES {
            let cell = &report.cells[position];
            position += 1;
            if cell.seed_block != seed_block || cell.family != *family {
                return Err(replication_invalid("cell_order"));
            }
            // Paired identity: matches minus discordant wins are the
            // concordant correct cases, identical for both arms.
            if cell.n_cases != report.budget.cases_per_block
                || cell.t6_matches > cell.n_cases
                || cell.c6_matches > cell.n_cases
                || cell.t6_only > cell.t6_matches
                || cell.c6_only > cell.c6_matches
                || cell.t6_matches - cell.t6_only != cell.c6_matches - cell.c6_only
            {
                return Err(replication_invalid("paired_count_drift"));
            }
        }
    }
    let cells = &report.cells;
    if report.pooled_t6_matches != cells.iter().map(|c| c.t6_matches).sum::<u64>()
        || report.pooled_c6_matches != cells.iter().map(|c| c.c6_matches).sum::<u64>()
        || report.pooled_t6_only != cells.iter().map(|c| c.t6_only).sum::<u64>()
        || report.pooled_c6_only != cells.iter().map(|c| c.c6_only).sum::<u64>()
    {
        return Err(replication_invalid("pooled_sum_drift"));
    }
    if (
        report.blocks_c6_ahead,
        report.blocks_t6_ahead,
        report.blocks_tied,
    ) != block_tallies(cells)
    {
        return Err(replication_invalid("sign_tally_drift"));
    }
    if report.protected_or_final_access {
        return Err(replication_invalid("protected_or_final_access"));
    }
    if report.training_executed {
        return Err(replication_invalid("training_executed"));
    }
    if report.scientific_claim {
        return Err(replication_invalid("scientific_claim"));
    }
    if !report.experimental_non_final {
        return Err(replication_invalid("experimental_non_final"));
    }
    if collect_replication_cells(report.split, report.budget)? != report.cells {
        return Err(replication_invalid("case_evidence_drift"));
    }
    Ok(())
}

/// Phase-E input/noise robustness contract pin (slice 42).
pub const INPUT_NOISE_ROBUSTNESS_CONTRACT: &str = "tdi25-input-noise-robustness-v1";

/// Immutable input/noise robustness report on one non-final split.
#[derive(Clone, Debug, PartialEq)]
pub struct InputNoiseRobustnessReport {
    pub robustness_contract: &'static str,
    pub population_contract: &'static str,
    pub split: DataSplit,
    pub budget: StageCPreflightBudget,
    /// FNV-1a 64 of the contract pin; no free seed constant.
    pub noise_seed: u64,
    /// Identical reference capacities; no arm receives extra parameters.
    pub t6_capacity: ParameterReadoutCapacity,
    pub c6_capacity: ParameterReadoutCapacity,
    /// Family-major, seed block, then noise family, amplitude, arm.
    pub cells: Vec<InputNoiseCell>,
    /// Must remain false.
    pub protected_or_final_access: bool,
    /// Must remain false.
    pub training_executed: bool,
    /// Must remain false: no amplitude or family is selected.
    pub scientific_claim: bool,
    /// Must remain true.
    pub experimental_non_final: bool,
}

const fn input_noise_invalid(reason: &'static str) -> EvalError {
    EvalError::InputNoiseRobustnessInvalid { reason }
}

/// Seed of the declared perturbation stream, derived from the contract pin.
#[must_use]
pub fn input_noise_seed() -> u64 {
    INPUT_NOISE_ROBUSTNESS_CONTRACT
        .bytes()
        .fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
        })
}

fn collect_input_noise_cells(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<Vec<InputNoiseCell>, EvalError> {
    let mut cells = Vec::new();
    for family in REQUIRED_SYNTHESIS_FAMILIES {
        for seed_block in 0..budget.seed_blocks {
            cells.extend(evaluate_input_noise_robustness(
                split,
                *family,
                seed_block,
                budget.cases_per_block,
                input_noise_seed(),
            )?);
        }
    }
    Ok(cells)
}

/// Run the input/noise robustness study on the bounded matched population.
pub fn run_input_noise_robustness(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<InputNoiseRobustnessReport, EvalError> {
    validate_non_final_split(split)?;
    budget.validate()?;
    let cells = collect_input_noise_cells(split, budget)?;
    let report = InputNoiseRobustnessReport {
        robustness_contract: INPUT_NOISE_ROBUSTNESS_CONTRACT,
        population_contract: matched_reference::MATCHED_POPULATION_CONTRACT,
        split,
        budget,
        noise_seed: input_noise_seed(),
        t6_capacity: ParameterReadoutCapacity::reference_t6(),
        c6_capacity: ParameterReadoutCapacity::reference_c6(),
        cells,
        protected_or_final_access: false,
        training_executed: false,
        scientific_claim: false,
        experimental_non_final: true,
    };
    validate_input_noise_robustness_report(&report)?;
    Ok(report)
}

/// Parse a split label first; protected/final labels never generate a case.
pub fn run_input_noise_robustness_for_label(
    split_label: &str,
    budget: StageCPreflightBudget,
) -> Result<InputNoiseRobustnessReport, EvalError> {
    run_input_noise_robustness(parse_non_final_split(split_label)?, budget)
}

/// Validate an input/noise robustness report: pins, seed, matched capacity,
/// the complete declared grid in canonical order, paired counts, finite score
/// changes, flags and regenerated evidence.
pub fn validate_input_noise_robustness_report(
    report: &InputNoiseRobustnessReport,
) -> Result<(), EvalError> {
    validate_non_final_split(report.split)?;
    if report.robustness_contract != INPUT_NOISE_ROBUSTNESS_CONTRACT {
        return Err(input_noise_invalid("contract_drift"));
    }
    if report.population_contract != matched_reference::MATCHED_POPULATION_CONTRACT {
        return Err(input_noise_invalid("population_drift"));
    }
    if report.noise_seed != input_noise_seed() {
        return Err(input_noise_invalid("seed_drift"));
    }
    report.budget.validate()?;
    if report.t6_capacity != ParameterReadoutCapacity::reference_t6()
        || report.c6_capacity != ParameterReadoutCapacity::reference_c6()
    {
        return Err(input_noise_invalid("capacity_mismatch"));
    }
    let per_block =
        INPUT_NOISE_FAMILIES.len() * INPUT_NOISE_AMPLITUDES.len() * SEQUENCE_SCALING_ARMS.len();
    let expected = REQUIRED_SYNTHESIS_FAMILIES.len() as u64 * report.budget.seed_blocks;
    if report.cells.len() as u64 != expected * per_block as u64 {
        return Err(input_noise_invalid("cell_count"));
    }
    let mut position = 0usize;
    for family in REQUIRED_SYNTHESIS_FAMILIES {
        for seed_block in 0..report.budget.seed_blocks {
            for noise in INPUT_NOISE_FAMILIES {
                for amplitude in INPUT_NOISE_AMPLITUDES {
                    for arm in SEQUENCE_SCALING_ARMS {
                        let cell = &report.cells[position];
                        position += 1;
                        if cell.family != *family
                            || cell.seed_block != seed_block
                            || cell.noise != noise
                            || cell.amplitude.to_bits() != amplitude.to_bits()
                            || cell.arm != arm
                        {
                            return Err(input_noise_invalid("grid_drift"));
                        }
                        // Flips are bounded below by the net change and above
                        // by the cases that are correct on either side.
                        if cell.n_cases != report.budget.cases_per_block
                            || cell.clean_matches > cell.n_cases
                            || cell.noisy_matches > cell.n_cases
                            || cell.flips > cell.n_cases
                            || cell.flips < cell.clean_matches.abs_diff(cell.noisy_matches)
                            || cell.flips > cell.clean_matches + cell.noisy_matches
                            || (cell.flips + cell.clean_matches + cell.noisy_matches) % 2 != 0
                        {
                            return Err(input_noise_invalid("paired_count_drift"));
                        }
                        if !cell.max_abs_score_change.is_finite() || cell.max_abs_score_change < 0.0
                        {
                            return Err(input_noise_invalid("score_change_drift"));
                        }
                    }
                }
            }
        }
    }
    if report.protected_or_final_access {
        return Err(input_noise_invalid("protected_or_final_access"));
    }
    if report.training_executed {
        return Err(input_noise_invalid("training_executed"));
    }
    if report.scientific_claim {
        return Err(input_noise_invalid("scientific_claim"));
    }
    if !report.experimental_non_final {
        return Err(input_noise_invalid("experimental_non_final"));
    }
    if collect_input_noise_cells(report.split, report.budget)? != report.cells {
        return Err(input_noise_invalid("case_evidence_drift"));
    }
    Ok(())
}

/// Phase-E translation/origin stress suite contract pin (slice 43).
pub const TRANSLATION_ORIGIN_STRESS_CONTRACT: &str = "tdi25-translation-origin-stress-v1";

/// Immutable translation/origin stress report on one non-final split.
#[derive(Clone, Debug, PartialEq)]
pub struct TranslationOriginStressReport {
    pub stress_contract: &'static str,
    pub population_contract: &'static str,
    pub split: DataSplit,
    pub budget: StageCPreflightBudget,
    /// FNV-1a 64 of the contract pin; no free seed constant.
    pub stress_seed: u64,
    /// Identical reference capacities; no arm receives extra parameters.
    pub t6_capacity: ParameterReadoutCapacity,
    pub c6_capacity: ParameterReadoutCapacity,
    /// Family-major, seed block, then transformation, offset, arm.
    pub cells: Vec<OriginStressCell>,
    /// Must remain false.
    pub protected_or_final_access: bool,
    /// Must remain false.
    pub training_executed: bool,
    /// Must remain false: no transformation or offset is selected.
    pub scientific_claim: bool,
    /// Must remain true.
    pub experimental_non_final: bool,
}

const fn origin_stress_invalid(reason: &'static str) -> EvalError {
    EvalError::TranslationOriginStressInvalid { reason }
}

/// Seed of the declared offset directions, derived from the contract pin.
#[must_use]
pub fn translation_origin_stress_seed() -> u64 {
    TRANSLATION_ORIGIN_STRESS_CONTRACT
        .bytes()
        .fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
        })
}

fn collect_origin_stress_cells(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<Vec<OriginStressCell>, EvalError> {
    let mut cells = Vec::new();
    for family in REQUIRED_SYNTHESIS_FAMILIES {
        for seed_block in 0..budget.seed_blocks {
            cells.extend(evaluate_translation_origin_stress(
                split,
                *family,
                seed_block,
                budget.cases_per_block,
                translation_origin_stress_seed(),
            )?);
        }
    }
    Ok(cells)
}

/// Run the translation/origin stress suite on the bounded matched population.
pub fn run_translation_origin_stress(
    split: DataSplit,
    budget: StageCPreflightBudget,
) -> Result<TranslationOriginStressReport, EvalError> {
    validate_non_final_split(split)?;
    budget.validate()?;
    let cells = collect_origin_stress_cells(split, budget)?;
    let report = TranslationOriginStressReport {
        stress_contract: TRANSLATION_ORIGIN_STRESS_CONTRACT,
        population_contract: matched_reference::MATCHED_POPULATION_CONTRACT,
        split,
        budget,
        stress_seed: translation_origin_stress_seed(),
        t6_capacity: ParameterReadoutCapacity::reference_t6(),
        c6_capacity: ParameterReadoutCapacity::reference_c6(),
        cells,
        protected_or_final_access: false,
        training_executed: false,
        scientific_claim: false,
        experimental_non_final: true,
    };
    validate_translation_origin_stress_report(&report)?;
    Ok(report)
}

/// Parse a split label first; protected/final labels never generate a case.
pub fn run_translation_origin_stress_for_label(
    split_label: &str,
    budget: StageCPreflightBudget,
) -> Result<TranslationOriginStressReport, EvalError> {
    run_translation_origin_stress(parse_non_final_split(split_label)?, budget)
}

/// Validate a translation/origin stress report: pins, seed, matched capacity,
/// the complete declared grid in canonical order, paired counts, the
/// structural C6 origin-shift invariant, finite changes, flags and
/// regenerated evidence.
pub fn validate_translation_origin_stress_report(
    report: &TranslationOriginStressReport,
) -> Result<(), EvalError> {
    validate_non_final_split(report.split)?;
    if report.stress_contract != TRANSLATION_ORIGIN_STRESS_CONTRACT {
        return Err(origin_stress_invalid("contract_drift"));
    }
    if report.population_contract != matched_reference::MATCHED_POPULATION_CONTRACT {
        return Err(origin_stress_invalid("population_drift"));
    }
    if report.stress_seed != translation_origin_stress_seed() {
        return Err(origin_stress_invalid("seed_drift"));
    }
    report.budget.validate()?;
    if report.t6_capacity != ParameterReadoutCapacity::reference_t6()
        || report.c6_capacity != ParameterReadoutCapacity::reference_c6()
    {
        return Err(origin_stress_invalid("capacity_mismatch"));
    }
    let per_block =
        ORIGIN_STRESS_TRANSFORMS.len() * ORIGIN_STRESS_OFFSETS.len() * SEQUENCE_SCALING_ARMS.len();
    let expected = REQUIRED_SYNTHESIS_FAMILIES.len() as u64 * report.budget.seed_blocks;
    if report.cells.len() as u64 != expected * per_block as u64 {
        return Err(origin_stress_invalid("cell_count"));
    }
    let mut position = 0usize;
    for family in REQUIRED_SYNTHESIS_FAMILIES {
        for seed_block in 0..report.budget.seed_blocks {
            for transform in ORIGIN_STRESS_TRANSFORMS {
                for offset in ORIGIN_STRESS_OFFSETS {
                    for arm in SEQUENCE_SCALING_ARMS {
                        let cell = &report.cells[position];
                        position += 1;
                        if cell.family != *family
                            || cell.seed_block != seed_block
                            || cell.transform != transform
                            || cell.offset.to_bits() != offset.to_bits()
                            || cell.arm != arm
                        {
                            return Err(origin_stress_invalid("grid_drift"));
                        }
                        if cell.n_cases != report.budget.cases_per_block
                            || cell.clean_matches > cell.n_cases
                            || cell.stressed_matches > cell.n_cases
                            || cell.flips > cell.n_cases
                            || cell.flips < cell.clean_matches.abs_diff(cell.stressed_matches)
                            || cell.flips > cell.clean_matches + cell.stressed_matches
                            || (cell.flips + cell.clean_matches + cell.stressed_matches) % 2 != 0
                        {
                            return Err(origin_stress_invalid("paired_count_drift"));
                        }
                        if !cell.max_abs_score_change.is_finite()
                            || cell.max_abs_score_change < 0.0
                            || !cell.max_abs_target_change.is_finite()
                            || cell.max_abs_target_change < 0.0
                        {
                            return Err(origin_stress_invalid("score_change_drift"));
                        }
                        // C6 reads no position: a rigid origin shift leaves
                        // its input, hence its score, bit-for-bit unchanged.
                        if transform == OriginStressTransform::OriginShift
                            && arm == ComparisonArm::C6
                            && cell.max_abs_score_change != 0.0
                        {
                            return Err(origin_stress_invalid("c6_origin_shift_drift"));
                        }
                    }
                }
            }
        }
    }
    if report.protected_or_final_access {
        return Err(origin_stress_invalid("protected_or_final_access"));
    }
    if report.training_executed {
        return Err(origin_stress_invalid("training_executed"));
    }
    if report.scientific_claim {
        return Err(origin_stress_invalid("scientific_claim"));
    }
    if !report.experimental_non_final {
        return Err(origin_stress_invalid("experimental_non_final"));
    }
    if collect_origin_stress_cells(report.split, report.budget)? != report.cells {
        return Err(origin_stress_invalid("case_evidence_drift"));
    }
    Ok(())
}

/// Reject any split identity outside Development/Validation.
pub fn validate_non_final_split(split: DataSplit) -> Result<(), EvalError> {
    match split {
        DataSplit::Development | DataSplit::Validation => Ok(()),
    }
}

/// Parse a split label, rejecting protected/final identities fail-closed.
pub fn parse_non_final_split(label: &str) -> Result<DataSplit, EvalError> {
    match label {
        "development" => Ok(DataSplit::Development),
        "validation" => Ok(DataSplit::Validation),
        "protected" | "final" => Err(EvalError::ProtectedOrFinalSplit),
        _ => Err(EvalError::ProtectedOrFinalSplit),
    }
}

/// Fail-closed Phase-C evaluator errors.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EvalError {
    ContractMismatch(&'static str),
    InvalidBudget,
    SplitMismatch {
        expected: DataSplit,
        actual: DataSplit,
    },
    CaseBudgetExceeded,
    Bridge(Tdi25Error),
    /// Parameter/readout capacities are unmatched across paired arms.
    ParameterReadoutMismatch {
        left_arm: ComparisonArm,
        right_arm: ComparisonArm,
        left_trainable: u64,
        right_trainable: u64,
        left_updates: u64,
        right_updates: u64,
        left_query_components: u64,
        right_query_components: u64,
    },
    /// Optimizer/update budgets are unmatched across paired arms.
    OptimizerUpdateBudgetMismatch {
        left_arm: ComparisonArm,
        right_arm: ComparisonArm,
        left_examples: u64,
        right_examples: u64,
        left_updates: u64,
        right_updates: u64,
    },
    /// Metric registry is empty, drifted, duplicated, or invented.
    MetricRegistryInvalid {
        reason: &'static str,
    },
    /// Protected/final or otherwise unknown split identity.
    ProtectedOrFinalSplit,
    /// Paired uncertainty engine rejected the request fail-closed.
    PairedUncertaintyInvalid {
        reason: &'static str,
    },
    /// Family-stratified synthesis rejected the request fail-closed.
    FamilyStratifiedSynthesisInvalid {
        reason: &'static str,
    },
    /// Failure/resource accounting rejected drift, capacity or dropped failures.
    FailureResourceAccountingInvalid {
        reason: &'static str,
    },
    /// Stage-C bounded preflight rejected budget, matrix, drift or claims.
    StageCPreflightInvalid {
        reason: &'static str,
    },
    /// Torsor reduction-point ablation rejected drift, identity or claims.
    ReductionPointAblationInvalid {
        reason: &'static str,
    },
    /// Direct vs factorized torsor bridge monitor rejected drift or residual.
    TorsorBridgeEquivalenceInvalid {
        reason: &'static str,
    },
    /// Chiral `gamma=0` ablation rejected drifted weights, identity or claims.
    ChiralGammaZeroAblationInvalid {
        reason: &'static str,
    },
    /// Chiral parity-shuffle control rejected a drifted shuffle, evidence or
    /// claims.
    ChiralParityShuffleControlInvalid {
        reason: &'static str,
    },
    /// Torsor structure-shuffle control rejected a drifted shuffle, evidence
    /// or claims.
    TorsorStructureShuffleControlInvalid {
        reason: &'static str,
    },
    /// G6 orthogonal-basis control rejected a drifted probe, invariance,
    /// evidence or claims.
    G6OrthogonalBasisControlInvalid {
        reason: &'static str,
    },
    /// Position-geometry ablation rejected a drifted arm set, geometry,
    /// evidence or claims.
    PositionGeometryAblationInvalid {
        reason: &'static str,
    },
    /// Sequence-length scaling rejected drifted lengths, masks, accounting,
    /// evidence or claims.
    SequenceLengthScalingInvalid {
        reason: &'static str,
    },
    /// Data-volume scaling rejected drifted counts, allocation, evidence or
    /// claims.
    DataVolumeScalingInvalid {
        reason: &'static str,
    },
    /// Stage-D attribution audit rejected a drifted registry, admissibility,
    /// evidence or claims.
    StageDAttributionAuditInvalid {
        reason: &'static str,
    },
    /// Multi-seed replication rejected drifted blocks, pairing, sums,
    /// evidence or claims.
    MultiSeedReplicationInvalid {
        reason: &'static str,
    },
    /// Input/noise robustness rejected a drifted grid, seed, counts, evidence
    /// or claims.
    InputNoiseRobustnessInvalid {
        reason: &'static str,
    },
    /// Translation/origin stress rejected a drifted grid, seed, transform,
    /// counts, evidence or claims.
    TranslationOriginStressInvalid {
        reason: &'static str,
    },
}

impl fmt::Display for EvalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ContractMismatch(field) => {
                write!(formatter, "TDI-25 evaluator contract mismatch: {field}")
            }
            Self::InvalidBudget => formatter.write_str("TDI-25 evaluator budget is invalid"),
            Self::SplitMismatch { expected, actual } => write!(
                formatter,
                "TDI-25 evaluator split mismatch: expected {}, got {}",
                expected.as_str(),
                actual.as_str()
            ),
            Self::CaseBudgetExceeded => {
                formatter.write_str("TDI-25 evaluator case budget exceeded")
            }
            Self::Bridge(error) => write!(formatter, "TDI-25 evaluator bridge failure: {error}"),
            Self::ParameterReadoutMismatch {
                left_arm,
                right_arm,
                left_trainable,
                right_trainable,
                left_updates,
                right_updates,
                left_query_components,
                right_query_components,
            } => write!(
                formatter,
                "parameter/readout mismatch: {} trainable={} updates={} query={} vs {} trainable={} updates={} query={}",
                left_arm.as_str(),
                left_trainable,
                left_updates,
                left_query_components,
                right_arm.as_str(),
                right_trainable,
                right_updates,
                right_query_components
            ),
            Self::OptimizerUpdateBudgetMismatch {
                left_arm,
                right_arm,
                left_examples,
                right_examples,
                left_updates,
                right_updates,
            } => write!(
                formatter,
                "optimizer/update-budget mismatch: {} examples={} updates={} vs {} examples={} updates={}",
                left_arm.as_str(),
                left_examples,
                left_updates,
                right_arm.as_str(),
                right_examples,
                right_updates
            ),
            Self::MetricRegistryInvalid { reason } => {
                write!(formatter, "metric registry invalid: {reason}")
            }
            Self::ProtectedOrFinalSplit => formatter
                .write_str("TDI-25 evaluator rejected protected/final or unknown split identity"),
            Self::PairedUncertaintyInvalid { reason } => {
                write!(formatter, "paired uncertainty invalid: {reason}")
            }
            Self::FamilyStratifiedSynthesisInvalid { reason } => {
                write!(formatter, "family-stratified synthesis invalid: {reason}")
            }
            Self::FailureResourceAccountingInvalid { reason } => {
                write!(formatter, "failure/resource accounting invalid: {reason}")
            }
            Self::StageCPreflightInvalid { reason } => {
                write!(formatter, "Stage-C preflight invalid: {reason}")
            }
            Self::ReductionPointAblationInvalid { reason } => {
                write!(
                    formatter,
                    "torsor reduction-point ablation invalid: {reason}"
                )
            }
            Self::TorsorBridgeEquivalenceInvalid { reason } => {
                write!(formatter, "torsor bridge equivalence invalid: {reason}")
            }
            Self::ChiralGammaZeroAblationInvalid { reason } => {
                write!(formatter, "chiral gamma=0 ablation invalid: {reason}")
            }
            Self::ChiralParityShuffleControlInvalid { reason } => {
                write!(formatter, "chiral parity-shuffle control invalid: {reason}")
            }
            Self::TorsorStructureShuffleControlInvalid { reason } => {
                write!(
                    formatter,
                    "torsor structure-shuffle control invalid: {reason}"
                )
            }
            Self::G6OrthogonalBasisControlInvalid { reason } => {
                write!(formatter, "G6 orthogonal-basis control invalid: {reason}")
            }
            Self::PositionGeometryAblationInvalid { reason } => {
                write!(formatter, "position-geometry ablation invalid: {reason}")
            }
            Self::SequenceLengthScalingInvalid { reason } => {
                write!(formatter, "sequence-length scaling invalid: {reason}")
            }
            Self::DataVolumeScalingInvalid { reason } => {
                write!(formatter, "data-volume scaling invalid: {reason}")
            }
            Self::StageDAttributionAuditInvalid { reason } => {
                write!(formatter, "Stage-D attribution audit invalid: {reason}")
            }
            Self::MultiSeedReplicationInvalid { reason } => {
                write!(formatter, "multi-seed replication invalid: {reason}")
            }
            Self::InputNoiseRobustnessInvalid { reason } => {
                write!(formatter, "input/noise robustness invalid: {reason}")
            }
            Self::TranslationOriginStressInvalid { reason } => {
                write!(formatter, "translation/origin stress invalid: {reason}")
            }
        }
    }
}

impl std::error::Error for EvalError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::experimental::tdi25_tasks::{
        DataSplit, chiral_reflection_pair_in_split, mixed_geometry_pair_in_split,
        neutral_control_pair_in_split, seal_chiral_reflection, seal_mixed_geometry,
        seal_neutral_control, seal_torsor_transport, torsor_transport_pair_in_split,
    };

    #[test]
    fn contracts_and_non_trained_budget_are_pinned() {
        assert_eq!(EVALUATOR_ENVELOPE_CONTRACT, "tdi25-evaluator-envelope-v1");
        assert_eq!(T6_EVALUATOR_CONTRACT, "tdi25-t6-evaluator-v1");
        assert_eq!(C6_EVALUATOR_CONTRACT, "tdi25-c6-evaluator-v1");
        assert_eq!(G6_EVALUATOR_CONTRACT, "tdi25-g6-evaluator-v1");
        assert_eq!(READOUT_BUDGET_CONTRACT, "tdi25-readout-budget-v1");
        assert_eq!(GENERIC6_CONTRACT, "tdi25-generic6-control-v1");
        let budget = ReadoutBudget::matched_non_trained();
        assert_eq!(budget.max_cases, 64);
        assert_eq!(budget.max_readout_scalars_per_case, 2);
        assert_eq!(budget.updates, 0);
        let t6 = EvaluatorConfig::t6(DataSplit::Development);
        let c6 = EvaluatorConfig::c6(DataSplit::Development);
        let g6 = EvaluatorConfig::g6(DataSplit::Development);
        assert_eq!(t6.envelope_contract, c6.envelope_contract);
        assert_eq!(c6.envelope_contract, g6.envelope_contract);
        assert_eq!(t6.budget, c6.budget);
        assert_eq!(c6.budget, g6.budget);
        assert_ne!(t6.arm_contract, c6.arm_contract);
        assert_ne!(c6.arm_contract, g6.arm_contract);
        assert_ne!(t6.arm_contract, g6.arm_contract);
    }

    #[test]
    fn development_and_validation_paths_are_deterministic_and_sealed() {
        for split in [DataSplit::Development, DataSplit::Validation] {
            let pair = torsor_transport_pair_in_split(7, split).unwrap();
            let sealed = seal_torsor_transport(pair.transported, pair.oracle);
            let rendered = run_inference_callback(&sealed, |input| format!("{input:?}"));
            assert!(!rendered.contains("expected_score"));
            assert!(!rendered.contains("oracle"));

            let evaluate = || {
                let mut run = T6EvaluatorRun::open(EvaluatorConfig::t6(split)).unwrap();
                run.evaluate_torsor_transport(&sealed).unwrap().clone()
            };
            let first = evaluate();
            let second = evaluate();
            assert_eq!(first, second);
            assert!(first.outcome.matches_oracle);
            assert_eq!(first.source_torsor_contract, PINNED_SOURCE_CONTRACTS.torsor);
            assert_eq!(first.label_contract, PROTECTED_LABEL_CONTRACT);
        }
    }

    #[test]
    fn mixed_path_uses_only_the_torsor_component_and_keeps_oracle_sealed() {
        let pair = mixed_geometry_pair_in_split(5, DataSplit::Development).unwrap();
        let sealed = seal_mixed_geometry(pair.transformed, pair.transformed_oracle);
        let rendered = run_inference_callback(&sealed, |input| format!("{input:?}"));
        assert!(!rendered.contains("expected_torsor_score"));
        assert!(!rendered.contains("handedness"));
        let mut run = T6EvaluatorRun::open(EvaluatorConfig::t6(DataSplit::Development)).unwrap();
        let record = run.evaluate_mixed(&sealed).unwrap();
        assert_eq!(record.family, TaskFamily::Mixed);
        assert!(record.outcome.matches_oracle);
    }

    #[test]
    fn split_contract_and_budget_drift_fail_closed() {
        let pair = torsor_transport_pair_in_split(1, DataSplit::Validation).unwrap();
        let sealed = seal_torsor_transport(pair.original, pair.oracle);
        let mut run = T6EvaluatorRun::open(EvaluatorConfig::t6(DataSplit::Development)).unwrap();
        assert!(matches!(
            run.evaluate_torsor_transport(&sealed),
            Err(EvalError::SplitMismatch { .. })
        ));

        let mut invalid = EvaluatorConfig::t6(DataSplit::Development);
        invalid.budget.updates = 1;
        assert_eq!(T6EvaluatorRun::open(invalid), Err(EvalError::InvalidBudget));

        let mut unbounded = EvaluatorConfig::t6(DataSplit::Development);
        unbounded.budget.max_cases = MAX_CASES_PER_RUN + 1;
        assert_eq!(
            T6EvaluatorRun::open(unbounded),
            Err(EvalError::InvalidBudget)
        );

        let mut wide = EvaluatorConfig::t6(DataSplit::Development);
        wide.budget.max_readout_scalars_per_case = MAX_READOUT_SCALARS_PER_CASE + 1;
        assert_eq!(T6EvaluatorRun::open(wide), Err(EvalError::InvalidBudget));

        let mut drifted = EvaluatorConfig::t6(DataSplit::Development);
        drifted.arm_contract = "not-t6";
        assert!(matches!(
            T6EvaluatorRun::open(drifted),
            Err(EvalError::ContractMismatch("arm_contract"))
        ));
    }

    #[test]
    fn case_limit_is_enforced_before_scoring() {
        let pair = torsor_transport_pair_in_split(2, DataSplit::Development).unwrap();
        let first = seal_torsor_transport(pair.original, pair.oracle);
        let second = seal_torsor_transport(pair.transported, pair.oracle);
        let mut config = EvaluatorConfig::t6(DataSplit::Development);
        config.budget.max_cases = 1;
        let mut run = T6EvaluatorRun::open(config).unwrap();
        assert!(run.evaluate_torsor_transport(&first).is_ok());
        assert_eq!(
            run.evaluate_torsor_transport(&second),
            Err(EvalError::CaseBudgetExceeded)
        );
        assert_eq!(run.records().len(), 1);
    }

    #[test]
    fn mismatched_oracle_and_family_provenance_fail_closed() {
        let first_pair = torsor_transport_pair_in_split(3, DataSplit::Development).unwrap();
        let second_pair = torsor_transport_pair_in_split(4, DataSplit::Development).unwrap();
        let mismatched = seal_torsor_transport(first_pair.original, second_pair.oracle);
        let mut run = T6EvaluatorRun::open(EvaluatorConfig::t6(DataSplit::Development)).unwrap();
        assert_eq!(
            run.evaluate_torsor_transport(&mismatched),
            Err(EvalError::ContractMismatch("torsor_transport_case"))
        );

        let mut rewritten_id = first_pair.original;
        rewritten_id.case_id = second_pair.original.case_id;
        let forged = seal_torsor_transport(rewritten_id, second_pair.oracle);
        assert_eq!(
            run.evaluate_torsor_transport(&forged),
            Err(EvalError::ContractMismatch("torsor_transport_case"))
        );

        let mut wrong_family_input = first_pair.original;
        wrong_family_input.task_family = TaskFamily::Mixed;
        let wrong_family = seal_torsor_transport(wrong_family_input, first_pair.oracle);
        assert_eq!(
            run.evaluate_torsor_transport(&wrong_family),
            Err(EvalError::ContractMismatch("torsor_transport_case"))
        );

        let mixed_pair = mixed_geometry_pair_in_split(5, DataSplit::Development).unwrap();
        let mut drifted_oracle = mixed_pair.base_oracle;
        drifted_oracle.generator_contract = "not-mixed";
        let drifted = seal_mixed_geometry(mixed_pair.base, drifted_oracle);
        assert_eq!(
            run.evaluate_mixed(&drifted),
            Err(EvalError::ContractMismatch("mixed_case"))
        );

        let other_mixed = mixed_geometry_pair_in_split(6, DataSplit::Development).unwrap();
        let mut rewritten_mixed_id = mixed_pair.base;
        rewritten_mixed_id.case_id = other_mixed.base.case_id;
        let forged_mixed = seal_mixed_geometry(rewritten_mixed_id, other_mixed.base_oracle);
        assert_eq!(
            run.evaluate_mixed(&forged_mixed),
            Err(EvalError::ContractMismatch("mixed_case"))
        );
    }

    #[test]
    fn c6_development_and_validation_paths_are_deterministic_and_sealed() {
        for split in [DataSplit::Development, DataSplit::Validation] {
            let pair = chiral_reflection_pair_in_split(7, split).unwrap();
            let sealed = seal_chiral_reflection(pair.left, pair.left_oracle);
            let rendered = run_inference_callback(&sealed, |input| format!("{input:?}"));
            assert!(!rendered.contains("expected_score"));
            assert!(!rendered.contains("handedness"));
            assert!(!rendered.contains("oracle"));

            let evaluate = || {
                let mut run = C6EvaluatorRun::open(EvaluatorConfig::c6(split)).unwrap();
                run.evaluate_chiral_reflection(&sealed).unwrap().clone()
            };
            let first = evaluate();
            let second = evaluate();
            assert_eq!(first, second);
            assert!(first.outcome.matches_oracle);
            assert_eq!(first.source_chiral_contract, PINNED_SOURCE_CONTRACTS.chiral);
            assert_eq!(first.source_chiral_contract, CHIRAL_CONTRACT);
            assert_eq!(first.label_contract, PROTECTED_LABEL_CONTRACT);
            assert_eq!(first.envelope_contract, EVALUATOR_ENVELOPE_CONTRACT);
            assert_eq!(first.arm_contract, C6_EVALUATOR_CONTRACT);
            assert_eq!(first.budget_contract, READOUT_BUDGET_CONTRACT);
        }
    }

    #[test]
    fn c6_mixed_path_uses_only_the_chiral_component_and_keeps_oracle_sealed() {
        let pair = mixed_geometry_pair_in_split(5, DataSplit::Development).unwrap();
        let sealed = seal_mixed_geometry(pair.transformed, pair.transformed_oracle);
        let rendered = run_inference_callback(&sealed, |input| format!("{input:?}"));
        assert!(!rendered.contains("expected_chiral_score"));
        assert!(!rendered.contains("handedness"));
        let mut run = C6EvaluatorRun::open(EvaluatorConfig::c6(DataSplit::Development)).unwrap();
        let record = run.evaluate_mixed(&sealed).unwrap();
        assert_eq!(record.family, TaskFamily::Mixed);
        assert!(record.outcome.matches_oracle);
        assert_eq!(record.arm_contract, C6_EVALUATOR_CONTRACT);
    }

    #[test]
    fn c6_split_contract_and_budget_drift_fail_closed() {
        let pair = chiral_reflection_pair_in_split(1, DataSplit::Validation).unwrap();
        let sealed = seal_chiral_reflection(pair.right, pair.right_oracle);
        let mut run = C6EvaluatorRun::open(EvaluatorConfig::c6(DataSplit::Development)).unwrap();
        assert!(matches!(
            run.evaluate_chiral_reflection(&sealed),
            Err(EvalError::SplitMismatch { .. })
        ));

        let mut invalid = EvaluatorConfig::c6(DataSplit::Development);
        invalid.budget.updates = 1;
        assert_eq!(C6EvaluatorRun::open(invalid), Err(EvalError::InvalidBudget));

        let mut unbounded = EvaluatorConfig::c6(DataSplit::Development);
        unbounded.budget.max_cases = MAX_CASES_PER_RUN + 1;
        assert_eq!(
            C6EvaluatorRun::open(unbounded),
            Err(EvalError::InvalidBudget)
        );

        let mut wide = EvaluatorConfig::c6(DataSplit::Development);
        wide.budget.max_readout_scalars_per_case = MAX_READOUT_SCALARS_PER_CASE + 1;
        assert_eq!(C6EvaluatorRun::open(wide), Err(EvalError::InvalidBudget));

        let mut drifted = EvaluatorConfig::c6(DataSplit::Development);
        drifted.arm_contract = "not-c6";
        assert!(matches!(
            C6EvaluatorRun::open(drifted),
            Err(EvalError::ContractMismatch("arm_contract"))
        ));

        // T6 config must not open a C6 run (and vice versa).
        assert!(matches!(
            C6EvaluatorRun::open(EvaluatorConfig::t6(DataSplit::Development)),
            Err(EvalError::ContractMismatch("arm_contract"))
        ));
        assert!(matches!(
            T6EvaluatorRun::open(EvaluatorConfig::c6(DataSplit::Development)),
            Err(EvalError::ContractMismatch("arm_contract"))
        ));

        // Protected/final populations are rejected by the typed split parser.
        assert_eq!(
            DataSplit::parse("protected"),
            Err(crate::experimental::tdi25_tasks::Tdi25TaskError::UnknownSplitIdentity)
        );
        assert_eq!(
            DataSplit::parse("final"),
            Err(crate::experimental::tdi25_tasks::Tdi25TaskError::UnknownSplitIdentity)
        );
    }

    #[test]
    fn c6_case_limit_is_enforced_before_scoring() {
        let pair = chiral_reflection_pair_in_split(2, DataSplit::Development).unwrap();
        let first = seal_chiral_reflection(pair.right, pair.right_oracle);
        let second = seal_chiral_reflection(pair.left, pair.left_oracle);
        let mut config = EvaluatorConfig::c6(DataSplit::Development);
        config.budget.max_cases = 1;
        let mut run = C6EvaluatorRun::open(config).unwrap();
        assert!(run.evaluate_chiral_reflection(&first).is_ok());
        assert_eq!(
            run.evaluate_chiral_reflection(&second),
            Err(EvalError::CaseBudgetExceeded)
        );
        assert_eq!(run.records().len(), 1);
    }

    #[test]
    fn c6_mismatched_oracle_and_family_provenance_fail_closed() {
        let first_pair = chiral_reflection_pair_in_split(3, DataSplit::Development).unwrap();
        let second_pair = chiral_reflection_pair_in_split(4, DataSplit::Development).unwrap();
        let mismatched = seal_chiral_reflection(first_pair.right, second_pair.right_oracle);
        let mut run = C6EvaluatorRun::open(EvaluatorConfig::c6(DataSplit::Development)).unwrap();
        assert_eq!(
            run.evaluate_chiral_reflection(&mismatched),
            Err(EvalError::ContractMismatch("chiral_reflection_case"))
        );

        let mut rewritten_id = first_pair.right;
        rewritten_id.case_id = second_pair.right.case_id;
        let forged = seal_chiral_reflection(rewritten_id, second_pair.right_oracle);
        assert_eq!(
            run.evaluate_chiral_reflection(&forged),
            Err(EvalError::ContractMismatch("chiral_reflection_case"))
        );

        let mut wrong_family_input = first_pair.right;
        wrong_family_input.task_family = TaskFamily::Mixed;
        let wrong_family = seal_chiral_reflection(wrong_family_input, first_pair.right_oracle);
        assert_eq!(
            run.evaluate_chiral_reflection(&wrong_family),
            Err(EvalError::ContractMismatch("chiral_reflection_case"))
        );

        let mixed_pair = mixed_geometry_pair_in_split(5, DataSplit::Development).unwrap();
        let mut drifted_oracle = mixed_pair.base_oracle;
        drifted_oracle.generator_contract = "not-mixed";
        let drifted = seal_mixed_geometry(mixed_pair.base, drifted_oracle);
        assert_eq!(
            run.evaluate_mixed(&drifted),
            Err(EvalError::ContractMismatch("mixed_case"))
        );

        let other_mixed = mixed_geometry_pair_in_split(6, DataSplit::Development).unwrap();
        let mut rewritten_mixed_id = mixed_pair.base;
        rewritten_mixed_id.case_id = other_mixed.base.case_id;
        let forged_mixed = seal_mixed_geometry(rewritten_mixed_id, other_mixed.base_oracle);
        assert_eq!(
            run.evaluate_mixed(&forged_mixed),
            Err(EvalError::ContractMismatch("mixed_case"))
        );
    }

    #[test]
    fn c6_scores_through_tdi24_chiral_semantics() {
        let pair = chiral_reflection_pair_in_split(9, DataSplit::Validation).unwrap();
        let sealed = seal_chiral_reflection(pair.right, pair.right_oracle);
        let direct =
            chiral_arm_score(pair.right.query, pair.right.key, pair.right.weights).unwrap();
        let mut run = C6EvaluatorRun::open(EvaluatorConfig::c6(DataSplit::Validation)).unwrap();
        let record = run.evaluate_chiral_reflection(&sealed).unwrap();
        assert!(approximately_equal(record.outcome.score, direct));
        assert!(approximately_equal(
            record.outcome.score,
            pair.right_oracle.expected_score
        ));
        assert!(record.outcome.matches_oracle);
    }

    #[test]
    fn g6_development_and_validation_paths_are_deterministic_and_sealed() {
        for split in [DataSplit::Development, DataSplit::Validation] {
            let pair = neutral_control_pair_in_split(7, split).unwrap();
            let sealed = seal_neutral_control(pair.class_a, pair.class_a_oracle);
            let rendered = run_inference_callback(&sealed, |input| format!("{input:?}"));
            assert!(!rendered.contains("expected_score"));
            assert!(!rendered.contains("oracle"));
            assert!(!rendered.contains("ClassA"));
            assert!(!rendered.contains("ClassB"));

            let evaluate = || {
                let mut run = G6EvaluatorRun::open(EvaluatorConfig::g6(split)).unwrap();
                run.evaluate_neutral_control(&sealed).unwrap().clone()
            };
            let first = evaluate();
            let second = evaluate();
            assert_eq!(first, second);
            assert!(first.outcome.matches_oracle);
            assert_eq!(first.source_generic_contract, GENERIC6_CONTRACT);
            assert_eq!(first.source_generic_contract, "tdi25-generic6-control-v1");
            assert_eq!(first.label_contract, PROTECTED_LABEL_CONTRACT);
            assert_eq!(first.envelope_contract, EVALUATOR_ENVELOPE_CONTRACT);
            assert_eq!(first.arm_contract, G6_EVALUATOR_CONTRACT);
            assert_eq!(first.budget_contract, READOUT_BUDGET_CONTRACT);
            assert_eq!(first.family, TaskFamily::Neutral);
        }
    }

    #[test]
    fn g6_split_contract_and_budget_drift_fail_closed() {
        let pair = neutral_control_pair_in_split(1, DataSplit::Validation).unwrap();
        let sealed = seal_neutral_control(pair.class_b, pair.class_b_oracle);
        let mut run = G6EvaluatorRun::open(EvaluatorConfig::g6(DataSplit::Development)).unwrap();
        assert!(matches!(
            run.evaluate_neutral_control(&sealed),
            Err(EvalError::SplitMismatch { .. })
        ));

        let mut invalid = EvaluatorConfig::g6(DataSplit::Development);
        invalid.budget.updates = 1;
        assert_eq!(G6EvaluatorRun::open(invalid), Err(EvalError::InvalidBudget));

        let mut unbounded = EvaluatorConfig::g6(DataSplit::Development);
        unbounded.budget.max_cases = MAX_CASES_PER_RUN + 1;
        assert_eq!(
            G6EvaluatorRun::open(unbounded),
            Err(EvalError::InvalidBudget)
        );

        let mut wide = EvaluatorConfig::g6(DataSplit::Development);
        wide.budget.max_readout_scalars_per_case = MAX_READOUT_SCALARS_PER_CASE + 1;
        assert_eq!(G6EvaluatorRun::open(wide), Err(EvalError::InvalidBudget));

        let mut drifted = EvaluatorConfig::g6(DataSplit::Development);
        drifted.arm_contract = "not-g6";
        assert!(matches!(
            G6EvaluatorRun::open(drifted),
            Err(EvalError::ContractMismatch("arm_contract"))
        ));

        // Cross-arm configs must not open a G6 run (and vice versa).
        assert!(matches!(
            G6EvaluatorRun::open(EvaluatorConfig::t6(DataSplit::Development)),
            Err(EvalError::ContractMismatch("arm_contract"))
        ));
        assert!(matches!(
            G6EvaluatorRun::open(EvaluatorConfig::c6(DataSplit::Development)),
            Err(EvalError::ContractMismatch("arm_contract"))
        ));
        assert!(matches!(
            T6EvaluatorRun::open(EvaluatorConfig::g6(DataSplit::Development)),
            Err(EvalError::ContractMismatch("arm_contract"))
        ));
        assert!(matches!(
            C6EvaluatorRun::open(EvaluatorConfig::g6(DataSplit::Development)),
            Err(EvalError::ContractMismatch("arm_contract"))
        ));

        // Protected/final populations are rejected by the typed split parser.
        assert_eq!(
            DataSplit::parse("protected"),
            Err(crate::experimental::tdi25_tasks::Tdi25TaskError::UnknownSplitIdentity)
        );
        assert_eq!(
            DataSplit::parse("final"),
            Err(crate::experimental::tdi25_tasks::Tdi25TaskError::UnknownSplitIdentity)
        );
    }

    #[test]
    fn g6_case_limit_is_enforced_before_scoring() {
        let pair = neutral_control_pair_in_split(2, DataSplit::Development).unwrap();
        let first = seal_neutral_control(pair.class_a, pair.class_a_oracle);
        let second = seal_neutral_control(pair.class_b, pair.class_b_oracle);
        let mut config = EvaluatorConfig::g6(DataSplit::Development);
        config.budget.max_cases = 1;
        let mut run = G6EvaluatorRun::open(config).unwrap();
        assert!(run.evaluate_neutral_control(&first).is_ok());
        assert_eq!(
            run.evaluate_neutral_control(&second),
            Err(EvalError::CaseBudgetExceeded)
        );
        assert_eq!(run.records().len(), 1);
    }

    #[test]
    fn g6_mismatched_oracle_and_family_provenance_fail_closed() {
        let first_pair = neutral_control_pair_in_split(3, DataSplit::Development).unwrap();
        let second_pair = neutral_control_pair_in_split(4, DataSplit::Development).unwrap();
        let mismatched = seal_neutral_control(first_pair.class_a, second_pair.class_a_oracle);
        let mut run = G6EvaluatorRun::open(EvaluatorConfig::g6(DataSplit::Development)).unwrap();
        assert_eq!(
            run.evaluate_neutral_control(&mismatched),
            Err(EvalError::ContractMismatch("neutral_control_case"))
        );

        let mut rewritten_id = first_pair.class_a;
        rewritten_id.case_id = second_pair.class_a.case_id;
        let forged = seal_neutral_control(rewritten_id, second_pair.class_a_oracle);
        assert_eq!(
            run.evaluate_neutral_control(&forged),
            Err(EvalError::ContractMismatch("neutral_control_case"))
        );

        let mut wrong_family_input = first_pair.class_a;
        wrong_family_input.task_family = TaskFamily::Mixed;
        let wrong_family = seal_neutral_control(wrong_family_input, first_pair.class_a_oracle);
        assert_eq!(
            run.evaluate_neutral_control(&wrong_family),
            Err(EvalError::ContractMismatch("neutral_control_case"))
        );
    }

    #[test]
    fn g6_scores_through_generic6_control_semantics() {
        let pair = neutral_control_pair_in_split(9, DataSplit::Validation).unwrap();
        let sealed = seal_neutral_control(pair.class_b, pair.class_b_oracle);
        let direct = generic_arm_score(pair.class_b.query, pair.class_b.key).unwrap();
        let mut run = G6EvaluatorRun::open(EvaluatorConfig::g6(DataSplit::Validation)).unwrap();
        let record = run.evaluate_neutral_control(&sealed).unwrap();
        assert!(approximately_equal(record.outcome.score, direct));
        assert!(approximately_equal(
            record.outcome.score,
            pair.class_b_oracle.expected_score
        ));
        assert!(record.outcome.matches_oracle);
        assert_eq!(record.source_generic_contract, GENERIC6_CONTRACT);
    }

    #[test]
    fn g6_admits_only_neutral_generic6_cases() {
        // MixedGeometryInput exposes torsor+chiral carriers only. Under
        // GENERIC6_CONTRACT the G6 evaluator must not invent Generic6 fields
        // from mixed geometry, so NeutralControl is the sole G6 case path.
        let mixed = mixed_geometry_pair_in_split(5, DataSplit::Development).unwrap();
        assert_eq!(mixed.base.task_family, TaskFamily::Mixed);
        assert_eq!(mixed.base.generator_contract, MIXED_GEOMETRY_TASK_CONTRACT);

        let neutral = neutral_control_pair_in_split(5, DataSplit::Development).unwrap();
        assert_eq!(neutral.class_a.task_family, TaskFamily::Neutral);
        let score = score_neutral_control(&neutral.class_a).unwrap();
        assert!(score.is_finite());
        let mut run = G6EvaluatorRun::open(EvaluatorConfig::g6(DataSplit::Development)).unwrap();
        let sealed = seal_neutral_control(neutral.class_a, neutral.class_a_oracle);
        assert!(
            run.evaluate_neutral_control(&sealed)
                .unwrap()
                .outcome
                .matches_oracle
        );
    }

    #[test]
    fn parameter_readout_matcher_accepts_matched_reference_arms() {
        assert_eq!(
            PARAMETER_READOUT_MATCHER_CONTRACT,
            "tdi25-parameter-readout-matcher-v1"
        );
        let t6 = ParameterReadoutCapacity::reference_t6();
        let c6 = ParameterReadoutCapacity::reference_c6();
        let g6 = ParameterReadoutCapacity::reference_g6();
        assert_eq!(t6.trainable_parameters, 0);
        assert_eq!(c6.updates, 0);
        assert_eq!(g6.max_cases, MAX_CASES_PER_RUN);
        assert_eq!(
            g6.max_readout_scalars_per_case,
            MAX_READOUT_SCALARS_PER_CASE
        );
        assert_eq!(t6.query_components, 6);
        assert_eq!(c6.query_components, 6);
        assert_eq!(g6.score_components, 1);
        // T6 declares extra key/external geometry; C6/G6 do not.
        assert_eq!(t6.key_components, 9);
        assert_eq!(t6.external_geometry_components, 3);
        assert_eq!(c6.key_components, 6);
        assert_eq!(g6.external_geometry_components, 0);
        assert_eq!(ComparisonArm::T6.as_str(), "t6");
        assert_eq!(ComparisonArm::C6.as_str(), "c6");
        assert_eq!(ComparisonArm::G6.as_str(), "g6");

        for (left, right) in [(t6, g6), (c6, g6), (t6, c6)] {
            let matched = match_parameter_readouts(left, right).unwrap();
            assert_eq!(matched.trainable_parameters, 0);
            assert_eq!(matched.updates, 0);
            assert_eq!(matched.max_cases, MAX_CASES_PER_RUN);
            assert_eq!(
                matched.max_readout_scalars_per_case,
                MAX_READOUT_SCALARS_PER_CASE
            );
            assert_eq!(matched.query_components, 6);
            assert_eq!(matched.score_components, 1);
            assert_eq!(matched.left_arm, left.arm);
            assert_eq!(matched.right_arm, right.arm);
            assert_eq!(matched.matcher_contract, PARAMETER_READOUT_MATCHER_CONTRACT);
            assert_eq!(matched.budget_contract, READOUT_BUDGET_CONTRACT);
        }
    }

    #[test]
    fn parameter_readout_matcher_rejects_invalid_readout_budgets() {
        let t6 = ParameterReadoutCapacity::reference_t6();
        let mut inflated = ParameterReadoutCapacity::reference_g6();
        inflated.updates = 4;
        assert_eq!(
            match_parameter_readouts(t6, inflated),
            Err(EvalError::InvalidBudget)
        );

        let mut drifted_scalars = ParameterReadoutCapacity::reference_c6();
        drifted_scalars.max_readout_scalars_per_case = 8;
        assert_eq!(
            match_parameter_readouts(t6, drifted_scalars),
            Err(EvalError::InvalidBudget)
        );

        let mut drifted_cases = ParameterReadoutCapacity::reference_c6();
        drifted_cases.max_cases = MAX_CASES_PER_RUN + 8;
        assert_eq!(
            match_parameter_readouts(ParameterReadoutCapacity::reference_g6(), drifted_cases),
            Err(EvalError::InvalidBudget)
        );

        let mut oversized_left = ParameterReadoutCapacity::reference_t6();
        let mut oversized_right = ParameterReadoutCapacity::reference_c6();
        oversized_left.max_cases = MAX_CASES_PER_RUN + 1;
        oversized_right.max_cases = MAX_CASES_PER_RUN + 1;
        assert_eq!(
            match_parameter_readouts(oversized_left, oversized_right),
            Err(EvalError::InvalidBudget)
        );

        let mut wide_left = ParameterReadoutCapacity::reference_c6();
        let mut wide_right = ParameterReadoutCapacity::reference_g6();
        wide_left.max_readout_scalars_per_case = MAX_READOUT_SCALARS_PER_CASE + 1;
        wide_right.max_readout_scalars_per_case = MAX_READOUT_SCALARS_PER_CASE + 1;
        assert_eq!(
            match_parameter_readouts(wide_left, wide_right),
            Err(EvalError::InvalidBudget)
        );
    }

    #[test]
    fn parameter_readout_matcher_rejects_carrier_width_and_contract_drift() {
        let c6 = ParameterReadoutCapacity::reference_c6();
        let mut wide = ParameterReadoutCapacity::reference_g6();
        wide.query_components = 8;
        assert_eq!(
            match_parameter_readouts(c6, wide),
            Err(EvalError::ContractMismatch(
                "parameter_readout_carrier_accounting"
            ))
        );

        let mut score_drift = ParameterReadoutCapacity::reference_g6();
        score_drift.score_components = 2;
        assert_eq!(
            match_parameter_readouts(c6, score_drift),
            Err(EvalError::ContractMismatch(
                "parameter_readout_carrier_accounting"
            ))
        );

        let mut paired_drift_left = ParameterReadoutCapacity::reference_t6();
        let mut paired_drift_right = ParameterReadoutCapacity::reference_c6();
        paired_drift_left.key_components = 6;
        paired_drift_left.external_geometry_components = 0;
        paired_drift_right.key_components = 9;
        paired_drift_right.external_geometry_components = 3;
        assert_eq!(
            match_parameter_readouts(paired_drift_left, paired_drift_right),
            Err(EvalError::ContractMismatch(
                "parameter_readout_carrier_accounting"
            ))
        );

        for capacity in [
            ParameterReadoutCapacity::reference_t6(),
            ParameterReadoutCapacity::reference_c6(),
            ParameterReadoutCapacity::reference_g6(),
        ] {
            assert_eq!(
                match_parameter_readouts(capacity, capacity),
                Err(EvalError::ContractMismatch(
                    "parameter_readout_distinct_arms"
                ))
            );
        }

        let mut drifted = ParameterReadoutCapacity::reference_t6();
        drifted.matcher_contract = "not-a-matcher";
        assert_eq!(
            match_parameter_readouts(drifted, ParameterReadoutCapacity::reference_g6()),
            Err(EvalError::ContractMismatch(
                "parameter_readout_matcher_contract"
            ))
        );

        let mut budget_drift = ParameterReadoutCapacity::reference_t6();
        budget_drift.budget_contract = "not-a-budget";
        assert_eq!(
            match_parameter_readouts(budget_drift, ParameterReadoutCapacity::reference_c6()),
            Err(EvalError::ContractMismatch("budget_contract"))
        );
    }

    #[test]
    fn optimizer_update_budget_matcher_accepts_matched_non_trained_refs() {
        assert_eq!(
            OPTIMIZER_UPDATE_BUDGET_MATCHER_CONTRACT,
            "tdi25-optimizer-update-budget-matcher-v1"
        );
        assert_eq!(StoppingRule::ExhaustExamples.as_str(), "exhaust_examples");
        assert_eq!(StoppingRule::FixedUpdates.as_str(), "fixed_updates");
        let t6 = OptimizerUpdateBudget::reference_t6();
        let c6 = OptimizerUpdateBudget::reference_c6();
        let g6 = OptimizerUpdateBudget::reference_g6();
        assert_eq!(t6.examples, 0);
        assert_eq!(c6.updates, NON_TRAINED_UPDATE_BUDGET);
        assert_eq!(g6.ordering_id, NON_TRAINED_ORDERING_ID);
        assert_eq!(t6.stopping, StoppingRule::ExhaustExamples);
        assert_eq!(
            c6.matcher_contract,
            OPTIMIZER_UPDATE_BUDGET_MATCHER_CONTRACT
        );
        assert_eq!(g6.arm, ComparisonArm::G6);

        for (left, right) in [(t6, g6), (c6, g6), (t6, c6)] {
            let matched = match_optimizer_update_budgets(left, right).unwrap();
            assert_eq!(matched.examples, 0);
            assert_eq!(matched.updates, 0);
            assert_eq!(matched.ordering_id, NON_TRAINED_ORDERING_ID);
            assert_eq!(matched.stopping, StoppingRule::ExhaustExamples);
            assert_eq!(matched.left_arm, left.arm);
            assert_eq!(matched.right_arm, right.arm);
            assert_eq!(
                matched.matcher_contract,
                OPTIMIZER_UPDATE_BUDGET_MATCHER_CONTRACT
            );
        }
    }

    #[test]
    fn optimizer_update_budget_matcher_rejects_unpaired_budgets() {
        let t6 = OptimizerUpdateBudget::reference_t6();
        let mut other_examples = OptimizerUpdateBudget::reference_c6();
        other_examples.examples = 8;
        assert_eq!(
            match_optimizer_update_budgets(t6, other_examples),
            Err(EvalError::OptimizerUpdateBudgetMismatch {
                left_arm: ComparisonArm::T6,
                right_arm: ComparisonArm::C6,
                left_examples: 0,
                right_examples: 8,
                left_updates: 0,
                right_updates: 0,
            })
        );

        let mut other_updates = OptimizerUpdateBudget::reference_g6();
        other_updates.examples = 8;
        other_updates.updates = 4;
        other_updates.stopping = StoppingRule::FixedUpdates;
        assert!(matches!(
            match_optimizer_update_budgets(t6, other_updates),
            Err(EvalError::OptimizerUpdateBudgetMismatch { .. })
        ));

        let mut other_order = OptimizerUpdateBudget::reference_c6();
        other_order.ordering_id = 2;
        assert!(matches!(
            match_optimizer_update_budgets(t6, other_order),
            Err(EvalError::OptimizerUpdateBudgetMismatch { .. })
        ));

        let mut other_stopping = OptimizerUpdateBudget::reference_g6();
        other_stopping.examples = 8;
        other_stopping.updates = 4;
        other_stopping.stopping = StoppingRule::FixedUpdates;
        assert!(matches!(
            match_optimizer_update_budgets(t6, other_stopping),
            Err(EvalError::OptimizerUpdateBudgetMismatch { .. })
        ));

        // Matched trained-shaped budgets are accepted so later trained paths
        // cannot bypass the matcher; mismatch still fails closed.
        let mut left_trained = OptimizerUpdateBudget::reference_t6();
        let mut right_trained = OptimizerUpdateBudget::reference_c6();
        left_trained.examples = 16;
        right_trained.examples = 16;
        left_trained.updates = 8;
        right_trained.updates = 8;
        left_trained.ordering_id = 99;
        right_trained.ordering_id = 99;
        left_trained.stopping = StoppingRule::FixedUpdates;
        right_trained.stopping = StoppingRule::FixedUpdates;
        let matched = match_optimizer_update_budgets(left_trained, right_trained).unwrap();
        assert_eq!(matched.examples, 16);
        assert_eq!(matched.updates, 8);
        assert_eq!(matched.ordering_id, 99);
        assert_eq!(matched.stopping, StoppingRule::FixedUpdates);

        right_trained.updates = 7;
        assert!(matches!(
            match_optimizer_update_budgets(left_trained, right_trained),
            Err(EvalError::OptimizerUpdateBudgetMismatch { .. })
        ));
    }

    #[test]
    fn optimizer_update_budget_matcher_rejects_internally_invalid_budgets() {
        let mut left = OptimizerUpdateBudget::reference_t6();
        let mut right = OptimizerUpdateBudget::reference_c6();
        // FixedUpdates with zero updates is unusable even when paired equal.
        left.stopping = StoppingRule::FixedUpdates;
        right.stopping = StoppingRule::FixedUpdates;
        left.examples = 16;
        right.examples = 16;
        left.updates = 0;
        right.updates = 0;
        assert_eq!(
            match_optimizer_update_budgets(left, right),
            Err(EvalError::InvalidBudget)
        );

        // FixedUpdates with zero examples is unusable.
        left.updates = 8;
        right.updates = 8;
        left.examples = 0;
        right.examples = 0;
        assert_eq!(
            match_optimizer_update_budgets(left, right),
            Err(EvalError::InvalidBudget)
        );

        // ExhaustExamples cannot carry a non-zero update budget.
        left = OptimizerUpdateBudget::reference_t6();
        right = OptimizerUpdateBudget::reference_g6();
        left.updates = 4;
        right.updates = 4;
        assert_eq!(
            match_optimizer_update_budgets(left, right),
            Err(EvalError::InvalidBudget)
        );
    }

    #[test]
    fn optimizer_update_budget_matcher_rejects_contract_and_arm_drift() {
        for budget in [
            OptimizerUpdateBudget::reference_t6(),
            OptimizerUpdateBudget::reference_c6(),
            OptimizerUpdateBudget::reference_g6(),
        ] {
            assert_eq!(
                match_optimizer_update_budgets(budget, budget),
                Err(EvalError::ContractMismatch(
                    "optimizer_update_budget_distinct_arms"
                ))
            );
        }

        let mut drifted = OptimizerUpdateBudget::reference_t6();
        drifted.matcher_contract = "not-an-opt-budget";
        assert_eq!(
            match_optimizer_update_budgets(drifted, OptimizerUpdateBudget::reference_g6()),
            Err(EvalError::ContractMismatch(
                "optimizer_update_budget_matcher_contract"
            ))
        );
    }

    #[test]
    fn metric_registry_pins_primary_families_cross_summary_and_secondaries() {
        assert_eq!(METRIC_REGISTRY_CONTRACT, "tdi25-metric-registry-v1");
        assert_eq!(
            PRIMARY_FAMILY_TORSOR_FAVORABLE,
            "family_paired_task_accuracy_torsor_favorable"
        );
        assert_eq!(CROSS_FAMILY_PAIRED_SUMMARY, "cross_family_paired_summary");
        assert_eq!(
            PINNED_PRIMARY_FAMILY_METRICS.len(),
            MAX_PRIMARY_FAMILY_METRICS
        );
        assert_eq!(
            PINNED_SECONDARY_DIAGNOSTICS.len(),
            MAX_SECONDARY_DIAGNOSTICS
        );
        assert_eq!(
            PrimaryFamilyMetricId::admitted_set(),
            PINNED_PRIMARY_FAMILY_METRICS
        );
        assert_eq!(
            SecondaryDiagnosticId::admitted_set(),
            PINNED_SECONDARY_DIAGNOSTICS
        );

        let pinned = MetricRegistry::pinned();
        assert_eq!(pinned.primary_family_metrics, PINNED_PRIMARY_FAMILY_METRICS);
        assert_eq!(
            pinned.cross_family_summary,
            CrossFamilySummaryMetricId::CrossFamilyPairedSummary
        );
        assert_eq!(pinned.secondaries, PINNED_SECONDARY_DIAGNOSTICS);
        assert_eq!(pinned.registry_contract, METRIC_REGISTRY_CONTRACT);
        assert!(pinned.experimental_non_final);
        validate_metric_registry(&pinned).unwrap();

        let frozen = freeze_metric_registry(
            PINNED_PRIMARY_FAMILY_METRICS,
            Some(CrossFamilySummaryMetricId::CrossFamilyPairedSummary),
            PINNED_SECONDARY_DIAGNOSTICS,
        )
        .unwrap();
        assert_eq!(frozen, pinned);

        assert_eq!(
            pinned.primary_family_metrics[0].as_str(),
            PRIMARY_FAMILY_TORSOR_FAVORABLE
        );
        assert_eq!(
            pinned.primary_family_metrics[0].family(),
            TaskFamily::TorsorFavorable
        );
        assert_eq!(
            pinned.primary_family_metrics[1].as_str(),
            PRIMARY_FAMILY_CHIRAL_FAVORABLE
        );
        assert_eq!(
            pinned.primary_family_metrics[2].as_str(),
            PRIMARY_FAMILY_MIXED
        );
        assert_eq!(
            pinned.primary_family_metrics[3].as_str(),
            PRIMARY_FAMILY_NEUTRAL
        );
        assert_eq!(
            pinned.cross_family_summary.as_str(),
            CROSS_FAMILY_PAIRED_SUMMARY
        );
        assert_eq!(
            pinned.secondaries[0].as_str(),
            SECONDARY_PAIRED_OUTCOME_DIFFERENCE
        );
        assert_eq!(
            pinned.secondaries[1].as_str(),
            SECONDARY_TORSOR_TRANSPORT_REDUCTION_POINT_IDENTITY_ERROR
        );
        assert_eq!(
            pinned.secondaries[2].as_str(),
            SECONDARY_CHIRAL_MIRROR_SWAP_PARITY_IDENTITY_ERROR
        );
        assert_eq!(
            pinned.secondaries[3].as_str(),
            SECONDARY_G6_ATTRIBUTION_CONTRAST
        );
        assert_eq!(
            pinned.secondaries[4].as_str(),
            SECONDARY_CALIBRATION_STABILITY
        );
        assert_eq!(
            pinned.secondaries[5].as_str(),
            SECONDARY_OP_COUNT_MEMORY_RUNTIME
        );

        for split in [DataSplit::Development, DataSplit::Validation] {
            assert!(pinned.admit_split(split).is_ok());
            let t6 = EvaluatorConfig::t6(split);
            assert_eq!(t6.metric_registry, pinned);
            assert!(T6EvaluatorRun::open(t6).is_ok());
            assert!(C6EvaluatorRun::open(EvaluatorConfig::c6(split)).is_ok());
            assert!(G6EvaluatorRun::open(EvaluatorConfig::g6(split)).is_ok());
        }

        assert_eq!(
            parse_primary_family_metric_id(PRIMARY_FAMILY_MIXED).unwrap(),
            PrimaryFamilyMetricId::Mixed
        );
        assert_eq!(
            parse_cross_family_summary_metric_id(CROSS_FAMILY_PAIRED_SUMMARY).unwrap(),
            CrossFamilySummaryMetricId::CrossFamilyPairedSummary
        );
        assert_eq!(
            parse_secondary_diagnostic_id(SECONDARY_G6_ATTRIBUTION_CONTRAST).unwrap(),
            SecondaryDiagnosticId::G6AttributionContrast
        );
    }

    #[test]
    fn metric_registry_rejects_empty_primary_duplicates_unknowns_and_sequence_drift() {
        assert_eq!(
            freeze_metric_registry(
                &[],
                Some(CrossFamilySummaryMetricId::CrossFamilyPairedSummary),
                PINNED_SECONDARY_DIAGNOSTICS,
            ),
            Err(EvalError::MetricRegistryInvalid {
                reason: "empty_primary",
            })
        );
        assert_eq!(
            freeze_metric_registry(
                PINNED_PRIMARY_FAMILY_METRICS,
                None,
                PINNED_SECONDARY_DIAGNOSTICS,
            ),
            Err(EvalError::MetricRegistryInvalid {
                reason: "empty_cross_family_summary",
            })
        );
        assert_eq!(
            parse_primary_family_metric_id(""),
            Err(EvalError::MetricRegistryInvalid {
                reason: "empty_primary",
            })
        );
        assert_eq!(
            parse_primary_family_metric_id("invented_primary"),
            Err(EvalError::MetricRegistryInvalid {
                reason: "unknown_primary",
            })
        );
        assert_eq!(
            parse_cross_family_summary_metric_id(""),
            Err(EvalError::MetricRegistryInvalid {
                reason: "empty_cross_family_summary",
            })
        );
        assert_eq!(
            parse_cross_family_summary_metric_id("invented_summary"),
            Err(EvalError::MetricRegistryInvalid {
                reason: "unknown_cross_family_summary",
            })
        );
        assert_eq!(
            parse_secondary_diagnostic_id("invented_secondary"),
            Err(EvalError::MetricRegistryInvalid {
                reason: "unknown_secondary",
            })
        );

        const DUPLICATE_SECONDARIES: &[SecondaryDiagnosticId] = &[
            SecondaryDiagnosticId::PairedOutcomeDifference,
            SecondaryDiagnosticId::TorsorTransportReductionPointIdentityError,
            SecondaryDiagnosticId::PairedOutcomeDifference,
        ];
        assert_eq!(
            freeze_metric_registry(
                PINNED_PRIMARY_FAMILY_METRICS,
                Some(CrossFamilySummaryMetricId::CrossFamilyPairedSummary),
                DUPLICATE_SECONDARIES,
            ),
            Err(EvalError::MetricRegistryInvalid {
                reason: "duplicate_secondary",
            })
        );

        const DUPLICATE_PRIMARIES: &[PrimaryFamilyMetricId] = &[
            PrimaryFamilyMetricId::TorsorFavorable,
            PrimaryFamilyMetricId::TorsorFavorable,
            PrimaryFamilyMetricId::Mixed,
            PrimaryFamilyMetricId::Neutral,
        ];
        assert_eq!(
            freeze_metric_registry(
                DUPLICATE_PRIMARIES,
                Some(CrossFamilySummaryMetricId::CrossFamilyPairedSummary),
                PINNED_SECONDARY_DIAGNOSTICS,
            ),
            Err(EvalError::MetricRegistryInvalid {
                reason: "duplicate_primary",
            })
        );

        const OVERSIZED_SECONDARIES: &[SecondaryDiagnosticId] = &[
            SecondaryDiagnosticId::PairedOutcomeDifference,
            SecondaryDiagnosticId::TorsorTransportReductionPointIdentityError,
            SecondaryDiagnosticId::ChiralMirrorSwapParityIdentityError,
            SecondaryDiagnosticId::G6AttributionContrast,
            SecondaryDiagnosticId::CalibrationStability,
            SecondaryDiagnosticId::OpCountMemoryRuntime,
            SecondaryDiagnosticId::PairedOutcomeDifference,
        ];
        assert_eq!(
            freeze_metric_registry(
                PINNED_PRIMARY_FAMILY_METRICS,
                Some(CrossFamilySummaryMetricId::CrossFamilyPairedSummary),
                OVERSIZED_SECONDARIES,
            ),
            Err(EvalError::MetricRegistryInvalid {
                reason: "too_many_secondaries",
            })
        );

        const TRUNCATED_SECONDARIES: &[SecondaryDiagnosticId] = &[
            SecondaryDiagnosticId::PairedOutcomeDifference,
            SecondaryDiagnosticId::TorsorTransportReductionPointIdentityError,
            SecondaryDiagnosticId::ChiralMirrorSwapParityIdentityError,
            SecondaryDiagnosticId::G6AttributionContrast,
            SecondaryDiagnosticId::CalibrationStability,
        ];
        assert_eq!(
            freeze_metric_registry(
                PINNED_PRIMARY_FAMILY_METRICS,
                Some(CrossFamilySummaryMetricId::CrossFamilyPairedSummary),
                TRUNCATED_SECONDARIES,
            ),
            Err(EvalError::MetricRegistryInvalid {
                reason: "secondary_sequence_mismatch",
            })
        );
        assert_eq!(
            freeze_metric_registry(
                PINNED_PRIMARY_FAMILY_METRICS,
                Some(CrossFamilySummaryMetricId::CrossFamilyPairedSummary),
                &[],
            ),
            Err(EvalError::MetricRegistryInvalid {
                reason: "secondary_sequence_mismatch",
            })
        );

        const REORDERED_SECONDARIES: &[SecondaryDiagnosticId] = &[
            SecondaryDiagnosticId::TorsorTransportReductionPointIdentityError,
            SecondaryDiagnosticId::PairedOutcomeDifference,
            SecondaryDiagnosticId::ChiralMirrorSwapParityIdentityError,
            SecondaryDiagnosticId::G6AttributionContrast,
            SecondaryDiagnosticId::CalibrationStability,
            SecondaryDiagnosticId::OpCountMemoryRuntime,
        ];
        assert_eq!(
            freeze_metric_registry(
                PINNED_PRIMARY_FAMILY_METRICS,
                Some(CrossFamilySummaryMetricId::CrossFamilyPairedSummary),
                REORDERED_SECONDARIES,
            ),
            Err(EvalError::MetricRegistryInvalid {
                reason: "secondary_sequence_mismatch",
            })
        );

        const REORDERED_PRIMARIES: &[PrimaryFamilyMetricId] = &[
            PrimaryFamilyMetricId::ChiralFavorable,
            PrimaryFamilyMetricId::TorsorFavorable,
            PrimaryFamilyMetricId::Mixed,
            PrimaryFamilyMetricId::Neutral,
        ];
        assert_eq!(
            freeze_metric_registry(
                REORDERED_PRIMARIES,
                Some(CrossFamilySummaryMetricId::CrossFamilyPairedSummary),
                PINNED_SECONDARY_DIAGNOSTICS,
            ),
            Err(EvalError::MetricRegistryInvalid {
                reason: "primary_sequence_mismatch",
            })
        );

        const TRUNCATED_PRIMARIES: &[PrimaryFamilyMetricId] = &[
            PrimaryFamilyMetricId::TorsorFavorable,
            PrimaryFamilyMetricId::ChiralFavorable,
            PrimaryFamilyMetricId::Mixed,
        ];
        assert_eq!(
            freeze_metric_registry(
                TRUNCATED_PRIMARIES,
                Some(CrossFamilySummaryMetricId::CrossFamilyPairedSummary),
                PINNED_SECONDARY_DIAGNOSTICS,
            ),
            Err(EvalError::MetricRegistryInvalid {
                reason: "primary_sequence_mismatch",
            })
        );

        let malformed = MetricRegistry {
            primary_family_metrics: &[],
            ..MetricRegistry::pinned()
        };
        assert_eq!(
            malformed.admit_split(DataSplit::Development),
            Err(EvalError::MetricRegistryInvalid {
                reason: "empty_primary",
            })
        );
        let mut config = EvaluatorConfig::t6(DataSplit::Development);
        config.metric_registry = malformed;
        assert_eq!(
            T6EvaluatorRun::open(config),
            Err(EvalError::MetricRegistryInvalid {
                reason: "empty_primary",
            })
        );

        let mut drifted = MetricRegistry::pinned();
        drifted.registry_contract = "not-a-metric-registry";
        assert_eq!(
            validate_metric_registry(&drifted),
            Err(EvalError::ContractMismatch("metric_registry_contract"))
        );
        let mut config = EvaluatorConfig::c6(DataSplit::Validation);
        config.metric_registry = drifted;
        assert_eq!(
            C6EvaluatorRun::open(config),
            Err(EvalError::ContractMismatch("metric_registry_contract"))
        );

        let mut non_experimental = MetricRegistry::pinned();
        non_experimental.experimental_non_final = false;
        assert_eq!(
            validate_metric_registry(&non_experimental),
            Err(EvalError::MetricRegistryInvalid {
                reason: "experimental_non_final_required",
            })
        );
    }

    #[test]
    fn metric_registry_contract_is_retained_on_scored_records() {
        let pair = torsor_transport_pair_in_split(26, DataSplit::Development).unwrap();
        let sealed = seal_torsor_transport(pair.transported, pair.oracle);
        let mut run = T6EvaluatorRun::open(EvaluatorConfig::t6(DataSplit::Development)).unwrap();
        let record = run.evaluate_torsor_transport(&sealed).unwrap();
        assert_eq!(record.metric_registry_contract, METRIC_REGISTRY_CONTRACT);

        let chiral = chiral_reflection_pair_in_split(26, DataSplit::Development).unwrap();
        let sealed = seal_chiral_reflection(chiral.left, chiral.left_oracle);
        let mut c6 = C6EvaluatorRun::open(EvaluatorConfig::c6(DataSplit::Development)).unwrap();
        let record = c6.evaluate_chiral_reflection(&sealed).unwrap();
        assert_eq!(record.metric_registry_contract, METRIC_REGISTRY_CONTRACT);

        let neutral = neutral_control_pair_in_split(26, DataSplit::Development).unwrap();
        let sealed = seal_neutral_control(neutral.class_a, neutral.class_a_oracle);
        let mut g6 = G6EvaluatorRun::open(EvaluatorConfig::g6(DataSplit::Development)).unwrap();
        let record = g6.evaluate_neutral_control(&sealed).unwrap();
        assert_eq!(record.metric_registry_contract, METRIC_REGISTRY_CONTRACT);
    }

    fn revealed(
        split: DataSplit,
        family: TaskFamily,
        seed_block: u64,
        case_id: u64,
        bit: bool,
    ) -> RevealedMatchOutcome {
        RevealedMatchOutcome::from_matches_oracle(split, family, seed_block, case_id, bit)
    }

    fn matches(
        split: DataSplit,
        family: TaskFamily,
        seed_block: u64,
        bits: &[(u64, bool)],
    ) -> Vec<RevealedMatchOutcome> {
        bits.iter()
            .map(|(case_id, bit)| revealed(split, family, seed_block, *case_id, *bit))
            .collect()
    }

    #[test]
    fn paired_uncertainty_happy_path_by_seed_block() {
        assert_eq!(PAIRED_UNCERTAINTY_CONTRACT, "tdi25-paired-uncertainty-v1");
        assert_eq!(UncertaintyMethod::WilsonScore.as_str(), "wilson_score");
        assert_eq!(
            UncertaintyMethod::BoundedHoeffdingPairedDifference.as_str(),
            "bounded_hoeffding_paired_difference"
        );
        assert_eq!(
            primary_family_metric_for(TaskFamily::TorsorFavorable),
            PrimaryFamilyMetricId::TorsorFavorable
        );

        // T6: T T F F  -> accuracy 0.5
        // C6: T T T F  -> accuracy 0.75
        // diffs: 0,0,1,0 -> mean 0.25
        let family = TaskFamily::TorsorFavorable;
        let seed_block = 17;
        let t6 = matches(
            DataSplit::Development,
            family,
            seed_block,
            &[(1, true), (2, true), (3, false), (4, false)],
        );
        let c6 = matches(
            DataSplit::Development,
            family,
            seed_block,
            &[(1, true), (2, true), (3, true), (4, false)],
        );
        let registry = MetricRegistry::pinned();
        let summary = summarize_paired_uncertainty_by_seed_block(
            DataSplit::Development,
            family,
            seed_block,
            &t6,
            &c6,
            &registry,
        )
        .unwrap();

        assert_eq!(summary.n_pairs, 4);
        assert_eq!(summary.family, family);
        assert_eq!(summary.seed_block, seed_block);
        assert!((summary.t6_accuracy - 0.5).abs() < 1e-12);
        assert!((summary.c6_accuracy - 0.75).abs() < 1e-12);
        assert!((summary.paired_difference_mean - 0.25).abs() < 1e-12);
        assert_eq!(
            summary.primary_family_metric,
            PrimaryFamilyMetricId::TorsorFavorable
        );
        assert_eq!(
            summary.cross_family_summary,
            CrossFamilySummaryMetricId::CrossFamilyPairedSummary
        );
        assert_eq!(
            summary.secondary_paired_outcome_difference,
            SecondaryDiagnosticId::PairedOutcomeDifference
        );
        assert_eq!(summary.uncertainty_contract, PAIRED_UNCERTAINTY_CONTRACT);
        assert_eq!(summary.metric_registry_contract, METRIC_REGISTRY_CONTRACT);
        assert!(summary.experimental_non_final);
        assert_eq!(
            summary.paired_difference_ci.method,
            UncertaintyMethod::BoundedHoeffdingPairedDifference
        );
        assert_eq!(
            summary.t6_accuracy_ci.method,
            UncertaintyMethod::WilsonScore
        );
        assert_eq!(
            summary.c6_accuracy_ci.method,
            UncertaintyMethod::WilsonScore
        );
        assert!(summary.paired_difference_ci.lower <= summary.paired_difference_mean);
        assert!(summary.paired_difference_ci.upper >= summary.paired_difference_mean);
        assert!(summary.paired_difference_ci.lower < summary.paired_difference_ci.upper);

        let t6_val = matches(
            DataSplit::Validation,
            family,
            seed_block,
            &[(1, true), (2, true), (3, false), (4, false)],
        );
        let c6_val = matches(
            DataSplit::Validation,
            family,
            seed_block,
            &[(1, true), (2, true), (3, true), (4, false)],
        );
        let validation = summarize_paired_uncertainty_by_seed_block(
            DataSplit::Validation,
            family,
            seed_block,
            &t6_val,
            &c6_val,
            &registry,
        )
        .unwrap();
        assert_eq!(validation.split, DataSplit::Validation);
    }

    #[test]
    fn paired_uncertainty_boundary_intervals_remain_non_degenerate() {
        let registry = MetricRegistry::pinned();
        let family = TaskFamily::ChiralFavorable;
        let seed_block = 3;
        let ties: Vec<_> = (0..MAX_CASES_PER_RUN)
            .map(|case_id| revealed(DataSplit::Development, family, seed_block, case_id, true))
            .collect();
        let all_ties = summarize_paired_uncertainty_by_seed_block(
            DataSplit::Development,
            family,
            seed_block,
            &ties,
            &ties,
            &registry,
        )
        .unwrap();
        assert_eq!(all_ties.paired_difference_mean, 0.0);
        assert!(all_ties.paired_difference_ci.lower < 0.0);
        assert!(all_ties.paired_difference_ci.upper > 0.0);

        let t6: Vec<_> = (0..MAX_CASES_PER_RUN)
            .map(|case_id| revealed(DataSplit::Development, family, seed_block, case_id, false))
            .collect();
        let c6: Vec<_> = (0..MAX_CASES_PER_RUN)
            .map(|case_id| revealed(DataSplit::Development, family, seed_block, case_id, true))
            .collect();
        let all_c6_wins = summarize_paired_uncertainty_by_seed_block(
            DataSplit::Development,
            family,
            seed_block,
            &t6,
            &c6,
            &registry,
        )
        .unwrap();
        assert_eq!(all_c6_wins.paired_difference_mean, 1.0);
        assert!(all_c6_wins.paired_difference_ci.lower < 1.0);
        assert_eq!(all_c6_wins.paired_difference_ci.upper, 1.0);
    }

    #[test]
    fn paired_uncertainty_rejects_empty_mismatch_nan_registry_and_protected() {
        let registry = MetricRegistry::pinned();
        let family = TaskFamily::Mixed;
        let seed_block = 9;
        let t6 = matches(
            DataSplit::Development,
            family,
            seed_block,
            &[(1, true), (2, false)],
        );
        let c6 = matches(
            DataSplit::Development,
            family,
            seed_block,
            &[(1, true), (2, true)],
        );

        assert_eq!(
            summarize_paired_uncertainty_by_seed_block(
                DataSplit::Development,
                family,
                seed_block,
                &[],
                &c6,
                &registry,
            ),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "empty_pairs",
            })
        );
        assert_eq!(
            summarize_paired_uncertainty_by_seed_block(
                DataSplit::Development,
                family,
                seed_block,
                &t6,
                &[],
                &registry,
            ),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "empty_pairs",
            })
        );
        assert_eq!(
            summarize_paired_uncertainty_by_seed_block(
                DataSplit::Development,
                family,
                seed_block,
                &t6,
                &matches(DataSplit::Development, family, seed_block, &[(1, true)]),
                &registry,
            ),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "length_mismatch",
            })
        );
        assert_eq!(
            summarize_paired_uncertainty_by_seed_block(
                DataSplit::Development,
                family,
                seed_block,
                &t6,
                &c6,
                &registry,
            )
            .and_then(|_| {
                ConfidenceInterval::checked(
                    f64::NAN,
                    1.0,
                    PAIRED_UNCERTAINTY_LEVEL,
                    UncertaintyMethod::WilsonScore,
                )
            }),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "non_finite",
            })
        );
        assert_eq!(
            ConfidenceInterval::checked(
                0.0,
                f64::INFINITY,
                PAIRED_UNCERTAINTY_LEVEL,
                UncertaintyMethod::BoundedHoeffdingPairedDifference,
            ),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "non_finite",
            })
        );

        let mut drifted = MetricRegistry::pinned();
        drifted.registry_contract = "not-paired-uncertainty-registry";
        assert_eq!(
            summarize_paired_uncertainty_by_seed_block(
                DataSplit::Development,
                family,
                seed_block,
                &t6,
                &c6,
                &drifted,
            ),
            Err(EvalError::ContractMismatch("metric_registry_contract"))
        );

        let malformed = MetricRegistry {
            secondaries: &[],
            ..MetricRegistry::pinned()
        };
        assert_eq!(
            summarize_paired_uncertainty_by_seed_block(
                DataSplit::Development,
                family,
                seed_block,
                &t6,
                &c6,
                &malformed,
            ),
            Err(EvalError::MetricRegistryInvalid {
                reason: "secondary_sequence_mismatch",
            })
        );

        assert_eq!(
            summarize_paired_uncertainty_by_seed_block(
                DataSplit::Development,
                family,
                seed_block,
                &matches(DataSplit::Development, family, seed_block, &[(1, true)]),
                &matches(DataSplit::Development, family, seed_block, &[(1, false)]),
                &registry,
            ),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "insufficient_pairs",
            })
        );

        let oversized: Vec<_> = (0..=MAX_CASES_PER_RUN)
            .map(|case_id| revealed(DataSplit::Development, family, seed_block, case_id, true))
            .collect();
        assert_eq!(
            summarize_paired_uncertainty_by_seed_block(
                DataSplit::Development,
                family,
                seed_block,
                &oversized,
                &oversized,
                &registry,
            ),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "too_many_pairs",
            })
        );

        let wrong_family = matches(
            DataSplit::Development,
            TaskFamily::Neutral,
            seed_block,
            &[(1, true), (2, false)],
        );
        assert_eq!(
            summarize_paired_uncertainty_by_seed_block(
                DataSplit::Development,
                family,
                seed_block,
                &wrong_family,
                &c6,
                &registry,
            ),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "family_mismatch",
            })
        );

        let wrong_seed = matches(
            DataSplit::Development,
            family,
            seed_block + 1,
            &[(1, true), (2, true)],
        );
        assert_eq!(
            summarize_paired_uncertainty_by_seed_block(
                DataSplit::Development,
                family,
                seed_block,
                &t6,
                &wrong_seed,
                &registry,
            ),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "seed_block_mismatch",
            })
        );

        assert_eq!(
            parse_non_final_split("protected"),
            Err(EvalError::ProtectedOrFinalSplit)
        );
        assert_eq!(
            parse_non_final_split("final"),
            Err(EvalError::ProtectedOrFinalSplit)
        );
        assert_eq!(
            parse_non_final_split("development").unwrap(),
            DataSplit::Development
        );
    }

    #[test]
    fn g6_attribution_contrast_is_secondary_only() {
        let registry = MetricRegistry::pinned();
        let seed_block = 5;
        let g6 = matches(
            DataSplit::Development,
            TaskFamily::Neutral,
            seed_block,
            &[(1, true), (2, false), (3, true), (4, false)],
        );
        let summary = summarize_g6_attribution_contrast_by_seed_block(
            DataSplit::Development,
            seed_block,
            &g6,
            &registry,
        )
        .unwrap();
        assert_eq!(summary.family, TaskFamily::Neutral);
        assert_eq!(summary.n_cases, 4);
        assert_eq!(
            summary.secondary_g6_attribution_contrast,
            SecondaryDiagnosticId::G6AttributionContrast
        );
        assert!((summary.g6_accuracy - 0.5).abs() < 1e-12);
        assert_eq!(
            summary.g6_accuracy_ci.method,
            UncertaintyMethod::WilsonScore
        );
        assert!(summary.experimental_non_final);

        // Non-Neutral family bits are rejected — G6 control is Neutral-only.
        let wrong_family = matches(
            DataSplit::Development,
            TaskFamily::Mixed,
            seed_block,
            &[(1, true), (2, false)],
        );
        assert_eq!(
            summarize_g6_attribution_contrast_by_seed_block(
                DataSplit::Development,
                seed_block,
                &wrong_family,
                &registry,
            ),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "family_mismatch",
            })
        );
    }

    #[test]
    fn paired_uncertainty_from_records_rejects_unmatched_targets_and_provenance() {
        let registry = MetricRegistry::pinned();
        let family = TaskFamily::Mixed;
        let seed_block = 11_u64;
        let t6_records = [
            T6EvalRecord {
                split: DataSplit::Development,
                family,
                case_id: 1,
                seed_block,
                outcome: T6Outcome {
                    score: 1.0,
                    matches_oracle: true,
                },
                canonical_digest: "pair-a".into(),
                envelope_contract: EVALUATOR_ENVELOPE_CONTRACT,
                arm_contract: T6_EVALUATOR_CONTRACT,
                budget_contract: READOUT_BUDGET_CONTRACT,
                metric_registry_contract: METRIC_REGISTRY_CONTRACT,
                source_torsor_contract: TORSOR_CONTRACT,
                label_contract: PROTECTED_LABEL_CONTRACT,
            },
            T6EvalRecord {
                split: DataSplit::Development,
                family,
                case_id: 2,
                seed_block,
                outcome: T6Outcome {
                    score: -1.0,
                    matches_oracle: false,
                },
                canonical_digest: "pair-b".into(),
                envelope_contract: EVALUATOR_ENVELOPE_CONTRACT,
                arm_contract: T6_EVALUATOR_CONTRACT,
                budget_contract: READOUT_BUDGET_CONTRACT,
                metric_registry_contract: METRIC_REGISTRY_CONTRACT,
                source_torsor_contract: TORSOR_CONTRACT,
                label_contract: PROTECTED_LABEL_CONTRACT,
            },
        ];
        let c6_records = [
            C6EvalRecord {
                split: DataSplit::Development,
                family,
                case_id: 1,
                seed_block,
                outcome: C6Outcome {
                    score: 1.0,
                    matches_oracle: true,
                },
                canonical_digest: "pair-a".into(),
                envelope_contract: EVALUATOR_ENVELOPE_CONTRACT,
                arm_contract: C6_EVALUATOR_CONTRACT,
                budget_contract: READOUT_BUDGET_CONTRACT,
                metric_registry_contract: METRIC_REGISTRY_CONTRACT,
                source_chiral_contract: CHIRAL_CONTRACT,
                label_contract: PROTECTED_LABEL_CONTRACT,
            },
            C6EvalRecord {
                split: DataSplit::Development,
                family,
                case_id: 2,
                seed_block,
                outcome: C6Outcome {
                    score: 1.0,
                    matches_oracle: true,
                },
                canonical_digest: "pair-b".into(),
                envelope_contract: EVALUATOR_ENVELOPE_CONTRACT,
                arm_contract: C6_EVALUATOR_CONTRACT,
                budget_contract: READOUT_BUDGET_CONTRACT,
                metric_registry_contract: METRIC_REGISTRY_CONTRACT,
                source_chiral_contract: CHIRAL_CONTRACT,
                label_contract: PROTECTED_LABEL_CONTRACT,
            },
        ];
        assert_eq!(
            summarize_paired_uncertainty_from_records(
                DataSplit::Development,
                family,
                seed_block,
                &t6_records,
                &c6_records,
                &registry,
            ),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "missing_common_target_contract",
            })
        );
        let t6_revealed =
            revealed_matches_from_t6_records(&t6_records, DataSplit::Development).unwrap();
        let c6_revealed =
            revealed_matches_from_c6_records(&c6_records, DataSplit::Development).unwrap();
        assert_eq!(t6_revealed[0].seed_block, seed_block);
        assert_eq!(t6_revealed[0].split, DataSplit::Development);
        assert_eq!(
            summarize_paired_uncertainty_by_seed_block(
                DataSplit::Development,
                family,
                seed_block,
                &t6_revealed,
                &c6_revealed,
                &registry,
            ),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "missing_common_target_contract",
            })
        );
        assert_eq!(
            summarize_paired_uncertainty_by_seed_block(
                DataSplit::Development,
                family,
                seed_block,
                &c6_revealed,
                &t6_revealed,
                &registry,
            ),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "arm_mismatch",
            })
        );

        let assert_record_provenance_rejected = |tampered: &[RevealedMatchOutcome]| {
            assert_eq!(
                summarize_paired_uncertainty_by_seed_block(
                    DataSplit::Development,
                    family,
                    seed_block,
                    tampered,
                    &c6_revealed,
                    &registry,
                ),
                Err(EvalError::PairedUncertaintyInvalid {
                    reason: "outcome_integrity_mismatch",
                })
            );
        };
        let mut tampered = t6_revealed.clone();
        tampered[0].family = TaskFamily::TorsorFavorable;
        assert_record_provenance_rejected(&tampered);
        let mut tampered = t6_revealed.clone();
        tampered[0].split = DataSplit::Validation;
        assert_record_provenance_rejected(&tampered);
        let mut tampered = t6_revealed.clone();
        tampered[0].seed_block += 1;
        assert_record_provenance_rejected(&tampered);
        let mut tampered = t6_revealed.clone();
        tampered[0].case_id += 1;
        assert_record_provenance_rejected(&tampered);
        let mut tampered = t6_revealed.clone();
        tampered[0].matches_oracle = !tampered[0].matches_oracle;
        assert_record_provenance_rejected(&tampered);

        // A single real block cannot be cloned and relabeled into the four
        // required strata after revelation.
        let mut relabeled_t6 = Vec::new();
        let mut relabeled_c6 = Vec::new();
        for relabeled_family in REQUIRED_SYNTHESIS_FAMILIES {
            let mut t6_block = t6_revealed.clone();
            let mut c6_block = c6_revealed.clone();
            for outcome in &mut t6_block {
                outcome.family = *relabeled_family;
            }
            for outcome in &mut c6_block {
                outcome.family = *relabeled_family;
            }
            relabeled_t6.extend(t6_block);
            relabeled_c6.extend(c6_block);
        }
        assert_eq!(
            synthesize_family_stratified_from_revealed_outcomes(
                DataSplit::Development,
                &relabeled_t6,
                &relabeled_c6,
                &registry,
            ),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "outcome_integrity_mismatch",
            })
        );

        // Typed surface admits only RevealedMatchOutcome — no ProtectedLabel ctor.
        let _revealed = RevealedMatchOutcome::from_matches_oracle(
            DataSplit::Development,
            family,
            seed_block,
            0,
            true,
        );
        assert!(_revealed.matches_oracle);

        let mut mismatched = c6_records.clone();
        mismatched[0].case_id = 99;
        assert_eq!(
            summarize_paired_uncertainty_from_records(
                DataSplit::Development,
                family,
                seed_block,
                &t6_records,
                &mismatched,
                &registry,
            ),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "pair_identity_mismatch",
            })
        );

        let mut digest_mismatched = c6_records.clone();
        digest_mismatched[0].canonical_digest = "different-canonical-case".into();
        let digest_mismatched_revealed =
            revealed_matches_from_c6_records(&digest_mismatched, DataSplit::Development).unwrap();
        assert_eq!(
            synthesize_family_stratified_from_revealed_outcomes(
                DataSplit::Development,
                &t6_revealed,
                &digest_mismatched_revealed,
                &registry,
            ),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "pair_identity_mismatch",
            })
        );

        let mut duplicate_t6 = t6_records.clone();
        duplicate_t6[1].case_id = duplicate_t6[0].case_id;
        duplicate_t6[1].canonical_digest = duplicate_t6[0].canonical_digest.clone();
        let mut duplicate_c6 = c6_records.clone();
        duplicate_c6[1].case_id = duplicate_c6[0].case_id;
        duplicate_c6[1].canonical_digest = duplicate_c6[0].canonical_digest.clone();
        assert_eq!(
            summarize_paired_uncertainty_from_records(
                DataSplit::Development,
                family,
                seed_block,
                &duplicate_t6,
                &duplicate_c6,
                &registry,
            ),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "duplicate_pair_identity",
            })
        );

        let mut stale_arm = t6_records.clone();
        stale_arm[0].arm_contract = "stale-t6-evaluator";
        assert_eq!(
            revealed_matches_from_t6_records(&stale_arm, DataSplit::Development),
            Err(EvalError::ContractMismatch("arm_contract"))
        );

        let mut wrong_seed = t6_records.clone();
        wrong_seed[1].seed_block = seed_block + 1;
        assert_eq!(
            summarize_paired_uncertainty_from_records(
                DataSplit::Development,
                family,
                seed_block,
                &wrong_seed,
                &c6_records,
                &registry,
            ),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "seed_block_mismatch",
            })
        );

        // Split retained on revealed outcomes cannot be relabeled.
        let mut validation_bits = matches(
            DataSplit::Validation,
            family,
            seed_block,
            &[(1, true), (2, false)],
        );
        let c6_dev = matches(
            DataSplit::Development,
            family,
            seed_block,
            &[(1, true), (2, true)],
        );
        assert_eq!(
            summarize_paired_uncertainty_by_seed_block(
                DataSplit::Development,
                family,
                seed_block,
                &validation_bits,
                &c6_dev,
                &registry,
            ),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "split_mismatch",
            })
        );
        let _ = &mut validation_bits;
    }

    #[test]
    fn paired_targets_are_integrity_bound_and_cannot_drift() {
        let family = TaskFamily::Mixed;
        let left = matches(DataSplit::Development, family, 11, &[(1, true), (2, false)]);
        let mut right = matches(DataSplit::Development, family, 11, &[(1, true), (2, true)]);
        right[0].shared_target_contract = Some("different-test-target");
        let classify = |right: &[RevealedMatchOutcome]| {
            summarize_paired_uncertainty_by_seed_block(
                DataSplit::Development,
                family,
                11,
                &left,
                right,
                &MetricRegistry::pinned(),
            )
        };
        assert_eq!(
            classify(&right),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "outcome_integrity_mismatch",
            })
        );
        right[0].integrity.shared_target_contract = right[0].shared_target_contract;
        assert_eq!(
            classify(&right),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "common_target_contract_mismatch",
            })
        );
    }

    fn family_summary(
        split: DataSplit,
        family: TaskFamily,
        seed_block: u64,
        t6_bits: &[(u64, bool)],
        c6_bits: &[(u64, bool)],
    ) -> PairedEffectSummary {
        let t6 = matches(split, family, seed_block, t6_bits);
        let c6 = matches(split, family, seed_block, c6_bits);
        summarize_paired_uncertainty_by_seed_block(
            split,
            family,
            seed_block,
            &t6,
            &c6,
            &MetricRegistry::pinned(),
        )
        .unwrap()
    }

    fn all_c6_win_bits(n: u64) -> Vec<(u64, bool)> {
        (0..n).map(|case_id| (case_id, true)).collect()
    }

    fn all_t6_win_bits(n: u64) -> Vec<(u64, bool)> {
        (0..n).map(|case_id| (case_id, true)).collect()
    }

    #[test]
    fn family_stratified_synthesis_accepts_concordant_families() {
        assert_eq!(
            FAMILY_STRATIFIED_SYNTHESIS_CONTRACT,
            "tdi25-family-stratified-synthesis-v1"
        );
        assert_eq!(SignedEffectClass::Positive.as_str(), "positive");
        assert_eq!(SignedEffectClass::Null.as_str(), "null");
        assert_eq!(SignedEffectClass::Harmful.as_str(), "harmful");
        assert_eq!(EffectSign::Negative.as_str(), "negative");
        assert_eq!(REQUIRED_SYNTHESIS_FAMILIES.len(), 4);

        let split = DataSplit::Development;
        let n = MAX_CASES_PER_RUN;
        let t6_false: Vec<(u64, bool)> = (0..n).map(|case_id| (case_id, false)).collect();
        let c6_true = all_c6_win_bits(n);
        let summaries = [
            family_summary(split, TaskFamily::TorsorFavorable, 1, &t6_false, &c6_true),
            family_summary(split, TaskFamily::ChiralFavorable, 1, &t6_false, &c6_true),
            family_summary(split, TaskFamily::Mixed, 1, &t6_false, &c6_true),
            family_summary(split, TaskFamily::Neutral, 1, &t6_false, &c6_true),
        ];
        let report =
            synthesize_family_stratified_effects(split, &summaries, &MetricRegistry::pinned())
                .unwrap();

        assert!(!report.sign_reversal_present);
        assert!(!report.seed_block_sign_reversal_present);
        assert!(report.claims_clean_pooled_win);
        assert!(report.experimental_non_final);
        assert_eq!(
            report.synthesis_contract,
            FAMILY_STRATIFIED_SYNTHESIS_CONTRACT
        );
        assert_eq!(report.uncertainty_contract, PAIRED_UNCERTAINTY_CONTRACT);
        assert_eq!(report.metric_registry_contract, METRIC_REGISTRY_CONTRACT);
        assert_eq!(
            report.cross_family_summary,
            CrossFamilySummaryMetricId::CrossFamilyPairedSummary
        );
        assert_eq!(report.family_effects.len(), 4);
        assert_eq!(report.seed_block_effects.len(), 4);
        for effect in &report.family_effects {
            assert_eq!(effect.effect_sign, EffectSign::Positive);
            assert_eq!(effect.outcome_class, SignedEffectClass::Positive);
        }
        let pooled = report
            .pooled_summary
            .expect("pooled admitted without reversal");
        assert_eq!(pooled.effect_sign, EffectSign::Positive);
        assert_eq!(pooled.outcome_class, SignedEffectClass::Positive);
        assert_eq!(pooled.n_pairs, n * 4);
        assert!((pooled.paired_difference_mean - 1.0).abs() < 1e-12);
    }

    #[test]
    fn family_stratified_synthesis_aggregates_multiple_seed_blocks() {
        let split = DataSplit::Development;
        let mut t6 = Vec::new();
        let mut c6 = Vec::new();
        for family in REQUIRED_SYNTHESIS_FAMILIES {
            for seed_block in [10, 11] {
                let t6_bits: Vec<(u64, bool)> = (0..MAX_CASES_PER_RUN)
                    .map(|case_id| (case_id, false))
                    .collect();
                let c6_bits: Vec<(u64, bool)> = (0..MAX_CASES_PER_RUN)
                    .map(|case_id| (case_id, true))
                    .collect();
                t6.extend(matches(split, *family, seed_block, &t6_bits));
                c6.extend(matches(split, *family, seed_block, &c6_bits));
            }
        }

        let report = synthesize_family_stratified_from_revealed_outcomes(
            split,
            &t6,
            &c6,
            &MetricRegistry::pinned(),
        )
        .unwrap();
        assert_eq!(report.family_effects.len(), 4);
        assert_eq!(report.seed_block_effects.len(), 8);
        assert!(!report.seed_block_sign_reversal_present);
        for effect in &report.family_effects {
            assert_eq!(effect.seed_block, 10);
            assert_eq!(effect.seed_blocks, vec![10, 11]);
            assert_eq!(effect.n_pairs, 2 * MAX_CASES_PER_RUN);
            assert_eq!(
                effect.paired_difference_ci.method,
                UncertaintyMethod::ClusterHoeffdingPairedDifference
            );
        }
        for family in REQUIRED_SYNTHESIS_FAMILIES {
            let retained_blocks: Vec<u64> = report
                .seed_block_effects
                .iter()
                .filter(|effect| effect.family == *family)
                .map(|effect| effect.seed_block)
                .collect();
            assert_eq!(retained_blocks, vec![10, 11]);
        }
    }

    #[test]
    fn family_stratified_synthesis_retains_opposed_seed_block_effects() {
        let split = DataSplit::Development;
        let mut t6 = Vec::new();
        let mut c6 = Vec::new();
        for family in REQUIRED_SYNTHESIS_FAMILIES {
            let t6_losses: Vec<(u64, bool)> = (0..MAX_CASES_PER_RUN)
                .map(|case_id| (case_id, false))
                .collect();
            let c6_wins: Vec<(u64, bool)> = (0..MAX_CASES_PER_RUN)
                .map(|case_id| (case_id, true))
                .collect();
            let t6_wins = c6_wins.clone();
            let c6_losses = t6_losses.clone();
            t6.extend(matches(split, *family, 20, &t6_losses));
            c6.extend(matches(split, *family, 20, &c6_wins));
            t6.extend(matches(split, *family, 21, &t6_wins));
            c6.extend(matches(split, *family, 21, &c6_losses));
        }

        let report = synthesize_family_stratified_from_revealed_outcomes(
            split,
            &t6,
            &c6,
            &MetricRegistry::pinned(),
        )
        .unwrap();
        assert_eq!(report.family_effects.len(), 4);
        assert_eq!(report.seed_block_effects.len(), 8);
        assert!(report.seed_block_sign_reversal_present);
        assert!(report.pooled_summary.is_none());
        assert!(!report.claims_clean_pooled_win);
        for family in REQUIRED_SYNTHESIS_FAMILIES {
            let retained: Vec<&FamilySignedEffect> = report
                .seed_block_effects
                .iter()
                .filter(|effect| effect.family == *family)
                .collect();
            assert_eq!(retained.len(), 2);
            assert_eq!(retained[0].seed_block, 20);
            assert_eq!(retained[0].outcome_class, SignedEffectClass::Positive);
            assert_eq!(retained[1].seed_block, 21);
            assert_eq!(retained[1].outcome_class, SignedEffectClass::Harmful);
        }
    }

    #[test]
    fn family_stratified_synthesis_detects_torsor_positive_chiral_negative_reversal() {
        let split = DataSplit::Validation;
        let n = MAX_CASES_PER_RUN;
        let t6_false: Vec<(u64, bool)> = (0..n).map(|case_id| (case_id, false)).collect();
        let c6_true = all_c6_win_bits(n);
        let t6_true = all_t6_win_bits(n);
        let c6_false: Vec<(u64, bool)> = (0..n).map(|case_id| (case_id, false)).collect();

        let ties_t6: Vec<(u64, bool)> = (0..n).map(|case_id| (case_id, true)).collect();
        let ties_c6 = ties_t6.clone();
        let summaries = [
            family_summary(split, TaskFamily::TorsorFavorable, 2, &t6_false, &c6_true),
            family_summary(split, TaskFamily::ChiralFavorable, 2, &t6_true, &c6_false),
            family_summary(split, TaskFamily::Mixed, 2, &ties_t6, &ties_c6),
            family_summary(split, TaskFamily::Neutral, 2, &ties_t6, &ties_c6),
        ];
        assert_eq!(summaries[0].family, TaskFamily::TorsorFavorable);
        assert!(summaries[0].paired_difference_mean > 0.0);
        assert_eq!(summaries[1].family, TaskFamily::ChiralFavorable);
        assert!(summaries[1].paired_difference_mean < 0.0);

        let report =
            synthesize_family_stratified_effects(split, &summaries, &MetricRegistry::pinned())
                .unwrap();

        assert!(report.sign_reversal_present);
        assert!(report.pooled_summary.is_none());
        assert!(!report.claims_clean_pooled_win);
        assert_eq!(
            report.family_effects[0].outcome_class,
            SignedEffectClass::Positive
        );
        assert_eq!(
            report.family_effects[1].outcome_class,
            SignedEffectClass::Harmful
        );
        assert_eq!(report.family_effects[0].effect_sign, EffectSign::Positive);
        assert_eq!(report.family_effects[1].effect_sign, EffectSign::Negative);
        assert_eq!(report.family_effects[2].effect_sign, EffectSign::Zero);
        assert_eq!(report.family_effects[3].effect_sign, EffectSign::Zero);
        assert_eq!(
            report.family_effects[2].outcome_class,
            SignedEffectClass::Inconclusive
        );
    }

    #[test]
    fn family_stratified_synthesis_does_not_claim_win_from_inconclusive_family_intervals() {
        let split = DataSplit::Development;
        let t6_false = [(0, false), (1, false)];
        let c6_mixed = [(0, true), (1, false)];
        let summaries = [
            family_summary(split, TaskFamily::TorsorFavorable, 8, &t6_false, &c6_mixed),
            family_summary(split, TaskFamily::ChiralFavorable, 8, &t6_false, &c6_mixed),
            family_summary(split, TaskFamily::Mixed, 8, &t6_false, &c6_mixed),
            family_summary(split, TaskFamily::Neutral, 8, &t6_false, &c6_mixed),
        ];
        let report =
            synthesize_family_stratified_effects(split, &summaries, &MetricRegistry::pinned())
                .unwrap();

        assert!(
            report
                .family_effects
                .iter()
                .all(|effect| effect.paired_difference_mean > 0.0)
        );
        assert!(
            report
                .family_effects
                .iter()
                .all(|effect| effect.outcome_class == SignedEffectClass::Inconclusive)
        );
        let pooled = report.pooled_summary.expect("no sign reversal");
        assert_eq!(pooled.outcome_class, SignedEffectClass::Inconclusive);
        assert!(!report.claims_clean_pooled_win);
    }

    #[test]
    fn family_stratified_synthesis_rejects_incomplete_required_families() {
        let split = DataSplit::Development;
        let n = MAX_CASES_PER_RUN;
        let t6_false: Vec<(u64, bool)> = (0..n).map(|case_id| (case_id, false)).collect();
        let c6_true = all_c6_win_bits(n);
        let incomplete = [
            family_summary(split, TaskFamily::TorsorFavorable, 3, &t6_false, &c6_true),
            family_summary(split, TaskFamily::ChiralFavorable, 3, &t6_false, &c6_true),
            family_summary(split, TaskFamily::Mixed, 3, &t6_false, &c6_true),
        ];
        assert_eq!(
            synthesize_family_stratified_effects(split, &incomplete, &MetricRegistry::pinned(),),
            Err(EvalError::FamilyStratifiedSynthesisInvalid {
                reason: "missing_family",
            })
        );

        let duplicate = [
            family_summary(split, TaskFamily::TorsorFavorable, 3, &t6_false, &c6_true),
            family_summary(split, TaskFamily::TorsorFavorable, 4, &t6_false, &c6_true),
        ];
        assert_eq!(
            synthesize_family_stratified_effects(split, &duplicate, &MetricRegistry::pinned(),),
            Err(EvalError::FamilyStratifiedSynthesisInvalid {
                reason: "duplicate_family",
            })
        );

        assert_eq!(
            synthesize_family_stratified_effects(split, &[], &MetricRegistry::pinned()),
            Err(EvalError::FamilyStratifiedSynthesisInvalid {
                reason: "empty_families",
            })
        );
    }

    #[test]
    fn family_stratified_synthesis_rejects_registry_drift_and_protected() {
        let split = DataSplit::Development;
        let n = MAX_CASES_PER_RUN;
        let t6_false: Vec<(u64, bool)> = (0..n).map(|case_id| (case_id, false)).collect();
        let c6_true = all_c6_win_bits(n);
        let summaries = [
            family_summary(split, TaskFamily::TorsorFavorable, 5, &t6_false, &c6_true),
            family_summary(split, TaskFamily::ChiralFavorable, 5, &t6_false, &c6_true),
            family_summary(split, TaskFamily::Mixed, 5, &t6_false, &c6_true),
            family_summary(split, TaskFamily::Neutral, 5, &t6_false, &c6_true),
        ];

        let mut drifted = MetricRegistry::pinned();
        drifted.registry_contract = "tdi25-metric-registry-drift";
        assert_eq!(
            synthesize_family_stratified_effects(split, &summaries, &drifted),
            Err(EvalError::ContractMismatch("metric_registry_contract"))
        );

        assert_eq!(
            parse_non_final_split("protected"),
            Err(EvalError::ProtectedOrFinalSplit)
        );
        assert_eq!(
            parse_non_final_split("final"),
            Err(EvalError::ProtectedOrFinalSplit)
        );

        let mut t6_all = Vec::new();
        let mut c6_all = Vec::new();
        let torsor_t6: Vec<(u64, bool)> = (0..n).map(|case_id| (case_id, false)).collect();
        let torsor_c6: Vec<(u64, bool)> = (0..n).map(|case_id| (case_id, true)).collect();
        let chiral_t6: Vec<(u64, bool)> = (0..n).map(|case_id| (case_id, true)).collect();
        let chiral_c6: Vec<(u64, bool)> = (0..n).map(|case_id| (case_id, false)).collect();
        let tie: Vec<(u64, bool)> = (0..n).map(|case_id| (case_id, true)).collect();
        for (family, t6_bits, c6_bits) in [
            (TaskFamily::TorsorFavorable, &torsor_t6, &torsor_c6),
            (TaskFamily::ChiralFavorable, &chiral_t6, &chiral_c6),
            (TaskFamily::Mixed, &tie, &tie),
            (TaskFamily::Neutral, &tie, &tie),
        ] {
            t6_all.extend(matches(split, family, 7, t6_bits));
            c6_all.extend(matches(split, family, 7, c6_bits));
        }
        let from_outcomes = synthesize_family_stratified_from_revealed_outcomes(
            split,
            &t6_all,
            &c6_all,
            &MetricRegistry::pinned(),
        )
        .unwrap();
        assert!(from_outcomes.sign_reversal_present);
        assert!(from_outcomes.pooled_summary.is_none());
        assert!(!from_outcomes.claims_clean_pooled_win);
    }

    #[test]
    fn family_stratified_synthesis_rejects_tampered_positive_intervals() {
        let split = DataSplit::Development;
        let n = MAX_CASES_PER_RUN;
        let losses: Vec<(u64, bool)> = (0..n).map(|case_id| (case_id, false)).collect();
        let mut one_c6_win = losses.clone();
        one_c6_win[0].1 = true;
        let mut summaries = [
            family_summary(split, TaskFamily::TorsorFavorable, 9, &losses, &one_c6_win),
            family_summary(split, TaskFamily::ChiralFavorable, 9, &losses, &one_c6_win),
            family_summary(split, TaskFamily::Mixed, 9, &losses, &one_c6_win),
            family_summary(split, TaskFamily::Neutral, 9, &losses, &one_c6_win),
        ];
        // The forged interval contains the mean but is not the engine-produced interval.
        for summary in &mut summaries {
            assert!(approximately_equal(
                summary.paired_difference_mean,
                1.0 / n as f64
            ));
            summary.paired_difference_ci.lower = 0.001;
            summary.paired_difference_ci.upper = 0.02;
        }
        assert_eq!(
            synthesize_family_stratified_effects(split, &summaries, &MetricRegistry::pinned()),
            Err(EvalError::FamilyStratifiedSynthesisInvalid {
                reason: "summary_integrity_mismatch",
            })
        );
    }

    #[test]
    fn signed_effect_classifier_rejects_mean_outside_interval() {
        let contradictory = ConfidenceInterval {
            lower: 0.1,
            upper: 0.2,
            confidence: PAIRED_UNCERTAINTY_LEVEL,
            method: UncertaintyMethod::BoundedHoeffdingPairedDifference,
        };
        assert_eq!(
            classify_signed_effect(-0.5, contradictory),
            Err(EvalError::FamilyStratifiedSynthesisInvalid {
                reason: "mean_outside_interval",
            })
        );
    }

    #[test]
    fn signed_effect_classifier_rejects_values_outside_paired_support() {
        let too_large = ConfidenceInterval {
            lower: 1.5,
            upper: 2.5,
            confidence: PAIRED_UNCERTAINTY_LEVEL,
            method: UncertaintyMethod::BoundedHoeffdingPairedDifference,
        };
        assert_eq!(
            classify_signed_effect(2.0, too_large),
            Err(EvalError::FamilyStratifiedSynthesisInvalid {
                reason: "paired_difference_out_of_bounds",
            })
        );

        let too_small = ConfidenceInterval {
            lower: -2.5,
            upper: -1.5,
            confidence: PAIRED_UNCERTAINTY_LEVEL,
            method: UncertaintyMethod::ClusterHoeffdingPairedDifference,
        };
        assert_eq!(
            classify_signed_effect(-2.0, too_small),
            Err(EvalError::FamilyStratifiedSynthesisInvalid {
                reason: "paired_difference_out_of_bounds",
            })
        );
    }

    #[test]
    fn effect_sign_rejects_non_finite_means() {
        assert_eq!(effect_sign_from_mean(1.0), Ok(EffectSign::Positive));
        assert_eq!(effect_sign_from_mean(-1.0), Ok(EffectSign::Negative));
        assert_eq!(effect_sign_from_mean(0.0), Ok(EffectSign::Zero));
        for non_finite in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                effect_sign_from_mean(non_finite),
                Err(EvalError::FamilyStratifiedSynthesisInvalid {
                    reason: "non_finite",
                })
            );
        }
    }

    #[test]
    fn signed_effect_classifier_rejects_confidence_and_method_drift() {
        let wrong_confidence = ConfidenceInterval {
            lower: 0.1,
            upper: 0.2,
            confidence: 0.0,
            method: UncertaintyMethod::BoundedHoeffdingPairedDifference,
        };
        assert_eq!(
            classify_signed_effect(0.15, wrong_confidence),
            Err(EvalError::FamilyStratifiedSynthesisInvalid {
                reason: "confidence_level_mismatch",
            })
        );

        let non_finite_confidence = ConfidenceInterval {
            confidence: f64::NAN,
            ..wrong_confidence
        };
        assert_eq!(
            classify_signed_effect(0.15, non_finite_confidence),
            Err(EvalError::FamilyStratifiedSynthesisInvalid {
                reason: "non_finite",
            })
        );

        for non_finite_mean in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(
                classify_signed_effect(non_finite_mean, wrong_confidence),
                Err(EvalError::FamilyStratifiedSynthesisInvalid {
                    reason: "non_finite",
                })
            );
        }

        let wrong_method = ConfidenceInterval {
            confidence: PAIRED_UNCERTAINTY_LEVEL,
            method: UncertaintyMethod::WilsonScore,
            ..wrong_confidence
        };
        assert_eq!(
            classify_signed_effect(0.15, wrong_method),
            Err(EvalError::FamilyStratifiedSynthesisInvalid {
                reason: "uncertainty_method_mismatch",
            })
        );
    }

    #[test]
    fn failure_resource_taxonomy_is_closed_and_protected_stays_hard() {
        assert_eq!(
            FAILURE_RESOURCE_ACCOUNTING_CONTRACT,
            "tdi25-failure-resource-accounting-v1"
        );
        assert_eq!(MAX_FAILURES_PER_ARM, MAX_CASES_PER_RUN);
        for class in [
            FailureClass::Invalid,
            FailureClass::Numerical,
            FailureClass::Resource,
            FailureClass::Task,
        ] {
            assert_eq!(parse_failure_class(class.as_str()).unwrap(), class);
        }
        assert_eq!(
            parse_failure_class("timeout"),
            Err(EvalError::FailureResourceAccountingInvalid {
                reason: "unknown_class",
            })
        );
        assert_eq!(
            parse_failure_class(""),
            Err(EvalError::FailureResourceAccountingInvalid {
                reason: "empty_class",
            })
        );
        let cases = [
            (
                EvalError::Bridge(Tdi25Error::NonFiniteGeneric),
                FailureClass::Numerical,
            ),
            (
                EvalError::Bridge(Tdi25Error::NonFiniteScalar { field: "score" }),
                FailureClass::Numerical,
            ),
            (
                EvalError::Bridge(Tdi25Error::InvalidScoreScale),
                FailureClass::Numerical,
            ),
            (
                EvalError::Bridge(Tdi25Error::ChiralInvariantViolation { field: "chi" }),
                FailureClass::Invalid,
            ),
            (
                EvalError::PairedUncertaintyInvalid {
                    reason: "non_finite",
                },
                FailureClass::Numerical,
            ),
            (EvalError::CaseBudgetExceeded, FailureClass::Resource),
            (
                EvalError::SplitMismatch {
                    expected: DataSplit::Development,
                    actual: DataSplit::Validation,
                },
                FailureClass::Task,
            ),
            (
                EvalError::ContractMismatch("mixed_case"),
                FailureClass::Task,
            ),
            (
                EvalError::ContractMismatch("arm_contract"),
                FailureClass::Invalid,
            ),
            (EvalError::InvalidBudget, FailureClass::Invalid),
        ];
        for (error, class) in cases {
            assert_eq!(classify_eval_error(&error).unwrap(), class, "{error:?}");
            assert!(!eval_error_message_code(&error).is_empty());
        }
        assert_eq!(
            classify_eval_error(&EvalError::ProtectedOrFinalSplit),
            Err(EvalError::ProtectedOrFinalSplit)
        );
        let mut ledger = FailureResourceLedger::open(DataSplit::Development).unwrap();
        assert_eq!(
            ledger.retain_failure(
                ComparisonArm::T6,
                TaskFamily::Mixed,
                Some(0),
                &EvalError::ProtectedOrFinalSplit,
            ),
            Err(EvalError::ProtectedOrFinalSplit)
        );
        assert!(ledger.failures().is_empty());
        assert_eq!(
            parse_non_final_split("final"),
            Err(EvalError::ProtectedOrFinalSplit)
        );
    }

    #[test]
    fn failure_resource_ledger_retains_failures_by_arm_and_counts_resources() {
        let split = DataSplit::Development;
        let mut ledger = FailureResourceLedger::open(split).unwrap();
        let torsor = torsor_transport_pair_in_split(2, split).unwrap();
        let other = torsor_transport_pair_in_split(3, split).unwrap();
        let mut tight = EvaluatorConfig::t6(split);
        tight.budget.max_cases = 1;
        let mut t6 = T6EvaluatorRun::open(tight).unwrap();
        let first = seal_torsor_transport(torsor.original, torsor.oracle);
        assert!(
            ledger
                .account_t6_torsor_transport(&mut t6, &first)
                .unwrap()
                .is_some()
        );
        // Budget exhausted: retained as a T6 resource failure.
        let second = seal_torsor_transport(torsor.transported, torsor.oracle);
        assert!(
            ledger
                .account_t6_torsor_transport(&mut t6, &second)
                .unwrap()
                .is_none()
        );
        // Non-canonical oracle pairing: retained as a T6 task failure.
        let mut fresh = T6EvaluatorRun::open(EvaluatorConfig::t6(split)).unwrap();
        let mislabeled = seal_torsor_transport(torsor.original, other.oracle);
        assert!(
            ledger
                .account_t6_torsor_transport(&mut fresh, &mislabeled)
                .unwrap()
                .is_none()
        );

        let chiral = chiral_reflection_pair_in_split(2, split).unwrap();
        let mut c6 = C6EvaluatorRun::open(EvaluatorConfig::c6(split)).unwrap();
        for case in [
            seal_chiral_reflection(chiral.right, chiral.right_oracle),
            seal_chiral_reflection(chiral.left, chiral.left_oracle),
        ] {
            assert!(
                ledger
                    .account_c6_chiral_reflection(&mut c6, &case)
                    .unwrap()
                    .is_some()
            );
        }
        let neutral = neutral_control_pair_in_split(2, split).unwrap();
        let mut g6 = G6EvaluatorRun::open(EvaluatorConfig::g6(split)).unwrap();
        let validation = neutral_control_pair_in_split(2, DataSplit::Validation).unwrap();
        assert!(
            ledger
                .account_g6_neutral_control(
                    &mut g6,
                    &seal_neutral_control(validation.class_a, validation.class_a_oracle),
                )
                .unwrap()
                .is_none()
        );
        assert!(
            ledger
                .account_g6_neutral_control(
                    &mut g6,
                    &seal_neutral_control(neutral.class_a, neutral.class_a_oracle),
                )
                .unwrap()
                .is_some()
        );

        let t6_account = ledger.account(ComparisonArm::T6);
        assert_eq!(t6_account.scored_cases, 1);
        assert_eq!(t6_account.readout_scalars_used, 2);
        assert_eq!(t6_account.resource_failures, 1);
        assert_eq!(t6_account.task_failures, 1);
        assert_eq!(t6_account.attempted_cases(), 3);
        assert_eq!(t6_account.attempts_by_family, [3, 0, 0, 0]);
        let c6_account = ledger.account(ComparisonArm::C6);
        assert_eq!(c6_account.scored_cases, 2);
        assert_eq!(c6_account.retained_failures(), 0);
        assert_eq!(c6_account.attempts_by_family, [0, 2, 0, 0]);
        let g6_account = ledger.account(ComparisonArm::G6);
        assert_eq!(g6_account.task_failures, 1);
        assert_eq!(g6_account.scored_cases, 1);

        let report = ledger.report().unwrap();
        assert_eq!(report.failures.len(), 3);
        assert!(report.failures.iter().all(|record| record.split == split));
        assert!(
            report
                .failures
                .iter()
                .all(|record| record.accounting_contract == FAILURE_RESOURCE_ACCOUNTING_CONTRACT)
        );
        assert!(!report.primary_attempts_matched);
        assert!(!report.primary_failure_free);
        assert_eq!(
            require_complete_primary_accounting(&report),
            Err(EvalError::FailureResourceAccountingInvalid {
                reason: "unmatched_primary_family_attempts",
            })
        );

        // Tampering: dropping a retained failure or forging readout use fails closed.
        let mut dropped = report.clone();
        dropped.failures.pop();
        assert_eq!(
            validate_failure_resource_report(&dropped),
            Err(EvalError::FailureResourceAccountingInvalid {
                reason: "failure_not_retained",
            })
        );
        let mut forged = report.clone();
        forged.arms[0].readout_scalars_used = 0;
        assert_eq!(
            validate_failure_resource_report(&forged),
            Err(EvalError::FailureResourceAccountingInvalid {
                reason: "readout_accounting_mismatch",
            })
        );
        let mut reordered = report.clone();
        reordered.arms.swap(0, 1);
        assert_eq!(
            validate_failure_resource_report(&reordered),
            Err(EvalError::FailureResourceAccountingInvalid {
                reason: "arm_order_mismatch",
            })
        );
        let mut flagged = report.clone();
        flagged.primary_failure_free = true;
        assert_eq!(
            validate_failure_resource_report(&flagged),
            Err(EvalError::FailureResourceAccountingInvalid {
                reason: "primary_flag_mismatch",
            })
        );
        let mut drifted = report;
        drifted.failures[0].accounting_contract = "tdi25-failure-resource-accounting-v0";
        assert_eq!(
            validate_failure_resource_report(&drifted),
            Err(EvalError::FailureResourceAccountingInvalid {
                reason: "contract_drift",
            })
        );

        // A run opened on another split cannot be accounted in this ledger.
        let mut validation_run =
            T6EvaluatorRun::open(EvaluatorConfig::t6(DataSplit::Validation)).unwrap();
        assert_eq!(
            ledger.account_t6_torsor_transport(&mut validation_run, &first),
            Err(EvalError::FailureResourceAccountingInvalid {
                reason: "run_split_mismatch",
            })
        );
    }

    #[test]
    fn matched_primary_blocks_account_both_arms_and_retain_atomic_failures() {
        let mut ledger = FailureResourceLedger::open(DataSplit::Validation).unwrap();
        for family in REQUIRED_SYNTHESIS_FAMILIES {
            let run = ledger
                .account_matched_primary_block(*family, 0, 8)
                .unwrap()
                .unwrap();
            assert_eq!(run.t6_outcomes().len(), 8);
        }
        let report = ledger.report().unwrap();
        assert!(report.primary_attempts_matched);
        assert!(report.primary_failure_free);
        require_complete_primary_accounting(&report).unwrap();
        assert_eq!(report.arms[0].scored_cases, 32);
        assert_eq!(report.arms[1].scored_cases, 32);
        assert_eq!(report.arms[0].readout_scalars_used, 64);
        assert_eq!(report.arms[2].attempted_cases(), 0);

        // An inadmissible block is retained once on both primary arms.
        assert!(
            ledger
                .account_matched_primary_block(TaskFamily::Mixed, 0, 1)
                .unwrap()
                .is_none()
        );
        let report = ledger.report().unwrap();
        assert!(report.primary_attempts_matched);
        assert!(!report.primary_failure_free);
        assert_eq!(report.arms[0].invalid_failures, 1);
        assert_eq!(report.arms[1].invalid_failures, 1);
        assert!(
            report
                .failures
                .iter()
                .all(|record| record.case_id.is_none())
        );
        assert_eq!(
            require_complete_primary_accounting(&report),
            Err(EvalError::FailureResourceAccountingInvalid {
                reason: "primary_failures_retained",
            })
        );
    }
}
