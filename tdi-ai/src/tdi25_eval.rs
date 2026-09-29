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
//! metric registry: no paired primary is admitted yet because no task family
//! exposes a common task-level oracle across T6 and C6. The known primary ids
//! remain candidates only, and the cross-family summary remains unavailable.
//! Likewise, G6 attribution is not admitted until one common family has matched
//! T6/C6/G6 paths. The ordered closed registry fails closed on drift, premature
//! pairing, duplicates, and invention.
//! Evaluator open paths require the pinned registry. All arms consume
//! sealed Development/Validation cases, keep task oracles outside inference
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

/// Primary family metric: torsor-favorable paired accuracy / score-match vs oracle.
pub const PRIMARY_FAMILY_TORSOR_FAVORABLE: &str = "family_paired_task_accuracy_torsor_favorable";

/// Primary family metric: chiral-favorable paired accuracy / score-match vs oracle.
pub const PRIMARY_FAMILY_CHIRAL_FAVORABLE: &str = "family_paired_task_accuracy_chiral_favorable";

/// Primary family metric: mixed-geometry paired accuracy / score-match vs oracle.
pub const PRIMARY_FAMILY_MIXED: &str = "family_paired_task_accuracy_mixed";

/// Primary family metric: neutral-control paired accuracy / score-match vs oracle.
pub const PRIMARY_FAMILY_NEUTRAL: &str = "family_paired_task_accuracy_neutral";

/// Reserved future cross-family summary metric id.
///
/// This id is parsed for forward contract diagnostics but is not admitted by
/// the v1 pinned registry until multiple families have matched T6/C6 paths.
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

/// Bounded capacity for future computable paired primary family metrics.
pub const MAX_PRIMARY_FAMILY_METRICS: usize = 4;

/// Maximum secondary diagnostics admitted in one frozen registry.
pub const MAX_SECONDARY_DIAGNOSTICS: usize = 6;

/// Shared ordering identity for non-trained Phase-C optimizer budgets.
pub const NON_TRAINED_ORDERING_ID: u64 = 0;

/// Maximum cases admitted to one non-final evaluator run.
pub const MAX_CASES_PER_RUN: u64 = 64;

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
/// One id per task family: paired accuracy / score-match rate versus the sealed
/// oracle on Development/Validation only.
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

/// Canonical ordered paired primary metrics frozen for Stage-C.
///
/// Empty by design: Mixed has matched case identities but arm-specific oracles,
/// while the remaining families lack matched T6/C6 paths. Candidate ids must
/// remain unadmitted until a common task-level target exists.
pub const PINNED_PRIMARY_FAMILY_METRICS: &[PrimaryFamilyMetricId] = &[];

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
    SecondaryDiagnosticId::TorsorTransportReductionPointIdentityError,
    SecondaryDiagnosticId::ChiralMirrorSwapParityIdentityError,
    SecondaryDiagnosticId::CalibrationStability,
    SecondaryDiagnosticId::OpCountMemoryRuntime,
];

/// Frozen Stage-C metric registry: ordered computable paired primaries,
/// an optional cross-family summary, and ordered secondaries.
///
/// Experimental and non-final only; never authorises protected/final evaluation
/// or a scientific superiority claim.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MetricRegistry {
    /// Ordered primary family quality metrics frozen before evaluation.
    pub primary_family_metrics: &'static [PrimaryFamilyMetricId],
    /// Cross-family summary, unavailable until multiple matched families exist.
    pub cross_family_summary: Option<CrossFamilySummaryMetricId>,
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
            cross_family_summary: None,
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

/// Validate a metric registry: reject contract drift, empty primary, any
/// premature cross-family summary, duplicate secondaries, oversized sets, invented ids,
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
    }
    for primary in registry.primary_family_metrics {
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
    if registry.cross_family_summary.is_some() {
        return Err(EvalError::MetricRegistryInvalid {
            reason: "cross_family_summary_unavailable",
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
    pub split: DataSplit,
    pub family: TaskFamily,
    pub case_id: u64,
    pub outcome: T6Outcome,
    pub canonical_digest: String,
    pub envelope_contract: &'static str,
    pub arm_contract: &'static str,
    pub budget_contract: &'static str,
    pub metric_registry_contract: &'static str,
    pub source_torsor_contract: &'static str,
    pub label_contract: &'static str,
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
    pub split: DataSplit,
    pub family: TaskFamily,
    pub case_id: u64,
    pub outcome: C6Outcome,
    pub canonical_digest: String,
    pub envelope_contract: &'static str,
    pub arm_contract: &'static str,
    pub budget_contract: &'static str,
    pub metric_registry_contract: &'static str,
    pub source_chiral_contract: &'static str,
    pub label_contract: &'static str,
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
    pub split: DataSplit,
    pub family: TaskFamily,
    pub case_id: u64,
    pub outcome: G6Outcome,
    pub canonical_digest: String,
    pub envelope_contract: &'static str,
    pub arm_contract: &'static str,
    pub budget_contract: &'static str,
    pub metric_registry_contract: &'static str,
    pub source_generic_contract: &'static str,
    pub label_contract: &'static str,
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
        assert!(PINNED_PRIMARY_FAMILY_METRICS.is_empty());
        assert!(PINNED_PRIMARY_FAMILY_METRICS.len() <= MAX_PRIMARY_FAMILY_METRICS);
        assert!(PINNED_SECONDARY_DIAGNOSTICS.len() <= MAX_SECONDARY_DIAGNOSTICS);
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
        assert_eq!(pinned.cross_family_summary, None);
        assert_eq!(pinned.secondaries, PINNED_SECONDARY_DIAGNOSTICS);
        assert_eq!(pinned.registry_contract, METRIC_REGISTRY_CONTRACT);
        assert!(pinned.experimental_non_final);
        validate_metric_registry(&pinned).unwrap();

        let frozen = freeze_metric_registry(
            PINNED_PRIMARY_FAMILY_METRICS,
            None,
            PINNED_SECONDARY_DIAGNOSTICS,
        )
        .unwrap();
        assert_eq!(frozen, pinned);

        assert!(pinned.primary_family_metrics.is_empty());
        assert_eq!(pinned.cross_family_summary, None);
        assert_eq!(
            pinned.secondaries[0].as_str(),
            SECONDARY_TORSOR_TRANSPORT_REDUCTION_POINT_IDENTITY_ERROR
        );
        assert_eq!(
            pinned.secondaries[1].as_str(),
            SECONDARY_CHIRAL_MIRROR_SWAP_PARITY_IDENTITY_ERROR
        );
        assert_eq!(
            pinned.secondaries[2].as_str(),
            SECONDARY_CALIBRATION_STABILITY
        );
        assert_eq!(
            pinned.secondaries[3].as_str(),
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
            freeze_metric_registry(&[], None, PINNED_SECONDARY_DIAGNOSTICS).unwrap(),
            MetricRegistry::pinned()
        );
        assert_eq!(
            freeze_metric_registry(
                PINNED_PRIMARY_FAMILY_METRICS,
                Some(CrossFamilySummaryMetricId::CrossFamilyPairedSummary),
                PINNED_SECONDARY_DIAGNOSTICS,
            ),
            Err(EvalError::MetricRegistryInvalid {
                reason: "cross_family_summary_unavailable",
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
            SecondaryDiagnosticId::TorsorTransportReductionPointIdentityError,
            SecondaryDiagnosticId::TorsorTransportReductionPointIdentityError,
        ];
        assert_eq!(
            freeze_metric_registry(PINNED_PRIMARY_FAMILY_METRICS, None, DUPLICATE_SECONDARIES,),
            Err(EvalError::MetricRegistryInvalid {
                reason: "duplicate_secondary",
            })
        );

        const DUPLICATE_PRIMARIES: &[PrimaryFamilyMetricId] =
            &[PrimaryFamilyMetricId::Mixed, PrimaryFamilyMetricId::Mixed];
        assert_eq!(
            freeze_metric_registry(DUPLICATE_PRIMARIES, None, PINNED_SECONDARY_DIAGNOSTICS,),
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
            freeze_metric_registry(PINNED_PRIMARY_FAMILY_METRICS, None, OVERSIZED_SECONDARIES,),
            Err(EvalError::MetricRegistryInvalid {
                reason: "too_many_secondaries",
            })
        );

        const TRUNCATED_SECONDARIES: &[SecondaryDiagnosticId] = &[
            SecondaryDiagnosticId::TorsorTransportReductionPointIdentityError,
            SecondaryDiagnosticId::ChiralMirrorSwapParityIdentityError,
            SecondaryDiagnosticId::CalibrationStability,
        ];
        assert_eq!(
            freeze_metric_registry(PINNED_PRIMARY_FAMILY_METRICS, None, TRUNCATED_SECONDARIES,),
            Err(EvalError::MetricRegistryInvalid {
                reason: "secondary_sequence_mismatch",
            })
        );
        assert_eq!(
            freeze_metric_registry(PINNED_PRIMARY_FAMILY_METRICS, None, &[],),
            Err(EvalError::MetricRegistryInvalid {
                reason: "secondary_sequence_mismatch",
            })
        );

        const REORDERED_SECONDARIES: &[SecondaryDiagnosticId] = &[
            SecondaryDiagnosticId::ChiralMirrorSwapParityIdentityError,
            SecondaryDiagnosticId::TorsorTransportReductionPointIdentityError,
            SecondaryDiagnosticId::CalibrationStability,
            SecondaryDiagnosticId::OpCountMemoryRuntime,
        ];
        assert_eq!(
            freeze_metric_registry(PINNED_PRIMARY_FAMILY_METRICS, None, REORDERED_SECONDARIES,),
            Err(EvalError::MetricRegistryInvalid {
                reason: "secondary_sequence_mismatch",
            })
        );

        const UNMATCHED_PRIMARIES: &[PrimaryFamilyMetricId] = &[
            PrimaryFamilyMetricId::TorsorFavorable,
            PrimaryFamilyMetricId::Mixed,
        ];
        assert_eq!(
            freeze_metric_registry(UNMATCHED_PRIMARIES, None, PINNED_SECONDARY_DIAGNOSTICS,),
            Err(EvalError::MetricRegistryInvalid {
                reason: "unknown_primary",
            })
        );

        const UNPAIRED_PRIMARY: &[PrimaryFamilyMetricId] =
            &[PrimaryFamilyMetricId::TorsorFavorable];
        assert_eq!(
            freeze_metric_registry(UNPAIRED_PRIMARY, None, PINNED_SECONDARY_DIAGNOSTICS,),
            Err(EvalError::MetricRegistryInvalid {
                reason: "unknown_primary",
            })
        );

        let malformed = MetricRegistry {
            primary_family_metrics: &[PrimaryFamilyMetricId::Mixed],
            ..MetricRegistry::pinned()
        };
        assert_eq!(
            malformed.admit_split(DataSplit::Development),
            Err(EvalError::MetricRegistryInvalid {
                reason: "unknown_primary",
            })
        );
        let mut config = EvaluatorConfig::t6(DataSplit::Development);
        config.metric_registry = malformed;
        assert_eq!(
            T6EvaluatorRun::open(config),
            Err(EvalError::MetricRegistryInvalid {
                reason: "unknown_primary",
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
    fn paired_and_g6_candidate_metrics_remain_unadmitted_without_common_targets() {
        assert!(PINNED_PRIMARY_FAMILY_METRICS.is_empty());
        assert!(!PINNED_SECONDARY_DIAGNOSTICS
            .contains(&SecondaryDiagnosticId::PairedOutcomeDifference));
        assert!(!PINNED_SECONDARY_DIAGNOSTICS
            .contains(&SecondaryDiagnosticId::G6AttributionContrast));
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
}
