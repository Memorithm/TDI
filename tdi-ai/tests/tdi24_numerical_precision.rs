#![cfg(feature = "experimental")]

//! TDI-24 slice 44: numerical precision study (Phase E).
//!
//! f64 reference versus f32 evaluation of the paired C6 / direct-only arms on
//! the Stage-C stream and the slice-43 reflection adversarial set, under a
//! declared forward-error bound with complete failure accounting. No
//! training, no protected/final access, no scientific claim.

use tdi_ai::experimental::tdi24_chiral::{Chiral6, chiral_score};
use tdi_ai::experimental::tdi24_eval::{
    EvalError, NUMERICAL_PRECISION_CONTRACT, NumericalPrecisionReport, PRECISION_ARMS,
    PRECISION_BOUND_FACTOR, PrecisionGroup, SequenceArm, StageCPreflightBudget, TrainableCapacity,
    chiral_score_f32, run_numerical_precision, run_numerical_precision_for_label,
    validate_numerical_precision_report,
};
use tdi_ai::experimental::tdi24_tasks::{DataSplit, TaskFamily};

fn study(split: DataSplit) -> NumericalPrecisionReport {
    run_numerical_precision(split, StageCPreflightBudget::bounded(4, 0)).unwrap()
}

fn rejection(report: &NumericalPrecisionReport) -> &'static str {
    match validate_numerical_precision_report(report) {
        Err(EvalError::NumericalPrecisionInvalid { reason }) => reason,
        other => panic!("expected a numerical-precision rejection, got {other:?}"),
    }
}

#[test]
fn contract_bound_and_arms_are_declared() {
    assert_eq!(NUMERICAL_PRECISION_CONTRACT, "tdi24-numerical-precision-v1");
    assert_eq!(PRECISION_BOUND_FACTOR, 32.0);
    assert_eq!(PRECISION_ARMS, [SequenceArm::C6, SequenceArm::DirectOnly]);
}

#[test]
fn f32_kernel_matches_f64_on_exact_inputs_and_detects_overflow() {
    let query = Chiral6::new([1.0, 2.0, -0.5], [0.25, -1.0, 3.0]).unwrap();
    let key = Chiral6::new([-2.0, 0.5, 1.0], [1.5, 0.75, -0.25]).unwrap();
    for arm in PRECISION_ARMS {
        let reference = chiral_score(query, key, arm.weights()).unwrap();
        let (single, condition) = chiral_score_f32(query, key, arm.weights());
        // Dyadic inputs: every f32 product and partial sum is exact.
        assert_eq!(single, reference);
        assert!(condition > 0.0);
    }
    let huge = Chiral6::new([1e30, 0.0, 0.0], [0.0, 0.0, 0.0]).unwrap();
    let (single, _) = chiral_score_f32(huge, huge, SequenceArm::C6.weights());
    assert!(!single.is_finite());
}

#[test]
fn smoke_study_accounts_every_case_within_the_declared_bound() {
    for split in [DataSplit::Development, DataSplit::Validation] {
        let report = study(split);
        validate_numerical_precision_report(&report).unwrap();
        assert_eq!(report.capacity, TrainableCapacity::reference_c6());
        assert_eq!(report.cells.len(), 5 * 2);
        assert_eq!(
            report.cells[0].group,
            PrecisionGroup::StageC(TaskFamily::ReflectionDiscriminative)
        );
        assert_eq!(report.cells[8].group, PrecisionGroup::ReflectionAdversarial);
        for cell in &report.cells {
            let expected = match cell.group {
                PrecisionGroup::StageC(_) => 8,
                PrecisionGroup::ReflectionAdversarial => 2 * 4 * 10,
            };
            assert_eq!(cell.n_cases, expected);
            assert_eq!(
                cell.within_tolerance + cell.tolerance_failures + cell.non_finite_f32,
                cell.n_cases
            );
            // Recorded outcome at the smoke budget: every case inside the
            // declared bound, no overflow and no decision flip.
            assert_eq!(cell.within_tolerance, cell.n_cases, "{cell:?}");
            assert_eq!(cell.non_finite_f32, 0);
            assert_eq!(cell.sign_flips, 0, "{cell:?}");
            assert!(cell.max_error_to_bound <= 1.0);
        }
        assert!(!report.protected_or_final_access);
        assert!(!report.training_executed);
        assert!(!report.scientific_claim);
        assert!(report.experimental_non_final);
    }
}

#[test]
fn tampered_reports_fail_closed() {
    let report = study(DataSplit::Validation);

    let mut drifted = report.clone();
    drifted.precision_contract = "tdi24-numerical-precision-v0";
    assert_eq!(rejection(&drifted), "contract_drift");

    let mut drifted = report.clone();
    drifted.bound_factor = 64.0;
    assert_eq!(rejection(&drifted), "tolerance_drift");

    let mut drifted = report.clone();
    drifted.capacity.trainable_parameters += 1;
    assert_eq!(rejection(&drifted), "capacity_mismatch");

    let mut drifted = report.clone();
    drifted.cells.pop();
    assert_eq!(rejection(&drifted), "cell_count");

    let mut drifted = report.clone();
    drifted.cells.swap(0, 1);
    assert_eq!(rejection(&drifted), "grid_drift");

    // A dropped case (silently removed failure) breaks the accounting.
    let mut drifted = report.clone();
    drifted.cells[0].within_tolerance -= 1;
    assert_eq!(rejection(&drifted), "failure_accounting_drift");

    let mut drifted = report.clone();
    drifted.cells[0].within_tolerance -= 1;
    drifted.cells[0].tolerance_failures += 1;
    assert_eq!(rejection(&drifted), "error_statistic_drift");

    let mut drifted = report.clone();
    drifted.cells[0].within_tolerance -= 1;
    drifted.cells[0].tolerance_failures += 1;
    drifted.cells[0].max_error_to_bound = 2.0;
    assert_eq!(rejection(&drifted), "case_evidence_drift");

    let mut drifted = report.clone();
    drifted.cells[0].max_abs_error = f64::NAN;
    assert_eq!(rejection(&drifted), "error_statistic_drift");

    for (flag, reason) in [
        (0, "protected_or_final_access"),
        (1, "training_executed"),
        (2, "scientific_claim"),
        (3, "experimental_non_final"),
    ] {
        let mut drifted = report.clone();
        match flag {
            0 => drifted.protected_or_final_access = true,
            1 => drifted.training_executed = true,
            2 => drifted.scientific_claim = true,
            _ => drifted.experimental_non_final = false,
        }
        assert_eq!(rejection(&drifted), reason);
    }
}

#[test]
fn protected_or_final_labels_and_bad_budgets_never_score() {
    for label in ["protected", "final", "holdout", ""] {
        assert_eq!(
            run_numerical_precision_for_label(label, StageCPreflightBudget::bounded(2, 0)),
            Err(EvalError::ProtectedOrFinalSplit)
        );
    }
    assert!(
        run_numerical_precision(DataSplit::Development, StageCPreflightBudget::bounded(0, 0))
            .is_err()
    );
    assert!(
        run_numerical_precision(DataSplit::Development, StageCPreflightBudget::bounded(9, 0))
            .is_err()
    );
}

#[test]
fn study_is_deterministic() {
    assert_eq!(study(DataSplit::Development), study(DataSplit::Development));
}
