//! Development diagnostic for simultaneous applicability of two experiences.
use tdi_ai::experimental::tdi2_intuition_engine::IntuitionEngine;
use tdi_ai::experimental::tdi2_intuition_experience::motif_experience_store;
use tdi_ai::experimental::tdi2_intuition_margin::top_two_margin;
use tdi_ai::experimental::tdi2_intuition_novelty::diagnose_novelty;
use tdi_ai::experimental::tdi2_intuition_tasks::motif_ambiguous_state;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store = motif_experience_store(4);
    let engine = IntuitionEngine::default();
    let state = motif_ambiguous_state(0, 1).expect("two distinct known motifs");
    let novelty = diagnose_novelty(&store, &state);
    let report = engine.run(&store, &state)?;
    let margin = top_two_margin(report.candidates())?.expect("two applicable experiences");
    assert_eq!(novelty.exact_applicable, 2);
    assert_eq!(report.candidates().len(), 2);
    assert_eq!(margin.absolute, 0.0);
    println!(
        "tdi2.1-ambiguity-v1;exact_applicable=2;candidates=2;margin_bits={:016x}",
        margin.absolute.to_bits()
    );
    Ok(())
}
