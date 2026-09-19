#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi21_attention::{AttentionConfig, AttentionMode};
use tdi_ai::experimental::tdi21_relational_attention::{
    RelationalAttentionConfig, RelationalAttentionError, RelationalAttentionReference,
    evaluate_relational_attention_episode,
};
use tdi_ai::experimental::tdi21_relational_binding::RelationalRead;
use tdi_ai::experimental::tdi21_relational_tasks::{
    DevelopmentRelationalSet, RELATIONAL_V1_ENTITY_BITS, RELATIONAL_V1_RELATION_BITS,
    ValidationRelationalSet,
};

fn config(mode: AttentionMode, history_capacity: usize) -> RelationalAttentionConfig {
    RelationalAttentionConfig {
        attention: AttentionConfig {
            mode,
            history_capacity,
            payload_bits: RELATIONAL_V1_ENTITY_BITS,
            max_events: 128,
        },
        entity_bits: RELATIONAL_V1_ENTITY_BITS,
        relation_bits: RELATIONAL_V1_RELATION_BITS,
    }
}

fn assert_episode(
    mode: AttentionMode,
    episode: &tdi_ai::experimental::tdi21_relational_tasks::RelationalEpisode,
) {
    let outcome = evaluate_relational_attention_episode(config(mode, 16), episode).unwrap();
    assert!(outcome.correct, "mode={mode:?};case={}", episode.case_id);
    assert_eq!(outcome.observed, episode.expected());

    let facts = episode.facts.len() as u64;
    let hops = episode.query.relations.len() as u64;
    let pairwise = facts * hops;
    assert_eq!(outcome.work.writes, facts);
    assert_eq!(outcome.work.lookups, hops);
    assert_eq!(outcome.work.pairwise_scores, pairwise);
    assert_eq!(
        outcome.work.weighted_value_terms,
        pairwise * u64::from(RELATIONAL_V1_ENTITY_BITS)
    );
    assert_eq!(outcome.work.presence_terms, pairwise);
    assert_eq!(outcome.work.exponentials, (facts + 1) * hops);
    assert_eq!(outcome.work.normalizations, (facts + 1) * hops);

    match mode {
        AttentionMode::DenseQk => {
            assert_eq!(outcome.work.float_dot_terms, pairwise * 64);
            assert_eq!(outcome.work.xor_popcount_words, 0);
        }
        AttentionMode::BinaryQk => {
            assert_eq!(outcome.work.float_dot_terms, 0);
            assert_eq!(outcome.work.xor_popcount_words, pairwise);
        }
    }
}

#[test]
fn both_attention_modes_solve_all_declared_relational_splits() {
    for mode in [AttentionMode::DenseQk, AttentionMode::BinaryQk] {
        for episode in DevelopmentRelationalSet::v1().episodes() {
            assert_episode(mode, episode);
        }
        for episode in ValidationRelationalSet::v1().episodes() {
            assert_episode(mode, episode);
        }
    }
}

#[test]
fn distractors_increase_attention_scan_work_even_when_the_answer_is_unchanged() {
    let development = DevelopmentRelationalSet::v1();
    let clean = &development.episodes()[1];
    let distracted = &development.episodes()[3];

    assert_eq!(
        clean.query.relations.len(),
        distracted.query.relations.len()
    );
    assert_eq!(clean.expected(), distracted.expected());
    assert!(distracted.facts.len() > clean.facts.len());

    for mode in [AttentionMode::DenseQk, AttentionMode::BinaryQk] {
        let clean_out = evaluate_relational_attention_episode(config(mode, 16), clean).unwrap();
        let distracted_out =
            evaluate_relational_attention_episode(config(mode, 16), distracted).unwrap();
        assert!(clean_out.correct);
        assert!(distracted_out.correct);
        assert!(
            distracted_out.work.pairwise_scores > clean_out.work.pairwise_scores,
            "attention control must pay for irrelevant retained writes"
        );
    }
}

#[test]
fn episode_capacity_failure_is_rejected_before_any_write_or_lookup() {
    let episode = &DevelopmentRelationalSet::v1().episodes()[2];
    assert!(episode.facts.len() > 2);

    for mode in [AttentionMode::DenseQk, AttentionMode::BinaryQk] {
        let mut reference = RelationalAttentionReference::new(config(mode, 2)).unwrap();
        let before_work = reference.work();
        let before_stored = reference.stored_events();
        assert_eq!(
            reference.run_episode(episode),
            Err(RelationalAttentionError::HistoryCapacityExhausted)
        );
        assert_eq!(reference.work(), before_work);
        assert_eq!(reference.stored_events(), before_stored);
    }
}

#[test]
fn missing_relational_fact_remains_an_explicit_attention_miss() {
    for mode in [AttentionMode::DenseQk, AttentionMode::BinaryQk] {
        let mut reference = RelationalAttentionReference::new(config(mode, 8)).unwrap();
        reference.bind(1, 1, 2).unwrap();
        assert_eq!(reference.recall(2, 1).unwrap(), RelationalRead::Miss);
        assert_eq!(reference.work().pairwise_scores, 1);
    }
}

#[test]
fn path_validation_rejects_errors_before_attention_lookup_work() {
    for mode in [AttentionMode::DenseQk, AttentionMode::BinaryQk] {
        let mut reference = RelationalAttentionReference::new(config(mode, 8)).unwrap();
        reference.bind(1, 1, 2).unwrap();
        let before = reference.work();

        assert_eq!(
            reference.compose_path(&[], 1),
            Err(RelationalAttentionError::EmptyRelationPath)
        );
        assert_eq!(
            reference.compose_path(&[1, 16], 1),
            Err(RelationalAttentionError::RelationOutOfRange)
        );
        assert_eq!(reference.work(), before);
    }
}

#[test]
fn attention_footprints_are_explicit_and_not_claimed_matched_to_boolean_memory() {
    let dense = RelationalAttentionReference::new(config(AttentionMode::DenseQk, 16)).unwrap();
    let binary = RelationalAttentionReference::new(config(AttentionMode::BinaryQk, 16)).unwrap();

    let dense_fp = dense.footprint();
    let binary_fp = binary.footprint();
    assert!(dense_fp.history_component_bits > binary_fp.history_component_bits);
    assert_eq!(
        dense_fp.score_component_bits,
        binary_fp.score_component_bits
    );
    assert!(dense_fp.history_component_bits > 0);
    assert!(binary_fp.history_component_bits > 0);
}
