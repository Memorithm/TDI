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
//! no RNG; no ProtectedLabel. All arms consume sealed
//! Development/Validation cases, keep task oracles outside inference
//! callbacks, and reject split or contract drift. They do not train, access
//! protected/final data, or authorize a scientific claim.

use core::fmt;

use super::tdi22_torsor::TORSOR_CONTRACT;
use super::tdi24_chiral::CHIRAL_CONTRACT;
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
    split: DataSplit,
    family: TaskFamily,
    seed_block: u64,
    case_id: u64,
    canonical_digest: String,
    matches_oracle: bool,
}

impl RevealedMatchOutcomeIntegrity {
    fn new(
        split: DataSplit,
        family: TaskFamily,
        seed_block: u64,
        case_id: u64,
        canonical_digest: &str,
        matches_oracle: bool,
    ) -> Self {
        Self {
            split,
            family,
            seed_block,
            case_id,
            canonical_digest: canonical_digest.to_owned(),
            matches_oracle,
        }
    }

    fn matches(&self, outcome: &RevealedMatchOutcome) -> bool {
        self.split == outcome.split
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
        let integrity = RevealedMatchOutcomeIntegrity::new(
            split,
            family,
            seed_block,
            case_id,
            &canonical_digest,
            matches_oracle,
        );
        Self {
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
        split: DataSplit,
        family: TaskFamily,
        seed_block: u64,
        case_id: u64,
        canonical_digest: String,
        matches_oracle: bool,
    ) -> Self {
        let integrity = RevealedMatchOutcomeIntegrity::new(
            split,
            family,
            seed_block,
            case_id,
            &canonical_digest,
            matches_oracle,
        );
        Self {
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
    require_pair_identity_alignment(t6_matches, c6_matches)?;

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
/// Hoeffding margin. Phase-C matched T6/C6 record pairs are currently emitted
/// for [`TaskFamily::Mixed`]; other families remain available on the revealed-bit
/// API for later matched evaluator paths.
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
/// - [`SignedEffectClass::Null`] when the interval contains zero
/// - [`SignedEffectClass::Inconclusive`] reserved for non-classifiable finite cases
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SignedEffectClass {
    /// Paired difference favors C6 (CI entirely above zero).
    Positive,
    /// No signed claim (CI contains zero).
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

/// Classify a paired-difference CI into a closed Stage-C outcome class.
pub fn classify_signed_effect(
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
    if ci.lower > 0.0 {
        Ok(SignedEffectClass::Positive)
    } else if ci.upper < 0.0 {
        Ok(SignedEffectClass::Harmful)
    } else {
        // Interval contains zero: null rather than a signed claim.
        Ok(SignedEffectClass::Null)
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
        SignedEffectClass::Null
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
    fn paired_uncertainty_from_records_accepts_scored_rejects_failures() {
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
        let summary = summarize_paired_uncertainty_from_records(
            DataSplit::Development,
            family,
            seed_block,
            &t6_records,
            &c6_records,
            &registry,
        )
        .unwrap();
        assert_eq!(summary.n_pairs, 2);
        assert!((summary.t6_accuracy - 0.5).abs() < 1e-12);
        assert!((summary.c6_accuracy - 1.0).abs() < 1e-12);
        assert!((summary.paired_difference_mean - 0.5).abs() < 1e-12);
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
            )
            .unwrap()
            .n_pairs,
            2
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

        // Single seed-block summaries use informative within-block Wilson/Hoeffding.
        assert_eq!(
            summary.t6_accuracy_ci.method,
            UncertaintyMethod::WilsonScore
        );
        assert_eq!(
            summary.c6_accuracy_ci.method,
            UncertaintyMethod::WilsonScore
        );
        assert_eq!(
            summary.paired_difference_ci.method,
            UncertaintyMethod::BoundedHoeffdingPairedDifference
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
            SignedEffectClass::Null
        );
    }

    #[test]
    fn family_stratified_synthesis_does_not_claim_win_from_null_family_intervals() {
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
                .all(|effect| effect.outcome_class == SignedEffectClass::Null)
        );
        let pooled = report.pooled_summary.expect("no sign reversal");
        assert_eq!(pooled.outcome_class, SignedEffectClass::Null);
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
}
