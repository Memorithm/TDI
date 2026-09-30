//! Public-API integration tests: the library is built without cfg(test).
//! Every correctness bit here originates in the two fixed production scorers.
#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi25_eval::matched_reference::{
    MATCHED_CHIRAL_WEIGHTS, MatchedPrimaryRun, common_target_contract,
};
use tdi_ai::experimental::tdi25_eval::{
    EvalError, MetricRegistry, REQUIRED_SYNTHESIS_FAMILIES, RevealedMatchOutcome,
    UncertaintyMethod, summarize_paired_uncertainty_by_seed_block,
    synthesize_family_stratified_effects, synthesize_family_stratified_from_revealed_outcomes,
};
use tdi_ai::experimental::tdi25_matched_matrix::{
    legacy_matched_family_paths, matched_family_paths, require_complete_primary_matrix,
};
use tdi_ai::experimental::tdi25_tasks::DataSplit;
use tdi_ai::experimental::tdi25_torsor_chiral::TaskFamily;

fn complete_outcomes(
    split: DataSplit,
    blocks: u64,
) -> (Vec<RevealedMatchOutcome>, Vec<RevealedMatchOutcome>) {
    let mut left = Vec::new();
    let mut right = Vec::new();
    for family in REQUIRED_SYNTHESIS_FAMILIES {
        for block in 0..blocks {
            let run = MatchedPrimaryRun::evaluate(split, *family, block, 64).unwrap();
            left.extend_from_slice(run.t6_outcomes());
            right.extend_from_slice(run.c6_outcomes());
        }
    }
    (left, right)
}

#[test]
fn production_synthesis_reaches_all_four_families_with_two_full_blocks() {
    for split in [DataSplit::Development, DataSplit::Validation] {
        let (left, right) = complete_outcomes(split, 2);
        assert_eq!(left.len(), 512);
        let report = synthesize_family_stratified_from_revealed_outcomes(
            split,
            &left,
            &right,
            &MetricRegistry::pinned(),
        )
        .unwrap();
        assert_eq!(report.family_effects.len(), 4);
        assert_eq!(report.seed_block_effects.len(), 8);
        assert!(report.experimental_non_final);
        for (effect, family) in report
            .family_effects
            .iter()
            .zip(REQUIRED_SYNTHESIS_FAMILIES)
        {
            assert_eq!(effect.family, *family);
            assert_eq!(effect.n_pairs, 128);
            assert_eq!(effect.seed_blocks, vec![0, 1]);
            assert_eq!(
                effect.paired_difference_ci.method,
                UncertaintyMethod::ClusterHoeffdingPairedDifference
            );
        }
    }
}

#[test]
fn sealed_production_summaries_also_reach_the_direct_synthesis_api() {
    for split in [DataSplit::Development, DataSplit::Validation] {
        let registry = MetricRegistry::pinned();
        let summaries = REQUIRED_SYNTHESIS_FAMILIES
            .iter()
            .map(|family| {
                let run = MatchedPrimaryRun::evaluate(split, *family, 0, 64).unwrap();
                summarize_paired_uncertainty_by_seed_block(
                    split,
                    *family,
                    0,
                    run.t6_outcomes(),
                    run.c6_outcomes(),
                    &registry,
                )
                .unwrap()
            })
            .collect::<Vec<_>>();
        let report = synthesize_family_stratified_effects(split, &summaries, &registry).unwrap();
        assert_eq!(report.family_effects.len(), 4);
        for summary in &summaries {
            assert_eq!(summary.n_pairs, 64);
            assert_eq!(
                summary.paired_difference_ci.method,
                UncertaintyMethod::BoundedHoeffdingPairedDifference
            );
        }
    }
}

#[test]
fn shared_targets_are_scored_from_actual_common_inputs_not_placeholder_bits() {
    for family in REQUIRED_SYNTHESIS_FAMILIES {
        let run = MatchedPrimaryRun::evaluate(DataSplit::Development, *family, 3, 64).unwrap();
        assert_eq!(run.split(), DataSplit::Development);
        assert_eq!(run.family(), *family);
        assert_eq!(run.seed_block(), 3);
        assert_eq!(run.shared_input_bytes_per_case(), 144);
        for (index, input) in run.inputs().iter().enumerate() {
            let q = input.query();
            let k = input.key();
            let p = input.key_position();
            let x = input.query_position();
            let a = [p[0] - x[0], p[1] - x[1], p[2] - x[2]];
            let cross = [
                a[1] * k[2] - a[2] * k[1],
                a[2] * k[0] - a[0] * k[2],
                a[0] * k[1] - a[1] * k[0],
            ];
            let t: f64 = (0..3)
                .map(|i| q[i] * k[i] + q[i + 3] * (k[i + 3] + cross[i]))
                .sum();
            let d: f64 = (0..6).map(|i| q[i] * k[i]).sum();
            let chi: f64 = (0..3).map(|i| q[i] * k[i + 3] - q[i + 3] * k[i]).sum();
            let c = d + chi;
            let target = match family {
                TaskFamily::TorsorFavorable => t,
                TaskFamily::ChiralFavorable => c,
                TaskFamily::Mixed => (t + c) / 2.0,
                TaskFamily::Neutral => (0..6).map(|i| (q[i] - k[i]) * (q[i] - k[i])).sum(),
            };
            assert_eq!(run.t6_scores()[index], t);
            assert_eq!(run.c6_scores()[index], c);
            let correct = |score: f64| {
                (score - target).abs() <= 1e-12 * (1.0 + score.abs().max(target.abs()))
            };
            assert_eq!(run.t6_outcomes()[index].matches_oracle, correct(t));
            assert_eq!(run.c6_outcomes()[index].matches_oracle, correct(c));
            assert_eq!(run.t6_outcomes()[index].case_id, index as u64);
            assert_eq!(run.c6_outcomes()[index].case_id, index as u64);
        }
    }
    assert_eq!(
        (
            MATCHED_CHIRAL_WEIGHTS.alpha,
            MATCHED_CHIRAL_WEIGHTS.beta,
            MATCHED_CHIRAL_WEIGHTS.gamma
        ),
        (1.0, 0.0, 1.0)
    );
}

#[test]
fn matrix_registers_new_targets_without_upgrading_legacy_records() {
    assert_eq!(require_complete_primary_matrix().unwrap().len(), 4);
    for family in REQUIRED_SYNTHESIS_FAMILIES {
        let paths = matched_family_paths(*family);
        assert!(paths.is_complete());
        assert_eq!(
            paths.shared_target_contract(),
            Some(common_target_contract(*family))
        );
        assert!(paths.t6().is_some());
        assert!(paths.c6().is_some());
        let legacy = legacy_matched_family_paths(*family);
        assert!(!legacy.is_complete());
        assert_eq!(legacy.shared_target_contract(), None);
    }
}

#[test]
fn dropping_a_production_family_still_blocks_pooled_synthesis() {
    let split = DataSplit::Development;
    let (mut left, mut right) = complete_outcomes(split, 1);
    left.retain(|outcome| outcome.family != TaskFamily::Neutral);
    right.retain(|outcome| outcome.family != TaskFamily::Neutral);
    assert_eq!(
        synthesize_family_stratified_from_revealed_outcomes(
            split,
            &left,
            &right,
            &MetricRegistry::pinned()
        ),
        Err(EvalError::FamilyStratifiedSynthesisInvalid {
            reason: "missing_family"
        })
    );
}

#[test]
fn swapped_production_arms_cannot_reverse_a_primary_result() {
    let split = DataSplit::Development;
    let run = MatchedPrimaryRun::evaluate(split, TaskFamily::TorsorFavorable, 0, 64).unwrap();
    assert_eq!(
        summarize_paired_uncertainty_by_seed_block(
            split,
            TaskFamily::TorsorFavorable,
            0,
            run.c6_outcomes(),
            run.t6_outcomes(),
            &MetricRegistry::pinned()
        ),
        Err(EvalError::PairedUncertaintyInvalid {
            reason: "arm_mismatch"
        })
    );
}

#[test]
fn every_public_evidence_field_remains_integrity_bound() {
    let split = DataSplit::Development;
    let run = MatchedPrimaryRun::evaluate(split, TaskFamily::Mixed, 0, 2).unwrap();
    for field in 0..5 {
        let mut left = run.t6_outcomes().to_vec();
        match field {
            0 => left[0].family = TaskFamily::Neutral,
            1 => left[0].split = DataSplit::Validation,
            2 => left[0].seed_block += 1,
            3 => left[0].case_id += 1,
            _ => left[0].matches_oracle = !left[0].matches_oracle,
        }
        assert_eq!(
            summarize_paired_uncertainty_by_seed_block(
                split,
                TaskFamily::Mixed,
                0,
                &left,
                run.c6_outcomes(),
                &MetricRegistry::pinned()
            ),
            Err(EvalError::PairedUncertaintyInvalid {
                reason: "outcome_integrity_mismatch"
            })
        );
    }
}

#[test]
fn repeated_blocks_and_mismatched_populations_are_rejected() {
    let split = DataSplit::Validation;
    let (mut left, mut right) = complete_outcomes(split, 1);
    left.push(left[0].clone());
    right.push(right[0].clone());
    assert_eq!(
        synthesize_family_stratified_from_revealed_outcomes(
            split,
            &left,
            &right,
            &MetricRegistry::pinned()
        ),
        Err(EvalError::PairedUncertaintyInvalid {
            reason: "duplicate_pair_identity"
        })
    );
    let a = MatchedPrimaryRun::evaluate(split, TaskFamily::Mixed, 0, 2).unwrap();
    let b = MatchedPrimaryRun::evaluate(split, TaskFamily::Mixed, 1, 2).unwrap();
    assert_eq!(
        synthesize_family_stratified_from_revealed_outcomes(
            split,
            a.t6_outcomes(),
            b.c6_outcomes(),
            &MetricRegistry::pinned()
        ),
        Err(EvalError::PairedUncertaintyInvalid {
            reason: "pair_identity_mismatch"
        })
    );
}

#[test]
fn non_final_boundaries_and_case_budget_remain_closed() {
    assert!(DataSplit::parse("protected").is_err());
    assert!(DataSplit::parse("final").is_err());
    for n in [0, 1, 65, u64::MAX] {
        assert_eq!(
            MatchedPrimaryRun::evaluate(DataSplit::Development, TaskFamily::Neutral, 0, n),
            Err(EvalError::InvalidBudget)
        );
    }
    let a = MatchedPrimaryRun::evaluate(DataSplit::Development, TaskFamily::Mixed, 0, 2).unwrap();
    let b = MatchedPrimaryRun::evaluate(DataSplit::Validation, TaskFamily::Mixed, 0, 2).unwrap();
    assert_eq!(
        synthesize_family_stratified_from_revealed_outcomes(
            DataSplit::Development,
            a.t6_outcomes(),
            b.c6_outcomes(),
            &MetricRegistry::pinned()
        ),
        Err(EvalError::PairedUncertaintyInvalid {
            reason: "pair_identity_mismatch"
        })
    );
}
