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
//! steps and stopping rule across paired arms. No training, primary-metric
//! freeze, confirmatory execution, or scientific claim is authorised here.

use core::fmt;

use super::tdi24_accounting::ScoreArm;
use super::tdi24_chiral::{
    CHIRAL_CONTRACT, CHIRAL_WIDTH, Chiral6, ChiralError, ChiralScoreWeights, chiral_score,
};
use super::tdi24_tasks::{
    DataSplit, DirectionTarget, HandednessTarget, InferenceView, LabeledCase, NonChiralTarget,
    ReflectionInvariantTarget, TaskFamily, canonicalize_inference_view, run_inference_callback,
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
        validate_non_final_split(config.split)?;
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
}
