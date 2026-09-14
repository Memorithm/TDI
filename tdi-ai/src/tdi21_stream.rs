//! Bounded, causal TDI-21 B2/B3 development references.
//!
//! This is hand-written logical routing, not a trained sequence model. The
//! candidate receives one present event and no oracle answer or future stream.
//! Lookup probes one direct slot or two slots in one bucket, never history.

use super::tdi21::{
    ArchitectureArm, BooleanState, Clause, Literal, MemoryRead, ResourceCounters,
    activate_route, counted_route_tag,
};

pub const STREAM_SEMANTICS: &str = "tdi21-causal-stream-v1";
pub const MAX_SLOTS: usize = 4096;
pub const MAX_EVENTS: u64 = 1_000_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryMode {
    Direct,
    TwoWay,
}

impl MemoryMode {
    #[must_use]
    pub const fn arm(self) -> ArchitectureArm {
        match self {
            Self::Direct => ArchitectureArm::B2BooleanDirect,
            Self::TwoWay => ArchitectureArm::B3BooleanAssociative,
        }
    }

    const fn ways(self) -> usize {
        match self {
            Self::Direct => 1,
            Self::TwoWay => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StreamConfig {
    pub mode: MemoryMode,
    /// Total entry count, not bucket count. Two-way mode requires an even count.
    pub slots: usize,
    /// Payload width. Identity tags remain full 64-bit values in both arms.
    pub payload_bits: u8,
    pub max_events: u64,
    pub route_salt: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamError {
    InvalidSlotCount,
    OddTwoWaySlotCount,
    InvalidPayloadWidth,
    InvalidEventBudget,
    AllocationFailed,
    EventBudgetExhausted,
    InvalidMarker,
    PayloadOutOfRange,
}

/// Marker bit zero requests storage; bit one inhibits it. Remaining marker
/// bits are invalid. The expected answer is deliberately not an event field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    Write {
        key: u64,
        payload: BooleanState,
        marker: BooleanState,
    },
    Recall {
        key: u64,
    },
    Conjunction {
        left: u64,
        right: u64,
    },
    Ignore,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepOutput {
    Quiet,
    Reply(MemoryRead),
}

/// Counts accepted events only. Rejected API calls do not mutate the candidate
/// or yield evidence. Validation, allocator and evaluator costs are not CPU
/// instructions inferred from these semantic categories.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StreamCounters {
    pub events: u64,
    pub replies: u64,
    pub rejected_writes: u64,
    pub slot_probes: u64,
    pub work: ResourceCounters,
}

/// Not a process-memory or allocator-overhead measurement. Inline bytes include
/// configuration, counters and Vec metadata; reserved bytes use actual buffer
/// capacities. Semantic bits count entry fields and replacement metadata only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemoryFootprint {
    pub entry_semantic_bits: usize,
    pub replacement_semantic_bits: usize,
    pub reserved_buffer_bytes: usize,
    pub inline_bytes: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Entry {
    tag: u64,
    payload: BooleanState,
}

/// Closed reference implementation: callers cannot replace lookup with an
/// unaccounted callback. There is no attention branch, history or oracle field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BooleanStream {
    config: StreamConfig,
    entries: Vec<Option<Entry>>,
    next_victim: Vec<bool>,
    counters: StreamCounters,
}

fn increment(value: &mut u64) {
    *value = value.checked_add(1).expect("TDI-21 stream counter overflow");
}

impl BooleanStream {
    pub fn new(config: StreamConfig) -> Result<Self, StreamError> {
        if config.slots == 0 || config.slots > MAX_SLOTS {
            return Err(StreamError::InvalidSlotCount);
        }
        if config.mode == MemoryMode::TwoWay && config.slots % 2 != 0 {
            return Err(StreamError::OddTwoWaySlotCount);
        }
        if config.payload_bits == 0 || config.payload_bits > 64 {
            return Err(StreamError::InvalidPayloadWidth);
        }
        if config.max_events == 0 || config.max_events > MAX_EVENTS {
            return Err(StreamError::InvalidEventBudget);
        }
        let mut entries = Vec::new();
        entries
            .try_reserve_exact(config.slots)
            .map_err(|_| StreamError::AllocationFailed)?;
        entries.resize(config.slots, None);
        let mut next_victim = Vec::new();
        if config.mode == MemoryMode::TwoWay {
            next_victim
                .try_reserve_exact(config.slots / 2)
                .map_err(|_| StreamError::AllocationFailed)?;
            next_victim.resize(config.slots / 2, false);
        }
        Ok(Self {
            config,
            entries,
            next_victim,
            counters: StreamCounters::default(),
        })
    }

    #[must_use]
    pub fn config(&self) -> StreamConfig {
        self.config
    }

    #[must_use]
    pub fn counters(&self) -> StreamCounters {
        self.counters
    }

    /// Reset is O(slots), separately from per-event work; it is not free. No
    /// allocation occurs in reset or step. Counters start a new episode.
    pub fn reset(&mut self) {
        self.entries.fill(None);
        self.next_victim.fill(false);
        self.counters = StreamCounters::default();
    }

    #[must_use]
    pub fn footprint(&self) -> MemoryFootprint {
        MemoryFootprint {
            entry_semantic_bits: self.config.slots * 129,
            replacement_semantic_bits: self.next_victim.len(),
            reserved_buffer_bytes: self.entries.capacity() * std::mem::size_of::<Option<Entry>>()
                + self.next_victim.capacity() * std::mem::size_of::<bool>(),
            inline_bytes: std::mem::size_of::<Self>(),
        }
    }

    /// Fail without mutation on malformed input or exhausted event budget.
    pub fn step(&mut self, event: Event) -> Result<StepOutput, StreamError> {
        if self.counters.events >= self.config.max_events {
            return Err(StreamError::EventBudgetExhausted);
        }
        if let Event::Write {
            payload, marker, ..
        } = event
        {
            if marker.bits() > 3 {
                return Err(StreamError::InvalidMarker);
            }
            if self.config.payload_bits < 64 && payload.bits() >> self.config.payload_bits != 0 {
                return Err(StreamError::PayloadOutOfRange);
            }
        }
        increment(&mut self.counters.events);
        let output = match event {
            Event::Write {
                key,
                payload,
                marker,
            } => {
                let clause = Clause {
                    literals: [Literal::Bit(0), Literal::NotBit(1)],
                };
                if activate_route(marker, &clause, &mut self.counters.work) {
                    let tag = self.tag(key);
                    self.write(tag, payload);
                } else {
                    increment(&mut self.counters.rejected_writes);
                }
                StepOutput::Quiet
            }
            Event::Recall { key } => {
                let tag = self.tag(key);
                StepOutput::Reply(self.read(tag))
            }
            Event::Conjunction { left, right } => {
                let left_tag = self.tag(left);
                let right_tag = self.tag(right);
                let result = match (self.read(left_tag), self.read(right_tag)) {
                    (MemoryRead::Hit(a), MemoryRead::Hit(b)) => {
                        self.counters.work.charge_word_boolean_evals(1);
                        MemoryRead::Hit(a.and_state(b))
                    }
                    _ => MemoryRead::Miss,
                };
                StepOutput::Reply(result)
            }
            Event::Ignore => StepOutput::Quiet,
        };
        if matches!(output, StepOutput::Reply(_)) {
            increment(&mut self.counters.replies);
        }
        Ok(output)
    }

    fn tag(&mut self, key: u64) -> u64 {
        counted_route_tag(
            BooleanState::from_bits(key),
            self.config.route_salt,
            &mut self.counters.work,
        )
    }

    fn bucket(&self, tag: u64) -> (usize, usize) {
        let ways = self.config.mode.ways();
        let buckets = self.config.slots / ways;
        let bucket = ((tag as u128) % (buckets as u128)) as usize;
        (bucket, bucket * ways)
    }

    fn read(&mut self, tag: u64) -> MemoryRead {
        increment(&mut self.counters.work.memory_reads);
        let (_, start) = self.bucket(tag);
        for index in start..start + self.config.mode.ways() {
            increment(&mut self.counters.slot_probes);
            if let Some(entry) = self.entries[index] {
                increment(&mut self.counters.work.tag_equality_checks);
                if entry.tag == tag {
                    return MemoryRead::Hit(entry.payload);
                }
            }
        }
        increment(&mut self.counters.work.memory_misses);
        MemoryRead::Miss
    }

    fn write(&mut self, tag: u64, payload: BooleanState) {
        increment(&mut self.counters.work.memory_writes);
        let (bucket, start) = self.bucket(tag);
        let mut vacant = None;
        for index in start..start + self.config.mode.ways() {
            increment(&mut self.counters.slot_probes);
            if let Some(entry) = self.entries[index] {
                increment(&mut self.counters.work.tag_equality_checks);
                if entry.tag == tag {
                    self.entries[index] = Some(Entry { tag, payload });
                    return;
                }
            } else if vacant.is_none() {
                vacant = Some(index);
            }
        }
        let index = if let Some(index) = vacant {
            index
        } else {
            // A collision here means a full bucket requiring eviction. Merely
            // sharing a bucket is not an eviction or a false hit.
            increment(&mut self.counters.work.memory_collisions);
            increment(&mut self.counters.work.memory_replacements);
            if self.config.mode == MemoryMode::TwoWay {
                let index = start + usize::from(self.next_victim[bucket]);
                self.next_victim[bucket] = !self.next_victim[bucket];
                index
            } else {
                start
            }
        };
        self.entries[index] = Some(Entry { tag, payload });
    }
}
