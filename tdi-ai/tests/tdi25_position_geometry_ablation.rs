//! TDI-25 slice 37: position-geometry ablation.
//!
//! Phase-D ablation on the bounded matched Development/Validation population:
//! T6 is scored under the generated geometry and every internal arm of the
//! frozen position-geometry registry, all reported, none selected. No
//! protected/final execution, no training, no scientific claim.
#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi22_torsor::TORSOR_CONTRACT;
use tdi_ai::experimental::tdi25_eval::matched_reference::{
    MATCHED_POPULATION_CONTRACT, MatchedPrimaryRun,
};
use tdi_ai::experimental::tdi25_eval::{
    EvalError, POSITION_GEOMETRY_ABLATION_ARMS, POSITION_GEOMETRY_ABLATION_CONTRACT,
    ParameterReadoutCapacity, PositionGeometryAblationReport, REQUIRED_SYNTHESIS_FAMILIES,
    StageCPreflightBudget, position_geometry_ablation_points, run_position_geometry_ablation,
    run_position_geometry_ablation_for_label, validate_position_geometry_ablation_report,
};
use tdi_ai::experimental::tdi25_tasks::{
    DataSplit, POSITION_GEOMETRY_ARM_CONTRACT, PositionGeometryArm, position_geometry_point,
};
use tdi_ai::experimental::tdi25_torsor_chiral::TaskFamily;

fn smoke(split: DataSplit) -> PositionGeometryAblationReport {
    run_position_geometry_ablation(split, StageCPreflightBudget::smoke()).unwrap()
}

fn rejection(report: &PositionGeometryAblationReport) -> &'static str {
    match validate_position_geometry_ablation_report(report) {
        Err(EvalError::PositionGeometryAblationInvalid { reason }) => reason,
        other => panic!("expected a position-geometry rejection, got {other:?}"),
    }
}

#[test]
fn arm_set_is_the_generated_geometry_plus_every_internal_registry_arm() {
    assert_eq!(
        POSITION_GEOMETRY_ABLATION_CONTRACT,
        "tdi25-position-geometry-ablation-v1"
    );
    assert_eq!(
        POSITION_GEOMETRY_ABLATION_ARMS,
        [
            None,
            Some(PositionGeometryArm::Linear),
            Some(PositionGeometryArm::Helical),
            Some(PositionGeometryArm::Learned),
        ]
    );
    let run =
        MatchedPrimaryRun::evaluate(DataSplit::Development, TaskFamily::Neutral, 0, 2).unwrap();
    assert_eq!(
        position_geometry_ablation_points(&run.inputs()[0], 4, 0),
        Err(EvalError::PositionGeometryAblationInvalid {
            reason: "arm_not_registered"
        })
    );
}

#[test]
fn positions_come_from_the_frozen_registry() {
    let report = smoke(DataSplit::Development);
    for case in &report.cases {
        if let Some(arm) = POSITION_GEOMETRY_ABLATION_ARMS[case.arm_index] {
            let key = position_geometry_point(arm, 2 * case.case_id, None)
                .unwrap()
                .point;
            let query = position_geometry_point(arm, 2 * case.case_id + 1, None)
                .unwrap()
                .point;
            assert_eq!(case.key_position, [key.x, key.y, key.z]);
            assert_eq!(case.query_position, [query.x, query.y, query.z]);
        }
    }
}

#[test]
fn matched_arm_reproduces_the_matched_t6_primary_exactly() {
    let budget = StageCPreflightBudget::smoke();
    let report = smoke(DataSplit::Validation);
    let arms = POSITION_GEOMETRY_ABLATION_ARMS.len();
    let mut position = 0usize;
    for family in REQUIRED_SYNTHESIS_FAMILIES {
        for seed_block in 0..budget.seed_blocks {
            let run = MatchedPrimaryRun::evaluate(
                DataSplit::Validation,
                *family,
                seed_block,
                budget.cases_per_block,
            )
            .unwrap();
            for (index, outcome) in run.t6_outcomes().iter().enumerate() {
                let case = &report.cases[position];
                assert_eq!(case.arm_index, 0);
                assert_eq!(case.score.to_bits(), run.t6_scores()[index].to_bits());
                assert_eq!(case.matches_target, outcome.matches_oracle);
                assert_eq!(case.key_position, run.inputs()[index].key_position());
                position += arms;
            }
        }
    }
}

#[test]
fn smoke_ablation_covers_every_arm_and_family_on_both_non_final_splits() {
    for split in [DataSplit::Development, DataSplit::Validation] {
        let budget = StageCPreflightBudget::smoke();
        let report = smoke(split);
        assert_eq!(report.population_contract, MATCHED_POPULATION_CONTRACT);
        assert_eq!(report.geometry_contract, POSITION_GEOMETRY_ARM_CONTRACT);
        assert_eq!(report.torsor_contract, TORSOR_CONTRACT);
        assert_eq!(
            report.cases.len() as u64,
            budget.cases_per_arm() * POSITION_GEOMETRY_ABLATION_ARMS.len() as u64
        );
        assert_eq!(report.capacity, ParameterReadoutCapacity::reference_t6());
        assert!(!report.protected_or_final_access);
        assert!(!report.training_executed);
        assert!(!report.scientific_claim);
        assert!(report.experimental_non_final);
        validate_position_geometry_ablation_report(&report).unwrap();
    }
}

#[test]
fn tampered_reports_fail_closed() {
    let report = smoke(DataSplit::Development);
    let tamper = |mutate: &dyn Fn(&mut PositionGeometryAblationReport), reason: &str| {
        let mut tampered = report.clone();
        mutate(&mut tampered);
        assert_eq!(rejection(&tampered), reason);
    };
    tamper(&|r| r.ablation_contract = "legacy", "contract_drift");
    tamper(&|r| r.population_contract = "legacy", "population_drift");
    tamper(
        &|r| r.geometry_contract = "legacy",
        "geometry_contract_drift",
    );
    tamper(&|r| r.torsor_contract = "legacy", "torsor_contract_drift");
    tamper(
        &|r| {
            r.arms.pop();
        },
        "arm_set_drift",
    );
    tamper(
        &|r| r.capacity = ParameterReadoutCapacity::reference_c6(),
        "capacity_mismatch",
    );
    tamper(
        &|r| {
            r.cases.pop();
        },
        "case_count",
    );
    tamper(&|r| r.cases.swap(0, 1), "case_order");
    tamper(&|r| r.summaries[0].matches += 1, "family_summary");
    tamper(&|r| r.cases[1].score += 1.0, "case_evidence_drift");
    tamper(
        &|r| r.cases[2].key_position[0] += 1.0,
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
            run_position_geometry_ablation_for_label(label, StageCPreflightBudget::smoke()),
            Err(EvalError::ProtectedOrFinalSplit)
        );
    }
    assert!(matches!(
        run_position_geometry_ablation(
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
fn ablation_is_deterministic() {
    let budget = StageCPreflightBudget {
        seed_blocks: 1,
        cases_per_block: 4,
    };
    assert_eq!(
        run_position_geometry_ablation(DataSplit::Validation, budget).unwrap(),
        run_position_geometry_ablation(DataSplit::Validation, budget).unwrap()
    );
}
