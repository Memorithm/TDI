//! Development evaluation of structural transfer to novel entity identities.
use tdi_ai::experimental::tdi2_intuition_campaign::{
    CampaignDomain, analogy_cases, frozen_analogy_manifest,
};
use tdi_ai::experimental::tdi2_intuition_transfer_baseline::evaluate_transfer_ablation;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = frozen_analogy_manifest(CampaignDomain::Development);
    let cases = analogy_cases(manifest.start_id, manifest.count)?;
    let mut transfer_exact = 0_u64;
    let mut no_transfer_exact = 0_u64;
    for case in &cases {
        let result = evaluate_transfer_ablation(case)?;
        transfer_exact += u64::from(result.transfer_exact);
        no_transfer_exact += u64::from(result.no_transfer_exact);
    }
    assert_eq!(transfer_exact, 32);
    assert_eq!(no_transfer_exact, 0);
    println!(
        "tdi2.1-analogy-development-v1;cases=32;transfer_exact={transfer_exact};no_transfer_exact={no_transfer_exact}"
    );
    Ok(())
}
