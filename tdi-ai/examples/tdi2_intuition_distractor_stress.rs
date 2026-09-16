//! Development stress test for irrelevant Boolean predicates.
use tdi_ai::experimental::tdi2_intuition_engine::IntuitionEngine;
use tdi_ai::experimental::tdi2_intuition_experience::motif_experience_store;
use tdi_ai::experimental::tdi2_intuition_inference::IntuitionOutcome;
use tdi_ai::experimental::tdi2_intuition_tasks::motif_retrieval_stress_case;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = motif_experience_store(4);
    let engine = IntuitionEngine::default();
    for distractors in [0_u16, 1, 8, 64, 256] {
        let mut correct = 0_u64;
        for id in 1_000..1_034 {
            let case = motif_retrieval_stress_case(id, distractors);
            let outcome = engine.run(&store, case.query())?.outcome();
            if matches!(outcome, IntuitionOutcome::Selected { template_id, .. } if template_id == case.expected_template())
            {
                correct += 1;
            }
        }
        assert_eq!(correct, 34);
        println!(
            "tdi2.1-distractor-stress-v1;distractors={distractors};correct={correct};total=34"
        );
    }
    Ok(())
}
