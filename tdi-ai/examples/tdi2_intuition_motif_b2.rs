//! Paired TDI-2.1 Development comparison against B2 nearest-Boolean.
//!
//! A tie is an informative negative result: the simple motif task does not by
//! itself demonstrate a benefit from experience weighting over crisp overlap.

use tdi_ai::experimental::tdi2_intuition_baselines::nearest_boolean_baseline;
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
        |case| match nearest_boolean_baseline(&store, case.query()) {
            Some(result) => IntuitionOutcome::Selected {
                template_id: result.template_id(),
                weight: result.score(),
                support: 0,
            },
            None => IntuitionOutcome::InsufficientExperience,
        },
    );
    println!("{}", comparison.canonical_record());
    Ok(())
}
