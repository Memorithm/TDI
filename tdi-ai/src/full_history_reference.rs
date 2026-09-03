//! Deterministic full-history A0 reference semantics for TDI-8.1.
//!
//! A0 is the contextual competent full-history control from the frozen TDI-8.0
//! preregistration. It retains every accessible key/value history item and uses
//! deterministic hard content attention for readout. This module does not freeze
//! experimental dimensions, horizons, task encodings or any TDI-8.2 surface.

use core::{fmt, mem};

use crate::{
    MemoryAccounting, MemoryAccountingError, ReferenceArm, ReferenceSnapshot, StorageBits,
};

const HISTORY_COUNT_METADATA_BITS: u128 = 64;
const LAYOUT_STATIC_BITS: u128 = 128;
const READOUT_SCALAR_TEMP_BITS: u128 = 128;

/// Platform-independent full-history key/value layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FullHistoryLayout {
    key_width: u64,
    value_width: u64,
}

impl FullHistoryLayout {
    /// Construct a non-empty key/value layout.
    pub fn new(key_width: u64, value_width: u64) -> Result<Self, A0ReferenceError> {
        if key_width == 0 {
            return Err(A0ReferenceError::ZeroKeyWidth);
        }
        if value_width == 0 {
            return Err(A0ReferenceError::ZeroValueWidth);
        }
        Ok(Self {
            key_width,
            value_width,
        })
    }

    /// Number of binary64 coordinates in one content key/query.
    #[must_use]
    pub const fn key_width(self) -> u64 {
        self.key_width
    }

    /// Number of binary64 coordinates in one stored/read value.
    #[must_use]
    pub const fn value_width(self) -> u64 {
        self.value_width
    }

    fn host_widths(self) -> Result<(usize, usize), A0ReferenceError> {
        let key_width = usize::try_from(self.key_width).map_err(|_| {
            A0ReferenceError::HostDimensionTooLarge {
                component: "key_width",
                value: self.key_width,
            }
        })?;
        let value_width = usize::try_from(self.value_width).map_err(|_| {
            A0ReferenceError::HostDimensionTooLarge {
                component: "value_width",
                value: self.value_width,
            }
        })?;
        validate_vector_capacity("key", key_width, mem::size_of::<f64>())?;
        validate_vector_capacity("value", value_width, mem::size_of::<f64>())?;
        Ok((key_width, value_width))
    }
}

/// Fail-closed errors from the deterministic A0 full-history reference.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum A0ReferenceError {
    /// Content keys/queries must have at least one coordinate.
    ZeroKeyWidth,
    /// Stored/read values must have at least one coordinate.
    ZeroValueWidth,
    /// A platform-independent width cannot be represented by the host index type.
    HostDimensionTooLarge {
        /// Name of the offending dimension.
        component: &'static str,
        /// Declared dimension value.
        value: u64,
    },
    /// A required host vector would exceed the representable `Vec` byte capacity.
    HostVectorCapacityTooLarge {
        /// Logical vector component.
        component: &'static str,
        /// Requested number of elements.
        elements: usize,
        /// Size of one element in bytes.
        element_bytes: usize,
    },
    /// The host allocator rejected a validated reservation.
    HostAllocationFailed {
        /// Logical vector component.
        component: &'static str,
        /// Requested additional or total elements.
        elements: usize,
    },
    /// Extending the flat complete-history arrays overflowed host indexing.
    HostHistoryLengthOverflow {
        /// History component that overflowed.
        component: &'static str,
    },
    /// More than `u64::MAX` history items would be required.
    HistoryItemCountOverflow,
    /// A history key has the wrong width.
    KeyWidthMismatch {
        /// Required key width.
        expected: usize,
        /// Supplied key width.
        actual: usize,
    },
    /// A stored value has the wrong width.
    ValueWidthMismatch {
        /// Required value width.
        expected: usize,
        /// Supplied value width.
        actual: usize,
    },
    /// A query has the wrong width.
    QueryWidthMismatch {
        /// Required query width.
        expected: usize,
        /// Supplied query width.
        actual: usize,
    },
    /// A key contains a non-finite binary64 coordinate.
    NonFiniteKey {
        /// Invalid key coordinate.
        index: usize,
    },
    /// A value contains a non-finite binary64 coordinate.
    NonFiniteValue {
        /// Invalid value coordinate.
        index: usize,
    },
    /// A query contains a non-finite binary64 coordinate.
    NonFiniteQuery {
        /// Invalid query coordinate.
        index: usize,
    },
    /// Fixed-order squared-L2 scoring produced a non-finite intermediate.
    NonFiniteDistance {
        /// History item being scored.
        item_index: u64,
        /// Key/query coordinate that produced the invalid intermediate.
        coordinate: usize,
    },
    /// A content read was requested before any accessible history existed.
    EmptyHistory,
    /// Exact A0 storage accounting overflowed `u128`.
    AccountingOverflow,
    /// Common TDI-8 memory-accounting validation failed.
    MemoryAccounting(MemoryAccountingError),
}

impl fmt::Display for A0ReferenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroKeyWidth => {
                formatter.write_str("A0 full history requires a non-zero key width")
            }
            Self::ZeroValueWidth => {
                formatter.write_str("A0 full history requires a non-zero value width")
            }
            Self::HostDimensionTooLarge { component, value } => write!(
                formatter,
                "A0 {component}={value} does not fit the host index type"
            ),
            Self::HostVectorCapacityTooLarge {
                component,
                elements,
                element_bytes,
            } => write!(
                formatter,
                "A0 {component} vector capacity is too large: {elements} elements × {element_bytes} bytes"
            ),
            Self::HostAllocationFailed {
                component,
                elements,
            } => write!(
                formatter,
                "host allocation failed for A0 {component}: {elements} elements"
            ),
            Self::HostHistoryLengthOverflow { component } => {
                write!(formatter, "A0 {component} history length overflow")
            }
            Self::HistoryItemCountOverflow => {
                formatter.write_str("A0 history item count exceeds u64::MAX")
            }
            Self::KeyWidthMismatch { expected, actual } => write!(
                formatter,
                "A0 key width mismatch: expected {expected}, got {actual}"
            ),
            Self::ValueWidthMismatch { expected, actual } => write!(
                formatter,
                "A0 value width mismatch: expected {expected}, got {actual}"
            ),
            Self::QueryWidthMismatch { expected, actual } => write!(
                formatter,
                "A0 query width mismatch: expected {expected}, got {actual}"
            ),
            Self::NonFiniteKey { index } => {
                write!(formatter, "A0 key coordinate {index} is not finite")
            }
            Self::NonFiniteValue { index } => {
                write!(formatter, "A0 value coordinate {index} is not finite")
            }
            Self::NonFiniteQuery { index } => {
                write!(formatter, "A0 query coordinate {index} is not finite")
            }
            Self::NonFiniteDistance {
                item_index,
                coordinate,
            } => write!(
                formatter,
                "A0 content distance became non-finite at item {item_index}, coordinate {coordinate}"
            ),
            Self::EmptyHistory => formatter.write_str("A0 content read requires non-empty history"),
            Self::AccountingOverflow => formatter.write_str("A0 storage accounting overflow"),
            Self::MemoryAccounting(error) => write!(formatter, "memory accounting: {error}"),
        }
    }
}

impl std::error::Error for A0ReferenceError {}

impl From<MemoryAccountingError> for A0ReferenceError {
    fn from(error: MemoryAccountingError) -> Self {
        Self::MemoryAccounting(error)
    }
}

fn validate_vector_capacity(
    component: &'static str,
    elements: usize,
    element_bytes: usize,
) -> Result<(), A0ReferenceError> {
    let bytes = elements.checked_mul(element_bytes).ok_or(
        A0ReferenceError::HostVectorCapacityTooLarge {
            component,
            elements,
            element_bytes,
        },
    )?;
    if bytes > isize::MAX as usize {
        return Err(A0ReferenceError::HostVectorCapacityTooLarge {
            component,
            elements,
            element_bytes,
        });
    }
    Ok(())
}

fn allocate_zeroed(component: &'static str, elements: usize) -> Result<Vec<f64>, A0ReferenceError> {
    validate_vector_capacity(component, elements, mem::size_of::<f64>())?;
    let mut values = Vec::new();
    values
        .try_reserve_exact(elements)
        .map_err(|_| A0ReferenceError::HostAllocationFailed {
            component,
            elements,
        })?;
    values.resize(elements, 0.0);
    Ok(values)
}

fn clone_f64_slice(component: &'static str, values: &[f64]) -> Result<Vec<f64>, A0ReferenceError> {
    let mut cloned = Vec::new();
    validate_vector_capacity(component, values.len(), mem::size_of::<f64>())?;
    cloned
        .try_reserve_exact(values.len())
        .map_err(|_| A0ReferenceError::HostAllocationFailed {
            component,
            elements: values.len(),
        })?;
    cloned.extend_from_slice(values);
    Ok(cloned)
}

fn checked_add(left: u128, right: u128) -> Result<u128, A0ReferenceError> {
    left.checked_add(right)
        .ok_or(A0ReferenceError::AccountingOverflow)
}

fn checked_mul(left: u128, right: u128) -> Result<u128, A0ReferenceError> {
    left.checked_mul(right)
        .ok_or(A0ReferenceError::AccountingOverflow)
}

/// Complete persistent A0 state exposed through TDI snapshots.
#[derive(Clone, Debug, PartialEq)]
pub struct A0StateSnapshot {
    layout: FullHistoryLayout,
    item_count: u64,
    keys: Vec<f64>,
    values: Vec<f64>,
}

impl A0StateSnapshot {
    /// Frozen software-oracle key/value layout for this snapshot.
    #[must_use]
    pub const fn layout(&self) -> FullHistoryLayout {
        self.layout
    }

    /// Number of accessible history items.
    #[must_use]
    pub const fn item_count(&self) -> u64 {
        self.item_count
    }

    /// Flat insertion-ordered key coordinates.
    #[must_use]
    pub fn keys(&self) -> &[f64] {
        &self.keys
    }

    /// Flat insertion-ordered value coordinates.
    #[must_use]
    pub fn values(&self) -> &[f64] {
        &self.values
    }
}

/// Deterministic A0 hard-content-attention readout.
#[derive(Clone, Debug, PartialEq)]
pub struct A0Readout {
    selected_index: u64,
    squared_distance: f64,
    coefficients: Vec<f64>,
    value: Vec<f64>,
}

impl A0Readout {
    /// History item selected by the deterministic content read.
    #[must_use]
    pub const fn selected_index(&self) -> u64 {
        self.selected_index
    }

    /// Fixed-order squared-L2 distance for the selected key/query pair.
    #[must_use]
    pub const fn squared_distance(&self) -> f64 {
        self.squared_distance
    }

    /// One coefficient per accessible history item. Hard attention is one-hot.
    #[must_use]
    pub fn coefficients(&self) -> &[f64] {
        &self.coefficients
    }

    /// Value associated with the selected full-history item.
    #[must_use]
    pub fn value(&self) -> &[f64] {
        &self.value
    }
}

/// Competent deterministic A0 full-history reference.
///
/// Every appended key/value item remains accessible until [`Self::clear`]. A
/// content read scans the entire history in insertion order, computes squared
/// L2 distance in ascending coordinate order, and selects the smallest finite
/// distance. Exact distance ties select the most recently appended item. The
/// exposed read coefficients are therefore deterministic one-hot hard attention.
#[derive(Clone, Debug, PartialEq)]
pub struct A0Reference {
    layout: FullHistoryLayout,
    item_count: u64,
    keys: Vec<f64>,
    values: Vec<f64>,
}

impl A0Reference {
    /// Construct an empty full-history control without freezing dimensions.
    pub fn new(layout: FullHistoryLayout) -> Result<Self, A0ReferenceError> {
        let _ = layout.host_widths()?;
        Ok(Self {
            layout,
            item_count: 0,
            keys: Vec::new(),
            values: Vec::new(),
        })
    }

    /// Declared key/value layout.
    #[must_use]
    pub const fn layout(&self) -> FullHistoryLayout {
        self.layout
    }

    /// Number of accessible history items retained without truncation.
    #[must_use]
    pub const fn item_count(&self) -> u64 {
        self.item_count
    }

    /// Whether no accessible history item has yet been appended.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.item_count == 0
    }

    /// Borrow one insertion-ordered history item.
    #[must_use]
    pub fn history_item(&self, index: u64) -> Option<(&[f64], &[f64])> {
        if index >= self.item_count {
            return None;
        }
        let (key_width, value_width) = self.layout.host_widths().ok()?;
        let index = usize::try_from(index).ok()?;
        let key_start = index.checked_mul(key_width)?;
        let value_start = index.checked_mul(value_width)?;
        Some((
            &self.keys[key_start..key_start + key_width],
            &self.values[value_start..value_start + value_width],
        ))
    }

    /// Append one complete accessible history item.
    ///
    /// Validation and both reservations complete before either history payload
    /// is extended, so rejected values cannot leave a partially appended item.
    pub fn append(&mut self, key: &[f64], value: &[f64]) -> Result<u64, A0ReferenceError> {
        let (key_width, value_width) = self.layout.host_widths()?;
        if key.len() != key_width {
            return Err(A0ReferenceError::KeyWidthMismatch {
                expected: key_width,
                actual: key.len(),
            });
        }
        if value.len() != value_width {
            return Err(A0ReferenceError::ValueWidthMismatch {
                expected: value_width,
                actual: value.len(),
            });
        }
        if let Some(index) = key.iter().position(|coordinate| !coordinate.is_finite()) {
            return Err(A0ReferenceError::NonFiniteKey { index });
        }
        if let Some(index) = value.iter().position(|coordinate| !coordinate.is_finite()) {
            return Err(A0ReferenceError::NonFiniteValue { index });
        }
        if self.item_count == u64::MAX {
            return Err(A0ReferenceError::HistoryItemCountOverflow);
        }

        let next_key_len = self
            .keys
            .len()
            .checked_add(key_width)
            .ok_or(A0ReferenceError::HostHistoryLengthOverflow { component: "key" })?;
        let next_value_len = self
            .values
            .len()
            .checked_add(value_width)
            .ok_or(A0ReferenceError::HostHistoryLengthOverflow { component: "value" })?;
        validate_vector_capacity("key_history", next_key_len, mem::size_of::<f64>())?;
        validate_vector_capacity("value_history", next_value_len, mem::size_of::<f64>())?;

        self.keys.try_reserve_exact(key_width).map_err(|_| {
            A0ReferenceError::HostAllocationFailed {
                component: "key_history",
                elements: key_width,
            }
        })?;
        self.values.try_reserve_exact(value_width).map_err(|_| {
            A0ReferenceError::HostAllocationFailed {
                component: "value_history",
                elements: value_width,
            }
        })?;

        let index = self.item_count;
        self.keys.extend_from_slice(key);
        self.values.extend_from_slice(value);
        self.item_count += 1;
        Ok(index)
    }

    /// Read the complete accessible history with deterministic hard content
    /// attention.
    pub fn read(&self, query: &[f64]) -> Result<A0Readout, A0ReferenceError> {
        let (key_width, value_width) = self.layout.host_widths()?;
        if query.len() != key_width {
            return Err(A0ReferenceError::QueryWidthMismatch {
                expected: key_width,
                actual: query.len(),
            });
        }
        if let Some(index) = query.iter().position(|coordinate| !coordinate.is_finite()) {
            return Err(A0ReferenceError::NonFiniteQuery { index });
        }
        if self.item_count == 0 {
            return Err(A0ReferenceError::EmptyHistory);
        }

        let item_count = usize::try_from(self.item_count)
            .map_err(|_| A0ReferenceError::HistoryItemCountOverflow)?;
        let mut best_index = 0usize;
        let mut best_distance = f64::INFINITY;

        for item_index in 0..item_count {
            let key_start = item_index
                .checked_mul(key_width)
                .ok_or(A0ReferenceError::HostHistoryLengthOverflow { component: "key" })?;
            let key = &self.keys[key_start..key_start + key_width];
            let mut distance = 0.0f64;
            for (coordinate, (query_value, key_value)) in query.iter().zip(key.iter()).enumerate() {
                let difference = *query_value - *key_value;
                let squared = difference * difference;
                if !difference.is_finite() || !squared.is_finite() {
                    return Err(A0ReferenceError::NonFiniteDistance {
                        item_index: u64::try_from(item_index)
                            .map_err(|_| A0ReferenceError::HistoryItemCountOverflow)?,
                        coordinate,
                    });
                }
                distance += squared;
                if !distance.is_finite() {
                    return Err(A0ReferenceError::NonFiniteDistance {
                        item_index: u64::try_from(item_index)
                            .map_err(|_| A0ReferenceError::HistoryItemCountOverflow)?,
                        coordinate,
                    });
                }
            }

            if distance <= best_distance {
                best_distance = distance;
                best_index = item_index;
            }
        }

        let mut coefficients = allocate_zeroed("read_coefficients", item_count)?;
        coefficients[best_index] = 1.0;
        let value_start = best_index
            .checked_mul(value_width)
            .ok_or(A0ReferenceError::HostHistoryLengthOverflow { component: "value" })?;
        let mut value = allocate_zeroed("read_value", value_width)?;
        value.copy_from_slice(&self.values[value_start..value_start + value_width]);

        Ok(A0Readout {
            selected_index: u64::try_from(best_index)
                .map_err(|_| A0ReferenceError::HistoryItemCountOverflow)?,
            squared_distance: best_distance,
            coefficients,
            value,
        })
    }

    /// Clear all accessible history while retaining the declared layout.
    pub fn clear(&mut self) {
        self.keys.clear();
        self.values.clear();
        self.item_count = 0;
    }

    /// Exact architecture-level accounting for the current full-history state.
    pub fn memory_accounting(&self) -> Result<MemoryAccounting, A0ReferenceError> {
        let item_count = u128::from(self.item_count);
        let key_width = u128::from(self.layout.key_width());
        let value_width = u128::from(self.layout.value_width());
        let coordinates_per_item = checked_add(key_width, value_width)?;
        let history_coordinates = checked_mul(item_count, coordinates_per_item)?;
        let history_payload_bits = checked_mul(history_coordinates, 64)?;
        let cumulative_history = checked_add(history_payload_bits, HISTORY_COUNT_METADATA_BITS)?;

        let temporary_working = if self.item_count == 0 {
            0
        } else {
            let readout_coordinates = checked_add(item_count, value_width)?;
            let readout_bits = checked_mul(readout_coordinates, 64)?;
            checked_add(readout_bits, READOUT_SCALAR_TEMP_BITS)?
        };

        let accounting = MemoryAccounting::zero()
            .with_temporary_working(StorageBits::new(temporary_working))
            .with_cumulative_history(StorageBits::new(cumulative_history))
            .with_static_parameters(StorageBits::new(LAYOUT_STATIC_BITS));
        accounting.validate_for_arm(ReferenceArm::A0)?;
        Ok(accounting)
    }

    /// Clone a complete A0 persistent-state snapshot with exact current
    /// accounting.
    pub fn snapshot(&self) -> Result<ReferenceSnapshot<A0StateSnapshot>, A0ReferenceError> {
        let state = A0StateSnapshot {
            layout: self.layout,
            item_count: self.item_count,
            keys: clone_f64_slice("snapshot_keys", &self.keys)?,
            values: clone_f64_slice("snapshot_values", &self.values)?,
        };
        Ok(ReferenceSnapshot::new(
            ReferenceArm::A0,
            state,
            self.memory_accounting()?,
        )?)
    }
}

#[cfg(test)]
mod tests {
    use super::{A0Reference, A0ReferenceError, FullHistoryLayout};
    use crate::ReferenceArm;

    fn model() -> A0Reference {
        A0Reference::new(FullHistoryLayout::new(2, 2).expect("layout")).expect("A0")
    }

    #[test]
    fn layout_rejects_zero_widths() {
        assert_eq!(
            FullHistoryLayout::new(0, 1),
            Err(A0ReferenceError::ZeroKeyWidth)
        );
        assert_eq!(
            FullHistoryLayout::new(1, 0),
            Err(A0ReferenceError::ZeroValueWidth)
        );
    }

    #[test]
    fn append_retains_complete_history_in_order() {
        let mut a0 = model();
        assert_eq!(a0.append(&[1.0, 0.0], &[10.0, 11.0]).expect("append"), 0);
        assert_eq!(a0.append(&[0.0, 1.0], &[20.0, 21.0]).expect("append"), 1);
        assert_eq!(a0.item_count(), 2);
        assert_eq!(
            a0.history_item(0),
            Some((&[1.0, 0.0][..], &[10.0, 11.0][..]))
        );
        assert_eq!(
            a0.history_item(1),
            Some((&[0.0, 1.0][..], &[20.0, 21.0][..]))
        );
    }

    #[test]
    fn rejected_append_cannot_partially_mutate_history() {
        let mut a0 = model();
        a0.append(&[1.0, 0.0], &[10.0, 11.0])
            .expect("seed history");
        let before = a0.snapshot().expect("snapshot");
        assert_eq!(
            a0.append(&[0.0, f64::NAN], &[20.0, 21.0]),
            Err(A0ReferenceError::NonFiniteKey { index: 1 })
        );
        let after = a0.snapshot().expect("snapshot");
        assert_eq!(before.state(), after.state());
    }

    #[test]
    fn hard_content_read_selects_nearest_complete_history_item() {
        let mut a0 = model();
        a0.append(&[1.0, 0.0], &[10.0, 11.0]).expect("append");
        a0.append(&[0.0, 1.0], &[20.0, 21.0]).expect("append");

        let readout = a0.read(&[0.1, 0.9]).expect("read");
        assert_eq!(readout.selected_index(), 1);
        assert_eq!(readout.value(), &[20.0, 21.0]);
        assert_eq!(readout.coefficients(), &[0.0, 1.0]);
        assert!((readout.squared_distance() - 0.02).abs() < 1e-15);
    }

    #[test]
    fn exact_distance_ties_select_the_most_recent_item() {
        let mut a0 = model();
        a0.append(&[1.0, 0.0], &[10.0, 11.0]).expect("append");
        a0.append(&[1.0, 0.0], &[20.0, 21.0]).expect("append");

        let readout = a0.read(&[1.0, 0.0]).expect("read");
        assert_eq!(readout.selected_index(), 1);
        assert_eq!(readout.value(), &[20.0, 21.0]);
        assert_eq!(readout.coefficients(), &[0.0, 1.0]);
        assert_eq!(readout.squared_distance(), 0.0);
    }

    #[test]
    fn read_rejects_empty_or_non_finite_inputs_without_mutation() {
        let mut a0 = model();
        assert_eq!(a0.read(&[0.0, 0.0]), Err(A0ReferenceError::EmptyHistory));
        a0.append(&[1.0, 0.0], &[10.0, 11.0]).expect("append");
        let before = a0.snapshot().expect("snapshot");
        assert_eq!(
            a0.read(&[f64::INFINITY, 0.0]),
            Err(A0ReferenceError::NonFiniteQuery { index: 0 })
        );
        let after = a0.snapshot().expect("snapshot");
        assert_eq!(before.state(), after.state());
    }

    #[test]
    fn fixed_order_distance_fails_closed_on_non_finite_intermediate() {
        let layout = FullHistoryLayout::new(1, 1).expect("layout");
        let mut a0 = A0Reference::new(layout).expect("A0");
        a0.append(&[-1.0e308], &[1.0]).expect("append");
        assert_eq!(
            a0.read(&[1.0e308]),
            Err(A0ReferenceError::NonFiniteDistance {
                item_index: 0,
                coordinate: 0,
            })
        );
    }

    #[test]
    fn accounting_grows_with_history_and_reports_peak_read_temporaries() {
        let mut a0 = model();
        let empty = a0.memory_accounting().expect("empty accounting");
        assert_eq!(empty.cumulative_history().get(), 64);
        assert_eq!(empty.temporary_working().get(), 0);
        assert_eq!(empty.static_parameters().get(), 128);

        a0.append(&[1.0, 0.0], &[10.0, 11.0]).expect("append");
        a0.append(&[0.0, 1.0], &[20.0, 21.0]).expect("append");
        let memory = a0.memory_accounting().expect("accounting");
        assert_eq!(memory.cumulative_history().get(), 64 + 2 * 4 * 64);
        assert_eq!(memory.temporary_working().get(), (2 + 2) * 64 + 128);
        assert_eq!(memory.static_parameters().get(), 128);
        memory
            .validate_for_arm(ReferenceArm::A0)
            .expect("valid A0 accounting");
    }

    #[test]
    fn snapshot_and_clear_cover_the_complete_persistent_history() {
        let mut a0 = model();
        a0.append(&[1.0, 0.0], &[10.0, 11.0]).expect("append");
        a0.append(&[0.0, 1.0], &[20.0, 21.0]).expect("append");
        let snapshot = a0.snapshot().expect("snapshot");
        assert_eq!(snapshot.arm(), ReferenceArm::A0);
        assert_eq!(snapshot.state().layout(), a0.layout());
        assert_eq!(snapshot.state().item_count(), 2);
        assert_eq!(snapshot.state().keys(), &[1.0, 0.0, 0.0, 1.0]);
        assert_eq!(snapshot.state().values(), &[10.0, 11.0, 20.0, 21.0]);

        let layout = a0.layout();
        a0.clear();
        assert_eq!(a0.layout(), layout);
        assert_eq!(a0.item_count(), 0);
        assert!(a0.is_empty());
    }
}
