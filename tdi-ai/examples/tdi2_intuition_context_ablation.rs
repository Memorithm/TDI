//! Paired Development ablation of context predicates.
use tdi_ai::experimental::tdi2_intuition::{BooleanState, PredicateId};
use tdi_ai::experimental::tdi2_intuition_ablations::remove_context;
use tdi_ai::experimental::tdi2_intuition_campaign::{
    CampaignDomain, frozen_context_manifest, run_paired_campaign, transferable_context_cases,
};
use tdi_ai::experimental::tdi2_intuition_engine::IntuitionEngine;
use tdi_ai::experimental::tdi2_intuition_experience::context_experience_store;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = frozen_context_manifest(CampaignDomain::Development);
    let cases = transferable_context_cases(manifest.start_id, manifest.count)?;
    let store = context_experience_store(4);
    let engine = IntuitionEngine::default();
    let contexts = [PredicateId::new(30_000), PredicateId::new(30_001)];
    let comparison = run_paired_campaign(
        manifest.domain,
        &cases,
        |case| {
            engine
                .run(&store, case.query())
                .expect("valid evidence")
                .outcome()
        },
        |case| {
            let ablated: BooleanState = remove_context(case.query(), &contexts);
            engine
                .run(&store, &ablated)
                .expect("valid ablation")
                .outcome()
        },
    );
    assert_eq!(comparison.intuition_only, 52);
    assert_eq!(comparison.baseline_only, 0);
    println!("{}", comparison.canonical_record());
    Ok(())
}
