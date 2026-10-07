//! TDI-25 slice 34: chiral parity-shuffle control.
//!
//! Phase-D control on the bounded matched Development/Validation population:
//! query and key carrier slots are relabelled by the unchanged TDI-24
//! slice-34 parity shuffle drawn from an already-registered TDI-25 seed. The
//! six values, weights and capacity are preserved; the `H+`/`H-` semantics are
//! destroyed. No protected/final execution, no training, no scientific claim.
#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi24_chiral::{CHIRAL_CONTRACT, Chiral6, chiral_score};
use tdi_ai::experimental::tdi24_eval::{
    PARITY_SHUFFLE_CONTROL_CONTRACT, ParityShuffle, parity_shuffle_from_seed,
};
use tdi_ai::experimental::tdi25_eval::matched_reference::{
    MATCHED_CHIRAL_WEIGHTS, MATCHED_POPULATION_CONTRACT, MatchedPrimaryRun,
};
use tdi_ai::experimental::tdi25_eval::{
    CHIRAL_PARITY_SHUFFLE_CONTROL_CONTRACT, ChiralParityShuffleControlReport, EvalError,
    ParameterReadoutCapacity, REQUIRED_SYNTHESIS_FAMILIES, StageCPreflightBudget,
    chiral_parity_odd_channel, chiral_parity_shuffle_registered_seed, parity_shuffled_chiral_score,
    run_chiral_parity_shuffle_control, run_chiral_parity_shuffle_control_for_label,
    validate_chiral_parity_shuffle_control_report,
};
use tdi_ai::experimental::tdi25_tasks::{DataSplit, SeedDomain, mix_registered_seed};
use tdi_ai::experimental::tdi25_torsor_chiral::TaskFamily;

fn smoke(split: DataSplit) -> ChiralParityShuffleControlReport {
    run_chiral_parity_shuffle_control(split, StageCPreflightBudget::smoke()).unwrap()
}

fn rejection(report: &ChiralParityShuffleControlReport) -> &'static str {
    match validate_chiral_parity_shuffle_control_report(report) {
        Err(EvalError::ChiralParityShuffleControlInvalid { reason }) => reason,
        other => panic!("expected a chiral parity-shuffle rejection, got {other:?}"),
    }
}

#[test]
fn control_reuses_the_tdi24_shuffle_and_an_already_registered_seed() {
    assert_eq!(
        CHIRAL_PARITY_SHUFFLE_CONTROL_CONTRACT,
        "tdi25-chiral-parity-shuffle-control-v1"
    );
    for split in [DataSplit::Development, DataSplit::Validation] {
        let seed = chiral_parity_shuffle_registered_seed(split);
        assert_eq!(seed.domain, SeedDomain::from_split(split));
        assert_eq!(seed.family, TaskFamily::ChiralFavorable);
        assert_eq!(seed.local_seed, 0);
        // Same mixed seed as the first ChiralFavorable matched-population case.
        assert_eq!(
            seed.mixed_seed,
            mix_registered_seed(
                SeedDomain::from_split(split),
                TaskFamily::ChiralFavorable,
                0
            )
        );
        let report = smoke(split);
        assert_eq!(report.shuffle_contract, PARITY_SHUFFLE_CONTROL_CONTRACT);
        assert_eq!(report.registered_seed, seed);
        assert_eq!(
            report.shuffle,
            parity_shuffle_from_seed(seed.mixed_seed).unwrap()
        );
        let moves = report.shuffle.cross_sector_moves();
        assert!(
            (1..3).contains(&moves),
            "sectors must be mixed, got {moves}"
        );
    }
}

#[test]
fn smoke_control_covers_every_family_on_both_non_final_splits() {
    for split in [DataSplit::Development, DataSplit::Validation] {
        let budget = StageCPreflightBudget::smoke();
        let report = smoke(split);
        assert_eq!(report.split, split);
        assert_eq!(
            report.control_contract,
            CHIRAL_PARITY_SHUFFLE_CONTROL_CONTRACT
        );
        assert_eq!(report.population_contract, MATCHED_POPULATION_CONTRACT);
        assert_eq!(report.chiral_contract, CHIRAL_CONTRACT);
        assert_eq!(report.cases.len() as u64, budget.cases_per_arm());
        assert_eq!(report.families.len(), REQUIRED_SYNTHESIS_FAMILIES.len());
        for summary in &report.families {
            assert_eq!(summary.n_cases, budget.seed_blocks * budget.cases_per_block);
            assert!(summary.parity_odd_changed <= summary.n_cases);
        }
        // Identical weights and capacity: the shuffle changes no coefficient
        // and adds no parameter.
        assert_eq!(report.reference_weights, MATCHED_CHIRAL_WEIGHTS);
        assert_eq!(report.shuffled_weights, report.reference_weights);
        assert_eq!(report.reference_capacity, report.shuffled_capacity);
        assert_eq!(
            report.reference_capacity,
            ParameterReadoutCapacity::reference_c6()
        );
        assert!(!report.protected_or_final_access);
        assert!(!report.training_executed);
        assert!(!report.scientific_claim);
        assert!(report.experimental_non_final);
        for case in &report.cases {
            assert!(case.six_values_preserved);
            assert!(case.direct_products_preserved);
        }
        // The shuffle is not a no-op on the parity-odd observable.
        assert!(report.families.iter().any(|f| f.parity_odd_changed > 0));
        validate_chiral_parity_shuffle_control_report(&report).unwrap();
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
                let shuffled = chiral_score(
                    report.shuffle.apply(query),
                    report.shuffle.apply(key),
                    MATCHED_CHIRAL_WEIGHTS,
                )
                .unwrap();
                assert_eq!(case.shuffled_score.to_bits(), shuffled.to_bits());
                assert_eq!(
                    case.reference_parity_odd.to_bits(),
                    chiral_parity_odd_channel(query, key).unwrap().to_bits()
                );
                position += 1;
            }
        }
    }
}

#[test]
fn hand_calculated_shuffled_score() {
    let query = Chiral6::new([1.0, 0.5, -1.0], [0.5, -1.5, 1.0]).unwrap();
    let key = Chiral6::new([-0.5, 1.0, 1.5], [1.0, 0.5, -2.0]).unwrap();
    // Swap slots 2 and 3: one parity-even slot moves into the odd sector.
    let shuffle = ParityShuffle {
        permutation: [0, 1, 3, 2, 4, 5],
        source_seed: 0,
        accepted_draw: 1,
    };
    // Pq = (1, 0.5, 0.5 | -1, -1.5, 1), Pk = (-0.5, 1, 1 | 1.5, 0.5, -2)
    // s = -3.75 (same coordinate products)
    // chi = (1.5 - 0.5) + (0.25 + 1.5) + (-1 - 1) = 0.75
    assert_eq!(
        parity_shuffled_chiral_score(query, key, &shuffle).unwrap(),
        -3.75 + 0.75
    );
    assert_eq!(chiral_parity_odd_channel(query, key).unwrap(), 3.5);
    assert_eq!(
        chiral_parity_odd_channel(shuffle.apply(query), shuffle.apply(key)).unwrap(),
        0.75
    );
}

#[test]
fn block_preserving_or_swapping_shuffles_are_rejected() {
    let query = Chiral6::new([1.0, 0.5, -1.0], [0.5, -1.5, 1.0]).unwrap();
    let key = Chiral6::new([-0.5, 1.0, 1.5], [1.0, 0.5, -2.0]).unwrap();
    for permutation in [[0, 1, 2, 3, 4, 5], [3, 4, 5, 0, 1, 2], [2, 0, 1, 4, 5, 3]] {
        let shuffle = ParityShuffle {
            permutation,
            source_seed: 0,
            accepted_draw: 1,
        };
        assert_eq!(
            parity_shuffled_chiral_score(query, key, &shuffle),
            Err(EvalError::ChiralParityShuffleControlInvalid {
                reason: "sector_blocks_preserved"
            })
        );
        let mut report = smoke(DataSplit::Development);
        report.shuffle = shuffle;
        assert_eq!(rejection(&report), "sector_blocks_preserved");
    }
}

#[test]
fn tampered_reports_fail_closed() {
    let report = smoke(DataSplit::Development);
    let tamper = |mutate: &dyn Fn(&mut ChiralParityShuffleControlReport), reason: &str| {
        let mut tampered = report.clone();
        mutate(&mut tampered);
        assert_eq!(rejection(&tampered), reason);
    };
    tamper(
        &|r| r.control_contract = "tdi25-chiral-parity-shuffle-control-v0",
        "contract_drift",
    );
    tamper(&|r| r.population_contract = "legacy", "population_drift");
    tamper(&|r| r.chiral_contract = "legacy", "chiral_contract_drift");
    tamper(&|r| r.shuffle_contract = "legacy", "shuffle_contract_drift");
    tamper(&|r| r.registered_seed.local_seed = 1, "seed_drift");
    tamper(&|r| r.shuffle.source_seed ^= 1, "shuffle_not_reproducible");
    tamper(
        &|r| r.shuffle.permutation.swap(0, 1),
        "shuffle_not_reproducible",
    );
    tamper(
        &|r| r.reference_weights.gamma = 2.0,
        "reference_weights_drift",
    );
    tamper(
        &|r| r.shuffled_weights.gamma = 0.0,
        "shuffled_weights_drift",
    );
    tamper(
        &|r| r.shuffled_capacity = ParameterReadoutCapacity::reference_t6(),
        "capacity_mismatch",
    );
    tamper(
        &|r| {
            r.cases.pop();
        },
        "case_count",
    );
    tamper(&|r| r.cases.swap(0, 1), "case_order");
    tamper(
        &|r| r.cases[0].six_values_preserved = false,
        "six_values_drift",
    );
    tamper(
        &|r| r.cases[0].direct_products_preserved = false,
        "direct_product_multiset_drift",
    );
    tamper(&|r| r.families[0].shuffled_matches += 1, "family_summary");
    // Coherent forgeries that keep every per-case identity intact are still
    // rejected because each case is regenerated from the canonical population.
    tamper(&|r| r.cases[0].shuffled_score += 1.0, "case_evidence_drift");
    tamper(
        &|r| {
            let changed = r.cases[0].reference_parity_odd.to_bits()
                != r.cases[0].shuffled_parity_odd.to_bits();
            r.cases[0].shuffled_parity_odd = r.cases[0].reference_parity_odd;
            if changed {
                let family = r.cases[0].family;
                let summary = r.families.iter_mut().find(|f| f.family == family).unwrap();
                summary.parity_odd_changed -= 1;
            }
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
            run_chiral_parity_shuffle_control_for_label(label, StageCPreflightBudget::smoke()),
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
            run_chiral_parity_shuffle_control(DataSplit::Development, budget),
            Err(EvalError::StageCPreflightInvalid { .. })
        ));
    }
}

#[test]
fn control_is_deterministic() {
    let budget = StageCPreflightBudget {
        seed_blocks: 1,
        cases_per_block: 4,
    };
    let left = run_chiral_parity_shuffle_control(DataSplit::Validation, budget).unwrap();
    let right = run_chiral_parity_shuffle_control(DataSplit::Validation, budget).unwrap();
    assert_eq!(left, right);
}
