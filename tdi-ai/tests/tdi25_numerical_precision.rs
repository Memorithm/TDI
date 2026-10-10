//! TDI-25 slice 46: numerical precision study (Phase E).
//!
//! T6 and C6 scored with the unchanged f64 matched reference and with f32
//! kernels (inputs rounded to f32, reference operation order) on the clean
//! matched population and on every slice-43 translation/origin stress, under
//! the upstream TDI-24 slice-44 declared forward-error bound, with complete
//! failure accounting. No training, no protected/final access, no claim.
#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi24_eval::PRECISION_BOUND_FACTOR;
use tdi_ai::experimental::tdi25_eval::matched_reference::{
    MATCHED_POPULATION_CONTRACT, MatchedPrimaryRun,
};
use tdi_ai::experimental::tdi25_eval::{
    EvalError, NUMERICAL_PRECISION_CONTRACT, NumericalPrecisionReport, ORIGIN_STRESS_OFFSETS,
    ORIGIN_STRESS_TRANSFORMS, PrecisionInput, REQUIRED_SYNTHESIS_FAMILIES, SEQUENCE_SCALING_ARMS,
    StageCPreflightBudget, c6_score_f32, precision_inputs, run_numerical_precision,
    run_numerical_precision_for_label, t6_score_f32, translation_origin_stress_seed,
    validate_numerical_precision_report,
};
use tdi_ai::experimental::tdi25_tasks::DataSplit;
use tdi_ai::experimental::tdi25_torsor_chiral::ComparisonArm;

fn study(split: DataSplit) -> NumericalPrecisionReport {
    run_numerical_precision(split, StageCPreflightBudget::smoke()).unwrap()
}

fn rejection(report: &NumericalPrecisionReport) -> &'static str {
    match validate_numerical_precision_report(report) {
        Err(EvalError::MatchedPrecisionInvalid { reason }) => reason,
        other => panic!("expected a numerical precision rejection, got {other:?}"),
    }
}

#[test]
fn contract_and_reused_pins_are_declared() {
    assert_eq!(NUMERICAL_PRECISION_CONTRACT, "tdi25-numerical-precision-v1");
    let classes = precision_inputs();
    assert_eq!(
        classes.len(),
        1 + ORIGIN_STRESS_TRANSFORMS.len() * ORIGIN_STRESS_OFFSETS.len()
    );
    assert_eq!(classes[0], PrecisionInput::Clean);
    let report = study(DataSplit::Validation);
    assert_eq!(report.bound_factor, PRECISION_BOUND_FACTOR);
    assert_eq!(report.stress_seed, translation_origin_stress_seed());
    assert_eq!(report.population_contract, MATCHED_POPULATION_CONTRACT);
    assert_eq!(
        report.cells.len() as u64,
        REQUIRED_SYNTHESIS_FAMILIES.len() as u64
            * StageCPreflightBudget::smoke().seed_blocks
            * (classes.len() * SEQUENCE_SCALING_ARMS.len()) as u64
    );
}

#[test]
fn f32_kernels_are_exact_on_the_dyadic_clean_population() {
    // Clean matched inputs are small half-integers: every f32 product and
    // partial sum is exact, so f32 must reproduce the f64 primary exactly.
    for family in REQUIRED_SYNTHESIS_FAMILIES {
        let run = MatchedPrimaryRun::evaluate(DataSplit::Validation, *family, 0, 8).unwrap();
        for (index, input) in run.inputs().iter().enumerate() {
            let (t6, t6_condition) = t6_score_f32(input);
            let (c6, c6_condition) = c6_score_f32(input).unwrap();
            assert_eq!(t6, run.t6_scores()[index]);
            assert_eq!(c6, run.c6_scores()[index]);
            assert!(t6_condition >= t6.abs() && c6_condition >= c6.abs());
        }
    }
}

#[test]
fn smoke_study_accounts_every_case() {
    for split in [DataSplit::Development, DataSplit::Validation] {
        let report = study(split);
        validate_numerical_precision_report(&report).unwrap();
        for cell in &report.cells {
            assert_eq!(
                cell.within_tolerance + cell.tolerance_failures + cell.non_finite_f32,
                cell.n_cases
            );
            if cell.input == PrecisionInput::Clean {
                assert_eq!(cell.within_tolerance, cell.n_cases);
                assert_eq!(cell.max_abs_error, 0.0);
            }
            // C6 reads no position: origin shift leaves its f32 error at zero.
            if cell.arm == ComparisonArm::C6
                && matches!(cell.input, PrecisionInput::OriginStress(transform, _)
                    if transform == ORIGIN_STRESS_TRANSFORMS[0])
            {
                assert_eq!(cell.max_abs_error, 0.0);
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
    let report = study(DataSplit::Validation);

    let mut drifted = report.clone();
    drifted.precision_contract = "tdi25-numerical-precision-v0";
    assert_eq!(rejection(&drifted), "contract_drift");

    let mut drifted = report.clone();
    drifted.population_contract = "other";
    assert_eq!(rejection(&drifted), "population_drift");

    let mut drifted = report.clone();
    drifted.stress_seed ^= 1;
    assert_eq!(rejection(&drifted), "seed_drift");

    let mut drifted = report.clone();
    drifted.bound_factor = 64.0;
    assert_eq!(rejection(&drifted), "tolerance_drift");

    let mut drifted = report.clone();
    drifted.cells.pop();
    assert_eq!(rejection(&drifted), "cell_count");

    let mut drifted = report.clone();
    drifted.cells.swap(0, 1);
    assert_eq!(rejection(&drifted), "grid_drift");

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
fn protected_or_final_labels_never_generate_a_case() {
    for label in ["protected", "final", "holdout", ""] {
        assert!(matches!(
            run_numerical_precision_for_label(label, StageCPreflightBudget::smoke()),
            Err(EvalError::ProtectedOrFinalSplit)
        ));
    }
}

#[test]
fn study_is_deterministic() {
    assert_eq!(study(DataSplit::Development), study(DataSplit::Development));
}
