#![cfg(feature = "experimental")]

//! TDI-24 slice 34: parity-shuffle control (Phase D).
//!
//! Relabels the six carrier slots of query and key with one fixed permutation
//! drawn from the reused Slice-18 registered seed, rejecting draws that keep or
//! swap the parity sectors as blocks. Weights and capacity are unchanged; the
//! `H+`/`H-` structure is destroyed reproducibly. These tests qualify software
//! semantics and exact identities only: no training, no protected/final access,
//! no attribution or scientific claim.

use tdi_ai::experimental::tdi24_eval::{
    C6_REFERENCE_WEIGHTS, EvalArm, EvalError, EvalOutcome, MAX_PARITY_SHUFFLE_DRAWS,
    MAX_PREFLIGHT_PAIRS_PER_FAMILY, PARITY_SHUFFLE_CONTROL_CONTRACT, ParityShuffle,
    ParityShuffleControlReport, STAGE_C_PREFLIGHT_FAMILIES, StageCPreflightBudget,
    TrainableCapacity, parity_shuffle_from_seed, run_parity_shuffle_control,
    run_parity_shuffle_control_for_label, run_stage_c_preflight, validate_parity_shuffle,
    validate_parity_shuffle_control_report,
};
use tdi_ai::experimental::tdi24_tasks::{DataSplit, SeedDomain, TaskFamily, register_seed};

#[test]
fn contract_pin_and_draw_bound_are_declared() {
    assert_eq!(
        PARITY_SHUFFLE_CONTROL_CONTRACT,
        "tdi24-parity-shuffle-control-v1"
    );
    assert_eq!(MAX_PARITY_SHUFFLE_DRAWS, 64);
}

#[test]
fn shuffle_is_reproducible_and_mixes_the_parity_sectors() {
    for split in [DataSplit::Development, DataSplit::Validation] {
        for first_pair_id in 0..16 {
            let seed = register_seed(
                SeedDomain::from_split(split),
                TaskFamily::ReflectionDiscriminative,
                first_pair_id,
            );
            let shuffle = parity_shuffle_from_seed(seed.mixed_seed).unwrap();
            assert_eq!(shuffle, parity_shuffle_from_seed(seed.mixed_seed).unwrap());
            validate_parity_shuffle(&shuffle).unwrap();
            let moves = shuffle.cross_sector_moves();
            assert!(moves == 1 || moves == 2, "moves={moves}");
            let mut sorted = shuffle.permutation;
            sorted.sort_unstable();
            assert_eq!(sorted, [0, 1, 2, 3, 4, 5]);
        }
    }
}

#[test]
fn block_preserving_or_swapping_shuffles_are_rejected() {
    let reject = |permutation: [usize; 6], reason: &'static str| {
        let shuffle = ParityShuffle {
            permutation,
            source_seed: 0,
            accepted_draw: 1,
        };
        assert_eq!(
            validate_parity_shuffle(&shuffle),
            Err(EvalError::ParityShuffleControlInvalid { reason })
        );
    };
    reject([0, 1, 2, 3, 4, 5], "sector_blocks_preserved");
    reject([2, 0, 1, 5, 3, 4], "sector_blocks_preserved");
    reject([3, 4, 5, 0, 1, 2], "sector_blocks_preserved");
    reject([0, 0, 2, 3, 4, 5], "not_a_permutation");
    reject([0, 1, 2, 3, 4, 6], "not_a_permutation");
    let mixing = ParityShuffle {
        permutation: [3, 1, 2, 0, 4, 5],
        source_seed: 0,
        accepted_draw: 1,
    };
    validate_parity_shuffle(&mixing).unwrap();
    assert_eq!(
        validate_parity_shuffle(&ParityShuffle {
            accepted_draw: 0,
            ..mixing
        }),
        Err(EvalError::ParityShuffleControlInvalid {
            reason: "draw_budget_exhausted"
        })
    );
}

#[test]
fn control_runs_on_both_non_final_splits_with_matched_capacity() {
    for split in [DataSplit::Development, DataSplit::Validation] {
        let budget = StageCPreflightBudget::bounded(4, 3);
        let report = run_parity_shuffle_control(split, budget).unwrap();
        validate_parity_shuffle_control_report(&report).unwrap();
        assert_eq!(report.split, split);
        assert_eq!(report.cases.len(), 32);
        assert_eq!(report.reference_weights, C6_REFERENCE_WEIGHTS);
        assert_eq!(report.shuffled_weights, report.reference_weights);
        assert_eq!(report.reference_capacity, TrainableCapacity::reference_c6());
        assert_eq!(report.reference_capacity, report.shuffled_capacity);
        assert_eq!(report.reference_capacity.trainable_parameters, 0);
        assert!(!report.protected_or_final_access);
        assert!(!report.training_executed);
        assert!(!report.scientific_claim);
        assert!(report.experimental_non_final);
        assert_eq!(
            report.registered_seed,
            register_seed(
                SeedDomain::from_split(split),
                TaskFamily::ReflectionDiscriminative,
                3
            )
        );
        for case in &report.cases {
            assert!(case.direct_products_preserved);
        }
        assert_eq!(report.families.len(), STAGE_C_PREFLIGHT_FAMILIES.len());
        for (summary, family) in report.families.iter().zip(STAGE_C_PREFLIGHT_FAMILIES) {
            assert_eq!(summary.family, *family);
            assert_eq!(summary.n_cases, 8);
            assert!(summary.reference_correct <= summary.n_cases);
            assert!(summary.shuffled_correct <= summary.n_cases);
            assert!(summary.parity_odd_changed <= summary.n_cases);
        }
    }
}

#[test]
fn shuffle_changes_the_parity_odd_channel_on_the_case_stream() {
    let budget = StageCPreflightBudget::bounded(MAX_PREFLIGHT_PAIRS_PER_FAMILY, 0);
    for split in [DataSplit::Development, DataSplit::Validation] {
        let report = run_parity_shuffle_control(split, budget).unwrap();
        let changed: u64 = report.families.iter().map(|f| f.parity_odd_changed).sum();
        assert!(changed > 0, "shuffle left every parity-odd channel intact");
    }
}

#[test]
fn reference_side_reproduces_the_stage_c_c6_evaluator_exactly() {
    let budget = StageCPreflightBudget::bounded(3, 1);
    let control = run_parity_shuffle_control(DataSplit::Development, budget).unwrap();
    let preflight = run_stage_c_preflight(DataSplit::Development, budget).unwrap();
    assert_eq!(control.cases.len(), preflight.c6_records.len());
    for (case, record) in control.cases.iter().zip(&preflight.c6_records) {
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
fn control_is_deterministic_and_bounded() {
    let budget = StageCPreflightBudget::bounded(2, 0);
    assert_eq!(
        run_parity_shuffle_control(DataSplit::Validation, budget).unwrap(),
        run_parity_shuffle_control(DataSplit::Validation, budget).unwrap()
    );
    let full = StageCPreflightBudget::bounded(MAX_PREFLIGHT_PAIRS_PER_FAMILY, 0);
    assert_eq!(
        run_parity_shuffle_control(DataSplit::Development, full)
            .unwrap()
            .cases
            .len(),
        64
    );
    assert_eq!(
        run_parity_shuffle_control(
            DataSplit::Development,
            StageCPreflightBudget::bounded(MAX_PREFLIGHT_PAIRS_PER_FAMILY + 1, 0)
        ),
        Err(EvalError::StageCPreflightInvalid {
            reason: "budget_exceeds_case_cap"
        })
    );
}

#[test]
fn protected_or_final_labels_never_generate_a_control_case() {
    for label in ["protected", "final", "holdout", ""] {
        assert_eq!(
            run_parity_shuffle_control_for_label(label, StageCPreflightBudget::bounded(1, 0)),
            Err(EvalError::ProtectedOrFinalSplit)
        );
    }
}

#[test]
fn drifted_control_reports_fail_closed() {
    let report =
        run_parity_shuffle_control(DataSplit::Development, StageCPreflightBudget::bounded(1, 0))
            .unwrap();
    let reject = |mutate: &dyn Fn(&mut ParityShuffleControlReport), reason: &'static str| {
        let mut tampered = report.clone();
        mutate(&mut tampered);
        assert_eq!(
            validate_parity_shuffle_control_report(&tampered),
            Err(EvalError::ParityShuffleControlInvalid { reason })
        );
    };
    reject(
        &|r| r.control_contract = "tdi24-parity-shuffle-control-v0",
        "contract_drift",
    );
    reject(&|r| r.registered_seed.local_seed += 1, "seed_drift");
    reject(
        &|r| r.shuffle.permutation = [0, 1, 2, 3, 4, 5],
        "sector_blocks_preserved",
    );
    reject(
        &|r| {
            let alternative = if r.shuffle.permutation == [3, 1, 2, 0, 4, 5] {
                [0, 3, 2, 1, 4, 5]
            } else {
                [3, 1, 2, 0, 4, 5]
            };
            r.shuffle.permutation = alternative;
        },
        "shuffle_not_reproducible",
    );
    reject(
        &|r| r.reference_weights.gamma = 2.0,
        "reference_weights_drift",
    );
    reject(&|r| r.shuffled_weights.beta = 0.5, "shuffled_weights_drift");
    reject(
        &|r| r.shuffled_capacity.trainable_parameters = 1,
        "capacity_mismatch",
    );
    reject(
        &|r| {
            r.cases.pop();
        },
        "case_count",
    );
    reject(
        &|r| r.cases[0].direct_products_preserved = false,
        "direct_product_multiset_drift",
    );
    reject(
        &|r| r.families[0].shuffled_correct += 1,
        "family_summary_drift",
    );
    reject(
        &|r| r.cases[0].reference_score += 0.5,
        "score_channel_drift",
    );
    reject(&|r| r.cases[0].shuffled_score += 0.5, "score_channel_drift");
    reject(
        &|r| r.cases[0].shuffled_channels.chiral += 0.5,
        "score_channel_drift",
    );
    reject(&|r| r.cases[0].case_id += 1_000, "case_evidence_drift");
    reject(
        // Self-consistent forgery: beta = 0, so the mirror-even channel does
        // not enter the score and only regeneration can expose it.
        &|r| r.cases[0].shuffled_channels.mirrored += 0.5,
        "case_evidence_drift",
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
