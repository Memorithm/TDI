#![cfg(feature = "experimental")]

use std::collections::BTreeMap;
use tdi_ai::experimental::tdi21::{
    BooleanState, DirectAddressMemory, MemoryRead, ResourceCounters, route_tag,
};
use tdi_ai::experimental::tdi21_stream::{
    BooleanStream, Event, MemoryMode, StepOutput, StreamConfig, StreamError,
};

fn config(mode: MemoryMode) -> StreamConfig {
    StreamConfig {
        mode,
        slots: 4,
        payload_bits: 8,
        max_events: 10_000,
        route_salt: 0,
    }
}

fn write(key: u64, payload: u64) -> Event {
    Event::Write {
        key,
        payload: BooleanState::from_bits(payload),
        marker: BooleanState::from_bits(1),
    }
}

fn reply(payload: u64) -> StepOutput {
    StepOutput::Reply(MemoryRead::Hit(BooleanState::from_bits(payload)))
}

// Evaluator-owned oracle, deliberately absent from the candidate module.
// It uses original identifiers and independent Boolean expressions, not the
// candidate's route transform, memory layout, clause evaluator or output.
fn oracle(event: Event, facts: &mut BTreeMap<u64, u64>) -> StepOutput {
    match event {
        Event::Write {
            key,
            payload,
            marker,
        } => {
            if marker.bits() == 1 {
                facts.insert(key, payload.bits());
            }
            StepOutput::Quiet
        }
        Event::Recall { key } => match facts.get(&key) {
            Some(&payload) => reply(payload),
            None => StepOutput::Reply(MemoryRead::Miss),
        },
        Event::Conjunction { left, right } => match (facts.get(&left), facts.get(&right)) {
            (Some(a), Some(b)) => reply(a & b),
            _ => StepOutput::Reply(MemoryRead::Miss),
        },
        Event::Ignore => StepOutput::Quiet,
    }
}

#[test]
fn prefix_causality_absence_false_and_override_are_distinct() {
    for mode in [MemoryMode::Direct, MemoryMode::TwoWay] {
        let mut machine = BooleanStream::new(config(mode)).unwrap();
        let mut truth = BTreeMap::new();
        let events = [
            Event::Recall { key: 1 },
            write(1, 0),
            Event::Recall { key: 1 },
            write(1, 3),
            Event::Recall { key: 1 },
            Event::Conjunction { left: 1, right: 0 },
            write(0, 5),
            Event::Conjunction { left: 1, right: 0 },
        ];
        for event in events {
            let expected = oracle(event, &mut truth);
            assert_eq!(machine.step(event), Ok(expected));
        }
        assert_eq!(machine.counters().work.pairwise_comparisons, 0);
    }
}

#[test]
fn genuine_delay_is_processed_without_repeated_memory_lookup() {
    for mode in [MemoryMode::Direct, MemoryMode::TwoWay] {
        let mut machine = BooleanStream::new(config(mode)).unwrap();
        let footprint = machine.footprint();
        assert_eq!(machine.step(write(1, 0)), Ok(StepOutput::Quiet));
        for _ in 0..4096 {
            assert_eq!(machine.step(Event::Ignore), Ok(StepOutput::Quiet));
        }
        assert_eq!(machine.step(Event::Recall { key: 1 }), Ok(reply(0)));
        assert_eq!(machine.counters().events, 4098);
        assert_eq!(machine.counters().work.memory_reads, 1);
        assert_eq!(machine.counters().work.memory_writes, 1);
        assert_eq!(machine.counters().work.address_derivations, 2);
        assert_eq!(machine.footprint(), footprint);
    }
}

#[test]
fn two_way_memory_handles_one_collision_but_not_unlimited_recall() {
    let mut direct = BooleanStream::new(config(MemoryMode::Direct)).unwrap();
    let mut two_way = BooleanStream::new(config(MemoryMode::TwoWay)).unwrap();
    for event in [write(1, 1), write(5, 2)] {
        direct.step(event).unwrap();
        two_way.step(event).unwrap();
    }
    assert_eq!(
        direct.step(Event::Recall { key: 1 }),
        Ok(StepOutput::Reply(MemoryRead::Miss))
    );
    assert_eq!(two_way.step(Event::Recall { key: 1 }), Ok(reply(1)));
    assert_eq!(two_way.step(Event::Recall { key: 5 }), Ok(reply(2)));
    two_way.step(write(5, 7)).unwrap();
    // Updating an existing identifier must not move the replacement cursor.
    two_way.step(write(9, 3)).unwrap();
    assert_eq!(
        two_way.step(Event::Recall { key: 1 }),
        Ok(StepOutput::Reply(MemoryRead::Miss))
    );
    assert_eq!(two_way.step(Event::Recall { key: 5 }), Ok(reply(7)));
    assert_eq!(two_way.counters().work.memory_replacements, 1);
    assert_eq!(direct.counters().work.memory_replacements, 1);
    // Equal entry count is NOT equal semantic-memory budget.
    assert_eq!(direct.footprint().replacement_semantic_bits, 0);
    assert_eq!(two_way.footprint().replacement_semantic_bits, 2);
}

#[test]
fn rejected_markers_never_overwrite_a_stored_fact() {
    let mut machine = BooleanStream::new(config(MemoryMode::TwoWay)).unwrap();
    machine.step(write(1, 3)).unwrap();
    for marker in [0, 2, 3] {
        machine
            .step(Event::Write {
                key: 1,
                payload: BooleanState::from_bits(255),
                marker: BooleanState::from_bits(marker),
            })
            .unwrap();
    }
    assert_eq!(machine.step(Event::Recall { key: 1 }), Ok(reply(3)));
    assert_eq!(machine.counters().rejected_writes, 3);
    assert_eq!(machine.counters().work.memory_writes, 1);
}

#[test]
fn errors_are_atomic_and_reset_starts_an_independent_episode() {
    let mut cfg = config(MemoryMode::TwoWay);
    cfg.max_events = 2;
    let mut machine = BooleanStream::new(cfg).unwrap();
    machine.step(write(1, 7)).unwrap();
    let snapshot = machine.clone();
    assert_eq!(
        machine.step(write(1, 256)),
        Err(StreamError::PayloadOutOfRange)
    );
    assert_eq!(machine, snapshot);
    let malformed = Event::Write {
        key: 1,
        payload: BooleanState::from_bits(0),
        marker: BooleanState::from_bits(4),
    };
    assert_eq!(machine.step(malformed), Err(StreamError::InvalidMarker));
    assert_eq!(machine, snapshot);
    assert_eq!(machine.step(Event::Recall { key: 1 }), Ok(reply(7)));
    let snapshot = machine.clone();
    assert_eq!(
        machine.step(Event::Ignore),
        Err(StreamError::EventBudgetExhausted)
    );
    assert_eq!(machine, snapshot);
    let footprint = machine.footprint();
    machine.reset();
    assert_eq!(machine.counters(), Default::default());
    assert_eq!(machine.footprint(), footprint);
    assert_eq!(
        machine.step(Event::Recall { key: 1 }),
        Ok(StepOutput::Reply(MemoryRead::Miss))
    );
}

#[test]
fn constructors_reject_invalid_capacity_width_and_budget() {
    for case in 0..7 {
        let mut cfg = config(MemoryMode::TwoWay);
        match case {
            0 => cfg.slots = 0,
            1 => cfg.slots = 4097,
            2 => cfg.slots = 3,
            3 => cfg.payload_bits = 0,
            4 => cfg.payload_bits = 65,
            5 => cfg.max_events = 0,
            _ => cfg.max_events = 1_000_001,
        }
        assert!(BooleanStream::new(cfg).is_err());
    }
}

#[test]
fn every_length_four_stream_matches_an_independent_oracle_without_eviction() {
    let alphabet = [
        write(0, 0),
        write(0, 1),
        write(1, 2),
        Event::Recall { key: 0 },
        Event::Recall { key: 1 },
        Event::Conjunction { left: 0, right: 1 },
        Event::Ignore,
    ];
    for mode in [MemoryMode::Direct, MemoryMode::TwoWay] {
        for mut code in 0..7_usize.pow(4) {
            let mut machine = BooleanStream::new(config(mode)).unwrap();
            let mut truth = BTreeMap::new();
            for _ in 0..4 {
                let event = alphabet[code % 7];
                code /= 7;
                let before = machine.counters().slot_probes;
                assert_eq!(machine.step(event), Ok(oracle(event, &mut truth)));
                let probes = machine.counters().slot_probes - before;
                assert!(probes <= 4);
            }
            assert_eq!(machine.counters().work.memory_replacements, 0);
        }
    }
}

#[test]
fn direct_stream_matches_original_storage_on_overflowing_identifier_set() {
    let mut cfg = config(MemoryMode::Direct);
    cfg.payload_bits = 64;
    cfg.route_salt = 7;
    let mut machine = BooleanStream::new(cfg).unwrap();
    let mut memory = DirectAddressMemory::<4>::default();
    let mut work = ResourceCounters::default();
    for key in [0, u64::MAX, 1, 5, 9, 1_u64 << 32] {
        let payload = BooleanState::from_bits(key);
        let tag = route_tag(BooleanState::from_bits(key), cfg.route_salt);
        memory.write(tag, payload, &mut work);
        machine.step(write(key, key)).unwrap();
        for query in [0, 1, 5, 9, u64::MAX] {
            let tag = route_tag(BooleanState::from_bits(query), cfg.route_salt);
            let expected = StepOutput::Reply(memory.read(tag, &mut work));
            assert_eq!(machine.step(Event::Recall { key: query }), Ok(expected));
        }
    }
}
