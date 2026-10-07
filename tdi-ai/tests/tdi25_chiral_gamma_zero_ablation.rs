//! TDI-25 slice 33: chiral `gamma=0` ablation.
//!
//! Phase-D attribution ablation on the bounded matched Development/Validation
//! population: the matched C6 weights keep `alpha`/`beta` and only the
//! parity-odd coefficient is zeroed. No protected/final execution, no
//! training, no scientific claim.
#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi24_chiral::{CHIRAL_CONTRACT, Chiral6, chiral_score};
use tdi_ai::experimental::tdi25_eval::matched_reference::{
    MATCHED_CHIRAL_WEIGHTS, MATCHED_POPULATION_CONTRACT, MatchedPrimaryRun,
};
use tdi_ai::experimental::tdi25_eval::{
    CHIRAL_GAMMA_ZERO_ABLATION_CONTRACT, ChiralGammaZeroAblationReport, EvalError,
    ParameterReadoutCapacity, REQUIRED_SYNTHESIS_FAMILIES, StageCPreflightBudget,
    chiral_gamma_zero_weights, chiral_parity_odd_channel, gamma_zero_chiral_score,
    run_chiral_gamma_zero_ablation, run_chiral_gamma_zero_ablation_for_label,
    validate_chiral_gamma_zero_ablation_report,
};
use tdi_ai::experimental::tdi25_tasks::DataSplit;
use tdi_ai::experimental::tdi25_torsor_chiral::TaskFamily;

fn smoke(split: DataSplit) -> ChiralGammaZeroAblationReport {
    run_chiral_gamma_zero_ablation(split, StageCPreflightBudget::smoke()).unwrap()
}

fn rejection(report: &ChiralGammaZeroAblationReport) -> &'static str {
    match validate_chiral_gamma_zero_ablation_report(report) {
        Err(EvalError::ChiralGammaZeroAblationInvalid { reason }) => reason,
        other => panic!("expected a chiral gamma=0 ablation rejection, got {other:?}"),
    }
}

#[test]
fn gamma_zero_changes_only_the_parity_odd_coefficient() {
    assert_eq!(
        CHIRAL_GAMMA_ZERO_ABLATION_CONTRACT,
        "tdi25-chiral-gamma-zero-ablation-v1"
    );
    let ablated = chiral_gamma_zero_weights(MATCHED_CHIRAL_WEIGHTS);
    assert_eq!(ablated.alpha, MATCHED_CHIRAL_WEIGHTS.alpha);
    assert_eq!(ablated.beta, MATCHED_CHIRAL_WEIGHTS.beta);
    assert_eq!(ablated.gamma, 0.0);
    assert_ne!(MATCHED_CHIRAL_WEIGHTS.gamma, 0.0);
}

#[test]
fn smoke_ablation_covers_every_family_on_both_non_final_splits() {
    for split in [DataSplit::Development, DataSplit::Validation] {
        let budget = StageCPreflightBudget::smoke();
        let report = smoke(split);
        assert_eq!(report.split, split);
        assert_eq!(
            report.ablation_contract,
            CHIRAL_GAMMA_ZERO_ABLATION_CONTRACT
        );
        assert_eq!(report.population_contract, MATCHED_POPULATION_CONTRACT);
        assert_eq!(report.chiral_contract, CHIRAL_CONTRACT);
        assert_eq!(report.cases.len() as u64, budget.cases_per_arm());
        assert_eq!(report.families.len(), REQUIRED_SYNTHESIS_FAMILIES.len());
        for summary in &report.families {
            assert_eq!(summary.n_cases, budget.seed_blocks * budget.cases_per_block);
            assert!(summary.parity_odd_active_cases <= summary.n_cases);
        }
        // Matched capacity: the ablation removes a coefficient, not a parameter.
        assert_eq!(report.reference_capacity, report.ablated_capacity);
        assert_eq!(
            report.reference_capacity,
            ParameterReadoutCapacity::reference_c6()
        );
        assert!(!report.protected_or_final_access);
        assert!(!report.training_executed);
        assert!(!report.scientific_claim);
        assert!(report.experimental_non_final);
        for case in &report.cases {
            assert_eq!(
                case.reference_score,
                case.ablated_score + MATCHED_CHIRAL_WEIGHTS.gamma * case.parity_odd_channel
            );
            assert!(case.enantiomorphic_split_closed);
        }
        validate_chiral_gamma_zero_ablation_report(&report).unwrap();
    }
}

#[test]
fn reference_side_reproduces_the_matched_c6_primary_exactly() {
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
            for (index, outcome) in run.c6_outcomes().iter().enumerate() {
                let case = &report.cases[position];
                assert_eq!(
                    case.reference_score.to_bits(),
                    run.c6_scores()[index].to_bits()
                );
                assert_eq!(case.reference_matches_target, outcome.matches_oracle);
                let query = Chiral6::from_array(run.inputs()[index].query()).unwrap();
                let key = Chiral6::from_array(run.inputs()[index].key()).unwrap();
                assert_eq!(
                    case.ablated_score.to_bits(),
                    gamma_zero_chiral_score(query, key).unwrap().to_bits()
                );
                assert_eq!(
                    case.parity_odd_channel.to_bits(),
                    chiral_parity_odd_channel(query, key).unwrap().to_bits()
                );
                position += 1;
            }
        }
    }
}

#[test]
fn chiral_favorable_target_is_the_reference_score_and_records_the_degeneracy() {
    // The ChiralFavorable common target is `s + chi`, which is exactly the
    // matched C6 score (alpha=1, beta=0, gamma=1). The reference therefore
    // matches by construction and the ablation can only match where chi = 0.
    for split in [DataSplit::Development, DataSplit::Validation] {
        let report = smoke(split);
        let chiral = report
            .families
            .iter()
            .find(|summary| summary.family == TaskFamily::ChiralFavorable)
            .unwrap();
        assert_eq!(chiral.reference_matches, chiral.n_cases);
        assert_eq!(
            chiral.ablated_matches,
            chiral.n_cases - chiral.parity_odd_active_cases
        );
    }
}

#[test]
fn hand_calculated_gamma_zero_score_and_parity_odd_channel() {
    let query = Chiral6::new([1.0, 0.5, -1.0], [0.5, -1.5, 1.0]).unwrap();
    let key = Chiral6::new([-0.5, 1.0, 1.5], [1.0, 0.5, -2.0]).unwrap();
    // s = -0.5 + 0.5 - 1.5 + 0.5 - 0.75 - 2.0 = -3.75
    // chi = sum q+_i k-_i - q-_i k+_i = (1 - (-0.25)) + (0.25 - (-1.5)) + (2 - 1.5) = 3.5
    assert_eq!(gamma_zero_chiral_score(query, key).unwrap(), -3.75);
    assert_eq!(chiral_parity_odd_channel(query, key).unwrap(), 3.5);
    assert_eq!(
        chiral_score(query, key, MATCHED_CHIRAL_WEIGHTS).unwrap(),
        -3.75 + 3.5
    );
}

#[test]
fn tampered_reports_fail_closed() {
    let report = smoke(DataSplit::Development);
    let tamper = |mutate: &dyn Fn(&mut ChiralGammaZeroAblationReport), reason: &str| {
        let mut tampered = report.clone();
        mutate(&mut tampered);
        assert_eq!(rejection(&tampered), reason);
    };
    tamper(
        &|r| r.ablation_contract = "tdi25-chiral-gamma-zero-ablation-v0",
        "contract_drift",
    );
    tamper(&|r| r.population_contract = "legacy", "population_drift");
    tamper(&|r| r.chiral_contract = "legacy", "chiral_contract_drift");
    tamper(
        &|r| r.reference_weights.gamma = 2.0,
        "reference_weights_drift",
    );
    tamper(&|r| r.ablated_weights.gamma = 1.0, "ablated_weights_drift");
    tamper(&|r| r.ablated_weights.beta = 0.5, "ablated_weights_drift");
    tamper(
        &|r| r.ablated_capacity = ParameterReadoutCapacity::reference_t6(),
        "capacity_mismatch",
    );
    tamper(
        &|r| {
            r.cases.pop();
        },
        "case_count",
    );
    tamper(&|r| r.cases.swap(0, 1), "case_order");
    let active = report
        .cases
        .iter()
        .position(|case| case.parity_odd_channel != 0.0)
        .unwrap();
    tamper(
        &|r| r.cases[active].ablated_score = r.cases[active].reference_score,
        "parity_odd_residual",
    );
    tamper(
        &|r| r.cases[0].enantiomorphic_split_closed = false,
        "enantiomorphic_split",
    );
    tamper(&|r| r.families[0].ablated_matches += 1, "family_summary");
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
fn inactive_parity_odd_channel_must_be_an_exact_no_op() {
    let report = smoke(DataSplit::Development);
    let Some(inactive) = report
        .cases
        .iter()
        .position(|case| case.parity_odd_channel == 0.0)
    else {
        return;
    };
    let mut tampered = report.clone();
    tampered.cases[inactive].ablated_matches_target =
        !tampered.cases[inactive].reference_matches_target;
    assert_eq!(rejection(&tampered), "inactive_channel_drift");
}

#[test]
fn protected_or_final_labels_and_bad_budgets_never_score() {
    for label in ["protected", "final", "holdout", ""] {
        assert_eq!(
            run_chiral_gamma_zero_ablation_for_label(label, StageCPreflightBudget::smoke()),
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
            run_chiral_gamma_zero_ablation(DataSplit::Development, budget),
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
    let left = run_chiral_gamma_zero_ablation(DataSplit::Validation, budget).unwrap();
    let right = run_chiral_gamma_zero_ablation(DataSplit::Validation, budget).unwrap();
    assert_eq!(left, right);
}
