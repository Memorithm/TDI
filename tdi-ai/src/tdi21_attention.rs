//! Isolated B0/B1 attention references for public TDI-21 development tasks.
//!
//! These are hand-constructed identity/recency controls, NOT trained
//! Transformers, FlashAttention kernels or evidence of model superiority.
//! B0 uses a real f64 dot product; B1 uses equivalent packed bipolar Q/K.
//! Both retain f64 values, normalized softmax and weighted value aggregation.
//! No Boolean candidate imports this module or calls these references.
//!
//! ```
//! use tdi_ai::experimental::tdi21::{BooleanState, MemoryRead};
//! use tdi_ai::experimental::tdi21_attention::{AttentionConfig, AttentionMode, AttentionReference};
//! use tdi_ai::experimental::tdi21_stream::{Event, StepOutput};
//! let mut reference = AttentionReference::new(AttentionConfig {
//!     mode: AttentionMode::DenseQk, history_capacity: 4,
//!     payload_bits: 8, max_events: 8,
//! }).unwrap();
//! reference.step(Event::Write {
//!     key: 7, payload: BooleanState::from_bits(0),
//!     marker: BooleanState::from_bits(1),
//! }).unwrap();
//! assert_eq!(reference.step(Event::Recall { key: 7 }).unwrap(),
//!     StepOutput::Reply(MemoryRead::Hit(BooleanState::from_bits(0))));
//! assert_eq!(reference.work().pairwise_scores, 1);
//! reference.reset();
//! assert_eq!(reference.stored_events(), 0);
//! ```

use super::tdi21::{ArchitectureArm, BooleanState, MemoryRead};
use super::tdi21_stream::{Event, StepOutput};

/// Versioned, public development semantics; no scientific freeze is implied.
pub const ATTENTION_SEMANTICS: &str = "tdi21-identity-recency-softmax-v1";
/// Explicit safety bound on admitted write history, including overrides.
pub const MAX_REFERENCE_HISTORY: usize = 256;
/// Independent safety bound on accepted events per reference instance.
pub const MAX_REFERENCE_EVENTS: u64 = 16_384;
const WIDTH: usize = 64;
const SHARPNESS: f64 = 16.0;

/// Two reference implementations with identical mathematical Q/K semantics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttentionMode {
    DenseQk,
    BinaryQk,
}

impl AttentionMode {
    /// References remain outside the strict Boolean candidate family.
    #[must_use]
    pub const fn arm(self) -> ArchitectureArm {
        match self {
            Self::DenseQk => ArchitectureArm::B0AttentionReference,
            Self::BinaryQk => ArchitectureArm::B1BinaryAttentionReference,
        }
    }
}

/// Declared bounds. Capacity is append-only writes, not unique dictionary keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttentionConfig {
    pub mode: AttentionMode,
    pub history_capacity: usize,
    pub payload_bits: u8,
    pub max_events: u64,
}

/// Errors are explicit; there is no eviction, silent truncation or fallback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttentionError {
    InvalidCapacity,
    InvalidWidth,
    InvalidEventBudget,
    AllocationFailed,
    EventBudgetExhausted,
    HistoryCapacityExhausted,
    InvalidMarker,
    PayloadOutOfRange,
    NonFiniteArithmetic,
}

/// Native semantic work categories, NOT CPU instructions or a FLOP-equivalent.
/// A dot/value term denotes one scalar product accumulated into a sum.
/// Logit construction, normalization sums, validation and index arithmetic
/// are not disguised as measured hardware work. All increments are checked.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AttentionWork {
    pub events: u64,
    pub writes: u64,
    pub inhibited_writes: u64,
    pub lookups: u64,
    pub pairwise_scores: u64,
    pub float_dot_terms: u64,
    pub xor_popcount_words: u64,
    pub exponentials: u64,
    pub normalizations: u64,
    pub weighted_value_terms: u64,
    pub key_component_encodes: u64,
    pub value_component_encodes: u64,
    pub presence_terms: u64,
    pub readout_comparisons: u64,
    pub conjunctions: u64,
}

/// Component payload reservations and actual Rust object/buffer sizes.
/// These exclude allocator bookkeeping, temporary query/readout arrays,
/// executable code, evaluator state and process memory. Not a matched budget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttentionFootprint {
    pub history_component_bits: usize,
    pub score_component_bits: usize,
    pub reserved_buffer_bytes: usize,
    pub inline_bytes: usize,
}

#[derive(Clone, Debug, PartialEq)]
struct DenseEntry {
    key: [f64; WIDTH],
    value: [f64; WIDTH],
}

#[derive(Clone, Debug, PartialEq)]
struct BinaryEntry {
    key: u64,
    value: [f64; WIDTH],
}

#[derive(Clone, Debug, PartialEq)]
enum History {
    Dense(Vec<DenseEntry>),
    Binary(Vec<BinaryEntry>),
}

/// A closed causal reference. Stores only admitted past writes; step receives
/// neither oracle answers nor a future stream. QK scores are computed for ALL
/// stored writes, including stale overrides. Null is an explicit extra lane.
#[derive(Clone, Debug, PartialEq)]
pub struct AttentionReference {
    config: AttentionConfig,
    history: History,
    scores: Vec<f64>,
    last_row_len: usize,
    work: AttentionWork,
}

fn charge(counter: &mut u64, amount: u64) {
    *counter = counter
        .checked_add(amount)
        .expect("TDI-21 attention counter overflow");
}

fn encode(word: u64) -> [f64; WIDTH] {
    std::array::from_fn(|bit| if word & (1_u64 << bit) == 0 { -1.0 } else { 1.0 })
}

fn reserved<T>(capacity: usize) -> Result<Vec<T>, AttentionError> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(capacity)
        .map_err(|_| AttentionError::AllocationFailed)?;
    Ok(values)
}

fn normalize(row: &mut [f64], work: &mut AttentionWork) -> Result<(), AttentionError> {
    if row.is_empty() || row.iter().any(|value| !value.is_finite()) {
        return Err(AttentionError::NonFiniteArithmetic);
    }
    let maximum = row.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let mut denominator = 0.0;
    for value in row.iter_mut() {
        *value = (*value - maximum).exp();
        denominator += *value;
        charge(&mut work.exponentials, 1);
    }
    if !denominator.is_finite() || denominator <= 0.0 {
        return Err(AttentionError::NonFiniteArithmetic);
    }
    for value in row {
        *value /= denominator;
        charge(&mut work.normalizations, 1);
    }
    Ok(())
}

impl AttentionReference {
    /// Validate before fallible allocation. No allocation occurs during step.
    pub fn new(config: AttentionConfig) -> Result<Self, AttentionError> {
        if config.history_capacity == 0 || config.history_capacity > MAX_REFERENCE_HISTORY {
            return Err(AttentionError::InvalidCapacity);
        }
        if config.payload_bits == 0 || config.payload_bits > 64 {
            return Err(AttentionError::InvalidWidth);
        }
        if config.max_events == 0 || config.max_events > MAX_REFERENCE_EVENTS {
            return Err(AttentionError::InvalidEventBudget);
        }
        let history = match config.mode {
            AttentionMode::DenseQk => History::Dense(reserved(config.history_capacity)?),
            AttentionMode::BinaryQk => History::Binary(reserved(config.history_capacity)?),
        };
        let mut scores = reserved(config.history_capacity + 1)?;
        scores.resize(config.history_capacity + 1, 0.0);
        Ok(Self {
            config,
            history,
            scores,
            last_row_len: 0,
            work: AttentionWork::default(),
        })
    }

    /// Configuration used by this instance, including the reference arm.
    #[must_use]
    pub fn config(&self) -> AttentionConfig {
        self.config
    }

    /// Number of accepted writes retained, including overwritten identifiers.
    #[must_use]
    pub fn stored_events(&self) -> usize {
        match &self.history {
            History::Dense(entries) => entries.len(),
            History::Binary(entries) => entries.len(),
        }
    }

    /// Observed native counters for successful steps in the current episode.
    #[must_use]
    pub fn work(&self) -> AttentionWork {
        self.work
    }

    /// Last normalized row, with null at index zero. Empty before any lookup.
    /// After a conjunction, this is the second operand's row only.
    #[must_use]
    pub fn last_weights(&self) -> &[f64] {
        &self.scores[..self.last_row_len]
    }

    /// Clear history, counters and diagnostics without reallocating buffers.
    /// Reset/initialization cost is separate from per-event work counters.
    pub fn reset(&mut self) {
        match &mut self.history {
            History::Dense(entries) => entries.clear(),
            History::Binary(entries) => entries.clear(),
        }
        self.scores.fill(0.0);
        self.last_row_len = 0;
        self.work = AttentionWork::default();
    }

    /// Report reserved representation components, not total process memory.
    #[must_use]
    pub fn footprint(&self) -> AttentionFootprint {
        let (entry_bits, bytes) = match &self.history {
            History::Dense(entries) => (8192, entries.capacity() * std::mem::size_of::<DenseEntry>()),
            History::Binary(entries) => (4160, entries.capacity() * std::mem::size_of::<BinaryEntry>()),
        };
        AttentionFootprint {
            history_component_bits: self.config.history_capacity * entry_bits,
            score_component_bits: (self.config.history_capacity + 1) * 64,
            reserved_buffer_bytes: bytes + self.scores.capacity() * std::mem::size_of::<f64>(),
            inline_bytes: std::mem::size_of::<Self>(),
        }
    }

    /// Process one event. Input/capacity/budget errors leave all state intact.
    /// A numerical error aborts the step; discard its diagnostic row and reset.
    /// Such an error MUST reject evaluation, never reduce its query denominator.
    pub fn step(&mut self, event: Event) -> Result<StepOutput, AttentionError> {
        if self.work.events >= self.config.max_events {
            return Err(AttentionError::EventBudgetExhausted);
        }
        if let Event::Write { payload, marker, .. } = event {
            if marker.bits() > 3 {
                return Err(AttentionError::InvalidMarker);
            }
            if self.config.payload_bits < 64 && payload.bits() >> self.config.payload_bits != 0 {
                return Err(AttentionError::PayloadOutOfRange);
            }
            if marker.bits() == 1 && self.stored_events() == self.config.history_capacity {
                return Err(AttentionError::HistoryCapacityExhausted);
            }
        }
        let mut work = self.work;
        let output = match event {
            Event::Write { key, payload, marker } => {
                if marker.bits() == 1 {
                    let value = encode(payload.bits());
                    match &mut self.history {
                        History::Dense(entries) => {
                            entries.push(DenseEntry { key: encode(key), value });
                            charge(&mut work.key_component_encodes, WIDTH as u64);
                        }
                        History::Binary(entries) => entries.push(BinaryEntry { key, value }),
                    }
                    charge(&mut work.value_component_encodes, WIDTH as u64);
                    charge(&mut work.writes, 1);
                } else {
                    charge(&mut work.inhibited_writes, 1);
                }
                StepOutput::Quiet
            }
            Event::Recall { key } => StepOutput::Reply(self.lookup(key, &mut work)?),
            Event::Conjunction { left, right } => {
                let a = self.lookup(left, &mut work)?;
                let b = self.lookup(right, &mut work)?;
                let result = match (a, b) {
                    (MemoryRead::Hit(a), MemoryRead::Hit(b)) => {
                        charge(&mut work.conjunctions, 1);
                        MemoryRead::Hit(BooleanState::from_bits(a.bits() & b.bits()))
                    }
                    _ => MemoryRead::Miss,
                };
                StepOutput::Reply(result)
            }
            Event::Ignore => StepOutput::Quiet,
        };
        charge(&mut work.events, 1);
        self.work = work;
        Ok(output)
    }

    fn lookup(&mut self, key: u64, work: &mut AttentionWork) -> Result<MemoryRead, AttentionError> {
        let count = self.stored_events();
        self.scores[0] = 0.0; // Null lane: no value, no presence.
        let capacity = self.config.history_capacity as f64;
        let logit = |dot: f64, index: usize| {
            SHARPNESS * ((dot - 64.0) * (capacity + 1.0) + (index + 1) as f64)
        };
        match &self.history {
            History::Dense(entries) => {
                let query = encode(key);
                charge(&mut work.key_component_encodes, WIDTH as u64);
                for (index, entry) in entries.iter().enumerate() {
                    let dot = query.iter().zip(&entry.key).map(|(q, k)| q * k).sum();
                    self.scores[index + 1] = logit(dot, index);
                    charge(&mut work.float_dot_terms, WIDTH as u64);
                }
            }
            History::Binary(entries) => {
                for (index, entry) in entries.iter().enumerate() {
                    let dot = 64.0 - 2.0 * f64::from((key ^ entry.key).count_ones());
                    self.scores[index + 1] = logit(dot, index);
                    charge(&mut work.xor_popcount_words, 1);
                }
            }
        }
        charge(&mut work.lookups, 1);
        charge(&mut work.pairwise_scores, count as u64);
        normalize(&mut self.scores[..=count], work)?;
        self.last_row_len = count + 1;
        let mut values = [0.0; WIDTH];
        let mut presence = 0.0;
        for index in 0..count {
            let value = match &self.history {
                History::Dense(entries) => &entries[index].value,
                History::Binary(entries) => &entries[index].value,
            };
            let weight = self.scores[index + 1];
            presence += weight;
            for bit in 0..usize::from(self.config.payload_bits) {
                values[bit] += weight * value[bit];
                charge(&mut work.weighted_value_terms, 1);
            }
            charge(&mut work.presence_terms, 1);
        }
        if !presence.is_finite() || values.iter().any(|value| !value.is_finite()) {
            return Err(AttentionError::NonFiniteArithmetic);
        }
        charge(&mut work.readout_comparisons, 1);
        if presence <= 0.5 {
            return Ok(MemoryRead::Miss);
        }
        let mut bits = 0_u64;
        for (bit, &value) in values.iter().enumerate().take(usize::from(self.config.payload_bits)) {
            charge(&mut work.readout_comparisons, 1);
            if value > 0.0 {
                bits |= 1_u64 << bit;
            }
        }
        Ok(MemoryRead::Hit(BooleanState::from_bits(bits)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalization_rejects_nonfinite_before_mutation() {
        for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut row = [1.0, invalid];
            let mut work = AttentionWork::default();
            assert_eq!(normalize(&mut row, &mut work), Err(AttentionError::NonFiniteArithmetic));
            assert_eq!(row[0], 1.0);
            assert_eq!(work, AttentionWork::default());
        }
    }

    #[test]
    fn normalization_is_stable_for_large_logits() {
        let mut row = [1_000_000.0, 1_000_000.0, -1_000_000.0];
        let mut work = AttentionWork::default();
        normalize(&mut row, &mut work).unwrap();
        assert_eq!(row, [0.5, 0.5, 0.0]);
        assert_eq!(work.exponentials, 3);
        assert_eq!(work.normalizations, 3);
    }
}
