//! Evaluator-only materialization of TDI-21 B4 admission samples.
//!
//! This bridge removes manual transcription between explicit causal sequence
//! cases and the conflict-preserving distributional objective. Candidate-side
//! code never receives the evaluator future probe or counterfactual outcomes.

use super::tdi21_distributional_objective::{
    AdmissionOutcomeSample, MAX_DISTRIBUTIONAL_SAMPLES,
};
use super::tdi21_predicate_identifiability::{
    AdmissionAuditCase, IdentifiabilityError, audit_admission_case,
};
use super::tdi21_stream::StreamConfig;

pub const SEQUENCE_MATERIALIZER_SEMANTICS: &str = "tdi21-b4-sequence-materializer-v1";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MaterializationSummary {
    pub cases: u64,
    pub both_succeed: u64,
    pub admit_only: u64,
    pub inhibit_only: u64,
    pub neither_succeeds: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaterializedAdmissionSamples {
    samples: Vec<AdmissionOutcomeSample>,
    summary: MaterializationSummary,
}

impl MaterializedAdmissionSamples {
    #[must_use]
    pub fn samples(&self) -> &[AdmissionOutcomeSample] {
        &self.samples
    }

    #[must_use]
    pub const fn summary(&self) -> MaterializationSummary {
        self.summary
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SequenceMaterializationError {
    EmptyCases,
    TooManyCases,
    CountOverflow,
    AllocationFailed,
    Case {
        index: usize,
        source: IdentifiabilityError,
    },
}

fn increment(value: &mut u64) -> Result<(), SequenceMaterializationError> {
    *value = value
        .checked_add(1)
        .ok_or(SequenceMaterializationError::CountOverflow)?;
    Ok(())
}

/// Materialize evaluator-owned counterfactual outcomes from explicit causal
/// sequence cases using the same audited semantics as the v1 identifiability
/// result. No Boolean label is invented or majority-collapsed.
pub fn materialize_admission_cases(
    config: StreamConfig,
    cases: &[AdmissionAuditCase],
) -> Result<MaterializedAdmissionSamples, SequenceMaterializationError> {
    if cases.is_empty() {
        return Err(SequenceMaterializationError::EmptyCases);
    }
    if cases.len() > MAX_DISTRIBUTIONAL_SAMPLES {
        return Err(SequenceMaterializationError::TooManyCases);
    }

    let mut samples = Vec::new();
    samples
        .try_reserve_exact(cases.len())
        .map_err(|_| SequenceMaterializationError::AllocationFailed)?;
    let mut summary = MaterializationSummary::default();

    for (index, case) in cases.iter().enumerate() {
        let audit = audit_admission_case(config, case).map_err(|source| {
            SequenceMaterializationError::Case { index, source }
        })?;
        let sample = AdmissionOutcomeSample {
            assignment: audit.predicates.assignment(),
            admit_succeeds: audit.admit.succeeds,
            inhibit_succeeds: audit.inhibit.succeeds,
        };
        match (sample.admit_succeeds, sample.inhibit_succeeds) {
            (true, true) => increment(&mut summary.both_succeed)?,
            (true, false) => increment(&mut summary.admit_only)?,
            (false, true) => increment(&mut summary.inhibit_only)?,
            (false, false) => increment(&mut summary.neither_succeeds)?,
        }
        increment(&mut summary.cases)?;
        samples.push(sample);
    }

    Ok(MaterializedAdmissionSamples { samples, summary })
}
