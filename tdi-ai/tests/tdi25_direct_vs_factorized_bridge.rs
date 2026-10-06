//! TDI-25 slice 32: direct vs factorized torsor bridge.
//!
//! Phase-D numerical-equivalence monitor on the bounded matched
//! Development/Validation population. No protected/final execution, no
//! training, no scientific claim.
#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi22_torsor::{
    TORSOR_CONTRACT, Torsor3, Twist3, Vec3, direct_pairing, factorized_pairing,
};
use tdi_ai::experimental::tdi25_eval::matched_reference::{
    MATCHED_POPULATION_CONTRACT, MatchedPrimaryRun,
};
use tdi_ai::experimental::tdi25_eval::{
    BRIDGE_EQUIVALENCE_RELATIVE_TOLERANCE, DIRECT_VS_FACTORIZED_BRIDGE_CONTRACT, EvalError,
    ParameterReadoutCapacity, REQUIRED_SYNTHESIS_FAMILIES, StageCPreflightBudget,
    TRANSPORT_IDENTITY_RELATIVE_TOLERANCE, TorsorBridgeEquivalenceReport, bridge_equivalence_holds,
    direct_torsor_bridge_score, run_direct_vs_factorized_bridge,
    run_direct_vs_factorized_bridge_for_label, validate_direct_vs_factorized_bridge_report,
};
use tdi_ai::experimental::tdi25_tasks::DataSplit;
use tdi_ai::experimental::tdi25_torsor_chiral::{TaskFamily, torsor_arm_score};

fn v(x: f64, y: f64, z: f64) -> Vec3 {
    Vec3::new(x, y, z).unwrap()
}

fn smoke(split: DataSplit) -> TorsorBridgeEquivalenceReport {
    run_direct_vs_factorized_bridge(split, StageCPreflightBudget::smoke()).unwrap()
}

fn rejection(report: &TorsorBridgeEquivalenceReport) -> &'static str {
    match validate_direct_vs_factorized_bridge_report(report) {
        Err(EvalError::TorsorBridgeEquivalenceInvalid { reason }) => reason,
        other => panic!("expected a torsor bridge rejection, got {other:?}"),
    }
}

#[test]
fn declared_tolerance_reuses_the_shared_matched_reference_value() {
    // No new numeric value: the monitor reuses the v1 scalar tolerance.
    assert_eq!(
        BRIDGE_EQUIVALENCE_RELATIVE_TOLERANCE.to_bits(),
        TRANSPORT_IDENTITY_RELATIVE_TOLERANCE.to_bits()
    );
    assert_eq!(BRIDGE_EQUIVALENCE_RELATIVE_TOLERANCE, 1e-12);
}

#[test]
fn smoke_bridge_covers_every_family_on_both_non_final_splits() {
    for split in [DataSplit::Development, DataSplit::Validation] {
        let budget = StageCPreflightBudget::smoke();
        let report = smoke(split);
        assert_eq!(report.split, split);
        assert_eq!(report.bridge_contract, DIRECT_VS_FACTORIZED_BRIDGE_CONTRACT);
        assert_eq!(report.population_contract, MATCHED_POPULATION_CONTRACT);
        assert_eq!(report.torsor_contract, TORSOR_CONTRACT);
        assert_eq!(
            report.relative_tolerance,
            BRIDGE_EQUIVALENCE_RELATIVE_TOLERANCE
        );
        assert_eq!(report.cases.len() as u64, budget.cases_per_arm());
        assert_eq!(report.families.len(), REQUIRED_SYNTHESIS_FAMILIES.len());
        for summary in &report.families {
            assert_eq!(summary.n_cases, budget.seed_blocks * budget.cases_per_block);
        }
        // Matched capacity: both forms are the same T6 score.
        assert_eq!(report.factorized_capacity, report.direct_capacity);
        assert_eq!(
            report.factorized_capacity,
            ParameterReadoutCapacity::reference_t6()
        );
        assert!(!report.protected_or_final_access);
        assert!(!report.training_executed);
        assert!(!report.scientific_claim);
        assert!(report.experimental_non_final);
        validate_direct_vs_factorized_bridge_report(&report).unwrap();
    }
}

#[test]
fn factorized_side_reproduces_the_matched_t6_primary_exactly() {
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
                assert_eq!(case.case_id, index as u64);
                assert_eq!(
                    case.factorized_score.to_bits(),
                    run.t6_scores()[index].to_bits()
                );
                assert_eq!(case.factorized_matches_target, outcome.matches_oracle);
                position += 1;
            }
        }
    }
    assert_eq!(position, report.cases.len());
}

#[test]
fn matched_population_bridge_is_bit_exact_and_records_the_degeneracy() {
    // Recorded degeneracy: the v1 matched population draws every scalar from
    // the half-integers {-2.0, ..., 1.5}, so both forms are evaluated exactly
    // in f64 and the monitored residual is identically zero on every case.
    for split in [DataSplit::Development, DataSplit::Validation] {
        let report = smoke(split);
        for case in &report.cases {
            assert!(case.bit_identical);
            assert_eq!(case.residual, 0.0);
            assert_eq!(case.direct_matches_target, case.factorized_matches_target);
            assert!(bridge_equivalence_holds(
                case.factorized_score,
                case.direct_score
            ));
        }
        for summary in &report.families {
            assert_eq!(summary.bit_identical_cases, summary.n_cases);
            assert_eq!(summary.max_abs_residual, 0.0);
            assert_eq!(summary.direct_matches, summary.factorized_matches);
        }
        // Second recorded degeneracy: the TorsorFavorable common target is the
        // upstream direct pairing itself, so the direct side matches it by
        // construction on every case.
        let torsor = report
            .families
            .iter()
            .find(|summary| summary.family == TaskFamily::TorsorFavorable)
            .unwrap();
        assert_eq!(torsor.direct_matches, torsor.n_cases);
    }
}

#[test]
fn bridge_forms_delegate_to_the_upstream_tdi22_pairings() {
    let query = Twist3::new(v(1.0, 0.0, 0.0), v(0.0, 0.0, 1.0)).unwrap();
    let key = Torsor3::new(v(0.0, 1.0, 0.0), v(0.0, 0.0, 2.0), v(1.0, 0.0, 0.0)).unwrap();
    let q = v(0.0, 0.0, 0.0);
    // Direct: v.R + omega.M(Q) = 0 + omega.(M(P) + (P - Q) x R) = 2 + 1.
    assert_eq!(direct_torsor_bridge_score(query, key, q).unwrap(), 3.0);
    assert_eq!(direct_pairing(query, key, q).unwrap(), 3.0);
    // Factorized: (v + Q x omega).R + omega.C with C = M(P) + P x R = (0,0,3).
    assert_eq!(factorized_pairing(query, key, q).unwrap(), 3.0);
    assert_eq!(torsor_arm_score(query, key, q).unwrap(), 3.0);
}

#[test]
fn off_population_roundoff_is_monitored_by_tolerance_not_bit_equality() {
    // Non-dyadic carriers: the two algebraically equal forms round
    // differently, and the monitor accepts the residual under the shared
    // relative tolerance rather than requiring bit identity.
    let query = Twist3::new(v(0.1, 0.2, 0.3), v(0.7, -0.3, 0.9)).unwrap();
    let key = Torsor3::new(v(0.3, 0.6, -0.1), v(-0.2, 0.5, 0.4), v(1.1, -0.7, 0.2)).unwrap();
    let q = v(0.9, 0.3, -1.3);
    let direct = direct_torsor_bridge_score(query, key, q).unwrap();
    let factorized = torsor_arm_score(query, key, q).unwrap();
    assert_ne!(direct.to_bits(), factorized.to_bits());
    assert!(bridge_equivalence_holds(factorized, direct));
    assert!(!bridge_equivalence_holds(factorized, direct + 1e-6));
    assert!(!bridge_equivalence_holds(f64::NAN, direct));
}

#[test]
fn tampered_reports_fail_closed() {
    let report = smoke(DataSplit::Development);

    let mut tampered = report.clone();
    tampered.cases[0].direct_score += 1.0;
    tampered.cases[0].residual =
        tampered.cases[0].factorized_score - tampered.cases[0].direct_score;
    tampered.cases[0].bit_identical = false;
    assert_eq!(rejection(&tampered), "bridge_residual");

    let mut tampered = report.clone();
    tampered.cases[0].residual = 1.0;
    assert_eq!(rejection(&tampered), "residual_drift");

    let mut tampered = report.clone();
    tampered.cases[0].bit_identical = false;
    assert_eq!(rejection(&tampered), "bit_identity_drift");

    let mut tampered = report.clone();
    tampered.cases[0].direct_matches_target = !tampered.cases[0].direct_matches_target;
    assert_eq!(rejection(&tampered), "match_bit_drift");

    let mut tampered = report.clone();
    tampered.relative_tolerance = 1e-9;
    assert_eq!(rejection(&tampered), "tolerance_drift");

    let mut tampered = report.clone();
    tampered.direct_capacity = ParameterReadoutCapacity::reference_c6();
    assert_eq!(rejection(&tampered), "capacity_mismatch");

    let mut tampered = report.clone();
    tampered.factorized_capacity = ParameterReadoutCapacity::reference_g6();
    tampered.direct_capacity = ParameterReadoutCapacity::reference_g6();
    assert_eq!(rejection(&tampered), "capacity_mismatch");

    let mut tampered = report.clone();
    tampered.bridge_contract = "tdi25-direct-vs-factorized-bridge-v0";
    assert_eq!(rejection(&tampered), "contract_drift");

    let mut tampered = report.clone();
    tampered.population_contract = "tdi25-legacy-phase-b";
    assert_eq!(rejection(&tampered), "population_drift");

    let mut tampered = report.clone();
    tampered.torsor_contract = "tdi22-torsor-dual-pairing-v0";
    assert_eq!(rejection(&tampered), "torsor_contract_drift");

    let mut tampered = report.clone();
    tampered.cases.swap(0, 1);
    assert_eq!(rejection(&tampered), "case_order");

    let mut tampered = report.clone();
    tampered.cases.pop();
    assert_eq!(rejection(&tampered), "case_count");

    let mut tampered = report.clone();
    tampered.families[0].direct_matches += 1;
    assert_eq!(rejection(&tampered), "family_summary");

    let mut tampered = report.clone();
    tampered.families[1].max_abs_residual = 1e-15;
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
            run_direct_vs_factorized_bridge_for_label(label, StageCPreflightBudget::smoke()),
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
            run_direct_vs_factorized_bridge(DataSplit::Development, budget),
            Err(EvalError::StageCPreflightInvalid { .. })
        ));
    }
}

#[test]
fn bridge_monitor_is_deterministic() {
    let budget = StageCPreflightBudget {
        seed_blocks: 1,
        cases_per_block: 4,
    };
    let left = run_direct_vs_factorized_bridge(DataSplit::Development, budget).unwrap();
    let right = run_direct_vs_factorized_bridge(DataSplit::Development, budget).unwrap();
    assert_eq!(left, right);
}
