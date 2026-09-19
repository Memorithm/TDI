#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi21_attention::AttentionMode;
use tdi_ai::experimental::tdi21_relational_address_search::{
    development_from_relational_tasks, fit_relational_address,
};
use tdi_ai::experimental::tdi21_relational_matched_component::
    evaluate_matched_component_episode;
use tdi_ai::experimental::tdi21_relational_tasks::{
    DevelopmentRelationalSet, ValidationRelationalSet,
};

fn selected() -> tdi_ai::experimental::tdi21_relational_address_search::AddressSearchResult {
    let development =
        development_from_relational_tasks(&DevelopmentRelationalSet::v1()).unwrap();
    fit_relational_address(&development).unwrap()
}

fn assert_report(
    mode: AttentionMode,
    episode: &tdi_ai::experimental::tdi21_relational_tasks::RelationalEpisode,
) {
    let selected = selected();
    let report = evaluate_matched_component_episode(mode, &selected, episode).unwrap();

    assert!(report.attention.correct);
    assert!(report.boolean.correct);
    assert_eq!(report.attention.observed, episode.expected());
    assert_eq!(report.boolean.observed, episode.expected());
    assert_eq!(report.development_samples, 5);
    assert_eq!(report.development_bit_mismatches, 0);
    assert_eq!(report.development_search_work.candidate_rules_evaluated, 8 * 18);
    assert_eq!(report.development_search_work.sample_bit_evaluations, 8 * 18 * 5);

    let facts = episode.facts.len() as u64;
    let hops = episode.query.relations.len() as u64;
    assert_eq!(report.attention.work.pairwise_scores, facts * hops);
    assert_eq!(report.attention.work.lookups, hops);
    assert_eq!(report.boolean.counters.work.pairwise_comparisons, 0);
    assert_eq!(report.boolean.counters.work.memory_reads, hops);
    assert_eq!(report.boolean.counters.work.memory_writes, facts);
    assert_eq!(report.boolean.address_work.predictions, facts + hops);
    assert_eq!(report.boolean.address_work.anf_term_evaluations, (facts + hops) * 6);

    assert_eq!(report.envelope.attention_history_capacity, episode.facts.len());
    assert!(report.envelope.boolean_declared_component_bits <= report.envelope.attention_declared_component_bits);
    assert_eq!(
        report.envelope.attention_declared_component_bits
            - report.envelope.boolean_declared_component_bits,
        report.envelope.unallocated_component_bits
    );
}

#[test]
fn both_reference_modes_and_b4_share_all_current_relational_episodes() {
    for mode in [AttentionMode::DenseQk, AttentionMode::BinaryQk] {
        for episode in DevelopmentRelationalSet::v1().episodes() {
            assert_report(mode, episode);
        }
        for episode in ValidationRelationalSet::v1().episodes() {
            assert_report(mode, episode);
        }
    }
}

#[test]
fn task_sized_distractor_episode_has_explicit_component_ceiling_and_scan_cost() {
    let development = DevelopmentRelationalSet::v1();
    let clean = &development.episodes()[1];
    let distracted = &development.episodes()[3];
    let selected = selected();

    for mode in [AttentionMode::DenseQk, AttentionMode::BinaryQk] {
        let clean_report =
            evaluate_matched_component_episode(mode, &selected, clean).unwrap();
        let distracted_report =
            evaluate_matched_component_episode(mode, &selected, distracted).unwrap();
        assert!(clean_report.attention.correct && clean_report.boolean.correct);
        assert!(distracted_report.attention.correct && distracted_report.boolean.correct);
        assert_eq!(clean.query.relations.len(), distracted.query.relations.len());
        assert!(
            distracted_report.attention.work.pairwise_scores
                > clean_report.attention.work.pairwise_scores
        );
        assert_eq!(
            distracted_report.boolean.counters.work.memory_reads,
            clean_report.boolean.counters.work.memory_reads
        );
    }

    let binary = evaluate_matched_component_episode(
        AttentionMode::BinaryQk,
        &selected,
        distracted,
    )
    .unwrap();
    assert_eq!(binary.envelope.attention_declared_component_bits, 16_960);
    assert_eq!(binary.envelope.boolean_slots, 126);
    assert_eq!(binary.envelope.boolean_declared_component_bits, 16_773);

    let dense = evaluate_matched_component_episode(
        AttentionMode::DenseQk,
        &selected,
        distracted,
    )
    .unwrap();
    assert_eq!(dense.envelope.attention_declared_component_bits, 33_088);
    assert_eq!(dense.envelope.boolean_slots, 250);
    assert_eq!(dense.envelope.boolean_declared_component_bits, 32_831);
}