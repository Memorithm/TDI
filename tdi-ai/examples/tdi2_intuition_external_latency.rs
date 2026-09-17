//! Wall-clock latency observations kept strictly outside the intuition algorithm.
use std::time::Duration;
use tdi_ai::experimental::tdi2_intuition_cost::measure_repeated_external;
use tdi_ai::experimental::tdi2_intuition_engine::IntuitionEngine;
use tdi_ai::experimental::tdi2_intuition_experience::motif_experience_store;
use tdi_ai::experimental::tdi2_intuition_tasks::motif_retrieval_case;

fn median_ns(mut samples: Vec<Duration>) -> u128 {
    samples.sort_unstable();
    samples[samples.len() / 2].as_nanos()
}

fn main() {
    const REPEATS: usize = 2_000;
    let case = motif_retrieval_case(1_000);
    let engine = IntuitionEngine::default();
    let support_one = motif_experience_store(1);
    let support_sixty_four = motif_experience_store(64);
    let one = measure_repeated_external(REPEATS, || {
        engine
            .run(&support_one, case.query())
            .expect("valid run")
            .outcome()
    });
    let sixty_four = measure_repeated_external(REPEATS, || {
        engine
            .run(&support_sixty_four, case.query())
            .expect("valid run")
            .outcome()
    });
    assert_eq!(one.len(), REPEATS);
    assert_eq!(sixty_four.len(), REPEATS);
    println!(
        "tdi2.1-external-latency-v1;repeats={REPEATS};support1_median_ns={};support64_median_ns={}",
        median_ns(one),
        median_ns(sixty_four)
    );
}
