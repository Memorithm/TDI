//! Leakage-safe binary64 task encoding support for bounded TDI-8.1.
//!
//! This module sits immediately behind [`crate::task_execution::SymbolicTaskAdapter`].
//! Its arm-facing encoders accept only the symbolic arguments that contract exposes:
//! association key/value, payload value, distractor token, association query key,
//! and payload query position. Exact query targets, generator source indices, and
//! T3 collision-class annotations cannot be supplied to these encoders.
//!
//! The module also provides runner-side logical memory-key helpers and a read-only
//! physical direct-mapped projection audit. Those diagnostics may inspect the
//! immutable generated task, but they never turn generator metadata into arm input.
//!
//! No recurrent dimensions beyond the lossless minimum, table capacity, projection
//! seed, fusion gain, matched budget, horizon, population, deficit, interval method,
//! or TDI-8.2 surface is selected here.

use core::{fmt, mem};

use crate::associative_memory::DirectMappedAssociativeMemory;
use crate::task_generators::{TaskEvent, TaskInstance, TaskSymbol};

const LIMB_SCALE: f64 = 4_294_967_296.0;
const LIMB_MASK: u64 = 0xffff_ffff;
const EVENT_TAG_SCALE: f64 = 8.0;
const A0_NAMESPACE_SCALE: f64 = 8.0;
const PAYLOAD_KEY_DOMAIN: u64 = 0x7438_3170_6179_6c64;
const DISTRACTOR_READ_DOMAIN: u64 = 0x7438_3164_6973_7472;
const SEARCH_STEP: u64 = 0x9e37_79b9_7f4a_7c15;

/// Finite binary64 coordinates required for one exact `u64`.
pub const EXACT_U64_BINARY64_WIDTH: usize = 2;
/// Minimum recurrent input width needed by the leakage-safe arm-facing frame.
///
/// The largest exposed stimulus is an association: one event tag, two exact key
/// limbs, and two exact value limbs. Query targets and evaluator metadata are not
/// part of this width.
pub const MIN_TASK_INPUT_WIDTH: u64 = 5;
/// A0 key width: one namespace coordinate plus two exact integer limbs.
pub const A0_TASK_KEY_WIDTH: usize = 3;
/// A0 value width: two exact integer limbs.
pub const A0_TASK_VALUE_WIDTH: usize = 2;

/// Exact finite binary64 representation of one `u64`.
///
/// The high and low 32-bit limbs are divided by `2^32`. Every coordinate is an
/// exact finite binary fraction in `[0, 1)`, so the complete `u64` domain is
/// represented injectively without lossy integer-to-`f64` casts.
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

    /// Return the exact two-coordinate representation.
    #[must_use]
    pub const fn coordinates(self) -> [f64; EXACT_U64_BINARY64_WIDTH] {
        self.0
    }

    /// Decode only a canonical exact two-limb representation.
    pub fn decode(coordinates: [f64; EXACT_U64_BINARY64_WIDTH]) -> Result<u64, TaskEncodingError> {
        let high = decode_limb(0, coordinates[0])?;
        let low = decode_limb(1, coordinates[1])?;
        Ok((u64::from(high) << 32) | u64::from(low))
    }
}

fn decode_limb(index: usize, value: f64) -> Result<u32, TaskEncodingError> {
    if !value.is_finite()
        || value.to_bits() == (-0.0f64).to_bits()
        || !(0.0..1.0).contains(&value)
    {
        return Err(TaskEncodingError::NonCanonicalEncodedLimb {
            index,
            value_bits: value.to_bits(),
        });
    }
    let scaled = value * LIMB_SCALE;
    if scaled.fract() != 0.0 || scaled > f64::from(u32::MAX) {
        return Err(TaskEncodingError::NonCanonicalEncodedLimb {
            index,
            value_bits: value.to_bits(),
        });
    }
    Ok(scaled as u32)
}

/// Caller-selected arm-facing recurrent input layout.
///
/// Only the lossless minimum is enforced. Larger widths are deterministic zero
/// padding and do not constitute a frozen TDI-8.1 experimental dimension.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TaskInputLayout {
    width: u64,
}

impl TaskInputLayout {
    /// Require enough coordinates for every leakage-safe symbolic stimulus.
    pub fn new(width: u64) -> Result<Self, TaskEncodingError> {
        if width < MIN_TASK_INPUT_WIDTH {
            return Err(TaskEncodingError::InputWidthTooSmall {
                minimum: MIN_TASK_INPUT_WIDTH,
                actual: width,
            });
        }
        let host_width = usize::try_from(width)
            .map_err(|_| TaskEncodingError::HostDimensionTooLarge { value: width })?;
        validate_vec_capacity("task input", host_width, mem::size_of::<f64>())?;
        Ok(Self { width })
    }

    /// Caller-selected width.
    #[must_use]
    pub const fn width(self) -> u64 {
        self.width
    }

    fn host_width(self) -> Result<usize, TaskEncodingError> {
        usize::try_from(self.width)
            .map_err(|_| TaskEncodingError::HostDimensionTooLarge { value: self.width })
    }
}

/// Stateless lossless encoder for the exact arm-facing symbolic contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LosslessTaskEncoder {
    layout: TaskInputLayout,
}

impl LosslessTaskEncoder {
    /// Construct an encoder from a caller-selected layout.
    #[must_use]
    pub const fn new(layout: TaskInputLayout) -> Self {
        Self { layout }
    }

    /// Recurrent input layout used by every encoded stimulus.
    #[must_use]
    pub const fn layout(self) -> TaskInputLayout {
        self.layout
    }

    /// Encode `SymbolicTaskAdapter::associate(key_code, value)`.
    pub fn association(
        self,
        key_code: u64,
        value: TaskSymbol,
    ) -> Result<Vec<f64>, TaskEncodingError> {
        let mut input = self.blank_input()?;
        input[0] = event_tag(EventTag::Association);
        fill_exact_u64(&mut input, 1, key_code);
        fill_exact_u64(&mut input, 3, value.code());
        Ok(input)
    }

    /// Encode `SymbolicTaskAdapter::payload(value)`.
    ///
    /// Payload source position is intentionally not accepted; chronological call
    /// order is the only arm-visible source-order signal.
    pub fn payload(self, value: TaskSymbol) -> Result<Vec<f64>, TaskEncodingError> {
        let mut input = self.blank_input()?;
        input[0] = event_tag(EventTag::Payload);
        fill_exact_u64(&mut input, 1, value.code());
        Ok(input)
    }

    /// Encode `SymbolicTaskAdapter::distractor(token)`.
    pub fn distractor(self, token: TaskSymbol) -> Result<Vec<f64>, TaskEncodingError> {
        let mut input = self.blank_input()?;
        input[0] = event_tag(EventTag::Distractor);
        fill_exact_u64(&mut input, 1, token.code());
        Ok(input)
    }

    /// Encode `SymbolicTaskAdapter::query_association(key_code)`.
    ///
    /// No target, source index, or generator collision class can enter this API.
    pub fn query_association(self, key_code: u64) -> Result<Vec<f64>, TaskEncodingError> {
        let mut input = self.blank_input()?;
        input[0] = event_tag(EventTag::QueryAssociation);
        fill_exact_u64(&mut input, 1, key_code);
        Ok(input)
    }

    /// Encode `SymbolicTaskAdapter::query_payload(position)`.
    ///
    /// The requested position is the symbolic query itself; its exact target is
    /// not accepted by this API.
    pub fn query_payload(self, position: u64) -> Result<Vec<f64>, TaskEncodingError> {
        let mut input = self.blank_input()?;
        input[0] = event_tag(EventTag::QueryPayload);
        fill_exact_u64(&mut input, 1, position);
        Ok(input)
    }

    fn blank_input(self) -> Result<Vec<f64>, TaskEncodingError> {
        let width = self.layout.host_width()?;
        let mut input = reserve_vec("task input", width)?;
        input.resize(width, 0.0);
        Ok(input)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum EventTag {
    Association,
    Payload,
    Distractor,
    QueryAssociation,
    QueryPayload,
}

fn event_tag(tag: EventTag) -> f64 {
    let code = match tag {
        EventTag::Association => 1u32,
        EventTag::Payload => 2u32,
        EventTag::Distractor => 3u32,
        EventTag::QueryAssociation => 4u32,
        EventTag::QueryPayload => 5u32,
    };
    f64::from(code) / EVENT_TAG_SCALE
}

fn fill_exact_u64(input: &mut [f64], offset: usize, value: u64) {
    let encoded = ExactU64Binary64::encode(value).coordinates();
    input[offset] = encoded[0];
    input[offset + 1] = encoded[1];
}

/// One exact A0 history item.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct A0EncodedItem {
    key: [f64; A0_TASK_KEY_WIDTH],
    value: [f64; A0_TASK_VALUE_WIDTH],
}

impl A0EncodedItem {
    /// Exact namespaced key.
    #[must_use]
    pub const fn key(self) -> [f64; A0_TASK_KEY_WIDTH] {
        self.key
    }

    /// Exact symbolic value.
    #[must_use]
    pub const fn value(self) -> [f64; A0_TASK_VALUE_WIDTH] {
        self.value
    }
}

/// Exact A0 association item from arm-visible arguments only.
#[must_use]
pub fn a0_association_item(key_code: u64, value: TaskSymbol) -> A0EncodedItem {
    A0EncodedItem {
        key: a0_key(A0Namespace::Association, key_code),
        value: encoded_symbol(value),
    }
}

/// Exact A0 ordered-payload item.
///
/// A concrete adapter supplies `position` from its own payload call counter, not
/// from generator-side payload provenance.
#[must_use]
pub fn a0_payload_item(position: u64, value: TaskSymbol) -> A0EncodedItem {
    A0EncodedItem {
        key: a0_key(A0Namespace::Payload, position),
        value: encoded_symbol(value),
    }
}

/// Exact A0 distractor item in a namespace disjoint from task queries.
#[must_use]
pub fn a0_distractor_item(token: TaskSymbol) -> A0EncodedItem {
    A0EncodedItem {
        key: a0_key(A0Namespace::Distractor, token.code()),
        value: encoded_symbol(token),
    }
}

/// A0 association query key. Exact target remains runner-owned.
#[must_use]
pub fn a0_association_query_key(key_code: u64) -> [f64; A0_TASK_KEY_WIDTH] {
    a0_key(A0Namespace::Association, key_code)
}

/// A0 payload query key. Exact target remains runner-owned.
#[must_use]
pub fn a0_payload_query_key(position: u64) -> [f64; A0_TASK_KEY_WIDTH] {
    a0_key(A0Namespace::Payload, position)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum A0Namespace {
    Association,
    Payload,
    Distractor,
}

fn a0_key(namespace: A0Namespace, identifier: u64) -> [f64; A0_TASK_KEY_WIDTH] {
    let namespace_code = match namespace {
        A0Namespace::Association => 1u32,
        A0Namespace::Payload => 2u32,
        A0Namespace::Distractor => 3u32,
    };
    let encoded = ExactU64Binary64::encode(identifier).coordinates();
    [
        f64::from(namespace_code) / A0_NAMESPACE_SCALE,
        encoded[0],
        encoded[1],
    ]
}

fn encoded_symbol(symbol: TaskSymbol) -> [f64; A0_TASK_VALUE_WIDTH] {
    ExactU64Binary64::encode(symbol.code()).coordinates()
}

/// A2/A3 logical key for one association.
#[must_use]
pub const fn association_memory_key(key_code: u64) -> u64 {
    key_code
}

/// A2/A3 domain-separated logical key for one ordered payload position.
#[must_use]
pub fn payload_memory_key(position: u64) -> u64 {
    mix64(position ^ PAYLOAD_KEY_DOMAIN)
}

/// Leakage-safe payload write-key cursor for a concrete adapter.
///
/// `SymbolicTaskAdapter::payload` exposes no source position. A concrete adapter
/// advances this counter once per payload call, reproducing the generated T2
/// positions solely from chronological call order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PayloadKeyCursor {
    next_position: u64,
}

impl PayloadKeyCursor {
    /// Reset to the first payload position.
    pub fn reset(&mut self) {
        self.next_position = 0;
    }

    /// Current zero-based payload position.
    #[must_use]
    pub const fn next_position(self) -> u64 {
        self.next_position
    }

    /// Return the next payload write key and advance exactly once.
    pub fn next_write_key(&mut self) -> Result<u64, TaskEncodingError> {
        let position = self.next_position;
        self.next_position = self
            .next_position
            .checked_add(1)
            .ok_or(TaskEncodingError::PayloadPositionOverflow)?;
        Ok(payload_memory_key(position))
    }
}

/// Select a deterministic read-only distractor key outside one immutable task's
/// complete logical A2/A3 write-key set.
pub fn distractor_read_key_for_instance(instance: &TaskInstance) -> Result<u64, TaskEncodingError> {
    let write_keys = logical_write_keys(instance)?;
    let mut candidate = mix64(instance.seed() ^ DISTRACTOR_READ_DOMAIN);
    for _ in 0..=write_keys.len() {
        if !write_keys.contains(&candidate) {
            return Ok(candidate);
        }
        candidate = candidate.wrapping_add(SEARCH_STEP);
    }
    Err(TaskEncodingError::NoDistinctDistractorReadKey)
}

fn logical_write_keys(instance: &TaskInstance) -> Result<Vec<u64>, TaskEncodingError> {
    let mut keys = reserve_vec("logical write keys", instance.event_count())?;
    let mut payload_cursor = PayloadKeyCursor::default();
    for event in instance.events() {
        match *event {
            TaskEvent::Associate { key, .. } => keys.push(association_memory_key(key.code())),
            TaskEvent::Payload { position, .. } => {
                ensure_payload_position(payload_cursor.next_position(), position)?;
                keys.push(payload_cursor.next_write_key()?);
            }
            TaskEvent::Distractor { .. }
            | TaskEvent::QueryAssociation { .. }
            | TaskEvent::QueryPayload { .. } => {}
        }
    }
    Ok(keys)
}

fn ensure_payload_position(expected: u64, observed: u64) -> Result<(), TaskEncodingError> {
    if expected == observed {
        Ok(())
    } else {
        Err(TaskEncodingError::PayloadPositionMismatch { expected, observed })
    }
}

fn mix64(mut value: u64) -> u64 {
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

/// Deterministic runner-side audit of one concrete A2/A3 projection.
///
/// Generator-side T3 class reuse and physical direct-mapped replacements are
/// deliberately separate counters. The audit does not inspect or mutate memory
/// payloads and cannot emit task success, a deficit, or a hypothesis verdict.
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

    /// Generator-side class reuse count; this is not a physical collision count.
    #[must_use]
    pub const fn generator_class_reuses(self) -> u64 {
        self.generator_class_reuses
    }

    #[must_use]
    pub const fn class_aligned_physical_replacements(self) -> u64 {
        self.class_aligned_physical_replacements
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OccupiedProjection {
    address: u64,
    key: u64,
    generator_class: Option<u64>,
}

/// Replay only logical keys through one concrete direct-mapped address function.
pub fn audit_associative_projection(
    instance: &TaskInstance,
    memory: &DirectMappedAssociativeMemory,
) -> Result<ProjectionAudit, TaskEncodingError> {
    let mut occupied: Vec<OccupiedProjection> =
        reserve_vec("projection occupied addresses", instance.event_count())?;
    let mut seen_classes: Vec<u64> =
        reserve_vec("projection generator classes", instance.event_count())?;
    let mut payload_cursor = PayloadKeyCursor::default();

    let mut planned_writes = 0u64;
    let mut physical_replacement_collisions = 0u64;
    let mut query_hits = 0u64;
    let mut query_collision_misses = 0u64;
    let mut query_empty = 0u64;
    let mut generator_class_reuses = 0u64;
    let mut class_aligned_physical_replacements = 0u64;

    for event in instance.events() {
        let (write_key, query_key, generator_class) = match *event {
            TaskEvent::Associate { key, .. } => {
                let class = Some(key.collision_class());
                if seen_classes.contains(&key.collision_class()) {
                    generator_class_reuses = checked_increment(generator_class_reuses)?;
                } else {
                    seen_classes.push(key.collision_class());
                }
                (Some(association_memory_key(key.code())), None, class)
            }
            TaskEvent::Payload { position, .. } => {
                ensure_payload_position(payload_cursor.next_position(), position)?;
                (Some(payload_cursor.next_write_key()?), None, None)
            }
            TaskEvent::Distractor { .. } => (None, None, None),
            TaskEvent::QueryAssociation { key, .. } => {
                (None, Some(association_memory_key(key.code())), None)
            }
            TaskEvent::QueryPayload { position, .. } => {
                (None, Some(payload_memory_key(position)), None)
            }
        };

        if let Some(key) = query_key {
            let address = memory.address_for(key);
            if let Some(slot) = occupied.iter().find(|slot| slot.address == address) {
                if slot.key == key {
                    query_hits = checked_increment(query_hits)?;
                } else {
                    query_collision_misses = checked_increment(query_collision_misses)?;
                }
            } else {
                query_empty = checked_increment(query_empty)?;
            }
        }

        if let Some(key) = write_key {
            planned_writes = checked_increment(planned_writes)?;
            let address = memory.address_for(key);
            if let Some(slot) = occupied.iter_mut().find(|slot| slot.address == address) {
                if slot.key != key {
                    physical_replacement_collisions =
                        checked_increment(physical_replacement_collisions)?;
                    if slot.generator_class.is_some() && slot.generator_class == generator_class {
                        class_aligned_physical_replacements =
                            checked_increment(class_aligned_physical_replacements)?;
                    }
                }
                slot.key = key;
                slot.generator_class = generator_class;
            } else {
                occupied.push(OccupiedProjection {
                    address,
                    key,
                    generator_class,
                });
            }
        }
    }

    let distinct_occupied_addresses =
        u64::try_from(occupied.len()).map_err(|_| TaskEncodingError::EventCountTooLarge)?;
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

fn checked_increment(value: u64) -> Result<u64, TaskEncodingError> {
    value
        .checked_add(1)
        .ok_or(TaskEncodingError::EventCountTooLarge)
}

/// Fail-closed lossless-encoding and diagnostic errors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TaskEncodingError {
    InputWidthTooSmall {
        minimum: u64,
        actual: u64,
    },
    HostDimensionTooLarge {
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
    PayloadPositionOverflow,
    PayloadPositionMismatch {
        expected: u64,
        observed: u64,
    },
    EventCountTooLarge,
    NoDistinctDistractorReadKey,
}

impl fmt::Display for TaskEncodingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputWidthTooSmall { minimum, actual } => write!(
                formatter,
                "TDI-8.1 lossless task encoding requires input width >= {minimum}, got {actual}"
            ),
            Self::HostDimensionTooLarge { value } => {
                write!(
                    formatter,
                    "task input width {value} does not fit the host index type"
                )
            }
            Self::HostVectorCapacityTooLarge {
                component,
                elements,
                element_bytes,
            } => write!(
                formatter,
                "{component} capacity too large: {elements} elements x {element_bytes} bytes"
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
            Self::PayloadPositionOverflow => {
                formatter.write_str("payload call-order position overflowed u64")
            }
            Self::PayloadPositionMismatch { expected, observed } => write!(
                formatter,
                "generated payload position mismatch: expected chronological position {expected}, observed {observed}"
            ),
            Self::EventCountTooLarge => {
                formatter.write_str("task projection accounting exceeded u64")
            }
            Self::NoDistinctDistractorReadKey => formatter.write_str(
                "could not select a distractor read key outside the instance write-key set",
            ),
        }
    }
}

impl std::error::Error for TaskEncodingError {}

fn validate_vec_capacity(
    component: &'static str,
    elements: usize,
    element_bytes: usize,
) -> Result<(), TaskEncodingError> {
    let bytes = elements.checked_mul(element_bytes).ok_or(
        TaskEncodingError::HostVectorCapacityTooLarge {
            component,
            elements,
            element_bytes,
        },
    )?;
    if bytes > isize::MAX as usize {
        return Err(TaskEncodingError::HostVectorCapacityTooLarge {
            component,
            elements,
            element_bytes,
        });
    }
    Ok(())
}

fn reserve_vec<T>(component: &'static str, elements: usize) -> Result<Vec<T>, TaskEncodingError> {
    validate_vec_capacity(component, elements, mem::size_of::<T>())?;
    let mut values = Vec::new();
    values
        .try_reserve_exact(elements)
        .map_err(|_| TaskEncodingError::HostAllocationFailed {
            component,
            elements,
        })?;
    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::{
        A0_TASK_KEY_WIDTH, ExactU64Binary64, LosslessTaskEncoder, MIN_TASK_INPUT_WIDTH,
        PayloadKeyCursor, TaskEncodingError, TaskInputLayout, a0_association_item,
        a0_association_query_key, a0_distractor_item, a0_payload_item, a0_payload_query_key,
        audit_associative_projection, distractor_read_key_for_instance, payload_memory_key,
    };
    use crate::associative_memory::{AssociativeMemoryLayout, DirectMappedAssociativeMemory};
    use crate::task_generators::{
        T1Config, T2Config, T3Config, TaskEvent, TaskSymbol, generate_t1, generate_t2, generate_t3,
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
            Err(TaskEncodingError::NonCanonicalEncodedLimb { index: 0, .. })
        ));
        assert!(matches!(
            ExactU64Binary64::decode([1.0, 0.0]),
            Err(TaskEncodingError::NonCanonicalEncodedLimb { index: 0, .. })
        ));
        assert!(matches!(
            ExactU64Binary64::decode([0.1, 0.0]),
            Err(TaskEncodingError::NonCanonicalEncodedLimb { index: 0, .. })
        ));
        assert!(matches!(
            ExactU64Binary64::decode([-0.0, 0.0]),
            Err(TaskEncodingError::NonCanonicalEncodedLimb { index: 0, .. })
        ));
    }

    #[test]
    fn layout_enforces_only_the_leakage_safe_lossless_minimum() {
        assert_eq!(
            TaskInputLayout::new(MIN_TASK_INPUT_WIDTH - 1),
            Err(TaskEncodingError::InputWidthTooSmall {
                minimum: MIN_TASK_INPUT_WIDTH,
                actual: MIN_TASK_INPUT_WIDTH - 1,
            })
        );
        let layout = TaskInputLayout::new(MIN_TASK_INPUT_WIDTH + 4).expect("layout");
        assert_eq!(layout.width(), MIN_TASK_INPUT_WIDTH + 4);
    }

    #[test]
    fn query_frames_contain_request_only_and_zero_padding() {
        let encoder = LosslessTaskEncoder::new(TaskInputLayout::new(9).expect("layout"));
        let association = encoder
            .query_association(0x1234_5678_9abc_def0)
            .expect("association query");
        let payload = encoder
            .query_payload(0xfedc_ba98_7654_3210)
            .expect("payload query");

        assert_eq!(association.len(), 9);
        assert_eq!(payload.len(), 9);
        assert!(association[3..].iter().all(|value| *value == 0.0));
        assert!(payload[3..].iter().all(|value| *value == 0.0));
        assert_ne!(association[0].to_bits(), payload[0].to_bits());
    }

    #[test]
    fn association_frame_contains_only_key_and_value_after_tag() {
        let encoder =
            LosslessTaskEncoder::new(TaskInputLayout::new(MIN_TASK_INPUT_WIDTH).expect("layout"));
        let key = 0xaaaa_bbbb_cccc_dddd;
        let value = TaskSymbol::new(0x1111_2222_3333_4444);
        let frame = encoder.association(key, value).expect("association");

        assert_eq!(frame.len(), MIN_TASK_INPUT_WIDTH as usize);
        assert_eq!(
            ExactU64Binary64::decode([frame[1], frame[2]]).expect("key"),
            key
        );
        assert_eq!(
            ExactU64Binary64::decode([frame[3], frame[4]]).expect("value"),
            value.code()
        );
    }

    #[test]
    fn payload_frame_has_no_source_position_feature() {
        let encoder = LosslessTaskEncoder::new(TaskInputLayout::new(7).expect("layout"));
        let value = TaskSymbol::new(77);
        let frame = encoder.payload(value).expect("payload");
        assert_eq!(
            ExactU64Binary64::decode([frame[1], frame[2]]).expect("payload value"),
            value.code()
        );
        assert!(frame[3..].iter().all(|coordinate| *coordinate == 0.0));
    }

    #[test]
    fn a0_namespaces_task_items_and_queries_without_targets() {
        let value = TaskSymbol::new(99);
        let association = a0_association_item(7, value);
        let payload = a0_payload_item(7, value);
        let distractor = a0_distractor_item(TaskSymbol::new(7));
        assert_eq!(association.key().len(), A0_TASK_KEY_WIDTH);
        assert_eq!(association.key(), a0_association_query_key(7));
        assert_eq!(payload.key(), a0_payload_query_key(7));
        assert_ne!(association.key()[0].to_bits(), payload.key()[0].to_bits());
        assert_ne!(
            association.key()[0].to_bits(),
            distractor.key()[0].to_bits()
        );
    }

    #[test]
    fn payload_cursor_reconstructs_generator_positions_from_call_order() {
        let instance = generate_t2(29, T2Config::new(3, 4).expect("config")).expect("T2");
        let mut cursor = PayloadKeyCursor::default();
        let mut observed = Vec::new();
        for event in instance.events() {
            if let TaskEvent::Payload { position, .. } = *event {
                assert_eq!(cursor.next_position(), position);
                observed.push(cursor.next_write_key().expect("payload key"));
            }
        }
        let expected: Vec<_> = (0..3).map(payload_memory_key).collect();
        assert_eq!(observed, expected);
    }

    #[test]
    fn distractor_read_key_is_outside_complete_logical_write_set() {
        let instance = generate_t1(31, T1Config::new(7, 5, 3).expect("config")).expect("T1");
        let distractor = distractor_read_key_for_instance(&instance).expect("distractor key");
        for event in instance.events() {
            if let TaskEvent::Associate { key, .. } = *event {
                assert_ne!(distractor, key.code());
            }
        }
    }

    #[test]
    fn projection_audit_separates_generator_classes_from_physical_collisions() {
        let instance = generate_t3(41, T3Config::new(8, 3, 4, 20, 3).expect("config")).expect("T3");
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

        let wide_audit = audit_associative_projection(&instance, &wide).expect("wide audit");
        let narrow_audit = audit_associative_projection(&instance, &narrow).expect("narrow audit");
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
}
