#![cfg(feature = "experimental")]

use std::collections::BTreeSet;
use tdi_ai::experimental::tdi21_relational_binding::{RelationalConfig, RelationalRead};
use tdi_ai::experimental::tdi21_relational_tasks::{
    DevelopmentRelationalSet, RELATIONAL_EPISODES_PER_SPLIT, RelationalSplit,
    ValidationRelationalSet, evaluate_relational_episode, exact_split_overlap,
};
use tdi_ai::experimental::tdi21_stream::{MemoryMode, StreamConfig};

fn config() -> RelationalConfig {
    RelationalConfig {
        stream: StreamConfig {
            mode: MemoryMode::TwoWay,
            slots: 128,
            payload_bits: 8,
            max_events: 128,
            route_salt: 0x5444_4932_3100_0021,
        },
        entity_bits: 8,
        relation_bits: 8,
    }
}

#[test]
fn v1_splits_are_typed_fixed_and_exactly_disjoint() {
    let development = DevelopmentRelationalSet::v1();
    let validation = ValidationRelationalSet::v1();
    assert_eq!(development.episodes().len(), RELATIONAL_EPISODES_PER_SPLIT);
    assert_eq!(validation.episodes().len(), RELATIONAL_EPISODES_PER_SPLIT);
    assert!(!exact_split_overlap(&development, &validation));

    for (index, episode) in development.episodes().iter().enumerate() {
        assert_eq!(episode.split, RelationalSplit::Development);
        assert_eq!(usize::from(episode.case_id), index);
    }
    for (index, episode) in validation.episodes().iter().enumerate() {
        assert_eq!(episode.split, RelationalSplit::Validation);
        assert_eq!(usize::from(episode.case_id), index);
    }
}

#[test]
fn entity_and_relation_identifier_namespaces_do_not_overlap_across_splits() {
    let development = DevelopmentRelationalSet::v1();
    let validation = ValidationRelationalSet::v1();

    let mut development_entities = BTreeSet::new();
    let mut development_relations = BTreeSet::new();
    for episode in development.episodes() {
        development_entities.insert(episode.query.subject);
        development_relations.extend(episode.query.relations.iter().copied());
        for fact in &episode.facts {
            development_entities.insert(fact.subject);
            development_entities.insert(fact.object);
            development_relations.insert(fact.relation);
        }
    }

    let mut validation_entities = BTreeSet::new();
    let mut validation_relations = BTreeSet::new();
    for episode in validation.episodes() {
        validation_entities.insert(episode.query.subject);
        validation_relations.extend(episode.query.relations.iter().copied());
        for fact in &episode.facts {
            validation_entities.insert(fact.subject);
            validation_entities.insert(fact.object);
            validation_relations.insert(fact.relation);
        }
    }

    assert!(development_entities.is_disjoint(&validation_entities));
    assert!(development_relations.is_disjoint(&validation_relations));
}

fn assert_episode_accounting(
    outcome: &tdi_ai::experimental::tdi21_relational_tasks::RelationalEpisodeOutcome,
    fact_count: usize,
    hop_count: usize,
) {
    assert_eq!(outcome.counters.work.memory_writes, fact_count as u64);
    assert_eq!(outcome.counters.work.memory_reads, hop_count as u64);
    assert_eq!(
        outcome.relational_work.address_derivations,
        (fact_count + hop_count) as u64
    );
    assert_eq!(outcome.counters.work.pairwise_comparisons, 0);
}

#[test]
fn all_v1_development_and_validation_episodes_retrieve_the_declared_answer() {
    for development in DevelopmentRelationalSet::v1().episodes() {
        let outcome = evaluate_relational_episode(config(), development).unwrap();
        assert!(outcome.correct, "development case {}", development.case_id);
        assert_eq!(outcome.observed, development.expected());
        assert_episode_accounting(
            &outcome,
            development.facts.len(),
            development.query.relations.len(),
        );
    }

    for validation in ValidationRelationalSet::v1().episodes() {
        let outcome = evaluate_relational_episode(config(), validation).unwrap();
        assert!(outcome.correct, "validation case {}", validation.case_id);
        assert_eq!(outcome.observed, validation.expected());
        assert_episode_accounting(
            &outcome,
            validation.facts.len(),
            validation.query.relations.len(),
        );
    }
}

#[test]
fn split_topology_matches_while_identifiers_change() {
    let development = DevelopmentRelationalSet::v1();
    let validation = ValidationRelationalSet::v1();

    for (left, right) in development.episodes().iter().zip(validation.episodes()) {
        assert_eq!(left.case_id, right.case_id);
        assert_eq!(left.facts.len(), right.facts.len());
        assert_eq!(left.query.relations.len(), right.query.relations.len());
        assert_ne!(left.query.subject, right.query.subject);
        assert_ne!(left.expected(), right.expected());
        assert!(matches!(left.expected(), RelationalRead::Hit(_)));
        assert!(matches!(right.expected(), RelationalRead::Hit(_)));
    }
}
