//! Deterministic bounded associative-memory reference semantics for TDI-8.1.
//!
//! This module implements one explicit direct-mapped reference table. It is a
//! bounded software oracle for address/write/read/collision/replacement
//! behavior; it does not freeze TDI-8.1 experimental dimensions or establish a
//! scientific advantage for ASSR.

use core::fmt;

use crate::StorageBits;

const OCCUPANCY_BITS_PER_SLOT: u128 = 8;
const TAG_BITS_PER_SLOT: u128 = 64;
const LAYOUT_METADATA_BITS: u128 = 128;
const PROJECTION_STATIC_BITS: u128 = 256;

/// Logical bounded-table layout used by the deterministic reference memory.
///
/// Slot count and payload width are represented as `u64` so the declared
/// metadata width is platform independent. Runtime allocation additionally
/// requires both dimensions to fit the host `usize` representation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AssociativeMemoryLayout {
    slot_count: u64,
    payload_width: u64,
}

impl AssociativeMemoryLayout {
    /// Construct a non-empty bounded associative-memory layout.
    pub fn new(slot_count: u64, payload_width: u64) -> Result<Self, AssociativeMemoryError> {
        if slot_count == 0 {
            return Err(AssociativeMemoryError::ZeroSlots);
        }
        if payload_width == 0 {
            return Err(AssociativeMemoryError::ZeroPayloadWidth);
        }
        Ok(Self {
            slot_count,
            payload_width,
        })
    }

    /// Number of direct-mapped slots.
    #[must_use]
    pub const fn slot_count(self) -> u64 {
        self.slot_count
    }

    /// Number of binary64 values stored in each payload.
    #[must_use]
    pub const fn payload_width(self) -> u64 {
        self.payload_width
    }

    fn host_dimensions(self) -> Result<(usize, usize), AssociativeMemoryError> {
        let slots = usize::try_from(self.slot_count).map_err(|_| {
            AssociativeMemoryError::HostDimensionTooLarge {
                component: "slot_count",
                value: self.slot_count,
            }
        })?;
        let width = usize::try_from(self.payload_width).map_err(|_| {
            AssociativeMemoryError::HostDimensionTooLarge {
                component: "payload_width",
                value: self.payload_width,
            }
        })?;
        slots
            .checked_mul(width)
            .ok_or(AssociativeMemoryError::HostPayloadLengthOverflow)?;
        Ok((slots, width))
    }

    fn storage_accounting(self) -> Result<AssociativeStorageAccounting, AssociativeMemoryError> {
        let slots = u128::from(self.slot_count);
        let width = u128::from(self.payload_width);

        let payload_values = slots
            .checked_mul(width)
            .ok_or(AssociativeMemoryError::AccountingOverflow)?;
        let payload_bits = payload_values
            .checked_mul(64)
            .ok_or(AssociativeMemoryError::AccountingOverflow)?;

        let per_slot_metadata = OCCUPANCY_BITS_PER_SLOT
            .checked_add(TAG_BITS_PER_SLOT)
            .ok_or(AssociativeMemoryError::AccountingOverflow)?;
        let table_metadata = slots
            .checked_mul(per_slot_metadata)
            .ok_or(AssociativeMemoryError::AccountingOverflow)?;
        let metadata_bits = table_metadata
            .checked_add(LAYOUT_METADATA_BITS)
            .ok_or(AssociativeMemoryError::AccountingOverflow)?;

        Ok(AssociativeStorageAccounting {
            payload_bits: StorageBits::new(payload_bits),
            metadata_bits: StorageBits::new(metadata_bits),
            static_parameter_bits: StorageBits::new(PROJECTION_STATIC_BITS),
        })
    }
}

/// Exact declared storage split for the associative-memory primitive.
///
/// The payload and metadata quantities are the architecture-state accounting
/// consumed by an A2/A3 memory table. Static projection constants are reported
/// separately. Rust allocator/container headers are implementation overhead and
/// are deliberately not converted into an architecture-level memory claim.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AssociativeStorageAccounting {
    payload_bits: StorageBits,
    metadata_bits: StorageBits,
    static_parameter_bits: StorageBits,
}

impl AssociativeStorageAccounting {
    /// Binary64 payload storage across all slots.
    #[must_use]
    pub const fn payload_bits(self) -> StorageBits {
        self.payload_bits
    }

    /// Occupancy, key-tag and explicit layout metadata.
    #[must_use]
    pub const fn metadata_bits(self) -> StorageBits {
        self.metadata_bits
    }

    /// Address-projection seed and fixed integer-mixing constants.
    #[must_use]
    pub const fn static_parameter_bits(self) -> StorageBits {
        self.static_parameter_bits
    }
}

/// Fail-closed errors from the bounded reference associative memory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssociativeMemoryError {
    /// A bounded memory must expose at least one addressable slot.
    ZeroSlots,
    /// Each slot must carry a non-empty payload.
    ZeroPayloadWidth,
    /// A declared platform-independent dimension cannot be represented by the
    /// current host index type.
    HostDimensionTooLarge {
        /// Name of the offending dimension.
        component: &'static str,
        /// Declared dimension value.
        value: u64,
    },
    /// `slot_count * payload_width` cannot be represented as a host allocation
    /// length.
    HostPayloadLengthOverflow,
    /// Exact bit accounting overflowed `u128`.
    AccountingOverflow,
    /// A write payload did not match the frozen table width.
    PayloadWidthMismatch {
        /// Required payload width.
        expected: usize,
        /// Supplied payload width.
        actual: usize,
    },
    /// A write payload contained a non-finite binary64 value.
    NonFinitePayload {
        /// Index of the invalid payload element.
        index: usize,
    },
}

impl fmt::Display for AssociativeMemoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroSlots => formatter.write_str("associative memory requires at least one slot"),
            Self::ZeroPayloadWidth => {
                formatter.write_str("associative memory requires a non-zero payload width")
            }
            Self::HostDimensionTooLarge { component, value } => {
                write!(
                    formatter,
                    "{component}={value} does not fit the host index type"
                )
            }
            Self::HostPayloadLengthOverflow => {
                formatter.write_str("associative payload length overflows the host index type")
            }
            Self::AccountingOverflow => {
                formatter.write_str("associative-memory storage accounting overflow")
            }
            Self::PayloadWidthMismatch { expected, actual } => write!(
                formatter,
                "associative payload width mismatch: expected {expected}, got {actual}"
            ),
            Self::NonFinitePayload { index } => {
                write!(
                    formatter,
                    "associative payload element {index} is not finite"
                )
            }
        }
    }
}

impl std::error::Error for AssociativeMemoryError {}

/// Deterministic result of writing one key/payload association.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssociativeWriteOutcome {
    /// The projected slot was empty.
    Inserted {
        /// Direct-mapped slot index.
        address: u64,
    },
    /// The same key already occupied the projected slot and its payload was
    /// replaced in place.
    Updated {
        /// Direct-mapped slot index.
        address: u64,
    },
    /// A different key occupied the projected slot and was deterministically
    /// evicted by the new association.
    ReplacedCollision {
        /// Direct-mapped slot index.
        address: u64,
        /// Key tag that was evicted.
        evicted_key: u64,
    },
}

/// Deterministic result of reading one key from the bounded table.
#[derive(Clone, Debug, PartialEq)]
pub enum AssociativeRead<'a> {
    /// The requested key is resident and the stored payload is returned.
    Hit {
        /// Direct-mapped slot index.
        address: u64,
        /// Borrowed binary64 payload.
        payload: &'a [f64],
    },
    /// The projected slot has never been populated or has been explicitly
    /// cleared.
    Empty {
        /// Direct-mapped slot index.
        address: u64,
    },
    /// The projected slot is occupied by a different key tag.
    CollisionMiss {
        /// Direct-mapped slot index.
        address: u64,
        /// Key currently resident at that address.
        resident_key: u64,
    },
}

/// Bounded direct-mapped associative memory with deterministic integer address
/// projection and replacement.
///
/// Address projection is `splitmix64(key + seed) mod slot_count`. A collision
/// replaces the resident entry at that single address. Reads never mutate the
/// table. This simple policy is intentionally explicit and reproducible; later
/// TDI-8.1 work may compare or replace it only through a reviewed bounded
/// protocol before any TDI-8.2 surface exists.
#[derive(Clone, Debug, PartialEq)]
pub struct DirectMappedAssociativeMemory {
    layout: AssociativeMemoryLayout,
    projection_seed: u64,
    occupied: Vec<u8>,
    tags: Vec<u64>,
    payloads: Vec<f64>,
}

impl DirectMappedAssociativeMemory {
    /// Allocate an empty bounded table.
    pub fn new(
        layout: AssociativeMemoryLayout,
        projection_seed: u64,
    ) -> Result<Self, AssociativeMemoryError> {
        let (slots, width) = layout.host_dimensions()?;
        let payload_len = slots
            .checked_mul(width)
            .ok_or(AssociativeMemoryError::HostPayloadLengthOverflow)?;
        Ok(Self {
            layout,
            projection_seed,
            occupied: vec![0; slots],
            tags: vec![0; slots],
            payloads: vec![0.0; payload_len],
        })
    }

    /// Declared bounded table layout.
    #[must_use]
    pub const fn layout(&self) -> AssociativeMemoryLayout {
        self.layout
    }

    /// Seed used by the deterministic integer address projection.
    #[must_use]
    pub const fn projection_seed(&self) -> u64 {
        self.projection_seed
    }

    /// Exact declared storage split for this table representation.
    pub fn storage_accounting(
        &self,
    ) -> Result<AssociativeStorageAccounting, AssociativeMemoryError> {
        self.layout.storage_accounting()
    }

    /// Deterministically project a key to one bounded slot.
    #[must_use]
    pub fn address_for(&self, key: u64) -> u64 {
        let mixed = splitmix64(key.wrapping_add(self.projection_seed));
        mixed % self.layout.slot_count
    }

    /// Read an association without mutating replacement state.
    #[must_use]
    pub fn read(&self, key: u64) -> AssociativeRead<'_> {
        let address = self.address_for(key);
        let index = usize::try_from(address).expect("address is bounded by allocated slot count");
        if self.occupied[index] == 0 {
            return AssociativeRead::Empty { address };
        }
        let resident_key = self.tags[index];
        if resident_key != key {
            return AssociativeRead::CollisionMiss {
                address,
                resident_key,
            };
        }

        let width = usize::try_from(self.layout.payload_width)
            .expect("validated payload width must fit host index type");
        let start = index
            .checked_mul(width)
            .expect("validated table dimensions must fit host index type");
        AssociativeRead::Hit {
            address,
            payload: &self.payloads[start..start + width],
        }
    }

    /// Write an association using deterministic direct-mapped replacement.
    ///
    /// The full payload is validated before any table state is modified, so a
    /// rejected write cannot partially mutate the reference memory.
    pub fn write(
        &mut self,
        key: u64,
        payload: &[f64],
    ) -> Result<AssociativeWriteOutcome, AssociativeMemoryError> {
        let width = usize::try_from(self.layout.payload_width)
            .expect("validated payload width must fit host index type");
        if payload.len() != width {
            return Err(AssociativeMemoryError::PayloadWidthMismatch {
                expected: width,
                actual: payload.len(),
            });
        }
        if let Some(index) = payload.iter().position(|value| !value.is_finite()) {
            return Err(AssociativeMemoryError::NonFinitePayload { index });
        }

        let address = self.address_for(key);
        let index = usize::try_from(address).expect("address is bounded by allocated slot count");
        let outcome = if self.occupied[index] == 0 {
            AssociativeWriteOutcome::Inserted { address }
        } else if self.tags[index] == key {
            AssociativeWriteOutcome::Updated { address }
        } else {
            AssociativeWriteOutcome::ReplacedCollision {
                address,
                evicted_key: self.tags[index],
            }
        };

        let start = index
            .checked_mul(width)
            .expect("validated table dimensions must fit host index type");
        self.payloads[start..start + width].copy_from_slice(payload);
        self.tags[index] = key;
        self.occupied[index] = 1;
        Ok(outcome)
    }

    /// Remove all resident associations while preserving capacity and the
    /// projection seed.
    pub fn clear(&mut self) {
        self.occupied.fill(0);
        self.tags.fill(0);
        self.payloads.fill(0.0);
    }
}

/// Stable SplitMix64 integer mixer used only for bounded reference addressing.
#[must_use]
fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::{
        AssociativeMemoryError, AssociativeMemoryLayout, AssociativeRead, AssociativeWriteOutcome,
        DirectMappedAssociativeMemory,
    };
    use crate::StorageBits;

    fn memory(slot_count: u64, payload_width: u64) -> DirectMappedAssociativeMemory {
        let layout = AssociativeMemoryLayout::new(slot_count, payload_width).expect("valid layout");
        DirectMappedAssociativeMemory::new(layout, 0x1234_5678_9abc_def0).expect("bounded memory")
    }

    #[test]
    fn layout_rejects_degenerate_memory() {
        assert_eq!(
            AssociativeMemoryLayout::new(0, 1),
            Err(AssociativeMemoryError::ZeroSlots)
        );
        assert_eq!(
            AssociativeMemoryLayout::new(1, 0),
            Err(AssociativeMemoryError::ZeroPayloadWidth)
        );
    }

    #[test]
    fn address_projection_is_deterministic_and_bounded() {
        let table = memory(7, 2);
        let first = table.address_for(42);
        assert_eq!(first, table.address_for(42));
        assert!(first < 7);
        for key in 0..1024 {
            assert!(table.address_for(key) < 7);
        }
    }

    #[test]
    fn empty_insert_and_hit_have_explicit_semantics() {
        let mut table = memory(8, 2);
        let address = table.address_for(17);
        assert_eq!(table.read(17), AssociativeRead::Empty { address });
        assert_eq!(
            table.write(17, &[1.5, -2.0]).expect("finite write"),
            AssociativeWriteOutcome::Inserted { address }
        );
        assert_eq!(
            table.read(17),
            AssociativeRead::Hit {
                address,
                payload: &[1.5, -2.0],
            }
        );
    }

    #[test]
    fn same_key_update_replaces_payload_in_place() {
        let mut table = memory(4, 2);
        table.write(9, &[1.0, 2.0]).expect("first write");
        let address = table.address_for(9);
        assert_eq!(
            table.write(9, &[3.0, 4.0]).expect("update"),
            AssociativeWriteOutcome::Updated { address }
        );
        assert_eq!(
            table.read(9),
            AssociativeRead::Hit {
                address,
                payload: &[3.0, 4.0],
            }
        );
    }

    #[test]
    fn collision_is_observable_and_replacement_is_deterministic() {
        let mut table = memory(1, 1);
        assert_eq!(
            table.write(10, &[1.0]).expect("first write"),
            AssociativeWriteOutcome::Inserted { address: 0 }
        );
        assert_eq!(
            table.write(20, &[2.0]).expect("colliding write"),
            AssociativeWriteOutcome::ReplacedCollision {
                address: 0,
                evicted_key: 10,
            }
        );
        assert_eq!(
            table.read(10),
            AssociativeRead::CollisionMiss {
                address: 0,
                resident_key: 20,
            }
        );
        assert_eq!(
            table.read(20),
            AssociativeRead::Hit {
                address: 0,
                payload: &[2.0],
            }
        );
    }

    #[test]
    fn rejected_write_cannot_partially_mutate_table() {
        let mut table = memory(1, 2);
        table.write(3, &[5.0, 6.0]).expect("seed state");
        assert_eq!(
            table.write(4, &[7.0, f64::NAN]),
            Err(AssociativeMemoryError::NonFinitePayload { index: 1 })
        );
        assert_eq!(
            table.read(3),
            AssociativeRead::Hit {
                address: 0,
                payload: &[5.0, 6.0],
            }
        );
    }

    #[test]
    fn payload_width_mismatch_fails_closed() {
        let mut table = memory(2, 2);
        assert_eq!(
            table.write(1, &[1.0]),
            Err(AssociativeMemoryError::PayloadWidthMismatch {
                expected: 2,
                actual: 1,
            })
        );
    }

    #[test]
    fn declared_storage_accounting_includes_payload_metadata_and_projection_constants() {
        let table = memory(4, 3);
        let accounting = table.storage_accounting().expect("exact accounting");
        assert_eq!(accounting.payload_bits(), StorageBits::new(768));
        assert_eq!(accounting.metadata_bits(), StorageBits::new(416));
        assert_eq!(accounting.static_parameter_bits(), StorageBits::new(256));
    }

    #[test]
    fn clear_removes_residency_without_changing_projection() {
        let mut table = memory(3, 1);
        let address = table.address_for(99);
        table.write(99, &[8.0]).expect("write");
        table.clear();
        assert_eq!(table.address_for(99), address);
        assert_eq!(table.read(99), AssociativeRead::Empty { address });
    }
}
