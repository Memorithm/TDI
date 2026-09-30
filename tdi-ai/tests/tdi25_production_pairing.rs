//! Production-API regressions: the library is compiled without cfg(test).
//! No synthetic outcome constructor, record mutation or oracle access is used.
#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi25_eval::{
    C6EvaluatorRun, EvalError, EvaluatorConfig, G6EvaluatorRun, MetricRegistry,
    REQUIRED_SYNTHESIS_FAMILIES, RevealedMatchOutcome, T6EvaluatorRun,
    revealed_matches_from_c6_records, revealed_matches_from_g6_records,
    revealed_matches_from_t6_records, summarize_g6_attribution_contrast_by_seed_block,
    summarize_paired_uncertainty_by_seed_block, summarize_paired_uncertainty_from_records,
    synthesize_family_stratified_from_revealed_outcomes,
};
use tdi_ai::experimental::tdi25_matched_matrix::matched_family_paths;
use tdi_ai::experimental::tdi25_tasks::{
    DataSplit, mixed_geometry_pair_in_split, neutral_control_pair_in_split, seal_mixed_geometry,
    seal_neutral_control,
};
use tdi_ai::experimental::tdi25_torsor_chiral::TaskFamily;

fn mixed_runs(split: DataSplit, seed: u64) -> (T6EvaluatorRun, C6EvaluatorRun) {
    let pair = mixed_geometry_pair_in_split(seed, split).unwrap();
    let cases = [
        seal_mixed_geometry(pair.base, pair.base_oracle),
        seal_mixed_geometry(pair.transformed, pair.transformed_oracle),
    ];
    let mut t6 = T6EvaluatorRun::open(EvaluatorConfig::t6(split)).unwrap();
    let mut c6 = C6EvaluatorRun::open(EvaluatorConfig::c6(split)).unwrap();
    for case in &cases {
        t6.evaluate_mixed(case).unwrap();
        c6.evaluate_mixed(case).unwrap();
    }
    (t6, c6)
}

fn mixed_outcomes(
    split: DataSplit,
    seed: u64,
) -> (Vec<RevealedMatchOutcome>, Vec<RevealedMatchOutcome>) {
    let (t6, c6) = mixed_runs(split, seed);
    (
        revealed_matches_from_t6_records(t6.records(), split).unwrap(),
        revealed_matches_from_c6_records(c6.records(), split).unwrap(),
    )
}

#[test]
fn genuine_mixed_records_cannot_become_a_primary_comparison() {
    for split in [DataSplit::Development, DataSplit::Validation] {
        let (t6, c6) = mixed_runs(split, 5);
        let left = revealed_matches_from_t6_records(t6.records(), split).unwrap();
        let right = revealed_matches_from_c6_records(c6.records(), split).unwrap();
        assert_eq!(left.len(), 2);
        assert!(
            left.iter()
                .chain(&right)
                .all(|outcome| outcome.matches_oracle)
        );
        let block = left[0].seed_block;
        let registry = MetricRegistry::pinned();
        assert_eq!(
            summarize_paired_uncertainty_from_records(
                split,
                TaskFamily::Mixed,
                block,
                t6.records(),
                c6.records(),
                &registry
            ),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "missing_common_target_contract"
            }),
        );
        assert_eq!(
            summarize_paired_uncertainty_by_seed_block(
                split,
                TaskFamily::Mixed,
                block,
                &left,
                &right,
                &registry
            ),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "missing_common_target_contract"
            }),
        );
    }
}

#[test]
fn replicated_real_blocks_do_not_hide_the_missing_target() {
    for split in [DataSplit::Development, DataSplit::Validation] {
        let (mut left, mut right) = mixed_outcomes(split, 5);
        let (left_next, right_next) = mixed_outcomes(split, 6);
        left.extend(left_next);
        right.extend(right_next);
        assert_eq!(
            synthesize_family_stratified_from_revealed_outcomes(
                split,
                &left,
                &right,
                &MetricRegistry::pinned()
            ),
            Err(EvalError::FamilyStratifiedSynthesisInvalid {
                reason: "missing_common_target_contract"
            }),
        );
    }
}

#[test]
fn real_arm_swaps_still_fail_before_target_admission() {
    let split = DataSplit::Development;
    let (left, right) = mixed_outcomes(split, 5);
    assert_eq!(
        summarize_paired_uncertainty_by_seed_block(
            split,
            TaskFamily::Mixed,
            left[0].seed_block,
            &right,
            &left,
            &MetricRegistry::pinned()
        ),
        Err(EvalError::PairedUncertaintyInvalid {
            reason: "arm_mismatch"
        }),
    );
}

#[test]
fn real_block_cannot_be_relabeled_as_four_genuine_families() {
    let split = DataSplit::Development;
    let (left, right) = mixed_outcomes(split, 5);
    let mut relabeled_left = Vec::new();
    let mut relabeled_right = Vec::new();
    for family in REQUIRED_SYNTHESIS_FAMILIES {
        for (source, destination) in [(&left, &mut relabeled_left), (&right, &mut relabeled_right)]
        {
            let mut outcomes = source.clone();
            for outcome in &mut outcomes {
                outcome.family = *family;
            }
            destination.extend(outcomes);
        }
    }
    assert_eq!(
        synthesize_family_stratified_from_revealed_outcomes(
            split,
            &relabeled_left,
            &relabeled_right,
            &MetricRegistry::pinned()
        ),
        Err(EvalError::PairedUncertaintyInvalid {
            reason: "outcome_integrity_mismatch"
        }),
    );
}

#[test]
fn different_real_populations_remain_unpairable() {
    let split = DataSplit::Validation;
    let (left, _) = mixed_outcomes(split, 5);
    let (_, right) = mixed_outcomes(split, 6);
    assert_eq!(
        synthesize_family_stratified_from_revealed_outcomes(
            split,
            &left,
            &right,
            &MetricRegistry::pinned()
        ),
        Err(EvalError::PairedUncertaintyInvalid {
            reason: "pair_identity_mismatch"
        }),
    );
}

#[test]
fn genuine_g6_attribution_is_retained_but_never_primary() {
    for split in [DataSplit::Development, DataSplit::Validation] {
        let pair = neutral_control_pair_in_split(7, split).unwrap();
        let cases = [
            seal_neutral_control(pair.class_a, pair.class_a_oracle),
            seal_neutral_control(pair.class_b, pair.class_b_oracle),
        ];
        let mut run = G6EvaluatorRun::open(EvaluatorConfig::g6(split)).unwrap();
        for case in &cases {
            run.evaluate_neutral_control(case).unwrap();
        }
        let outcomes = revealed_matches_from_g6_records(run.records(), split).unwrap();
        let block = outcomes[0].seed_block;
        let registry = MetricRegistry::pinned();
        let summary =
            summarize_g6_attribution_contrast_by_seed_block(split, block, &outcomes, &registry)
                .unwrap();
        assert_eq!(summary.n_cases, 2);
        assert_eq!(summary.g6_accuracy, 1.0);
        assert_eq!(
            summarize_paired_uncertainty_by_seed_block(
                split,
                TaskFamily::Neutral,
                block,
                &outcomes,
                &outcomes,
                &registry
            ),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "arm_mismatch"
            }),
        );
    }
}

#[test]
fn no_existing_primary_family_has_a_common_target() {
    assert_eq!(REQUIRED_SYNTHESIS_FAMILIES.len(), 4);
    for family in REQUIRED_SYNTHESIS_FAMILIES {
        let paths = matched_family_paths(*family);
        assert!(!paths.is_complete());
        assert_eq!(paths.shared_target_contract(), None);
    }
}
