//! Deterministic symbolic T1/T2/T3 task generators for TDI-8.1.
//!
//! The generators intentionally stop before architecture-specific vector
//! encoding. A generated symbolic instance is therefore identical across A0,
//! A1, A2 and A3. Concrete horizon values, seed ranges and validation/final
//! populations remain external inputs and are not frozen by this module.

use core::{fmt, mem};

const SPLITMIX_GAMMA: u64 = 0x9e37_79b9_7f4a_7c15;
const DOMAIN_T1_KEY: u64 = 0x7431_2d6b_6579_0001;
const DOMAIN_T1_VALUE: u64 = 0x7431_2d76_616c_0001;
const DOMAIN_T1_DISTRACTOR: u64 = 0x7431_2d64_6973_0001;
const DOMAIN_T1_QUERY: u64 = 0x7431_2d71_7565_0001;
const DOMAIN_T2_PAYLOAD: u64 = 0x7432_2d70_6179_0001;
const DOMAIN_T2_DISTRACTOR: u64 = 0x7432_2d64_6973_0001;
const DOMAIN_T3_PREFIX: u64 = 0x7433_2d70_7265_0001;
const DOMAIN_T3_SUFFIX_STRIDE: u64 = 0x7433_2d73_7472_0001;
const DOMAIN_T3_SUFFIX_OFFSET: u64 = 0x7433_2d6f_6666_0001;
const DOMAIN_T3_VALUE: u64 = 0x7433_2d76_616c_0001;
const DOMAIN_T3_DISTRACTOR: u64 = 0x7433_2d64_6973_0001;

/// Frozen task-family vocabulary from the TDI-8.0 preregistration.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TaskFamily {
    /// T1: key/value associations followed by delayed keyed queries.
    AssociativeRecall,
    /// T2: bounded payload copied after a delay containing irrelevant inputs.
    DelayedCopy,
    /// T3: associative recall under controlled similarity/interference pressure.
    InterferenceRecall,
}

/// Frozen short/medium/long horizon-stratum vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HorizonStratum {
    /// Short primary horizon.
    Short,
    /// Medium primary horizon.
    Medium,
    /// Long primary horizon.
    Long,
}

/// Explicit horizon values supplied by later bounded development work.
///
/// This type validates ordering only. It deliberately provides no default or
/// repository-chosen concrete horizon values.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HorizonPlan {
    short: u64,
    medium: u64,
    long: u64,
}

impl HorizonPlan {
    /// Require three positive, strictly increasing horizon values.
    pub fn new(short: u64, medium: u64, long: u64) -> Result<Self, TaskGeneratorError> {
        if short == 0 || medium == 0 || long == 0 {
            return Err(TaskGeneratorError::ZeroHorizon);
        }
        if !(short < medium && medium < long) {
            return Err(TaskGeneratorError::NonIncreasingHorizons {
                short,
                medium,
                long,
            });
        }
        Ok(Self {
            short,
            medium,
            long,
        })
    }

    /// Resolve one frozen stratum label to its externally supplied horizon.
    #[must_use]
    pub const fn value(self, stratum: HorizonStratum) -> u64 {
        match stratum {
            HorizonStratum::Short => self.short,
            HorizonStratum::Medium => self.medium,
            HorizonStratum::Long => self.long,
        }
    }

    /// Externally supplied short horizon.
    #[must_use]
    pub const fn short(self) -> u64 {
        self.short
    }

    /// Externally supplied medium horizon.
    #[must_use]
    pub const fn medium(self) -> u64 {
        self.medium
    }

    /// Externally supplied long horizon.
    #[must_use]
    pub const fn long(self) -> u64 {
        self.long
    }
}

/// Architecture-independent discrete symbol generated before execution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TaskSymbol(u64);

impl TaskSymbol {
    /// Construct an exact symbolic value from a stable integer code.
    #[must_use]
    pub const fn new(code: u64) -> Self {
        Self(code)
    }

    /// Stable exact code consumed later by a frozen arm adapter.
    #[must_use]
    pub const fn code(self) -> u64 {
        self.0
    }
}

/// Symbolic associative key with generator-side interference metadata.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TaskKey {
    code: u64,
    collision_class: u64,
}

impl TaskKey {
    /// Construct a symbolic key and its generator-side collision class.
    #[must_use]
    pub const fn new(code: u64, collision_class: u64) -> Self {
        Self {
            code,
            collision_class,
        }
    }

    /// Stable exact key code.
    #[must_use]
    pub const fn code(self) -> u64 {
        self.code
    }

    /// Generator-side interference/collision class.
    ///
    /// This label is not evidence that a concrete A2/A3 associative projection
    /// maps the key to a particular physical slot. That must be verified later
    /// by the frozen arm adapter/evaluator.
    #[must_use]
    pub const fn collision_class(self) -> u64 {
        self.collision_class
    }
}

/// One architecture-independent event in a generated task instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TaskEvent {
    /// Introduce one key/value association.
    Associate {
        /// Symbolic key.
        key: TaskKey,
        /// Symbolic target value.
        value: TaskSymbol,
        /// Zero-based insertion/source index within the association set.
        source_index: u64,
    },
    /// Introduce one ordered payload item for T2.
    Payload {
        /// Zero-based payload position.
        position: u64,
        /// Symbolic payload value.
        value: TaskSymbol,
    },
    /// Irrelevant input inserted to create delay/stress.
    Distractor {
        /// Symbolic distractor token.
        token: TaskSymbol,
    },
    /// Query a previously introduced association.
    QueryAssociation {
        /// Query key generated before execution.
        key: TaskKey,
        /// Exact target value generated before execution.
        target: TaskSymbol,
        /// Source association index used to distinguish recent/old recall.
        source_index: u64,
    },
    /// Request reproduction of one T2 payload position.
    QueryPayload {
        /// Original payload position.
        position: u64,
        /// Exact target value generated before execution.
        target: TaskSymbol,
    },
}

/// One complete symbolic task instance shared bit-for-bit across arms.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskInstance {
    family: TaskFamily,
    seed: u64,
    events: Vec<TaskEvent>,
    query_count: u64,
}

impl TaskInstance {
    /// Frozen task family.
    #[must_use]
    pub const fn family(&self) -> TaskFamily {
        self.family
    }

    /// Exact generator seed supplied by the caller.
    #[must_use]
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    /// Complete ordered symbolic event sequence.
    #[must_use]
    pub fn events(&self) -> &[TaskEvent] {
        &self.events
    }

    /// Number of exact target queries in the sequence.
    #[must_use]
    pub const fn query_count(&self) -> u64 {
        self.query_count
    }

    /// Total number of generated symbolic events.
    #[must_use]
    pub fn event_count(&self) -> usize {
        self.events.len()
    }
}

/// Explicit T1 configuration. Concrete values remain caller supplied.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct T1Config {
    association_count: u64,
    delay_steps: u64,
    query_count: u64,
}

impl T1Config {
    /// Construct a T1 configuration with at least one unqueried distractor
    /// association and at least one delayed query.
    pub fn new(
        association_count: u64,
        delay_steps: u64,
        query_count: u64,
    ) -> Result<Self, TaskGeneratorError> {
        if association_count < 2 {
            return Err(TaskGeneratorError::T1NeedsDistractorAssociation);
        }
        if delay_steps == 0 {
            return Err(TaskGeneratorError::ZeroDelay {
                family: TaskFamily::AssociativeRecall,
            });
        }
        if query_count == 0 || query_count >= association_count {
            return Err(TaskGeneratorError::InvalidQueryCount {
                family: TaskFamily::AssociativeRecall,
                query_count,
                association_count,
            });
        }
        Ok(Self {
            association_count,
            delay_steps,
            query_count,
        })
    }

    /// Number of associations introduced before delay/query.
    #[must_use]
    pub const fn association_count(self) -> u64 {
        self.association_count
    }

    /// Number of irrelevant delay events.
    #[must_use]
    pub const fn delay_steps(self) -> u64 {
        self.delay_steps
    }

    /// Number of delayed keyed queries.
    #[must_use]
    pub const fn query_count(self) -> u64 {
        self.query_count
    }
}

/// Explicit T2 configuration. Concrete values remain caller supplied.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct T2Config {
    payload_len: u64,
    delay_steps: u64,
}

impl T2Config {
    /// Construct a non-empty bounded payload and a strictly positive delay.
    pub fn new(payload_len: u64, delay_steps: u64) -> Result<Self, TaskGeneratorError> {
        if payload_len == 0 {
            return Err(TaskGeneratorError::ZeroPayloadLength);
        }
        if delay_steps == 0 {
            return Err(TaskGeneratorError::ZeroDelay {
                family: TaskFamily::DelayedCopy,
            });
        }
        Ok(Self {
            payload_len,
            delay_steps,
        })
    }

    /// Number of ordered payload symbols to reproduce.
    #[must_use]
    pub const fn payload_len(self) -> u64 {
        self.payload_len
    }

    /// Number of irrelevant delay events.
    #[must_use]
    pub const fn delay_steps(self) -> u64 {
        self.delay_steps
    }
}

/// Explicit T3 configuration. Concrete values remain caller supplied.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct T3Config {
    association_count: u64,
    delay_steps: u64,
    query_count: u64,
    shared_prefix_bits: u8,
    collision_classes: u64,
}

impl T3Config {
    /// Construct controlled similarity/interference pressure.
    ///
    /// `shared_prefix_bits` must be in `1..=63`, leaving a finite suffix space
    /// for distinct exact key codes. `collision_classes` is smaller than the
    /// association count so at least one class is reused.
    pub fn new(
        association_count: u64,
        delay_steps: u64,
        query_count: u64,
        shared_prefix_bits: u8,
        collision_classes: u64,
    ) -> Result<Self, TaskGeneratorError> {
        if association_count < 2 {
            return Err(TaskGeneratorError::T3NeedsMultipleAssociations);
        }
        if delay_steps == 0 {
            return Err(TaskGeneratorError::ZeroDelay {
                family: TaskFamily::InterferenceRecall,
            });
        }
        if query_count < 2 || query_count > association_count {
            return Err(TaskGeneratorError::InvalidQueryCount {
                family: TaskFamily::InterferenceRecall,
                query_count,
                association_count,
            });
        }
        if !(1..=63).contains(&shared_prefix_bits) {
            return Err(TaskGeneratorError::InvalidSharedPrefixBits { shared_prefix_bits });
        }
        if collision_classes == 0 || collision_classes >= association_count {
            return Err(TaskGeneratorError::InvalidCollisionClassCount {
                collision_classes,
                association_count,
            });
        }
        let suffix_bits = 64u32 - u32::from(shared_prefix_bits);
        let suffix_capacity = 1u128 << suffix_bits;
        if u128::from(association_count) > suffix_capacity {
            return Err(TaskGeneratorError::InsufficientDistinctKeySpace {
                association_count,
                suffix_bits: u8::try_from(suffix_bits).expect("suffix bits fit u8"),
            });
        }
        Ok(Self {
            association_count,
            delay_steps,
            query_count,
            shared_prefix_bits,
            collision_classes,
        })
    }

    /// Number of competing associations.
    #[must_use]
    pub const fn association_count(self) -> u64 {
        self.association_count
    }

    /// Number of irrelevant delay events before recall.
    #[must_use]
    pub const fn delay_steps(self) -> u64 {
        self.delay_steps
    }

    /// Number of queries, including oldest and most recent associations.
    #[must_use]
    pub const fn query_count(self) -> u64 {
        self.query_count
    }

    /// Number of high key-code bits shared by every competing key.
    #[must_use]
    pub const fn shared_prefix_bits(self) -> u8 {
        self.shared_prefix_bits
    }

    /// Number of generator-side collision/interference classes.
    #[must_use]
    pub const fn collision_classes(self) -> u64 {
        self.collision_classes
    }
}

/// Fail-closed task-generator validation/allocation errors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TaskGeneratorError {
    /// A horizon plan may not contain zero.
    ZeroHorizon,
    /// Short/medium/long values must be strictly increasing.
    NonIncreasingHorizons {
        /// Short horizon.
        short: u64,
        /// Medium horizon.
        medium: u64,
        /// Long horizon.
        long: u64,
    },
    /// T1 requires at least one queried and one unqueried association.
    T1NeedsDistractorAssociation,
    /// T2 payload must contain at least one symbol.
    ZeroPayloadLength,
    /// T3 requires at least two competing associations.
    T3NeedsMultipleAssociations,
    /// Delay must be positive for all three frozen task families.
    ZeroDelay {
        /// Frozen family whose delay was invalid.
        family: TaskFamily,
    },
    /// Query count violates the family-specific range.
    InvalidQueryCount {
        /// Frozen task family.
        family: TaskFamily,
        /// Requested query count.
        query_count: u64,
        /// Available association count for keyed tasks.
        association_count: u64,
    },
    /// T3 shared prefix must leave at least one suffix bit.
    InvalidSharedPrefixBits {
        /// Requested shared prefix length.
        shared_prefix_bits: u8,
    },
    /// T3 must reuse at least one non-zero generator-side collision class.
    InvalidCollisionClassCount {
        /// Requested class count.
        collision_classes: u64,
        /// Number of competing associations.
        association_count: u64,
    },
    /// Requested T3 associations exceed the distinct suffix code space.
    InsufficientDistinctKeySpace {
        /// Requested association count.
        association_count: u64,
        /// Available suffix width.
        suffix_bits: u8,
    },
    /// A count does not fit the host index representation.
    HostCountTooLarge {
        /// Name of the count.
        component: &'static str,
        /// Platform-independent value.
        value: u64,
    },
    /// Total generated event count overflowed host indexing.
    HostEventCountOverflow,
    /// A validated event/vector reservation exceeded host byte capacity.
    HostVectorCapacityTooLarge {
        /// Logical vector component.
        component: &'static str,
        /// Requested element count.
        elements: usize,
    },
    /// The host allocator rejected a validated reservation.
    HostAllocationFailed {
        /// Logical vector component.
        component: &'static str,
        /// Requested element count.
        elements: usize,
    },
}

impl fmt::Display for TaskGeneratorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroHorizon => formatter.write_str("TDI-8 horizon values must be positive"),
            Self::NonIncreasingHorizons {
                short,
                medium,
                long,
            } => write!(
                formatter,
                "TDI-8 horizons must satisfy short < medium < long, got {short}, {medium}, {long}"
            ),
            Self::T1NeedsDistractorAssociation => formatter.write_str(
                "T1 requires at least two associations so at least one association is unqueried",
            ),
            Self::ZeroPayloadLength => formatter.write_str("T2 payload length must be positive"),
            Self::T3NeedsMultipleAssociations => {
                formatter.write_str("T3 requires at least two competing associations")
            }
            Self::ZeroDelay { family } => write!(
                formatter,
                "{family:?} requires a strictly positive delayed-retrieval interval"
            ),
            Self::InvalidQueryCount {
                family,
                query_count,
                association_count,
            } => write!(
                formatter,
                "invalid {family:?} query count {query_count} for {association_count} associations"
            ),
            Self::InvalidSharedPrefixBits { shared_prefix_bits } => write!(
                formatter,
                "T3 shared prefix bits must be in 1..=63, got {shared_prefix_bits}"
            ),
            Self::InvalidCollisionClassCount {
                collision_classes,
                association_count,
            } => write!(
                formatter,
                "T3 collision classes must be in 1..association_count, got {collision_classes} for {association_count} associations"
            ),
            Self::InsufficientDistinctKeySpace {
                association_count,
                suffix_bits,
            } => write!(
                formatter,
                "T3 cannot fit {association_count} distinct keys into {suffix_bits} suffix bits"
            ),
            Self::HostCountTooLarge { component, value } => {
                write!(
                    formatter,
                    "{component}={value} does not fit the host index type"
                )
            }
            Self::HostEventCountOverflow => {
                formatter.write_str("TDI-8 generated event count overflowed host indexing")
            }
            Self::HostVectorCapacityTooLarge {
                component,
                elements,
            } => write!(
                formatter,
                "TDI-8 {component} vector capacity is too large: {elements} elements"
            ),
            Self::HostAllocationFailed {
                component,
                elements,
            } => write!(
                formatter,
                "host allocation failed for TDI-8 {component}: {elements} elements"
            ),
        }
    }
}

impl std::error::Error for TaskGeneratorError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    const fn new(seed: u64, domain: u64) -> Self {
        Self {
            state: seed ^ domain,
        }
    }

    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(SPLITMIX_GAMMA);
        mix64(self.state)
    }
}

fn mix64(mut value: u64) -> u64 {
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn host_count(component: &'static str, value: u64) -> Result<usize, TaskGeneratorError> {
    usize::try_from(value).map_err(|_| TaskGeneratorError::HostCountTooLarge { component, value })
}

fn checked_event_count(parts: &[usize]) -> Result<usize, TaskGeneratorError> {
    parts.iter().try_fold(0usize, |total, part| {
        total
            .checked_add(*part)
            .ok_or(TaskGeneratorError::HostEventCountOverflow)
    })
}

fn reserve_vec<T>(component: &'static str, elements: usize) -> Result<Vec<T>, TaskGeneratorError> {
    let bytes = elements.checked_mul(mem::size_of::<T>()).ok_or(
        TaskGeneratorError::HostVectorCapacityTooLarge {
            component,
            elements,
        },
    )?;
    if bytes > isize::MAX as usize {
        return Err(TaskGeneratorError::HostVectorCapacityTooLarge {
            component,
            elements,
        });
    }
    let mut values = Vec::new();
    values
        .try_reserve_exact(elements)
        .map_err(|_| TaskGeneratorError::HostAllocationFailed {
            component,
            elements,
        })?;
    Ok(values)
}

/// Generate one deterministic T1 associative-recall instance.
///
/// The queried subset is a seed-selected cyclic window over all associations;
/// because `query_count < association_count`, at least one introduced
/// association remains an explicit distractor association.
pub fn generate_t1(seed: u64, config: T1Config) -> Result<TaskInstance, TaskGeneratorError> {
    let association_count = host_count("T1 association_count", config.association_count)?;
    let delay_steps = host_count("T1 delay_steps", config.delay_steps)?;
    let query_count = host_count("T1 query_count", config.query_count)?;
    let total_events = checked_event_count(&[association_count, delay_steps, query_count])?;

    let mut events = reserve_vec("T1 events", total_events)?;
    let mut associations = reserve_vec("T1 associations", association_count)?;
    let mut key_stream = SplitMix64::new(seed, DOMAIN_T1_KEY);
    let mut value_stream = SplitMix64::new(seed, DOMAIN_T1_VALUE);
    let mut distractor_stream = SplitMix64::new(seed, DOMAIN_T1_DISTRACTOR);

    for source_index in 0..association_count {
        let source_index_u64 =
            u64::try_from(source_index).map_err(|_| TaskGeneratorError::HostEventCountOverflow)?;
        let key = TaskKey::new(key_stream.next(), source_index_u64);
        let value = TaskSymbol::new(value_stream.next());
        associations.push((key, value));
        events.push(TaskEvent::Associate {
            key,
            value,
            source_index: source_index_u64,
        });
    }

    for _ in 0..delay_steps {
        events.push(TaskEvent::Distractor {
            token: TaskSymbol::new(distractor_stream.next()),
        });
    }

    let query_start = usize::try_from(mix64(seed ^ DOMAIN_T1_QUERY) % config.association_count)
        .map_err(|_| TaskGeneratorError::HostEventCountOverflow)?;
    for query_offset in 0..query_count {
        let source_index = (query_start + query_offset) % association_count;
        let (key, target) = associations[source_index];
        events.push(TaskEvent::QueryAssociation {
            key,
            target,
            source_index: u64::try_from(source_index)
                .map_err(|_| TaskGeneratorError::HostEventCountOverflow)?,
        });
    }

    Ok(TaskInstance {
        family: TaskFamily::AssociativeRecall,
        seed,
        events,
        query_count: config.query_count,
    })
}

/// Generate one deterministic T2 delayed-copy instance.
pub fn generate_t2(seed: u64, config: T2Config) -> Result<TaskInstance, TaskGeneratorError> {
    let payload_len = host_count("T2 payload_len", config.payload_len)?;
    let delay_steps = host_count("T2 delay_steps", config.delay_steps)?;
    let total_events = checked_event_count(&[payload_len, delay_steps, payload_len])?;

    let mut events = reserve_vec("T2 events", total_events)?;
    let mut payload = reserve_vec("T2 payload", payload_len)?;
    let mut payload_stream = SplitMix64::new(seed, DOMAIN_T2_PAYLOAD);
    let mut distractor_stream = SplitMix64::new(seed, DOMAIN_T2_DISTRACTOR);

    for position in 0..payload_len {
        let position_u64 =
            u64::try_from(position).map_err(|_| TaskGeneratorError::HostEventCountOverflow)?;
        let value = TaskSymbol::new(payload_stream.next());
        payload.push(value);
        events.push(TaskEvent::Payload {
            position: position_u64,
            value,
        });
    }

    for _ in 0..delay_steps {
        events.push(TaskEvent::Distractor {
            token: TaskSymbol::new(distractor_stream.next()),
        });
    }

    for (position, target) in payload.into_iter().enumerate() {
        events.push(TaskEvent::QueryPayload {
            position: u64::try_from(position)
                .map_err(|_| TaskGeneratorError::HostEventCountOverflow)?,
            target,
        });
    }

    Ok(TaskInstance {
        family: TaskFamily::DelayedCopy,
        seed,
        events,
        query_count: config.payload_len,
    })
}

/// Generate one deterministic T3 interference-recall instance.
///
/// Every key shares the configured high-bit prefix. Distinct low suffixes are
/// generated by an odd affine permutation, so key uniqueness is exact rather
/// than probabilistic. `collision_class = source_index % collision_classes`
/// forces generator-side class reuse. The query set always contains source
/// index 0 (oldest) and `association_count - 1` (most recent).
pub fn generate_t3(seed: u64, config: T3Config) -> Result<TaskInstance, TaskGeneratorError> {
    let association_count = host_count("T3 association_count", config.association_count)?;
    let delay_steps = host_count("T3 delay_steps", config.delay_steps)?;
    let query_count = host_count("T3 query_count", config.query_count)?;
    let total_events = checked_event_count(&[association_count, delay_steps, query_count])?;

    let suffix_bits = 64u32 - u32::from(config.shared_prefix_bits);
    let suffix_mask = (1u64 << suffix_bits) - 1;
    let prefix_mask = !suffix_mask;
    let prefix = mix64(seed ^ DOMAIN_T3_PREFIX) & prefix_mask;
    let suffix_stride = (mix64(seed ^ DOMAIN_T3_SUFFIX_STRIDE) | 1) & suffix_mask;
    let suffix_offset = mix64(seed ^ DOMAIN_T3_SUFFIX_OFFSET) & suffix_mask;

    let mut events = reserve_vec("T3 events", total_events)?;
    let mut associations = reserve_vec("T3 associations", association_count)?;
    let mut value_stream = SplitMix64::new(seed, DOMAIN_T3_VALUE);
    let mut distractor_stream = SplitMix64::new(seed, DOMAIN_T3_DISTRACTOR);

    for source_index in 0..association_count {
        let source_index_u64 =
            u64::try_from(source_index).map_err(|_| TaskGeneratorError::HostEventCountOverflow)?;
        let suffix =
            suffix_offset.wrapping_add(source_index_u64.wrapping_mul(suffix_stride)) & suffix_mask;
        let key = TaskKey::new(prefix | suffix, source_index_u64 % config.collision_classes);
        let value = TaskSymbol::new(value_stream.next());
        associations.push((key, value));
        events.push(TaskEvent::Associate {
            key,
            value,
            source_index: source_index_u64,
        });
    }

    for _ in 0..delay_steps {
        events.push(TaskEvent::Distractor {
            token: TaskSymbol::new(distractor_stream.next()),
        });
    }

    let mut query_indices = reserve_vec("T3 query indices", query_count)?;
    query_indices.push(0usize);
    query_indices.push(association_count - 1);
    let mut candidate = 1usize;
    while query_indices.len() < query_count {
        if candidate != association_count - 1 {
            query_indices.push(candidate);
        }
        candidate += 1;
    }

    for source_index in query_indices {
        let (key, target) = associations[source_index];
        events.push(TaskEvent::QueryAssociation {
            key,
            target,
            source_index: u64::try_from(source_index)
                .map_err(|_| TaskGeneratorError::HostEventCountOverflow)?,
        });
    }

    Ok(TaskInstance {
        family: TaskFamily::InterferenceRecall,
        seed,
        events,
        query_count: config.query_count,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        HorizonPlan, HorizonStratum, T1Config, T2Config, T3Config, TaskEvent, TaskFamily,
        TaskGeneratorError, generate_t1, generate_t2, generate_t3,
    };

    #[test]
    fn horizon_plan_requires_positive_strictly_increasing_values_without_defaults() {
        assert_eq!(
            HorizonPlan::new(0, 2, 3),
            Err(TaskGeneratorError::ZeroHorizon)
        );
        assert!(matches!(
            HorizonPlan::new(3, 3, 9),
            Err(TaskGeneratorError::NonIncreasingHorizons { .. })
        ));
        let plan = HorizonPlan::new(3, 9, 27).expect("synthetic horizon plan");
        assert_eq!(plan.value(HorizonStratum::Short), 3);
        assert_eq!(plan.value(HorizonStratum::Medium), 9);
        assert_eq!(plan.value(HorizonStratum::Long), 27);
    }

    #[test]
    fn t1_is_seed_deterministic_and_contains_an_unqueried_distractor_association() {
        let config = T1Config::new(5, 3, 2).expect("synthetic T1 config");
        let left = generate_t1(17, config).expect("T1");
        let right = generate_t1(17, config).expect("T1");
        let different = generate_t1(18, config).expect("T1");
        assert_eq!(left, right);
        assert_ne!(left, different);
        assert_eq!(left.family(), TaskFamily::AssociativeRecall);
        assert_eq!(left.event_count(), 10);
        assert_eq!(left.query_count(), 2);

        let queried_sources: Vec<_> = left
            .events()
            .iter()
            .filter_map(|event| match event {
                TaskEvent::QueryAssociation { source_index, .. } => Some(*source_index),
                _ => None,
            })
            .collect();
        assert_eq!(queried_sources.len(), 2);
        assert!((0..5u64).any(|index| !queried_sources.contains(&index)));
        assert_eq!(
            left.events()
                .iter()
                .filter(|event| matches!(event, TaskEvent::Distractor { .. }))
                .count(),
            3
        );
    }

    #[test]
    fn t1_rejects_missing_delay_or_distractor_association() {
        assert_eq!(
            T1Config::new(1, 2, 1),
            Err(TaskGeneratorError::T1NeedsDistractorAssociation)
        );
        assert_eq!(
            T1Config::new(3, 0, 1),
            Err(TaskGeneratorError::ZeroDelay {
                family: TaskFamily::AssociativeRecall,
            })
        );
        assert!(matches!(
            T1Config::new(3, 2, 3),
            Err(TaskGeneratorError::InvalidQueryCount { .. })
        ));
    }

    #[test]
    fn t2_reproduces_the_original_payload_order_after_exact_delay() {
        let config = T2Config::new(3, 4).expect("synthetic T2 config");
        let instance = generate_t2(29, config).expect("T2");
        assert_eq!(instance.family(), TaskFamily::DelayedCopy);
        assert_eq!(instance.event_count(), 10);
        assert_eq!(instance.query_count(), 3);

        let payload: Vec<_> = instance
            .events()
            .iter()
            .filter_map(|event| match event {
                TaskEvent::Payload { position, value } => Some((*position, *value)),
                _ => None,
            })
            .collect();
        let queries: Vec<_> = instance
            .events()
            .iter()
            .filter_map(|event| match event {
                TaskEvent::QueryPayload { position, target } => Some((*position, *target)),
                _ => None,
            })
            .collect();
        assert_eq!(payload, queries);
        assert_eq!(
            instance
                .events()
                .iter()
                .filter(|event| matches!(event, TaskEvent::Distractor { .. }))
                .count(),
            4
        );
    }

    #[test]
    fn t3_enforces_shared_prefix_reused_collision_classes_and_recent_old_queries() {
        let config = T3Config::new(8, 3, 4, 20, 3).expect("synthetic T3 config");
        let instance = generate_t3(41, config).expect("T3");
        assert_eq!(instance.family(), TaskFamily::InterferenceRecall);
        assert_eq!(instance.event_count(), 15);

        let associations: Vec<_> = instance
            .events()
            .iter()
            .filter_map(|event| match event {
                TaskEvent::Associate { key, .. } => Some(*key),
                _ => None,
            })
            .collect();
        let suffix_bits = 64 - u32::from(config.shared_prefix_bits());
        let prefix_mask = !((1u64 << suffix_bits) - 1);
        let prefix = associations[0].code() & prefix_mask;
        assert!(
            associations
                .iter()
                .all(|key| key.code() & prefix_mask == prefix)
        );
        for (index, key) in associations.iter().enumerate() {
            assert_eq!(
                key.collision_class(),
                u64::try_from(index).expect("fixture index") % config.collision_classes()
            );
        }
        let mut codes: Vec<_> = associations.iter().map(|key| key.code()).collect();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), associations.len());

        let query_sources: Vec<_> = instance
            .events()
            .iter()
            .filter_map(|event| match event {
                TaskEvent::QueryAssociation { source_index, .. } => Some(*source_index),
                _ => None,
            })
            .collect();
        assert_eq!(query_sources.len(), 4);
        assert!(query_sources.contains(&0));
        assert!(query_sources.contains(&7));
    }

    #[test]
    fn t3_rejects_invalid_similarity_collision_or_query_pressure() {
        assert_eq!(
            T3Config::new(1, 2, 1, 8, 1),
            Err(TaskGeneratorError::T3NeedsMultipleAssociations)
        );
        assert!(matches!(
            T3Config::new(4, 2, 1, 8, 2),
            Err(TaskGeneratorError::InvalidQueryCount { .. })
        ));
        assert!(matches!(
            T3Config::new(4, 2, 2, 0, 2),
            Err(TaskGeneratorError::InvalidSharedPrefixBits { .. })
        ));
        assert!(matches!(
            T3Config::new(4, 2, 2, 8, 4),
            Err(TaskGeneratorError::InvalidCollisionClassCount { .. })
        ));
        assert!(matches!(
            T3Config::new(4, 2, 2, 63, 2),
            Err(TaskGeneratorError::InsufficientDistinctKeySpace { .. })
        ));
    }
}
