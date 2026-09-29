//! TDI-24 Phase-C matched evaluation machinery.
//!
//! Slice 21 lands the shared evaluator envelope and the V6 arm: a deterministic
//! non-final Development/Validation path that scores through the matched vector
//! reference, never observes sealed targets inside the inference callback, and
//! refuses protected/final population labels. Slice 22 adds the matched C6 arm
//! under the same envelope and readout budget. Slice 23 adds an explicit
//! trainable-capacity / parameter-count matcher that accepts only matched V6/C6
//! configurations and rejects unmatched capacity fail-closed. Slice 24 adds a
//! paired deterministic initialization-policy matcher so V6/C6 share one seed
//! stream and kind, rejecting unpaired draws fail-closed. Slice 25 adds an
//! optimizer/update-budget matcher requiring identical examples, ordering,
//! steps and stopping rule across paired arms. Slice 26 freezes the versioned
//! metric registry: one primary quality metric plus an ordered closed set of
//! secondary diagnostics for Stage-C, experimental and non-final only. Slice 27
//! adds the paired uncertainty engine: confidence intervals and effect
//! summaries over already-revealed V6/C6 match outcomes without accepting raw
//! protected labels. No training, confirmatory execution, protected/final
//! evaluation, or scientific claim is authorised here.

use core::fmt;

use super::tdi24_accounting::ScoreArm;
use super::tdi24_chiral::{
    CHIRAL_CONTRACT, CHIRAL_WIDTH, Chiral6, ChiralError, ChiralScoreWeights, chiral_score,
};
use super::tdi24_tasks::{
    DataSplit, DirectionTarget, HandednessTarget, InferenceView, LabeledCase, NonChiralTarget,
    PROTECTED_LABEL_CONTRACT, ReflectionInvariantTarget, TaskFamily, canonicalize_inference_view,
    run_inference_callback,
};
use super::tdi24_vector::{VECTOR6_CONTRACT, VECTOR6_WIDTH, Vector6, Vector6Error, vector6_score};

/// Shared evaluator envelope consumed by V6 now and by later matched arms.
pub const EVALUATOR_ENVELOPE_CONTRACT: &str = "tdi24-evaluator-envelope-v1";

/// Versioned V6 evaluator contract.
pub const V6_EVALUATOR_CONTRACT: &str = "tdi24-v6-evaluator-v1";

/// Versioned C6 evaluator contract.
pub const C6_EVALUATOR_CONTRACT: &str = "tdi24-c6-evaluator-v1";

/// Matched readout-budget contract shared across Phase-C evaluator arms.
pub const READOUT_BUDGET_CONTRACT: &str = "tdi24-readout-budget-v1";

/// Versioned trainable-capacity / parameter-count matcher contract.
pub const PARAMETER_COUNT_MATCHER_CONTRACT: &str = "tdi24-parameter-count-matcher-v1";

/// Versioned paired deterministic initialization-policy matcher contract.
pub const INITIALIZATION_MATCHER_CONTRACT: &str = "tdi24-initialization-matcher-v1";

/// Versioned optimizer/update-budget matcher contract.
pub const OPTIMIZER_UPDATE_BUDGET_CONTRACT: &str = "tdi24-optimizer-update-budget-v1";

/// Versioned Stage-C metric-registry contract.
pub const METRIC_REGISTRY_CONTRACT: &str = "tdi24-metric-registry-v1";

/// Primary metric id: paired task accuracy / score-match rate vs oracle.
pub const PRIMARY_METRIC_PAIRED_TASK_ACCURACY: &str = "paired_task_accuracy";

/// Secondary diagnostic: paired task outcome difference (V6 vs C6).
pub const SECONDARY_PAIRED_OUTCOME_DIFFERENCE: &str = "paired_outcome_difference";

/// Secondary diagnostic: mirror-swap identity error.
pub const SECONDARY_MIRROR_SWAP_IDENTITY_ERROR: &str = "mirror_swap_identity_error";

/// Secondary diagnostic: parity-equivariance / invariance error.
pub const SECONDARY_PARITY_EQUIVARIANCE_INVARIANCE_ERROR: &str =
    "parity_equivariance_invariance_error";

/// Secondary diagnostic: calibration / confidence error when scores are exposed.
pub const SECONDARY_CALIBRATION_CONFIDENCE_ERROR: &str = "calibration_confidence_error";

/// Secondary diagnostic: gradient / stability for trained arms.
pub const SECONDARY_GRADIENT_STABILITY: &str = "gradient_stability";

/// Secondary diagnostic: op-count / memory / latency / throughput under a qualified env only.
pub const SECONDARY_OP_COUNT_MEMORY_LATENCY_THROUGHPUT: &str = "op_count_memory_latency_throughput";

/// Maximum secondary diagnostics admitted in one frozen registry.
pub const MAX_SECONDARY_DIAGNOSTICS: usize = 6;

/// Versioned paired uncertainty engine contract.
pub const PAIRED_UNCERTAINTY_CONTRACT: &str = "tdi24-paired-uncertainty-v1";

/// Nominal two-sided confidence level for Stage-C paired uncertainty summaries.
pub const PAIRED_UNCERTAINTY_LEVEL: f64 = 0.95;

/// Φ^{-1}(0.975) critical value for two-sided 95% Wilson intervals.
/// Deterministic constant; no RNG and no runtime table lookup.
pub const NORMAL_CRITICAL_Z_95: f64 = 1.959_963_984_540_054;

/// ln(2 / 0.05), used by deterministic two-sided 95% Hoeffding bounds.
/// Kept as a literal so the uncertainty surface does not depend on runtime logs.
pub const HOEFFDING_LOG_40: f64 = 3.688_879_454_113_936_3;

/// Maximum cases admitted to one non-final evaluator run.
pub const MAX_CASES_PER_RUN: u64 = 64;

/// Maximum scalar fields retained per case readout (score + correctness flag).
pub const MAX_READOUT_SCALARS_PER_CASE: u64 = 2;

/// Non-trained update budget for the Slice-21 reference path.
pub const NON_TRAINED_UPDATE_BUDGET: u64 = 0;

/// Evaluation arm identity for Phase C.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EvalArm {
    /// Matched six-dimensional vector control.
    V6,
    /// Matched six-dimensional chiral candidate.
    C6,
}

impl EvalArm {
    /// Stable lowercase arm label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::V6 => "v6",
            Self::C6 => "c6",
        }
    }

    /// Arm-specific evaluator contract pin.
    #[must_use]
    pub const fn evaluator_contract(self) -> &'static str {
        match self {
            Self::V6 => V6_EVALUATOR_CONTRACT,
            Self::C6 => C6_EVALUATOR_CONTRACT,
        }
    }

    /// Accounting arm used for operation/storage pairing.
    #[must_use]
    pub const fn score_arm(self) -> ScoreArm {
        match self {
            Self::V6 => ScoreArm::V6,
            Self::C6 => ScoreArm::C6,
        }
    }
}

/// Fixed matched readout budget shared by V6 and later C6.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReadoutBudget {
    /// Maximum cases admitted to one run.
    pub max_cases: u64,
    /// Maximum scalar fields retained per case.
    pub max_readout_scalars_per_case: u64,
    /// Parameter-update steps; zero on the non-trained reference path.
    pub updates: u64,
    /// Budget contract pin.
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

/// Immutable configuration for one non-final evaluator run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EvaluatorConfig {
    /// Development or Validation only.
    pub split: DataSplit,
    /// Arm under evaluation.
    pub arm: EvalArm,
    /// Matched readout budget.
    pub budget: ReadoutBudget,
    /// Shared envelope contract pin.
    pub envelope_contract: &'static str,
    /// Validated, exact Stage-C metric registry.
    pub metric_registry: MetricRegistry,
}

impl EvaluatorConfig {
    /// Construct a fail-closed V6 Development/Validation configuration.
    pub fn v6(split: DataSplit) -> Result<Self, EvalError> {
        validate_non_final_split(split)?;
        Ok(Self {
            split,
            arm: EvalArm::V6,
            budget: ReadoutBudget::matched_non_trained(),
            envelope_contract: EVALUATOR_ENVELOPE_CONTRACT,
            metric_registry: MetricRegistry::pinned(),
        })
    }

    /// Construct a fail-closed C6 Development/Validation configuration.
    pub fn c6(split: DataSplit) -> Result<Self, EvalError> {
        validate_non_final_split(split)?;
        Ok(Self {
            split,
            arm: EvalArm::C6,
            budget: ReadoutBudget::matched_non_trained(),
            envelope_contract: EVALUATOR_ENVELOPE_CONTRACT,
            metric_registry: MetricRegistry::pinned(),
        })
    }
}

/// Retained failure category for one evaluated case.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvalFailure {
    /// Arm arithmetic rejected a non-finite intermediate.
    Numerical,
    /// Task/oracle mapping could not be applied.
    Task,
    /// Readout or case budget exhausted.
    Resource,
    /// Contract or split identity rejected.
    Contract,
}

impl EvalFailure {
    /// Stable lowercase failure token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Numerical => "numerical",
            Self::Task => "task",
            Self::Resource => "resource",
            Self::Contract => "contract",
        }
    }
}

/// Outcome retained for one evaluated case.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum EvalOutcome {
    /// Finite arm score plus sealed-label correctness on the evaluation path.
    Scored { score: f64, correct: bool },
    /// Typed failure retained instead of dropping the case.
    Failure(EvalFailure),
}

/// One immutable non-final evaluation record.
#[derive(Clone, Debug, PartialEq)]
pub struct EvalRecord {
    pub arm: EvalArm,
    pub split: DataSplit,
    pub family: TaskFamily,
    pub case_id: u64,
    pub group_id: u64,
    pub outcome: EvalOutcome,
    pub canonical_digest: String,
    pub envelope_contract: &'static str,
    pub arm_contract: &'static str,
    pub budget_contract: &'static str,
    pub metric_registry_contract: &'static str,
    pub vector_contract: &'static str,
    pub label_contract: &'static str,
}

/// Accumulator that enforces the matched readout budget.
#[derive(Clone, Debug, PartialEq)]
pub struct EvaluatorRun {
    config: EvaluatorConfig,
    records: Vec<EvalRecord>,
}

impl EvaluatorRun {
    /// Open a bounded non-final run.
    pub fn open(config: EvaluatorConfig) -> Result<Self, EvalError> {
        config.metric_registry.admit_split(config.split)?;
        if config.envelope_contract != EVALUATOR_ENVELOPE_CONTRACT {
            return Err(EvalError::ContractMismatch {
                field: "envelope_contract",
            });
        }
        if config.budget.contract != READOUT_BUDGET_CONTRACT {
            return Err(EvalError::ContractMismatch {
                field: "budget_contract",
            });
        }
        if config.budget.max_cases == 0 || config.budget.max_readout_scalars_per_case == 0 {
            return Err(EvalError::InvalidBudget);
        }
        Ok(Self {
            config,
            records: Vec::new(),
        })
    }

    /// Borrow the immutable run configuration.
    #[must_use]
    pub const fn config(&self) -> &EvaluatorConfig {
        &self.config
    }

    /// Borrow emitted records in admission order.
    #[must_use]
    pub fn records(&self) -> &[EvalRecord] {
        &self.records
    }

    /// Evaluate one sealed labeled case through the V6 path.
    pub fn evaluate_v6_binary<T, S>(
        &mut self,
        case: &LabeledCase<T>,
        oracle_sign: S,
    ) -> Result<&EvalRecord, EvalError>
    where
        S: FnOnce(&T) -> Result<i8, EvalError>,
    {
        if self.records.len() as u64 >= self.config.budget.max_cases {
            return Err(EvalError::CaseBudgetExceeded);
        }

        let view = case.inference_view();
        if view.split != self.config.split {
            return Err(EvalError::SplitMismatch {
                expected: self.config.split,
                actual: view.split,
            });
        }

        // Inference callback sees only the view; sealed target stays outside.
        let scored = run_inference_callback(case, score_v6_from_view);

        let outcome = match scored {
            Ok(score) => {
                if self.config.budget.max_readout_scalars_per_case < 2 {
                    EvalOutcome::Failure(EvalFailure::Resource)
                } else {
                    match oracle_sign(case.protected_label().reveal_for_evaluation()) {
                        Ok(sign) if sign == 1 || sign == -1 => {
                            let correct = score * f64::from(sign) > 0.0;
                            EvalOutcome::Scored { score, correct }
                        }
                        Ok(_) => EvalOutcome::Failure(EvalFailure::Task),
                        Err(_) => EvalOutcome::Failure(EvalFailure::Task),
                    }
                }
            }
            Err(EvalError::Numerical(_)) => EvalOutcome::Failure(EvalFailure::Numerical),
            Err(EvalError::ContractMismatch { .. }) => EvalOutcome::Failure(EvalFailure::Contract),
            Err(_) => EvalOutcome::Failure(EvalFailure::Task),
        };

        let digest = canonicalize_inference_view(view).digest;
        self.records.push(EvalRecord {
            arm: EvalArm::V6,
            split: view.split,
            family: view.family,
            case_id: view.case_id,
            group_id: view.group_id,
            outcome,
            canonical_digest: digest,
            envelope_contract: EVALUATOR_ENVELOPE_CONTRACT,
            arm_contract: V6_EVALUATOR_CONTRACT,
            budget_contract: READOUT_BUDGET_CONTRACT,
            metric_registry_contract: self.config.metric_registry.registry_contract,
            vector_contract: VECTOR6_CONTRACT,
            label_contract: view.label_contract,
        });
        Ok(self.records.last().expect("just pushed"))
    }

    /// Evaluate one sealed labeled case through the C6 path.
    pub fn evaluate_c6_binary<T, S>(
        &mut self,
        case: &LabeledCase<T>,
        oracle_sign: S,
    ) -> Result<&EvalRecord, EvalError>
    where
        S: FnOnce(&T) -> Result<i8, EvalError>,
    {
        if self.config.arm != EvalArm::C6 {
            return Err(EvalError::UnsupportedArm);
        }
        if self.records.len() as u64 >= self.config.budget.max_cases {
            return Err(EvalError::CaseBudgetExceeded);
        }
        let view = case.inference_view();
        if view.split != self.config.split {
            return Err(EvalError::SplitMismatch {
                expected: self.config.split,
                actual: view.split,
            });
        }
        let scored = run_inference_callback(case, score_c6_from_view);
        let outcome = match scored {
            Ok(score) if self.config.budget.max_readout_scalars_per_case >= 2 => {
                match oracle_sign(case.protected_label().reveal_for_evaluation()) {
                    Ok(sign) if sign == 1 || sign == -1 => EvalOutcome::Scored {
                        score,
                        correct: score * f64::from(sign) > 0.0,
                    },
                    _ => EvalOutcome::Failure(EvalFailure::Task),
                }
            }
            Ok(_) => EvalOutcome::Failure(EvalFailure::Resource),
            Err(EvalError::ChiralNumerical(_)) => EvalOutcome::Failure(EvalFailure::Numerical),
            Err(EvalError::ContractMismatch { .. }) => EvalOutcome::Failure(EvalFailure::Contract),
            Err(_) => EvalOutcome::Failure(EvalFailure::Task),
        };
        let digest = canonicalize_inference_view(view).digest;
        self.records.push(EvalRecord {
            arm: EvalArm::C6,
            split: view.split,
            family: view.family,
            case_id: view.case_id,
            group_id: view.group_id,
            outcome,
            canonical_digest: digest,
            envelope_contract: EVALUATOR_ENVELOPE_CONTRACT,
            arm_contract: C6_EVALUATOR_CONTRACT,
            budget_contract: READOUT_BUDGET_CONTRACT,
            metric_registry_contract: self.config.metric_registry.registry_contract,
            vector_contract: CHIRAL_CONTRACT,
            label_contract: view.label_contract,
        });
        Ok(self.records.last().expect("just pushed"))
    }
}

/// Score an inference view with the matched C6 chiral reference.
pub fn score_c6_from_view(view: &InferenceView) -> Result<f64, EvalError> {
    let q = view.query.as_array();
    let k = view.key.as_array();
    let query = Chiral6::from_array(q).map_err(EvalError::ChiralNumerical)?;
    let key = Chiral6::from_array(k).map_err(EvalError::ChiralNumerical)?;
    let weights = ChiralScoreWeights::new(1.0, 0.0, 1.0).map_err(EvalError::ChiralNumerical)?;
    chiral_score(query, key, weights).map_err(EvalError::ChiralNumerical)
}

/// Score an inference view with the matched V6 vector reference.
pub fn score_v6_from_view(view: &InferenceView) -> Result<f64, EvalError> {
    let query = Vector6::new(view.query.as_array()).map_err(EvalError::Numerical)?;
    let key = Vector6::new(view.key.as_array()).map_err(EvalError::Numerical)?;
    vector6_score(query, key).map_err(EvalError::Numerical)
}

/// Explicit trainable-capacity descriptor for one Phase-C arm.
///
/// The non-trained Slice-21/22 reference path declares zero trainable
/// parameters at matched carrier width. Later trained arms must publish an
/// exact count through this descriptor rather than silently compensating.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrainableCapacity {
    /// Arm whose capacity is declared.
    pub arm: EvalArm,
    /// Exact trainable parameter count for this configuration.
    pub trainable_parameters: u64,
    /// Carrier width consumed by the arm.
    pub carrier_width: u64,
    /// Matcher contract pin.
    pub matcher_contract: &'static str,
}

impl TrainableCapacity {
    /// Canonical non-trained V6 reference capacity.
    #[must_use]
    pub const fn reference_v6() -> Self {
        Self {
            arm: EvalArm::V6,
            trainable_parameters: 0,
            carrier_width: VECTOR6_WIDTH as u64,
            matcher_contract: PARAMETER_COUNT_MATCHER_CONTRACT,
        }
    }

    /// Canonical non-trained C6 reference capacity.
    #[must_use]
    pub const fn reference_c6() -> Self {
        Self {
            arm: EvalArm::C6,
            trainable_parameters: 0,
            carrier_width: CHIRAL_WIDTH as u64,
            matcher_contract: PARAMETER_COUNT_MATCHER_CONTRACT,
        }
    }
}

/// Witness that two arm capacities are matched under the Slice-23 contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MatchedParameterCount {
    pub trainable_parameters: u64,
    pub carrier_width: u64,
    pub left_arm: EvalArm,
    pub right_arm: EvalArm,
    pub matcher_contract: &'static str,
}

/// Accept only matched trainable-capacity configurations; reject mismatches
/// fail-closed without silently compensating.
pub fn match_parameter_counts(
    left: TrainableCapacity,
    right: TrainableCapacity,
) -> Result<MatchedParameterCount, EvalError> {
    if left.matcher_contract != PARAMETER_COUNT_MATCHER_CONTRACT
        || right.matcher_contract != PARAMETER_COUNT_MATCHER_CONTRACT
    {
        return Err(EvalError::ContractMismatch {
            field: "parameter_count_matcher_contract",
        });
    }
    if left.trainable_parameters != right.trainable_parameters {
        return Err(EvalError::ParameterCountMismatch {
            left_arm: left.arm,
            right_arm: right.arm,
            left_parameters: left.trainable_parameters,
            right_parameters: right.trainable_parameters,
        });
    }
    if left.carrier_width != right.carrier_width {
        return Err(EvalError::ParameterCountMismatch {
            left_arm: left.arm,
            right_arm: right.arm,
            left_parameters: left.trainable_parameters,
            right_parameters: right.trainable_parameters,
        });
    }
    Ok(MatchedParameterCount {
        trainable_parameters: left.trainable_parameters,
        carrier_width: left.carrier_width,
        left_arm: left.arm,
        right_arm: right.arm,
        matcher_contract: PARAMETER_COUNT_MATCHER_CONTRACT,
    })
}

/// Deterministic initialization kind shared by paired Phase-C arms.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InitializationKind {
    /// Non-trained reference path: no parameter draw is performed.
    ReferenceNone,
    /// Deterministic paired draw from a shared seed stream (trained arms).
    DeterministicPaired,
}

impl InitializationKind {
    /// Stable lowercase label for manifests and audits.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReferenceNone => "reference_none",
            Self::DeterministicPaired => "deterministic_paired",
        }
    }
}

/// Explicit initialization policy for one Phase-C arm.
///
/// Paired V6/C6 configurations must publish identical seed and kind through
/// this descriptor rather than silently using divergent draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InitializationPolicy {
    /// Arm whose initialization is declared.
    pub arm: EvalArm,
    /// Initialization kind for this configuration.
    pub kind: InitializationKind,
    /// Shared deterministic seed for the paired draw stream.
    pub seed: u64,
    /// Stream discriminator kept identical across paired arms.
    pub stream_id: u64,
    /// Matcher contract pin.
    pub matcher_contract: &'static str,
}

impl InitializationPolicy {
    /// Canonical non-trained V6 reference initialization (no parameter draw).
    #[must_use]
    pub const fn reference_v6(seed: u64) -> Self {
        Self {
            arm: EvalArm::V6,
            kind: InitializationKind::ReferenceNone,
            seed,
            stream_id: 0,
            matcher_contract: INITIALIZATION_MATCHER_CONTRACT,
        }
    }

    /// Canonical non-trained C6 reference initialization (no parameter draw).
    #[must_use]
    pub const fn reference_c6(seed: u64) -> Self {
        Self {
            arm: EvalArm::C6,
            kind: InitializationKind::ReferenceNone,
            seed,
            stream_id: 0,
            matcher_contract: INITIALIZATION_MATCHER_CONTRACT,
        }
    }
}

/// Witness that two arm initialization policies are matched under Slice 24.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MatchedInitialization {
    pub kind: InitializationKind,
    pub seed: u64,
    pub stream_id: u64,
    pub left_arm: EvalArm,
    pub right_arm: EvalArm,
    pub matcher_contract: &'static str,
}

/// Accept only paired deterministic initialization policies; reject unpaired
/// seeds/kinds fail-closed without silently compensating.
pub fn match_initialization(
    left: InitializationPolicy,
    right: InitializationPolicy,
) -> Result<MatchedInitialization, EvalError> {
    if left.matcher_contract != INITIALIZATION_MATCHER_CONTRACT
        || right.matcher_contract != INITIALIZATION_MATCHER_CONTRACT
    {
        return Err(EvalError::ContractMismatch {
            field: "initialization_matcher_contract",
        });
    }
    if left.kind != right.kind || left.seed != right.seed || left.stream_id != right.stream_id {
        return Err(EvalError::InitializationMismatch {
            left_arm: left.arm,
            right_arm: right.arm,
            left_seed: left.seed,
            right_seed: right.seed,
        });
    }
    Ok(MatchedInitialization {
        kind: left.kind,
        seed: left.seed,
        stream_id: left.stream_id,
        left_arm: left.arm,
        right_arm: right.arm,
        matcher_contract: INITIALIZATION_MATCHER_CONTRACT,
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
/// Paired V6/C6 configurations must publish identical examples, ordering seed,
/// update steps and stopping rule rather than silently diverging.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OptimizerUpdateBudget {
    /// Arm whose budget is declared.
    pub arm: EvalArm,
    /// Number of examples admitted under this budget.
    pub examples: u64,
    /// Parameter-update steps (zero on the non-trained reference path).
    pub updates: u64,
    /// Shared deterministic seed fixing example ordering across paired arms.
    pub ordering_seed: u64,
    /// Stopping rule applied identically to both arms.
    pub stopping: StoppingRule,
    /// Matcher contract pin.
    pub matcher_contract: &'static str,
}

impl OptimizerUpdateBudget {
    /// Canonical non-trained V6 reference budget.
    #[must_use]
    pub const fn reference_v6(examples: u64, ordering_seed: u64) -> Self {
        Self {
            arm: EvalArm::V6,
            examples,
            updates: NON_TRAINED_UPDATE_BUDGET,
            ordering_seed,
            stopping: StoppingRule::ExhaustExamples,
            matcher_contract: OPTIMIZER_UPDATE_BUDGET_CONTRACT,
        }
    }

    /// Canonical non-trained C6 reference budget.
    #[must_use]
    pub const fn reference_c6(examples: u64, ordering_seed: u64) -> Self {
        Self {
            arm: EvalArm::C6,
            examples,
            updates: NON_TRAINED_UPDATE_BUDGET,
            ordering_seed,
            stopping: StoppingRule::ExhaustExamples,
            matcher_contract: OPTIMIZER_UPDATE_BUDGET_CONTRACT,
        }
    }
}

/// Witness that two optimizer/update budgets are matched under Slice 25.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MatchedOptimizerUpdateBudget {
    pub examples: u64,
    pub updates: u64,
    pub ordering_seed: u64,
    pub stopping: StoppingRule,
    pub left_arm: EvalArm,
    pub right_arm: EvalArm,
    pub matcher_contract: &'static str,
}

/// Accept only matched optimizer/update budgets; reject unpaired examples,
/// ordering, steps or stopping rules fail-closed.
pub fn match_optimizer_update_budgets(
    left: OptimizerUpdateBudget,
    right: OptimizerUpdateBudget,
) -> Result<MatchedOptimizerUpdateBudget, EvalError> {
    if left.matcher_contract != OPTIMIZER_UPDATE_BUDGET_CONTRACT
        || right.matcher_contract != OPTIMIZER_UPDATE_BUDGET_CONTRACT
    {
        return Err(EvalError::ContractMismatch {
            field: "optimizer_update_budget_contract",
        });
    }
    if left.examples == 0 || right.examples == 0 {
        return Err(EvalError::InvalidBudget);
    }
    if left.examples != right.examples
        || left.updates != right.updates
        || left.ordering_seed != right.ordering_seed
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
        ordering_seed: left.ordering_seed,
        stopping: left.stopping,
        left_arm: left.arm,
        right_arm: right.arm,
        matcher_contract: OPTIMIZER_UPDATE_BUDGET_CONTRACT,
    })
}

/// Closed-set primary quality metric for Stage-C evaluation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PrimaryMetricId {
    /// Paired task accuracy / score-match rate versus the sealed oracle on
    /// Development/Validation only.
    PairedTaskAccuracy,
}

impl PrimaryMetricId {
    /// Stable lowercase metric id.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PairedTaskAccuracy => PRIMARY_METRIC_PAIRED_TASK_ACCURACY,
        }
    }
}

/// Parse a primary metric id from the closed Stage-C set.
pub fn parse_primary_metric_id(label: &str) -> Result<PrimaryMetricId, EvalError> {
    match label {
        PRIMARY_METRIC_PAIRED_TASK_ACCURACY => Ok(PrimaryMetricId::PairedTaskAccuracy),
        "" => Err(EvalError::MetricRegistryInvalid {
            reason: "empty_primary",
        }),
        _ => Err(EvalError::MetricRegistryInvalid {
            reason: "unknown_primary",
        }),
    }
}

/// Closed-set secondary diagnostics admitted for Stage-C under the frozen registry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SecondaryDiagnosticId {
    /// Paired task outcome difference across matched arms.
    PairedOutcomeDifference,
    /// Mirror-swap identity error.
    MirrorSwapIdentityError,
    /// Parity-equivariance / invariance error where applicable.
    ParityEquivarianceInvarianceError,
    /// Calibration / confidence error when predictions expose scores.
    CalibrationConfidenceError,
    /// Gradient / stability diagnostics for trained arms.
    GradientStability,
    /// Operation count, memory, latency, throughput under an explicitly qualified environment.
    OpCountMemoryLatencyThroughput,
}

impl SecondaryDiagnosticId {
    /// Stable lowercase diagnostic id.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PairedOutcomeDifference => SECONDARY_PAIRED_OUTCOME_DIFFERENCE,
            Self::MirrorSwapIdentityError => SECONDARY_MIRROR_SWAP_IDENTITY_ERROR,
            Self::ParityEquivarianceInvarianceError => {
                SECONDARY_PARITY_EQUIVARIANCE_INVARIANCE_ERROR
            }
            Self::CalibrationConfidenceError => SECONDARY_CALIBRATION_CONFIDENCE_ERROR,
            Self::GradientStability => SECONDARY_GRADIENT_STABILITY,
            Self::OpCountMemoryLatencyThroughput => SECONDARY_OP_COUNT_MEMORY_LATENCY_THROUGHPUT,
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
        SECONDARY_MIRROR_SWAP_IDENTITY_ERROR => Ok(SecondaryDiagnosticId::MirrorSwapIdentityError),
        SECONDARY_PARITY_EQUIVARIANCE_INVARIANCE_ERROR => {
            Ok(SecondaryDiagnosticId::ParityEquivarianceInvarianceError)
        }
        SECONDARY_CALIBRATION_CONFIDENCE_ERROR => {
            Ok(SecondaryDiagnosticId::CalibrationConfidenceError)
        }
        SECONDARY_GRADIENT_STABILITY => Ok(SecondaryDiagnosticId::GradientStability),
        SECONDARY_OP_COUNT_MEMORY_LATENCY_THROUGHPUT => {
            Ok(SecondaryDiagnosticId::OpCountMemoryLatencyThroughput)
        }
        _ => Err(EvalError::MetricRegistryInvalid {
            reason: "unknown_secondary",
        }),
    }
}

/// Canonical ordered secondary diagnostics frozen for Stage-C.
pub const PINNED_SECONDARY_DIAGNOSTICS: &[SecondaryDiagnosticId] = &[
    SecondaryDiagnosticId::PairedOutcomeDifference,
    SecondaryDiagnosticId::MirrorSwapIdentityError,
    SecondaryDiagnosticId::ParityEquivarianceInvarianceError,
    SecondaryDiagnosticId::CalibrationConfidenceError,
    SecondaryDiagnosticId::GradientStability,
    SecondaryDiagnosticId::OpCountMemoryLatencyThroughput,
];

/// Frozen Stage-C metric registry: one primary metric plus ordered secondaries.
///
/// Experimental and non-final only; never authorises protected/final evaluation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MetricRegistry {
    /// Primary quality metric frozen before evaluation.
    pub primary: PrimaryMetricId,
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
            primary: PrimaryMetricId::PairedTaskAccuracy,
            secondaries: PINNED_SECONDARY_DIAGNOSTICS,
            registry_contract: METRIC_REGISTRY_CONTRACT,
            experimental_non_final: true,
        }
    }

    /// Reject protected/final splits; Development/Validation only.
    pub fn admit_split(self, split: DataSplit) -> Result<(), EvalError> {
        validate_metric_registry(&self)?;
        validate_non_final_split(split)
    }
}

/// Freeze a metric registry under the versioned contract; fail-closed on invalid sets.
pub fn freeze_metric_registry(
    primary: Option<PrimaryMetricId>,
    secondaries: &'static [SecondaryDiagnosticId],
) -> Result<MetricRegistry, EvalError> {
    let Some(primary) = primary else {
        return Err(EvalError::MetricRegistryInvalid {
            reason: "empty_primary",
        });
    };
    let registry = MetricRegistry {
        primary,
        secondaries,
        registry_contract: METRIC_REGISTRY_CONTRACT,
        experimental_non_final: true,
    };
    validate_metric_registry(&registry)?;
    Ok(registry)
}

/// Validate a metric registry: reject contract drift, empty primary, duplicate
/// secondaries, oversized sets, invented ids, non-experimental finals, and any
/// secondary sequence that is not the complete canonical ordered registry.
pub fn validate_metric_registry(registry: &MetricRegistry) -> Result<(), EvalError> {
    if registry.registry_contract != METRIC_REGISTRY_CONTRACT {
        return Err(EvalError::ContractMismatch {
            field: "metric_registry_contract",
        });
    }
    if !registry.experimental_non_final {
        return Err(EvalError::MetricRegistryInvalid {
            reason: "experimental_non_final_required",
        });
    }
    // Re-parse primary through the closed-set gate (guards future variants).
    parse_primary_metric_id(registry.primary.as_str())?;
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

/// Interval construction method retained on every Stage-C uncertainty summary.
///
/// Choice (documented, deterministic, no RNG):
/// - [`UncertaintyMethod::WilsonScore`] for single-arm accuracy rates when
///   callers provide independent revealed match bits.
/// - [`UncertaintyMethod::BoundedHoeffdingPairedDifference`] for direct paired
///   differences in [-1, 1]. The finite-sample bound remains non-degenerate at
///   all-tie and all-win boundaries.
/// - The cluster-aware variants treat each record `group_id` as one independent
///   unit and use the observed cluster sizes in the Hoeffding range term. This
///   preserves related mirrored/reflected/reversal/nuisance cases as clusters.
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

/// Already-revealed match bit admitted into the uncertainty surface.
///
/// Construct only from evaluator-retained `matches_oracle` / `correct` flags
/// (or synthetic test vectors of the same shape). There is intentionally no
/// constructor from [`super::tdi24_tasks::ProtectedLabel`], raw sealed targets,
/// or leak oracles beyond what [`EvalRecord`] already retained.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RevealedMatchOutcome {
    /// Whether the arm score matched the sealed oracle on the evaluation path.
    pub matches_oracle: bool,
}

impl RevealedMatchOutcome {
    /// Build from an evaluator-retained correctness / match-oracle bit.
    #[must_use]
    pub const fn from_matches_oracle(matches_oracle: bool) -> Self {
        Self { matches_oracle }
    }
}

/// Paired V6/C6 effect summary bound to the frozen metric registry.
///
/// Experimental and non-final only; never authorises protected/final evaluation
/// or scientific claims.
#[derive(Clone, Debug, PartialEq)]
pub struct PairedEffectSummary {
    pub split: DataSplit,
    pub n_pairs: u64,
    pub v6_accuracy: f64,
    pub c6_accuracy: f64,
    /// Mean of per-pair differences `I(C6)-I(V6)`; equals `c6_accuracy - v6_accuracy`.
    pub paired_difference_mean: f64,
    pub paired_difference_ci: ConfidenceInterval,
    pub v6_accuracy_ci: ConfidenceInterval,
    pub c6_accuracy_ci: ConfidenceInterval,
    pub primary_metric: PrimaryMetricId,
    pub secondary_paired_outcome_difference: SecondaryDiagnosticId,
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
///
/// `sum_squared_cluster_sizes` is `n` for independent rows and
/// `sum_g n_g^2` for record groups. The paired-difference range is [-1, 1]
/// (width two); Bernoulli means use [0, 1] (width one). Bounds are clipped to
/// the natural range and remain non-degenerate at finite-sample boundaries.
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
    if registry.primary != PrimaryMetricId::PairedTaskAccuracy {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "primary_metric_mismatch",
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
    Ok(())
}

/// Summarise paired V6/C6 match outcomes into accuracy rates, paired effect, and CIs.
///
/// Consumes only already-revealed [`RevealedMatchOutcome`] bits (evaluator-retained
/// match-oracle / correctness flags). Rejects empty pairs, V6/C6 length mismatch,
/// non-finite statistics, contract / registry pin drift, and protected/final splits.
pub fn summarize_paired_uncertainty(
    split: DataSplit,
    v6_matches: &[RevealedMatchOutcome],
    c6_matches: &[RevealedMatchOutcome],
    registry: &MetricRegistry,
) -> Result<PairedEffectSummary, EvalError> {
    summarize_paired_uncertainty_inner(split, v6_matches, c6_matches, registry, None)
}

fn summarize_paired_uncertainty_inner(
    split: DataSplit,
    v6_matches: &[RevealedMatchOutcome],
    c6_matches: &[RevealedMatchOutcome],
    registry: &MetricRegistry,
    cluster_sum_squares: Option<u64>,
) -> Result<PairedEffectSummary, EvalError> {
    validate_non_final_split(split)?;
    require_pinned_metric_registry(registry)?;

    if v6_matches.is_empty() || c6_matches.is_empty() {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "empty_pairs",
        });
    }
    if v6_matches.len() != c6_matches.len() {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "length_mismatch",
        });
    }
    if v6_matches.len() < 2 {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "insufficient_pairs",
        });
    }
    if v6_matches.len() > MAX_CASES_PER_RUN as usize {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "too_many_pairs",
        });
    }

    let n = v6_matches.len();
    let mut v6_successes = 0_u64;
    let mut c6_successes = 0_u64;
    let mut differences = Vec::with_capacity(n);
    for (v6, c6) in v6_matches.iter().zip(c6_matches.iter()) {
        let v = u64::from(v6.matches_oracle);
        let c = u64::from(c6.matches_oracle);
        v6_successes += v;
        c6_successes += c;
        differences.push(c as f64 - v as f64);
    }

    let n_f = n as f64;
    let v6_accuracy = v6_successes as f64 / n_f;
    let c6_accuracy = c6_successes as f64 / n_f;
    if !v6_accuracy.is_finite() || !c6_accuracy.is_finite() {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "non_finite",
        });
    }

    let cluster_aware = cluster_sum_squares.is_some();
    let sum_squared_cluster_sizes = cluster_sum_squares.unwrap_or(n as u64);
    let (v6_accuracy_ci, c6_accuracy_ci) = if cluster_aware {
        (
            bounded_hoeffding_mean_interval(
                v6_accuracy,
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
            wilson_score_interval(v6_successes, n as u64)?,
            wilson_score_interval(c6_successes, n as u64)?,
        )
    };
    let (paired_difference_mean, paired_difference_ci) = bounded_hoeffding_paired_difference_ci(
        &differences,
        sum_squared_cluster_sizes,
        cluster_aware,
    )?;

    // Consistency: mean of paired differences must equal accuracy gap.
    let expected_gap = c6_accuracy - v6_accuracy;
    if !paired_difference_mean.is_finite() || (paired_difference_mean - expected_gap).abs() > 1e-12
    {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "non_finite",
        });
    }

    Ok(PairedEffectSummary {
        split,
        n_pairs: n as u64,
        v6_accuracy,
        c6_accuracy,
        paired_difference_mean,
        paired_difference_ci,
        v6_accuracy_ci,
        c6_accuracy_ci,
        primary_metric: registry.primary,
        secondary_paired_outcome_difference: SecondaryDiagnosticId::PairedOutcomeDifference,
        uncertainty_contract: PAIRED_UNCERTAINTY_CONTRACT,
        metric_registry_contract: registry.registry_contract,
        experimental_non_final: true,
    })
}

/// Extract revealed match bits from evaluator records for one arm/split.
///
/// Accepts only [`EvalOutcome::Scored`] rows whose arm, split, and metric-registry
/// contract match the expected pins. Failures and contract drift fail closed.
/// Does not accept [`super::tdi24_tasks::ProtectedLabel`] or raw sealed targets.
pub fn revealed_matches_from_records(
    records: &[EvalRecord],
    expected_arm: EvalArm,
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
        if record.arm != expected_arm {
            return Err(EvalError::PairedUncertaintyInvalid {
                reason: "arm_mismatch",
            });
        }
        if record.split != expected_split {
            return Err(EvalError::SplitMismatch {
                expected: expected_split,
                actual: record.split,
            });
        }
        if record.metric_registry_contract != METRIC_REGISTRY_CONTRACT {
            return Err(EvalError::ContractMismatch {
                field: "metric_registry_contract",
            });
        }
        if record.envelope_contract != EVALUATOR_ENVELOPE_CONTRACT {
            return Err(EvalError::ContractMismatch {
                field: "envelope_contract",
            });
        }
        if record.arm_contract != expected_arm.evaluator_contract() {
            return Err(EvalError::ContractMismatch {
                field: "arm_contract",
            });
        }
        if record.budget_contract != READOUT_BUDGET_CONTRACT {
            return Err(EvalError::ContractMismatch {
                field: "budget_contract",
            });
        }
        let expected_vector_contract = match expected_arm {
            EvalArm::V6 => VECTOR6_CONTRACT,
            EvalArm::C6 => CHIRAL_CONTRACT,
        };
        if record.vector_contract != expected_vector_contract {
            return Err(EvalError::ContractMismatch {
                field: "vector_contract",
            });
        }
        if record.label_contract != PROTECTED_LABEL_CONTRACT {
            return Err(EvalError::ContractMismatch {
                field: "label_contract",
            });
        }
        match record.outcome {
            EvalOutcome::Scored { correct, .. } => {
                out.push(RevealedMatchOutcome::from_matches_oracle(correct));
            }
            EvalOutcome::Failure(_) => {
                return Err(EvalError::PairedUncertaintyInvalid {
                    reason: "unscored_outcome",
                });
            }
        }
    }
    Ok(out)
}

/// Return sum_g n_g^2 for the record grouping, bounded by the evaluator cap.
fn sum_squared_group_sizes(records: &[EvalRecord]) -> Result<u64, EvalError> {
    let mut groups: Vec<(TaskFamily, u64, u64)> = Vec::with_capacity(records.len());
    for record in records {
        if let Some((_, _, size)) = groups
            .iter_mut()
            .find(|(family, group_id, _)| *family == record.family && *group_id == record.group_id)
        {
            *size += 1;
        } else {
            groups.push((record.family, record.group_id, 1));
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

/// Summarise paired V6/C6 [`EvalRecord`] slices without re-entering label oracles.
///
/// Records must be equal-length and identity-aligned in order: matching
/// `family`, `case_id`, `group_id`, and `canonical_digest` at each index.
/// Positional zip without identity checks is rejected fail-closed.
pub fn summarize_paired_uncertainty_from_records(
    split: DataSplit,
    v6_records: &[EvalRecord],
    c6_records: &[EvalRecord],
    registry: &MetricRegistry,
) -> Result<PairedEffectSummary, EvalError> {
    if v6_records.len() != c6_records.len() {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "length_mismatch",
        });
    }
    if v6_records.len() > MAX_CASES_PER_RUN as usize {
        return Err(EvalError::PairedUncertaintyInvalid {
            reason: "too_many_pairs",
        });
    }
    for (index, (left, right)) in v6_records.iter().zip(c6_records.iter()).enumerate() {
        if left.family != right.family
            || left.case_id != right.case_id
            || left.group_id != right.group_id
            || left.canonical_digest != right.canonical_digest
        {
            return Err(EvalError::PairedUncertaintyInvalid {
                reason: "pair_identity_mismatch",
            });
        }
        if v6_records[..index].iter().any(|prior| {
            prior.family == left.family
                && prior.case_id == left.case_id
                && prior.group_id == left.group_id
                && prior.canonical_digest == left.canonical_digest
        }) {
            return Err(EvalError::PairedUncertaintyInvalid {
                reason: "duplicate_pair_identity",
            });
        }
    }
    let cluster_sum_squares = sum_squared_group_sizes(v6_records)?;
    let v6 = revealed_matches_from_records(v6_records, EvalArm::V6, split)?;
    let c6 = revealed_matches_from_records(c6_records, EvalArm::C6, split)?;
    summarize_paired_uncertainty_inner(split, &v6, &c6, registry, Some(cluster_sum_squares))
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

/// Fail-closed evaluator errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvalError {
    /// Protected/final or otherwise unknown split identity.
    ProtectedOrFinalSplit,
    /// Case split does not match the opened run.
    SplitMismatch {
        expected: DataSplit,
        actual: DataSplit,
    },
    /// Envelope or budget contract pin drifted.
    ContractMismatch { field: &'static str },
    /// Budget fields are zero or otherwise inadmissible.
    InvalidBudget,
    /// Arm is not admitted by this slice.
    UnsupportedArm,
    /// Matched case budget exhausted.
    CaseBudgetExceeded,
    /// V6 arithmetic rejected the carriers.
    Numerical(Vector6Error),
    /// C6 arithmetic rejected the carriers.
    ChiralNumerical(ChiralError),
    /// Trainable parameter counts or carrier widths are unmatched.
    ParameterCountMismatch {
        left_arm: EvalArm,
        right_arm: EvalArm,
        left_parameters: u64,
        right_parameters: u64,
    },
    /// Initialization seed/kind/stream are unmatched across paired arms.
    InitializationMismatch {
        left_arm: EvalArm,
        right_arm: EvalArm,
        left_seed: u64,
        right_seed: u64,
    },
    /// Optimizer/update budgets are unmatched across paired arms.
    OptimizerUpdateBudgetMismatch {
        left_arm: EvalArm,
        right_arm: EvalArm,
        left_examples: u64,
        right_examples: u64,
        left_updates: u64,
        right_updates: u64,
    },
    /// Metric registry failed closed-set validation.
    MetricRegistryInvalid { reason: &'static str },
    /// Paired uncertainty engine rejected inputs or produced non-finite stats.
    PairedUncertaintyInvalid { reason: &'static str },
}

impl fmt::Display for EvalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProtectedOrFinalSplit => {
                formatter.write_str("tdi-24 evaluator rejects protected/final splits")
            }
            Self::SplitMismatch { expected, actual } => write!(
                formatter,
                "evaluator split mismatch: expected {}, got {}",
                expected.as_str(),
                actual.as_str()
            ),
            Self::ContractMismatch { field } => {
                write!(formatter, "evaluator contract mismatch: {field}")
            }
            Self::InvalidBudget => formatter.write_str("evaluator readout budget is invalid"),
            Self::UnsupportedArm => formatter.write_str("evaluator arm is not supported"),
            Self::CaseBudgetExceeded => formatter.write_str("evaluator case budget exceeded"),
            Self::Numerical(error) => write!(formatter, "v6 numerical failure: {error}"),
            Self::ChiralNumerical(error) => write!(formatter, "c6 numerical failure: {error}"),
            Self::ParameterCountMismatch {
                left_arm,
                right_arm,
                left_parameters,
                right_parameters,
            } => write!(
                formatter,
                "parameter-count mismatch: {}={} vs {}={}",
                left_arm.as_str(),
                left_parameters,
                right_arm.as_str(),
                right_parameters
            ),
            Self::InitializationMismatch {
                left_arm,
                right_arm,
                left_seed,
                right_seed,
            } => write!(
                formatter,
                "initialization mismatch: {} seed={} vs {} seed={}",
                left_arm.as_str(),
                left_seed,
                right_arm.as_str(),
                right_seed
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
            Self::PairedUncertaintyInvalid { reason } => {
                write!(formatter, "paired uncertainty invalid: {reason}")
            }
        }
    }
}

impl std::error::Error for EvalError {}

/// Map handedness oracle to a signed binary label.
pub fn handedness_sign(target: &HandednessTarget) -> Result<i8, EvalError> {
    match target {
        HandednessTarget::Right => Ok(1),
        HandednessTarget::Left => Ok(-1),
    }
}

/// Map reflection-invariant oracle to a signed binary label.
pub fn reflection_invariant_sign(target: &ReflectionInvariantTarget) -> Result<i8, EvalError> {
    match target {
        ReflectionInvariantTarget::ClassA => Ok(1),
        ReflectionInvariantTarget::ClassB => Ok(-1),
    }
}

/// Map direction oracle to a signed binary label.
pub fn direction_sign(target: &DirectionTarget) -> Result<i8, EvalError> {
    match target {
        DirectionTarget::Forward => Ok(1),
        DirectionTarget::Reverse => Ok(-1),
    }
}

/// Map non-chiral control oracle to a signed binary label.
pub fn non_chiral_sign(target: &NonChiralTarget) -> Result<i8, EvalError> {
    match target {
        NonChiralTarget::ClassA => Ok(1),
        NonChiralTarget::ClassB => Ok(-1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::experimental::tdi24_tasks::{
        PROTECTED_LABEL_CONTRACT, non_chiral_control_case, reflection_discriminative_pair,
        reflection_discriminative_pair_in_split, reflection_nuisance_pair, seal_non_chiral_control,
        seal_reflection_discriminative, seal_reflection_nuisance,
    };

    #[test]
    fn c6_evaluator_contract_and_budget_match_v6() {
        assert_eq!(C6_EVALUATOR_CONTRACT, "tdi24-c6-evaluator-v1");
        let v6 = EvaluatorConfig::v6(DataSplit::Development).unwrap();
        let c6 = EvaluatorConfig::c6(DataSplit::Development).unwrap();
        assert_eq!(v6.budget, c6.budget);
        assert_eq!(EvalArm::C6.score_arm(), ScoreArm::C6);
    }

    #[test]
    fn c6_development_and_validation_paths_are_deterministic_and_sealed() {
        for split in [DataSplit::Development, DataSplit::Validation] {
            let case = reflection_discriminative_pair_in_split(7, split)
                .unwrap()
                .right;
            let labeled = seal_reflection_discriminative(&case);
            let mut first = EvaluatorRun::open(EvaluatorConfig::c6(split).unwrap()).unwrap();
            let a = first
                .evaluate_c6_binary(&labeled, handedness_sign)
                .unwrap()
                .clone();
            let mut second = EvaluatorRun::open(EvaluatorConfig::c6(split).unwrap()).unwrap();
            let b = second
                .evaluate_c6_binary(&labeled, handedness_sign)
                .unwrap()
                .clone();
            assert_eq!(a, b);
            assert_eq!(a.arm, EvalArm::C6);
            assert_eq!(a.arm_contract, C6_EVALUATOR_CONTRACT);
            assert_eq!(a.vector_contract, CHIRAL_CONTRACT);
            assert!(matches!(a.outcome, EvalOutcome::Scored { .. }));
        }
    }

    #[test]
    fn v6_evaluator_contracts_and_budget_are_pinned() {
        assert_eq!(EVALUATOR_ENVELOPE_CONTRACT, "tdi24-evaluator-envelope-v1");
        assert_eq!(V6_EVALUATOR_CONTRACT, "tdi24-v6-evaluator-v1");
        assert_eq!(READOUT_BUDGET_CONTRACT, "tdi24-readout-budget-v1");
        let budget = ReadoutBudget::matched_non_trained();
        assert_eq!(budget.max_cases, 64);
        assert_eq!(budget.max_readout_scalars_per_case, 2);
        assert_eq!(budget.updates, 0);
        assert_eq!(EvalArm::V6.score_arm(), ScoreArm::V6);
        assert!(parse_non_final_split("protected").is_err());
        assert!(parse_non_final_split("final").is_err());
        assert_eq!(
            parse_non_final_split("development").unwrap(),
            DataSplit::Development
        );
    }

    #[test]
    fn v6_development_path_is_deterministic_and_label_sealed() {
        let mut run =
            EvaluatorRun::open(EvaluatorConfig::v6(DataSplit::Development).unwrap()).unwrap();
        let labeled =
            seal_reflection_discriminative(&reflection_discriminative_pair(3).unwrap().right);
        let rendered = run_inference_callback(&labeled, |view| format!("{view:?}"));
        assert!(!rendered.contains("Right"));
        assert!(!rendered.contains("Left"));
        assert!(!rendered.contains("target"));

        let first = run
            .evaluate_v6_binary(&labeled, handedness_sign)
            .unwrap()
            .clone();
        let second = {
            let mut again =
                EvaluatorRun::open(EvaluatorConfig::v6(DataSplit::Development).unwrap()).unwrap();
            again
                .evaluate_v6_binary(&labeled, handedness_sign)
                .unwrap()
                .clone()
        };
        assert_eq!(first, second);
        assert_eq!(first.arm, EvalArm::V6);
        assert_eq!(first.split, DataSplit::Development);
        assert_eq!(first.arm_contract, V6_EVALUATOR_CONTRACT);
        assert_eq!(first.vector_contract, VECTOR6_CONTRACT);
        assert_eq!(first.label_contract, PROTECTED_LABEL_CONTRACT);
        assert!(matches!(first.outcome, EvalOutcome::Scored { .. }));
        assert_eq!(handedness_sign(&HandednessTarget::Right).unwrap(), 1);
    }

    #[test]
    fn v6_validation_path_accepts_split_manifest_cases() {
        let mut run =
            EvaluatorRun::open(EvaluatorConfig::v6(DataSplit::Validation).unwrap()).unwrap();
        let labeled = seal_reflection_discriminative(
            &reflection_discriminative_pair_in_split(5, DataSplit::Validation)
                .unwrap()
                .left,
        );
        let record = run.evaluate_v6_binary(&labeled, handedness_sign).unwrap();
        assert_eq!(record.split, DataSplit::Validation);
        assert_eq!(record.family.as_str(), "reflection_discriminative");
    }

    #[test]
    fn v6_rejects_split_mismatch_and_budget_overflow() {
        let mut run =
            EvaluatorRun::open(EvaluatorConfig::v6(DataSplit::Development).unwrap()).unwrap();
        let validation = seal_reflection_discriminative(
            &reflection_discriminative_pair_in_split(1, DataSplit::Validation)
                .unwrap()
                .right,
        );
        assert!(matches!(
            run.evaluate_v6_binary(&validation, handedness_sign),
            Err(EvalError::SplitMismatch { .. })
        ));

        let mut tight = EvaluatorConfig::v6(DataSplit::Development).unwrap();
        tight.budget.max_cases = 1;
        let mut limited = EvaluatorRun::open(tight).unwrap();
        let a = seal_non_chiral_control(&non_chiral_control_case(0).unwrap());
        let b = seal_non_chiral_control(&non_chiral_control_case(1).unwrap());
        assert!(limited.evaluate_v6_binary(&a, non_chiral_sign).is_ok());
        assert_eq!(
            limited.evaluate_v6_binary(&b, non_chiral_sign),
            Err(EvalError::CaseBudgetExceeded)
        );
    }

    #[test]
    fn v6_scores_nuisance_and_non_chiral_families_without_oracle_leakage() {
        let mut run =
            EvaluatorRun::open(EvaluatorConfig::v6(DataSplit::Development).unwrap()).unwrap();
        let nuisance = seal_reflection_nuisance(&reflection_nuisance_pair(2).unwrap().canonical);
        let control = seal_non_chiral_control(&non_chiral_control_case(4).unwrap());
        let n = run
            .evaluate_v6_binary(&nuisance, reflection_invariant_sign)
            .unwrap();
        assert!(matches!(n.outcome, EvalOutcome::Scored { .. }));
        let c = run.evaluate_v6_binary(&control, non_chiral_sign).unwrap();
        assert!(matches!(c.outcome, EvalOutcome::Scored { .. }));
        // Non-chiral even-sector targets are linearly readable by V6 on several seeds.
        let mut hits = 0u32;
        for case_id in 0..8u64 {
            let sealed = seal_non_chiral_control(&non_chiral_control_case(case_id).unwrap());
            let mut probe =
                EvaluatorRun::open(EvaluatorConfig::v6(DataSplit::Development).unwrap()).unwrap();
            let record = probe.evaluate_v6_binary(&sealed, non_chiral_sign).unwrap();
            if matches!(record.outcome, EvalOutcome::Scored { correct: true, .. }) {
                hits += 1;
            }
        }
        assert!(
            hits >= 1,
            "V6 should recover at least one non-chiral even-sector label"
        );
        let leaked = run_inference_callback(&nuisance, |view| format!("{view:?}"));
        assert!(!leaked.contains("ClassA"));
        assert!(!leaked.contains("ClassB"));
    }

    #[test]
    fn parameter_count_matcher_accepts_matched_reference_capacities() {
        assert_eq!(
            PARAMETER_COUNT_MATCHER_CONTRACT,
            "tdi24-parameter-count-matcher-v1"
        );
        let v6 = TrainableCapacity::reference_v6();
        let c6 = TrainableCapacity::reference_c6();
        assert_eq!(v6.trainable_parameters, 0);
        assert_eq!(c6.trainable_parameters, 0);
        assert_eq!(v6.carrier_width, VECTOR6_WIDTH as u64);
        assert_eq!(c6.carrier_width, CHIRAL_WIDTH as u64);
        let matched = match_parameter_counts(v6, c6).unwrap();
        assert_eq!(matched.trainable_parameters, 0);
        assert_eq!(matched.carrier_width, VECTOR6_WIDTH as u64);
        assert_eq!(matched.left_arm, EvalArm::V6);
        assert_eq!(matched.right_arm, EvalArm::C6);
        assert_eq!(matched.matcher_contract, PARAMETER_COUNT_MATCHER_CONTRACT);
        // Matched capacity is admissible on Development and Validation only.
        for split in [DataSplit::Development, DataSplit::Validation] {
            assert!(validate_non_final_split(split).is_ok());
            let _ = EvaluatorConfig::v6(split).unwrap();
            let _ = EvaluatorConfig::c6(split).unwrap();
        }
        assert!(parse_non_final_split("protected").is_err());
        assert!(parse_non_final_split("final").is_err());
    }

    #[test]
    fn parameter_count_matcher_rejects_unmatched_trainable_capacity() {
        let v6 = TrainableCapacity::reference_v6();
        let mut inflated = TrainableCapacity::reference_c6();
        inflated.trainable_parameters = 16;
        assert_eq!(
            match_parameter_counts(v6, inflated),
            Err(EvalError::ParameterCountMismatch {
                left_arm: EvalArm::V6,
                right_arm: EvalArm::C6,
                left_parameters: 0,
                right_parameters: 16,
            })
        );

        let mut wide = TrainableCapacity::reference_c6();
        wide.carrier_width = 8;
        assert!(matches!(
            match_parameter_counts(v6, wide),
            Err(EvalError::ParameterCountMismatch { .. })
        ));

        let mut drifted = TrainableCapacity::reference_v6();
        drifted.matcher_contract = "not-a-matcher";
        assert_eq!(
            match_parameter_counts(drifted, TrainableCapacity::reference_c6()),
            Err(EvalError::ContractMismatch {
                field: "parameter_count_matcher_contract",
            })
        );
    }

    #[test]
    fn initialization_matcher_accepts_paired_reference_policies() {
        assert_eq!(
            INITIALIZATION_MATCHER_CONTRACT,
            "tdi24-initialization-matcher-v1"
        );
        let seed = 42_u64;
        let v6 = InitializationPolicy::reference_v6(seed);
        let c6 = InitializationPolicy::reference_c6(seed);
        assert_eq!(v6.kind, InitializationKind::ReferenceNone);
        assert_eq!(c6.kind, InitializationKind::ReferenceNone);
        assert_eq!(v6.seed, seed);
        assert_eq!(c6.seed, seed);
        assert_eq!(v6.stream_id, 0);
        assert_eq!(InitializationKind::ReferenceNone.as_str(), "reference_none");
        assert_eq!(
            InitializationKind::DeterministicPaired.as_str(),
            "deterministic_paired"
        );
        let matched = match_initialization(v6, c6).unwrap();
        assert_eq!(matched.kind, InitializationKind::ReferenceNone);
        assert_eq!(matched.seed, seed);
        assert_eq!(matched.stream_id, 0);
        assert_eq!(matched.left_arm, EvalArm::V6);
        assert_eq!(matched.right_arm, EvalArm::C6);
        assert_eq!(matched.matcher_contract, INITIALIZATION_MATCHER_CONTRACT);
        // Paired init remains admissible only on Development/Validation.
        for split in [DataSplit::Development, DataSplit::Validation] {
            assert!(validate_non_final_split(split).is_ok());
        }
        assert!(parse_non_final_split("protected").is_err());
        assert!(parse_non_final_split("final").is_err());
    }

    #[test]
    fn initialization_matcher_rejects_unpaired_seed_or_kind() {
        let v6 = InitializationPolicy::reference_v6(7);
        let other_seed = InitializationPolicy::reference_c6(8);
        assert_eq!(
            match_initialization(v6, other_seed),
            Err(EvalError::InitializationMismatch {
                left_arm: EvalArm::V6,
                right_arm: EvalArm::C6,
                left_seed: 7,
                right_seed: 8,
            })
        );

        let mut other_kind = InitializationPolicy::reference_c6(7);
        other_kind.kind = InitializationKind::DeterministicPaired;
        assert!(matches!(
            match_initialization(v6, other_kind),
            Err(EvalError::InitializationMismatch { .. })
        ));

        let mut other_stream = InitializationPolicy::reference_c6(7);
        other_stream.stream_id = 1;
        assert!(matches!(
            match_initialization(v6, other_stream),
            Err(EvalError::InitializationMismatch { .. })
        ));

        let mut drifted = InitializationPolicy::reference_v6(7);
        drifted.matcher_contract = "not-an-init-matcher";
        assert_eq!(
            match_initialization(drifted, InitializationPolicy::reference_c6(7)),
            Err(EvalError::ContractMismatch {
                field: "initialization_matcher_contract",
            })
        );
    }

    #[test]
    fn optimizer_update_budget_matcher_accepts_paired_reference_budgets() {
        assert_eq!(
            OPTIMIZER_UPDATE_BUDGET_CONTRACT,
            "tdi24-optimizer-update-budget-v1"
        );
        let examples = 16_u64;
        let ordering_seed = 99_u64;
        let v6 = OptimizerUpdateBudget::reference_v6(examples, ordering_seed);
        let c6 = OptimizerUpdateBudget::reference_c6(examples, ordering_seed);
        assert_eq!(v6.updates, NON_TRAINED_UPDATE_BUDGET);
        assert_eq!(c6.updates, NON_TRAINED_UPDATE_BUDGET);
        assert_eq!(v6.stopping, StoppingRule::ExhaustExamples);
        assert_eq!(StoppingRule::ExhaustExamples.as_str(), "exhaust_examples");
        assert_eq!(StoppingRule::FixedUpdates.as_str(), "fixed_updates");
        let matched = match_optimizer_update_budgets(v6, c6).unwrap();
        assert_eq!(matched.examples, examples);
        assert_eq!(matched.updates, 0);
        assert_eq!(matched.ordering_seed, ordering_seed);
        assert_eq!(matched.stopping, StoppingRule::ExhaustExamples);
        assert_eq!(matched.left_arm, EvalArm::V6);
        assert_eq!(matched.right_arm, EvalArm::C6);
        assert_eq!(matched.matcher_contract, OPTIMIZER_UPDATE_BUDGET_CONTRACT);
        for split in [DataSplit::Development, DataSplit::Validation] {
            assert!(validate_non_final_split(split).is_ok());
        }
        assert!(parse_non_final_split("protected").is_err());
        assert!(parse_non_final_split("final").is_err());
    }

    #[test]
    fn optimizer_update_budget_matcher_rejects_unpaired_budgets() {
        let v6 = OptimizerUpdateBudget::reference_v6(8, 1);
        let other_examples = OptimizerUpdateBudget::reference_c6(9, 1);
        assert_eq!(
            match_optimizer_update_budgets(v6, other_examples),
            Err(EvalError::OptimizerUpdateBudgetMismatch {
                left_arm: EvalArm::V6,
                right_arm: EvalArm::C6,
                left_examples: 8,
                right_examples: 9,
                left_updates: 0,
                right_updates: 0,
            })
        );

        let mut other_updates = OptimizerUpdateBudget::reference_c6(8, 1);
        other_updates.updates = 4;
        other_updates.stopping = StoppingRule::FixedUpdates;
        assert!(matches!(
            match_optimizer_update_budgets(v6, other_updates),
            Err(EvalError::OptimizerUpdateBudgetMismatch { .. })
        ));

        let other_order = OptimizerUpdateBudget::reference_c6(8, 2);
        assert!(matches!(
            match_optimizer_update_budgets(v6, other_order),
            Err(EvalError::OptimizerUpdateBudgetMismatch { .. })
        ));

        assert_eq!(
            match_optimizer_update_budgets(
                OptimizerUpdateBudget::reference_v6(0, 1),
                OptimizerUpdateBudget::reference_c6(0, 1)
            ),
            Err(EvalError::InvalidBudget)
        );

        let mut drifted = OptimizerUpdateBudget::reference_v6(8, 1);
        drifted.matcher_contract = "not-an-opt-budget";
        assert_eq!(
            match_optimizer_update_budgets(drifted, OptimizerUpdateBudget::reference_c6(8, 1)),
            Err(EvalError::ContractMismatch {
                field: "optimizer_update_budget_contract",
            })
        );
    }

    #[test]
    fn metric_registry_pins_primary_and_ordered_secondaries() {
        assert_eq!(METRIC_REGISTRY_CONTRACT, "tdi24-metric-registry-v1");
        assert_eq!(PRIMARY_METRIC_PAIRED_TASK_ACCURACY, "paired_task_accuracy");
        assert_eq!(
            PINNED_SECONDARY_DIAGNOSTICS.len(),
            MAX_SECONDARY_DIAGNOSTICS
        );
        assert_eq!(
            SecondaryDiagnosticId::admitted_set(),
            PINNED_SECONDARY_DIAGNOSTICS
        );

        let pinned = MetricRegistry::pinned();
        assert_eq!(pinned.primary, PrimaryMetricId::PairedTaskAccuracy);
        assert_eq!(pinned.primary.as_str(), PRIMARY_METRIC_PAIRED_TASK_ACCURACY);
        assert_eq!(pinned.secondaries, PINNED_SECONDARY_DIAGNOSTICS);
        assert_eq!(pinned.registry_contract, METRIC_REGISTRY_CONTRACT);
        assert!(pinned.experimental_non_final);
        validate_metric_registry(&pinned).unwrap();

        let frozen = freeze_metric_registry(
            Some(PrimaryMetricId::PairedTaskAccuracy),
            PINNED_SECONDARY_DIAGNOSTICS,
        )
        .unwrap();
        assert_eq!(frozen, pinned);

        assert_eq!(
            pinned.secondaries[0].as_str(),
            SECONDARY_PAIRED_OUTCOME_DIFFERENCE
        );
        assert_eq!(
            pinned.secondaries[1].as_str(),
            SECONDARY_MIRROR_SWAP_IDENTITY_ERROR
        );
        assert_eq!(
            pinned.secondaries[2].as_str(),
            SECONDARY_PARITY_EQUIVARIANCE_INVARIANCE_ERROR
        );
        assert_eq!(
            pinned.secondaries[3].as_str(),
            SECONDARY_CALIBRATION_CONFIDENCE_ERROR
        );
        assert_eq!(pinned.secondaries[4].as_str(), SECONDARY_GRADIENT_STABILITY);
        assert_eq!(
            pinned.secondaries[5].as_str(),
            SECONDARY_OP_COUNT_MEMORY_LATENCY_THROUGHPUT
        );

        for split in [DataSplit::Development, DataSplit::Validation] {
            assert!(pinned.admit_split(split).is_ok());
        }
        // Protected/final labels are rejected by the non-final split gate.
        assert!(parse_non_final_split("protected").is_err());
        assert!(parse_non_final_split("final").is_err());
        assert_eq!(
            parse_primary_metric_id(PRIMARY_METRIC_PAIRED_TASK_ACCURACY).unwrap(),
            PrimaryMetricId::PairedTaskAccuracy
        );
        assert_eq!(
            parse_secondary_diagnostic_id(SECONDARY_PAIRED_OUTCOME_DIFFERENCE).unwrap(),
            SecondaryDiagnosticId::PairedOutcomeDifference
        );
    }

    #[test]
    fn metric_registry_rejects_drift_duplicates_and_invention() {
        assert_eq!(
            freeze_metric_registry(None, PINNED_SECONDARY_DIAGNOSTICS),
            Err(EvalError::MetricRegistryInvalid {
                reason: "empty_primary",
            })
        );
        assert_eq!(
            parse_primary_metric_id(""),
            Err(EvalError::MetricRegistryInvalid {
                reason: "empty_primary",
            })
        );
        assert_eq!(
            parse_primary_metric_id("invented_primary"),
            Err(EvalError::MetricRegistryInvalid {
                reason: "unknown_primary",
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
            SecondaryDiagnosticId::MirrorSwapIdentityError,
            SecondaryDiagnosticId::PairedOutcomeDifference,
        ];
        assert_eq!(
            freeze_metric_registry(
                Some(PrimaryMetricId::PairedTaskAccuracy),
                DUPLICATE_SECONDARIES
            ),
            Err(EvalError::MetricRegistryInvalid {
                reason: "duplicate_secondary",
            })
        );

        const OVERSIZED_SECONDARIES: &[SecondaryDiagnosticId] = &[
            SecondaryDiagnosticId::PairedOutcomeDifference,
            SecondaryDiagnosticId::MirrorSwapIdentityError,
            SecondaryDiagnosticId::ParityEquivarianceInvarianceError,
            SecondaryDiagnosticId::CalibrationConfidenceError,
            SecondaryDiagnosticId::GradientStability,
            SecondaryDiagnosticId::OpCountMemoryLatencyThroughput,
            SecondaryDiagnosticId::PairedOutcomeDifference,
        ];
        assert_eq!(
            freeze_metric_registry(
                Some(PrimaryMetricId::PairedTaskAccuracy),
                OVERSIZED_SECONDARIES
            ),
            Err(EvalError::MetricRegistryInvalid {
                reason: "too_many_secondaries",
            })
        );

        const TRUNCATED_SECONDARIES: &[SecondaryDiagnosticId] = &[
            SecondaryDiagnosticId::PairedOutcomeDifference,
            SecondaryDiagnosticId::MirrorSwapIdentityError,
            SecondaryDiagnosticId::ParityEquivarianceInvarianceError,
            SecondaryDiagnosticId::CalibrationConfidenceError,
            SecondaryDiagnosticId::GradientStability,
        ];
        assert_eq!(
            freeze_metric_registry(
                Some(PrimaryMetricId::PairedTaskAccuracy),
                TRUNCATED_SECONDARIES
            ),
            Err(EvalError::MetricRegistryInvalid {
                reason: "secondary_sequence_mismatch",
            })
        );
        assert_eq!(
            freeze_metric_registry(Some(PrimaryMetricId::PairedTaskAccuracy), &[]),
            Err(EvalError::MetricRegistryInvalid {
                reason: "secondary_sequence_mismatch",
            })
        );

        const REORDERED_SECONDARIES: &[SecondaryDiagnosticId] = &[
            SecondaryDiagnosticId::MirrorSwapIdentityError,
            SecondaryDiagnosticId::PairedOutcomeDifference,
            SecondaryDiagnosticId::ParityEquivarianceInvarianceError,
            SecondaryDiagnosticId::CalibrationConfidenceError,
            SecondaryDiagnosticId::GradientStability,
            SecondaryDiagnosticId::OpCountMemoryLatencyThroughput,
        ];
        assert_eq!(
            freeze_metric_registry(
                Some(PrimaryMetricId::PairedTaskAccuracy),
                REORDERED_SECONDARIES
            ),
            Err(EvalError::MetricRegistryInvalid {
                reason: "secondary_sequence_mismatch",
            })
        );

        let malformed = MetricRegistry {
            secondaries: &[],
            ..MetricRegistry::pinned()
        };
        assert_eq!(
            malformed.admit_split(DataSplit::Development),
            Err(EvalError::MetricRegistryInvalid {
                reason: "secondary_sequence_mismatch",
            })
        );
        let mut config = EvaluatorConfig::v6(DataSplit::Development).unwrap();
        config.metric_registry = malformed;
        assert_eq!(
            EvaluatorRun::open(config),
            Err(EvalError::MetricRegistryInvalid {
                reason: "secondary_sequence_mismatch",
            })
        );

        let mut drifted = MetricRegistry::pinned();
        drifted.registry_contract = "not-a-metric-registry";
        assert_eq!(
            validate_metric_registry(&drifted),
            Err(EvalError::ContractMismatch {
                field: "metric_registry_contract",
            })
        );

        let mut finalized = MetricRegistry::pinned();
        finalized.experimental_non_final = false;
        assert_eq!(
            validate_metric_registry(&finalized),
            Err(EvalError::MetricRegistryInvalid {
                reason: "experimental_non_final_required",
            })
        );
        assert_eq!(
            finalized.admit_split(DataSplit::Development),
            Err(EvalError::MetricRegistryInvalid {
                reason: "experimental_non_final_required",
            })
        );
    }

    fn matches(bits: &[bool]) -> Vec<RevealedMatchOutcome> {
        bits.iter()
            .copied()
            .map(RevealedMatchOutcome::from_matches_oracle)
            .collect()
    }

    #[test]
    fn paired_uncertainty_happy_path_on_synthetic_match_vectors() {
        assert_eq!(PAIRED_UNCERTAINTY_CONTRACT, "tdi24-paired-uncertainty-v1");
        assert_eq!(UncertaintyMethod::WilsonScore.as_str(), "wilson_score");
        assert_eq!(
            UncertaintyMethod::BoundedHoeffdingPairedDifference.as_str(),
            "bounded_hoeffding_paired_difference"
        );

        // V6: T T F F  -> accuracy 0.5
        // C6: T T T F  -> accuracy 0.75
        // diffs: 0,0,1,0 -> mean 0.25
        let v6 = matches(&[true, true, false, false]);
        let c6 = matches(&[true, true, true, false]);
        let registry = MetricRegistry::pinned();
        let summary =
            summarize_paired_uncertainty(DataSplit::Development, &v6, &c6, &registry).unwrap();

        assert_eq!(summary.n_pairs, 4);
        assert!((summary.v6_accuracy - 0.5).abs() < 1e-12);
        assert!((summary.c6_accuracy - 0.75).abs() < 1e-12);
        assert!((summary.paired_difference_mean - 0.25).abs() < 1e-12);
        assert_eq!(summary.primary_metric, PrimaryMetricId::PairedTaskAccuracy);
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
            summary.v6_accuracy_ci.method,
            UncertaintyMethod::WilsonScore
        );
        assert_eq!(
            summary.c6_accuracy_ci.method,
            UncertaintyMethod::WilsonScore
        );
        assert!(summary.paired_difference_ci.lower <= summary.paired_difference_mean);
        assert!(summary.paired_difference_ci.upper >= summary.paired_difference_mean);
        assert!(summary.paired_difference_ci.lower < summary.paired_difference_ci.upper);
        assert!(summary.v6_accuracy_ci.lower <= summary.v6_accuracy);
        assert!(summary.v6_accuracy_ci.upper >= summary.v6_accuracy);

        let validation =
            summarize_paired_uncertainty(DataSplit::Validation, &v6, &c6, &registry).unwrap();
        assert_eq!(validation.split, DataSplit::Validation);
    }

    #[test]
    fn paired_uncertainty_boundary_intervals_remain_non_degenerate() {
        let registry = MetricRegistry::pinned();
        let ties = matches(&[true; MAX_CASES_PER_RUN as usize]);
        let all_ties =
            summarize_paired_uncertainty(DataSplit::Development, &ties, &ties, &registry).unwrap();
        assert_eq!(all_ties.paired_difference_mean, 0.0);
        assert!(all_ties.paired_difference_ci.lower < 0.0);
        assert!(all_ties.paired_difference_ci.upper > 0.0);

        let v6 = matches(&[false; MAX_CASES_PER_RUN as usize]);
        let c6 = matches(&[true; MAX_CASES_PER_RUN as usize]);
        let all_c6_wins =
            summarize_paired_uncertainty(DataSplit::Development, &v6, &c6, &registry).unwrap();
        assert_eq!(all_c6_wins.paired_difference_mean, 1.0);
        assert!(all_c6_wins.paired_difference_ci.lower < 1.0);
        assert_eq!(all_c6_wins.paired_difference_ci.upper, 1.0);
    }

    #[test]
    fn paired_uncertainty_rejects_empty_mismatch_nan_and_registry_drift() {
        let registry = MetricRegistry::pinned();
        let v6 = matches(&[true, false]);
        let c6 = matches(&[true, true]);

        assert_eq!(
            summarize_paired_uncertainty(DataSplit::Development, &[], &c6, &registry),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "empty_pairs",
            })
        );
        assert_eq!(
            summarize_paired_uncertainty(DataSplit::Development, &v6, &[], &registry),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "empty_pairs",
            })
        );
        assert_eq!(
            summarize_paired_uncertainty(DataSplit::Development, &v6, &matches(&[true]), &registry),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "length_mismatch",
            })
        );
        assert_eq!(
            summarize_paired_uncertainty(DataSplit::Development, &v6, &c6, &registry).and_then(
                |_| {
                    // Direct non-finite gate on ConfidenceInterval::checked.
                    ConfidenceInterval::checked(
                        f64::NAN,
                        1.0,
                        PAIRED_UNCERTAINTY_LEVEL,
                        UncertaintyMethod::WilsonScore,
                    )
                }
            ),
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
            summarize_paired_uncertainty(DataSplit::Development, &v6, &c6, &drifted),
            Err(EvalError::ContractMismatch {
                field: "metric_registry_contract",
            })
        );

        // Same contract pin but empty secondaries drifts away from MetricRegistry::pinned().
        let malformed = MetricRegistry {
            secondaries: &[],
            ..MetricRegistry::pinned()
        };
        assert_eq!(
            summarize_paired_uncertainty(DataSplit::Development, &v6, &c6, &malformed),
            Err(EvalError::MetricRegistryInvalid {
                reason: "secondary_sequence_mismatch",
            })
        );

        assert_eq!(
            summarize_paired_uncertainty(
                DataSplit::Development,
                &matches(&[true]),
                &matches(&[false]),
                &registry,
            ),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "insufficient_pairs",
            })
        );

        let oversized =
            vec![RevealedMatchOutcome::from_matches_oracle(true); MAX_CASES_PER_RUN as usize + 1];
        assert_eq!(
            summarize_paired_uncertainty(DataSplit::Development, &oversized, &oversized, &registry,),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "too_many_pairs",
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
    }

    #[test]
    fn paired_uncertainty_from_records_accepts_scored_rejects_failures() {
        let registry = MetricRegistry::pinned();
        let v6_records = [
            EvalRecord {
                arm: EvalArm::V6,
                split: DataSplit::Development,
                family: TaskFamily::ReflectionDiscriminative,
                case_id: 1,
                group_id: 1,
                outcome: EvalOutcome::Scored {
                    score: 1.0,
                    correct: true,
                },
                canonical_digest: "pair-a".into(),
                envelope_contract: EVALUATOR_ENVELOPE_CONTRACT,
                arm_contract: V6_EVALUATOR_CONTRACT,
                budget_contract: READOUT_BUDGET_CONTRACT,
                metric_registry_contract: METRIC_REGISTRY_CONTRACT,
                vector_contract: VECTOR6_CONTRACT,
                label_contract: PROTECTED_LABEL_CONTRACT,
            },
            EvalRecord {
                arm: EvalArm::V6,
                split: DataSplit::Development,
                family: TaskFamily::ReflectionDiscriminative,
                case_id: 2,
                group_id: 1,
                outcome: EvalOutcome::Scored {
                    score: -1.0,
                    correct: false,
                },
                canonical_digest: "pair-b".into(),
                envelope_contract: EVALUATOR_ENVELOPE_CONTRACT,
                arm_contract: V6_EVALUATOR_CONTRACT,
                budget_contract: READOUT_BUDGET_CONTRACT,
                metric_registry_contract: METRIC_REGISTRY_CONTRACT,
                vector_contract: VECTOR6_CONTRACT,
                label_contract: PROTECTED_LABEL_CONTRACT,
            },
        ];
        let c6_records = [
            EvalRecord {
                arm: EvalArm::C6,
                split: DataSplit::Development,
                family: TaskFamily::ReflectionDiscriminative,
                case_id: 1,
                group_id: 1,
                outcome: EvalOutcome::Scored {
                    score: 1.0,
                    correct: true,
                },
                canonical_digest: "pair-a".into(),
                envelope_contract: EVALUATOR_ENVELOPE_CONTRACT,
                arm_contract: C6_EVALUATOR_CONTRACT,
                budget_contract: READOUT_BUDGET_CONTRACT,
                metric_registry_contract: METRIC_REGISTRY_CONTRACT,
                vector_contract: CHIRAL_CONTRACT,
                label_contract: PROTECTED_LABEL_CONTRACT,
            },
            EvalRecord {
                arm: EvalArm::C6,
                split: DataSplit::Development,
                family: TaskFamily::ReflectionDiscriminative,
                case_id: 2,
                group_id: 1,
                outcome: EvalOutcome::Scored {
                    score: 1.0,
                    correct: true,
                },
                canonical_digest: "pair-b".into(),
                envelope_contract: EVALUATOR_ENVELOPE_CONTRACT,
                arm_contract: C6_EVALUATOR_CONTRACT,
                budget_contract: READOUT_BUDGET_CONTRACT,
                metric_registry_contract: METRIC_REGISTRY_CONTRACT,
                vector_contract: CHIRAL_CONTRACT,
                label_contract: PROTECTED_LABEL_CONTRACT,
            },
        ];

        let summary = summarize_paired_uncertainty_from_records(
            DataSplit::Development,
            &v6_records,
            &c6_records,
            &registry,
        )
        .unwrap();
        assert_eq!(summary.n_pairs, 2);
        assert!((summary.v6_accuracy - 0.5).abs() < 1e-12);
        assert!((summary.c6_accuracy - 1.0).abs() < 1e-12);
        assert!((summary.paired_difference_mean - 0.5).abs() < 1e-12);
        assert_eq!(sum_squared_group_sizes(&v6_records).unwrap(), 4);
        let mut same_numeric_group_different_family = v6_records.clone();
        same_numeric_group_different_family[1].family = TaskFamily::ReflectionNuisance;
        assert_eq!(
            sum_squared_group_sizes(&same_numeric_group_different_family).unwrap(),
            2
        );
        assert_eq!(
            summary.v6_accuracy_ci.method,
            UncertaintyMethod::ClusterHoeffdingBernoulliMean
        );
        assert_eq!(
            summary.c6_accuracy_ci.method,
            UncertaintyMethod::ClusterHoeffdingBernoulliMean
        );
        assert_eq!(
            summary.paired_difference_ci.method,
            UncertaintyMethod::ClusterHoeffdingPairedDifference
        );
        assert!(summary.paired_difference_ci.lower < summary.paired_difference_ci.upper);

        // Typed surface admits only RevealedMatchOutcome / EvalRecord scores —
        // there is no ProtectedLabel constructor on the uncertainty API.
        let _revealed = RevealedMatchOutcome::from_matches_oracle(true);
        assert!(_revealed.matches_oracle);

        let mut failed = c6_records.clone();
        failed[1].outcome = EvalOutcome::Failure(EvalFailure::Task);
        assert_eq!(
            summarize_paired_uncertainty_from_records(
                DataSplit::Development,
                &v6_records,
                &failed,
                &registry,
            ),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "unscored_outcome",
            })
        );

        let mut wrong_arm = v6_records.clone();
        wrong_arm[0].arm = EvalArm::C6;
        assert_eq!(
            revealed_matches_from_records(&wrong_arm, EvalArm::V6, DataSplit::Development),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "arm_mismatch",
            })
        );

        let oversized_records = vec![v6_records[0].clone(); MAX_CASES_PER_RUN as usize + 1];
        assert_eq!(
            revealed_matches_from_records(&oversized_records, EvalArm::V6, DataSplit::Development,),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "too_many_pairs",
            })
        );
        let oversized_c6_records = vec![c6_records[0].clone(); MAX_CASES_PER_RUN as usize + 1];
        assert_eq!(
            summarize_paired_uncertainty_from_records(
                DataSplit::Development,
                &oversized_records,
                &oversized_c6_records,
                &registry,
            ),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "too_many_pairs",
            })
        );

        let mut stale_arm_contract = v6_records.clone();
        stale_arm_contract[0].arm_contract = "stale-v6-evaluator";
        assert_eq!(
            revealed_matches_from_records(&stale_arm_contract, EvalArm::V6, DataSplit::Development,),
            Err(EvalError::ContractMismatch {
                field: "arm_contract",
            })
        );

        let mut stale_budget_contract = v6_records.clone();
        stale_budget_contract[0].budget_contract = "stale-readout-budget";
        assert_eq!(
            revealed_matches_from_records(
                &stale_budget_contract,
                EvalArm::V6,
                DataSplit::Development,
            ),
            Err(EvalError::ContractMismatch {
                field: "budget_contract",
            })
        );

        let mut stale_vector_contract = c6_records.clone();
        stale_vector_contract[0].vector_contract = VECTOR6_CONTRACT;
        assert_eq!(
            revealed_matches_from_records(
                &stale_vector_contract,
                EvalArm::C6,
                DataSplit::Development,
            ),
            Err(EvalError::ContractMismatch {
                field: "vector_contract",
            })
        );

        let mut stale_label_contract = v6_records.clone();
        stale_label_contract[0].label_contract = "stale-label-contract";
        assert_eq!(
            revealed_matches_from_records(
                &stale_label_contract,
                EvalArm::V6,
                DataSplit::Development,
            ),
            Err(EvalError::ContractMismatch {
                field: "label_contract",
            })
        );

        let mut mismatched = c6_records.clone();
        mismatched[0].case_id = 99;
        assert_eq!(
            summarize_paired_uncertainty_from_records(
                DataSplit::Development,
                &v6_records,
                &mismatched,
                &registry,
            ),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "pair_identity_mismatch",
            })
        );

        let mut duplicate_v6 = v6_records.clone();
        duplicate_v6[1].family = duplicate_v6[0].family;
        duplicate_v6[1].case_id = duplicate_v6[0].case_id;
        duplicate_v6[1].group_id = duplicate_v6[0].group_id;
        duplicate_v6[1].canonical_digest = duplicate_v6[0].canonical_digest.clone();
        let mut duplicate_c6 = c6_records.clone();
        duplicate_c6[1].family = duplicate_c6[0].family;
        duplicate_c6[1].case_id = duplicate_c6[0].case_id;
        duplicate_c6[1].group_id = duplicate_c6[0].group_id;
        duplicate_c6[1].canonical_digest = duplicate_c6[0].canonical_digest.clone();
        assert_eq!(
            summarize_paired_uncertainty_from_records(
                DataSplit::Development,
                &duplicate_v6,
                &duplicate_c6,
                &registry,
            ),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "duplicate_pair_identity",
            })
        );
    }
}
