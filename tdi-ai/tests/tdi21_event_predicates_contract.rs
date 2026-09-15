#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi21::BooleanState;
use tdi_ai::experimental::tdi21_event_predicates::{
    WritePredicate, WritePredicateError, encode_b4_write_predicates,
};
use tdi_ai::experimental::tdi21_stream::{
    BooleanStream, Event, MemoryMode, RouteObservation, StreamConfig,
};

fn config(mode: MemoryMode) -> StreamConfig {
    StreamConfig {
        mode,
        slots: 4,
        payload_bits: 8,
        max_events: 32,
        route_salt: 0,
    }
}

fn write(key: u64, payload: u64, marker: u64) -> Event {
    Event::Write {
        key,
        payload: BooleanState::from_bits(payload),
        marker: BooleanState::from_bits(marker),
    }
}

#[test]
fn local_observation_is_bounded_and_does_not_count_as_payload_read() {
    let mut stream = BooleanStream::new(config(MemoryMode::TwoWay)).unwrap();
    let before = stream.counters();
    let observation = stream.observe_key(1);
    let after = stream.counters();

    assert_eq!(
        observation,
        RouteObservation {
            exact_present: false,
            bucket_full: false,
            occupied_ways: 0,
            ways: 2,
            next_victim_way: 0,
        }
    );
    assert_eq!(after.events, before.events);
    assert_eq!(after.route_observations, before.route_observations + 1);
    assert_eq!(after.slot_probes, before.slot_probes + 2);
    assert_eq!(after.work.memory_reads, before.work.memory_reads);
    assert_eq!(after.work.memory_writes, before.work.memory_writes);
    assert_eq!(
        after.work.address_derivations,
        before.work.address_derivations + 1
    );
    assert_eq!(after.work.pairwise_comparisons, 0);
}

#[test]
fn observations_expose_only_local_presence_pressure_and_replacement_state() {
    let mut stream = BooleanStream::new(config(MemoryMode::TwoWay)).unwrap();

    stream.step(write(1, 17, 1)).unwrap();
    let same = stream.observe_key(1);
    assert!(same.exact_present);
    assert_eq!(same.occupied_ways, 1);
    assert!(!same.bucket_full);
    assert_eq!(same.next_victim_way, 0);

    let colliding = stream.observe_key(5);
    assert!(!colliding.exact_present);
    assert_eq!(colliding.occupied_ways, 1);
    assert!(!colliding.bucket_full);

    stream.step(write(5, 23, 1)).unwrap();
    let full = stream.observe_key(9);
    assert!(!full.exact_present);
    assert_eq!(full.occupied_ways, 2);
    assert!(full.bucket_full);
    assert_eq!(full.next_victim_way, 0);

    stream.step(write(9, 31, 1)).unwrap();
    let after_eviction = stream.observe_key(1);
    assert!(!after_eviction.exact_present);
    assert!(after_eviction.bucket_full);
    assert_eq!(after_eviction.occupied_ways, 2);
    assert_eq!(after_eviction.next_victim_way, 1);
    assert_eq!(stream.counters().work.pairwise_comparisons, 0);
}

#[test]
fn repeated_observation_does_not_move_replacement_cursor_or_change_footprint() {
    let mut stream = BooleanStream::new(config(MemoryMode::TwoWay)).unwrap();
    stream.step(write(1, 1, 1)).unwrap();
    stream.step(write(5, 5, 1)).unwrap();
    let footprint = stream.footprint();
    let first = stream.observe_key(9);
    let second = stream.observe_key(9);
    assert_eq!(first, second);
    assert_eq!(first.next_victim_way, 0);
    assert_eq!(stream.footprint(), footprint);
}

#[test]
fn predicate_vector_tracks_declared_marker_and_local_bucket_bits() {
    let event = write(9, 99, 1);

    let empty = encode_b4_write_predicates(
        event,
        RouteObservation {
            exact_present: false,
            bucket_full: false,
            occupied_ways: 0,
            ways: 2,
            next_victim_way: 0,
        },
    )
    .unwrap();
    assert_eq!(empty.assignment(), 0b000001);

    let exact = encode_b4_write_predicates(
        event,
        RouteObservation {
            exact_present: true,
            bucket_full: false,
            occupied_ways: 1,
            ways: 2,
            next_victim_way: 0,
        },
    )
    .unwrap();
    assert_eq!(exact.assignment(), 0b001101);
    assert!(exact.test(WritePredicate::ExactTagPresent));
    assert!(exact.test(WritePredicate::BucketHasAny));
    assert!(!exact.test(WritePredicate::BucketFull));

    let full_next_second = encode_b4_write_predicates(
        event,
        RouteObservation {
            exact_present: false,
            bucket_full: true,
            occupied_ways: 2,
            ways: 2,
            next_victim_way: 1,
        },
    )
    .unwrap();
    assert_eq!(full_next_second.assignment(), 0b111001);
    assert!(full_next_second.test(WritePredicate::NextVictimSecond));
}

#[test]
fn inhibited_marker_bits_are_preserved_without_payload_or_key_features() {
    let observation = RouteObservation {
        exact_present: false,
        bucket_full: false,
        occupied_ways: 0,
        ways: 2,
        next_victim_way: 0,
    };
    let marker_three = encode_b4_write_predicates(write(1, 255, 3), observation).unwrap();
    let different_key_payload = encode_b4_write_predicates(write(99, 0, 3), observation).unwrap();
    assert_eq!(marker_three.assignment(), 0b000011);
    assert_eq!(marker_three, different_key_payload);
}

#[test]
fn adapter_fails_closed_on_nonwrite_invalid_marker_and_non_b3_observation() {
    let two_way = RouteObservation {
        exact_present: false,
        bucket_full: false,
        occupied_ways: 0,
        ways: 2,
        next_victim_way: 0,
    };
    assert_eq!(
        encode_b4_write_predicates(Event::Recall { key: 1 }, two_way),
        Err(WritePredicateError::NotWriteEvent)
    );
    assert_eq!(
        encode_b4_write_predicates(write(1, 0, 4), two_way),
        Err(WritePredicateError::InvalidMarker)
    );

    let mut direct = BooleanStream::new(config(MemoryMode::Direct)).unwrap();
    let direct_observation = direct.observe_key(1);
    assert_eq!(
        encode_b4_write_predicates(write(1, 0, 1), direct_observation),
        Err(WritePredicateError::RequiresTwoWayObservation)
    );
}

#[test]
fn impossible_observation_shapes_are_rejected() {
    let event = write(1, 0, 1);
    for observation in [
        RouteObservation {
            exact_present: false,
            bucket_full: false,
            occupied_ways: 3,
            ways: 2,
            next_victim_way: 0,
        },
        RouteObservation {
            exact_present: false,
            bucket_full: false,
            occupied_ways: 2,
            ways: 2,
            next_victim_way: 0,
        },
        RouteObservation {
            exact_present: false,
            bucket_full: true,
            occupied_ways: 1,
            ways: 2,
            next_victim_way: 0,
        },
        RouteObservation {
            exact_present: false,
            bucket_full: false,
            occupied_ways: 0,
            ways: 2,
            next_victim_way: 2,
        },
        RouteObservation {
            exact_present: true,
            bucket_full: false,
            occupied_ways: 0,
            ways: 2,
            next_victim_way: 0,
        },
    ] {
        assert_eq!(
            encode_b4_write_predicates(event, observation),
            Err(WritePredicateError::InvalidObservation)
        );
    }
}
