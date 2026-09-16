//! Composed scalar reference engine for the TDI-2.1 intuition programme.
//!
//! The engine performs one bounded pass over stored templates. It does not
//! mutate experience, perform iterative deliberation, or use latency as an
//! algorithmic input.

use super::tdi2_intuition::BooleanState;
use super::tdi2_intuition_inference::{InferencePolicy, IntuitionOutcome, infer};
use super::tdi2_intuition_reliability::ReliabilityError;
use super::tdi2_intuition_selection::{Candidate, select_candidates};
use super::tdi2_intuition_store::ExperienceStore;
use super::tdi2_intuition_trace::InferenceTrace;
use super::tdi2_intuition_weight::ExperienceWeightPolicy;

/// Frozen policies used by one reference engine instance.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IntuitionEngine {
    experience_weight: ExperienceWeightPolicy,
    inference: InferencePolicy,
}

impl IntuitionEngine {
    /// Construct a reference engine from already-validated policies.
    #[must_use]
    pub const fn new(
        experience_weight: ExperienceWeightPolicy,
        inference: InferencePolicy,
    ) -> Self {
        Self {
            experience_weight,
            inference,
        }
    }

    /// Execute one read-only intuition pass.
    pub fn run(
        &self,
        store: &ExperienceStore,
        state: &BooleanState,
    ) -> Result<EngineReport, ReliabilityError> {
        let candidates = select_candidates(store, state, self.experience_weight)?;
        let outcome = infer(&candidates, self.inference);
        let trace = InferenceTrace::new(state, &candidates, outcome);
        Ok(EngineReport {
            candidates,
            outcome,
            trace,
        })
    }
}

impl Default for IntuitionEngine {
    fn default() -> Self {
        Self::new(ExperienceWeightPolicy::default(), InferencePolicy::default())
    }
}

/// Complete non-mutating result of one scalar reference inference.
#[derive(Clone, Debug, PartialEq)]
pub struct EngineReport {
    candidates: Vec<Candidate>,
    outcome: IntuitionOutcome,
    trace: InferenceTrace,
}

impl EngineReport {
    /// Ranked exact-applicability candidates.
    #[must_use]
    pub fn candidates(&self) -> &[Candidate] {
        &self.candidates
    }

    /// Selected template or explicit insufficient-experience result.
    #[must_use]
    pub const fn outcome(&self) -> IntuitionOutcome {
        self.outcome
    }

    /// Canonical provenance trace for this inference.
    #[must_use]
    pub const fn trace(&self) -> &InferenceTrace {
        &self.trace
    }
}

#[cfg(test)]
mod tests {
    use super::IntuitionEngine;
    use crate::experimental::tdi2_intuition::{BooleanState, PredicateId, Template, TemplateId};
    use crate::experimental::tdi2_intuition_inference::IntuitionOutcome;
    use crate::experimental::tdi2_intuition_relations::RelationalTemplate;
    use crate::experimental::tdi2_intuition_reliability::ReliabilityEvidence;
    use crate::experimental::tdi2_intuition_store::{ExperienceEntry, ExperienceStore};

    fn entry(id: u64, predicate: u32, successes: u64) -> ExperienceEntry {
        let base = Template::new(
            TemplateId::new(id),
            vec![PredicateId::new(predicate)],
            Vec::new(),
            Vec::new(),
        )
        .expect("valid template");
        ExperienceEntry::new(
            RelationalTemplate::new(base, Vec::new()).expect("valid relational template"),
            ReliabilityEvidence::new(successes, 0),
        )
    }

    #[test]
    fn engine_selects_stronger_applicable_experience_without_mutating_store() {
        let mut store = ExperienceStore::new(2).expect("valid store");
        store.insert(entry(1, 5, 2)).expect("insert");
        store.insert(entry(2, 5, 20)).expect("insert");
        let before = store.clone();
        let state = BooleanState::new(vec![PredicateId::new(5)]);
        let report = IntuitionEngine::default()
            .run(&store, &state)
            .expect("valid evidence");
        assert!(matches!(
            report.outcome(),
            IntuitionOutcome::Selected { template_id, .. } if template_id == TemplateId::new(2)
        ));
        assert_eq!(store, before);
        assert_eq!(report.candidates().len(), 2);
    }
}
