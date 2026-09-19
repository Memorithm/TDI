//! Development-only relational binding/retrieval prototype for TDI-21.
//!
//! The prototype deliberately reuses bounded BooleanStream memory rather than
//! token-pair similarity. Relation/subject pairs can be addressed either by
//! exact symbolic packing or by an exactly equivalent ANF/Zhegalkin identity
//! encoder. This is a task/substitution harness, not learned generalization.

use super::tdi21::{BooleanState, MemoryRead, ResourceCounters};
use super::tdi21_anf_synthesis::{
    AnfProgram, AnfSynthesisError, MAX_ANF_VARIABLES, synthesize_anf,
};
use super::tdi21_stream::{
    BooleanStream, Event, StepOutput, StreamConfig, StreamCounters, StreamError,
};

pub const RELATIONAL_BINDING_SEMANTICS: &str = "tdi21-relational-binding-v1";
pub const RELATIONAL_ANF_ADDRESS_SEMANTICS: &str = "tdi21-relational-anf-identity-v1";
pub const MAX_COMPOSITION_HOPS: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RelationalConfig {
    pub stream: StreamConfig,
    pub entity_bits: u8,
    pub relation_bits: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelationalAddressMode {
    ExactPacked,
    AnfIdentity,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RelationalWork {
    /// One semantic derivation of the `(relation, subject)` identifier.
    /// This is not a CPU-instruction or cycle count.
    pub address_derivations: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelationalError {
    InvalidEntityWidth,
    InvalidRelationWidth,
    AddressWidthOverflow,
    TooManyAnfAddressVariables,
    EntityOutOfRange,
    RelationOutOfRange,
    EmptyRelationPath,
    TooManyCompositionHops,
    EventBudgetExhausted,
    CounterOverflow,
    UnexpectedQuiet,
    Anf(AnfSynthesisError),
    Stream(StreamError),
}

impl From<StreamError> for RelationalError {
    fn from(value: StreamError) -> Self {
        Self::Stream(value)
    }
}

impl From<AnfSynthesisError> for RelationalError {
    fn from(value: AnfSynthesisError) -> Self {
        Self::Anf(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelationalRead {
    Hit(u64),
    Miss,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Addressing {
    ExactPacked,
    AnfIdentity(Vec<AnfProgram>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationalBinder {
    config: RelationalConfig,
    stream: BooleanStream,
    addressing: Addressing,
    work: RelationalWork,
    anf_work: ResourceCounters,
}

fn identity_anf_programs(variable_count: u8) -> Result<Vec<AnfProgram>, RelationalError> {
    if variable_count > MAX_ANF_VARIABLES {
        return Err(RelationalError::TooManyAnfAddressVariables);
    }
    let rows = 1usize << variable_count;
    let mut programs = Vec::new();
    programs
        .try_reserve_exact(variable_count as usize)
        .map_err(|_| AnfSynthesisError::AllocationFailed)?;
    for bit in 0..variable_count {
        let mut truth_table = Vec::new();
        truth_table
            .try_reserve_exact(rows)
            .map_err(|_| AnfSynthesisError::AllocationFailed)?;
        for assignment in 0..rows {
            truth_table.push(((assignment >> bit) & 1) != 0);
        }
        programs.push(synthesize_anf(variable_count, &truth_table)?);
    }
    Ok(programs)
}

impl RelationalBinder {
    fn validate_config(config: RelationalConfig) -> Result<u8, RelationalError> {
        if config.entity_bits == 0 || config.entity_bits > 32 {
            return Err(RelationalError::InvalidEntityWidth);
        }
        if config.relation_bits == 0 || config.relation_bits > 16 {
            return Err(RelationalError::InvalidRelationWidth);
        }
        let address_bits = config
            .entity_bits
            .checked_add(config.relation_bits)
            .ok_or(RelationalError::AddressWidthOverflow)?;
        if address_bits > 63 {
            return Err(RelationalError::AddressWidthOverflow);
        }
        if config.stream.payload_bits < config.entity_bits {
            return Err(RelationalError::InvalidEntityWidth);
        }
        Ok(address_bits)
    }

    pub fn new(config: RelationalConfig) -> Result<Self, RelationalError> {
        Self::new_with_mode(config, RelationalAddressMode::ExactPacked)
    }

    pub fn new_anf_identity(config: RelationalConfig) -> Result<Self, RelationalError> {
        Self::new_with_mode(config, RelationalAddressMode::AnfIdentity)
    }

    fn new_with_mode(
        config: RelationalConfig,
        mode: RelationalAddressMode,
    ) -> Result<Self, RelationalError> {
        let address_bits = Self::validate_config(config)?;
        let addressing = match mode {
            RelationalAddressMode::ExactPacked => Addressing::ExactPacked,
            RelationalAddressMode::AnfIdentity => {
                Addressing::AnfIdentity(identity_anf_programs(address_bits)?)
            }
        };
        Ok(Self {
            config,
            stream: BooleanStream::new(config.stream)?,
            addressing,
            work: RelationalWork::default(),
            anf_work: ResourceCounters::default(),
        })
    }

    #[must_use]
    pub const fn address_mode(&self) -> RelationalAddressMode {
        match &self.addressing {
            Addressing::ExactPacked => RelationalAddressMode::ExactPacked,
            Addressing::AnfIdentity(_) => RelationalAddressMode::AnfIdentity,
        }
    }

    #[must_use]
    pub fn counters(&self) -> StreamCounters {
        self.stream.counters()
    }

    #[must_use]
    pub const fn relational_work(&self) -> RelationalWork {
        self.work
    }

    #[must_use]
    pub fn anf_work(&self) -> ResourceCounters {
        self.anf_work
    }

    pub fn reset(&mut self) {
        self.stream.reset();
        self.work = RelationalWork::default();
        self.anf_work = ResourceCounters::default();
    }

    fn entity_limit(&self) -> u64 {
        1u64 << self.config.entity_bits
    }

    fn relation_limit(&self) -> u64 {
        1u64 << self.config.relation_bits
    }

    fn assignment(&self, relation: u64, subject: u64) -> Result<u64, RelationalError> {
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

    fn staged_address(
        &self,
        relation: u64,
        subject: u64,
    ) -> Result<(u64, u64, ResourceCounters), RelationalError> {
        let assignment = self.assignment(relation, subject)?;
        let next_address_count = self.next_address_count()?;
        let mut staged_anf_work = self.anf_work;
        let key = match &self.addressing {
            Addressing::ExactPacked => assignment,
            Addressing::AnfIdentity(programs) => {
                let mut encoded = 0u64;
                for (bit, program) in programs.iter().enumerate() {
                    if program.evaluate_counted(assignment, &mut staged_anf_work)? {
                        encoded |= 1u64 << bit;
                    }
                }
                encoded
            }
        };
        Ok((key, next_address_count, staged_anf_work))
    }

    fn commit_address_work(&mut self, next_address_count: u64, anf_work: ResourceCounters) {
        self.work.address_derivations = next_address_count;
        self.anf_work = anf_work;
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
        let (key, next_address_count, staged_anf_work) = self.staged_address(relation, subject)?;
        self.stream.step(Event::Write {
            key,
            payload: BooleanState::from_bits(object),
            marker: BooleanState::from_bits(1),
        })?;
        self.commit_address_work(next_address_count, staged_anf_work);
        Ok(())
    }

    pub fn recall(
        &mut self,
        relation: u64,
        subject: u64,
    ) -> Result<RelationalRead, RelationalError> {
        let (key, next_address_count, staged_anf_work) = self.staged_address(relation, subject)?;
        let output = self.stream.step(Event::Recall { key })?;
        self.commit_address_work(next_address_count, staged_anf_work);
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
