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
//! mismatches fail-closed without silently compensating. All arms consume
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

fn validate_capacity_carrier(capacity: ParameterReadoutCapacity) -> Result<(), EvalError> {
    let accounting = carrier_accounting(capacity.arm);
    if capacity.query_components != accounting.query_components as u64
        || capacity.score_components != accounting.score_components as u64
        || capacity.key_components != accounting.key_components as u64
        || capacity.external_geometry_components
            != accounting.external_geometry_components as u64
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

/// Immutable configuration for one Phase-C evaluator run.
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

    /// Construct a Development/Validation-only C6 configuration.
    #[must_use]
    pub const fn c6(split: DataSplit) -> Self {
        Self {
            split,
            budget: ReadoutBudget::matched_non_trained(),
            envelope_contract: EVALUATOR_ENVELOPE_CONTRACT,
            arm_contract: C6_EVALUATOR_CONTRACT,
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
    fn parameter_readout_matcher_rejects_inflated_updates_and_drifted_readout() {
        let t6 = ParameterReadoutCapacity::reference_t6();
        let mut inflated = ParameterReadoutCapacity::reference_g6();
        inflated.updates = 4;
        inflated.trainable_parameters = 4;
        assert_eq!(
            match_parameter_readouts(t6, inflated),
            Err(EvalError::ParameterReadoutMismatch {
                left_arm: ComparisonArm::T6,
                right_arm: ComparisonArm::G6,
                left_trainable: 0,
                right_trainable: 4,
                left_updates: 0,
                right_updates: 4,
                left_query_components: 6,
                right_query_components: 6,
            })
        );

        let mut drifted_scalars = ParameterReadoutCapacity::reference_c6();
        drifted_scalars.max_readout_scalars_per_case = 8;
        assert!(matches!(
            match_parameter_readouts(t6, drifted_scalars),
            Err(EvalError::ParameterReadoutMismatch { .. })
        ));

        let mut drifted_cases = ParameterReadoutCapacity::reference_c6();
        drifted_cases.max_cases = MAX_CASES_PER_RUN + 8;
        assert!(matches!(
            match_parameter_readouts(ParameterReadoutCapacity::reference_g6(), drifted_cases),
            Err(EvalError::ParameterReadoutMismatch { .. })
        ));
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
}
