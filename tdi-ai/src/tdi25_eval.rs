//! TDI-25 Phase-C matched evaluation machinery.
//!
//! Slice 21 introduces the shared bounded, non-final evaluator envelope and
//! the T6 arm. It consumes the TDI-22 factorized pairing through the existing
//! TDI-25 adapter, keeps task oracles outside inference callbacks, and rejects
//! split or contract drift. It does not train, access protected/final data, or
//! authorize a scientific claim.

use core::fmt;

use super::tdi22_torsor::TORSOR_CONTRACT;
use super::tdi25_tasks::{
    DataSplit, LabeledCase, MIXED_GEOMETRY_TASK_CONTRACT, MixedGeometryInput, MixedGeometryOracle,
    PROTECTED_LABEL_CONTRACT, TORSOR_TRANSPORT_TASK_CONTRACT, TorsorTransportInput,
    TorsorTransportOracle, canonicalize_mixed_geometry_input, canonicalize_torsor_transport_input,
    run_inference_callback,
};
use super::tdi25_torsor_chiral::{
    PINNED_SOURCE_CONTRACTS, TaskFamily, Tdi25Error, torsor_arm_score, validate_source_contracts,
};

/// Shared evaluator envelope for all later TDI-25 arms.
pub const EVALUATOR_ENVELOPE_CONTRACT: &str = "tdi25-evaluator-envelope-v1";

/// Versioned T6 evaluator contract.
pub const T6_EVALUATOR_CONTRACT: &str = "tdi25-t6-evaluator-v1";

/// Matched readout-budget contract shared by Phase-C evaluator arms.
pub const READOUT_BUDGET_CONTRACT: &str = "tdi25-readout-budget-v1";

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

/// Immutable configuration for one T6 run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EvaluatorConfig {
    pub split: DataSplit,
    pub budget: ReadoutBudget,
    pub envelope_contract: &'static str,
    pub arm_contract: &'static str,
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
            || config.budget.max_readout_scalars_per_case < 2
            || config.budget.updates != NON_TRAINED_UPDATE_BUDGET
        {
            return Err(EvalError::InvalidBudget);
        }
        let sources = validate_source_contracts().map_err(EvalError::Bridge)?;
        if sources.torsor != TORSOR_CONTRACT
            || sources.torsor != PINNED_SOURCE_CONTRACTS.torsor
        {
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
        if case.inference_input().generator_contract != TORSOR_TRANSPORT_TASK_CONTRACT
            || case.label_contract() != PROTECTED_LABEL_CONTRACT
        {
            return Err(EvalError::ContractMismatch("torsor_transport_case"));
        }
        let score =
            run_inference_callback(case, score_torsor_transport).map_err(EvalError::Bridge)?;
        let oracle = case.protected_label().reveal_for_evaluation();
        let record = T6EvalRecord {
            split: case.inference_input().split,
            family: case.inference_input().task_family,
            case_id: case.inference_input().case_id,
            outcome: T6Outcome {
                score,
                matches_oracle: approximately_equal(score, oracle.expected_score),
            },
            canonical_digest: canonicalize_torsor_transport_input(case.inference_input()).digest,
            envelope_contract: EVALUATOR_ENVELOPE_CONTRACT,
            arm_contract: T6_EVALUATOR_CONTRACT,
            budget_contract: READOUT_BUDGET_CONTRACT,
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
        if case.inference_input().generator_contract != MIXED_GEOMETRY_TASK_CONTRACT
            || case.label_contract() != PROTECTED_LABEL_CONTRACT
        {
            return Err(EvalError::ContractMismatch("mixed_case"));
        }
        let score = run_inference_callback(case, score_mixed_torsor).map_err(EvalError::Bridge)?;
        let oracle = case.protected_label().reveal_for_evaluation();
        let record = T6EvalRecord {
            split: case.inference_input().split,
            family: case.inference_input().task_family,
            case_id: case.inference_input().case_id,
            outcome: T6Outcome {
                score,
                matches_oracle: approximately_equal(score, oracle.expected_torsor_score),
            },
            canonical_digest: canonicalize_mixed_geometry_input(case.inference_input()).digest,
            envelope_contract: EVALUATOR_ENVELOPE_CONTRACT,
            arm_contract: T6_EVALUATOR_CONTRACT,
            budget_contract: READOUT_BUDGET_CONTRACT,
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

/// Inference-only score callback for torsor-favorable inputs.
pub fn score_torsor_transport(input: &TorsorTransportInput) -> Result<f64, Tdi25Error> {
    torsor_arm_score(input.query, input.key, input.query_position)
}

/// Inference-only T6 score callback for mixed inputs.
pub fn score_mixed_torsor(input: &MixedGeometryInput) -> Result<f64, Tdi25Error> {
    torsor_arm_score(input.torsor_query, input.torsor_key, input.query_position)
}

fn approximately_equal(left: f64, right: f64) -> bool {
    let scale = 1.0_f64.max(left.abs()).max(right.abs());
    (left - right).abs() <= 128.0 * f64::EPSILON * scale
}

/// Fail-closed T6 evaluator errors.
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
}

impl fmt::Display for EvalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ContractMismatch(field) => {
                write!(formatter, "T6 evaluator contract mismatch: {field}")
            }
            Self::InvalidBudget => formatter.write_str("T6 evaluator budget is invalid"),
            Self::SplitMismatch { expected, actual } => write!(
                formatter,
                "T6 evaluator split mismatch: expected {}, got {}",
                expected.as_str(),
                actual.as_str()
            ),
            Self::CaseBudgetExceeded => formatter.write_str("T6 evaluator case budget exceeded"),
            Self::Bridge(error) => write!(formatter, "T6 evaluator bridge failure: {error}"),
        }
    }
}

impl std::error::Error for EvalError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::experimental::tdi25_tasks::{
        DataSplit, mixed_geometry_pair_in_split, seal_mixed_geometry, seal_torsor_transport,
        torsor_transport_pair_in_split,
    };

    #[test]
    fn contracts_and_non_trained_budget_are_pinned() {
        assert_eq!(EVALUATOR_ENVELOPE_CONTRACT, "tdi25-evaluator-envelope-v1");
        assert_eq!(T6_EVALUATOR_CONTRACT, "tdi25-t6-evaluator-v1");
        assert_eq!(READOUT_BUDGET_CONTRACT, "tdi25-readout-budget-v1");
        let budget = ReadoutBudget::matched_non_trained();
        assert_eq!(budget.max_cases, 64);
        assert_eq!(budget.max_readout_scalars_per_case, 2);
        assert_eq!(budget.updates, 0);
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
}
