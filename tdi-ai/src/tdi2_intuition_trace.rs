//! Canonical provenance records for TDI-2.1 reference inference.

use super::tdi2_intuition::BooleanState;
use super::tdi2_intuition_inference::IntuitionOutcome;
use super::tdi2_intuition_selection::Candidate;

/// Immutable trace of one inference decision.
#[derive(Clone, Debug, PartialEq)]
pub struct InferenceTrace {
    predicate_ids: Vec<u32>,
    candidate_records: Vec<(u64, u64, u64)>,
    outcome: IntuitionOutcome,
}

impl InferenceTrace {
    /// Capture a canonical trace from an already-canonical state and ranked candidates.
    #[must_use]
    pub fn new(
        state: &BooleanState,
        candidates: &[Candidate],
        outcome: IntuitionOutcome,
    ) -> Self {
        let predicate_ids = state
            .predicates()
            .iter()
            .map(|predicate| predicate.raw())
            .collect();
        let candidate_records = candidates
            .iter()
            .map(|candidate| {
                (
                    candidate.template_id().raw(),
                    candidate.weight().to_bits(),
                    candidate.support(),
                )
            })
            .collect();
        Self {
            predicate_ids,
            candidate_records,
            outcome,
        }
    }

    /// Stable textual record suitable for artifact hashing by the surrounding TDI harness.
    #[must_use]
    pub fn canonical_record(&self) -> String {
        let predicates = self
            .predicate_ids
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let candidates = self
            .candidate_records
            .iter()
            .map(|(id, weight_bits, support)| format!("{id}:{weight_bits:016x}:{support}"))
            .collect::<Vec<_>>()
            .join(",");
        let outcome = match self.outcome {
            IntuitionOutcome::Selected {
                template_id,
                weight,
                support,
            } => format!(
                "selected:{}:{:016x}:{}",
                template_id.raw(),
                weight.to_bits(),
                support
            ),
            IntuitionOutcome::InsufficientExperience => "insufficient-experience".to_owned(),
        };
        format!(
            "tdi2.1-intuition-trace-v1;predicates={predicates};candidates={candidates};outcome={outcome}"
        )
    }

    /// Captured outcome.
    #[must_use]
    pub const fn outcome(&self) -> IntuitionOutcome {
        self.outcome
    }
}

#[cfg(test)]
mod tests {
    use super::InferenceTrace;
    use crate::experimental::tdi2_intuition::{BooleanState, PredicateId, TemplateId};
    use crate::experimental::tdi2_intuition_inference::IntuitionOutcome;
    use crate::experimental::tdi2_intuition_selection::Candidate;

    #[test]
    fn canonical_trace_is_order_stable() {
        let state = BooleanState::new(vec![PredicateId::new(9), PredicateId::new(2)]);
        let candidates = [Candidate::new(TemplateId::new(7), 1.5, 3)];
        let outcome = IntuitionOutcome::Selected {
            template_id: TemplateId::new(7),
            weight: 1.5,
            support: 3,
        };
        let left = InferenceTrace::new(&state, &candidates, outcome).canonical_record();
        let right = InferenceTrace::new(&state, &candidates, outcome).canonical_record();
        assert_eq!(left, right);
        assert!(left.contains("predicates=2,9"));
        assert!(left.contains("selected:7"));
    }
}
