//! Development-only relational binding/retrieval prototype for TDI-21.
//!
//! The prototype deliberately reuses the bounded BooleanStream memory rather
//! than introducing token-pair similarity. Relation/subject pairs are packed
//! into exact symbolic identifiers and addressed directly. It is a task harness
//! for later B4/B5 routing research, not evidence of learned relational generalization.

use super::tdi21::{BooleanState, MemoryRead};
use super::tdi21_stream::{
    BooleanStream, Event, StepOutput, StreamConfig, StreamCounters, StreamError,
};

pub const RELATIONAL_BINDING_SEMANTICS: &str = "tdi21-relational-binding-v1";
pub const MAX_COMPOSITION_HOPS: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RelationalConfig {
    pub stream: StreamConfig,
    pub entity_bits: u8,
    pub relation_bits: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelationalError {
    InvalidEntityWidth,
    InvalidRelationWidth,
    AddressWidthOverflow,
    EntityOutOfRange,
    RelationOutOfRange,
    EmptyRelationPath,
    TooManyCompositionHops,
    UnexpectedQuiet,
    Stream(StreamError),
}

impl From<StreamError> for RelationalError {
    fn from(value: StreamError) -> Self {
        Self::Stream(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelationalRead {
    Hit(u64),
    Miss,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationalBinder {
    config: RelationalConfig,
    stream: BooleanStream,
}

impl RelationalBinder {
    pub fn new(config: RelationalConfig) -> Result<Self, RelationalError> {
        if config.entity_bits == 0 || config.entity_bits > 32 {
            return Err(RelationalError::InvalidEntityWidth);
        }
        if config.relation_bits == 0 || config.relation_bits > 16 {
            return Err(RelationalError::InvalidRelationWidth);
        }
        if u16::from(config.entity_bits) + u16::from(config.relation_bits) > 63 {
            return Err(RelationalError::AddressWidthOverflow);
        }
        if config.stream.payload_bits < config.entity_bits {
            return Err(RelationalError::InvalidEntityWidth);
        }
        Ok(Self {
            config,
            stream: BooleanStream::new(config.stream)?,
        })
    }

    #[must_use]
    pub fn counters(&self) -> StreamCounters {
        self.stream.counters()
    }

    pub fn reset(&mut self) {
        self.stream.reset();
    }

    fn entity_limit(&self) -> u64 {
        1u64 << self.config.entity_bits
    }

    fn relation_limit(&self) -> u64 {
        1u64 << self.config.relation_bits
    }

    fn address(&self, relation: u64, subject: u64) -> Result<u64, RelationalError> {
        if subject >= self.entity_limit() {
            return Err(RelationalError::EntityOutOfRange);
        }
        if relation >= self.relation_limit() {
            return Err(RelationalError::RelationOutOfRange);
        }
        Ok((relation << self.config.entity_bits) | subject)
    }

    fn decode_reply(output: StepOutput) -> Result<RelationalRead, RelationalError> {
        match output {
            StepOutput::Reply(MemoryRead::Hit(state)) => Ok(RelationalRead::Hit(state.bits())),
            StepOutput::Reply(MemoryRead::Miss) => Ok(RelationalRead::Miss),
            StepOutput::Quiet => Err(RelationalError::UnexpectedQuiet),
        }
    }

    pub fn bind(
        &mut self,
        relation: u64,
        subject: u64,
        object: u64,
    ) -> Result<(), RelationalError> {
        if object >= self.entity_limit() {
            return Err(RelationalError::EntityOutOfRange);
        }
        let key = self.address(relation, subject)?;
        self.stream.step(Event::Write {
            key,
            payload: BooleanState::from_bits(object),
            marker: BooleanState::from_bits(1),
        })?;
        Ok(())
    }

    pub fn recall(
        &mut self,
        relation: u64,
        subject: u64,
    ) -> Result<RelationalRead, RelationalError> {
        let key = self.address(relation, subject)?;
        Self::decode_reply(self.stream.step(Event::Recall { key })?)
    }

    /// Follow a bounded relation path without scanning prior events.
    /// Every hop performs one exact-address memory lookup. Missing intermediate
    /// state terminates the path as an explicit miss.
    pub fn compose_path(
        &mut self,
        relations: &[u64],
        subject: u64,
    ) -> Result<RelationalRead, RelationalError> {
        if relations.is_empty() {
            return Err(RelationalError::EmptyRelationPath);
        }
        if relations.len() > MAX_COMPOSITION_HOPS {
            return Err(RelationalError::TooManyCompositionHops);
        }
        if subject >= self.entity_limit() {
            return Err(RelationalError::EntityOutOfRange);
        }

        let mut current = subject;
        for &relation in relations {
            current = match self.recall(relation, current)? {
                RelationalRead::Hit(value) => value,
                RelationalRead::Miss => return Ok(RelationalRead::Miss),
            };
        }
        Ok(RelationalRead::Hit(current))
    }

    pub fn compose2(
        &mut self,
        first_relation: u64,
        second_relation: u64,
        subject: u64,
    ) -> Result<RelationalRead, RelationalError> {
        self.compose_path(&[first_relation, second_relation], subject)
    }
}
