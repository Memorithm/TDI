//! Frozen non-final TDI-2.1 Validation campaign for motif retrieval.
//!
//! The experience store and inference policy are identical to Development.
//! No parameter is selected from Validation outcomes. This is not PrimaryHoldout.

use tdi_ai::experimental::tdi2_intuition_campaign::{
    CampaignDomain, CampaignResultArtifact, frozen_motif_manifest, motif_cases,
    run_synthetic_campaign,
};
use tdi_ai::experimental::tdi2_intuition_engine::IntuitionEngine;
use tdi_ai::experimental::tdi2_intuition_experience::motif_experience_store;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = frozen_motif_manifest(CampaignDomain::Validation);
    let store = motif_experience_store(4);
    let engine = IntuitionEngine::default();
    let cases = motif_cases(manifest.start_id, manifest.count)?;
    let summary = run_synthetic_campaign(manifest.domain, &cases, |case| {
        engine
            .run(&store, case.query())
            .expect("frozen validation evidence is valid")
            .outcome()
    });
    println!(
        "{}",
        CampaignResultArtifact::new(manifest, summary).canonical_record()
    );
    Ok(())
}
