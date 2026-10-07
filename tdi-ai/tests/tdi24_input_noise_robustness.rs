#![cfg(feature = "experimental")]

//! TDI-24 slice 42: input-noise robustness (Phase E).
//!
//! Declared deterministic perturbation families at declared amplitudes,
//! applied identically to C6 and its direct-only arm; label-free decision
//! stability. No training, no protected/final access, no scientific claim.

use tdi_ai::experimental::tdi24_chiral::Chiral6;
use tdi_ai::experimental::tdi24_eval::{
    EvalError, INPUT_NOISE_ROBUSTNESS_CONTRACT, InputNoiseRobustnessReport, NOISE_AMPLITUDES,
    NOISE_ARMS, NOISE_FAMILIES, NoiseFamily, SequenceArm, StageCPreflightBudget, TrainableCapacity,
    input_noise_seed, perturb_operand, run_input_noise_robustness,
    run_input_noise_robustness_for_label, validate_input_noise_robustness_report,
};
use tdi_ai::experimental::tdi24_tasks::DataSplit;

fn study(split: DataSplit) -> InputNoiseRobustnessReport {
    run_input_noise_robustness(split, StageCPreflightBudget::bounded(4, 0)).unwrap()
}

fn rejection(report: &InputNoiseRobustnessReport) -> &'static str {
    match validate_input_noise_robustness_report(report) {
        Err(EvalError::InputNoiseRobustnessInvalid { reason }) => reason,
        other => panic!("expected a noise-robustness rejection, got {other:?}"),
    }
}

#[test]
fn grid_is_declared() {
    assert_eq!(
        INPUT_NOISE_ROBUSTNESS_CONTRACT,
        "tdi24-input-noise-robustness-v1"
    );
    assert_eq!(
        NOISE_FAMILIES,
        [
            NoiseFamily::Isotropic,
            NoiseFamily::EvenOnly,
            NoiseFamily::OddOnly
        ]
    );
    assert_eq!(NOISE_AMPLITUDES, [1e-3, 1e-2, 1e-1]);
    assert_eq!(NOISE_ARMS, [SequenceArm::C6, SequenceArm::DirectOnly]);
}

#[test]
fn perturbations_are_bounded_masked_and_deterministic() {
    let value = Chiral6::new([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]).unwrap();
    for amplitude in NOISE_AMPLITUDES {
        let even = perturb_operand(value, NoiseFamily::EvenOnly, amplitude, 7, 1).unwrap();
        assert_eq!(even.odd(), value.odd());
        let odd = perturb_operand(value, NoiseFamily::OddOnly, amplitude, 7, 1).unwrap();
        assert_eq!(odd.even(), value.even());
        let iso = perturb_operand(value, NoiseFamily::Isotropic, amplitude, 7, 1).unwrap();
        for (a, b) in iso.as_array().iter().zip(value.as_array()) {
            assert!((a - b).abs() <= amplitude);
        }
        assert_eq!(
            iso,
            perturb_operand(value, NoiseFamily::Isotropic, amplitude, 7, 1).unwrap()
        );
    }
    assert_eq!(
        perturb_operand(value, NoiseFamily::Isotropic, 0.5, 0, 0),
        Err(EvalError::InputNoiseRobustnessInvalid {
            reason: "amplitude_not_registered"
        })
    );
}

#[test]
fn smoke_study_covers_the_full_grid_with_paired_counts() {
    for split in [DataSplit::Development, DataSplit::Validation] {
        let report = study(split);
        assert_eq!(report.noise_seed, input_noise_seed());
        assert_eq!(report.capacity, TrainableCapacity::reference_c6());
        assert_eq!(report.cells.len(), 3 * 3 * 2 * 4);
        for pair in report.cells.chunks(8) {
            for index in 0..4 {
                assert_eq!(pair[index].arm, SequenceArm::C6);
                assert_eq!(pair[index + 4].arm, SequenceArm::DirectOnly);
                assert_eq!(pair[index].n_cases, pair[index + 4].n_cases);
            }
        }
        assert!(!report.protected_or_final_access);
        assert!(!report.training_executed);
        assert!(!report.scientific_claim);
        assert!(report.experimental_non_final);
    }
}

#[test]
fn tampered_reports_fail_closed() {
    let report = study(DataSplit::Development);
    let tamper = |mutate: &dyn Fn(&mut InputNoiseRobustnessReport), reason: &str| {
        let mut tampered = report.clone();
        mutate(&mut tampered);
        assert_eq!(rejection(&tampered), reason);
    };
    tamper(&|r| r.robustness_contract = "legacy", "contract_drift");
    tamper(&|r| r.noise_seed ^= 1, "seed_drift");
    tamper(
        &|r| r.capacity = TrainableCapacity::reference_v6(),
        "capacity_mismatch",
    );
    tamper(
        &|r| {
            r.cells.pop();
        },
        "cell_count",
    );
    tamper(&|r| r.cells[0].amplitude = 0.5, "grid_drift");
    tamper(&|r| r.cells.swap(0, 4), "grid_drift");
    tamper(&|r| r.cells[0].n_cases += 1, "paired_count_drift");
    tamper(
        &|r| r.cells[0].sign_stable = r.cells[0].n_cases + 1,
        "paired_count_drift",
    );
    tamper(
        &|r| r.cells[0].max_abs_score_change = f64::NAN,
        "score_change_drift",
    );
    tamper(
        &|r| r.cells[0].max_abs_score_change += 1.0,
        "case_evidence_drift",
    );
    tamper(
        &|r| r.protected_or_final_access = true,
        "protected_or_final_access",
    );
    tamper(&|r| r.training_executed = true, "training_executed");
    tamper(&|r| r.scientific_claim = true, "scientific_claim");
    tamper(
        &|r| r.experimental_non_final = false,
        "experimental_non_final",
    );
}

#[test]
fn protected_or_final_labels_and_bad_budgets_never_score() {
    for label in ["protected", "final", "holdout", ""] {
        assert_eq!(
            run_input_noise_robustness_for_label(label, StageCPreflightBudget::bounded(1, 0)),
            Err(EvalError::ProtectedOrFinalSplit)
        );
    }
    assert!(
        run_input_noise_robustness(DataSplit::Development, StageCPreflightBudget::bounded(0, 0))
            .is_err()
    );
}

#[test]
fn study_is_deterministic() {
    assert_eq!(study(DataSplit::Validation), study(DataSplit::Validation));
}
