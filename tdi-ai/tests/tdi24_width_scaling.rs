#![cfg(feature = "experimental")]

//! TDI-24 slice 38: width scaling (Phase D).
//!
//! C6 and its direct-only matched arm at every preregistered matched width
//! `2n`, `n = 1..=3`. Software semantics only: no training, no width
//! selection, no protected/final access, no scientific claim.

use tdi_ai::experimental::tdi24_chiral::{Chiral6, chiral_score};
use tdi_ai::experimental::tdi24_eval::{
    C6_REFERENCE_WEIGHTS, EvalArm, EvalError, EvalOutcome, MAX_PREFLIGHT_PAIRS_PER_FAMILY,
    STAGE_C_PREFLIGHT_FAMILIES, StageCPreflightBudget, TrainableCapacity, WIDTH_SCALING_CONTRACT,
    WIDTH_SCALING_WIDTHS, WidthScalingReport, direct_only_weights, restrict_carrier_to_width,
    run_stage_c_preflight, run_width_scaling, run_width_scaling_for_label,
    validate_width_scaling_report,
};
use tdi_ai::experimental::tdi24_tasks::DataSplit;

fn rejection(report: &WidthScalingReport) -> &'static str {
    match validate_width_scaling_report(report) {
        Err(EvalError::WidthScalingInvalid { reason }) => reason,
        other => panic!("expected a width-scaling rejection, got {other:?}"),
    }
}

#[test]
fn contract_pin_and_width_set_are_declared() {
    assert_eq!(WIDTH_SCALING_CONTRACT, "tdi24-width-scaling-v1");
    assert_eq!(WIDTH_SCALING_WIDTHS, [2, 4, 6]);
}

#[test]
fn unregistered_widths_are_rejected_and_restriction_is_exact() {
    let carrier = Chiral6::new([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]).unwrap();
    for width in [0, 1, 3, 5, 8] {
        assert_eq!(
            restrict_carrier_to_width(carrier, width),
            Err(EvalError::WidthScalingInvalid {
                reason: "width_not_registered"
            })
        );
    }
    assert_eq!(
        restrict_carrier_to_width(carrier, 2).unwrap().as_array(),
        [1.0, 0.0, 0.0, 4.0, 0.0, 0.0]
    );
    assert_eq!(
        restrict_carrier_to_width(carrier, 4).unwrap().as_array(),
        [1.0, 2.0, 0.0, 4.0, 5.0, 0.0]
    );
    assert_eq!(restrict_carrier_to_width(carrier, 6).unwrap(), carrier);
}

#[test]
fn hand_calculated_width_two_score() {
    let q = Chiral6::new([1.0, 2.0, 3.0], [0.5, -1.0, 2.0]).unwrap();
    let k = Chiral6::new([2.0, 1.0, -1.0], [1.5, 0.5, 1.0]).unwrap();
    let q2 = restrict_carrier_to_width(q, 2).unwrap();
    let k2 = restrict_carrier_to_width(k, 2).unwrap();
    // s = 1*2 + 0.5*1.5 = 2.75; chi = 1*1.5 - 0.5*2 = 0.5.
    assert_eq!(chiral_score(q2, k2, C6_REFERENCE_WEIGHTS).unwrap(), 3.25);
    assert_eq!(
        chiral_score(q2, k2, direct_only_weights(C6_REFERENCE_WEIGHTS)).unwrap(),
        2.75
    );
}

#[test]
fn full_width_reproduces_the_stage_c_c6_evaluator_exactly() {
    let budget = StageCPreflightBudget::bounded(3, 1);
    let report = run_width_scaling(DataSplit::Development, budget).unwrap();
    let preflight = run_stage_c_preflight(DataSplit::Development, budget).unwrap();
    assert_eq!(report.cases.len(), preflight.c6_records.len());
    for (case, record) in report.cases.iter().zip(&preflight.c6_records) {
        assert_eq!(record.arm, EvalArm::C6);
        assert_eq!((case.family, case.case_id), (record.family, record.case_id));
        match record.outcome {
            EvalOutcome::Scored { score, correct } => {
                assert_eq!(score.to_bits(), case.c6_scores[2].to_bits());
                assert_eq!(correct, case.c6_correct[2]);
            }
            ref other => panic!("unexpected outcome {other:?}"),
        }
    }
}

#[test]
fn smoke_study_covers_every_width_and_family_on_both_non_final_splits() {
    let budget = StageCPreflightBudget::bounded(MAX_PREFLIGHT_PAIRS_PER_FAMILY, 0);
    for split in [DataSplit::Development, DataSplit::Validation] {
        let report = run_width_scaling(split, budget).unwrap();
        assert_eq!(report.widths, WIDTH_SCALING_WIDTHS);
        assert_eq!(report.cases.len() as u64, budget.cases_per_arm().unwrap());
        assert_eq!(
            report.summaries.len(),
            WIDTH_SCALING_WIDTHS.len() * STAGE_C_PREFLIGHT_FAMILIES.len()
        );
        assert_eq!(report.c6_capacity, TrainableCapacity::reference_c6());
        assert_eq!(report.direct_capacity, report.c6_capacity);
        assert!(!report.protected_or_final_access);
        assert!(!report.training_executed);
        assert!(!report.scientific_claim);
        assert!(report.experimental_non_final);
        validate_width_scaling_report(&report).unwrap();
    }
}

#[test]
fn tampered_reports_fail_closed() {
    let report =
        run_width_scaling(DataSplit::Development, StageCPreflightBudget::bounded(2, 0)).unwrap();
    let tamper = |mutate: &dyn Fn(&mut WidthScalingReport), reason: &str| {
        let mut tampered = report.clone();
        mutate(&mut tampered);
        assert_eq!(rejection(&tampered), reason);
    };
    tamper(&|r| r.scaling_contract = "legacy", "contract_drift");
    tamper(&|r| r.widths = [2, 4, 4], "width_set_drift");
    tamper(&|r| r.direct_weights.gamma = 1.0, "weights_drift");
    tamper(
        &|r| r.direct_capacity = TrainableCapacity::reference_v6(),
        "capacity_mismatch",
    );
    tamper(
        &|r| {
            r.cases.pop();
        },
        "case_count",
    );
    tamper(
        &|r| r.cases[0].c6_scores[2] += 1.0,
        "full_width_reference_drift",
    );
    tamper(&|r| r.summaries[0].c6_correct += 1, "summary_drift");
    tamper(
        &|r| r.cases[0].direct_scores[0] += 1.0,
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
            run_width_scaling_for_label(label, StageCPreflightBudget::bounded(1, 0)),
            Err(EvalError::ProtectedOrFinalSplit)
        );
    }
    assert_eq!(
        run_width_scaling(DataSplit::Development, StageCPreflightBudget::bounded(0, 0)),
        Err(EvalError::StageCPreflightInvalid {
            reason: "empty_budget"
        })
    );
}

#[test]
fn study_is_deterministic() {
    let budget = StageCPreflightBudget::bounded(2, 0);
    assert_eq!(
        run_width_scaling(DataSplit::Validation, budget).unwrap(),
        run_width_scaling(DataSplit::Validation, budget).unwrap()
    );
}
