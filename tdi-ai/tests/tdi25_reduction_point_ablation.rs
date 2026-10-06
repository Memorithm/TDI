//! TDI-25 slice 31: torsor reduction-point ablation.
//!
//! Phase-D attribution ablation on the bounded matched Development/Validation
//! population. No protected/final execution, no training, no scientific claim.
#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi22_torsor::{Torsor3, Twist3, Vec3, direct_pairing};
use tdi_ai::experimental::tdi25_eval::matched_reference::{
    MATCHED_POPULATION_CONTRACT, MatchedPrimaryRun,
};
use tdi_ai::experimental::tdi25_eval::{
    EvalError, ParameterReadoutCapacity, REQUIRED_SYNTHESIS_FAMILIES, ReductionPointAblationReport,
    StageCPreflightBudget, TORSOR_REDUCTION_POINT_ABLATION_CONTRACT,
    reduction_point_transport_term, run_torsor_reduction_point_ablation,
    run_torsor_reduction_point_ablation_for_label, transport_identity_holds,
    untransported_torsor_score, validate_torsor_reduction_point_ablation_report,
};
use tdi_ai::experimental::tdi25_tasks::DataSplit;
use tdi_ai::experimental::tdi25_torsor_chiral::{TaskFamily, torsor_arm_score};

fn v(x: f64, y: f64, z: f64) -> Vec3 {
    Vec3::new(x, y, z).unwrap()
}

fn smoke(split: DataSplit) -> ReductionPointAblationReport {
    run_torsor_reduction_point_ablation(split, StageCPreflightBudget::smoke()).unwrap()
}

fn rejection(report: &ReductionPointAblationReport) -> &'static str {
    match validate_torsor_reduction_point_ablation_report(report) {
        Err(EvalError::ReductionPointAblationInvalid { reason }) => reason,
        other => panic!("expected a reduction-point ablation rejection, got {other:?}"),
    }
}

#[test]
fn smoke_ablation_covers_every_family_on_both_non_final_splits() {
    for split in [DataSplit::Development, DataSplit::Validation] {
        let budget = StageCPreflightBudget::smoke();
        let report = smoke(split);
        assert_eq!(report.split, split);
        assert_eq!(
            report.ablation_contract,
            TORSOR_REDUCTION_POINT_ABLATION_CONTRACT
        );
        assert_eq!(report.population_contract, MATCHED_POPULATION_CONTRACT);
        assert_eq!(report.cases.len() as u64, budget.cases_per_arm());
        assert_eq!(report.families.len(), REQUIRED_SYNTHESIS_FAMILIES.len());
        for summary in &report.families {
            assert_eq!(summary.n_cases, budget.seed_blocks * budget.cases_per_block);
        }
        // Matched capacity: the ablation removes structure, not parameters.
        assert_eq!(report.reference_capacity, report.ablated_capacity);
        assert_eq!(
            report.reference_capacity,
            ParameterReadoutCapacity::reference_t6()
        );
        assert!(!report.protected_or_final_access);
        assert!(!report.training_executed);
        assert!(!report.scientific_claim);
        assert!(report.experimental_non_final);
        validate_torsor_reduction_point_ablation_report(&report).unwrap();
    }
}

#[test]
fn reference_side_reproduces_the_matched_t6_primary_exactly() {
    let budget = StageCPreflightBudget::smoke();
    let report = smoke(DataSplit::Development);
    let mut position = 0usize;
    for family in REQUIRED_SYNTHESIS_FAMILIES {
        for seed_block in 0..budget.seed_blocks {
            let run = MatchedPrimaryRun::evaluate(
                DataSplit::Development,
                *family,
                seed_block,
                budget.cases_per_block,
            )
            .unwrap();
            for (index, outcome) in run.t6_outcomes().iter().enumerate() {
                let case = &report.cases[position];
                assert_eq!(case.family, *family);
                assert_eq!(case.seed_block, seed_block);
                assert_eq!(
                    case.reference_score.to_bits(),
                    run.t6_scores()[index].to_bits()
                );
                assert_eq!(case.reference_matches_target, outcome.matches_oracle);
                position += 1;
            }
        }
    }
    assert_eq!(position, report.cases.len());
}

#[test]
fn coincident_reduction_points_make_the_ablation_an_exact_no_op() {
    // Chiral and Neutral matched populations set P = Q = 0 by protocol.
    let report = smoke(DataSplit::Validation);
    for case in report.cases.iter().filter(|case| {
        matches!(
            case.family,
            TaskFamily::ChiralFavorable | TaskFamily::Neutral
        )
    }) {
        assert!(case.reduction_points_coincide);
        assert_eq!(case.transport_term, 0.0);
        assert_eq!(case.ablated_score.to_bits(), case.reference_score.to_bits());
        assert_eq!(case.ablated_matches_target, case.reference_matches_target);
    }
    for summary in report.families.iter().filter(|summary| {
        matches!(
            summary.family,
            TaskFamily::ChiralFavorable | TaskFamily::Neutral
        )
    }) {
        assert_eq!(summary.transport_active_cases, 0);
        assert_eq!(summary.ablated_matches, summary.reference_matches);
    }
}

#[test]
fn removing_transport_changes_scores_only_through_the_transport_term() {
    for split in [DataSplit::Development, DataSplit::Validation] {
        let report = smoke(split);
        for family in [TaskFamily::TorsorFavorable, TaskFamily::Mixed] {
            let summary = report
                .families
                .iter()
                .find(|summary| summary.family == family)
                .unwrap();
            // The matched geometry draws make the transport structure live.
            assert!(summary.transport_active_cases > 0);
        }
        for case in &report.cases {
            assert!(transport_identity_holds(
                case.reference_score,
                case.ablated_score,
                case.transport_term
            ));
            if case.transport_term == 0.0 {
                continue;
            }
            assert_ne!(case.ablated_score, case.reference_score);
            // A reference that hits its target exactly cannot also be hit by
            // a score displaced by a non-negligible transport term.
            if case.family == TaskFamily::TorsorFavorable
                && case.reference_matches_target
                && case.transport_term.abs() > 1e-9
            {
                assert!(!case.ablated_matches_target);
            }
        }
    }
}

#[test]
fn hand_calculated_transport_term_and_untransported_score() {
    let query = Twist3::new(v(1.0, 0.0, 0.0), v(0.0, 0.0, 1.0)).unwrap();
    let key = Torsor3::new(v(0.0, 1.0, 0.0), v(0.0, 0.0, 2.0), v(1.0, 0.0, 0.0)).unwrap();
    let q = v(0.0, 0.0, 0.0);
    // v.R + omega.M(P) = 0 + 2.
    assert_eq!(untransported_torsor_score(query, key).unwrap(), 2.0);
    // (P - Q) x R = (1,0,0) x (0,1,0) = (0,0,1); omega.(0,0,1) = 1.
    assert_eq!(reduction_point_transport_term(query, key, q).unwrap(), 1.0);
    let reference = torsor_arm_score(query, key, q).unwrap();
    assert_eq!(reference, 3.0);
    assert_eq!(direct_pairing(query, key, q).unwrap(), 3.0);
    // Re-reducing the same torsor at Q restores the reference exactly.
    let at_query = key.transport(q).unwrap();
    assert_eq!(untransported_torsor_score(query, at_query).unwrap(), 3.0);
    assert_eq!(
        reduction_point_transport_term(query, at_query, q).unwrap(),
        0.0
    );
}

#[test]
fn tampered_reports_fail_closed() {
    let report = smoke(DataSplit::Development);
    let active = report
        .cases
        .iter()
        .position(|case| case.transport_term != 0.0)
        .unwrap();
    let coincident = report
        .cases
        .iter()
        .position(|case| case.reduction_points_coincide)
        .unwrap();

    let mut tampered = report.clone();
    tampered.cases[active].transport_term = 0.0;
    assert_eq!(rejection(&tampered), "transport_residual");

    let mut tampered = report.clone();
    tampered.cases[active].ablated_score = tampered.cases[active].reference_score;
    assert_eq!(rejection(&tampered), "transport_residual");

    let mut tampered = report.clone();
    tampered.cases[coincident].transport_term = 1.0;
    tampered.cases[coincident].ablated_score -= 1.0;
    assert_eq!(rejection(&tampered), "coincident_point_transport");

    let mut tampered = report.clone();
    tampered.ablated_capacity = ParameterReadoutCapacity::reference_c6();
    assert_eq!(rejection(&tampered), "capacity_mismatch");

    let mut tampered = report.clone();
    tampered.ablation_contract = "tdi25-torsor-reduction-point-ablation-v0";
    assert_eq!(rejection(&tampered), "contract_drift");

    let mut tampered = report.clone();
    tampered.population_contract = "tdi25-legacy-phase-b";
    assert_eq!(rejection(&tampered), "population_drift");

    let mut tampered = report.clone();
    tampered.cases.swap(0, 1);
    assert_eq!(rejection(&tampered), "case_order");

    let mut tampered = report.clone();
    tampered.cases.pop();
    assert_eq!(rejection(&tampered), "case_count");

    let mut tampered = report.clone();
    tampered.families[0].ablated_matches += 1;
    assert_eq!(rejection(&tampered), "family_summary");

    let mut tampered = report.clone();
    tampered.protected_or_final_access = true;
    assert_eq!(rejection(&tampered), "protected_or_final_access");

    let mut tampered = report.clone();
    tampered.training_executed = true;
    assert_eq!(rejection(&tampered), "training_executed");

    let mut tampered = report.clone();
    tampered.scientific_claim = true;
    assert_eq!(rejection(&tampered), "scientific_claim");

    let mut tampered = report;
    tampered.experimental_non_final = false;
    assert_eq!(rejection(&tampered), "experimental_non_final");
}

#[test]
fn protected_or_final_labels_and_bad_budgets_never_score() {
    for label in ["protected", "final", "holdout", ""] {
        assert_eq!(
            run_torsor_reduction_point_ablation_for_label(label, StageCPreflightBudget::smoke()),
            Err(EvalError::ProtectedOrFinalSplit)
        );
    }
    for budget in [
        StageCPreflightBudget {
            seed_blocks: 0,
            cases_per_block: 8,
        },
        StageCPreflightBudget {
            seed_blocks: 3,
            cases_per_block: 8,
        },
        StageCPreflightBudget {
            seed_blocks: 1,
            cases_per_block: 1,
        },
        StageCPreflightBudget {
            seed_blocks: 1,
            cases_per_block: 17,
        },
    ] {
        assert!(matches!(
            run_torsor_reduction_point_ablation(DataSplit::Development, budget),
            Err(EvalError::StageCPreflightInvalid { .. })
        ));
    }
}

#[test]
fn ablation_is_deterministic() {
    let budget = StageCPreflightBudget {
        seed_blocks: 1,
        cases_per_block: 4,
    };
    let left = run_torsor_reduction_point_ablation(DataSplit::Development, budget).unwrap();
    let right = run_torsor_reduction_point_ablation(DataSplit::Development, budget).unwrap();
    assert_eq!(left, right);
}
