//! Executable non-final TDI-2.1 Development campaign.
//!
//! The store is built from the frozen motif-class rules before evaluation.
//! Cases use later ids, so their distractor predicates were never stored.
//! This is Development evidence only, not a protected holdout or final result.

use tdi_ai::experimental::tdi2_intuition_campaign::{
    CampaignDomain, CampaignFamily, CampaignManifest, CampaignResultArtifact, motif_cases,
    run_synthetic_campaign,
};
use tdi_ai::experimental::tdi2_intuition_engine::IntuitionEngine;
use tdi_ai::experimental::tdi2_intuition_experience::motif_experience_store;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = CampaignManifest::new(
        CampaignDomain::Development,
        CampaignFamily::MotifRetrieval,
        1_000,
        34,
        0,
        0.0,
    )?;
    let store = motif_experience_store(4);
    let engine = IntuitionEngine::default();
    let cases = motif_cases(manifest.start_id, manifest.count)?;
    let summary = run_synthetic_campaign(manifest.domain, &cases, |case| {
        engine
            .run(&store, case.query())
            .expect("fixed development evidence is valid")
            .outcome()
    });
    let artifact = CampaignResultArtifact::new(manifest, summary);
    println!("{}", artifact.canonical_record());
    Ok(())
}
