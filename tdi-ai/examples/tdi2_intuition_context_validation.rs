//! Frozen non-final Validation for reusable context-conditioned templates.
use tdi_ai::experimental::tdi2_intuition_campaign::{
    CampaignDomain, CampaignResultArtifact, frozen_context_manifest, run_synthetic_campaign,
    transferable_context_cases,
};
use tdi_ai::experimental::tdi2_intuition_engine::IntuitionEngine;
use tdi_ai::experimental::tdi2_intuition_experience::context_experience_store;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = frozen_context_manifest(CampaignDomain::Validation);
    let cases = transferable_context_cases(manifest.start_id, manifest.count)?;
    let store = context_experience_store(4);
    let engine = IntuitionEngine::default();
    let summary = run_synthetic_campaign(manifest.domain, &cases, |case| {
        engine
            .run(&store, case.query())
            .expect("valid evidence")
            .outcome()
    });
    assert_eq!(summary.correct(), 52);
    println!(
        "{}",
        CampaignResultArtifact::new(manifest, summary).canonical_record()
    );
    Ok(())
}
