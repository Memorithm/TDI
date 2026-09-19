//! Development-only declared-component envelope for TDI-21 relational controls.
//!
//! This module equalizes only the explicitly reported representation components
//! of the current B0/B1 attention references and a two-way Boolean relational
//! memory plus address-program bits. It does NOT claim matched total memory,
//! compute, training/search cost, allocator overhead, bandwidth, latency or
//! hardware energy.

use super::tdi21_attention::{
    AttentionConfig, AttentionError, AttentionFootprint, AttentionMode, AttentionReference,
};
use super::tdi21_stream::{
    BooleanStream, MAX_SLOTS, MemoryFootprint, MemoryMode, StreamConfig, StreamError,
};

pub const RELATIONAL_COMPONENT_ENVELOPE_SEMANTICS: &str =
    "tdi21-relational-component-envelope-v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComponentEnvelopeError {
    Attention(AttentionError),
    Boolean(StreamError),
    ArithmeticOverflow,
    NoTwoWayBooleanConfigurationFits,
}

impl From<AttentionError> for ComponentEnvelopeError {
    fn from(value: AttentionError) -> Self {
        Self::Attention(value)
    }
}

impl From<StreamError> for ComponentEnvelopeError {
    fn from(value: StreamError) -> Self {
        Self::Boolean(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeclaredComponentEnvelope {
    pub attention_mode: AttentionMode,
    pub attention_history_capacity: usize,
    pub attention_history_component_bits: usize,
    pub attention_score_component_bits: usize,
    pub attention_declared_component_bits: usize,
    pub attention_reserved_buffer_bytes: usize,
    pub attention_inline_bytes: usize,

    pub boolean_slots: usize,
    pub boolean_memory_component_bits: usize,
    pub boolean_address_program_bits: usize,
    pub boolean_declared_component_bits: usize,
    pub boolean_reserved_buffer_bytes: usize,
    pub boolean_inline_bytes: usize,

    pub unallocated_component_bits: usize,
}

fn attention_component_bits(footprint: AttentionFootprint) -> Result<usize, ComponentEnvelopeError> {
    footprint
        .history_component_bits
        .checked_add(footprint.score_component_bits)
        .ok_or(ComponentEnvelopeError::ArithmeticOverflow)
}

fn boolean_memory_component_bits(
    footprint: MemoryFootprint,
) -> Result<usize, ComponentEnvelopeError> {
    footprint
        .entry_semantic_bits
        .checked_add(footprint.replacement_semantic_bits)
        .ok_or(ComponentEnvelopeError::ArithmeticOverflow)
}

fn boolean_footprint(
    slots: usize,
    payload_bits: u8,
    max_events: u64,
) -> Result<MemoryFootprint, ComponentEnvelopeError> {
    Ok(BooleanStream::new(StreamConfig {
        mode: MemoryMode::TwoWay,
        slots,
        payload_bits,
        max_events,
        route_salt: 0,
    })?
    .footprint())
}

/// Match the current two-way Boolean relational substrate to an attention
/// reference's declared representation-component ceiling.
///
/// The returned Boolean slot count is maximal under that ceiling after charging
/// the supplied address-program bits. This is intentionally NOT a total-budget match.
pub fn build_declared_component_envelope(
    attention: AttentionConfig,
    address_program_bits: usize,
) -> Result<DeclaredComponentEnvelope, ComponentEnvelopeError> {
    let reference = AttentionReference::new(attention)?;
    let attention_footprint = reference.footprint();
    let ceiling = attention_component_bits(attention_footprint)?;

    let mut best: Option<(usize, MemoryFootprint, usize)> = None;
    let mut slots = 2usize;
    while slots <= MAX_SLOTS {
        let footprint = boolean_footprint(slots, attention.payload_bits, attention.max_events)?;
        let memory_bits = boolean_memory_component_bits(footprint)?;
        let total = memory_bits
            .checked_add(address_program_bits)
            .ok_or(ComponentEnvelopeError::ArithmeticOverflow)?;
        if total <= ceiling {
            best = Some((slots, footprint, total));
        } else {
            break;
        }
        slots = slots
            .checked_add(2)
            .ok_or(ComponentEnvelopeError::ArithmeticOverflow)?;
    }

    let Some((boolean_slots, boolean_footprint, boolean_total)) = best else {
        return Err(ComponentEnvelopeError::NoTwoWayBooleanConfigurationFits);
    };
    let boolean_memory_bits = boolean_memory_component_bits(boolean_footprint)?;
    let unallocated_component_bits = ceiling
        .checked_sub(boolean_total)
        .ok_or(ComponentEnvelopeError::ArithmeticOverflow)?;

    Ok(DeclaredComponentEnvelope {
        attention_mode: attention.mode,
        attention_history_capacity: attention.history_capacity,
        attention_history_component_bits: attention_footprint.history_component_bits,
        attention_score_component_bits: attention_footprint.score_component_bits,
        attention_declared_component_bits: ceiling,
        attention_reserved_buffer_bytes: attention_footprint.reserved_buffer_bytes,
        attention_inline_bytes: attention_footprint.inline_bytes,

        boolean_slots,
        boolean_memory_component_bits: boolean_memory_bits,
        boolean_address_program_bits: address_program_bits,
        boolean_declared_component_bits: boolean_total,
        boolean_reserved_buffer_bytes: boolean_footprint.reserved_buffer_bytes,
        boolean_inline_bytes: boolean_footprint.inline_bytes,

        unallocated_component_bits,
    })
}