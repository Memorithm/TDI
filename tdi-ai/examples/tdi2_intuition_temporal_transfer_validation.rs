//! Frozen non-final Validation for reusable temporal experience.
use tdi_ai::experimental::tdi2_intuition_temporal_transfer::{
    TEMPORAL_TRANSFER_DOMAIN_CASES, TEMPORAL_TRANSFER_VALIDATION_START, select_temporal_experience,
    temporal_experience_library, transferable_temporal_case,
};
use tdi_ai::experimental::tdi2_intuition_weight::ExperienceWeightPolicy;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let experiences = temporal_experience_library(4);
    let mut correct = 0_u64;
    for offset in 0..TEMPORAL_TRANSFER_DOMAIN_CASES {
        let case = transferable_temporal_case(TEMPORAL_TRANSFER_VALIDATION_START + offset as u32);
        if select_temporal_experience(
            &experiences,
            case.query(),
            ExperienceWeightPolicy::default(),
        )?
        .is_some_and(|s| s.template_id == case.expected_template())
        {
            correct += 1;
        }
    }
    assert_eq!(correct, TEMPORAL_TRANSFER_DOMAIN_CASES as u64);
    println!(
        "tdi2.1-temporal-validation-v1;cases={};correct={correct}",
        TEMPORAL_TRANSFER_DOMAIN_CASES
    );
    Ok(())
}
