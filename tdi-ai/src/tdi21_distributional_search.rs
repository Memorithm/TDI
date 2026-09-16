//! Bounded Development-only sparse-ANF search for TDI-21 B4 distributions.
//!
//! Unlike exact truth-table search, this surface preserves repeated observable
//! states with conflicting counterfactual outcomes. Selection consumes only a
//! `DevelopmentAdmissionSet`; Validation is accepted only after a policy has
//! already been selected. This is non-final development scaffolding.

use super::tdi21_anf_search::{AnfSearchError, SearchEnvelope};
use super::tdi21_anf_synthesis::{AnfProgram, AnfSynthesisError, synthesize_anf};
use super::tdi21_distributional_objective::{
    DevelopmentAdmissionSet, DistributionalObjectiveError, DistributionalScore,
    ValidationAdmissionSet, score_development_policy, score_validation_policy,
};

pub const DISTRIBUTIONAL_SEARCH_SEMANTICS: &str = "tdi21-b4-distributional-anf-search-v1";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DistributionalSearchWork {
    pub candidates_evaluated: u64,
    pub state_evaluations: u64,
    pub monomial_evaluations: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DistributionalSearchResult {
    envelope: SearchEnvelope,
    program: AnfProgram,
    development_samples: u64,
    development_unique_states: u64,
    development_selected_failures: u64,
    development_hindsight_oracle_failures: u64,
    development_state_oracle_failures: u64,
    development_conflict_lower_bound: u64,
    development_policy_excess_failures: u64,
    work: DistributionalSearchWork,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DistributionalSearchError {
    VariableCountMismatch { envelope: u8, dataset: u8 },
    CandidateSpaceExceedsBudget { required: u64, budget: u64 },
    SelectionScoreMismatch { enumerated: u64, rescored: u64 },
    CounterOverflow,
    AllocationFailed,
    ExactSearchEnvelope(AnfSearchError),
    Program(AnfSynthesisError),
    Objective(DistributionalObjectiveError),
}

impl From<AnfSynthesisError> for DistributionalSearchError {
    fn from(value: AnfSynthesisError) -> Self {
        Self::Program(value)
    }
}

impl From<DistributionalObjectiveError> for DistributionalSearchError {
    fn from(value: DistributionalObjectiveError) -> Self {
        Self::Objective(value)
    }
}

impl DistributionalSearchResult {
    #[must_use]
    pub const fn envelope(&self) -> SearchEnvelope {
        self.envelope
    }

    #[must_use]
    pub const fn program(&self) -> &AnfProgram {
        &self.program
    }

    #[must_use]
    pub const fn development_samples(&self) -> u64 {
        self.development_samples
    }

    #[must_use]
    pub const fn development_unique_states(&self) -> u64 {
        self.development_unique_states
    }

    #[must_use]
    pub const fn development_selected_failures(&self) -> u64 {
        self.development_selected_failures
    }

    #[must_use]
    pub const fn development_hindsight_oracle_failures(&self) -> u64 {
        self.development_hindsight_oracle_failures
    }

    #[must_use]
    pub const fn development_state_oracle_failures(&self) -> u64 {
        self.development_state_oracle_failures
    }

    #[must_use]
    pub const fn development_conflict_lower_bound(&self) -> u64 {
        self.development_conflict_lower_bound
    }

    #[must_use]
    pub const fn development_policy_excess_failures(&self) -> u64 {
        self.development_policy_excess_failures
    }

    #[must_use]
    pub const fn work(&self) -> DistributionalSearchWork {
        self.work
    }
}

fn checked_add(target: &mut u64, amount: u64) -> Result<(), DistributionalSearchError> {
    *target = target
        .checked_add(amount)
        .ok_or(DistributionalSearchError::CounterOverflow)?;
    Ok(())
}

fn binomial(n: usize, k: usize) -> Result<u64, DistributionalSearchError> {
    if k > n {
        return Ok(0);
    }
    let k = k.min(n - k);
    let mut value = 1u64;
    for i in 0..k {
        value = value
            .checked_mul((n - i) as u64)
            .ok_or(DistributionalSearchError::CounterOverflow)?;
        value /= (i + 1) as u64;
    }
    Ok(value)
}

fn eligible_monomials(envelope: SearchEnvelope) -> Result<Vec<u64>, DistributionalSearchError> {
    envelope
        .validate()
        .map_err(DistributionalSearchError::ExactSearchEnvelope)?;
    let limit = 1u64 << envelope.variable_count;
    Ok((1..limit)
        .filter(|mask| mask.count_ones() <= u32::from(envelope.max_degree))
        .collect())
}

fn candidate_space_size(
    monomials: usize,
    max_terms: usize,
) -> Result<u64, DistributionalSearchError> {
    let mut combinations = 0u64;
    for terms in 0..=max_terms.min(monomials) {
        checked_add(&mut combinations, binomial(monomials, terms)?)?;
    }
    combinations
        .checked_mul(2)
        .ok_or(DistributionalSearchError::CounterOverflow)
}

fn enumerate_combinations(
    universe: &[u64],
    start: usize,
    remaining: usize,
    current: &mut Vec<u64>,
    out: &mut Vec<Vec<u64>>,
) -> Result<(), DistributionalSearchError> {
    if remaining == 0 {
        out.try_reserve(1)
            .map_err(|_| DistributionalSearchError::AllocationFailed)?;
        out.push(current.clone());
        return Ok(());
    }
    if universe.len().saturating_sub(start) < remaining {
        return Ok(());
    }
    let final_start = universe.len() - remaining;
    for index in start..=final_start {
        current.push(universe[index]);
        enumerate_combinations(universe, index + 1, remaining - 1, current, out)?;
        current.pop();
    }
    Ok(())
}

fn evaluate_sparse(constant: bool, monomials: &[u64], assignment: u64) -> bool {
    let mut value = constant;
    for &mask in monomials {
        value ^= assignment & mask == mask;
    }
    value
}

fn candidate_failures(
    development: &DevelopmentAdmissionSet,
    constant: bool,
    monomials: &[u64],
    work: &mut DistributionalSearchWork,
) -> Result<u64, DistributionalSearchError> {
    let mut failures = 0u64;
    for state in development.states() {
        checked_add(&mut work.state_evaluations, 1)?;
        checked_add(&mut work.monomial_evaluations, monomials.len() as u64)?;
        let outcomes = state.outcomes();
        let row_failures = if evaluate_sparse(constant, monomials, state.assignment()) {
            outcomes.failures_if_admit()
        } else {
            outcomes.failures_if_inhibit()
        };
        checked_add(&mut failures, row_failures)?;
    }
    Ok(failures)
}

fn degree(monomials: &[u64]) -> u32 {
    monomials
        .iter()
        .map(|mask| mask.count_ones())
        .max()
        .unwrap_or(0)
}

fn better_candidate(
    failures: u64,
    constant: bool,
    monomials: &[u64],
    best_failures: u64,
    best_constant: bool,
    best_monomials: &[u64],
) -> bool {
    (
        failures,
        monomials.len(),
        degree(monomials),
        constant,
        monomials,
    ) < (
        best_failures,
        best_monomials.len(),
        degree(best_monomials),
        best_constant,
        best_monomials,
    )
}

fn sparse_program(
    variable_count: u8,
    constant: bool,
    monomials: &[u64],
) -> Result<AnfProgram, DistributionalSearchError> {
    let rows = 1usize << variable_count;
    let mut table = Vec::new();
    table
        .try_reserve_exact(rows)
        .map_err(|_| DistributionalSearchError::AllocationFailed)?;
    for assignment in 0..rows as u64 {
        table.push(evaluate_sparse(constant, monomials, assignment));
    }
    Ok(synthesize_anf(variable_count, &table)?)
}

/// Exhaustively select a sparse deterministic ANF policy on Development only.
///
/// Primary objective is Development failure count with every original sample
/// retaining unit weight through its aggregated state's exact outcome counts.
/// Ties prefer fewer terms, lower degree, constant=false, then canonical masks.
pub fn fit_distributional_anf(
    development: &DevelopmentAdmissionSet,
    envelope: SearchEnvelope,
) -> Result<DistributionalSearchResult, DistributionalSearchError> {
    envelope
        .validate()
        .map_err(DistributionalSearchError::ExactSearchEnvelope)?;
    if envelope.variable_count != development.variable_count() {
        return Err(DistributionalSearchError::VariableCountMismatch {
            envelope: envelope.variable_count,
            dataset: development.variable_count(),
        });
    }

    let universe = eligible_monomials(envelope)?;
    let required = candidate_space_size(universe.len(), envelope.max_terms)?;
    if required > envelope.max_candidates {
        return Err(DistributionalSearchError::CandidateSpaceExceedsBudget {
            required,
            budget: envelope.max_candidates,
        });
    }

    let mut combinations = Vec::new();
    combinations
        .try_reserve_exact((required / 2) as usize)
        .map_err(|_| DistributionalSearchError::AllocationFailed)?;
    for term_count in 0..=envelope.max_terms.min(universe.len()) {
        enumerate_combinations(&universe, 0, term_count, &mut Vec::new(), &mut combinations)?;
    }

    let mut work = DistributionalSearchWork::default();
    let mut best: Option<(u64, bool, Vec<u64>)> = None;
    for constant in [false, true] {
        for monomials in &combinations {
            checked_add(&mut work.candidates_evaluated, 1)?;
            let failures = candidate_failures(development, constant, monomials, &mut work)?;
            match &best {
                None => best = Some((failures, constant, monomials.clone())),
                Some((best_failures, best_constant, best_monomials))
                    if better_candidate(
                        failures,
                        constant,
                        monomials,
                        *best_failures,
                        *best_constant,
                        best_monomials,
                    ) =>
                {
                    best = Some((failures, constant, monomials.clone()));
                }
                _ => {}
            }
        }
    }

    let (enumerated_failures, constant, monomials) =
        best.expect("validated candidate space always contains constant policies");
    let program = sparse_program(envelope.variable_count, constant, &monomials)?;
    let score = score_development_policy(&program, development)?;
    if enumerated_failures != score.selected_failures() {
        return Err(DistributionalSearchError::SelectionScoreMismatch {
            enumerated: enumerated_failures,
            rescored: score.selected_failures(),
        });
    }

    Ok(DistributionalSearchResult {
        envelope,
        program,
        development_samples: score.samples(),
        development_unique_states: score.unique_states(),
        development_selected_failures: score.selected_failures(),
        development_hindsight_oracle_failures: score.hindsight_oracle_failures(),
        development_state_oracle_failures: score.deterministic_state_oracle_failures(),
        development_conflict_lower_bound: score.deterministic_conflict_lower_bound(),
        development_policy_excess_failures: score.policy_excess_failures(),
        work,
    })
}

/// Evaluate a fixed selected policy on Validation without refitting it.
pub fn evaluate_distributional_validation(
    result: &DistributionalSearchResult,
    validation: &ValidationAdmissionSet,
) -> Result<DistributionalScore, DistributionalSearchError> {
    if validation.variable_count() != result.envelope.variable_count {
        return Err(DistributionalSearchError::VariableCountMismatch {
            envelope: result.envelope.variable_count,
            dataset: validation.variable_count(),
        });
    }
    Ok(score_validation_policy(result.program(), validation)?)
}
