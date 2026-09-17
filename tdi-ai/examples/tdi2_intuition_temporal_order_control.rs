//! Matched control reversing temporal frame order without changing frame contents.
use tdi_ai::experimental::tdi2_intuition_temporal::BooleanSequence;
use tdi_ai::experimental::tdi2_intuition_temporal_transfer::{
    TEMPORAL_TRANSFER_DEVELOPMENT_START, TEMPORAL_TRANSFER_DOMAIN_CASES,
    select_temporal_experience, temporal_experience_library, transferable_temporal_case,
};
use tdi_ai::experimental::tdi2_intuition_weight::ExperienceWeightPolicy;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let experiences = temporal_experience_library(4);
    let policy = ExperienceWeightPolicy::default();
    let mut original_correct = 0_u64;
    let mut reversed_correct = 0_u64;
    for offset in 0..TEMPORAL_TRANSFER_DOMAIN_CASES {
        let case = transferable_temporal_case(TEMPORAL_TRANSFER_DEVELOPMENT_START + offset as u32);
        if select_temporal_experience(&experiences, case.query(), policy)?
            .is_some_and(|s| s.template_id == case.expected_template())
        {
            original_correct += 1;
        }
        let reversed = BooleanSequence::new(case.query().frames().iter().cloned().rev().collect());
        if select_temporal_experience(&experiences, &reversed, policy)?
            .is_some_and(|s| s.template_id == case.expected_template())
        {
            reversed_correct += 1;
        }
    }
    assert_eq!(original_correct, TEMPORAL_TRANSFER_DOMAIN_CASES as u64);
    assert_eq!(reversed_correct, 0);
    println!(
        "tdi2.1-temporal-order-control-v1;cases={};original_correct={original_correct};reversed_correct={reversed_correct}",
        TEMPORAL_TRANSFER_DOMAIN_CASES
    );
    Ok(())
}
