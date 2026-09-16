#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi21_relational_binding::{
    MAX_COMPOSITION_HOPS, RelationalBinder, RelationalConfig, RelationalError, RelationalRead,
};
use tdi_ai::experimental::tdi21_stream::{MemoryMode, StreamConfig};

fn config(slots: usize) -> RelationalConfig {
    RelationalConfig {
        stream: StreamConfig {
            mode: MemoryMode::TwoWay,
            slots,
            payload_bits: 8,
            max_events: 128,
            route_salt: 0x5444_4932_3100_0021,
        },
        entity_bits: 8,
        relation_bits: 8,
    }
}

#[test]
fn exact_one_hop_binding_uses_no_pairwise_scoring() {
    let mut binder = RelationalBinder::new(config(32)).unwrap();
    binder.bind(3, 7, 11).unwrap();
    assert_eq!(binder.recall(3, 7).unwrap(), RelationalRead::Hit(11));
    assert_eq!(binder.recall(3, 8).unwrap(), RelationalRead::Miss);
    assert_eq!(binder.counters().work.pairwise_comparisons, 0);
}

#[test]
fn two_hop_composition_recovers_the_terminal_object() {
    let mut binder = RelationalBinder::new(config(64)).unwrap();
    binder.bind(1, 3, 5).unwrap();
    binder.bind(2, 5, 9).unwrap();
    assert_eq!(binder.compose2(1, 2, 3).unwrap(), RelationalRead::Hit(9));
    assert_eq!(binder.counters().work.memory_reads, 2);
    assert_eq!(binder.counters().work.pairwise_comparisons, 0);
}

#[test]
fn three_hop_path_is_composed_with_one_lookup_per_hop() {
    let mut binder = RelationalBinder::new(config(128)).unwrap();
    binder.bind(1, 3, 5).unwrap();
    binder.bind(2, 5, 9).unwrap();
    binder.bind(4, 9, 13).unwrap();
    assert_eq!(
        binder.compose_path(&[1, 2, 4], 3).unwrap(),
        RelationalRead::Hit(13)
    );
    assert_eq!(binder.counters().work.memory_reads, 3);
    assert_eq!(binder.counters().work.pairwise_comparisons, 0);
}

#[test]
fn missing_intermediate_remains_missing_not_false_or_zero() {
    let mut binder = RelationalBinder::new(config(32)).unwrap();
    binder.bind(2, 5, 0).unwrap();
    assert_eq!(binder.compose2(1, 2, 3).unwrap(), RelationalRead::Miss);
    assert_eq!(binder.counters().work.memory_reads, 1);
}

#[test]
fn consistent_identifier_renaming_preserves_relational_structure() {
    let mut first = RelationalBinder::new(config(64)).unwrap();
    first.bind(1, 3, 5).unwrap();
    first.bind(2, 5, 9).unwrap();
    assert_eq!(first.compose2(1, 2, 3).unwrap(), RelationalRead::Hit(9));

    let mut renamed = RelationalBinder::new(config(64)).unwrap();
    renamed.bind(6, 31, 47).unwrap();
    renamed.bind(4, 47, 83).unwrap();
    assert_eq!(renamed.compose2(6, 4, 31).unwrap(), RelationalRead::Hit(83));
    assert_eq!(renamed.counters().work.pairwise_comparisons, 0);
}

#[test]
fn irrelevant_relations_do_not_change_an_existing_two_hop_answer_without_eviction() {
    let mut binder = RelationalBinder::new(config(128)).unwrap();
    binder.bind(1, 3, 5).unwrap();
    binder.bind(2, 5, 9).unwrap();
    for i in 0..16 {
        binder.bind(7, 100 + i, 150 + i).unwrap();
    }
    assert_eq!(binder.compose2(1, 2, 3).unwrap(), RelationalRead::Hit(9));
}

#[test]
fn bounded_memory_loss_is_explicit_not_a_false_hit() {
    let mut binder = RelationalBinder::new(config(2)).unwrap();
    binder.bind(1, 3, 5).unwrap();
    binder.bind(2, 5, 9).unwrap();
    binder.bind(3, 7, 11).unwrap();
    assert_eq!(binder.compose2(1, 2, 3).unwrap(), RelationalRead::Miss);
    assert!(binder.counters().work.memory_replacements > 0);
    assert_eq!(binder.counters().work.pairwise_comparisons, 0);
}

#[test]
fn relation_path_length_and_identifiers_are_validated_before_reads() {
    let mut binder = RelationalBinder::new(config(32)).unwrap();
    let before = binder.counters();
    assert_eq!(
        binder.compose_path(&[], 1),
        Err(RelationalError::EmptyRelationPath)
    );
    let too_long = vec![1; MAX_COMPOSITION_HOPS + 1];
    assert_eq!(
        binder.compose_path(&too_long, 1),
        Err(RelationalError::TooManyCompositionHops)
    );
    assert_eq!(
        binder.compose_path(&[1, 256], 1),
        Err(RelationalError::RelationOutOfRange)
    );
    assert_eq!(binder.counters(), before);
}

#[test]
fn insufficient_event_budget_rejects_the_whole_path_before_reads() {
    let mut cfg = config(32);
    cfg.stream.max_events = 3;
    let mut binder = RelationalBinder::new(cfg).unwrap();
    binder.bind(1, 3, 5).unwrap();
    binder.bind(2, 5, 9).unwrap();
    let before = binder.counters();
    assert_eq!(
        binder.compose2(1, 2, 3),
        Err(RelationalError::EventBudgetExhausted)
    );
    assert_eq!(binder.counters(), before);
}

#[test]
fn width_and_identifier_bounds_fail_closed() {
    let mut invalid = config(16);
    invalid.entity_bits = 0;
    assert_eq!(
        RelationalBinder::new(invalid),
        Err(RelationalError::InvalidEntityWidth)
    );

    let mut binder = RelationalBinder::new(config(16)).unwrap();
    assert_eq!(
        binder.bind(256, 1, 2),
        Err(RelationalError::RelationOutOfRange)
    );
    assert_eq!(
        binder.bind(1, 256, 2),
        Err(RelationalError::EntityOutOfRange)
    );
    assert_eq!(
        binder.bind(1, 1, 256),
        Err(RelationalError::EntityOutOfRange)
    );
}
