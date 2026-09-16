//! Executable non-final TDI-2.1 campaign smoke test.
//!
//! This example intentionally runs the no-experience abstention control. It is
//! infrastructure validation, not a scientific result and not a holdout run.

use tdi_ai::experimental::tdi2_intuition_campaign::{
    CampaignDomain, CampaignFamily, CampaignManifest, CampaignResultArtifact, motif_cases,
    run_synthetic_campaign,
};
use tdi_ai::experimental::tdi2_intuition_inference::IntuitionOutcome;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = CampaignManifest::new(
        CampaignDomain::Development,
        CampaignFamily::MotifRetrieval,
        0,
        8,
        0,
        0.0,
    )?;
    let cases = motif_cases(manifest.start_id, manifest.count)?;
    let summary = run_synthetic_campaign(manifest.domain, &cases, |_| {
        IntuitionOutcome::InsufficientExperience
    });
    let artifact = CampaignResultArtifact::new(manifest, summary);
    println!("{}", artifact.canonical_record());
    Ok(())
}
