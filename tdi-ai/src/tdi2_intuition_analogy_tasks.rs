//! Controlled analogical-transfer task family for TDI-2.1.

use super::tdi2_intuition::{PredicateId, RoleId, Template, TemplateId};
use super::tdi2_intuition_relations::{RelationId, RelationalTemplate, RoleRelation};
use super::tdi2_intuition_transfer::RoleBinding;
use super::tdi2_intuition_transfer_eval::ExpectedRelation;
use super::tdi2_intuition_transfer_tasks::novel_identity_bindings;

/// One frozen analogical-transfer case with independently specified target relations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnalogyCase {
    /// Source experiential template.
    pub template: RelationalTemplate,
    /// Mapping to novel concrete entities.
    pub bindings: Vec<RoleBinding>,
    /// Exact target relation set expected after valid transfer.
    pub expected: Vec<ExpectedRelation>,
}

/// Build a deterministic three-role relational analogy.
#[must_use]
pub fn analogy_case(case_id: u32) -> AnalogyCase {
    let roles = [RoleId::new(1), RoleId::new(2), RoleId::new(3)];
    let base = Template::new(
        TemplateId::new(200_000 + u64::from(case_id)),
        vec![PredicateId::new(30_000 + case_id)],
        Vec::new(),
        roles.to_vec(),
    )
    .expect("fixed analogy template is valid");
    let relations = vec![
        RoleRelation::new(roles[0], RelationId::new(1), roles[1]),
        RoleRelation::new(roles[1], RelationId::new(2), roles[2]),
    ];
    let template = RelationalTemplate::new(base, relations).expect("fixed relations are valid");
    let bindings = novel_identity_bindings(&roles, case_id);

    let resolve = |role: RoleId| {
        bindings
            .iter()
            .find(|binding| binding.role() == role)
            .expect("all fixed roles are bound")
            .entity()
    };
    let expected = vec![
        ExpectedRelation {
            left: resolve(roles[0]),
            relation: RelationId::new(1),
            right: resolve(roles[1]),
        },
        ExpectedRelation {
            left: resolve(roles[1]),
            relation: RelationId::new(2),
            right: resolve(roles[2]),
        },
    ];

    AnalogyCase {
        template,
        bindings,
        expected,
    }
}

#[cfg(test)]
mod tests {
    use super::analogy_case;

    #[test]
    fn source_structure_is_stable_while_entities_change() {
        let first = analogy_case(1);
        let second = analogy_case(2);
        assert_eq!(first.template.relations(), second.template.relations());
        assert_ne!(first.bindings, second.bindings);
        assert_ne!(first.expected, second.expected);
    }
}
