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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RelationalWork {
    /// One semantic derivation of the packed `(relation, subject)` identifier.
    /// This is not a CPU-instruction or cycle count.
    pub address_derivations: u64,
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
    EventBudgetExhausted,
    CounterOverflow,
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
    work: RelationalWork,
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
            work: RelationalWork::default(),
        })
    }

    #[must_use]
    pub fn counters(&self) -> StreamCounters {
        self.stream.counters()
    }

    #[must_use]
    pub const fn relational_work(&self) -> RelationalWork {
        self.work
    }

    pub fn reset(&mut self) {
        self.stream.reset();
        self.work = RelationalWork::default();
    }

    fn entity_limit(&self) -> u64 {
        1u64 << self.config.entity_bits
    }

    fn relation_limit(&self) -> u64 {
        1u64 << self.config.relation_bits
    }

    fn packed_address(&self, relation: u64, subject: u64) -> Result<u64, RelationalError> {
        if subject >= self.entity_limit() {
            return Err(RelationalError::EntityOutOfRange);
        }
        if relation >= self.relation_limit() {
            return Err(RelationalError::RelationOutOfRange);
        }
        Ok((relation << self.config.entity_bits) | subject)
    }

    fn next_address_count(&self) -> Result<u64, RelationalError> {
        self.work
            .address_derivations
            .checked_add(1)
            .ok_or(RelationalError::CounterOverflow)
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
        let key = self.packed_address(relation, subject)?;
        let next_address_count = self.next_address_count()?;
        self.stream.step(Event::Write {
            key,
            payload: BooleanState::from_bits(object),
            marker: BooleanState::from_bits(1),
        })?;
        self.work.address_derivations = next_address_count;
        Ok(())
    }

    pub fn recall(
        &mut self,
        relation: u64,
        subject: u64,
    ) -> Result<RelationalRead, RelationalError> {
        let key = self.packed_address(relation, subject)?;
        let next_address_count = self.next_address_count()?;
        let output = self.stream.step(Event::Recall { key })?;
        self.work.address_derivations = next_address_count;
        Self::decode_reply(output)
    }

    /// Follow a bounded relation path without scanning prior events.
    /// Every hop performs one exact-address memory lookup. Missing intermediate
    /// state terminates the path as an explicit miss. Contract errors are
    /// validated before the first lookup so they cannot leave partial evidence.
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
        if relations
            .iter()
            .any(|&relation| relation >= self.relation_limit())
        {
            return Err(RelationalError::RelationOutOfRange);
        }
        let required = u64::try_from(relations.len()).expect("bounded hop count fits u64");
        if self
            .stream
            .counters()
            .events
            .checked_add(required)
            .is_none_or(|events| events > self.config.stream.max_events)
        {
            return Err(RelationalError::EventBudgetExhausted);
        }
        if self.work.address_derivations.checked_add(required).is_none() {
            return Err(RelationalError::CounterOverflow);
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
