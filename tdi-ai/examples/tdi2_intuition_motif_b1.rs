//! Paired Development comparison against B1: deterministic invalid experience.
use tdi_ai::experimental::tdi2_intuition_baselines::invalid_experience_target;
use tdi_ai::experimental::tdi2_intuition_campaign::{
    CampaignDomain, frozen_motif_manifest, motif_cases, run_paired_campaign,
};
use tdi_ai::experimental::tdi2_intuition_engine::IntuitionEngine;
use tdi_ai::experimental::tdi2_intuition_experience::motif_experience_store;
use tdi_ai::experimental::tdi2_intuition_inference::IntuitionOutcome;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = frozen_motif_manifest(CampaignDomain::Development);
    let cases = motif_cases(manifest.start_id, manifest.count)?;
    let store = motif_experience_store(4);
    let engine = IntuitionEngine::default();
    let comparison = run_paired_campaign(
        manifest.domain,
        &cases,
        |case| {
            engine
                .run(&store, case.query())
                .expect("valid evidence")
                .outcome()
        },
        |case| IntuitionOutcome::Selected {
            template_id: invalid_experience_target(case),
            weight: 0.0,
            support: 0,
        },
    );
    println!("{}", comparison.canonical_record());
    Ok(())
}
