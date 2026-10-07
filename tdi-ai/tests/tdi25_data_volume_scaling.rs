//! TDI-25 slice 39: data-volume scaling.
//!
//! Phase-D study on the bounded matched Development/Validation population: T6
//! and C6 on identical nested prefixes of preregistered per-block sample
//! counts. No adaptive arm-specific allocation, no protected/final execution,
//! no training, no count selection, no scientific claim.
#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi25_eval::matched_reference::{
    MATCHED_POPULATION_CONTRACT, MatchedPrimaryRun,
};
use tdi_ai::experimental::tdi25_eval::{
    DATA_VOLUME_COUNTS, DATA_VOLUME_SCALING_CONTRACT, DataVolumeScalingReport, EvalError,
    ParameterReadoutCapacity, REQUIRED_SYNTHESIS_FAMILIES, StageCPreflightBudget,
    run_data_volume_scaling, run_data_volume_scaling_for_label,
    validate_data_volume_scaling_report,
};
use tdi_ai::experimental::tdi25_tasks::DataSplit;
use tdi_ai::experimental::tdi25_torsor_chiral::ComparisonArm;

fn smoke(split: DataSplit) -> DataVolumeScalingReport {
    run_data_volume_scaling(split, StageCPreflightBudget::smoke()).unwrap()
}

fn rejection(report: &DataVolumeScalingReport) -> &'static str {
    match validate_data_volume_scaling_report(report) {
        Err(EvalError::DataVolumeScalingInvalid { reason }) => reason,
        other => panic!("expected a data-volume rejection, got {other:?}"),
    }
}

#[test]
fn counts_are_preregistered() {
    assert_eq!(DATA_VOLUME_SCALING_CONTRACT, "tdi25-data-volume-scaling-v1");
    assert_eq!(DATA_VOLUME_COUNTS, [8, 16, 32]);
}

#[test]
fn both_arms_receive_identical_allocation_at_every_count() {
    for split in [DataSplit::Development, DataSplit::Validation] {
        let report = smoke(split);
        assert_eq!(report.population_contract, MATCHED_POPULATION_CONTRACT);
        assert_eq!(report.t6_capacity, ParameterReadoutCapacity::reference_t6());
        assert_eq!(report.c6_capacity, ParameterReadoutCapacity::reference_c6());
        assert_eq!(
            report.cells.len(),
            DATA_VOLUME_COUNTS.len() * REQUIRED_SYNTHESIS_FAMILIES.len() * 2
        );
        for pair in report.cells.chunks(2) {
            assert_eq!(
                (pair[0].arm, pair[1].arm),
                (ComparisonArm::T6, ComparisonArm::C6)
            );
            assert_eq!(pair[0].n_cases, pair[1].n_cases);
            assert_eq!(pair[0].family, pair[1].family);
            assert_eq!(
                pair[0].n_cases,
                report.budget.seed_blocks * pair[0].cases_per_block
            );
        }
        assert!(!report.protected_or_final_access);
        assert!(!report.training_executed);
        assert!(!report.scientific_claim);
        assert!(report.experimental_non_final);
    }
}

#[test]
fn smallest_count_matches_the_matched_primary_run() {
    let report = smoke(DataSplit::Development);
    let family = REQUIRED_SYNTHESIS_FAMILIES[0];
    let mut t6 = 0;
    for seed_block in 0..report.budget.seed_blocks {
        let run =
            MatchedPrimaryRun::evaluate(DataSplit::Development, family, seed_block, 8).unwrap();
        t6 += run
            .t6_outcomes()
            .iter()
            .filter(|o| o.matches_oracle)
            .count() as u64;
    }
    assert_eq!(report.cells[0].matches, t6);
}

#[test]
fn match_counts_are_monotone_in_nested_volume() {
    let report = smoke(DataSplit::Validation);
    let per_count = REQUIRED_SYNTHESIS_FAMILIES.len() * 2;
    for index in 0..per_count {
        let small = report.cells[index].matches;
        let medium = report.cells[per_count + index].matches;
        let large = report.cells[2 * per_count + index].matches;
        assert!(small <= medium && medium <= large);
    }
}

#[test]
fn tampered_reports_fail_closed() {
    let report = smoke(DataSplit::Development);
    let tamper = |mutate: &dyn Fn(&mut DataVolumeScalingReport), reason: &str| {
        let mut tampered = report.clone();
        mutate(&mut tampered);
        assert_eq!(rejection(&tampered), reason);
    };
    tamper(&|r| r.scaling_contract = "legacy", "contract_drift");
    tamper(&|r| r.population_contract = "legacy", "population_drift");
    tamper(&|r| r.counts = vec![8, 16, 64], "count_set_drift");
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
    tamper(&|r| r.cells[1].n_cases += 1, "allocation_drift");
    tamper(
        &|r| r.cells[0].matches = r.cells[0].n_cases + 1,
        "allocation_drift",
    );
    tamper(
        &|r| {
            r.cells[0].matches = r.cells[0]
                .matches
                .saturating_sub(1)
                .max(u64::from(r.cells[0].matches == 0))
        },
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
            run_data_volume_scaling_for_label(label, StageCPreflightBudget::smoke()),
            Err(EvalError::ProtectedOrFinalSplit)
        );
    }
    assert!(matches!(
        run_data_volume_scaling(
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
    assert_eq!(smoke(DataSplit::Validation), smoke(DataSplit::Validation));
}
