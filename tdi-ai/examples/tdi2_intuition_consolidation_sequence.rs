//! Explicit post-outcome consolidation sequence.
use tdi_ai::experimental::tdi2_intuition::TemplateId;
use tdi_ai::experimental::tdi2_intuition_consolidation::{
    ValidationOutcome, consolidate_validation,
};
use tdi_ai::experimental::tdi2_intuition_reliability::ReliabilityEvidence;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let id = TemplateId::new(900_003);
    let mut evidence = ReliabilityEvidence::new(2, 1);
    for outcome in [
        ValidationOutcome::Confirmed,
        ValidationOutcome::Confirmed,
        ValidationOutcome::Refuted,
    ] {
        let update = consolidate_validation(id, evidence, outcome)?;
        println!("{}", update.canonical_record());
        evidence = update.after();
    }
    assert_eq!(evidence, ReliabilityEvidence::new(4, 2));
    println!(
        "tdi2.1-consolidation-sequence-v1;final_successes={};final_failures={};support={}",
        evidence.successes(),
        evidence.failures(),
        evidence.support()
    );
    Ok(())
}
