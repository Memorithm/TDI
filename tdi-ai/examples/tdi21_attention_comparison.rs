//! Six mechanisms on fixed public software fixtures, not a matched benchmark.

use tdi_ai::experimental::tdi21::BooleanState;
use tdi_ai::experimental::tdi21_attention::{
    ATTENTION_SEMANTICS, AttentionConfig, AttentionMode, AttentionReference,
};
use tdi_ai::experimental::tdi21_evaluation::{
    ControlArm, DevelopmentEpisode, evaluate_boolean, evaluate_control, fit_slots,
};
use tdi_ai::experimental::tdi21_stream::{
    Event, MemoryMode, StepOutput, StreamConfig, StreamError,
};

fn write(key: u64, payload: u64) -> Event {
    Event::Write {
        key,
        payload: BooleanState::from_bits(payload),
        marker: BooleanState::from_bits(1),
    }
}

fn main() {
    let sha = std::env::var("TDI21_SOURCE_SHA").expect("use the clean-checkout development script");
    assert!(sha.len() == 40 && sha.bytes().all(|byte| byte.is_ascii_hexdigit()));
    let mut delay = vec![write(1, 0)];
    delay.extend(std::iter::repeat_n(Event::Ignore, 4096));
    delay.push(Event::Recall { key: 1 });
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
    let reverse = vec![
        write(1, 1),
        write(3, 3),
        write(5, 5),
        write(9, 9),
        Event::Recall { key: 3 },
    ];
    let mixed = vec![
        Event::Recall { key: 7 },
        write(7, 0),
        Event::Recall { key: 7 },
        write(7, 3),
        Event::Write {
            key: 7,
            payload: BooleanState::from_bits(255),
            marker: BooleanState::from_bits(3),
        },
        Event::Recall { key: 7 },
        write(0, 5),
        Event::Conjunction { left: 7, right: 0 },
        Event::Recall { key: 9 },
        Event::Conjunction { left: 7, right: 9 },
        Event::Recall { key: 0 },
    ];
    println!("{ATTENTION_SEMANTICS};source_sha={sha};scope=development-only;training=none");
    println!(
        "NOT matched total budgets: Boolean substrate ceiling=518 bits; attention history capacity=16 writes"
    );
    for (id, events, expected_other) in [
        ("delay", delay, [1, 1, 0, 1]),
        ("pair", pair, [1, 2, 0, 2]),
        ("saturated", saturated, [1, 2, 0, 3]),
        ("mixed", mixed, [7, 7, 3, 7]),
        ("reverse", reverse, [1, 0, 0, 1]),
    ] {
        let episode = DevelopmentEpisode::new(&events, 8).unwrap();
        for mode in [AttentionMode::DenseQk, AttentionMode::BinaryQk] {
            let config = AttentionConfig {
                mode,
                history_capacity: 16,
                payload_bits: 8,
                max_events: 8192,
            };
            let mut reference = AttentionReference::new(config).unwrap();
            let mut outputs: Vec<Result<StepOutput, StreamError>> = Vec::new();
            for &event in episode.events() {
                // Rejection fails the executable. It cannot remove a difficult
                // query from the scorer or become a fabricated successful miss.
                let output = reference.step(event).expect("reference execution rejected");
                outputs.push(Ok(output));
            }
            let score = episode.score_outputs(&outputs).unwrap();
            assert!(score.all_correct(), "fixture={id};mode={mode:?}");
            println!(
                "fixture={id};arm={:?};accuracy={:?};score={score:?};config={config:?};work={:?};footprint={:?}",
                mode.arm(),
                score.accuracy_ratio(),
                reference.work(),
                reference.footprint(),
            );
        }
        for (index, mode) in [MemoryMode::Direct, MemoryMode::TwoWay]
            .into_iter()
            .enumerate()
        {
            let config = StreamConfig {
                mode,
                slots: fit_slots(mode, 518).unwrap(),
                payload_bits: 8,
                max_events: 8192,
                route_salt: 0,
            };
            let report = evaluate_boolean(&episode, config, 518).unwrap();
            assert_eq!(report.score.correct_queries(), expected_other[index]);
            println!("fixture={id};boolean={report:?}");
        }
        for (index, arm) in [ControlArm::NoMemory, ControlArm::ExactDictionary]
            .into_iter()
            .enumerate()
        {
            let report = evaluate_control(&episode, arm).unwrap();
            assert_eq!(report.score.correct_queries(), expected_other[index + 2]);
            println!("fixture={id};control={report:?}");
        }
    }
}
