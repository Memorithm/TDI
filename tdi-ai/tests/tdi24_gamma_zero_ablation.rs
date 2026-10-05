#![cfg(feature = "experimental")]

//! TDI-24 slice 31: `gamma=0` ablation (Phase D).
//!
//! Removes only the parity-odd coefficient from the matched C6 reference and
//! reruns the bounded Stage-C case stream on synthetic Development/Validation
//! cases. These tests qualify software semantics and exact identities only: no
//! training, no protected/final access, no attribution or scientific claim.

use tdi_ai::experimental::tdi24_eval::{
    C6_REFERENCE_WEIGHTS, EvalArm, EvalError, EvalOutcome, GAMMA_ZERO_ABLATION_CONTRACT,
    MAX_PREFLIGHT_PAIRS_PER_FAMILY, STAGE_C_PREFLIGHT_FAMILIES, StageCPreflightBudget,
    TrainableCapacity, gamma_zero_weights, run_gamma_zero_ablation,
    run_gamma_zero_ablation_for_label, run_stage_c_preflight, validate_gamma_zero_ablation_report,
};
use tdi_ai::experimental::tdi24_tasks::DataSplit;

#[test]
fn gamma_zero_changes_only_the_parity_odd_coefficient() {
    assert_eq!(GAMMA_ZERO_ABLATION_CONTRACT, "tdi24-gamma-zero-ablation-v1");
    let ablated = gamma_zero_weights(C6_REFERENCE_WEIGHTS);
    assert_eq!(ablated.alpha, C6_REFERENCE_WEIGHTS.alpha);
    assert_eq!(ablated.beta, C6_REFERENCE_WEIGHTS.beta);
    assert_eq!(ablated.gamma, 0.0);
    assert_ne!(C6_REFERENCE_WEIGHTS.gamma, 0.0);
}

#[test]
fn gamma_zero_ablation_runs_on_both_non_final_splits_with_exact_identities() {
    for split in [DataSplit::Development, DataSplit::Validation] {
        let budget = StageCPreflightBudget::bounded(4, 3);
        let report = run_gamma_zero_ablation(split, budget).unwrap();
        validate_gamma_zero_ablation_report(&report).unwrap();
        assert_eq!(report.split, split);
        assert_eq!(report.cases.len(), 32);
        assert_eq!(report.reference_capacity, TrainableCapacity::reference_c6());
        assert_eq!(report.reference_capacity, report.ablated_capacity);
        assert_eq!(report.reference_capacity.trainable_parameters, 0);
        assert!(!report.protected_or_final_access);
        assert!(!report.training_executed);
        assert!(!report.scientific_claim);
        assert!(report.experimental_non_final);
        for case in &report.cases {
            assert_eq!(
                case.reference_score,
                case.ablated_score + C6_REFERENCE_WEIGHTS.gamma * case.parity_odd_channel
            );
        }
        assert_eq!(report.families.len(), STAGE_C_PREFLIGHT_FAMILIES.len());
        for (summary, family) in report.families.iter().zip(STAGE_C_PREFLIGHT_FAMILIES) {
            assert_eq!(summary.family, *family);
            assert_eq!(summary.n_cases, 8);
            assert!(summary.reference_correct <= summary.n_cases);
            assert!(summary.ablated_correct <= summary.n_cases);
        }
    }
}

#[test]
fn reference_side_reproduces_the_stage_c_c6_evaluator_exactly() {
    let budget = StageCPreflightBudget::bounded(3, 1);
    let ablation = run_gamma_zero_ablation(DataSplit::Development, budget).unwrap();
    let preflight = run_stage_c_preflight(DataSplit::Development, budget).unwrap();
    assert_eq!(ablation.cases.len(), preflight.c6_records.len());
    for (case, record) in ablation.cases.iter().zip(&preflight.c6_records) {
        assert_eq!(record.arm, EvalArm::C6);
        assert_eq!((case.family, case.case_id), (record.family, record.case_id));
        match record.outcome {
            EvalOutcome::Scored { score, correct } => {
                assert_eq!(score.to_bits(), case.reference_score.to_bits());
                assert_eq!(correct, case.reference_correct);
            }
            ref other => panic!("unexpected outcome {other:?}"),
        }
    }
}

#[test]
fn gamma_zero_ablation_is_deterministic_and_bounded() {
    let budget = StageCPreflightBudget::bounded(2, 0);
    assert_eq!(
        run_gamma_zero_ablation(DataSplit::Validation, budget).unwrap(),
        run_gamma_zero_ablation(DataSplit::Validation, budget).unwrap()
    );
    let full = StageCPreflightBudget::bounded(MAX_PREFLIGHT_PAIRS_PER_FAMILY, 0);
    assert_eq!(
        run_gamma_zero_ablation(DataSplit::Development, full)
            .unwrap()
            .cases
            .len(),
        64
    );
    assert_eq!(
        run_gamma_zero_ablation(
            DataSplit::Development,
            StageCPreflightBudget::bounded(MAX_PREFLIGHT_PAIRS_PER_FAMILY + 1, 0)
        ),
        Err(EvalError::StageCPreflightInvalid {
            reason: "budget_exceeds_case_cap"
        })
    );
}

#[test]
fn protected_or_final_labels_never_generate_an_ablation_case() {
    for label in ["protected", "final", "holdout", ""] {
        assert_eq!(
            run_gamma_zero_ablation_for_label(label, StageCPreflightBudget::bounded(1, 0)),
            Err(EvalError::ProtectedOrFinalSplit)
        );
    }
}

#[test]
fn drifted_ablation_reports_fail_closed() {
    let report =
        run_gamma_zero_ablation(DataSplit::Development, StageCPreflightBudget::bounded(1, 0))
            .unwrap();
    let reject =
        |mutate: &dyn Fn(&mut tdi_ai::experimental::tdi24_eval::GammaZeroAblationReport),
         reason: &'static str| {
            let mut tampered = report.clone();
            mutate(&mut tampered);
            assert_eq!(
                validate_gamma_zero_ablation_report(&tampered),
                Err(EvalError::GammaZeroAblationInvalid { reason })
            );
        };
    reject(
        &|r| r.ablation_contract = "tdi24-gamma-zero-ablation-v0",
        "contract_drift",
    );
    reject(&|r| r.ablated_weights.beta = 0.5, "ablated_weights_drift");
    reject(&|r| r.ablated_weights.gamma = 1.0, "ablated_weights_drift");
    reject(
        &|r| r.reference_weights.gamma = 2.0,
        "reference_weights_drift",
    );
    reject(
        &|r| r.ablated_capacity.trainable_parameters = 1,
        "capacity_mismatch",
    );
    reject(
        &|r| {
            r.cases.pop();
        },
        "case_count",
    );
    reject(
        &|r| r.cases[0].ablated_score = r.cases[0].reference_score,
        "parity_odd_residual",
    );
    reject(
        &|r| r.families[0].ablated_correct += 1,
        "family_summary_drift",
    );
    reject(
        &|r| r.protected_or_final_access = true,
        "protected_or_final_access",
    );
    reject(&|r| r.training_executed = true, "training_executed");
    reject(&|r| r.scientific_claim = true, "scientific_claim");
    reject(
        &|r| r.experimental_non_final = false,
        "experimental_non_final",
    );
}
