#![cfg(feature = "experimental")]

//! TDI-24 slice 46: CPU reference cost study (Phase E).
//!
//! Deterministic source-level operation counts and logical carrier memory
//! of the unchanged slice-09 accounting, executed over the bounded Stage-C
//! stream for V6 and C6. No wall-clock timing, no performance claim, no
//! training, no protected/final access, no scientific claim.

use tdi_ai::experimental::tdi24_accounting::{
    REFERENCE_ACCOUNTING_CONTRACT, ScoreArm, pair_score_accounting,
};
use tdi_ai::experimental::tdi24_eval::{
    EvalArm, EvalError, REFERENCE_COST_ARMS, REFERENCE_COST_CONTRACT, REFERENCE_COST_TIMING_STATUS,
    ReferenceCostReport, StageCPreflightBudget, TrainableCapacity, run_reference_cost,
    run_reference_cost_for_label, validate_reference_cost_report,
};
use tdi_ai::experimental::tdi24_tasks::DataSplit;

fn study(split: DataSplit) -> ReferenceCostReport {
    run_reference_cost(split, StageCPreflightBudget::bounded(4, 0)).unwrap()
}

fn rejection(report: &ReferenceCostReport) -> &'static str {
    match validate_reference_cost_report(report) {
        Err(EvalError::ReferenceCostInvalid { reason }) => reason,
        other => panic!("expected a reference-cost rejection, got {other:?}"),
    }
}

#[test]
fn contracts_and_arms_are_declared() {
    assert_eq!(REFERENCE_COST_CONTRACT, "tdi24-reference-cost-v1");
    assert_eq!(REFERENCE_COST_TIMING_STATUS, "timing_not_measured");
    assert_eq!(REFERENCE_COST_ARMS, [EvalArm::V6, EvalArm::C6]);
}

#[test]
fn smoke_study_executes_both_arms_with_exact_totals() {
    for split in [DataSplit::Development, DataSplit::Validation] {
        let report = study(split);
        validate_reference_cost_report(&report).unwrap();
        assert_eq!(report.accounting_contract, REFERENCE_ACCOUNTING_CONTRACT);
        assert_eq!(report.cases_per_arm, 32);
        assert_eq!(report.v6_capacity, TrainableCapacity::reference_v6());
        assert_eq!(report.c6_capacity, TrainableCapacity::reference_c6());
        assert!(!report.timing_measured);
        assert_eq!(report.timing_status, "timing_not_measured");
        // Per-pair slice-09 accounting: V6 (6M, 6A, 12P), C6 (21M, 26A, 41P),
        // both 12 carrier scalars = 96 B, over 32 scored cases.
        let expected = [(6_u64, 6, 12), (21, 26, 41)];
        for (cell, (mult, add, pred)) in report.cells.iter().zip(expected) {
            assert_eq!(cell.accounting, pair_score_accounting(cell.arm.score_arm()));
            assert_eq!(cell.cases_scored, 32);
            assert_eq!(cell.scores_finite, 32);
            assert_eq!(cell.total_multiplications, 32 * mult);
            assert_eq!(cell.total_additions, 32 * add);
            assert_eq!(cell.total_validity_predicates, 32 * pred);
            assert_eq!(cell.total_carrier_bytes, 32 * 96);
        }
        assert_eq!(report.cells[0].accounting.arm, ScoreArm::V6);
        assert_eq!(report.cells[1].accounting.arm, ScoreArm::C6);
        assert!(!report.performance_claim);
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
    drifted.cost_contract = "tdi24-reference-cost-v0";
    assert_eq!(rejection(&drifted), "contract_drift");

    let mut drifted = report.clone();
    drifted.accounting_contract = "tdi24-reference-accounting-v2";
    assert_eq!(rejection(&drifted), "accounting_contract_drift");

    let mut drifted = report.clone();
    drifted.c6_capacity.trainable_parameters += 1;
    assert_eq!(rejection(&drifted), "capacity_mismatch");

    let mut drifted = report.clone();
    drifted.timing_measured = true;
    assert_eq!(rejection(&drifted), "timing_status_drift");

    let mut drifted = report.clone();
    drifted.cells.pop();
    assert_eq!(rejection(&drifted), "cell_count");

    let mut drifted = report.clone();
    drifted.cells.swap(0, 1);
    assert_eq!(rejection(&drifted), "grid_drift");

    let mut drifted = report.clone();
    drifted.cells[0].accounting.multiplications += 1;
    assert_eq!(rejection(&drifted), "accounting_drift");

    let mut drifted = report.clone();
    drifted.cells[0].scores_finite -= 1;
    assert_eq!(rejection(&drifted), "failure_accounting_drift");

    let mut drifted = report.clone();
    drifted.cells[1].total_multiplications += 1;
    assert_eq!(rejection(&drifted), "totals_drift");

    for (flag, reason) in [
        (0, "performance_claim"),
        (1, "protected_or_final_access"),
        (2, "training_executed"),
        (3, "scientific_claim"),
        (4, "experimental_non_final"),
    ] {
        let mut drifted = report.clone();
        match flag {
            0 => drifted.performance_claim = true,
            1 => drifted.protected_or_final_access = true,
            2 => drifted.training_executed = true,
            3 => drifted.scientific_claim = true,
            _ => drifted.experimental_non_final = false,
        }
        assert_eq!(rejection(&drifted), reason);
    }
}

#[test]
fn protected_or_final_labels_and_bad_budgets_never_score() {
    for label in ["protected", "final", "holdout", ""] {
        assert_eq!(
            run_reference_cost_for_label(label, StageCPreflightBudget::bounded(2, 0)),
            Err(EvalError::ProtectedOrFinalSplit)
        );
    }
    assert!(
        run_reference_cost(DataSplit::Development, StageCPreflightBudget::bounded(0, 0)).is_err()
    );
    assert!(
        run_reference_cost(DataSplit::Development, StageCPreflightBudget::bounded(9, 0)).is_err()
    );
}

#[test]
fn study_is_deterministic() {
    assert_eq!(study(DataSplit::Development), study(DataSplit::Development));
}
