//! Relation ablation control for analogical transfer.
use tdi_ai::experimental::tdi2_intuition_ablations::remove_relations;
use tdi_ai::experimental::tdi2_intuition_campaign::{
    CampaignDomain, analogy_cases, frozen_analogy_manifest,
};
use tdi_ai::experimental::tdi2_intuition_transfer::{RoleMap, transfer_relations};
use tdi_ai::experimental::tdi2_intuition_transfer_eval::evaluate_transfer;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = frozen_analogy_manifest(CampaignDomain::Development);
    let cases = analogy_cases(manifest.start_id, manifest.count)?;
    let mut full_exact = 0_u64;
    let mut relation_ablated_exact = 0_u64;
    for case in &cases {
        let map = RoleMap::for_template(&case.template, case.bindings.clone())?;
        full_exact += u64::from(
            evaluate_transfer(&transfer_relations(&case.template, &map), &case.expected).is_exact(),
        );
        let ablated = remove_relations(&case.template)?;
        let ablated_map = RoleMap::for_template(&ablated, case.bindings.clone())?;
        relation_ablated_exact += u64::from(
            evaluate_transfer(&transfer_relations(&ablated, &ablated_map), &case.expected)
                .is_exact(),
        );
    }
    assert_eq!(full_exact, 32);
    assert_eq!(relation_ablated_exact, 0);
    println!(
        "tdi2.1-analogy-relation-ablation-v1;cases=32;full_exact={full_exact};relation_ablated_exact={relation_ablated_exact}"
    );
    Ok(())
}
