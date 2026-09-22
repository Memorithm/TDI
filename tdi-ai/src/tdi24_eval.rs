//! TDI-24 Phase-C matched evaluation machinery.
//!
//! Slice 21 lands the shared evaluator envelope and the V6 arm: a deterministic
//! non-final Development/Validation path that scores through the matched vector
//! reference, never observes sealed targets inside the inference callback, and
//! refuses protected/final population labels. No training, primary-metric freeze,
//! confirmatory execution, or scientific claim is authorised here.

use core::fmt;

use super::tdi24_accounting::ScoreArm;
use super::tdi24_chiral::{
    CHIRAL_CONTRACT, Chiral6, ChiralError, ChiralScoreWeights, chiral_score,
};
use super::tdi24_tasks::{
    DataSplit, DirectionTarget, HandednessTarget, InferenceView, LabeledCase, NonChiralTarget,
    ReflectionInvariantTarget, TaskFamily, canonicalize_inference_view, run_inference_callback,
};
use super::tdi24_vector::{VECTOR6_CONTRACT, Vector6, Vector6Error, vector6_score};

/// Shared evaluator envelope consumed by V6 now and by later matched arms.
pub const EVALUATOR_ENVELOPE_CONTRACT: &str = "tdi24-evaluator-envelope-v1";

/// Versioned V6 evaluator contract.
pub const V6_EVALUATOR_CONTRACT: &str = "tdi24-v6-evaluator-v1";

/// Versioned C6 evaluator contract.
pub const C6_EVALUATOR_CONTRACT: &str = "tdi24-c6-evaluator-v1";

/// Matched readout-budget contract shared across Phase-C evaluator arms.
pub const READOUT_BUDGET_CONTRACT: &str = "tdi24-readout-budget-v1";

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
}
