//! Canonical provenance records for TDI-2.1 reference inference.

use super::tdi2_intuition::BooleanState;
use super::tdi2_intuition_inference::IntuitionOutcome;
use super::tdi2_intuition_relations::RelationalTemplate;
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
    pub fn new(state: &BooleanState, candidates: &[Candidate], outcome: IntuitionOutcome) -> Self {
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

/// Canonical structural record that intentionally excludes the template id and evidence.
///
/// Two templates with the same returned record are structurally equivalent for
/// Boolean applicability and role-relation transfer, even if they have different ids.
#[must_use]
pub fn template_structure_record(template: &RelationalTemplate) -> String {
    let required = template
        .base()
        .required()
        .iter()
        .map(|predicate| predicate.raw().to_string())
        .collect::<Vec<_>>()
        .join(",");
    let forbidden = template
        .base()
        .forbidden()
        .iter()
        .map(|predicate| predicate.raw().to_string())
        .collect::<Vec<_>>()
        .join(",");
    let roles = template
        .base()
        .roles()
        .iter()
        .map(|role| role.raw().to_string())
        .collect::<Vec<_>>()
        .join(",");
    let relations = template
        .relations()
        .iter()
        .map(|relation| {
            format!(
                "{}:{}:{}",
                relation.left().raw(),
                relation.relation().raw(),
                relation.right().raw()
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "tdi2.1-template-structure-v1;required={required};forbidden={forbidden};roles={roles};relations={relations}"
    )
}

#[cfg(test)]
mod tests {
    use super::{InferenceTrace, template_structure_record};
    use crate::experimental::tdi2_intuition::{
        BooleanState, PredicateId, RoleId, Template, TemplateId,
    };
    use crate::experimental::tdi2_intuition_inference::IntuitionOutcome;
    use crate::experimental::tdi2_intuition_relations::{
        RelationId, RelationalTemplate, RoleRelation,
    };
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

    #[test]
    fn structural_record_ignores_template_identity() {
        let make = |id| {
            let base = Template::new(
                TemplateId::new(id),
                vec![PredicateId::new(1)],
                Vec::new(),
                vec![RoleId::new(1), RoleId::new(2)],
            )
            .expect("template");
            RelationalTemplate::new(
                base,
                vec![RoleRelation::new(
                    RoleId::new(1),
                    RelationId::new(3),
                    RoleId::new(2),
                )],
            )
            .expect("relation")
        };
        assert_eq!(
            template_structure_record(&make(1)),
            template_structure_record(&make(9))
        );
    }
}
