//! Development-only functional use of a selected relational address program.
//!
//! A selected unary Boolean encoder maps the packed relation/subject input to a
//! bounded memory key. This tests task-level transfer separately from exact
//! address reconstruction. Non-injective encoders can alias distinct symbolic
//! identities; that limitation is explicit and retained in controls.

use super::tdi21::{BooleanState, MemoryRead};
use super::tdi21_relational_address_search::{
    AddressSearchResult, RELATIONAL_ADDRESS_BITS,
};
use super::tdi21_relational_binding::{MAX_COMPOSITION_HOPS, RelationalConfig, RelationalRead};
use super::tdi21_relational_tasks::{
    RELATIONAL_V1_ENTITY_BITS, RELATIONAL_V1_RELATION_BITS, RelationalEpisode,
};
use super::tdi21_stream::{
    BooleanStream, Event, StepOutput, StreamCounters, StreamError,
};

pub const LEARNED_RELATIONAL_ADDRESS_SEMANTICS: &str =
    "tdi21-learned-relational-address-v1";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LearnedAddressWork {
    pub predictions: u64,
    pub rule_evaluations: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LearnedRelationalError {
    RequiresV1Widths,
    EntityOutOfRange,
    RelationOutOfRange,
    EmptyRelationPath,
    TooManyCompositionHops,
    EventBudgetExhausted,
    CounterOverflow,
    UnexpectedQuiet,
    Stream(StreamError),
}

impl From<StreamError> for LearnedRelationalError {
    fn from(value: StreamError) -> Self {
        Self::Stream(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LearnedRelationalBinder {
    config: RelationalConfig,
    stream: BooleanStream,
    encoder: AddressSearchResult,
    work: LearnedAddressWork,
}

impl LearnedRelationalBinder {
    pub fn new(
        config: RelationalConfig,
        encoder: AddressSearchResult,
    ) -> Result<Self, LearnedRelationalError> {
        if config.entity_bits != RELATIONAL_V1_ENTITY_BITS
            || config.relation_bits != RELATIONAL_V1_RELATION_BITS
            || RELATIONAL_ADDRESS_BITS != 8
        {
            return Err(LearnedRelationalError::RequiresV1Widths);
        }
        Ok(Self {
            config,
            stream: BooleanStream::new(config.stream)?,
            encoder,
            work: LearnedAddressWork::default(),
        })
    }

    #[must_use]
    pub fn counters(&self) -> StreamCounters {
        self.stream.counters()
    }

    #[must_use]
    pub const fn work(&self) -> LearnedAddressWork {
        self.work
    }

    #[must_use]
    pub const fn encoder(&self) -> &AddressSearchResult {
        &self.encoder
    }

    pub fn reset(&mut self) {
        self.stream.reset();
        self.work = LearnedAddressWork::default();
    }

    fn pack_input(&self, relation: u64, subject: u64) -> Result<u8, LearnedRelationalError> {
        let entity_limit = 1u64 << self.config.entity_bits;
        let relation_limit = 1u64 << self.config.relation_bits;
        if subject >= entity_limit {
            return Err(LearnedRelationalError::EntityOutOfRange);
        }
        if relation >= relation_limit {
            return Err(LearnedRelationalError::RelationOutOfRange);
        }
        Ok(((relation << self.config.entity_bits) | subject) as u8)
    }

    fn staged_key(
        &self,
        relation: u64,
        subject: u64,
    ) -> Result<(u64, LearnedAddressWork), LearnedRelationalError> {
        let input = self.pack_input(relation, subject)?;
        let mut staged = self.work;
        staged.predictions = staged
            .predictions
            .checked_add(1)
            .ok_or(LearnedRelationalError::CounterOverflow)?;
        staged.rule_evaluations = staged
            .rule_evaluations
            .checked_add(u64::from(RELATIONAL_ADDRESS_BITS))
            .ok_or(LearnedRelationalError::CounterOverflow)?;
        Ok((u64::from(self.encoder.predict(input)), staged))
    }

    fn decode(output: StepOutput) -> Result<RelationalRead, LearnedRelationalError> {
        match output {
            StepOutput::Reply(MemoryRead::Hit(state)) => Ok(RelationalRead::Hit(state.bits())),
            StepOutput::Reply(MemoryRead::Miss) => Ok(RelationalRead::Miss),
            StepOutput::Quiet => Err(LearnedRelationalError::UnexpectedQuiet),
        }
    }

    pub fn bind(
        &mut self,
        relation: u64,
        subject: u64,
        object: u64,
    ) -> Result<(), LearnedRelationalError> {
        if object >= (1u64 << self.config.entity_bits) {
            return Err(LearnedRelationalError::EntityOutOfRange);
        }
        let (key, staged) = self.staged_key(relation, subject)?;
        self.stream.step(Event::Write {
            key,
            payload: BooleanState::from_bits(object),
            marker: BooleanState::from_bits(1),
        })?;
        self.work = staged;
        Ok(())
    }

    pub fn recall(
        &mut self,
        relation: u64,
        subject: u64,
    ) -> Result<RelationalRead, LearnedRelationalError> {
        let (key, staged) = self.staged_key(relation, subject)?;
        let output = self.stream.step(Event::Recall { key })?;
        self.work = staged;
        Self::decode(output)
    }

    pub fn compose_path(
        &mut self,
        relations: &[u64],
        subject: u64,
    ) -> Result<RelationalRead, LearnedRelationalError> {
        if relations.is_empty() {
            return Err(LearnedRelationalError::EmptyRelationPath);
        }
        if relations.len() > MAX_COMPOSITION_HOPS {
            return Err(LearnedRelationalError::TooManyCompositionHops);
        }
        let entity_limit = 1u64 << self.config.entity_bits;
        let relation_limit = 1u64 << self.config.relation_bits;
        if subject >= entity_limit {
            return Err(LearnedRelationalError::EntityOutOfRange);
        }
        if relations.iter().any(|&relation| relation >= relation_limit) {
            return Err(LearnedRelationalError::RelationOutOfRange);
        }
        let required = u64::try_from(relations.len()).expect("bounded path length fits u64");
        if self
            .stream
            .counters()
            .events
            .checked_add(required)
            .is_none_or(|events| events > self.config.stream.max_events)
        {
            return Err(LearnedRelationalError::EventBudgetExhausted);
        }
        if self.work.predictions.checked_add(required).is_none()
            || self
                .work
                .rule_evaluations
                .checked_add(required * u64::from(RELATIONAL_ADDRESS_BITS))
                .is_none()
        {
            return Err(LearnedRelationalError::CounterOverflow);
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

    pub fn run_episode(
        &mut self,
        episode: &RelationalEpisode,
    ) -> Result<RelationalRead, LearnedRelationalError> {
        for fact in &episode.facts {
            self.bind(fact.relation, fact.subject, fact.object)?;
        }
        self.compose_path(&episode.query.relations, episode.query.subject)
    }
}
