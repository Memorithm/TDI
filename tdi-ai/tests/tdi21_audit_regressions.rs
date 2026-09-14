#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi21::{
    AnfTerm, ArchitectureArm, BooleanState, CandidateConfig, Clause, ClauseValidationError,
    DirectAddressMemory, ForbiddenMechanisms, Literal, MemoryRead, ResourceCounters,
    activate_route, counted_evaluate_anf, counted_route_tag, delayed_bit_recall, route_tag,
};

fn old_route(mut value: u64, salt: u64) -> u64 {
    value ^= salt;
    value ^= value.rotate_left(13);
    value ^= value.rotate_right(7);
    value ^ value.rotate_left(17)
}

fn binary_rank(columns: impl IntoIterator<Item = u64>) -> usize {
    let mut basis = [0_u64; 64];
    let mut rank = 0;
    for mut column in columns {
        while column != 0 {
            let pivot = 63 - column.leading_zeros() as usize;
            if basis[pivot] == 0 {
                basis[pivot] = column;
                rank += 1;
                break;
            }
            column ^= basis[pivot];
        }
    }
    rank
}

fn undo_left(mut value: u64, mut shift: u32) -> u64 {
    while shift < 64 {
        value ^= value << shift;
        shift *= 2;
    }
    value
}

fn undo_right(mut value: u64, mut shift: u32) -> u64 {
    while shift < 64 {
        value ^= value >> shift;
        shift *= 2;
    }
    value
}

fn inverse_route(value: u64, salt: u64) -> u64 {
    undo_left(undo_right(undo_left(value, 17), 7), 13) ^ salt
}

#[test]
fn preserve_historical_rank_loss_counterexample() {
    let salt = 0x5444_4932_3100_0001;
    assert_eq!(old_route(0, salt), old_route(u64::MAX, salt));
    assert_eq!(binary_rank((0..64).map(|i| old_route(1 << i, 0))), 61);
    assert_eq!(
        binary_rank((0..64).map(|i| route_tag(BooleanState::from_bits(1 << i), 0))),
        64
    );
}

#[test]
fn corrected_route_has_independent_inverse_and_preserves_complements() {
    for salt in [0, 7, 0x5444_4932_3100_0001, u64::MAX] {
        for value in 0..=u16::MAX as u64 {
            let tag = route_tag(BooleanState::from_bits(value), salt);
            assert_eq!(inverse_route(tag, salt), value);
            assert_ne!(tag, route_tag(BooleanState::from_bits(!value), salt));
        }
        for bit in 0..64 {
            let value = 1_u64 << bit;
            assert_eq!(
                inverse_route(route_tag(BooleanState::from_bits(value), salt), salt),
                value
            );
        }
    }
}

#[test]
fn complementary_keys_cannot_be_silent_false_hits() {
    let first = route_tag(BooleanState::from_bits(0), 7);
    let second = route_tag(BooleanState::from_bits(u64::MAX), 7);
    let mut memory = DirectAddressMemory::<1>::default();
    let mut work = ResourceCounters::default();
    memory.write(first, BooleanState::from_bits(1), &mut work);
    assert_eq!(memory.read(second, &mut work), MemoryRead::Miss);
    memory.write(second, BooleanState::from_bits(0), &mut work);
    assert_eq!(memory.read(first, &mut work), MemoryRead::Miss);
    assert_eq!(work.memory_collisions, 1);
    assert_eq!(work.memory_replacements, 1);
    assert_eq!(work.tag_equality_checks, 3);
}

#[test]
fn full_width_modulo_precedes_pointer_narrowing() {
    let mut memory = DirectAddressMemory::<3>::default();
    let mut work = ResourceCounters::default();
    memory.write(1, BooleanState::from_bits(1), &mut work);
    memory.write(1_u64 << 32, BooleanState::from_bits(2), &mut work);
    // Both full-width identifiers are congruent to one modulo three.
    assert_eq!(work.memory_collisions, 1);
    assert_eq!(memory.read(1, &mut work), MemoryRead::Miss);
    assert_eq!(
        memory.read(1_u64 << 32, &mut work),
        MemoryRead::Hit(BooleanState::from_bits(2))
    );
}

#[test]
fn invalid_literal_is_not_true_after_negation() {
    for bit in 64..=u8::MAX {
        for literal in [Literal::Bit(bit), Literal::NotBit(bit)] {
            let clause = Clause {
                literals: [literal],
            };
            let mut work = ResourceCounters::default();
            assert_eq!(
                clause.validate(64),
                Err(ClauseValidationError::BitOutOfRange(bit))
            );
            assert!(!activate_route(
                BooleanState::from_bits(0),
                &clause,
                &mut work
            ));
            assert_eq!(work.route_activations, 0);
        }
    }
}

#[test]
fn clause_validation_observes_declared_width_and_short_circuit_tail() {
    let clause = Clause {
        literals: [Literal::Bit(0), Literal::NotBit(8)],
    };
    assert_eq!(
        clause.validate(8),
        Err(ClauseValidationError::BitOutOfRange(8))
    );
    assert_eq!(clause.validate(9), Ok(()));
    assert_eq!(clause.validate(0), Err(ClauseValidationError::InvalidWidth));
    assert_eq!(
        clause.validate(65),
        Err(ClauseValidationError::InvalidWidth)
    );
}

#[test]
fn invalid_manifest_dimensions_are_not_structurally_valid() {
    assert!(!delayed_bit_recall::<0>(0, false).candidate_structurally_valid());
    let valid = delayed_bit_recall::<1>(0, false);
    let mut evidence = valid;
    evidence.manifest.sequence_len = 0;
    assert!(!evidence.candidate_structurally_valid());
    for width in [0, 65, u8::MAX] {
        evidence = valid;
        evidence.manifest.state_width_bits = width;
        assert!(!evidence.candidate_structurally_valid());
    }
}

#[test]
fn every_prohibited_declaration_is_rejected() {
    for flag in 0..8 {
        let mut forbidden = ForbiddenMechanisms::default();
        match flag {
            0 => forbidden.qkv_projection = true,
            1 => forbidden.pairwise_dot_product = true,
            2 => forbidden.cosine_similarity = true,
            3 => forbidden.softmax = true,
            4 => forbidden.hamming_attention_score = true,
            5 => forbidden.dense_pairwise_matrix = true,
            6 => forbidden.attention_fallback = true,
            _ => forbidden.linear_history_scan_addressing = true,
        }
        let config = CandidateConfig {
            arm: ArchitectureArm::B2BooleanDirect,
            state_width_bits: 64,
            memory_slots: 1,
            forbidden,
        };
        assert!(config.validate().is_err());
    }
}

#[test]
fn counted_routes_and_anf_do_not_disappear_from_work_records() {
    let mut work = ResourceCounters::default();
    let state = BooleanState::from_bits(5);
    assert_eq!(counted_route_tag(state, 7, &mut work), route_tag(state, 7));
    assert_eq!(work.address_derivations, 1);
    assert_eq!(work.word_boolean_evals, 7);
    let terms = [AnfTerm { variables: 0 }, AnfTerm { variables: 0 }];
    assert!(!counted_evaluate_anf(false, &terms, 0, &mut work));
    assert_eq!(work.anf_term_evals, 2);
    assert_eq!(work.pairwise_comparisons, 0);
}

#[test]
#[should_panic(expected = "TDI-21 counter overflow")]
fn counter_overflow_cannot_wrap_into_valid_evidence() {
    let memory = DirectAddressMemory::<1>::default();
    let mut work = ResourceCounters {
        memory_reads: u64::MAX,
        ..ResourceCounters::default()
    };
    let _ = memory.read(0, &mut work);
}
