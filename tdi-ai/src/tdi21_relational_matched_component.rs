//! Development-only same-episode comparison under one declared component ceiling.
//!
//! This is not a matched total-budget experiment. It matches only the current
//! representation components covered by `tdi21_relational_resource_envelope`
//! and preserves search/inference work as separate evidence.

use super::tdi21_attention::{AttentionConfig, AttentionMode};
use super::tdi21_relational_address_search::{AddressSearchResult, AddressSearchWork};
use super::tdi21_relational_attention::{
    RelationalAttentionConfig, RelationalAttentionError, RelationalAttentionOutcome,
    evaluate_relational_attention_episode,
};
use super::tdi21_relational_binding::{RelationalConfig, RelationalRead};
use super::tdi21_relational_learned::{
    LearnedAddressWork, LearnedRelationalBinder, LearnedRelationalError,
};
use super::tdi21_relational_resource_envelope::{
    ComponentEnvelopeError, DeclaredComponentEnvelope, build_declared_component_envelope,
};
use super::tdi21_relational_tasks::{
    RELATIONAL_V1_ENTITY_BITS, RELATIONAL_V1_RELATION_BITS, RelationalEpisode,
};
use super::tdi21_stream::{MemoryMode, StreamConfig, StreamCounters};

pub const RELATIONAL_MATCHED_COMPONENT_SEMANTICS: &str = "tdi21-relational-matched-component-v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatchedComponentError {
    EmptyFactSet,
    TooManyFactsForAttentionHistory,
    ArithmeticOverflow,
    AddressProgramInvalid,
    Envelope(ComponentEnvelopeError),
    Attention(RelationalAttentionError),
    Boolean(LearnedRelationalError),
}

impl From<ComponentEnvelopeError> for MatchedComponentError {
    fn from(value: ComponentEnvelopeError) -> Self {
        Self::Envelope(value)
    }
}
impl From<RelationalAttentionError> for MatchedComponentError {
    fn from(value: RelationalAttentionError) -> Self {
        Self::Attention(value)
    }
}
impl From<LearnedRelationalError> for MatchedComponentError {
    fn from(value: LearnedRelationalError) -> Self {
        Self::Boolean(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BooleanRelationalOutcome {
    pub observed: RelationalRead,
    pub expected: RelationalRead,
    pub correct: bool,
    pub counters: StreamCounters,
    pub address_work: LearnedAddressWork,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MatchedComponentEpisodeReport {
    pub envelope: DeclaredComponentEnvelope,
    pub attention: RelationalAttentionOutcome,
    pub boolean: BooleanRelationalOutcome,
    pub development_search_work: AddressSearchWork,
    pub development_samples: usize,
    pub development_bit_mismatches: u64,
}

pub fn evaluate_matched_component_episode(
    mode: AttentionMode,
    selected_address: &AddressSearchResult,
    episode: &RelationalEpisode,
) -> Result<MatchedComponentEpisodeReport, MatchedComponentError> {
    if episode.facts.is_empty() {
        return Err(MatchedComponentError::EmptyFactSet);
    }
    let history_capacity = episode.facts.len();
    if history_capacity > 256 {
        return Err(MatchedComponentError::TooManyFactsForAttentionHistory);
    }
    let required_events = episode
        .facts
        .len()
        .checked_add(episode.query.relations.len())
        .ok_or(MatchedComponentError::ArithmeticOverflow)?;
    let max_events =
        u64::try_from(required_events).map_err(|_| MatchedComponentError::ArithmeticOverflow)?;

    let attention_config = AttentionConfig {
        mode,
        history_capacity,
        payload_bits: RELATIONAL_V1_ENTITY_BITS,
        max_events,
    };
    let program_bits = selected_address
        .anf_program_semantic_bits()
        .map_err(|_| MatchedComponentError::AddressProgramInvalid)?;
    let envelope = build_declared_component_envelope(attention_config, program_bits)?;

    let attention = evaluate_relational_attention_episode(
        RelationalAttentionConfig {
            attention: attention_config,
            entity_bits: RELATIONAL_V1_ENTITY_BITS,
            relation_bits: RELATIONAL_V1_RELATION_BITS,
        },
        episode,
    )?;

    let mut boolean = LearnedRelationalBinder::new(
        RelationalConfig {
            stream: StreamConfig {
                mode: MemoryMode::TwoWay,
                slots: envelope.boolean_slots,
                payload_bits: RELATIONAL_V1_ENTITY_BITS,
                max_events,
                route_salt: 0x5444_4932_3100_0021,
            },
            entity_bits: RELATIONAL_V1_ENTITY_BITS,
            relation_bits: RELATIONAL_V1_RELATION_BITS,
        },
        selected_address.clone(),
    )?;
    let observed = boolean.run_episode(episode)?;
    let boolean_outcome = BooleanRelationalOutcome {
        observed,
        expected: episode.expected(),
        correct: observed == episode.expected(),
        counters: boolean.counters(),
        address_work: boolean.work(),
    };

    Ok(MatchedComponentEpisodeReport {
        envelope,
        attention,
        boolean: boolean_outcome,
        development_search_work: selected_address.work(),
        development_samples: selected_address.development_samples(),
        development_bit_mismatches: selected_address.development_bit_mismatches(),
    })
}
