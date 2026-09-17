//! Development OOD test: situations contain no known motif predicate.
use tdi_ai::experimental::tdi2_intuition_engine::IntuitionEngine;
use tdi_ai::experimental::tdi2_intuition_experience::motif_experience_store;
use tdi_ai::experimental::tdi2_intuition_inference::IntuitionOutcome;
use tdi_ai::experimental::tdi2_intuition_novelty::diagnose_novelty;
use tdi_ai::experimental::tdi2_intuition_tasks::motif_ood_state;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = motif_experience_store(4);
    let engine = IntuitionEngine::default();
    let mut abstained = 0_u64;
    for id in 0..34 {
        let state = motif_ood_state(id, 8);
        let novelty = diagnose_novelty(&store, &state);
        assert!(novelty.is_uncovered());
        if matches!(
            engine.run(&store, &state)?.outcome(),
            IntuitionOutcome::InsufficientExperience
        ) {
            abstained += 1;
        }
    }
    assert_eq!(abstained, 34);
    println!("tdi2.1-ood-v1;cases=34;uncovered=34;abstained={abstained}");
    Ok(())
}
