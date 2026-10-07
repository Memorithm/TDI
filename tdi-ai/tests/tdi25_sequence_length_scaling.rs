//! TDI-25 slice 38: sequence-length scaling.
//!
//! Phase-D study on the bounded matched Development/Validation population: T6
//! and C6 on identical non-overlapping windows of preregistered lengths, under
//! both shared mask policies, with exact resource accounting. No
//! protected/final execution, no training, no length selection, no
//! scientific claim.
#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi24_attention::{MASKING_CONTRACT, MaskPolicy, NORMALIZER_CONTRACT};
use tdi_ai::experimental::tdi25_eval::matched_reference::MATCHED_POPULATION_CONTRACT;
use tdi_ai::experimental::tdi25_eval::{
    EvalError, ParameterReadoutCapacity, REQUIRED_SYNTHESIS_FAMILIES,
    SEQUENCE_LENGTH_SCALING_CONTRACT, SEQUENCE_SCALING_ARMS, SEQUENCE_SCALING_LENGTHS,
    SEQUENCE_SCALING_POLICIES, SequenceLengthScalingReport, StageCPreflightBudget,
    evaluate_sequence_length_scaling, run_sequence_length_scaling,
    run_sequence_length_scaling_for_label, validate_sequence_length_scaling_report,
};
use tdi_ai::experimental::tdi25_tasks::DataSplit;
use tdi_ai::experimental::tdi25_torsor_chiral::{ComparisonArm, TaskFamily};

fn smoke(split: DataSplit) -> SequenceLengthScalingReport {
    run_sequence_length_scaling(split, StageCPreflightBudget::smoke()).unwrap()
}

fn rejection(report: &SequenceLengthScalingReport) -> &'static str {
    match validate_sequence_length_scaling_report(report) {
        Err(EvalError::SequenceLengthScalingInvalid { reason }) => reason,
        other => panic!("expected a sequence-length rejection, got {other:?}"),
    }
}

#[test]
fn lengths_masks_and_arms_are_preregistered() {
    assert_eq!(
        SEQUENCE_LENGTH_SCALING_CONTRACT,
        "tdi25-sequence-length-scaling-v1"
    );
    assert_eq!(SEQUENCE_SCALING_LENGTHS, [2, 4, 8]);
    assert_eq!(
        SEQUENCE_SCALING_POLICIES,
        [MaskPolicy::Full, MaskPolicy::Causal]
    );
    assert_eq!(
        SEQUENCE_SCALING_ARMS,
        [ComparisonArm::T6, ComparisonArm::C6]
    );
}

#[test]
fn smoke_study_has_exact_matched_resource_accounting() {
    for split in [DataSplit::Development, DataSplit::Validation] {
        let report = smoke(split);
        assert_eq!(report.population_contract, MATCHED_POPULATION_CONTRACT);
        assert_eq!(report.normalizer_contract, NORMALIZER_CONTRACT);
        assert_eq!(report.masking_contract, MASKING_CONTRACT);
        assert_eq!(report.t6_capacity, ParameterReadoutCapacity::reference_t6());
        assert_eq!(report.c6_capacity, ParameterReadoutCapacity::reference_c6());
        assert_eq!(
            report.cells.len() as u64,
            REQUIRED_SYNTHESIS_FAMILIES.len() as u64 * report.budget.seed_blocks * 12
        );
        // T6 and C6 consume identical windows and identical resources.
        for pair in report.cells.chunks(2) {
            let (t6, c6) = (&pair[0], &pair[1]);
            assert_eq!((t6.arm, c6.arm), (ComparisonArm::T6, ComparisonArm::C6));
            assert_eq!(t6.windows, c6.windows);
            assert_eq!(t6.score_evaluations, c6.score_evaluations);
            assert_eq!(t6.normalizer_calls, c6.normalizer_calls);
            assert_eq!(t6.masked_entries, c6.masked_entries);
        }
        assert!(!report.protected_or_final_access);
        assert!(!report.training_executed);
        assert!(!report.scientific_claim);
        assert!(report.experimental_non_final);
    }
}

#[test]
fn remainder_cases_are_counted_and_never_scored() {
    let cells =
        evaluate_sequence_length_scaling(DataSplit::Development, TaskFamily::Mixed, 0, 6).unwrap();
    let eight: Vec<_> = cells.iter().filter(|c| c.length == 8).collect();
    assert!(
        eight
            .iter()
            .all(|c| c.windows == 0 && c.unwindowed_cases == 6)
    );
    assert!(eight.iter().all(|c| c.score_evaluations == 0));
    let four: Vec<_> = cells.iter().filter(|c| c.length == 4).collect();
    assert!(
        four.iter()
            .all(|c| c.windows == 1 && c.unwindowed_cases == 2)
    );
}

#[test]
fn tampered_reports_fail_closed() {
    let report = smoke(DataSplit::Development);
    let tamper = |mutate: &dyn Fn(&mut SequenceLengthScalingReport), reason: &str| {
        let mut tampered = report.clone();
        mutate(&mut tampered);
        assert_eq!(rejection(&tampered), reason);
    };
    tamper(&|r| r.scaling_contract = "legacy", "contract_drift");
    tamper(&|r| r.population_contract = "legacy", "population_drift");
    tamper(
        &|r| r.normalizer_contract = "legacy",
        "normalizer_contract_drift",
    );
    tamper(
        &|r| r.masking_contract = "legacy",
        "normalizer_contract_drift",
    );
    tamper(&|r| r.lengths = vec![2, 4, 16], "length_set_drift");
    tamper(
        &|r| r.c6_capacity = ParameterReadoutCapacity::reference_t6(),
        "capacity_mismatch",
    );
    tamper(
        &|r| {
            r.cells.pop();
        },
        "cell_count",
    );
    tamper(&|r| r.cells.swap(0, 1), "cell_order");
    tamper(
        &|r| r.cells[0].score_evaluations += 1,
        "cost_accounting_drift",
    );
    tamper(&|r| r.cells[2].masked_entries += 1, "cost_accounting_drift");
    tamper(
        &|r| r.cells[0].unwindowed_cases += 1,
        "cost_accounting_drift",
    );
    tamper(&|r| r.cells[0].max_row_sum_error = 1.0, "row_sum_drift");
    tamper(
        &|r| r.cells[0].max_row_sum_error = f64::NAN,
        "row_sum_drift",
    );
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
            run_sequence_length_scaling_for_label(label, StageCPreflightBudget::smoke()),
            Err(EvalError::ProtectedOrFinalSplit)
        );
    }
    assert!(matches!(
        run_sequence_length_scaling(
            DataSplit::Development,
            StageCPreflightBudget {
                seed_blocks: 0,
                cases_per_block: 8,
            }
        ),
        Err(EvalError::StageCPreflightInvalid { .. })
    ));
}

#[test]
fn study_is_deterministic() {
    let budget = StageCPreflightBudget {
        seed_blocks: 1,
        cases_per_block: 4,
    };
    assert_eq!(
        run_sequence_length_scaling(DataSplit::Validation, budget).unwrap(),
        run_sequence_length_scaling(DataSplit::Validation, budget).unwrap()
    );
}
