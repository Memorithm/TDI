#![cfg(feature = "experimental")]

//! TDI-24 slice 39: sequence-length scaling (Phase D).
//!
//! Matched lengths and masks with bounded cost accounting. Software semantics
//! only: no training, no length selection, no protected/final access, no
//! scientific claim.

use tdi_ai::experimental::tdi24_attention::{MASKING_CONTRACT, MaskPolicy, NORMALIZER_CONTRACT};
use tdi_ai::experimental::tdi24_chiral::Chiral6;
use tdi_ai::experimental::tdi24_eval::{
    EvalError, MAX_PREFLIGHT_PAIRS_PER_FAMILY, SEQUENCE_ARMS, SEQUENCE_LENGTH_SCALING_CONTRACT,
    SEQUENCE_LENGTHS, SEQUENCE_MASK_POLICIES, STAGE_C_PREFLIGHT_FAMILIES, SequenceArm,
    SequenceLengthScalingReport, StageCPreflightBudget, TrainableCapacity,
    run_sequence_length_scaling, run_sequence_length_scaling_for_label, sequence_window_rows,
    validate_sequence_length_scaling_report,
};
use tdi_ai::experimental::tdi24_tasks::DataSplit;

fn rejection(report: &SequenceLengthScalingReport) -> &'static str {
    match validate_sequence_length_scaling_report(report) {
        Err(EvalError::SequenceLengthScalingInvalid { reason }) => reason,
        other => panic!("expected a sequence-length rejection, got {other:?}"),
    }
}

#[test]
fn contract_pin_lengths_masks_and_arms_are_declared() {
    assert_eq!(
        SEQUENCE_LENGTH_SCALING_CONTRACT,
        "tdi24-sequence-length-scaling-v1"
    );
    assert_eq!(SEQUENCE_LENGTHS, [2, 4, 8]);
    assert_eq!(
        SEQUENCE_MASK_POLICIES,
        [MaskPolicy::Full, MaskPolicy::Causal]
    );
    assert_eq!(SEQUENCE_ARMS, [SequenceArm::C6, SequenceArm::DirectOnly]);
}

#[test]
fn window_rows_follow_the_shared_mask_and_normalizer() {
    let a = Chiral6::new([1.0, 0.0, 0.0], [0.0, 0.0, 0.0]).unwrap();
    let b = Chiral6::new([0.0, 1.0, 0.0], [0.0, 0.0, 0.0]).unwrap();
    // Direct-only logits: row 0 = [1, 0], row 1 = [0, 1].
    let full =
        sequence_window_rows(&[a, b], &[a, b], MaskPolicy::Full, SequenceArm::DirectOnly).unwrap();
    let e = core::f64::consts::E;
    assert!((full[0][0] - e / (e + 1.0)).abs() < 1e-15);
    assert!((full[1][1] - e / (e + 1.0)).abs() < 1e-15);
    let causal = sequence_window_rows(
        &[a, b],
        &[a, b],
        MaskPolicy::Causal,
        SequenceArm::DirectOnly,
    )
    .unwrap();
    assert_eq!(causal[0], vec![1.0, 0.0]);
    assert_eq!(
        sequence_window_rows(&[a], &[a], MaskPolicy::Full, SequenceArm::C6),
        Err(EvalError::SequenceLengthScalingInvalid {
            reason: "length_not_registered"
        })
    );
}

#[test]
fn smoke_study_has_exact_bounded_cost_accounting() {
    let budget = StageCPreflightBudget::bounded(MAX_PREFLIGHT_PAIRS_PER_FAMILY, 0);
    let per_family = budget.cases_per_arm().unwrap() / STAGE_C_PREFLIGHT_FAMILIES.len() as u64;
    for split in [DataSplit::Development, DataSplit::Validation] {
        let report = run_sequence_length_scaling(split, budget).unwrap();
        assert_eq!(report.normalizer_contract, NORMALIZER_CONTRACT);
        assert_eq!(report.masking_contract, MASKING_CONTRACT);
        assert_eq!(report.capacity, TrainableCapacity::reference_c6());
        assert_eq!(report.cells.len(), 3 * 2 * 2 * 4);
        for cell in &report.cells {
            let length = cell.length as u64;
            assert_eq!(cell.windows * length + cell.unwindowed_cases, per_family);
            assert_eq!(cell.score_evaluations, cell.windows * length * length);
            assert_eq!(cell.normalizer_calls, cell.windows * length);
            assert!(cell.max_row_sum_error <= 1e-12);
        }
        assert!(!report.protected_or_final_access);
        assert!(!report.training_executed);
        assert!(!report.scientific_claim);
        assert!(report.experimental_non_final);
        validate_sequence_length_scaling_report(&report).unwrap();
    }
}

#[test]
fn tampered_reports_fail_closed() {
    let report =
        run_sequence_length_scaling(DataSplit::Development, StageCPreflightBudget::bounded(4, 0))
            .unwrap();
    let tamper = |mutate: &dyn Fn(&mut SequenceLengthScalingReport), reason: &str| {
        let mut tampered = report.clone();
        mutate(&mut tampered);
        assert_eq!(rejection(&tampered), reason);
    };
    tamper(&|r| r.scaling_contract = "legacy", "contract_drift");
    tamper(
        &|r| r.normalizer_contract = "legacy",
        "normalizer_contract_drift",
    );
    tamper(&|r| r.lengths = [2, 4, 16], "length_set_drift");
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
    tamper(
        &|r| r.cells[0].score_evaluations += 1,
        "cost_accounting_drift",
    );
    tamper(&|r| r.cells[0].masked_entries += 1, "cost_accounting_drift");
    tamper(&|r| r.cells[0].max_row_sum_error = 1.0, "row_sum_drift");
    tamper(&|r| r.cells[0].length = 0, "length_not_registered");
    tamper(&|r| r.cells[0].length = 3, "length_not_registered");
    tamper(
        &|r| r.cells[0].self_retrieval_rows ^= 1,
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
            run_sequence_length_scaling_for_label(label, StageCPreflightBudget::bounded(1, 0)),
            Err(EvalError::ProtectedOrFinalSplit)
        );
    }
    assert_eq!(
        run_sequence_length_scaling(DataSplit::Development, StageCPreflightBudget::bounded(0, 0)),
        Err(EvalError::StageCPreflightInvalid {
            reason: "empty_budget"
        })
    );
}

#[test]
fn study_is_deterministic() {
    let budget = StageCPreflightBudget::bounded(4, 0);
    assert_eq!(
        run_sequence_length_scaling(DataSplit::Validation, budget).unwrap(),
        run_sequence_length_scaling(DataSplit::Validation, budget).unwrap()
    );
}
