//! Exact distributional objective for non-identifying TDI-21 B4 observations.
//!
//! This module does not fit a policy. It aggregates evaluator-only Development
//! or Validation samples that may share the same causal predicate assignment
//! but have different counterfactual Admit/Inhibit outcomes. A deterministic
//! ANF policy can then be scored without collapsing those conflicts into a fake
//! single truth-table label.

use std::collections::BTreeMap;

use super::tdi21_anf_synthesis::{AnfProgram, AnfSynthesisError};

pub const DISTRIBUTIONAL_OBJECTIVE_SEMANTICS: &str =
    "tdi21-b4-admission-distributional-objective-v1";
pub const MAX_DISTRIBUTIONAL_VARIABLES: u8 = 6;
pub const MAX_DISTRIBUTIONAL_SAMPLES: usize = 4096;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AdmissionOutcomeSample {
    pub assignment: u64,
    pub admit_succeeds: bool,
    pub inhibit_succeeds: bool,
}

/// Exact four-way aggregation for one observable predicate assignment.
/// Construction is internal so count invariants cannot be forged by callers.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OutcomeCounts {
    both_succeed: u64,
    admit_only: u64,
    inhibit_only: u64,
    neither_succeeds: u64,
}

impl OutcomeCounts {
    #[must_use]
    pub const fn both_succeed(self) -> u64 {
        self.both_succeed
    }

    #[must_use]
    pub const fn admit_only(self) -> u64 {
        self.admit_only
    }

    #[must_use]
    pub const fn inhibit_only(self) -> u64 {
        self.inhibit_only
    }

    #[must_use]
    pub const fn neither_succeeds(self) -> u64 {
        self.neither_succeeds
    }

    #[must_use]
    pub const fn observations(self) -> u64 {
        self.both_succeed + self.admit_only + self.inhibit_only + self.neither_succeeds
    }

    #[must_use]
    pub const fn failures_if_admit(self) -> u64 {
        self.inhibit_only + self.neither_succeeds
    }

    #[must_use]
    pub const fn failures_if_inhibit(self) -> u64 {
        self.admit_only + self.neither_succeeds
    }

    /// Failure floor when action may vary per episode with evaluator hindsight.
    #[must_use]
    pub const fn hindsight_oracle_failures(self) -> u64 {
        self.neither_succeeds
    }

    /// Additional failures forced by mapping this observable state to one fixed
    /// deterministic action, relative to the per-episode hindsight oracle.
    #[must_use]
    pub const fn deterministic_conflict_penalty(self) -> u64 {
        if self.admit_only < self.inhibit_only {
            self.admit_only
        } else {
            self.inhibit_only
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AggregatedAdmissionState {
    assignment: u64,
    outcomes: OutcomeCounts,
}

impl AggregatedAdmissionState {
    #[must_use]
    pub const fn assignment(self) -> u64 {
        self.assignment
    }

    #[must_use]
    pub const fn outcomes(self) -> OutcomeCounts {
        self.outcomes
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AdmissionDataset {
    variable_count: u8,
    sample_count: u64,
    states: Vec<AggregatedAdmissionState>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DevelopmentAdmissionSet(AdmissionDataset);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidationAdmissionSet(AdmissionDataset);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DistributionalObjectiveError {
    ZeroVariables,
    TooManyVariables,
    EmptySamples,
    TooManySamples,
    AssignmentOutOfRange { index: usize, assignment: u64 },
    CountOverflow,
    AllocationFailed,
    ProgramArityMismatch { program: u8, dataset: u8 },
    Program(AnfSynthesisError),
}

impl From<AnfSynthesisError> for DistributionalObjectiveError {
    fn from(value: AnfSynthesisError) -> Self {
        Self::Program(value)
    }
}

fn increment(value: &mut u64) -> Result<(), DistributionalObjectiveError> {
    *value = value
        .checked_add(1)
        .ok_or(DistributionalObjectiveError::CountOverflow)?;
    Ok(())
}

fn aggregate_samples(
    variable_count: u8,
    samples: &[AdmissionOutcomeSample],
) -> Result<AdmissionDataset, DistributionalObjectiveError> {
    if variable_count == 0 {
        return Err(DistributionalObjectiveError::ZeroVariables);
    }
    if variable_count > MAX_DISTRIBUTIONAL_VARIABLES {
        return Err(DistributionalObjectiveError::TooManyVariables);
    }
    if samples.is_empty() {
        return Err(DistributionalObjectiveError::EmptySamples);
    }
    if samples.len() > MAX_DISTRIBUTIONAL_SAMPLES {
        return Err(DistributionalObjectiveError::TooManySamples);
    }

    let limit = 1u64 << variable_count;
    let mut counts = BTreeMap::<u64, OutcomeCounts>::new();
    for (index, sample) in samples.iter().copied().enumerate() {
        if sample.assignment >= limit {
            return Err(DistributionalObjectiveError::AssignmentOutOfRange {
                index,
                assignment: sample.assignment,
            });
        }
        let row = counts.entry(sample.assignment).or_default();
        match (sample.admit_succeeds, sample.inhibit_succeeds) {
            (true, true) => increment(&mut row.both_succeed)?,
            (true, false) => increment(&mut row.admit_only)?,
            (false, true) => increment(&mut row.inhibit_only)?,
            (false, false) => increment(&mut row.neither_succeeds)?,
        }
    }

    let mut states = Vec::new();
    states
        .try_reserve_exact(counts.len())
        .map_err(|_| DistributionalObjectiveError::AllocationFailed)?;
    states.extend(
        counts
            .into_iter()
            .map(|(assignment, outcomes)| AggregatedAdmissionState {
                assignment,
                outcomes,
            }),
    );
    Ok(AdmissionDataset {
        variable_count,
        sample_count: samples.len() as u64,
        states,
    })
}

macro_rules! impl_dataset {
    ($name:ident) => {
        impl $name {
            pub fn new(
                variable_count: u8,
                samples: &[AdmissionOutcomeSample],
            ) -> Result<Self, DistributionalObjectiveError> {
                aggregate_samples(variable_count, samples).map(Self)
            }

            #[must_use]
            pub const fn variable_count(&self) -> u8 {
                self.0.variable_count
            }

            #[must_use]
            pub const fn sample_count(&self) -> u64 {
                self.0.sample_count
            }

            #[must_use]
            pub fn states(&self) -> &[AggregatedAdmissionState] {
                &self.0.states
            }
        }
    };
}

impl_dataset!(DevelopmentAdmissionSet);
impl_dataset!(ValidationAdmissionSet);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DistributionalScore {
    samples: u64,
    unique_states: u64,
    admit_selected_samples: u64,
    inhibit_selected_samples: u64,
    selected_failures: u64,
    hindsight_oracle_failures: u64,
    deterministic_state_oracle_failures: u64,
    deterministic_conflict_lower_bound: u64,
    runtime_anf_term_evaluations: u64,
}

impl DistributionalScore {
    #[must_use]
    pub const fn samples(self) -> u64 {
        self.samples
    }

    #[must_use]
    pub const fn unique_states(self) -> u64 {
        self.unique_states
    }

    #[must_use]
    pub const fn admit_selected_samples(self) -> u64 {
        self.admit_selected_samples
    }

    #[must_use]
    pub const fn inhibit_selected_samples(self) -> u64 {
        self.inhibit_selected_samples
    }

    #[must_use]
    pub const fn selected_failures(self) -> u64 {
        self.selected_failures
    }

    #[must_use]
    pub const fn hindsight_oracle_failures(self) -> u64 {
        self.hindsight_oracle_failures
    }

    #[must_use]
    pub const fn deterministic_state_oracle_failures(self) -> u64 {
        self.deterministic_state_oracle_failures
    }

    #[must_use]
    pub const fn deterministic_conflict_lower_bound(self) -> u64 {
        self.deterministic_conflict_lower_bound
    }

    /// Runtime ANF term work implied if the selected policy were evaluated once
    /// per original sample. The scorer itself evaluates once per unique state.
    #[must_use]
    pub const fn runtime_anf_term_evaluations(self) -> u64 {
        self.runtime_anf_term_evaluations
    }

    #[must_use]
    pub const fn selected_successes(self) -> u64 {
        self.samples - self.selected_failures
    }

    /// Excess failures above the best deterministic action chosen separately
    /// for each observable state. This isolates policy-form error from feature
    /// non-identifiability.
    #[must_use]
    pub const fn policy_excess_failures(self) -> u64 {
        self.selected_failures - self.deterministic_state_oracle_failures
    }
}

fn checked_add(target: &mut u64, amount: u64) -> Result<(), DistributionalObjectiveError> {
    *target = target
        .checked_add(amount)
        .ok_or(DistributionalObjectiveError::CountOverflow)?;
    Ok(())
}

fn score_dataset(
    program: &AnfProgram,
    dataset: &AdmissionDataset,
) -> Result<DistributionalScore, DistributionalObjectiveError> {
    if program.variable_count() != dataset.variable_count {
        return Err(DistributionalObjectiveError::ProgramArityMismatch {
            program: program.variable_count(),
            dataset: dataset.variable_count,
        });
    }

    let mut score = DistributionalScore {
        samples: dataset.sample_count,
        unique_states: dataset.states.len() as u64,
        ..DistributionalScore::default()
    };
    for state in &dataset.states {
        let choose_admit = program.evaluate(state.assignment)?;
        let observations = state.outcomes.observations();
        if choose_admit {
            checked_add(&mut score.admit_selected_samples, observations)?;
            checked_add(
                &mut score.selected_failures,
                state.outcomes.failures_if_admit(),
            )?;
        } else {
            checked_add(&mut score.inhibit_selected_samples, observations)?;
            checked_add(
                &mut score.selected_failures,
                state.outcomes.failures_if_inhibit(),
            )?;
        }
        let hindsight = state.outcomes.hindsight_oracle_failures();
        let conflict = state.outcomes.deterministic_conflict_penalty();
        checked_add(&mut score.hindsight_oracle_failures, hindsight)?;
        checked_add(&mut score.deterministic_conflict_lower_bound, conflict)?;
        checked_add(
            &mut score.deterministic_state_oracle_failures,
            hindsight
                .checked_add(conflict)
                .ok_or(DistributionalObjectiveError::CountOverflow)?,
        )?;
        let term_evals = observations
            .checked_mul(program.terms().len() as u64)
            .ok_or(DistributionalObjectiveError::CountOverflow)?;
        checked_add(&mut score.runtime_anf_term_evaluations, term_evals)?;
    }
    Ok(score)
}

pub fn score_development_policy(
    program: &AnfProgram,
    development: &DevelopmentAdmissionSet,
) -> Result<DistributionalScore, DistributionalObjectiveError> {
    score_dataset(program, &development.0)
}

pub fn score_validation_policy(
    program: &AnfProgram,
    validation: &ValidationAdmissionSet,
) -> Result<DistributionalScore, DistributionalObjectiveError> {
    score_dataset(program, &validation.0)
}
