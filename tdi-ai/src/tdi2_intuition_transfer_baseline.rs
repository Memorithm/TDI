//! No-transfer control for TDI-2.1 analogical experiments.

use super::tdi2_intuition_relations::RelationalTemplate;
use super::tdi2_intuition_transfer::EntityId;
use super::tdi2_intuition_transfer_eval::ExpectedRelation;

/// Copy abstract role identities directly into the concrete entity namespace.
///
/// This deliberately performs no adaptation and therefore acts as the matched
/// no-transfer control on novel-identity tasks.
#[must_use]
pub fn literal_role_identity_baseline(template: &RelationalTemplate) -> Vec<ExpectedRelation> {
    template
        .relations()
        .iter()
        .map(|relation| ExpectedRelation {
            left: EntityId::new(u32::from(relation.left().raw())),
            relation: relation.relation(),
            right: EntityId::new(u32::from(relation.right().raw())),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::literal_role_identity_baseline;
    use crate::experimental::tdi2_intuition_analogy_tasks::analogy_case;

    #[test]
    fn no_transfer_control_does_not_match_novel_entity_target() {
        let case = analogy_case(11);
        let baseline = literal_role_identity_baseline(&case.template);
        assert_ne!(baseline, case.expected);
    }
}
