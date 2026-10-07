//! TDI-25 slice 35: torsor structure-shuffle control.
//!
//! Phase-D control on the bounded matched Development/Validation population:
//! twist and torsor carrier slots are relabelled by the unchanged TDI-24
//! slice-34 shuffle drawn from the already-registered TDI-25 seed
//! `(split domain, TorsorFavorable, 0)`. The six values, both geometry points
//! and the T6 capacity are preserved; the Varignon pairing semantics are
//! destroyed. No protected/final execution, no training, no scientific claim.
#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi22_torsor::{TORSOR_CONTRACT, Torsor3, Twist3, Vec3};
use tdi_ai::experimental::tdi24_eval::{
    PARITY_SHUFFLE_CONTROL_CONTRACT, ParityShuffle, parity_shuffle_from_seed,
};
use tdi_ai::experimental::tdi25_eval::matched_reference::{
    MATCHED_POPULATION_CONTRACT, MatchedPrimaryRun,
};
use tdi_ai::experimental::tdi25_eval::{
    EvalError, ParameterReadoutCapacity, REQUIRED_SYNTHESIS_FAMILIES, StageCPreflightBudget,
    TORSOR_STRUCTURE_SHUFFLE_CONTROL_CONTRACT, TorsorStructureShuffleControlReport,
    reduction_point_transport_term, run_torsor_structure_shuffle_control,
    run_torsor_structure_shuffle_control_for_label, structure_shuffled_torsor_score,
    torsor_structure_shuffle_registered_seed, validate_torsor_structure_shuffle_control_report,
};
use tdi_ai::experimental::tdi25_tasks::{DataSplit, SeedDomain, mix_registered_seed};
use tdi_ai::experimental::tdi25_torsor_chiral::{TaskFamily, torsor_arm_score};

fn smoke(split: DataSplit) -> TorsorStructureShuffleControlReport {
    run_torsor_structure_shuffle_control(split, StageCPreflightBudget::smoke()).unwrap()
}

fn rejection(report: &TorsorStructureShuffleControlReport) -> &'static str {
    match validate_torsor_structure_shuffle_control_report(report) {
        Err(EvalError::TorsorStructureShuffleControlInvalid { reason }) => reason,
        other => panic!("expected a torsor structure-shuffle rejection, got {other:?}"),
    }
}

fn v(values: [f64; 3]) -> Vec3 {
    Vec3::new(values[0], values[1], values[2]).unwrap()
}

fn carriers(query: [f64; 6], key: [f64; 6], key_position: [f64; 3]) -> (Twist3, Torsor3) {
    (
        Twist3::new(
            v([query[0], query[1], query[2]]),
            v([query[3], query[4], query[5]]),
        )
        .unwrap(),
        Torsor3::new(
            v([key[0], key[1], key[2]]),
            v([key[3], key[4], key[5]]),
            v(key_position),
        )
        .unwrap(),
    )
}

fn permute(values: [f64; 6], shuffle: &ParityShuffle) -> [f64; 6] {
    let mut permuted = [0.0; 6];
    for (slot, value) in permuted.iter_mut().enumerate() {
        *value = values[shuffle.permutation[slot]];
    }
    permuted
}

#[test]
fn control_reuses_the_tdi24_shuffle_and_an_already_registered_seed() {
    assert_eq!(
        TORSOR_STRUCTURE_SHUFFLE_CONTROL_CONTRACT,
        "tdi25-torsor-structure-shuffle-control-v1"
    );
    for split in [DataSplit::Development, DataSplit::Validation] {
        let seed = torsor_structure_shuffle_registered_seed(split);
        assert_eq!(seed.domain, SeedDomain::from_split(split));
        assert_eq!(seed.family, TaskFamily::TorsorFavorable);
        assert_eq!(seed.local_seed, 0);
        // Same mixed seed as the first TorsorFavorable matched-population case.
        assert_eq!(
            seed.mixed_seed,
            mix_registered_seed(
                SeedDomain::from_split(split),
                TaskFamily::TorsorFavorable,
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
            "linear/angular blocks must be mixed, got {moves}"
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
            TORSOR_STRUCTURE_SHUFFLE_CONTROL_CONTRACT
        );
        assert_eq!(report.population_contract, MATCHED_POPULATION_CONTRACT);
        assert_eq!(report.torsor_contract, TORSOR_CONTRACT);
        assert_eq!(report.cases.len() as u64, budget.cases_per_arm());
        assert_eq!(report.families.len(), REQUIRED_SYNTHESIS_FAMILIES.len());
        for summary in &report.families {
            assert_eq!(summary.n_cases, budget.seed_blocks * budget.cases_per_block);
            assert!(summary.transport_changed <= summary.n_cases);
        }
        assert_eq!(report.reference_capacity, report.shuffled_capacity);
        assert_eq!(
            report.reference_capacity,
            ParameterReadoutCapacity::reference_t6()
        );
        assert!(!report.protected_or_final_access);
        assert!(!report.training_executed);
        assert!(!report.scientific_claim);
        assert!(report.experimental_non_final);
        for case in &report.cases {
            assert!(case.six_values_preserved);
            assert!(case.untransported_products_preserved);
        }
        // The shuffle is not a no-op on the Varignon transport term.
        assert!(report.families.iter().any(|f| f.transport_changed > 0));
        validate_torsor_structure_shuffle_control_report(&report).unwrap();
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
                let input = &run.inputs()[index];
                assert_eq!(
                    case.reference_score.to_bits(),
                    run.t6_scores()[index].to_bits()
                );
                assert_eq!(case.reference_matches_target, outcome.matches_oracle);
                let (query, key) = carriers(
                    permute(input.query(), &report.shuffle),
                    permute(input.key(), &report.shuffle),
                    input.key_position(),
                );
                let shuffled = torsor_arm_score(query, key, v(input.query_position())).unwrap();
                assert_eq!(case.shuffled_score.to_bits(), shuffled.to_bits());
                assert_eq!(
                    structure_shuffled_torsor_score(input, &report.shuffle)
                        .unwrap()
                        .to_bits(),
                    shuffled.to_bits()
                );
                let (query, key) = carriers(input.query(), input.key(), input.key_position());
                assert_eq!(
                    case.reference_transport_term.to_bits(),
                    reduction_point_transport_term(query, key, v(input.query_position()))
                        .unwrap()
                        .to_bits()
                );
                position += 1;
            }
        }
    }
}

#[test]
fn hand_calculated_shuffled_transport_term() {
    // Twist v = (0,0,0), omega = (0,0,1); torsor R = (1,0,0), M(P) = 0,
    // P = (0,1,0), Q = 0. Transport term omega.((P - Q) x R) = (0,0,1).((0,1,0) x (1,0,0)) = -1.
    let (query, key) = carriers(
        [0.0, 0.0, 0.0, 0.0, 0.0, 1.0],
        [1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
    );
    let origin = v([0.0, 0.0, 0.0]);
    assert_eq!(
        reduction_point_transport_term(query, key, origin).unwrap(),
        -1.0
    );
    // Swap slots 2 and 3: both carriers are zero there and slot 5 (omega_z)
    // is untouched, so this particular pair keeps its transport term.
    let shuffle = ParityShuffle {
        permutation: [0, 1, 3, 2, 4, 5],
        source_seed: 0,
        accepted_draw: 1,
    };
    let (query, key) = carriers(
        permute([0.0, 0.0, 0.0, 0.0, 0.0, 1.0], &shuffle),
        permute([1.0, 0.0, 0.0, 0.0, 0.0, 0.0], &shuffle),
        [0.0, 1.0, 0.0],
    );
    assert_eq!(
        reduction_point_transport_term(query, key, origin).unwrap(),
        -1.0
    );
    let shuffle = ParityShuffle {
        permutation: [5, 1, 2, 3, 4, 0],
        source_seed: 0,
        accepted_draw: 1,
    };
    let (query, key) = carriers(
        permute([0.0, 0.0, 0.0, 0.0, 0.0, 1.0], &shuffle),
        permute([1.0, 0.0, 0.0, 0.0, 0.0, 0.0], &shuffle),
        [0.0, 1.0, 0.0],
    );
    // omega' = 0 and R' = 0: the transport term vanishes.
    assert_eq!(
        reduction_point_transport_term(query, key, origin).unwrap(),
        0.0
    );
}

#[test]
fn block_preserving_or_swapping_shuffles_are_rejected() {
    let run =
        MatchedPrimaryRun::evaluate(DataSplit::Development, TaskFamily::Neutral, 0, 2).unwrap();
    for permutation in [[0, 1, 2, 3, 4, 5], [3, 4, 5, 0, 1, 2], [2, 0, 1, 4, 5, 3]] {
        let shuffle = ParityShuffle {
            permutation,
            source_seed: 0,
            accepted_draw: 1,
        };
        assert_eq!(
            structure_shuffled_torsor_score(&run.inputs()[0], &shuffle),
            Err(EvalError::TorsorStructureShuffleControlInvalid {
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
    let tamper = |mutate: &dyn Fn(&mut TorsorStructureShuffleControlReport), reason: &str| {
        let mut tampered = report.clone();
        mutate(&mut tampered);
        assert_eq!(rejection(&tampered), reason);
    };
    tamper(
        &|r| r.control_contract = "tdi25-torsor-structure-shuffle-control-v0",
        "contract_drift",
    );
    tamper(&|r| r.population_contract = "legacy", "population_drift");
    tamper(&|r| r.torsor_contract = "legacy", "torsor_contract_drift");
    tamper(&|r| r.shuffle_contract = "legacy", "shuffle_contract_drift");
    tamper(&|r| r.registered_seed.local_seed = 1, "seed_drift");
    tamper(&|r| r.shuffle.source_seed ^= 1, "shuffle_not_reproducible");
    tamper(
        &|r| r.shuffle.permutation.swap(0, 1),
        "shuffle_not_reproducible",
    );
    tamper(
        &|r| r.shuffled_capacity = ParameterReadoutCapacity::reference_c6(),
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
        &|r| r.cases[0].untransported_products_preserved = false,
        "untransported_product_multiset_drift",
    );
    tamper(&|r| r.families[0].shuffled_matches += 1, "family_summary");
    tamper(&|r| r.cases[0].shuffled_score += 1.0, "case_evidence_drift");
    tamper(
        &|r| {
            let changed = r.cases[0].reference_transport_term.to_bits()
                != r.cases[0].shuffled_transport_term.to_bits();
            r.cases[0].shuffled_transport_term = r.cases[0].reference_transport_term;
            if changed {
                let family = r.cases[0].family;
                let summary = r.families.iter_mut().find(|f| f.family == family).unwrap();
                summary.transport_changed -= 1;
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
            run_torsor_structure_shuffle_control_for_label(label, StageCPreflightBudget::smoke()),
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
            run_torsor_structure_shuffle_control(DataSplit::Development, budget),
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
    let left = run_torsor_structure_shuffle_control(DataSplit::Validation, budget).unwrap();
    let right = run_torsor_structure_shuffle_control(DataSplit::Validation, budget).unwrap();
    assert_eq!(left, right);
}
