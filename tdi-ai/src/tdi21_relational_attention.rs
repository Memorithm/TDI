//! Development-only B0/B1 relational attention controls for TDI-21.
//!
//! These controls execute the same explicit relational facts and bounded
//! relation-path queries as the Boolean relational harness, but storage/retrieval
//! is delegated to the existing isolated attention references. They are not
//! trained Transformers and are not a matched-total-budget comparison.

use super::tdi21::{BooleanState, MemoryRead};
use super::tdi21_attention::{
    AttentionConfig, AttentionError, AttentionFootprint, AttentionMode, AttentionReference,
    AttentionWork,
};
use super::tdi21_relational_binding::{MAX_COMPOSITION_HOPS, RelationalRead};
use super::tdi21_relational_tasks::RelationalEpisode;
use super::tdi21_stream::{Event, StepOutput};

pub const RELATIONAL_ATTENTION_SEMANTICS: &str = "tdi21-relational-attention-controls-v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RelationalAttentionConfig {
    pub attention: AttentionConfig,
    pub entity_bits: u8,
    pub relation_bits: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelationalAttentionError {
    InvalidEntityWidth,
    InvalidRelationWidth,
    AddressWidthOverflow,
    PayloadWidthMismatch,
    EntityOutOfRange,
    RelationOutOfRange,
    EmptyRelationPath,
    TooManyCompositionHops,
    EventBudgetExhausted,
    HistoryCapacityExhausted,
    CounterOverflow,
    UnexpectedQuiet,
    Attention(AttentionError),
}

impl From<AttentionError> for RelationalAttentionError {
    fn from(value: AttentionError) -> Self {
        Self::Attention(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RelationalAttentionOutcome {
    pub observed: RelationalRead,
    pub expected: RelationalRead,
    pub correct: bool,
    pub mode: AttentionMode,
    pub work: AttentionWork,
    pub footprint: AttentionFootprint,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RelationalAttentionReference {
    config: RelationalAttentionConfig,
    inner: AttentionReference,
}

impl RelationalAttentionReference {
    pub fn new(config: RelationalAttentionConfig) -> Result<Self, RelationalAttentionError> {
        if config.entity_bits == 0 || config.entity_bits > 32 {
            return Err(RelationalAttentionError::InvalidEntityWidth);
        }
        if config.relation_bits == 0 || config.relation_bits > 16 {
            return Err(RelationalAttentionError::InvalidRelationWidth);
        }
        let address_bits = config
            .entity_bits
            .checked_add(config.relation_bits)
            .ok_or(RelationalAttentionError::AddressWidthOverflow)?;
        if address_bits > 63 {
            return Err(RelationalAttentionError::AddressWidthOverflow);
        }
        if config.attention.payload_bits < config.entity_bits {
            return Err(RelationalAttentionError::PayloadWidthMismatch);
        }
        Ok(Self {
            config,
            inner: AttentionReference::new(config.attention)?,
        })
    }

    #[must_use]
    pub const fn config(&self) -> RelationalAttentionConfig {
        self.config
    }

    #[must_use]
    pub fn work(&self) -> AttentionWork {
        self.inner.work()
    }

    #[must_use]
    pub fn footprint(&self) -> AttentionFootprint {
        self.inner.footprint()
    }

    #[must_use]
    pub fn stored_events(&self) -> usize {
        self.inner.stored_events()
    }

    pub fn reset(&mut self) {
        self.inner.reset();
    }

    fn entity_limit(&self) -> u64 {
        1u64 << self.config.entity_bits
    }

    fn relation_limit(&self) -> u64 {
        1u64 << self.config.relation_bits
    }

    fn pack(&self, relation: u64, subject: u64) -> Result<u64, RelationalAttentionError> {
        if subject >= self.entity_limit() {
            return Err(RelationalAttentionError::EntityOutOfRange);
        }
        if relation >= self.relation_limit() {
            return Err(RelationalAttentionError::RelationOutOfRange);
        }
        Ok((relation << self.config.entity_bits) | subject)
    }

    fn decode(output: StepOutput) -> Result<RelationalRead, RelationalAttentionError> {
        match output {
            StepOutput::Reply(MemoryRead::Hit(state)) => Ok(RelationalRead::Hit(state.bits())),
            StepOutput::Reply(MemoryRead::Miss) => Ok(RelationalRead::Miss),
            StepOutput::Quiet => Err(RelationalAttentionError::UnexpectedQuiet),
        }
    }

    pub fn bind(
        &mut self,
        relation: u64,
        subject: u64,
        object: u64,
    ) -> Result<(), RelationalAttentionError> {
        if object >= self.entity_limit() {
            return Err(RelationalAttentionError::EntityOutOfRange);
        }
        let key = self.pack(relation, subject)?;
        self.inner.step(Event::Write {
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
    ) -> Result<RelationalRead, RelationalAttentionError> {
        let key = self.pack(relation, subject)?;
        Self::decode(self.inner.step(Event::Recall { key })?)
    }

    fn validate_path(
        &self,
        relations: &[u64],
        subject: u64,
    ) -> Result<(), RelationalAttentionError> {
        if relations.is_empty() {
            return Err(RelationalAttentionError::EmptyRelationPath);
        }
        if relations.len() > MAX_COMPOSITION_HOPS {
            return Err(RelationalAttentionError::TooManyCompositionHops);
        }
        if subject >= self.entity_limit() {
            return Err(RelationalAttentionError::EntityOutOfRange);
        }
        if relations
            .iter()
            .any(|&relation| relation >= self.relation_limit())
        {
            return Err(RelationalAttentionError::RelationOutOfRange);
        }
        let required = u64::try_from(relations.len())
            .map_err(|_| RelationalAttentionError::CounterOverflow)?;
        if self
            .inner
            .work()
            .events
            .checked_add(required)
            .is_none_or(|events| events > self.config.attention.max_events)
        {
            return Err(RelationalAttentionError::EventBudgetExhausted);
        }
        Ok(())
    }

    pub fn compose_path(
        &mut self,
        relations: &[u64],
        subject: u64,
    ) -> Result<RelationalRead, RelationalAttentionError> {
        self.validate_path(relations, subject)?;
        let mut current = subject;
        for &relation in relations {
            current = match self.recall(relation, current)? {
                RelationalRead::Hit(value) => value,
                RelationalRead::Miss => return Ok(RelationalRead::Miss),
            };
        }
        Ok(RelationalRead::Hit(current))
    }

    fn validate_episode(
        &self,
        episode: &RelationalEpisode,
    ) -> Result<(), RelationalAttentionError> {
        self.validate_path(&episode.query.relations, episode.query.subject)?;
        for fact in &episode.facts {
            if fact.subject >= self.entity_limit() || fact.object >= self.entity_limit() {
                return Err(RelationalAttentionError::EntityOutOfRange);
            }
            if fact.relation >= self.relation_limit() {
                return Err(RelationalAttentionError::RelationOutOfRange);
            }
        }
        let needed_history = self
            .stored_events()
            .checked_add(episode.facts.len())
            .ok_or(RelationalAttentionError::CounterOverflow)?;
        if needed_history > self.config.attention.history_capacity {
            return Err(RelationalAttentionError::HistoryCapacityExhausted);
        }
        let writes = u64::try_from(episode.facts.len())
            .map_err(|_| RelationalAttentionError::CounterOverflow)?;
        let reads = u64::try_from(episode.query.relations.len())
            .map_err(|_| RelationalAttentionError::CounterOverflow)?;
        let required = writes
            .checked_add(reads)
            .ok_or(RelationalAttentionError::CounterOverflow)?;
        if self
            .inner
            .work()
            .events
            .checked_add(required)
            .is_none_or(|events| events > self.config.attention.max_events)
        {
            return Err(RelationalAttentionError::EventBudgetExhausted);
        }
        Ok(())
    }

    pub fn run_episode(
        &mut self,
        episode: &RelationalEpisode,
    ) -> Result<RelationalRead, RelationalAttentionError> {
        self.validate_episode(episode)?;
        for fact in &episode.facts {
            self.bind(fact.relation, fact.subject, fact.object)?;
        }
        self.compose_path(&episode.query.relations, episode.query.subject)
    }
}

pub fn evaluate_relational_attention_episode(
    config: RelationalAttentionConfig,
    episode: &RelationalEpisode,
) -> Result<RelationalAttentionOutcome, RelationalAttentionError> {
    let mut reference = RelationalAttentionReference::new(config)?;
    let observed = reference.run_episode(episode)?;
    Ok(RelationalAttentionOutcome {
        observed,
        expected: episode.expected(),
        correct: observed == episode.expected(),
        mode: config.attention.mode,
        work: reference.work(),
        footprint: reference.footprint(),
    })
}
