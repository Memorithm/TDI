//! Deterministic bounded VSA/holographic workspace reference for TDI-8.1.
//!
//! The primitive implements a deliberately small subset of the VSA operations
//! permitted by the frozen TDI-8.0 preregistration: bipolar binding, additive
//! bundling/superposition, unbinding/retrieval, and fixed-order dot similarity.
//! It is a software oracle only. It does not freeze TDI-8.1 experimental
//! dimensions, establish A3 semantics by itself, or create any TDI-8.2 surface.

use core::{fmt, mem};

use crate::StorageBits;

const BITS_PER_F64: u128 = 64;
const LAYOUT_STATIC_BITS: u128 = 64;
const ROLE_PROJECTION_STATIC_BITS: u128 = 256;

/// Logical width of the bounded binary64 VSA workspace.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VsaWorkspaceLayout {
    width: u64,
}

impl VsaWorkspaceLayout {
    /// Construct a non-empty VSA workspace layout.
    pub fn new(width: u64) -> Result<Self, VsaWorkspaceError> {
        if width == 0 {
            return Err(VsaWorkspaceError::ZeroWidth);
        }
        Ok(Self { width })
    }

    /// Number of binary64 coordinates in the persistent workspace.
    #[must_use]
    pub const fn width(self) -> u64 {
        self.width
    }

    fn host_width(self) -> Result<usize, VsaWorkspaceError> {
        let width =
            usize::try_from(self.width).map_err(|_| VsaWorkspaceError::HostDimensionTooLarge {
                component: "width",
                value: self.width,
            })?;
        validate_vector_capacity(width)?;
        Ok(width)
    }

    fn storage_accounting(self) -> Result<VsaWorkspaceAccounting, VsaWorkspaceError> {
        let workspace_bits = u128::from(self.width)
            .checked_mul(BITS_PER_F64)
            .ok_or(VsaWorkspaceError::AccountingOverflow)?;
        let static_parameter_bits = LAYOUT_STATIC_BITS
            .checked_add(ROLE_PROJECTION_STATIC_BITS)
            .ok_or(VsaWorkspaceError::AccountingOverflow)?;
        Ok(VsaWorkspaceAccounting {
            workspace_bits: StorageBits::new(workspace_bits),
            temporary_working_bits: StorageBits::new(workspace_bits),
            static_parameter_bits: StorageBits::new(static_parameter_bits),
        })
    }
}

/// Exact architecture-level storage split for the VSA primitive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VsaWorkspaceAccounting {
    workspace_bits: StorageBits,
    temporary_working_bits: StorageBits,
    static_parameter_bits: StorageBits,
}

impl VsaWorkspaceAccounting {
    /// Persistent binary64 VSA workspace storage.
    #[must_use]
    pub const fn workspace_bits(self) -> StorageBits {
        self.workspace_bits
    }

    /// Maximum width-sized temporary vector used by one primitive operation.
    #[must_use]
    pub const fn temporary_working_bits(self) -> StorageBits {
        self.temporary_working_bits
    }

    /// Layout width, role-projection seed, and fixed SplitMix64 constants.
    #[must_use]
    pub const fn static_parameter_bits(self) -> StorageBits {
        self.static_parameter_bits
    }
}

/// Fail-closed errors from the deterministic VSA reference workspace.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VsaWorkspaceError {
    /// The workspace must contain at least one coordinate.
    ZeroWidth,
    /// A platform-independent width cannot be represented by this host.
    HostDimensionTooLarge {
        /// Name of the offending dimension.
        component: &'static str,
        /// Declared dimension value.
        value: u64,
    },
    /// The requested binary64 vector would exceed the host Vec capacity bound.
    HostVectorCapacityTooLarge {
        /// Requested number of binary64 elements.
        elements: usize,
    },
    /// The host allocator refused a validated vector reservation.
    HostAllocationFailed {
        /// Requested number of binary64 elements.
        elements: usize,
    },
    /// Exact architecture-level storage accounting overflowed.
    AccountingOverflow,
    /// A binding/retrieval payload had the wrong width.
    PayloadWidthMismatch {
        /// Required width.
        expected: usize,
        /// Supplied width.
        actual: usize,
    },
    /// An input payload contained a non-finite binary64 value.
    NonFinitePayload {
        /// Invalid coordinate.
        index: usize,
    },
    /// Bundling produced a non-finite workspace coordinate before commit.
    NonFiniteWorkspaceIntermediate {
        /// Invalid coordinate.
        index: usize,
    },
    /// Similarity accumulation produced a non-finite value.
    NonFiniteSimilarity,
}

impl fmt::Display for VsaWorkspaceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroWidth => formatter.write_str("VSA workspace width must be non-zero"),
            Self::HostDimensionTooLarge { component, value } => {
                write!(
                    formatter,
                    "{component}={value} does not fit the host index type"
                )
            }
            Self::HostVectorCapacityTooLarge { elements } => write!(
                formatter,
                "VSA vector capacity is too large: {elements} binary64 elements"
            ),
            Self::HostAllocationFailed { elements } => write!(
                formatter,
                "host allocation failed for VSA vector with {elements} elements"
            ),
            Self::AccountingOverflow => formatter.write_str("VSA workspace accounting overflow"),
            Self::PayloadWidthMismatch { expected, actual } => write!(
                formatter,
                "VSA payload width mismatch: expected {expected}, got {actual}"
            ),
            Self::NonFinitePayload { index } => {
                write!(formatter, "VSA payload coordinate {index} is not finite")
            }
            Self::NonFiniteWorkspaceIntermediate { index } => write!(
                formatter,
                "VSA workspace coordinate {index} became non-finite during bundling"
            ),
            Self::NonFiniteSimilarity => {
                formatter.write_str("VSA fixed-order similarity became non-finite")
            }
        }
    }
}

impl std::error::Error for VsaWorkspaceError {}

/// Fully validated candidate persistent VSA state.
///
/// This crate-private carrier allows A3 to prepare a fallible bundle update
/// before mutating A2, then commit the already-owned vector without allocation
/// or another numeric failure after the A2 transition succeeds.
#[derive(Debug)]
pub(crate) struct PreparedVsaBundle {
    components: Vec<f64>,
}

/// Bounded deterministic VSA workspace with seeded bipolar role projection.
///
/// A role coordinate is generated on demand as `-1` or `+1` from
/// `SplitMix64(key + seed + coordinate)`. Binding and unbinding are therefore
/// element-wise multiplication by the same bipolar role. Bundling is fixed-order
/// addition into the persistent workspace. No role vectors are stored.
#[derive(Clone, Debug, PartialEq)]
pub struct BoundedVsaWorkspace {
    layout: VsaWorkspaceLayout,
    role_seed: u64,
    components: Vec<f64>,
}

impl BoundedVsaWorkspace {
    /// Allocate a zeroed bounded workspace.
    pub fn new(layout: VsaWorkspaceLayout, role_seed: u64) -> Result<Self, VsaWorkspaceError> {
        let width = layout.host_width()?;
        Ok(Self {
            layout,
            role_seed,
            components: allocate_zeroed(width)?,
        })
    }

    /// Declared workspace layout.
    #[must_use]
    pub const fn layout(&self) -> VsaWorkspaceLayout {
        self.layout
    }

    /// Seed used by the deterministic bipolar role projection.
    #[must_use]
    pub const fn role_seed(&self) -> u64 {
        self.role_seed
    }

    /// Persistent superposed workspace coordinates.
    #[must_use]
    pub fn components(&self) -> &[f64] {
        &self.components
    }

    /// Exact declared storage split for this VSA representation.
    pub fn storage_accounting(&self) -> Result<VsaWorkspaceAccounting, VsaWorkspaceError> {
        self.layout.storage_accounting()
    }

    /// Bind one finite payload to a deterministic bipolar role.
    ///
    /// This operation does not mutate the persistent workspace.
    pub fn bind(&self, key: u64, payload: &[f64]) -> Result<Vec<f64>, VsaWorkspaceError> {
        self.validate_payload(payload)?;
        let mut bound = allocate_zeroed(payload.len())?;
        for (index, (output, value)) in bound.iter_mut().zip(payload.iter()).enumerate() {
            *output = *value * self.role_sign(key, index);
        }
        Ok(bound)
    }

    /// Prepare one complete finite bundle update without persistent mutation.
    ///
    /// The returned vector owns every fallible allocation and numeric result
    /// required by the update. A caller may therefore execute another atomic
    /// mechanism transition and commit this prepared state only after that
    /// transition succeeds.
    pub(crate) fn prepare_bundle(
        &self,
        key: u64,
        payload: &[f64],
    ) -> Result<PreparedVsaBundle, VsaWorkspaceError> {
        self.validate_payload(payload)?;
        let mut next = allocate_zeroed(payload.len())?;
        for (index, ((next_value, current), payload_value)) in next
            .iter_mut()
            .zip(self.components.iter())
            .zip(payload.iter())
            .enumerate()
        {
            let bound = *payload_value * self.role_sign(key, index);
            let bundled = *current + bound;
            if !bundled.is_finite() {
                return Err(VsaWorkspaceError::NonFiniteWorkspaceIntermediate { index });
            }
            *next_value = bundled;
        }
        Ok(PreparedVsaBundle { components: next })
    }

    /// Commit a bundle state produced by [`Self::prepare_bundle`].
    ///
    /// This operation performs no allocation, numeric calculation or validation
    /// and is therefore infallible. The prepared carrier is crate-private so it
    /// can only originate from the validated preparation path above.
    pub(crate) fn commit_prepared_bundle(&mut self, prepared: PreparedVsaBundle) {
        self.components = prepared.components;
    }

    /// Add one bound key/payload pair to the persistent superposition.
    ///
    /// The complete next workspace is computed and validated before commit, so
    /// rejected bundling cannot partially mutate persistent state.
    pub fn bundle(&mut self, key: u64, payload: &[f64]) -> Result<(), VsaWorkspaceError> {
        let prepared = self.prepare_bundle(key, payload)?;
        self.commit_prepared_bundle(prepared);
        Ok(())
    }

    /// Unbind/retrieve the current superposition with one deterministic role.
    pub fn unbind(&self, key: u64) -> Result<Vec<f64>, VsaWorkspaceError> {
        let mut retrieved = allocate_zeroed(self.components.len())?;
        for (index, (output, value)) in retrieved.iter_mut().zip(self.components.iter()).enumerate()
        {
            *output = *value * self.role_sign(key, index);
        }
        Ok(retrieved)
    }

    /// Fixed-order unnormalized dot similarity between one unbound retrieval and
    /// a finite candidate vector.
    pub fn similarity(&self, key: u64, candidate: &[f64]) -> Result<f64, VsaWorkspaceError> {
        self.validate_payload(candidate)?;
        let mut score = 0.0_f64;
        for (index, (workspace_value, candidate_value)) in
            self.components.iter().zip(candidate.iter()).enumerate()
        {
            let retrieved = *workspace_value * self.role_sign(key, index);
            let product = retrieved * *candidate_value;
            if !product.is_finite() {
                return Err(VsaWorkspaceError::NonFiniteSimilarity);
            }
            score += product;
            if !score.is_finite() {
                return Err(VsaWorkspaceError::NonFiniteSimilarity);
            }
        }
        Ok(score)
    }

    /// Reset the persistent workspace while preserving layout and role seed.
    pub fn clear(&mut self) {
        self.components.fill(0.0);
    }

    fn validate_payload(&self, payload: &[f64]) -> Result<(), VsaWorkspaceError> {
        if payload.len() != self.components.len() {
            return Err(VsaWorkspaceError::PayloadWidthMismatch {
                expected: self.components.len(),
                actual: payload.len(),
            });
        }
        if let Some(index) = payload.iter().position(|value| !value.is_finite()) {
            return Err(VsaWorkspaceError::NonFinitePayload { index });
        }
        Ok(())
    }

    fn role_sign(&self, key: u64, index: usize) -> f64 {
        let coordinate = u64::try_from(index).expect("validated VSA width must fit u64");
        let mixed = splitmix64(key.wrapping_add(self.role_seed).wrapping_add(coordinate));
        if mixed & 1 == 0 { -1.0 } else { 1.0 }
    }
}

fn validate_vector_capacity(elements: usize) -> Result<(), VsaWorkspaceError> {
    let bytes = elements
        .checked_mul(mem::size_of::<f64>())
        .ok_or(VsaWorkspaceError::HostVectorCapacityTooLarge { elements })?;
    if bytes > isize::MAX as usize {
        return Err(VsaWorkspaceError::HostVectorCapacityTooLarge { elements });
    }
    Ok(())
}

fn allocate_zeroed(elements: usize) -> Result<Vec<f64>, VsaWorkspaceError> {
    validate_vector_capacity(elements)?;
    let mut values = Vec::new();
    values
        .try_reserve_exact(elements)
        .map_err(|_| VsaWorkspaceError::HostAllocationFailed { elements })?;
    values.resize(elements, 0.0);
    Ok(values)
}

/// Stable SplitMix64 integer mixer used only for deterministic bipolar roles.
#[must_use]
fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::{BoundedVsaWorkspace, VsaWorkspaceError, VsaWorkspaceLayout};
    use crate::StorageBits;

    fn workspace(width: u64) -> BoundedVsaWorkspace {
        let layout = VsaWorkspaceLayout::new(width).expect("valid VSA layout");
        BoundedVsaWorkspace::new(layout, 0x1234_5678_9abc_def0).expect("bounded workspace")
    }

    #[test]
    fn layout_rejects_zero_width() {
        assert_eq!(
            VsaWorkspaceLayout::new(0),
            Err(VsaWorkspaceError::ZeroWidth)
        );
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn oversized_vector_capacity_fails_closed_before_allocation() {
        let elements = (isize::MAX as usize / core::mem::size_of::<f64>()) + 1;
        let layout = VsaWorkspaceLayout::new(elements as u64).expect("logical layout");
        assert_eq!(
            BoundedVsaWorkspace::new(layout, 0),
            Err(VsaWorkspaceError::HostVectorCapacityTooLarge { elements })
        );
    }

    #[test]
    fn single_bundle_unbinds_exactly_for_bipolar_role() {
        let mut vsa = workspace(4);
        let payload = [1.5, -2.0, 0.25, 8.0];
        vsa.bundle(17, &payload).expect("finite bundle");
        let retrieved = vsa.unbind(17).expect("bounded retrieval");
        let retrieved_bits: Vec<_> = retrieved.iter().map(|value| value.to_bits()).collect();
        let payload_bits: Vec<_> = payload.iter().map(|value| value.to_bits()).collect();
        assert_eq!(retrieved_bits, payload_bits);
    }

    #[test]
    fn binding_and_superposition_are_deterministic() {
        let mut left = workspace(8);
        let mut right = left.clone();
        let first = [1.0, 2.0, 3.0, 4.0, -1.0, -2.0, -3.0, -4.0];
        let second = [0.5, 0.25, -0.5, -0.25, 2.0, 1.0, -2.0, -1.0];

        left.bundle(11, &first).expect("first left bundle");
        left.bundle(29, &second).expect("second left bundle");
        right.bundle(11, &first).expect("first right bundle");
        right.bundle(29, &second).expect("second right bundle");

        let left_bits: Vec<_> = left
            .components()
            .iter()
            .map(|value| value.to_bits())
            .collect();
        let right_bits: Vec<_> = right
            .components()
            .iter()
            .map(|value| value.to_bits())
            .collect();
        assert_eq!(left_bits, right_bits);
        assert_eq!(left.unbind(11), right.unbind(11));
    }

    #[test]
    fn prepared_bundle_matches_direct_bundle_bit_exactly() {
        let mut direct = workspace(4);
        let mut prepared = direct.clone();
        direct
            .bundle(17, &[1.5, -2.0, 0.25, 8.0])
            .expect("direct bundle");
        let next = prepared
            .prepare_bundle(17, &[1.5, -2.0, 0.25, 8.0])
            .expect("prepared bundle");
        prepared.commit_prepared_bundle(next);
        let direct_bits: Vec<_> = direct
            .components()
            .iter()
            .map(|value| value.to_bits())
            .collect();
        let prepared_bits: Vec<_> = prepared
            .components()
            .iter()
            .map(|value| value.to_bits())
            .collect();
        assert_eq!(direct_bits, prepared_bits);
    }

    #[test]
    fn rejected_bundle_cannot_mutate_workspace() {
        let mut vsa = workspace(2);
        vsa.bundle(3, &[5.0, 6.0]).expect("seed state");
        let before = vsa.components().to_vec();
        assert_eq!(
            vsa.bundle(4, &[7.0, f64::NAN]),
            Err(VsaWorkspaceError::NonFinitePayload { index: 1 })
        );
        assert_eq!(vsa.components(), before.as_slice());
    }

    #[test]
    fn payload_width_mismatch_fails_closed() {
        let mut vsa = workspace(2);
        assert_eq!(
            vsa.bundle(1, &[1.0]),
            Err(VsaWorkspaceError::PayloadWidthMismatch {
                expected: 2,
                actual: 1,
            })
        );
        assert_eq!(
            vsa.similarity(1, &[1.0]),
            Err(VsaWorkspaceError::PayloadWidthMismatch {
                expected: 2,
                actual: 1,
            })
        );
    }

    #[test]
    fn single_item_similarity_matches_exact_squared_norm() {
        let mut vsa = workspace(2);
        vsa.bundle(41, &[1.0, 2.0]).expect("bundle");
        assert_eq!(vsa.similarity(41, &[1.0, 2.0]), Ok(5.0));
    }

    #[test]
    fn declared_storage_accounts_workspace_working_vector_and_static_projection() {
        let vsa = workspace(4);
        let accounting = vsa.storage_accounting().expect("exact accounting");
        assert_eq!(accounting.workspace_bits(), StorageBits::new(256));
        assert_eq!(accounting.temporary_working_bits(), StorageBits::new(256));
        assert_eq!(accounting.static_parameter_bits(), StorageBits::new(320));
    }

    #[test]
    fn clear_resets_superposition_without_changing_role_projection() {
        let mut vsa = workspace(3);
        let bound_before = vsa.bind(99, &[1.0, 2.0, 3.0]).expect("bind");
        vsa.bundle(99, &[1.0, 2.0, 3.0]).expect("bundle");
        vsa.clear();
        assert_eq!(vsa.components(), &[0.0, 0.0, 0.0]);
        assert_eq!(
            vsa.bind(99, &[1.0, 2.0, 3.0]).expect("bind after clear"),
            bound_before
        );
    }
}
