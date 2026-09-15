#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi21::BooleanState;
use tdi_ai::experimental::tdi21_anf_synthesis::{
    AlgebraicBooleanStream, AlgebraicStreamError, AnfSynthesisError, MAX_ANF_VARIABLES,
    marker_acceptance_program, synthesize_anf,
};
use tdi_ai::experimental::tdi21_stream::{
    BooleanStream, Event, MemoryMode, StreamConfig, StreamError,
};

fn write(key: u64, payload: u64, marker: u64) -> Event {
    Event::Write {
        key,
        payload: BooleanState::from_bits(payload),
        marker: BooleanState::from_bits(marker),
    }
}

fn b3_config() -> StreamConfig {
    StreamConfig {
        mode: MemoryMode::TwoWay,
        slots: 8,
        payload_bits: 8,
        max_events: 1024,
        route_salt: 0x5444_4932_3142_3401,
    }
}

#[test]
fn mobius_synthesis_recovers_every_three_variable_boolean_function() {
    for function_bits in 0u16..=255 {
        let table: Vec<bool> = (0..8)
            .map(|assignment| (function_bits >> assignment) & 1 != 0)
            .collect();
        let program = synthesize_anf(3, &table).unwrap();
        for (assignment, expected) in table.iter().copied().enumerate() {
            assert_eq!(program.evaluate(assignment as u64), Ok(expected));
        }
        assert!(program.terms().windows(2).all(|pair| {
            pair[0].variables < pair[1].variables
        }));
    }
}

#[test]
fn marker_gate_has_the_expected_canonical_zhegalkin_form() {
    let program = marker_acceptance_program();
    assert!(!program.constant());
    assert_eq!(program.variable_count(), 2);
    let masks: Vec<_> = program.terms().iter().map(|term| term.variables).collect();
    assert_eq!(masks, vec![0b01, 0b11]);
    assert_eq!(program.evaluate(0), Ok(false));
    assert_eq!(program.evaluate(1), Ok(true));
    assert_eq!(program.evaluate(2), Ok(false));
    assert_eq!(program.evaluate(3), Ok(false));
    assert_eq!(
        program.evaluate(4),
        Err(AnfSynthesisError::AssignmentOutOfRange)
    );
    assert_eq!(program.semantic_bits(), 9 + 2 * 64);
}

#[test]
fn synthesis_fails_closed_on_invalid_shapes_and_bounds() {
    assert_eq!(synthesize_anf(0, &[]), Err(AnfSynthesisError::ZeroVariables));
    assert_eq!(
        synthesize_anf(MAX_ANF_VARIABLES + 1, &[]),
        Err(AnfSynthesisError::TooManyVariables)
    );
    assert_eq!(
        synthesize_anf(3, &[false; 7]),
        Err(AnfSynthesisError::TruthTableLengthMismatch {
            expected: 8,
            actual: 7,
        })
    );
}

#[test]
fn b4_requires_the_b3_two_way_substrate() {
    let mut cfg = b3_config();
    cfg.mode = MemoryMode::Direct;
    assert_eq!(
        AlgebraicBooleanStream::new(cfg),
        Err(AlgebraicStreamError::RequiresTwoWayB3)
    );
}

#[test]
fn b4_matches_b3_on_every_length_four_public_stream() {
    let alphabet = [
        write(0, 0, 1),
        write(0, 1, 1),
        write(1, 2, 1),
        write(1, 3, 3),
        Event::Recall { key: 0 },
        Event::Recall { key: 1 },
        Event::Conjunction { left: 0, right: 1 },
        Event::Ignore,
    ];
    let mut checked = 0usize;
    for mut code in 0..8_usize.pow(4) {
        let events: Vec<_> = (0..4)
            .map(|_| {
                let event = alphabet[code % 8];
                code /= 8;
                event
            })
            .collect();
        let mut b3 = BooleanStream::new(b3_config()).unwrap();
        let mut b4 = AlgebraicBooleanStream::new(b3_config()).unwrap();
        for event in events {
            assert_eq!(b4.step(event).unwrap(), b3.step(event).unwrap());
        }
        assert_eq!(b4.counters(), b3.counters());
        assert_eq!(b4.anf_work().pairwise_comparisons, 0);
        checked += 1;
    }
    assert_eq!(checked, 4096);
}

#[test]
fn b4_charges_anf_terms_without_hiding_b3_work() {
    let mut b3 = BooleanStream::new(b3_config()).unwrap();
    let mut b4 = AlgebraicBooleanStream::new(b3_config()).unwrap();
    let events = [
        write(7, 0, 1),
        write(7, 255, 3),
        Event::Recall { key: 7 },
        Event::Ignore,
    ];
    for event in events {
        assert_eq!(b4.step(event).unwrap(), b3.step(event).unwrap());
    }
    assert_eq!(b4.counters(), b3.counters());
    // Two valid writes * two canonical ANF monomials.
    assert_eq!(b4.anf_work().anf_term_evals, 4);
    assert_eq!(b4.anf_work().pairwise_comparisons, 0);
    assert_eq!(b4.arm().is_boolean_candidate(), true);
    assert_eq!(
        b4.footprint().anf_program_semantic_bits,
        marker_acceptance_program().semantic_bits()
    );
}

#[test]
fn rejected_invalid_marker_is_atomic_and_not_charged_as_anf_work() {
    let mut b4 = AlgebraicBooleanStream::new(b3_config()).unwrap();
    let before = b4.clone();
    assert_eq!(
        b4.step(write(1, 1, 4)),
        Err(AlgebraicStreamError::Stream(StreamError::InvalidMarker))
    );
    assert_eq!(b4, before);
}

#[test]
fn reset_preserves_program_and_capacity_but_clears_episode_work() {
    let mut b4 = AlgebraicBooleanStream::new(b3_config()).unwrap();
    let program = b4.admission_program().clone();
    let footprint = b4.footprint();
    b4.step(write(3, 5, 1)).unwrap();
    assert!(b4.anf_work().anf_term_evals > 0);
    b4.reset();
    assert_eq!(b4.admission_program(), &program);
    assert_eq!(b4.footprint(), footprint);
    assert_eq!(b4.anf_work().anf_term_evals, 0);
    assert_eq!(b4.counters().events, 0);
}
