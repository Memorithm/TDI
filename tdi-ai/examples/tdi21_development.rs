//! Reproducible development fixtures, not a comparative scientific benchmark.

use std::collections::BTreeMap;
use tdi_ai::experimental::tdi21::{BooleanState, EvidenceRecord, MemoryRead, RunManifest};
use tdi_ai::experimental::tdi21_provenance::canonical_evidence_record;
use tdi_ai::experimental::tdi21_stream::{
    BooleanStream, Event, MemoryMode, STREAM_SEMANTICS, StepOutput, StreamConfig,
};

fn write(key: u64, payload: u64) -> Event {
    Event::Write {
        key,
        payload: BooleanState::from_bits(payload),
        marker: BooleanState::from_bits(1),
    }
}

fn oracle(event: Event, facts: &mut BTreeMap<u64, u64>) -> StepOutput {
    let value = match event {
        Event::Write {
            key,
            payload,
            marker,
        } => {
            if marker.bits() == 1 {
                facts.insert(key, payload.bits());
            }
            return StepOutput::Quiet;
        }
        Event::Recall { key } => facts.get(&key).copied(),
        Event::Conjunction { left, right } => {
            facts.get(&left).zip(facts.get(&right)).map(|(a, b)| a & b)
        }
        Event::Ignore => return StepOutput::Quiet,
    };
    StepOutput::Reply(match value {
        Some(bits) => MemoryRead::Hit(BooleanState::from_bits(bits)),
        None => MemoryRead::Miss,
    })
}

fn main() {
    let sha = std::env::var("TDI21_SOURCE_SHA").expect("use check-tdi21-development.sh");
    let mut delayed = vec![write(1, 0)];
    delayed.extend(std::iter::repeat_n(Event::Ignore, 4096));
    delayed.push(Event::Recall { key: 1 });
    let pair = vec![
        write(1, 1),
        write(5, 0),
        Event::Recall { key: 1 },
        Event::Recall { key: 5 },
    ];
    let saturated = vec![
        write(1, 1),
        write(5, 0),
        write(9, 1),
        Event::Recall { key: 1 },
        Event::Recall { key: 5 },
        Event::Recall { key: 9 },
    ];
    println!("semantics={STREAM_SEMANTICS};scope=development-only;training=none");
    for (id, events) in [("delay", delayed), ("pair", pair), ("saturated", saturated)] {
        for mode in [MemoryMode::Direct, MemoryMode::TwoWay] {
            let cfg = StreamConfig {
                mode,
                slots: 4,
                payload_bits: 8,
                max_events: 10_000,
                route_salt: 0,
            };
            let mut machine = BooleanStream::new(cfg).unwrap();
            let mut truth = BTreeMap::new();
            let mut correct_queries = 0;
            for &event in &events {
                let expected = oracle(event, &mut truth);
                let observed = machine.step(event).unwrap();
                if matches!(observed, StepOutput::Reply(_)) && expected == observed {
                    correct_queries += 1;
                }
            }
            let stats = machine.counters();
            let evidence = EvidenceRecord {
                manifest: RunManifest {
                    arm: mode.arm(),
                    sequence_len: events.len(),
                    state_width_bits: cfg.payload_bits,
                    memory_slots: cfg.slots,
                    development_seed: 0,
                },
                correct: correct_queries == stats.replies,
                resources: stats.work,
            };
            let record = canonical_evidence_record(id, &sha, &evidence).unwrap();
            println!("{record}");
            println!(
                "fixture={id};mode={mode:?};queries={};correct_queries={correct_queries};slot_probes={};route_salt={};footprint={:?}",
                stats.replies,
                stats.slot_probes,
                cfg.route_salt,
                machine.footprint(),
            );
            // These are software regression expectations for three published
            // fixtures, not acceptance thresholds chosen for scientific claims.
            let expected_correct = match (id, mode) {
                ("delay", _) | ("pair", MemoryMode::Direct) | ("saturated", MemoryMode::Direct) => {
                    1
                }
                _ => 2,
            };
            assert_eq!(correct_queries, expected_correct);
            assert_eq!(stats.work.pairwise_comparisons, 0);
        }
    }
}
