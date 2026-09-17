//! External abstention-threshold sweep over fixed TDI-2.1 experience.
use tdi_ai::experimental::tdi2_intuition_campaign::{
    CampaignDomain, frozen_motif_manifest, motif_cases, run_synthetic_campaign,
};
use tdi_ai::experimental::tdi2_intuition_engine::IntuitionEngine;
use tdi_ai::experimental::tdi2_intuition_experience::motif_experience_store;
use tdi_ai::experimental::tdi2_intuition_inference::InferencePolicy;
use tdi_ai::experimental::tdi2_intuition_scaling::threshold_schedule;
use tdi_ai::experimental::tdi2_intuition_weight::ExperienceWeightPolicy;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = frozen_motif_manifest(CampaignDomain::Development);
    let cases = motif_cases(manifest.start_id, manifest.count)?;
    let store = motif_experience_store(4);
    let thresholds = threshold_schedule(2.0, 4)?;
    for threshold in thresholds {
        let engine = IntuitionEngine::new(
            ExperienceWeightPolicy::default(),
            InferencePolicy::new(threshold)?,
        );
        let summary = run_synthetic_campaign(manifest.domain, &cases, |case| {
            engine
                .run(&store, case.query())
                .expect("valid evidence")
                .outcome()
        });
        println!(
            "tdi2.1-threshold-sweep-v1;threshold_bits={:016x};selected={};correct={};abstained={}",
            threshold.to_bits(),
            summary.selected(),
            summary.correct(),
            summary.abstained()
        );
    }
    Ok(())
}
