//! Non-final calibration diagnostic over explicit reliability evidence.
use tdi_ai::experimental::tdi2_intuition_calibration::{
    brier_score, calibration_observation_from_evidence, expected_calibration_error,
};
use tdi_ai::experimental::tdi2_intuition_reliability::ReliabilityEvidence;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let observations = [
        calibration_observation_from_evidence(ReliabilityEvidence::new(8, 2), 1.0, 1.0, true)?,
        calibration_observation_from_evidence(ReliabilityEvidence::new(2, 8), 1.0, 1.0, false)?,
        calibration_observation_from_evidence(ReliabilityEvidence::new(5, 5), 1.0, 1.0, true)?,
        calibration_observation_from_evidence(ReliabilityEvidence::new(5, 5), 1.0, 1.0, false)?,
    ];
    let ece = expected_calibration_error(&observations, 4)?.expect("non-empty observations");
    let brier = brier_score(&observations)?.expect("non-empty observations");
    assert!((ece - 0.125).abs() < 1e-12);
    assert!((brier - 0.15625).abs() < 1e-12);
    println!(
        "tdi2.1-calibration-diagnostic-v1;observations=4;ece_bits={:016x};brier_bits={:016x}",
        ece.to_bits(),
        brier.to_bits()
    );
    Ok(())
}
