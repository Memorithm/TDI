//! Exact replay check for deterministic non-final TDI-2.1 artifacts.
use tdi_ai::experimental::tdi2_intuition_campaign::{
    CampaignDomain, CampaignResultArtifact, frozen_context_manifest, frozen_motif_manifest,
    motif_cases, run_synthetic_campaign, transferable_context_cases,
};
use tdi_ai::experimental::tdi2_intuition_engine::IntuitionEngine;
use tdi_ai::experimental::tdi2_intuition_experience::{
    context_experience_store, motif_experience_store,
};

fn motif_record() -> Result<String, Box<dyn std::error::Error>> {
    let manifest = frozen_motif_manifest(CampaignDomain::Development);
    let cases = motif_cases(manifest.start_id, manifest.count)?;
    let store = motif_experience_store(4);
    let engine = IntuitionEngine::default();
    let summary = run_synthetic_campaign(manifest.domain, &cases, |case| {
        engine.run(&store, case.query()).unwrap().outcome()
    });
    Ok(CampaignResultArtifact::new(manifest, summary).canonical_record())
}

fn context_record() -> Result<String, Box<dyn std::error::Error>> {
    let manifest = frozen_context_manifest(CampaignDomain::Development);
    let cases = transferable_context_cases(manifest.start_id, manifest.count)?;
    let store = context_experience_store(4);
    let engine = IntuitionEngine::default();
    let summary = run_synthetic_campaign(manifest.domain, &cases, |case| {
        engine.run(&store, case.query()).unwrap().outcome()
    });
    Ok(CampaignResultArtifact::new(manifest, summary).canonical_record())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let motif_a = motif_record()?;
    let motif_b = motif_record()?;
    let context_a = context_record()?;
    let context_b = context_record()?;
    assert_eq!(motif_a, motif_b);
    assert_eq!(context_a, context_b);
    println!("tdi2.1-deterministic-replay-v1;motif_equal=true;context_equal=true");
    Ok(())
}
