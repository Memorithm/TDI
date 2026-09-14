#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi21::{BooleanState, MemoryRead};
use tdi_ai::experimental::tdi21_attention::{
    AttentionConfig, AttentionError, AttentionMode, AttentionReference, AttentionWork,
};
use tdi_ai::experimental::tdi21_evaluation::{DevelopmentEpisode, EvaluationError};
use tdi_ai::experimental::tdi21_stream::{Event, StepOutput, StreamError};

fn config(mode: AttentionMode, capacity: usize, width: u8) -> AttentionConfig {
    AttentionConfig { mode, history_capacity: capacity, payload_bits: width, max_events: 1024 }
}

fn write(key: u64, bits: u64) -> Event {
    Event::Write {
        key, payload: BooleanState::from_bits(bits), marker: BooleanState::from_bits(1),
    }
}

fn hit(bits: u64) -> StepOutput {
    StepOutput::Reply(MemoryRead::Hit(BooleanState::from_bits(bits)))
}

#[test]
fn reference_arms_cannot_be_mislabelled_as_boolean_candidates() {
    for mode in [AttentionMode::DenseQk, AttentionMode::BinaryQk] {
        assert!(!mode.arm().is_boolean_candidate());
    }
    // Limited direct-import regression, NOT an attestation of arbitrary code.
    assert!(!include_str!("../src/tdi21_stream.rs").contains("tdi21_attention"));
    assert!(!include_str!("../src/tdi21_boolean_relational.rs").contains("tdi21_attention"));
}

#[test]
fn every_query_containing_length_four_stream_matches_the_prefix_oracle() {
    let alphabet = [
        write(0, 0), write(0, 1), write(1, 2),
        Event::Recall { key: 0 }, Event::Recall { key: 1 },
        Event::Conjunction { left: 0, right: 1 }, Event::Ignore,
    ];
    let mut admitted = 0;
    for mut code in 0..7_usize.pow(4) {
        let events: Vec<_> = (0..4).map(|_| {
            let event = alphabet[code % 7];
            code /= 7;
            event
        }).collect();
        let episode = match DevelopmentEpisode::new(&events, 8) {
            Ok(episode) => episode,
            Err(EvaluationError::NoQueries) => continue,
            Err(error) => panic!("unexpected admission error: {error:?}"),
        };
        admitted += 1;
        let mut dense = AttentionReference::new(config(AttentionMode::DenseQk, 4, 8)).unwrap();
        let mut binary = AttentionReference::new(config(AttentionMode::BinaryQk, 4, 8)).unwrap();
        let mut outputs: Vec<Result<StepOutput, StreamError>> = Vec::new();
        for &event in episode.events() {
            let a = dense.step(event).unwrap();
            let b = binary.step(event).unwrap();
            assert_eq!(a, b);
            assert_eq!(dense.last_weights(), binary.last_weights());
            outputs.push(Ok(a));
        }
        assert!(episode.score_outputs(&outputs).unwrap().all_correct());
        assert_eq!(dense.work().pairwise_scores, binary.work().pairwise_scores);
    }
    // 7^4 strings minus the 4^4 strings containing no query.
    assert_eq!(admitted, 2145);
}

#[test]
fn original_identifier_bits_are_not_lost_through_f64_casts() {
    for mode in [AttentionMode::DenseQk, AttentionMode::BinaryQk] {
        let mut reference = AttentionReference::new(config(mode, 2, 64)).unwrap();
        reference.step(write(0, 0)).unwrap();
        reference.step(write(u64::MAX, 1_u64 << 63)).unwrap();
        assert_eq!(reference.step(Event::Recall { key: 0 }).unwrap(), hit(0));
        assert_eq!(reference.step(Event::Recall { key: u64::MAX }).unwrap(), hit(1_u64 << 63));
        for bit in 0..64 {
            for key in [1_u64 << bit, !(1_u64 << bit)] {
                assert_eq!(reference.step(Event::Recall { key }).unwrap(),
                    StepOutput::Reply(MemoryRead::Miss));
            }
        }
    }
}

#[test]
fn stable_softmax_preserves_latest_of_256_overrides() {
    for mode in [AttentionMode::DenseQk, AttentionMode::BinaryQk] {
        let mut reference = AttentionReference::new(config(mode, 256, 64)).unwrap();
        for i in 0..256 {
            reference.step(write(7, if i % 2 == 0 { 0 } else { u64::MAX })).unwrap();
        }
        assert_eq!(reference.step(Event::Recall { key: 7 }).unwrap(), hit(u64::MAX));
        let weights = reference.last_weights();
        assert_eq!(weights.len(), 257);
        assert!(weights.iter().all(|w| w.is_finite() && *w >= 0.0));
        assert!((weights.iter().sum::<f64>() - 1.0).abs() < 1e-12);
        assert!(weights[256] > 0.9999);
        // No exact key: the explicit null lane, not an equality shortcut, wins.
        assert_eq!(reference.step(Event::Recall { key: 6 }).unwrap(),
            StepOutput::Reply(MemoryRead::Miss));
        assert!(reference.last_weights()[0] > 0.9999);
    }
}

#[test]
fn real_dot_and_binary_operations_are_counted_separately() {
    for mode in [AttentionMode::DenseQk, AttentionMode::BinaryQk] {
        let mut reference = AttentionReference::new(config(mode, 4, 8)).unwrap();
        reference.step(write(0, 3)).unwrap();
        reference.step(write(1, 5)).unwrap();
        assert_eq!(reference.step(Event::Conjunction { left: 0, right: 1 }).unwrap(), hit(1));
        let work = reference.work();
        assert_eq!(work.events, 3);
        assert_eq!(work.lookups, 2);
        assert_eq!(work.pairwise_scores, 4);
        assert_eq!(work.exponentials, 6);
        assert_eq!(work.normalizations, 6);
        assert_eq!(work.weighted_value_terms, 32);
        assert_eq!(work.value_component_encodes, 128);
        assert_eq!(work.presence_terms, 4);
        assert_eq!(work.readout_comparisons, 18);
        assert_eq!(work.conjunctions, 1);
        match mode {
            AttentionMode::DenseQk => {
                assert_eq!(work.float_dot_terms, 256);
                assert_eq!(work.xor_popcount_words, 0);
                assert_eq!(work.key_component_encodes, 256);
            }
            AttentionMode::BinaryQk => {
                assert_eq!(work.float_dot_terms, 0);
                assert_eq!(work.xor_popcount_words, 4);
                assert_eq!(work.key_component_encodes, 0);
            }
        }
    }
}

#[test]
fn capacity_rejection_is_atomic_and_does_not_become_silent_eviction() {
    for mode in [AttentionMode::DenseQk, AttentionMode::BinaryQk] {
        let mut reference = AttentionReference::new(config(mode, 1, 8)).unwrap();
        reference.step(write(7, 0)).unwrap();
        let before = reference.clone();
        assert_eq!(reference.step(write(7, 1)), Err(AttentionError::HistoryCapacityExhausted));
        assert_eq!(reference, before);
        assert_eq!(reference.step(write(8, 1)), Err(AttentionError::HistoryCapacityExhausted));
        assert_eq!(reference, before);
        assert_eq!(reference.step(write(7, 256)), Err(AttentionError::PayloadOutOfRange));
        assert_eq!(reference, before);
        assert_eq!(reference.step(Event::Write {
            key: 7, payload: BooleanState::from_bits(0), marker: BooleanState::from_bits(4),
        }), Err(AttentionError::InvalidMarker));
        assert_eq!(reference, before);
        reference.step(Event::Write {
            key: 7, payload: BooleanState::from_bits(1), marker: BooleanState::from_bits(3),
        }).unwrap();
        assert_eq!(reference.work().inhibited_writes, 1);
        assert_eq!(reference.step(Event::Recall { key: 7 }).unwrap(), hit(0));
    }
}

#[test]
fn empty_history_null_lane_delay_reset_and_event_budget_are_explicit() {
    for mode in [AttentionMode::DenseQk, AttentionMode::BinaryQk] {
        let mut cfg = config(mode, 4, 8);
        cfg.max_events = 4100;
        let mut reference = AttentionReference::new(cfg).unwrap();
        assert!(reference.last_weights().is_empty());
        assert_eq!(reference.step(Event::Recall { key: 7 }).unwrap(),
            StepOutput::Reply(MemoryRead::Miss));
        assert_eq!(reference.last_weights(), &[1.0]);
        assert_eq!(reference.work().pairwise_scores, 0);
        reference.step(write(7, 0)).unwrap();
        for _ in 0..4096 { reference.step(Event::Ignore).unwrap(); }
        assert_eq!(reference.step(Event::Recall { key: 7 }).unwrap(), hit(0));
        assert_eq!(reference.work().pairwise_scores, 1);
        reference.step(Event::Ignore).unwrap();
        let before = reference.clone();
        assert_eq!(reference.step(Event::Ignore), Err(AttentionError::EventBudgetExhausted));
        assert_eq!(reference, before);
        let footprint = reference.footprint();
        reference.reset();
        assert_eq!(reference.work(), AttentionWork::default());
        assert_eq!(reference.footprint(), footprint);
        assert_eq!(reference.config(), cfg);
        assert_eq!(reference.stored_events(), 0);
        assert!(reference.last_weights().is_empty());
    }
}

#[test]
fn constructor_bounds_and_component_memory_are_not_fabricated() {
    let dense = AttentionReference::new(config(AttentionMode::DenseQk, 4, 8)).unwrap();
    let binary = AttentionReference::new(config(AttentionMode::BinaryQk, 4, 8)).unwrap();
    assert_eq!(dense.footprint().history_component_bits, 4 * 8192);
    assert_eq!(binary.footprint().history_component_bits, 4 * 4160);
    assert_eq!(dense.footprint().score_component_bits, 5 * 64);
    assert_eq!(binary.footprint().score_component_bits, 5 * 64);
    for case in 0..6 {
        let mut cfg = config(AttentionMode::DenseQk, 4, 8);
        match case {
            0 => cfg.history_capacity = 0,
            1 => cfg.history_capacity = 257,
            2 => cfg.payload_bits = 0,
            3 => cfg.payload_bits = 65,
            4 => cfg.max_events = 0,
            _ => cfg.max_events = 16_385,
        }
        assert!(AttentionReference::new(cfg).is_err());
    }
}
