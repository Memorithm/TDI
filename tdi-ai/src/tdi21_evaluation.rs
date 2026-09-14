//! Development-only, evaluator-owned scoring and competence controls.
//!
//! Candidate execution receives one `Event`, never the expected answer. The
//! independent oracle scans original event prefixes; its work is NOT candidate
//! work. No final population, attention baseline or learning claim lives here.
//!
//! ```
//! use tdi_ai::experimental::tdi21_evaluation::DevelopmentEpisode;
//! use tdi_ai::experimental::tdi21_stream::{Event, StepOutput};
//! let episode = DevelopmentEpisode::new(&[Event::Recall { key: 7 }], 8).unwrap();
//! let score = episode.score_outputs(&[Ok(StepOutput::Quiet)]).unwrap();
//! assert_eq!(score.accuracy_ratio(), Some((0, 1)));
//! assert!(!score.all_correct());
//! ```

use std::collections::BTreeMap;

use super::tdi21::{BooleanState, MemoryRead};
use super::tdi21_stream::{
    BooleanStream, Event, MAX_SLOTS, MemoryFootprint, MemoryMode, StepOutput, StreamConfig,
    StreamCounters, StreamError,
};

/// Version for the evaluator, not for the candidate's routing algorithm.
pub const EVALUATION_SEMANTICS: &str = "tdi21-development-evaluation-v1";
/// Independent bounds on the development oracle, not a scientific task grid.
pub const MAX_EPISODE_EVENTS: usize = 16_384;
/// Accepted writes, including overrides, are bounded before oracle execution.
pub const MAX_ACCEPTED_WRITES: usize = 4096;
/// Prefix inspections abort before excessive evaluator work is performed.
pub const MAX_ORACLE_PROBES: u64 = 1_000_000;

/// Disjoint categories. The first seven exhaust expected query outcomes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Outcome {
    ExactValue,
    CorrectAbsence,
    WrongValue,
    FalseHit,
    ForgottenValue,
    OmittedReply,
    QueryError,
    CorrectQuiet,
    SpuriousReply,
    QuietError,
}

/// Exhaustive score, counting expected queries rather than emitted replies.
/// Private counts preserve the partition invariant and checked event total.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ScoreCard {
    counts: [u64; 10],
    events: u64,
}

/// Validation errors never authorize or silently truncate a development run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvaluationError {
    EmptyEpisode,
    TooManyEvents,
    TooManyWrites,
    NoQueries,
    InvalidPayloadWidth,
    InvalidEvent { index: usize },
    OracleBudgetExceeded,
    AllocationFailed,
    OutputCountMismatch,
    CounterOverflow,
    InvalidSlotCount,
    InsufficientMemoryEnvelope,
    MemoryEnvelopeExceeded,
    PayloadWidthMismatch,
    EventEnvelopeExceeded,
    CandidateConstruction(StreamError),
    CandidateAccountingMismatch,
}

fn classify(expected: StepOutput, observed: Result<StepOutput, StreamError>) -> Outcome {
    match (expected, observed) {
        (StepOutput::Quiet, Ok(StepOutput::Quiet)) => Outcome::CorrectQuiet,
        (StepOutput::Quiet, Ok(StepOutput::Reply(_))) => Outcome::SpuriousReply,
        (StepOutput::Quiet, Err(_)) => Outcome::QuietError,
        (StepOutput::Reply(_), Err(_)) => Outcome::QueryError,
        (StepOutput::Reply(_), Ok(StepOutput::Quiet)) => Outcome::OmittedReply,
        (StepOutput::Reply(MemoryRead::Miss), Ok(StepOutput::Reply(MemoryRead::Miss))) => {
            Outcome::CorrectAbsence
        }
        (StepOutput::Reply(MemoryRead::Miss), Ok(StepOutput::Reply(MemoryRead::Hit(_)))) => {
            Outcome::FalseHit
        }
        (StepOutput::Reply(MemoryRead::Hit(_)), Ok(StepOutput::Reply(MemoryRead::Miss))) => {
            Outcome::ForgottenValue
        }
        (StepOutput::Reply(MemoryRead::Hit(a)), Ok(StepOutput::Reply(MemoryRead::Hit(b)))) => {
            if a == b {
                Outcome::ExactValue
            } else {
                Outcome::WrongValue
            }
        }
    }
}

impl ScoreCard {
    /// Add one evaluator expectation and one observed output, atomically.
    /// The expectation must come from the evaluator, not the scored mechanism.
    pub fn record(
        &mut self,
        expected: StepOutput,
        observed: Result<StepOutput, StreamError>,
    ) -> Result<(), EvaluationError> {
        let events = self
            .events
            .checked_add(1)
            .ok_or(EvaluationError::CounterOverflow)?;
        let index = classify(expected, observed) as usize;
        let count = self.counts[index]
            .checked_add(1)
            .ok_or(EvaluationError::CounterOverflow)?;
        self.counts[index] = count;
        self.events = events;
        Ok(())
    }

    /// Number of observations in a named disjoint category.
    #[must_use]
    pub fn count(&self, outcome: Outcome) -> u64 {
        self.counts[outcome as usize]
    }

    /// Number of scored input events, including non-query events.
    #[must_use]
    pub fn events(&self) -> u64 {
        self.events
    }

    /// All evaluator-expected queries, including omitted or failed replies.
    #[must_use]
    pub fn queries(&self) -> u64 {
        self.counts[..7].iter().sum()
    }

    /// Exact values plus correct absence reports. A stored zero is a value.
    #[must_use]
    pub fn correct_queries(&self) -> u64 {
        self.count(Outcome::ExactValue) + self.count(Outcome::CorrectAbsence)
    }

    /// Exact numerator/denominator; no queries returns None, not perfect accuracy.
    #[must_use]
    pub fn accuracy_ratio(&self) -> Option<(u64, u64)> {
        let queries = self.queries();
        (queries != 0).then_some((self.correct_queries(), queries))
    }

    /// Non-vacuous correctness, also rejecting spurious replies and quiet errors.
    #[must_use]
    pub fn all_correct(&self) -> bool {
        self.queries() > 0
            && self.correct_queries() == self.queries()
            && self.count(Outcome::SpuriousReply) == 0
            && self.count(Outcome::QuietError) == 0
    }
}

/// Immutable public development inputs and private evaluator expectations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DevelopmentEpisode {
    events: Vec<Event>,
    expected: Vec<StepOutput>,
    payload_bits: u8,
    oracle_probes: u64,
}

fn oracle_lookup(
    prefix: &[Event],
    key: u64,
    probes: &mut u64,
) -> Result<Option<u64>, EvaluationError> {
    for event in prefix.iter().rev() {
        if *probes == MAX_ORACLE_PROBES {
            return Err(EvaluationError::OracleBudgetExceeded);
        }
        *probes += 1;
        if let Event::Write {
            key: stored,
            payload,
            marker,
        } = *event
        {
            if stored == key && marker.bits() == 1 {
                return Ok(Some(payload.bits()));
            }
        }
    }
    Ok(None)
}

fn reply(value: Option<u64>) -> StepOutput {
    StepOutput::Reply(match value {
        Some(bits) => MemoryRead::Hit(BooleanState::from_bits(bits)),
        None => MemoryRead::Miss,
    })
}

impl DevelopmentEpisode {
    /// Validate a bounded public development sequence and compile its oracle.
    /// Prefix scanning is independent of the candidate's routes and storage.
    /// Malformed writes are rejected even when their marker inhibits storage.
    pub fn new(events: &[Event], payload_bits: u8) -> Result<Self, EvaluationError> {
        if events.is_empty() {
            return Err(EvaluationError::EmptyEpisode);
        }
        if events.len() > MAX_EPISODE_EVENTS {
            return Err(EvaluationError::TooManyEvents);
        }
        if payload_bits == 0 || payload_bits > 64 {
            return Err(EvaluationError::InvalidPayloadWidth);
        }
        let mut writes = 0;
        let mut queries = 0;
        for (index, event) in events.iter().enumerate() {
            match *event {
                Event::Write {
                    payload, marker, ..
                } => {
                    if marker.bits() > 3
                        || (payload_bits < 64 && payload.bits() >> payload_bits != 0)
                    {
                        return Err(EvaluationError::InvalidEvent { index });
                    }
                    if marker.bits() == 1 {
                        writes += 1;
                    }
                }
                Event::Recall { .. } | Event::Conjunction { .. } => queries += 1,
                Event::Ignore => {}
            }
        }
        if writes > MAX_ACCEPTED_WRITES {
            return Err(EvaluationError::TooManyWrites);
        }
        if queries == 0 {
            return Err(EvaluationError::NoQueries);
        }
        let mut owned = Vec::new();
        owned
            .try_reserve_exact(events.len())
            .map_err(|_| EvaluationError::AllocationFailed)?;
        owned.extend_from_slice(events);
        let mut expected = Vec::new();
        expected
            .try_reserve_exact(events.len())
            .map_err(|_| EvaluationError::AllocationFailed)?;
        let mut probes = 0;
        for (index, event) in events.iter().enumerate() {
            let prefix = &events[..index];
            let output = match *event {
                Event::Recall { key } => reply(oracle_lookup(prefix, key, &mut probes)?),
                Event::Conjunction { left, right } => {
                    let a = oracle_lookup(prefix, left, &mut probes)?;
                    let b = oracle_lookup(prefix, right, &mut probes)?;
                    reply(a.zip(b).map(|(a, b)| a & b))
                }
                _ => StepOutput::Quiet,
            };
            expected.push(output);
        }
        Ok(Self {
            events: owned,
            expected,
            payload_bits,
            oracle_probes: probes,
        })
    }

    /// Original public inputs only; no oracle answers are exposed here.
    #[must_use]
    pub fn events(&self) -> &[Event] {
        &self.events
    }

    /// Oracle prefix inspections; never add these to candidate work counters.
    #[must_use]
    pub fn oracle_probes(&self) -> u64 {
        self.oracle_probes
    }

    /// Score a completed trace. Truncation or extra outputs is an error.
    pub fn score_outputs(
        &self,
        outputs: &[Result<StepOutput, StreamError>],
    ) -> Result<ScoreCard, EvaluationError> {
        if outputs.len() != self.expected.len() {
            return Err(EvaluationError::OutputCountMismatch);
        }
        let mut score = ScoreCard::default();
        for (&expected, &observed) in self.expected.iter().zip(outputs) {
            score.record(expected, observed)?;
        }
        Ok(score)
    }
}

/// Cost of the existing memory substrate, NOT total architecture memory.
/// Full 64-bit identity and payload fields remain charged at every width.
pub fn substrate_bits(mode: MemoryMode, slots: usize) -> Result<usize, EvaluationError> {
    if slots == 0 || slots > MAX_SLOTS || (mode == MemoryMode::TwoWay && slots % 2 != 0) {
        return Err(EvaluationError::InvalidSlotCount);
    }
    Ok(slots * 129
        + if mode == MemoryMode::TwoWay {
            slots / 2
        } else {
            0
        })
}

/// Maximum entries under a COMMON memory-substrate ceiling, charging B3 metadata.
/// Clamped by the existing runtime's allocation bound. This is not equal total
/// CPU/training/allocator budget or a statistical comparison authorization.
///
/// ```
/// use tdi_ai::experimental::tdi21_evaluation::fit_slots;
/// use tdi_ai::experimental::tdi21_stream::MemoryMode;
/// assert_eq!(fit_slots(MemoryMode::Direct, 516), Ok(4));
/// assert_eq!(fit_slots(MemoryMode::TwoWay, 516), Ok(2));
/// ```
pub fn fit_slots(mode: MemoryMode, ceiling_bits: usize) -> Result<usize, EvaluationError> {
    let slots = match mode {
        MemoryMode::Direct => (ceiling_bits / 129).min(MAX_SLOTS),
        // Two entries and one round-robin replacement bit per bucket.
        MemoryMode::TwoWay => (ceiling_bits / 259).min(MAX_SLOTS / 2) * 2,
    };
    if slots == 0 {
        return Err(EvaluationError::InsufficientMemoryEnvelope);
    }
    Ok(slots)
}

/// A completed non-final run with declared budget and actual native counters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BooleanReport {
    pub config: StreamConfig,
    pub ceiling_bits: usize,
    pub score: ScoreCard,
    pub counters: StreamCounters,
    pub footprint: MemoryFootprint,
}

/// Run a fresh B2/B3 instance. Both modes can use the SAME episode and ceiling.
/// The candidate sees only the current event. Expected answers stay here.
pub fn evaluate_boolean(
    episode: &DevelopmentEpisode,
    config: StreamConfig,
    ceiling_bits: usize,
) -> Result<BooleanReport, EvaluationError> {
    let required = substrate_bits(config.mode, config.slots)?;
    if required > ceiling_bits {
        return Err(EvaluationError::MemoryEnvelopeExceeded);
    }
    if config.payload_bits != episode.payload_bits {
        return Err(EvaluationError::PayloadWidthMismatch);
    }
    if config.max_events < episode.events.len() as u64 {
        return Err(EvaluationError::EventEnvelopeExceeded);
    }
    let mut machine = BooleanStream::new(config).map_err(EvaluationError::CandidateConstruction)?;
    let mut score = ScoreCard::default();
    for (&event, &expected) in episode.events.iter().zip(&episode.expected) {
        score.record(expected, machine.step(event))?;
    }
    let counters = machine.counters();
    let footprint = machine.footprint();
    if footprint.entry_semantic_bits + footprint.replacement_semantic_bits != required
        || !counters.work.candidate_is_pairwise_free()
    {
        return Err(EvaluationError::CandidateAccountingMismatch);
    }
    Ok(BooleanReport {
        config,
        ceiling_bits,
        score,
        counters,
        footprint,
    })
}

/// Task-competence controls, deliberately outside the B0-B5 architecture ladder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlArm {
    NoMemory,
    ExactDictionary,
}

/// Native logical map operations, not tree-comparison counts or FLOPs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ControlWork {
    pub map_reads: u64,
    pub map_writes: u64,
    pub conjunctions: u64,
    pub stored_facts: usize,
}

/// The dictionary is NOT matched to the candidate's memory ceiling. Its entry
/// payload lower bound excludes tree nodes, links, allocator and object state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControlReport {
    pub arm: ControlArm,
    pub score: ScoreCard,
    pub work: ControlWork,
    pub entry_payload_bits_lower_bound: usize,
}

/// Independent control execution over exactly the same input events.
/// An ordered dictionary uses original identifiers, not the Boolean route.
/// The bounded episode limits its possible entries to MAX_ACCEPTED_WRITES.
/// No expected answer is passed into the control's step implementation.
pub fn evaluate_control(
    episode: &DevelopmentEpisode,
    arm: ControlArm,
) -> Result<ControlReport, EvaluationError> {
    let mut facts = BTreeMap::new();
    let mut work = ControlWork::default();
    let mut score = ScoreCard::default();
    for (&event, &expected) in episode.events.iter().zip(&episode.expected) {
        let observed = control_step(event, arm, &mut facts, &mut work);
        score.record(expected, Ok(observed))?;
    }
    work.stored_facts = facts.len();
    Ok(ControlReport {
        arm,
        score,
        work,
        entry_payload_bits_lower_bound: work.stored_facts * 128,
    })
}

fn control_step(
    event: Event,
    arm: ControlArm,
    facts: &mut BTreeMap<u64, u64>,
    work: &mut ControlWork,
) -> StepOutput {
    if arm == ControlArm::NoMemory {
        return match event {
            Event::Recall { .. } | Event::Conjunction { .. } => reply(None),
            _ => StepOutput::Quiet,
        };
    }
    match event {
        Event::Write {
            key,
            payload,
            marker,
        } => {
            if marker.bits() == 1 {
                work.map_writes += 1;
                facts.insert(key, payload.bits());
            }
            StepOutput::Quiet
        }
        Event::Recall { key } => {
            work.map_reads += 1;
            reply(facts.get(&key).copied())
        }
        Event::Conjunction { left, right } => {
            work.map_reads += 2;
            let value = facts.get(&left).zip(facts.get(&right)).map(|(a, b)| {
                work.conjunctions += 1;
                a & b
            });
            reply(value)
        }
        Event::Ignore => StepOutput::Quiet,
    }
}
