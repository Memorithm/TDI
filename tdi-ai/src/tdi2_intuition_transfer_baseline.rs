//! No-transfer control for TDI-2.1 analogical experiments.

use super::tdi2_intuition_analogy_tasks::AnalogyCase;
use super::tdi2_intuition_relations::RelationalTemplate;
use super::tdi2_intuition_transfer::{EntityId, RoleMap, TransferError, transfer_relations};
use super::tdi2_intuition_transfer_eval::{ExpectedRelation, evaluate_transfer};

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

/// Paired exactness result for transfer enabled versus transfer ablated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TransferAblationResult {
    /// Whether role-preserving transfer exactly reproduced the target relation set.
    pub transfer_exact: bool,
    /// Whether literal role identity exactly reproduced the same target relation set.
    pub no_transfer_exact: bool,
}

fn canonical_expected(mut relations: Vec<ExpectedRelation>) -> Vec<ExpectedRelation> {
    relations.sort_unstable();
    relations.dedup();
    relations
}

/// Compare structural transfer against the literal-role control on one frozen analogy case.
pub fn evaluate_transfer_ablation(
    case: &AnalogyCase,
) -> Result<TransferAblationResult, TransferError> {
    let role_map = RoleMap::for_template(&case.template, case.bindings.clone())?;
    let transferred = transfer_relations(&case.template, &role_map);
    let transfer_exact = evaluate_transfer(&transferred, &case.expected).is_exact();
    let no_transfer_exact = canonical_expected(literal_role_identity_baseline(&case.template))
        == canonical_expected(case.expected.clone());
    Ok(TransferAblationResult {
        transfer_exact,
        no_transfer_exact,
    })
}

#[cfg(test)]
mod tests {
    use super::{evaluate_transfer_ablation, literal_role_identity_baseline};
    use crate::experimental::tdi2_intuition_analogy_tasks::analogy_case;

    #[test]
    fn no_transfer_control_does_not_match_novel_entity_target() {
        let case = analogy_case(11);
        let baseline = literal_role_identity_baseline(&case.template);
        assert_ne!(baseline, case.expected);
    }

    #[test]
    fn paired_ablation_attributes_novel_identity_success_to_transfer() {
        let case = analogy_case(11);
        let result = evaluate_transfer_ablation(&case).expect("valid role map");
        assert!(result.transfer_exact);
        assert!(!result.no_transfer_exact);
    }
}
