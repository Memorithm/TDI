#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi21::{BooleanState, MemoryRead};
use tdi_ai::experimental::tdi21_evaluation::{
    ControlArm, DevelopmentEpisode, EvaluationError, Outcome, ScoreCard, evaluate_boolean,
    evaluate_control, fit_slots, substrate_bits,
};
use tdi_ai::experimental::tdi21_stream::{
    Event, MemoryMode, StepOutput, StreamConfig, StreamError,
};

fn hit(bits: u64) -> StepOutput {
    StepOutput::Reply(MemoryRead::Hit(BooleanState::from_bits(bits)))
}

fn miss() -> StepOutput {
    StepOutput::Reply(MemoryRead::Miss)
}

fn write(key: u64, bits: u64, marker: u64) -> Event {
    Event::Write {
        key,
        payload: BooleanState::from_bits(bits),
        marker: BooleanState::from_bits(marker),
    }
}

fn config(mode: MemoryMode, ceiling: usize) -> StreamConfig {
    StreamConfig {
        mode,
        slots: fit_slots(mode, ceiling).unwrap(),
        payload_bits: 8,
        max_events: 10_000,
        route_salt: 0,
    }
}

#[test]
fn each_outcome_has_its_own_category_and_fixed_query_denominator() {
    let quiet = StepOutput::Quiet;
    let error = Err(StreamError::EventBudgetExhausted);
    let cases = [
        (hit(0), Ok(hit(0)), Outcome::ExactValue),
        (miss(), Ok(miss()), Outcome::CorrectAbsence),
        (hit(0), Ok(hit(1)), Outcome::WrongValue),
        (miss(), Ok(hit(0)), Outcome::FalseHit),
        (hit(0), Ok(miss()), Outcome::ForgottenValue),
        (miss(), Ok(quiet), Outcome::OmittedReply),
        (hit(0), error, Outcome::QueryError),
        (quiet, Ok(quiet), Outcome::CorrectQuiet),
        (quiet, Ok(miss()), Outcome::SpuriousReply),
        (quiet, error, Outcome::QuietError),
    ];
    let mut score = ScoreCard::default();
    for (expected, observed, category) in cases {
        score.record(expected, observed).unwrap();
        assert_eq!(score.count(category), 1);
    }
    assert_eq!(score.events(), 10);
    assert_eq!(score.accuracy_ratio(), Some((2, 7)));
    assert!(!score.all_correct());
}

#[test]
fn zero_queries_silence_and_truncated_outputs_never_pass() {
    let mut score = ScoreCard::default();
    assert_eq!(score.accuracy_ratio(), None);
    score
        .record(StepOutput::Quiet, Ok(StepOutput::Quiet))
        .unwrap();
    assert!(!score.all_correct());
    let events = [write(7, 0, 1), Event::Recall { key: 7 }];
    let episode = DevelopmentEpisode::new(&events, 8).unwrap();
    let outputs = [Ok(StepOutput::Quiet); 2];
    let silent = episode.score_outputs(&outputs).unwrap();
    assert_eq!(silent.accuracy_ratio(), Some((0, 1)));
    assert_eq!(silent.count(Outcome::OmittedReply), 1);
    assert!(!silent.all_correct());
    assert_eq!(
        episode.score_outputs(&outputs[..1]),
        Err(EvaluationError::OutputCountMismatch)
    );
    assert_eq!(
        episode.score_outputs(&[Ok(StepOutput::Quiet); 3]),
        Err(EvaluationError::OutputCountMismatch)
    );
}

#[test]
fn correct_query_does_not_hide_spurious_or_failed_nonquery_events() {
    for observed in [Ok(miss()), Err(StreamError::InvalidMarker)] {
        let mut score = ScoreCard::default();
        score.record(miss(), Ok(miss())).unwrap();
        score.record(StepOutput::Quiet, observed).unwrap();
        assert_eq!(score.accuracy_ratio(), Some((1, 1)));
        assert!(!score.all_correct());
    }
}

#[test]
fn independent_oracle_respects_prefixes_markers_overrides_and_zero() {
    let events = [
        Event::Recall { key: 7 },
        write(7, 0, 1),
        Event::Recall { key: 7 },
        write(7, 255, 3),
        Event::Recall { key: 7 },
        write(7, 5, 1),
        Event::Recall { key: 7 },
        Event::Conjunction { left: 7, right: 8 },
        write(8, 3, 1),
        Event::Conjunction { left: 7, right: 8 },
        Event::Recall { key: u64::MAX },
    ];
    let episode = DevelopmentEpisode::new(&events, 8).unwrap();
    let outputs = [
        miss(),
        StepOutput::Quiet,
        hit(0),
        StepOutput::Quiet,
        hit(0),
        StepOutput::Quiet,
        hit(5),
        miss(),
        StepOutput::Quiet,
        hit(1),
        miss(),
    ]
    .map(Ok);
    let score = episode.score_outputs(&outputs).unwrap();
    assert!(score.all_correct());
    assert_eq!(score.accuracy_ratio(), Some((7, 7)));
    assert_eq!(score.count(Outcome::CorrectAbsence), 3);
    assert_eq!(episode.events(), &events);
    assert!(episode.oracle_probes() > 0);
    let exact = evaluate_control(&episode, ControlArm::ExactDictionary).unwrap();
    let empty = evaluate_control(&episode, ControlArm::NoMemory).unwrap();
    assert_eq!(exact.score, score);
    assert_eq!(exact.work.stored_facts, 2);
    assert_eq!(exact.work.map_writes, 3);
    assert_eq!(empty.score.accuracy_ratio(), Some((3, 7)));
    assert_eq!(empty.score.count(Outcome::ForgottenValue), 4);
    assert_eq!(empty.entry_payload_bits_lower_bound, 0);
}

#[test]
fn common_memory_ceiling_charges_the_extra_replacement_bits() {
    assert_eq!(fit_slots(MemoryMode::Direct, 516), Ok(4));
    assert_eq!(fit_slots(MemoryMode::TwoWay, 516), Ok(2));
    assert_eq!(fit_slots(MemoryMode::TwoWay, 518), Ok(4));
    for ceiling in 0..2048 {
        for mode in [MemoryMode::Direct, MemoryMode::TwoWay] {
            let brute = (1..=16)
                .filter(|&slots| substrate_bits(mode, slots).is_ok_and(|n| n <= ceiling))
                .max();
            assert_eq!(fit_slots(mode, ceiling).ok(), brute);
        }
    }
    for mode in [MemoryMode::Direct, MemoryMode::TwoWay] {
        assert_eq!(fit_slots(mode, usize::MAX), Ok(4096));
        assert_eq!(
            substrate_bits(mode, usize::MAX),
            Err(EvaluationError::InvalidSlotCount)
        );
    }
    assert_eq!(
        substrate_bits(MemoryMode::TwoWay, 3),
        Err(EvaluationError::InvalidSlotCount)
    );
}

#[test]
fn candidate_runs_check_envelopes_before_execution() {
    let episode = DevelopmentEpisode::new(&[write(1, 0, 1), Event::Recall { key: 1 }], 8).unwrap();
    let cfg = config(MemoryMode::TwoWay, 518);
    assert_eq!(
        evaluate_boolean(&episode, cfg, 516),
        Err(EvaluationError::MemoryEnvelopeExceeded)
    );
    let mut bad = cfg;
    bad.payload_bits = 7;
    assert_eq!(
        evaluate_boolean(&episode, bad, 518),
        Err(EvaluationError::PayloadWidthMismatch)
    );
    bad = cfg;
    bad.max_events = 1;
    assert_eq!(
        evaluate_boolean(&episode, bad, 518),
        Err(EvaluationError::EventEnvelopeExceeded)
    );
    let report = evaluate_boolean(&episode, cfg, 518).unwrap();
    assert!(report.score.all_correct());
    assert_eq!(report.score.queries(), 1);
    assert_eq!(report.counters.work.pairwise_comparisons, 0);
    assert_eq!(evaluate_boolean(&episode, cfg, 518).unwrap(), report);
}

#[test]
fn capacity_failure_is_scored_against_truth_not_surviving_memory() {
    let events = [
        write(1, 1, 1),
        write(5, 0, 1),
        write(9, 3, 1),
        Event::Recall { key: 1 },
        Event::Recall { key: 5 },
        Event::Recall { key: 9 },
    ];
    let episode = DevelopmentEpisode::new(&events, 8).unwrap();
    let direct = evaluate_boolean(&episode, config(MemoryMode::Direct, 518), 518).unwrap();
    let two = evaluate_boolean(&episode, config(MemoryMode::TwoWay, 518), 518).unwrap();
    let exact = evaluate_control(&episode, ControlArm::ExactDictionary).unwrap();
    let empty = evaluate_control(&episode, ControlArm::NoMemory).unwrap();
    assert_eq!(direct.score.accuracy_ratio(), Some((1, 3)));
    assert_eq!(two.score.accuracy_ratio(), Some((2, 3)));
    assert_eq!(exact.score.accuracy_ratio(), Some((3, 3)));
    assert_eq!(empty.score.accuracy_ratio(), Some((0, 3)));
    assert_eq!(direct.score.count(Outcome::ForgottenValue), 2);
    assert_eq!(two.score.count(Outcome::ForgottenValue), 1);
    assert!(!two.score.all_correct());
}

#[test]
fn reverse_control_preserves_two_way_nondominance() {
    let events = [
        write(1, 1, 1),
        write(3, 3, 1),
        write(5, 5, 1),
        write(9, 9, 1),
        Event::Recall { key: 3 },
    ];
    let episode = DevelopmentEpisode::new(&events, 8).unwrap();
    let direct = evaluate_boolean(&episode, config(MemoryMode::Direct, 518), 518).unwrap();
    let two = evaluate_boolean(&episode, config(MemoryMode::TwoWay, 518), 518).unwrap();
    assert!(direct.score.all_correct());
    assert_eq!(two.score.accuracy_ratio(), Some((0, 1)));
}

#[test]
fn episodes_fail_closed_on_invalid_inputs_and_excess_oracle_work() {
    assert_eq!(
        DevelopmentEpisode::new(&[], 8),
        Err(EvaluationError::EmptyEpisode)
    );
    assert_eq!(
        DevelopmentEpisode::new(&[Event::Ignore], 8),
        Err(EvaluationError::NoQueries)
    );
    for width in [0, 65] {
        assert_eq!(
            DevelopmentEpisode::new(&[Event::Recall { key: 0 }], width),
            Err(EvaluationError::InvalidPayloadWidth)
        );
    }
    for invalid in [write(1, 256, 0), write(1, 0, 4)] {
        assert_eq!(
            DevelopmentEpisode::new(&[invalid, Event::Recall { key: 1 }], 8),
            Err(EvaluationError::InvalidEvent { index: 0 })
        );
    }
    let long = vec![Event::Recall { key: 0 }; 16_385];
    assert_eq!(
        DevelopmentEpisode::new(&long, 8),
        Err(EvaluationError::TooManyEvents)
    );
    let expensive = vec![Event::Recall { key: 0 }; 2000];
    assert_eq!(
        DevelopmentEpisode::new(&expensive, 8),
        Err(EvaluationError::OracleBudgetExceeded)
    );
    let mut writes = vec![write(0, 0, 1); 4097];
    writes.push(Event::Recall { key: 0 });
    assert_eq!(
        DevelopmentEpisode::new(&writes, 8),
        Err(EvaluationError::TooManyWrites)
    );
}

#[test]
fn exhaustive_three_event_development_controls_keep_the_oracle_independent() {
    let alphabet = [
        write(0, 0, 1),
        write(0, 3, 1),
        write(1, 1, 1),
        write(1, 7, 3),
        Event::Recall { key: 0 },
        Event::Recall { key: 1 },
        Event::Conjunction { left: 0, right: 1 },
        Event::Ignore,
    ];
    let mut query_episodes = 0;
    for mut code in 0..8_usize.pow(3) {
        let mut events = Vec::new();
        for _ in 0..3 {
            events.push(alphabet[code % 8]);
            code /= 8;
        }
        let episode = match DevelopmentEpisode::new(&events, 8) {
            Err(EvaluationError::NoQueries) => continue,
            other => other.unwrap(),
        };
        query_episodes += 1;
        let exact = evaluate_control(&episode, ControlArm::ExactDictionary).unwrap();
        assert!(exact.score.all_correct());
        for mode in [MemoryMode::Direct, MemoryMode::TwoWay] {
            let report = evaluate_boolean(&episode, config(mode, 518), 518).unwrap();
            assert_eq!(report.score, exact.score);
        }
    }
    assert_eq!(query_episodes, 8_usize.pow(3) - 5_usize.pow(3));
}
