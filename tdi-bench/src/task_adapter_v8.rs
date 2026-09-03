//! Lossless task-adapter foundation for bounded TDI-8.1 evaluation.
//!
//! One already-generated symbolic task instance is mapped to one deterministic
//! schedule shared by A0/A1/A2/A3. This layer owns evaluation semantics, not
//! architecture primitives, and intentionally chooses no recurrent parameters,
//! memory capacities, horizons, population ranges or TDI-8.2 surface.

use core::{fmt, mem};

use tdi_ai::associative_memory::DirectMappedAssociativeMemory;
use tdi_ai::full_history_reference::FullHistoryLayout;
use tdi_ai::task_generators::{TaskEvent, TaskFamily, TaskInstance, TaskSymbol};

const LIMB_SCALE: f64 = 4_294_967_296.0;
const LIMB_MASK: u64 = 0xffff_ffff;
const EVENT_TAG_SCALE: f64 = 8.0;
const A0_NAMESPACE_SCALE: f64 = 8.0;
const PAYLOAD_KEY_DOMAIN: u64 = 0x7438_3170_6179_6c64;
const DISTRACTOR_READ_DOMAIN: u64 = 0x7438_3164_6973_7472;
const SEARCH_STEP: u64 = 0x9e37_79b9_7f4a_7c15;

/// Finite binary64 coordinates required for one exact `u64`.
pub const EXACT_U64_BINARY64_WIDTH: usize = 2;
/// Minimum A1/A2/A3 recurrent-input width for one lossless event frame.
pub const MIN_TASK_EVENT_INPUT_WIDTH: u64 = 9;
/// A0 task key width: one namespace tag plus two exact integer limbs.
pub const A0_TASK_KEY_WIDTH: u64 = 3;
/// A0 task value width: two exact integer limbs.
pub const A0_TASK_VALUE_WIDTH: u64 = 2;

/// Exact finite binary64 representation of one `u64`.
///
/// High/low 32-bit limbs are divided by `2^32`; every coordinate is therefore
/// exactly representable, finite and in `[0, 1)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExactU64Binary64([f64; EXACT_U64_BINARY64_WIDTH]);

impl ExactU64Binary64 {
    /// Encode one integer without information loss.
    #[must_use]
    pub fn encode(value: u64) -> Self {
        let high = (value >> 32) as u32;
        let low = (value & LIMB_MASK) as u32;
        Self([f64::from(high) / LIMB_SCALE, f64::from(low) / LIMB_SCALE])
    }

    /// Exact finite coordinates.
    #[must_use]
    pub const fn coordinates(self) -> [f64; EXACT_U64_BINARY64_WIDTH] {
        self.0
    }

    /// Decode only a canonical exact two-limb representation.
    pub fn decode(coordinates: [f64; EXACT_U64_BINARY64_WIDTH]) -> Result<u64, TaskAdapterError> {
        let high = decode_limb(0, coordinates[0])?;
        let low = decode_limb(1, coordinates[1])?;
        Ok((u64::from(high) << 32) | u64::from(low))
    }
}

fn decode_limb(index: usize, value: f64) -> Result<u32, TaskAdapterError> {
    if !value.is_finite() || !(0.0..1.0).contains(&value) {
        return Err(TaskAdapterError::NonCanonicalEncodedLimb {
            index,
            value_bits: value.to_bits(),
        });
    }
    let scaled = value * LIMB_SCALE;
    if scaled.fract() != 0.0 || scaled > f64::from(u32::MAX) {
        return Err(TaskAdapterError::NonCanonicalEncodedLimb {
            index,
            value_bits: value.to_bits(),
        });
    }
    Ok(scaled as u32)
}

/// Caller-supplied recurrent-input shape for A1/A2/A3 task frames.
///
/// The first nine coordinates have fixed lossless semantics. Additional
/// coordinates are deterministic zero padding, so later bounded development can
/// select a concrete input/VSA width without changing symbolic task identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TaskAdapterLayout {
    recurrent_input_width: u64,
}

impl TaskAdapterLayout {
    /// Construct a layout large enough for the fixed lossless event frame.
    pub fn new(recurrent_input_width: u64) -> Result<Self, TaskAdapterError> {
        if recurrent_input_width < MIN_TASK_EVENT_INPUT_WIDTH {
            return Err(TaskAdapterError::InputWidthTooSmall {
                minimum: MIN_TASK_EVENT_INPUT_WIDTH,
                actual: recurrent_input_width,
            });
        }
        let width = usize::try_from(recurrent_input_width).map_err(|_| {
            TaskAdapterError::HostDimensionTooLarge {
                component: "recurrent_input_width",
                value: recurrent_input_width,
            }
        })?;
        validate_vec_capacity("recurrent_input", width, mem::size_of::<f64>())?;
        Ok(Self {
            recurrent_input_width,
        })
    }

    /// Caller-supplied recurrent input width.
    #[must_use]
    pub const fn recurrent_input_width(self) -> u64 {
        self.recurrent_input_width
    }

    fn host_width(self) -> Result<usize, TaskAdapterError> {
        usize::try_from(self.recurrent_input_width).map_err(|_| {
            TaskAdapterError::HostDimensionTooLarge {
                component: "recurrent_input_width",
                value: self.recurrent_input_width,
            }
        })
    }

    /// A0 key/value layout for the same symbolic schedule.
    #[must_use]
    pub fn a0_layout(self) -> FullHistoryLayout {
        let _ = self;
        FullHistoryLayout::new(A0_TASK_KEY_WIDTH, A0_TASK_VALUE_WIDTH)
            .expect("fixed adapter widths are non-zero")
    }
}

/// Deterministic A0 action derived from one symbolic task event.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum A0TaskAction {
    /// Append one accessible full-history item.
    Append {
        /// Namespaced content key.
        key: [f64; A0_TASK_KEY_WIDTH as usize],
        /// Exact symbolic value.
        value: [f64; A0_TASK_VALUE_WIDTH as usize],
    },
    /// Read one exact target from full history.
    Read {
        /// Namespaced query key.
        key: [f64; A0_TASK_KEY_WIDTH as usize],
        /// Exact expected value.
        target: [f64; A0_TASK_VALUE_WIDTH as usize],
    },
}

/// One symbolic event mapped to the concrete A0/A1/A2/A3 call surfaces.
#[derive(Clone, Debug, PartialEq)]
pub struct TaskEventPlan {
    source_event: TaskEvent,
    recurrent_input: Vec<f64>,
    memory_read_key: u64,
    memory_write_key: Option<u64>,
    vsa_store_key: Option<u64>,
    a0_action: A0TaskAction,
    expected_target: Option<TaskSymbol>,
}

impl TaskEventPlan {
    /// Original symbolic event, unchanged.
    #[must_use]
    pub const fn source_event(&self) -> TaskEvent {
        self.source_event
    }

    /// Lossless event frame consumed identically by A1/A2/A3.
    #[must_use]
    pub fn recurrent_input(&self) -> &[f64] {
        &self.recurrent_input
    }

    /// Key passed to the A2/A3 read path.
    #[must_use]
    pub const fn memory_read_key(&self) -> u64 {
        self.memory_read_key
    }

    /// Optional A2/A3 post-transition write key.
    #[must_use]
    pub const fn memory_write_key(&self) -> Option<u64> {
        self.memory_write_key
    }

    /// Optional A3 VSA role key. When present, the evaluator stores the exact
    /// recurrent-input frame after the integrated A3 step.
    #[must_use]
    pub const fn vsa_store_key(&self) -> Option<u64> {
        self.vsa_store_key
    }

    /// A0 action for the same symbolic event.
    #[must_use]
    pub const fn a0_action(&self) -> A0TaskAction {
        self.a0_action
    }

    /// Exact symbolic target on task-query events only.
    #[must_use]
    pub const fn expected_target(&self) -> Option<TaskSymbol> {
        self.expected_target
    }
}

/// Complete deterministic schedule for one generated task instance.
#[derive(Clone, Debug, PartialEq)]
pub struct TaskAdapterPlan {
    family: TaskFamily,
    seed: u64,
    layout: TaskAdapterLayout,
    distractor_read_key: u64,
    events: Vec<TaskEventPlan>,
}

impl TaskAdapterPlan {
    /// Frozen task family inherited from the generator.
    #[must_use]
    pub const fn family(&self) -> TaskFamily {
        self.family
    }

    /// Exact generator seed inherited from the generator.
    #[must_use]
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    /// Adapter layout used for this schedule.
    #[must_use]
    pub const fn layout(&self) -> TaskAdapterLayout {
        self.layout
    }

    /// Dedicated no-write key selected outside this instance's write-key set.
    #[must_use]
    pub const fn distractor_read_key(&self) -> u64 {
        self.distractor_read_key
    }

    /// Ordered event schedule shared by all four arms.
    #[must_use]
    pub fn events(&self) -> &[TaskEventPlan] {
        &self.events
    }
}

/// Deterministic projection/collision audit for one concrete A2/A3 table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectionAudit {
    planned_writes: u64,
    distinct_occupied_addresses: u64,
    physical_replacement_collisions: u64,
    query_hits: u64,
    query_collision_misses: u64,
    query_empty: u64,
    generator_class_reuses: u64,
    class_aligned_physical_replacements: u64,
}

impl ProjectionAudit {
    #[must_use]
    pub const fn planned_writes(self) -> u64 {
        self.planned_writes
    }

    #[must_use]
    pub const fn distinct_occupied_addresses(self) -> u64 {
        self.distinct_occupied_addresses
    }

    #[must_use]
    pub const fn physical_replacement_collisions(self) -> u64 {
        self.physical_replacement_collisions
    }

    #[must_use]
    pub const fn query_hits(self) -> u64 {
        self.query_hits
    }

    #[must_use]
    pub const fn query_collision_misses(self) -> u64 {
        self.query_collision_misses
    }

    #[must_use]
    pub const fn query_empty(self) -> u64 {
        self.query_empty
    }

    /// Generator metadata reuse; not a physical collision count.
    #[must_use]
    pub const fn generator_class_reuses(self) -> u64 {
        self.generator_class_reuses
    }

    #[must_use]
    pub const fn class_aligned_physical_replacements(self) -> u64 {
        self.class_aligned_physical_replacements
    }
}

/// Fail-closed adaptation errors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TaskAdapterError {
    InputWidthTooSmall {
        minimum: u64,
        actual: u64,
    },
    HostDimensionTooLarge {
        component: &'static str,
        value: u64,
    },
    HostVectorCapacityTooLarge {
        component: &'static str,
        elements: usize,
        element_bytes: usize,
    },
    HostAllocationFailed {
        component: &'static str,
        elements: usize,
    },
    NonCanonicalEncodedLimb {
        index: usize,
        value_bits: u64,
    },
    EventCountTooLarge,
    NoDistinctDistractorReadKey,
}

impl fmt::Display for TaskAdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputWidthTooSmall { minimum, actual } => write!(
                formatter,
                "TDI-8.1 task adapter requires input width >= {minimum}, got {actual}"
            ),
            Self::HostDimensionTooLarge { component, value } => {
                write!(
                    formatter,
                    "{component}={value} does not fit the host index type"
                )
            }
            Self::HostVectorCapacityTooLarge {
                component,
                elements,
                element_bytes,
            } => write!(
                formatter,
                "{component} capacity too large: {elements} elements × {element_bytes} bytes"
            ),
            Self::HostAllocationFailed {
                component,
                elements,
            } => write!(
                formatter,
                "host allocation failed for {component}: {elements} elements"
            ),
            Self::NonCanonicalEncodedLimb { index, value_bits } => write!(
                formatter,
                "non-canonical exact-u64 limb {index}: bits={value_bits:016x}"
            ),
            Self::EventCountTooLarge => {
                formatter.write_str("task adapter event count exceeds u64 provenance range")
            }
            Self::NoDistinctDistractorReadKey => formatter.write_str(
                "could not select a distractor read key outside the instance write-key set",
            ),
        }
    }
}

impl std::error::Error for TaskAdapterError {}

fn validate_vec_capacity(
    component: &'static str,
    elements: usize,
    element_bytes: usize,
) -> Result<(), TaskAdapterError> {
    let bytes = elements.checked_mul(element_bytes).ok_or(
        TaskAdapterError::HostVectorCapacityTooLarge {
            component,
            elements,
            element_bytes,
        },
    )?;
    if bytes > isize::MAX as usize {
        return Err(TaskAdapterError::HostVectorCapacityTooLarge {
            component,
            elements,
            element_bytes,
        });
    }
    Ok(())
}

fn reserve_vec<T>(component: &'static str, elements: usize) -> Result<Vec<T>, TaskAdapterError> {
    validate_vec_capacity(component, elements, mem::size_of::<T>())?;
    let mut values = Vec::new();
    values
        .try_reserve_exact(elements)
        .map_err(|_| TaskAdapterError::HostAllocationFailed {
            component,
            elements,
        })?;
    Ok(values)
}

fn mix64(mut value: u64) -> u64 {
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn payload_memory_key(position: u64) -> u64 {
    mix64(position ^ PAYLOAD_KEY_DOMAIN)
}

fn event_tag(event: TaskEvent) -> f64 {
    let code = match event {
        TaskEvent::Associate { .. } => 1u32,
        TaskEvent::Payload { .. } => 2u32,
        TaskEvent::Distractor { .. } => 3u32,
        TaskEvent::QueryAssociation { .. } => 4u32,
        TaskEvent::QueryPayload { .. } => 5u32,
    };
    f64::from(code) / EVENT_TAG_SCALE
}

fn a0_namespace(event: TaskEvent) -> f64 {
    let code = match event {
        TaskEvent::Associate { .. } | TaskEvent::QueryAssociation { .. } => 1u32,
        TaskEvent::Payload { .. } | TaskEvent::QueryPayload { .. } => 2u32,
        TaskEvent::Distractor { .. } => 3u32,
    };
    f64::from(code) / A0_NAMESPACE_SCALE
}

fn a0_key(event: TaskEvent, identifier: u64) -> [f64; A0_TASK_KEY_WIDTH as usize] {
    let encoded = ExactU64Binary64::encode(identifier).coordinates();
    [a0_namespace(event), encoded[0], encoded[1]]
}

fn encoded_symbol(symbol: TaskSymbol) -> [f64; A0_TASK_VALUE_WIDTH as usize] {
    ExactU64Binary64::encode(symbol.code()).coordinates()
}

fn allocate_input(width: usize) -> Result<Vec<f64>, TaskAdapterError> {
    let mut input = reserve_vec("task event input", width)?;
    input.resize(width, 0.0);
    Ok(input)
}

fn fill_pair(input: &mut [f64], offset: usize, value: u64) {
    let encoded = ExactU64Binary64::encode(value).coordinates();
    input[offset] = encoded[0];
    input[offset + 1] = encoded[1];
}

fn encode_event(event: TaskEvent, width: usize) -> Result<Vec<f64>, TaskAdapterError> {
    let mut input = allocate_input(width)?;
    input[0] = event_tag(event);
    match event {
        TaskEvent::Associate {
            key,
            value,
            source_index,
        } => {
            fill_pair(&mut input, 1, key.code());
            fill_pair(&mut input, 3, key.collision_class());
            fill_pair(&mut input, 5, value.code());
            fill_pair(&mut input, 7, source_index);
        }
        TaskEvent::Payload { position, value } => {
            fill_pair(&mut input, 1, position);
            fill_pair(&mut input, 5, value.code());
            fill_pair(&mut input, 7, position);
        }
        TaskEvent::Distractor { token } => {
            fill_pair(&mut input, 1, token.code());
            fill_pair(&mut input, 5, token.code());
        }
        TaskEvent::QueryAssociation {
            key,
            target,
            source_index,
        } => {
            fill_pair(&mut input, 1, key.code());
            fill_pair(&mut input, 3, key.collision_class());
            fill_pair(&mut input, 5, target.code());
            fill_pair(&mut input, 7, source_index);
        }
        TaskEvent::QueryPayload { position, target } => {
            fill_pair(&mut input, 1, position);
            fill_pair(&mut input, 5, target.code());
            fill_pair(&mut input, 7, position);
        }
    }
    Ok(input)
}

fn write_key(event: TaskEvent) -> Option<u64> {
    match event {
        TaskEvent::Associate { key, .. } => Some(key.code()),
        TaskEvent::Payload { position, .. } => Some(payload_memory_key(position)),
        TaskEvent::Distractor { .. }
        | TaskEvent::QueryAssociation { .. }
        | TaskEvent::QueryPayload { .. } => None,
    }
}

fn query_key(event: TaskEvent) -> Option<u64> {
    match event {
        TaskEvent::QueryAssociation { key, .. } => Some(key.code()),
        TaskEvent::QueryPayload { position, .. } => Some(payload_memory_key(position)),
        _ => None,
    }
}

fn select_distractor_read_key(instance: &TaskInstance) -> Result<u64, TaskAdapterError> {
    let mut write_keys = reserve_vec("task write keys", instance.event_count())?;
    for event in instance.events() {
        if let Some(key) = write_key(*event) {
            write_keys.push(key);
        }
    }

    let mut candidate = mix64(instance.seed() ^ DISTRACTOR_READ_DOMAIN);
    for _ in 0..=write_keys.len() {
        if !write_keys.contains(&candidate) {
            return Ok(candidate);
        }
        candidate = candidate.wrapping_add(SEARCH_STEP);
    }
    Err(TaskAdapterError::NoDistinctDistractorReadKey)
}

/// Build one deterministic architecture-adapter schedule from one symbolic task.
pub fn build_task_adapter_plan(
    instance: &TaskInstance,
    layout: TaskAdapterLayout,
) -> Result<TaskAdapterPlan, TaskAdapterError> {
    let width = layout.host_width()?;
    let distractor_read_key = select_distractor_read_key(instance)?;
    let mut events = reserve_vec("task event plans", instance.event_count())?;

    for source_event in instance.events() {
        let source_event = *source_event;
        let recurrent_input = encode_event(source_event, width)?;
        let memory_write_key = write_key(source_event);
        let memory_read_key = query_key(source_event)
            .or(memory_write_key)
            .unwrap_or(distractor_read_key);
        let vsa_store_key = memory_write_key;

        let (a0_action, expected_target) = match source_event {
            TaskEvent::Associate { key, value, .. } => (
                A0TaskAction::Append {
                    key: a0_key(source_event, key.code()),
                    value: encoded_symbol(value),
                },
                None,
            ),
            TaskEvent::Payload { position, value } => (
                A0TaskAction::Append {
                    key: a0_key(source_event, position),
                    value: encoded_symbol(value),
                },
                None,
            ),
            TaskEvent::Distractor { token } => (
                A0TaskAction::Append {
                    key: a0_key(source_event, token.code()),
                    value: encoded_symbol(token),
                },
                None,
            ),
            TaskEvent::QueryAssociation { key, target, .. } => (
                A0TaskAction::Read {
                    key: a0_key(source_event, key.code()),
                    target: encoded_symbol(target),
                },
                Some(target),
            ),
            TaskEvent::QueryPayload { position, target } => (
                A0TaskAction::Read {
                    key: a0_key(source_event, position),
                    target: encoded_symbol(target),
                },
                Some(target),
            ),
        };

        events.push(TaskEventPlan {
            source_event,
            recurrent_input,
            memory_read_key,
            memory_write_key,
            vsa_store_key,
            a0_action,
            expected_target,
        });
    }

    Ok(TaskAdapterPlan {
        family: instance.family(),
        seed: instance.seed(),
        layout,
        distractor_read_key,
        events,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OccupiedProjection {
    address: u64,
    key: u64,
    generator_class: Option<u64>,
}

fn generator_class(event: TaskEvent) -> Option<u64> {
    match event {
        TaskEvent::Associate { key, .. } => Some(key.collision_class()),
        _ => None,
    }
}

/// Measure actual A2/A3 address collisions for one concrete associative table.
///
/// Generator class reuse and physical direct-mapped collisions are reported
/// separately. The audit does not inspect or mutate payload/recurrent state and
/// therefore cannot manufacture a task-success result.
pub fn audit_associative_projection(
    plan: &TaskAdapterPlan,
    memory: &DirectMappedAssociativeMemory,
) -> Result<ProjectionAudit, TaskAdapterError> {
    let mut occupied: Vec<OccupiedProjection> =
        reserve_vec("projection occupied addresses", plan.events.len())?;
    let mut seen_classes: Vec<u64> =
        reserve_vec("projection generator classes", plan.events.len())?;

    let mut planned_writes = 0u64;
    let mut physical_replacement_collisions = 0u64;
    let mut query_hits = 0u64;
    let mut query_collision_misses = 0u64;
    let mut query_empty = 0u64;
    let mut generator_class_reuses = 0u64;
    let mut class_aligned_physical_replacements = 0u64;

    for event in &plan.events {
        let class = generator_class(event.source_event);
        if let Some(class) = class {
            if seen_classes.contains(&class) {
                generator_class_reuses = generator_class_reuses
                    .checked_add(1)
                    .ok_or(TaskAdapterError::EventCountTooLarge)?;
            } else {
                seen_classes.push(class);
            }
        }

        if event.expected_target.is_some() {
            let address = memory.address_for(event.memory_read_key);
            if let Some(slot) = occupied.iter().find(|slot| slot.address == address) {
                if slot.key == event.memory_read_key {
                    query_hits = query_hits
                        .checked_add(1)
                        .ok_or(TaskAdapterError::EventCountTooLarge)?;
                } else {
                    query_collision_misses = query_collision_misses
                        .checked_add(1)
                        .ok_or(TaskAdapterError::EventCountTooLarge)?;
                }
            } else {
                query_empty = query_empty
                    .checked_add(1)
                    .ok_or(TaskAdapterError::EventCountTooLarge)?;
            }
        }

        if let Some(key) = event.memory_write_key {
            planned_writes = planned_writes
                .checked_add(1)
                .ok_or(TaskAdapterError::EventCountTooLarge)?;
            let address = memory.address_for(key);
            if let Some(slot) = occupied.iter_mut().find(|slot| slot.address == address) {
                if slot.key != key {
                    physical_replacement_collisions = physical_replacement_collisions
                        .checked_add(1)
                        .ok_or(TaskAdapterError::EventCountTooLarge)?;
                    if slot.generator_class.is_some() && slot.generator_class == class {
                        class_aligned_physical_replacements = class_aligned_physical_replacements
                            .checked_add(1)
                            .ok_or(TaskAdapterError::EventCountTooLarge)?;
                    }
                }
                slot.key = key;
                slot.generator_class = class;
            } else {
                occupied.push(OccupiedProjection {
                    address,
                    key,
                    generator_class: class,
                });
            }
        }
    }

    let distinct_occupied_addresses =
        u64::try_from(occupied.len()).map_err(|_| TaskAdapterError::EventCountTooLarge)?;
    Ok(ProjectionAudit {
        planned_writes,
        distinct_occupied_addresses,
        physical_replacement_collisions,
        query_hits,
        query_collision_misses,
        query_empty,
        generator_class_reuses,
        class_aligned_physical_replacements,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        A0_TASK_KEY_WIDTH, A0_TASK_VALUE_WIDTH, A0TaskAction, ExactU64Binary64,
        MIN_TASK_EVENT_INPUT_WIDTH, TaskAdapterError, TaskAdapterLayout, TaskEventPlan,
        audit_associative_projection, build_task_adapter_plan,
    };
    use tdi_ai::associative_memory::{AssociativeMemoryLayout, DirectMappedAssociativeMemory};
    use tdi_ai::task_generators::{
        T1Config, T2Config, T3Config, TaskEvent, generate_t1, generate_t2, generate_t3,
    };

    #[test]
    fn exact_u64_codec_round_trips_edge_values() {
        for value in [0, 1, u32::MAX as u64, 1u64 << 32, u64::MAX] {
            let encoded = ExactU64Binary64::encode(value).coordinates();
            assert!(encoded.iter().all(|coordinate| coordinate.is_finite()));
            assert!(
                encoded
                    .iter()
                    .all(|coordinate| (0.0..1.0).contains(coordinate))
            );
            assert_eq!(ExactU64Binary64::decode(encoded).expect("canonical"), value);
        }
    }

    #[test]
    fn decoder_rejects_noncanonical_coordinates() {
        assert!(matches!(
            ExactU64Binary64::decode([f64::NAN, 0.0]),
            Err(TaskAdapterError::NonCanonicalEncodedLimb { index: 0, .. })
        ));
        assert!(matches!(
            ExactU64Binary64::decode([1.0, 0.0]),
            Err(TaskAdapterError::NonCanonicalEncodedLimb { index: 0, .. })
        ));
        assert!(matches!(
            ExactU64Binary64::decode([0.1, 0.0]),
            Err(TaskAdapterError::NonCanonicalEncodedLimb { index: 0, .. })
        ));
    }

    #[test]
    fn layout_only_enforces_minimum_lossless_width() {
        assert_eq!(
            TaskAdapterLayout::new(MIN_TASK_EVENT_INPUT_WIDTH - 1),
            Err(TaskAdapterError::InputWidthTooSmall {
                minimum: MIN_TASK_EVENT_INPUT_WIDTH,
                actual: MIN_TASK_EVENT_INPUT_WIDTH - 1,
            })
        );
        let layout = TaskAdapterLayout::new(MIN_TASK_EVENT_INPUT_WIDTH + 4).expect("layout");
        assert_eq!(
            layout.recurrent_input_width(),
            MIN_TASK_EVENT_INPUT_WIDTH + 4
        );
        assert_eq!(layout.a0_layout().key_width(), A0_TASK_KEY_WIDTH);
        assert_eq!(layout.a0_layout().value_width(), A0_TASK_VALUE_WIDTH);
    }

    #[test]
    fn symbolic_t1_maps_to_shared_recurrent_schedule_and_exact_a0_targets() {
        let instance = generate_t1(17, T1Config::new(5, 3, 2).expect("config")).expect("T1");
        let layout = TaskAdapterLayout::new(MIN_TASK_EVENT_INPUT_WIDTH).expect("layout");
        let plan = build_task_adapter_plan(&instance, layout).expect("plan");
        assert_eq!(plan.events().len(), instance.events().len());
        assert!(plan.events().iter().all(|event| {
            event.recurrent_input().len() == MIN_TASK_EVENT_INPUT_WIDTH as usize
                && event
                    .recurrent_input()
                    .iter()
                    .all(|value| value.is_finite())
        }));

        let query_count = plan
            .events()
            .iter()
            .filter(|event| event.expected_target().is_some())
            .count();
        assert_eq!(query_count, 2);
        for event in plan.events() {
            if let A0TaskAction::Read { target, .. } = event.a0_action() {
                let target_code = ExactU64Binary64::decode(target).expect("exact target");
                assert_eq!(
                    Some(target_code),
                    event.expected_target().map(|value| value.code())
                );
            }
        }
    }

    #[test]
    fn t2_writes_and_queries_use_identical_derived_memory_keys() {
        let instance = generate_t2(29, T2Config::new(3, 4).expect("config")).expect("T2");
        let plan = build_task_adapter_plan(
            &instance,
            TaskAdapterLayout::new(MIN_TASK_EVENT_INPUT_WIDTH).expect("layout"),
        )
        .expect("plan");
        let writes: Vec<_> = plan
            .events()
            .iter()
            .filter_map(TaskEventPlan::memory_write_key)
            .collect();
        let queries: Vec<_> = plan
            .events()
            .iter()
            .filter(|event| event.expected_target().is_some())
            .map(TaskEventPlan::memory_read_key)
            .collect();
        assert_eq!(writes, queries);
    }

    #[test]
    fn distractor_read_key_is_outside_logical_write_set() {
        let instance = generate_t1(31, T1Config::new(7, 5, 3).expect("config")).expect("T1");
        let plan = build_task_adapter_plan(
            &instance,
            TaskAdapterLayout::new(MIN_TASK_EVENT_INPUT_WIDTH).expect("layout"),
        )
        .expect("plan");
        assert!(
            plan.events()
                .iter()
                .filter_map(TaskEventPlan::memory_write_key)
                .all(|key| key != plan.distractor_read_key())
        );
    }

    #[test]
    fn projection_audit_separates_generator_classes_from_physical_collisions() {
        let instance = generate_t3(41, T3Config::new(8, 3, 4, 20, 3).expect("config")).expect("T3");
        let plan = build_task_adapter_plan(
            &instance,
            TaskAdapterLayout::new(MIN_TASK_EVENT_INPUT_WIDTH).expect("layout"),
        )
        .expect("plan");
        let wide = DirectMappedAssociativeMemory::new(
            AssociativeMemoryLayout::new(1_024, 2).expect("layout"),
            11,
        )
        .expect("wide memory");
        let narrow = DirectMappedAssociativeMemory::new(
            AssociativeMemoryLayout::new(1, 2).expect("layout"),
            11,
        )
        .expect("narrow memory");

        let wide_audit = audit_associative_projection(&plan, &wide).expect("wide audit");
        let narrow_audit = audit_associative_projection(&plan, &narrow).expect("narrow audit");
        assert_eq!(wide_audit.generator_class_reuses(), 5);
        assert_eq!(narrow_audit.generator_class_reuses(), 5);
        assert_eq!(narrow_audit.planned_writes(), 8);
        assert_eq!(narrow_audit.distinct_occupied_addresses(), 1);
        assert_eq!(narrow_audit.physical_replacement_collisions(), 7);
        assert!(wide_audit.physical_replacement_collisions() < 7);
        assert_eq!(
            narrow_audit.query_hits()
                + narrow_audit.query_collision_misses()
                + narrow_audit.query_empty(),
            4
        );
    }

    #[test]
    fn source_event_order_is_preserved_exactly() {
        let instance = generate_t2(99, T2Config::new(2, 2).expect("config")).expect("T2");
        let plan = build_task_adapter_plan(
            &instance,
            TaskAdapterLayout::new(MIN_TASK_EVENT_INPUT_WIDTH).expect("layout"),
        )
        .expect("plan");
        let recovered: Vec<TaskEvent> = plan
            .events()
            .iter()
            .map(TaskEventPlan::source_event)
            .collect();
        assert_eq!(recovered, instance.events());
    }
}
