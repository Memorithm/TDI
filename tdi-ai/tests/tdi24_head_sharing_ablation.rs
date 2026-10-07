#![cfg(feature = "experimental")]

//! TDI-24 slice 37: head-sharing ablation (Phase D).
//!
//! Two-head C6 scores with one shared chiral structure versus a per-head
//! structure, at identical weights and capacity. Software semantics only: no
//! training, no protected/final access, no attribution or scientific claim.

use tdi_ai::experimental::tdi24_eval::{
    C6_REFERENCE_WEIGHTS, EvalArm, EvalError, EvalOutcome, FIXED_M_SENSITIVITY_CONTRACT,
    HEAD_SHARING_ABLATION_CONTRACT, HEAD_SHARING_HEAD_COUNT, HeadSharingAblationReport,
    MAX_PREFLIGHT_PAIRS_PER_FAMILY, PER_HEAD_BASIS_RANKS, STAGE_C_PREFLIGHT_FAMILIES,
    StageCPreflightBudget, TrainableCapacity, run_fixed_m_sensitivity, run_head_sharing_ablation,
    run_head_sharing_ablation_for_label, run_stage_c_preflight,
    validate_head_sharing_ablation_report,
};
use tdi_ai::experimental::tdi24_tasks::DataSplit;

fn rejection(report: &HeadSharingAblationReport) -> &'static str {
    match validate_head_sharing_ablation_report(report) {
        Err(EvalError::HeadSharingAblationInvalid { reason }) => reason,
        other => panic!("expected a head-sharing rejection, got {other:?}"),
    }
}

#[test]
fn contract_pin_and_head_rule_are_declared() {
    assert_eq!(
        HEAD_SHARING_ABLATION_CONTRACT,
        "tdi24-head-sharing-ablation-v1"
    );
    assert_eq!(HEAD_SHARING_HEAD_COUNT, 2);
    assert_eq!(PER_HEAD_BASIS_RANKS, [0, 1]);
}

#[test]
fn shared_arm_reproduces_the_single_head_stage_c_c6_evaluator_exactly() {
    let budget = StageCPreflightBudget::bounded(3, 1);
    let report = run_head_sharing_ablation(DataSplit::Development, budget).unwrap();
    let preflight = run_stage_c_preflight(DataSplit::Development, budget).unwrap();
    assert_eq!(report.cases.len(), preflight.c6_records.len());
    for (case, record) in report.cases.iter().zip(&preflight.c6_records) {
        assert_eq!(record.arm, EvalArm::C6);
        assert_eq!((case.family, case.case_id), (record.family, record.case_id));
        match record.outcome {
            EvalOutcome::Scored { score, correct } => {
                assert_eq!(score.to_bits(), case.shared_score.to_bits());
                assert_eq!(correct, case.shared_correct);
            }
            ref other => panic!("unexpected outcome {other:?}"),
        }
    }
}

#[test]
fn per_head_arm_is_the_mean_of_slice_35_basis_scores() {
    let budget = StageCPreflightBudget::bounded(2, 0);
    let report = run_head_sharing_ablation(DataSplit::Validation, budget).unwrap();
    let fixed = run_fixed_m_sensitivity(DataSplit::Validation, budget).unwrap();
    for (index, case) in report.cases.iter().enumerate() {
        let group = &fixed.cases[index * 20..(index + 1) * 20];
        for (head, rank) in PER_HEAD_BASIS_RANKS.iter().enumerate() {
            assert_eq!(
                case.per_head_head_scores[head].to_bits(),
                group[*rank].score.to_bits()
            );
        }
        let mean = (group[0].score + group[1].score) / 2.0;
        assert_eq!(case.per_head_score.to_bits(), mean.to_bits());
    }
}

#[test]
fn smoke_ablation_covers_every_family_on_both_non_final_splits() {
    let budget = StageCPreflightBudget::bounded(MAX_PREFLIGHT_PAIRS_PER_FAMILY, 0);
    for split in [DataSplit::Development, DataSplit::Validation] {
        let report = run_head_sharing_ablation(split, budget).unwrap();
        assert_eq!(report.ablation_contract, HEAD_SHARING_ABLATION_CONTRACT);
        assert_eq!(report.basis_contract, FIXED_M_SENSITIVITY_CONTRACT);
        assert_eq!(report.cases.len() as u64, budget.cases_per_arm().unwrap());
        assert_eq!(report.families.len(), STAGE_C_PREFLIGHT_FAMILIES.len());
        assert_eq!(report.shared_weights, C6_REFERENCE_WEIGHTS);
        assert_eq!(report.per_head_weights, report.shared_weights);
        assert_eq!(report.shared_capacity, TrainableCapacity::reference_c6());
        assert_eq!(report.per_head_capacity, report.shared_capacity);
        assert!(!report.protected_or_final_access);
        assert!(!report.training_executed);
        assert!(!report.scientific_claim);
        assert!(report.experimental_non_final);
        // The per-head structure is not a no-op on the case stream.
        assert!(
            report
                .cases
                .iter()
                .any(|c| c.per_head_score.to_bits() != c.shared_score.to_bits())
        );
        validate_head_sharing_ablation_report(&report).unwrap();
    }
}

#[test]
fn tampered_reports_fail_closed() {
    let report =
        run_head_sharing_ablation(DataSplit::Development, StageCPreflightBudget::bounded(2, 0))
            .unwrap();
    let tamper = |mutate: &dyn Fn(&mut HeadSharingAblationReport), reason: &str| {
        let mut tampered = report.clone();
        mutate(&mut tampered);
        assert_eq!(rejection(&tampered), reason);
    };
    tamper(
        &|r| r.ablation_contract = "tdi24-head-sharing-ablation-v0",
        "contract_drift",
    );
    tamper(&|r| r.basis_contract = "legacy", "basis_contract_drift");
    tamper(&|r| r.head_count = 3, "head_count_drift");
    tamper(&|r| r.per_head_basis_ranks = [0, 19], "basis_rank_drift");
    tamper(&|r| r.shared_basis_ranks = [0, 1], "basis_rank_drift");
    tamper(&|r| r.per_head_weights.gamma = 0.0, "weights_drift");
    tamper(
        &|r| r.per_head_capacity = TrainableCapacity::reference_v6(),
        "capacity_mismatch",
    );
    tamper(
        &|r| {
            r.cases.pop();
        },
        "case_count",
    );
    tamper(
        &|r| r.cases[0].shared_score += 1.0,
        "shared_reference_drift",
    );
    tamper(
        &|r| r.cases[0].per_head_head_scores[0] += 1.0,
        "canonical_head_drift",
    );
    tamper(&|r| r.cases[0].per_head_score += 1.0, "head_mean_drift");
    tamper(&|r| r.families[0].shared_correct += 1, "summary_drift");
    tamper(&|r| r.cases.swap(0, 1), "case_evidence_drift");
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
            run_head_sharing_ablation_for_label(label, StageCPreflightBudget::bounded(1, 0)),
            Err(EvalError::ProtectedOrFinalSplit)
        );
    }
    assert_eq!(
        run_head_sharing_ablation(
            DataSplit::Development,
            StageCPreflightBudget::bounded(MAX_PREFLIGHT_PAIRS_PER_FAMILY + 1, 0)
        ),
        Err(EvalError::StageCPreflightInvalid {
            reason: "budget_exceeds_case_cap"
        })
    );
    assert_eq!(
        run_head_sharing_ablation(DataSplit::Development, StageCPreflightBudget::bounded(0, 0)),
        Err(EvalError::StageCPreflightInvalid {
            reason: "empty_budget"
        })
    );
}

#[test]
fn ablation_is_deterministic() {
    let budget = StageCPreflightBudget::bounded(2, 0);
    assert_eq!(
        run_head_sharing_ablation(DataSplit::Validation, budget).unwrap(),
        run_head_sharing_ablation(DataSplit::Validation, budget).unwrap()
    );
}
