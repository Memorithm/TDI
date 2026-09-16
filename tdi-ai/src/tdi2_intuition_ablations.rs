//! Controlled ablations for TDI-2.1 mechanism attribution.

use super::tdi2_intuition::{BooleanState, PredicateId};
use super::tdi2_intuition_relations::{RelationError, RelationalTemplate};
use super::tdi2_intuition_selection::Candidate;

/// Remove a frozen set of context predicates from one Boolean query.
#[must_use]
pub fn remove_context(
    state: &BooleanState,
    context_predicates: &[PredicateId],
) -> BooleanState {
    let mut masked = context_predicates.to_vec();
    masked.sort_unstable();
    masked.dedup();
    BooleanState::new(
        state
            .predicates()
            .iter()
            .copied()
            .filter(|predicate| masked.binary_search(predicate).is_err())
            .collect(),
    )
}

/// Preserve Boolean applicability and roles while removing all role relations.
pub fn remove_relations(template: &RelationalTemplate) -> Result<RelationalTemplate, RelationError> {
    RelationalTemplate::new(template.base().clone(), Vec::new())
}

/// Remove experience-strength ranking while preserving the exact candidate set.
///
/// All candidates receive the same weight and are ordered by template id. Historical
/// support is retained only for reporting, not ordering.
#[must_use]
pub fn equalize_experience_weights(candidates: &[Candidate]) -> Vec<Candidate> {
    let mut result = candidates
        .iter()
        .map(|candidate| Candidate::new(candidate.template_id(), 1.0, candidate.support()))
        .collect::<Vec<_>>();
    result.sort_unstable_by_key(|candidate| candidate.template_id());
    result
}

#[cfg(test)]
mod tests {
    use super::{equalize_experience_weights, remove_context, remove_relations};
    use crate::experimental::tdi2_intuition::{BooleanState, PredicateId, RoleId, Template, TemplateId};
    use crate::experimental::tdi2_intuition_relations::{RelationId, RelationalTemplate, RoleRelation};
    use crate::experimental::tdi2_intuition_selection::Candidate;

    #[test]
    fn context_ablation_preserves_non_context_structure() {
        let state = BooleanState::new(vec![
            PredicateId::new(1),
            PredicateId::new(2),
            PredicateId::new(9),
        ]);
        let ablated = remove_context(&state, &[PredicateId::new(9)]);
        assert_eq!(ablated.predicates(), &[PredicateId::new(1), PredicateId::new(2)]);
        assert_eq!(state.len(), 3);
    }

    #[test]
    fn relation_ablation_preserves_base_template() {
        let base = Template::new(
            TemplateId::new(1),
            vec![PredicateId::new(1)],
            Vec::new(),
            vec![RoleId::new(1), RoleId::new(2)],
        )
        .expect("template");
        let template = RelationalTemplate::new(
            base.clone(),
            vec![RoleRelation::new(RoleId::new(1), RelationId::new(7), RoleId::new(2))],
        )
        .expect("relations");
        let ablated = remove_relations(&template).expect("ablation");
        assert_eq!(ablated.base(), &base);
        assert!(ablated.relations().is_empty());
    }

    #[test]
    fn weight_ablation_removes_experience_ordering() {
        let candidates = [
            Candidate::new(TemplateId::new(9), 10.0, 100),
            Candidate::new(TemplateId::new(2), 1.0, 1),
        ];
        let ablated = equalize_experience_weights(&candidates);
        assert_eq!(ablated[0].template_id(), TemplateId::new(2));
        assert_eq!(ablated[0].weight(), 1.0);
        assert_eq!(ablated[1].weight(), 1.0);
    }
}
