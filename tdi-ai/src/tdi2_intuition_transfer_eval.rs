//! Exact structural-transfer evaluation for TDI-2.1.

use super::tdi2_intuition_relations::RelationId;
use super::tdi2_intuition_transfer::{EntityId, EntityRelation};

/// Expected concrete relation in a novel target situation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExpectedRelation {
    /// Expected source entity.
    pub left: EntityId,
    /// Expected relation identity.
    pub relation: RelationId,
    /// Expected destination entity.
    pub right: EntityId,
}

/// Exact structural-transfer accounting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TransferEvaluation {
    /// Expected target relations.
    pub expected: usize,
    /// Exact expected relations reproduced by transfer.
    pub matched: usize,
    /// Expected relations not reproduced.
    pub missing: usize,
    /// Produced relations absent from the target specification.
    pub unexpected: usize,
}

impl TransferEvaluation {
    /// Whether transfer exactly reproduced the target relation set.
    #[must_use]
    pub const fn is_exact(self) -> bool {
        self.missing == 0 && self.unexpected == 0
    }
}

/// Compare a transferred relation set with a frozen target relation set.
#[must_use]
pub fn evaluate_transfer(
    actual: &[EntityRelation],
    expected: &[ExpectedRelation],
) -> TransferEvaluation {
    let mut expected_keys = expected
        .iter()
        .map(|relation| (relation.left.raw(), relation.relation.raw(), relation.right.raw()))
        .collect::<Vec<_>>();
    expected_keys.sort_unstable();
    expected_keys.dedup();

    let mut actual_keys = actual
        .iter()
        .map(|relation| {
            (
                relation.left().raw(),
                relation.relation().raw(),
                relation.right().raw(),
            )
        })
        .collect::<Vec<_>>();
    actual_keys.sort_unstable();
    actual_keys.dedup();

    let matched = expected_keys
        .iter()
        .filter(|key| actual_keys.binary_search(key).is_ok())
        .count();
    let unexpected = actual_keys
        .iter()
        .filter(|key| expected_keys.binary_search(key).is_err())
        .count();

    TransferEvaluation {
        expected: expected_keys.len(),
        matched,
        missing: expected_keys.len() - matched,
        unexpected,
    }
}

#[cfg(test)]
mod tests {
    use super::{ExpectedRelation, evaluate_transfer};
    use crate::experimental::tdi2_intuition::{PredicateId, RoleId, Template, TemplateId};
    use crate::experimental::tdi2_intuition_relations::{RelationId, RelationalTemplate, RoleRelation};
    use crate::experimental::tdi2_intuition_transfer::{EntityId, RoleBinding, RoleMap, transfer_relations};

    #[test]
    fn exact_role_transfer_scores_exactly() {
        let base = Template::new(
            TemplateId::new(1),
            vec![PredicateId::new(1)],
            Vec::new(),
            vec![RoleId::new(1), RoleId::new(2)],
        )
        .expect("template");
        let template = RelationalTemplate::new(
            base,
            vec![RoleRelation::new(RoleId::new(1), RelationId::new(7), RoleId::new(2))],
        )
        .expect("relation");
        let map = RoleMap::for_template(
            &template,
            vec![
                RoleBinding::new(RoleId::new(1), EntityId::new(10)),
                RoleBinding::new(RoleId::new(2), EntityId::new(20)),
            ],
        )
        .expect("map");
        let actual = transfer_relations(&template, &map);
        let score = evaluate_transfer(
            &actual,
            &[ExpectedRelation { left: EntityId::new(10), relation: RelationId::new(7), right: EntityId::new(20) }],
        );
        assert!(score.is_exact());
    }
}
