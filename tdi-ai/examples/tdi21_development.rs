//! Public development fixtures with independent scoring and competence controls.
//! No attention, training or confirmatory comparison is performed here.

use tdi_ai::experimental::tdi21::{BooleanState, EvidenceRecord, RunManifest};
use tdi_ai::experimental::tdi21_evaluation::{
    ControlArm, DevelopmentEpisode, EVALUATION_SEMANTICS, Outcome, ScoreCard,
    evaluate_boolean, evaluate_control, fit_slots,
};
use tdi_ai::experimental::tdi21_provenance::canonical_evidence_record;
use tdi_ai::experimental::tdi21_stream::{Event, MemoryMode, STREAM_SEMANTICS, StreamConfig};

fn write(key: u64, payload: u64) -> Event {
    Event::Write {
        key,
        payload: BooleanState::from_bits(payload),
        marker: BooleanState::from_bits(1),
    }
}

fn print_score(score: &ScoreCard) {
    let (correct, queries) = score.accuracy_ratio().expect("fixture requires queries");
    println!(
        "expected_queries={queries};correct_queries={correct};all_correct={};exact_values={};correct_absences={};wrong_values={};false_hits={};forgotten_values={};omitted_replies={};query_errors={};spurious_replies={};quiet_errors={}",
        score.all_correct(),
        score.count(Outcome::ExactValue),
        score.count(Outcome::CorrectAbsence),
        score.count(Outcome::WrongValue),
        score.count(Outcome::FalseHit),
        score.count(Outcome::ForgottenValue),
        score.count(Outcome::OmittedReply),
        score.count(Outcome::QueryError),
        score.count(Outcome::SpuriousReply),
        score.count(Outcome::QuietError),
    );
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
    let mixed = vec![
        Event::Recall { key: 7 },
        write(7, 0),
        Event::Recall { key: 7 },
        Event::Write {
            key: 7,
            payload: BooleanState::from_bits(255),
            marker: BooleanState::from_bits(3),
        },
        Event::Recall { key: 7 },
        write(7, 5),
        Event::Recall { key: 7 },
        Event::Conjunction { left: 7, right: 8 },
        write(8, 3),
        Event::Conjunction { left: 7, right: 8 },
        Event::Recall { key: u64::MAX },
    ];
    let reverse = vec![
        write(1, 1),
        write(3, 3),
        write(5, 5),
        write(9, 9),
        Event::Recall { key: 3 },
    ];
    // Published regression expectations, not tuned scientific acceptance rules.
    // Fields: id, inputs, B2 exact queries, B3 exact queries, all queries, absent queries.
    let fixtures = [
        ("delay", delayed, 1, 1, 1, 0),
        ("pair", pair, 1, 2, 2, 0),
        ("saturated", saturated, 1, 2, 3, 0),
        ("mixed", mixed, 7, 7, 7, 3),
        ("reverse", reverse, 1, 0, 1, 0),
    ];
    const CEILING: usize = 518;
    println!(
        "semantics={STREAM_SEMANTICS};evaluation={EVALUATION_SEMANTICS};scope=development-only;training=none;memory_scope=substrate-only;ceiling_bits={CEILING}"
    );
    for (id, events, b2, b3, queries, absences) in fixtures {
        let episode = DevelopmentEpisode::new(&events, 8).unwrap();
        for mode in [MemoryMode::Direct, MemoryMode::TwoWay] {
            let cfg = StreamConfig {
                mode,
                slots: fit_slots(mode, CEILING).unwrap(),
                payload_bits: 8,
                max_events: 10_000,
                route_salt: 0,
            };
            let report = evaluate_boolean(&episode, cfg, CEILING).unwrap();
            let evidence = EvidenceRecord {
                manifest: RunManifest {
                    arm: mode.arm(),
                    sequence_len: events.len(),
                    state_width_bits: cfg.payload_bits,
                    memory_slots: cfg.slots,
                    development_seed: 0,
                },
                correct: report.score.all_correct(),
                resources: report.counters.work,
            };
            let record = canonical_evidence_record(id, &sha, &evidence).unwrap();
            println!("{record}");
            println!(
                "fixture={id};mode={mode:?};slots={};ceiling_bits={CEILING};max_events={};route_salt={};slot_probes={};oracle_probes={};footprint={:?}",
                cfg.slots,
                cfg.max_events,
                cfg.route_salt,
                report.counters.slot_probes,
                episode.oracle_probes(),
                report.footprint,
            );
            print_score(&report.score);
            let expected_correct = if mode == MemoryMode::Direct { b2 } else { b3 };
            assert_eq!(report.score.accuracy_ratio(), Some((expected_correct, queries)));
            assert_eq!(report.counters.work.pairwise_comparisons, 0);
        }
        for arm in [ControlArm::NoMemory, ControlArm::ExactDictionary] {
            let report = evaluate_control(&episode, arm).unwrap();
            println!(
                "fixture={id};control={arm:?};matched_memory_budget=false;entry_payload_bits_lower_bound={};native_map_work={:?}",
                report.entry_payload_bits_lower_bound,
                report.work,
            );
            print_score(&report.score);
            let expected = if arm == ControlArm::NoMemory { absences } else { queries };
            assert_eq!(report.score.accuracy_ratio(), Some((expected, queries)));
        }
    }
}
