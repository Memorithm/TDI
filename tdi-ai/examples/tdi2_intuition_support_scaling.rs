//! Development-only support scaling for TDI-2.1 motif experience.
use tdi_ai::experimental::tdi2_intuition_campaign::{
    CampaignDomain, frozen_motif_manifest, motif_cases, run_synthetic_campaign,
};
use tdi_ai::experimental::tdi2_intuition_engine::IntuitionEngine;
use tdi_ai::experimental::tdi2_intuition_experience::motif_experience_store;
use tdi_ai::experimental::tdi2_intuition_scaling::support_schedule;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = frozen_motif_manifest(CampaignDomain::Development);
    let cases = motif_cases(manifest.start_id, manifest.count)?;
    let engine = IntuitionEngine::default();
    for support in support_schedule(8) {
        let store = motif_experience_store(support);
        let summary = run_synthetic_campaign(manifest.domain, &cases, |case| {
            engine
                .run(&store, case.query())
                .expect("valid evidence")
                .outcome()
        });
        if support == 0 {
            assert_eq!(summary.abstained(), manifest.count as u64);
        } else {
            assert_eq!(summary.correct(), manifest.count as u64);
        }
        println!(
            "tdi2.1-support-scaling-v1;support={support};total={};selected={};correct={};abstained={}",
            summary.total(),
            summary.selected(),
            summary.correct(),
            summary.abstained()
        );
    }
    Ok(())
}
