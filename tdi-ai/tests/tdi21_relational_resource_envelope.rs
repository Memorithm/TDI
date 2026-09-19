#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi21_attention::{AttentionConfig, AttentionMode};
use tdi_ai::experimental::tdi21_relational_address_search::{
    AddressSample, DevelopmentAddressSet, development_from_relational_tasks, fit_relational_address,
};
use tdi_ai::experimental::tdi21_relational_resource_envelope::{
    ComponentEnvelopeError, build_declared_component_envelope,
};
use tdi_ai::experimental::tdi21_relational_tasks::DevelopmentRelationalSet;

fn attention(mode: AttentionMode) -> AttentionConfig {
    AttentionConfig {
        mode,
        history_capacity: 16,
        payload_bits: 4,
        max_events: 128,
    }
}

fn sparse_program_bits() -> usize {
    let development = development_from_relational_tasks(&DevelopmentRelationalSet::v1()).unwrap();
    fit_relational_address(&development)
        .unwrap()
        .anf_program_semantic_bits()
        .unwrap()
}

fn identity_program_bits() -> usize {
    let mut cases = vec![AddressSample {
        input: 0,
        expected: 0,
    }];
    for bit in 0..8 {
        let value = 1u8 << bit;
        cases.push(AddressSample {
            input: value,
            expected: value,
        });
    }
    fit_relational_address(&DevelopmentAddressSet::new(&cases).unwrap())
        .unwrap()
        .anf_program_semantic_bits()
        .unwrap()
}

#[test]
fn binary_attention_component_ceiling_maps_to_maximal_boolean_capacity() {
    assert_eq!(sparse_program_bits(), 456);
    let envelope =
        build_declared_component_envelope(attention(AttentionMode::BinaryQk), 456).unwrap();

    assert_eq!(envelope.attention_history_component_bits, 66_560);
    assert_eq!(envelope.attention_score_component_bits, 1_088);
    assert_eq!(envelope.attention_declared_component_bits, 67_648);
    assert_eq!(envelope.boolean_slots, 518);
    assert_eq!(envelope.boolean_memory_component_bits, 67_081);
    assert_eq!(envelope.boolean_declared_component_bits, 67_537);
    assert_eq!(envelope.unallocated_component_bits, 111);
}

#[test]
fn identity_program_cost_reduces_binary_matched_slot_capacity() {
    assert_eq!(identity_program_bits(), 584);
    let envelope =
        build_declared_component_envelope(attention(AttentionMode::BinaryQk), 584).unwrap();

    assert_eq!(envelope.attention_declared_component_bits, 67_648);
    assert_eq!(envelope.boolean_slots, 516);
    assert_eq!(envelope.boolean_memory_component_bits, 66_822);
    assert_eq!(envelope.boolean_declared_component_bits, 67_406);
    assert_eq!(envelope.unallocated_component_bits, 242);
}

#[test]
fn dense_attention_component_ceiling_is_reported_without_total_budget_claim() {
    let sparse = build_declared_component_envelope(attention(AttentionMode::DenseQk), 456).unwrap();
    let identity =
        build_declared_component_envelope(attention(AttentionMode::DenseQk), 584).unwrap();

    assert_eq!(sparse.attention_history_component_bits, 131_072);
    assert_eq!(sparse.attention_score_component_bits, 1_088);
    assert_eq!(sparse.attention_declared_component_bits, 132_160);
    assert_eq!(sparse.boolean_slots, 1_016);
    assert_eq!(sparse.boolean_declared_component_bits, 132_028);
    assert_eq!(sparse.unallocated_component_bits, 132);
    assert_eq!(identity.boolean_slots, 1_016);
    assert_eq!(identity.boolean_declared_component_bits, 132_156);
    assert_eq!(identity.unallocated_component_bits, 4);
}

#[test]
fn oversized_address_program_fails_before_fabricating_a_match() {
    let ceiling = 67_648;
    assert_eq!(
        build_declared_component_envelope(attention(AttentionMode::BinaryQk), ceiling + 1,),
        Err(ComponentEnvelopeError::NoTwoWayBooleanConfigurationFits)
    );
}

