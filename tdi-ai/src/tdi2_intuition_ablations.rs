//! Controlled ablations for TDI-2.1 mechanism attribution.

use super::tdi2_intuition::{BooleanState, PredicateId};
use super::tdi2_intuition_relations::{RelationError, RelationalTemplate};

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

#[cfg(test)]
mod tests {
    use super::{remove_context, remove_relations};
    use crate::experimental::tdi2_intuition::{BooleanState, PredicateId, RoleId, Template, TemplateId};
    use crate::experimental::tdi2_intuition_relations::{RelationId, RelationalTemplate, RoleRelation};

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
}
