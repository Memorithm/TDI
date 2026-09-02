//! Deterministic bounded A3 reference semantics for TDI-8.1.
//!
//! A3 combines the same fixed-order recurrent rule used by the A1/A2 software
//! oracle, the bounded direct-mapped associative memory, and the bounded VSA
//! workspace.  This file deliberately freezes only software semantics.  It does
//! not freeze experimental widths, seeds, gains, budgets, horizons, or any
//! TDI-8.2 surface.
//!
//! One A3 step uses one logical `read_key` and optional `write_key` for both
//! external memories.  The operation order is:
//!
//! 1. compute a complete recurrent candidate;
//! 2. associative lookup and coordinate-wise fusion;
//! 3. VSA unbind/retrieval and coordinate-wise fusion;
//! 4. when a write is requested, atomically bundle the final candidate into the
//!    VSA workspace;
//! 5. commit the already-prevalidated associative write;
//! 6. commit the recurrent state.
//!
//! Every recoverable failure occurs before persistent mutation.  The VSA bundle
//! itself computes and validates a complete next workspace before committing it.
//! After that succeeds, the associative write is structurally infallible under
//! constructor and finite-state invariants (matching width, finite payload, no
//! allocation); an unexpected associative error is treated as an internal
//! invariant violation and aborts the run rather than returning a partially
//! committed scientific record.

use core::fmt;

use crate::associative_memory::{
    AssociativeMemoryError, AssociativeMemoryLayout, AssociativeRead, AssociativeWriteOutcome,
    DirectMappedAssociativeMemory,
};
use crate::assr_reference::{A2ReadStatus, RecurrentLayout};
use crate::vsa_workspace::{BoundedVsaWorkspace, VsaWorkspaceError, VsaWorkspaceLayout};
use crate::{
    MemoryAccounting, MemoryAccountingError, ReferenceArm, ReferenceSnapshot, StorageBits,
};

const BITS_PER_F64: u128 = 64;
const RECURRENT_LAYOUT_STATIC_BITS: u128 = 128;
const TWO_FUSION_GAINS_STATIC_BITS: u128 = 128;

/// Frozen-by-construction recurrent parameter tensors for the A3 reference.
///
/// The representation intentionally mirrors the A1/A2 reference: matrices are
/// row-major and accumulation order is bias, recurrent coordinates, then input
/// coordinates, followed by hard-tanh clipping.
#[derive(Clone, Debug, PartialEq)]
pub struct A3RecurrentParameters {
    layout: RecurrentLayout,
    input_to_state: Vec<f64>,
    recurrent_to_state: Vec<f64>,
    bias: Vec<f64>,
}

impl A3RecurrentParameters {
    /// Validate shapes and finite binary64 values.
    pub fn new(
        layout: RecurrentLayout,
        input_to_state: Vec<f64>,
        recurrent_to_state: Vec<f64>,
        bias: Vec<f64>,
    ) -> Result<Self, A3ReferenceError> {
        let input_width = usize::try_from(layout.input_width()).map_err(|_| {
            A3ReferenceError::HostDimensionTooLarge {
                component: "input_width",
                value: layout.input_width(),
            }
        })?;
        let state_width = usize::try_from(layout.state_width()).map_err(|_| {
            A3ReferenceError::HostDimensionTooLarge {
                component: "state_width",
                value: layout.state_width(),
            }
        })?;
        let expected_input = state_width
            .checked_mul(input_width)
            .ok_or(A3ReferenceError::HostLengthOverflow {
                component: "input_to_state",
            })?;
        let expected_recurrent = state_width
            .checked_mul(state_width)
            .ok_or(A3ReferenceError::HostLengthOverflow {
                component: "recurrent_to_state",
            })?;
        validate_length("input_to_state", expected_input, input_to_state.len())?;
        validate_length(
            "recurrent_to_state",
            expected_recurrent,
            recurrent_to_state.len(),
        )?;
        validate_length("bias", state_width, bias.len())?;
        validate_finite("input_to_state", &input_to_state)?;
        validate_finite("recurrent_to_state", &recurrent_to_state)?;
        validate_finite("bias", &bias)?;
        Ok(Self {
            layout,
            input_to_state,
            recurrent_to_state,
            bias,
        })
    }

    /// Declared recurrent layout.
    #[must_use]
    pub const fn layout(&self) -> RecurrentLayout {
        self.layout
    }

    fn static_parameter_bits(&self) -> Result<StorageBits, A3ReferenceError> {
        let values = self
            .input_to_state
            .len()
            .checked_add(self.recurrent_to_state.len())
            .and_then(|count| count.checked_add(self.bias.len()))
            .ok_or(A3ReferenceError::AccountingOverflow)?;
        let value_bits = (values as u128)
            .checked_mul(BITS_PER_F64)
            .ok_or(A3ReferenceError::AccountingOverflow)?;
        let total = value_bits
            .checked_add(RECURRENT_LAYOUT_STATIC_BITS)
            .ok_or(A3ReferenceError::AccountingOverflow)?;
        Ok(StorageBits::new(total))
    }
}

/// Fail-closed errors for bounded A3 construction and recoverable steps.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum A3ReferenceError {
    /// A platform-independent dimension cannot be represented on this host.
    HostDimensionTooLarge {
        /// Component name.
        component: &'static str,
        /// Declared value.
        value: u64,
    },
    /// A derived flattened tensor length overflowed the host index type.
    HostLengthOverflow {
        /// Tensor/component name.
        component: &'static str,
    },
    /// A host vector reservation failed.
    HostAllocationFailed {
        /// Requested number of binary64 values.
        elements: usize,
    },
    /// A recurrent parameter tensor has the wrong flattened length.
    ParameterLengthMismatch {
        /// Tensor name.
        component: &'static str,
        /// Required length.
        expected: usize,
        /// Supplied length.
        actual: usize,
    },
    /// A recurrent parameter tensor contains a non-finite value.
    NonFiniteParameter {
        /// Tensor name.
        component: &'static str,
        /// Invalid coordinate.
        index: usize,
    },
    /// A step input has the wrong width.
    InputWidthMismatch {
        /// Required width.
        expected: usize,
        /// Supplied width.
        actual: usize,
    },
    /// A step input contains a non-finite value.
    NonFiniteInput {
        /// Invalid input coordinate.
        index: usize,
    },
    /// A deterministic accumulation/fusion produced a non-finite value.
    NonFiniteIntermediate {
        /// Recurrent coordinate being computed.
        state_index: usize,
    },
    /// Associative payload width must equal recurrent-state width.
    AssociativePayloadWidthMismatch {
        /// Recurrent-state width.
        state_width: u64,
        /// Associative payload width.
        payload_width: u64,
    },
    /// The bounded VSA workspace width must equal recurrent-state width for the
    /// reference coordinate-wise A3 fusion rule.
    VsaWorkspaceWidthMismatch {
        /// Recurrent-state width.
        state_width: u64,
        /// VSA workspace width.
        vsa_width: u64,
    },
    /// Associative fusion gain must be finite.
    NonFiniteAssociativeFusionGain,
    /// VSA fusion gain must be finite.
    NonFiniteVsaFusionGain,
    /// Exact bit accounting overflowed.
    AccountingOverflow,
    /// Bounded associative-memory failure before commit.
    AssociativeMemory(AssociativeMemoryError),
    /// Bounded VSA failure before commit.
    VsaWorkspace(VsaWorkspaceError),
    /// Common TDI-8 memory-accounting validation failed.
    MemoryAccounting(MemoryAccountingError),
}

impl fmt::Display for A3ReferenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for A3ReferenceError {}

impl From<AssociativeMemoryError> for A3ReferenceError {
    fn from(error: AssociativeMemoryError) -> Self {
        Self::AssociativeMemory(error)
    }
}

impl From<VsaWorkspaceError> for A3ReferenceError {
    fn from(error: VsaWorkspaceError) -> Self {
        Self::VsaWorkspace(error)
    }
}

impl From<MemoryAccountingError> for A3ReferenceError {
    fn from(error: MemoryAccountingError) -> Self {
        Self::MemoryAccounting(error)
    }
}

/// Deterministic observable outcomes from one A3 step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct A3StepReport {
    associative_read: A2ReadStatus,
    associative_write: Option<AssociativeWriteOutcome>,
    vsa_bundled: bool,
}

impl A3StepReport {
    /// Associative lookup result observed before the optional write.
    #[must_use]
    pub const fn associative_read(self) -> A2ReadStatus {
        self.associative_read
    }

    /// Optional direct-mapped associative write outcome.
    #[must_use]
    pub const fn associative_write(self) -> Option<AssociativeWriteOutcome> {
        self.associative_write
    }

    /// Whether the same optional write key was bundled into the VSA workspace.
    #[must_use]
    pub const fn vsa_bundled(self) -> bool {
        self.vsa_bundled
    }
}

/// Complete persistent A3 state used by later intervention or snapshot layers.
#[derive(Clone, Debug, PartialEq)]
pub struct A3StateSnapshot {
    recurrent_state: Vec<f64>,
    associative_memory: DirectMappedAssociativeMemory,
    vsa_workspace: BoundedVsaWorkspace,
}

impl A3StateSnapshot {
    /// Persistent recurrent state.
    #[must_use]
    pub fn recurrent_state(&self) -> &[f64] {
        &self.recurrent_state
    }

    /// Complete bounded associative memory.
    #[must_use]
    pub const fn associative_memory(&self) -> &DirectMappedAssociativeMemory {
        &self.associative_memory
    }

    /// Complete bounded VSA workspace.
    #[must_use]
    pub const fn vsa_workspace(&self) -> &BoundedVsaWorkspace {
        &self.vsa_workspace
    }
}

/// Bounded A3 recurrent + associative-memory + VSA reference.
#[derive(Clone, Debug, PartialEq)]
pub struct A3Reference {
    parameters: A3RecurrentParameters,
    recurrent_state: Vec<f64>,
    associative_memory: DirectMappedAssociativeMemory,
    vsa_workspace: BoundedVsaWorkspace,
    associative_fusion_gain: f64,
    vsa_fusion_gain: f64,
}

impl A3Reference {
    /// Construct a zero-state A3 reference.
    ///
    /// The reference currently uses direct coordinate-wise fusion, so both
    /// external-memory payload widths must equal recurrent-state width.  This is
    /// a software-oracle representation choice, not a freeze of the eventual
    /// experimental numeric width.
    pub fn new(
        parameters: A3RecurrentParameters,
        memory_layout: AssociativeMemoryLayout,
        associative_projection_seed: u64,
        associative_fusion_gain: f64,
        vsa_layout: VsaWorkspaceLayout,
        vsa_role_seed: u64,
        vsa_fusion_gain: f64,
    ) -> Result<Self, A3ReferenceError> {
        let state_width = parameters.layout.state_width();
        if memory_layout.payload_width() != state_width {
            return Err(A3ReferenceError::AssociativePayloadWidthMismatch {
                state_width,
                payload_width: memory_layout.payload_width(),
            });
        }
        if vsa_layout.width() != state_width {
            return Err(A3ReferenceError::VsaWorkspaceWidthMismatch {
                state_width,
                vsa_width: vsa_layout.width(),
            });
        }
        if !associative_fusion_gain.is_finite() {
            return Err(A3ReferenceError::NonFiniteAssociativeFusionGain);
        }
        if !vsa_fusion_gain.is_finite() {
            return Err(A3ReferenceError::NonFiniteVsaFusionGain);
        }
        let state_elements = usize::try_from(state_width).map_err(|_| {
            A3ReferenceError::HostDimensionTooLarge {
                component: "state_width",
                value: state_width,
            }
        })?;
        let recurrent_state = allocate_zeroed(state_elements)?;
        Ok(Self {
            parameters,
            recurrent_state,
            associative_memory: DirectMappedAssociativeMemory::new(
                memory_layout,
                associative_projection_seed,
            )?,
            vsa_workspace: BoundedVsaWorkspace::new(vsa_layout, vsa_role_seed)?,
            associative_fusion_gain,
            vsa_fusion_gain,
        })
    }

    /// Current recurrent state.
    #[must_use]
    pub fn state(&self) -> &[f64] {
        &self.recurrent_state
    }

    /// Bounded associative state.
    #[must_use]
    pub const fn associative_memory(&self) -> &DirectMappedAssociativeMemory {
        &self.associative_memory
    }

    /// Bounded VSA/holographic workspace.
    #[must_use]
    pub const fn vsa_workspace(&self) -> &BoundedVsaWorkspace {
        &self.vsa_workspace
    }

    /// Advance one deterministic A3 step.
    pub fn step(
        &mut self,
        input: &[f64],
        read_key: u64,
        write_key: Option<u64>,
    ) -> Result<A3StepReport, A3ReferenceError> {
        let mut next = self.compute_recurrent_candidate(input)?;

        let associative_read = match self.associative_memory.read(read_key) {
            AssociativeRead::Empty { address } => A2ReadStatus::Empty { address },
            AssociativeRead::CollisionMiss {
                address,
                resident_key,
            } => A2ReadStatus::CollisionMiss {
                address,
                resident_key,
            },
            AssociativeRead::Hit { address, payload } => {
                fuse_in_place(&mut next, payload, self.associative_fusion_gain)?;
                A2ReadStatus::Hit { address }
            }
        };

        let vsa_retrieved = self.vsa_workspace.unbind(read_key)?;
        fuse_in_place(&mut next, &vsa_retrieved, self.vsa_fusion_gain)?;
        drop(vsa_retrieved);

        let associative_write = if let Some(key) = write_key {
            // This is the final recoverable/fallible persistent-state operation.
            // `bundle` validates and constructs a complete next VSA vector before
            // replacing the old workspace.  If it returns Err, associative and
            // recurrent persistent state are still unchanged.
            self.vsa_workspace.bundle(key, &next)?;

            // Constructor invariants prove `next` has exactly the associative
            // payload width, and every element is finite after hard-tanh.  The
            // direct-mapped write performs no allocation.  A failure here thus
            // signals implementation-contract drift rather than a scientific
            // record that can safely continue.
            let outcome = self
                .associative_memory
                .write(key, &next)
                .unwrap_or_else(|error| {
                    panic!(
                        "A3 prevalidated associative commit violated internal invariant: {error}"
                    )
                });
            Some(outcome)
        } else {
            None
        };

        self.recurrent_state = next;
        Ok(A3StepReport {
            associative_read,
            associative_write,
            vsa_bundled: write_key.is_some(),
        })
    }

    /// Clear all persistent dynamic state while preserving static parameters.
    pub fn reset(&mut self) {
        self.recurrent_state.fill(0.0);
        self.associative_memory.clear();
        self.vsa_workspace.clear();
    }

    /// Exact architecture-level memory accounting.
    ///
    /// `temporary_working` is the true peak of the recurrent candidate plus one
    /// width-sized VSA temporary vector.  Retrieval is dropped before a VSA
    /// bundle is prepared, so two VSA temporary vectors are never live together.
    pub fn memory_accounting(&self) -> Result<MemoryAccounting, A3ReferenceError> {
        let recurrent_bits = storage_bits_for_values(self.parameters.layout.state_width())?;
        let associative = self.associative_memory.storage_accounting()?;
        let vsa = self.vsa_workspace.storage_accounting()?;
        let temporary_bits = recurrent_bits
            .get()
            .checked_add(vsa.temporary_working_bits().get())
            .ok_or(A3ReferenceError::AccountingOverflow)?;
        let static_bits = self
            .parameters
            .static_parameter_bits()?
            .get()
            .checked_add(associative.static_parameter_bits().get())
            .and_then(|bits| bits.checked_add(vsa.static_parameter_bits().get()))
            .and_then(|bits| bits.checked_add(TWO_FUSION_GAINS_STATIC_BITS))
            .ok_or(A3ReferenceError::AccountingOverflow)?;

        let accounting = MemoryAccounting::zero()
            .with_recurrent_state(recurrent_bits)
            .with_associative_payload(associative.payload_bits())
            .with_associative_metadata(associative.metadata_bits())
            .with_vsa_workspace(vsa.workspace_bits())
            .with_temporary_working(StorageBits::new(temporary_bits))
            .with_static_parameters(StorageBits::new(static_bits));
        accounting.validate_for_arm(ReferenceArm::A3)?;
        Ok(accounting)
    }

    /// Clone a validated framework-independent A3 snapshot.
    pub fn snapshot(&self) -> Result<ReferenceSnapshot<A3StateSnapshot>, A3ReferenceError> {
        let state = A3StateSnapshot {
            recurrent_state: self.recurrent_state.clone(),
            associative_memory: self.associative_memory.clone(),
            vsa_workspace: self.vsa_workspace.clone(),
        };
        Ok(ReferenceSnapshot::new(
            ReferenceArm::A3,
            state,
            self.memory_accounting()?,
        )?)
    }

    fn compute_recurrent_candidate(&self, input: &[f64]) -> Result<Vec<f64>, A3ReferenceError> {
        let input_width = usize::try_from(self.parameters.layout.input_width()).map_err(|_| {
            A3ReferenceError::HostDimensionTooLarge {
                component: "input_width",
                value: self.parameters.layout.input_width(),
            }
        })?;
        let state_width = usize::try_from(self.parameters.layout.state_width()).map_err(|_| {
            A3ReferenceError::HostDimensionTooLarge {
                component: "state_width",
                value: self.parameters.layout.state_width(),
            }
        })?;
        if input.len() != input_width {
            return Err(A3ReferenceError::InputWidthMismatch {
                expected: input_width,
                actual: input.len(),
            });
        }
        if let Some(index) = input.iter().position(|value| !value.is_finite()) {
            return Err(A3ReferenceError::NonFiniteInput { index });
        }

        let mut next = allocate_zeroed(state_width)?;
        for (row, next_value) in next.iter_mut().enumerate() {
            let mut accumulator = self.parameters.bias[row];
            let recurrent_row = row.checked_mul(state_width).ok_or(
                A3ReferenceError::HostLengthOverflow {
                    component: "recurrent_to_state row",
                },
            )?;
            for column in 0..state_width {
                accumulator += self.parameters.recurrent_to_state[recurrent_row + column]
                    * self.recurrent_state[column];
                if !accumulator.is_finite() {
                    return Err(A3ReferenceError::NonFiniteIntermediate {
                        state_index: row,
                    });
                }
            }
            let input_row = row
                .checked_mul(input_width)
                .ok_or(A3ReferenceError::HostLengthOverflow {
                    component: "input_to_state row",
                })?;
            for (column, input_value) in input.iter().enumerate() {
                accumulator += self.parameters.input_to_state[input_row + column] * *input_value;
                if !accumulator.is_finite() {
                    return Err(A3ReferenceError::NonFiniteIntermediate {
                        state_index: row,
                    });
                }
            }
            *next_value = hard_tanh(accumulator);
        }
        Ok(next)
    }
}

fn validate_length(
    component: &'static str,
    expected: usize,
    actual: usize,
) -> Result<(), A3ReferenceError> {
    if expected == actual {
        Ok(())
    } else {
        Err(A3ReferenceError::ParameterLengthMismatch {
            component,
            expected,
            actual,
        })
    }
}

fn validate_finite(component: &'static str, values: &[f64]) -> Result<(), A3ReferenceError> {
    if let Some(index) = values.iter().position(|value| !value.is_finite()) {
        Err(A3ReferenceError::NonFiniteParameter { component, index })
    } else {
        Ok(())
    }
}

fn allocate_zeroed(elements: usize) -> Result<Vec<f64>, A3ReferenceError> {
    let bytes = elements
        .checked_mul(core::mem::size_of::<f64>())
        .ok_or(A3ReferenceError::HostAllocationFailed { elements })?;
    if bytes > isize::MAX as usize {
        return Err(A3ReferenceError::HostAllocationFailed { elements });
    }
    let mut values = Vec::new();
    values
        .try_reserve_exact(elements)
        .map_err(|_| A3ReferenceError::HostAllocationFailed { elements })?;
    values.resize(elements, 0.0);
    Ok(values)
}

fn storage_bits_for_values(values: u64) -> Result<StorageBits, A3ReferenceError> {
    let bits = u128::from(values)
        .checked_mul(BITS_PER_F64)
        .ok_or(A3ReferenceError::AccountingOverflow)?;
    Ok(StorageBits::new(bits))
}

fn fuse_in_place(
    target: &mut [f64],
    source: &[f64],
    gain: f64,
) -> Result<(), A3ReferenceError> {
    debug_assert_eq!(target.len(), source.len());
    for (index, (target_value, source_value)) in target.iter_mut().zip(source.iter()).enumerate() {
        let fused = *target_value + gain * *source_value;
        if !fused.is_finite() {
            return Err(A3ReferenceError::NonFiniteIntermediate {
                state_index: index,
            });
        }
        *target_value = hard_tanh(fused);
    }
    Ok(())
}

fn hard_tanh(value: f64) -> f64 {
    value.clamp(-1.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::{A3RecurrentParameters, A3Reference, A3ReferenceError};
    use crate::ReferenceArm;
    use crate::associative_memory::AssociativeMemoryLayout;
    use crate::assr_reference::{A2Reference, RecurrentLayout, RecurrentParameters};
    use crate::vsa_workspace::VsaWorkspaceLayout;

    fn layout() -> RecurrentLayout {
        RecurrentLayout::new(2, 2).expect("valid layout")
    }

    fn raw_parameters() -> (Vec<f64>, Vec<f64>, Vec<f64>) {
        (
            vec![1.0, 0.0, 0.0, 1.0],
            vec![0.0, 0.0, 0.0, 0.0],
            vec![0.0, 0.0],
        )
    }

    fn a3_parameters() -> A3RecurrentParameters {
        let (input, recurrent, bias) = raw_parameters();
        A3RecurrentParameters::new(layout(), input, recurrent, bias).expect("A3 parameters")
    }

    fn a3(associative_gain: f64, vsa_gain: f64) -> A3Reference {
        A3Reference::new(
            a3_parameters(),
            AssociativeMemoryLayout::new(4, 2).expect("memory layout"),
            17,
            associative_gain,
            VsaWorkspaceLayout::new(2).expect("VSA layout"),
            29,
            vsa_gain,
        )
        .expect("A3")
    }

    #[test]
    fn a3_matches_a2_before_vsa_contains_information() {
        let (input, recurrent, bias) = raw_parameters();
        let mut a2 = A2Reference::new(
            RecurrentParameters::new(layout(), input, recurrent, bias).expect("A2 parameters"),
            AssociativeMemoryLayout::new(4, 2).expect("memory layout"),
            17,
            0.5,
        )
        .expect("A2");
        let mut a3 = a3(0.5, 0.75);

        let a2_report = a2.step(&[0.25, -0.5], 7, None).expect("A2 step");
        let a3_report = a3.step(&[0.25, -0.5], 7, None).expect("A3 step");
        assert_eq!(a3.state(), a2.state());
        assert_eq!(a3_report.associative_read(), a2_report.read());
        assert_eq!(a3_report.associative_write(), None);
        assert!(!a3_report.vsa_bundled());
    }

    #[test]
    fn vsa_write_then_read_contributes_to_later_state() {
        let mut reference = a3(0.0, 1.0);
        reference
            .step(&[0.5, -0.25], 11, Some(11))
            .expect("first write");
        reference
            .step(&[0.0, 0.0], 11, None)
            .expect("VSA read");
        assert_eq!(reference.state(), &[0.5, -0.25]);
    }

    #[test]
    fn rejected_step_does_not_mutate_any_persistent_a3_state() {
        let mut reference = a3(0.5, 0.5);
        reference
            .step(&[0.25, 0.5], 9, Some(9))
            .expect("seed step");
        let before = reference.snapshot().expect("snapshot");

        assert_eq!(
            reference.step(&[f64::NAN, 0.0], 9, Some(9)),
            Err(A3ReferenceError::NonFiniteInput { index: 0 })
        );
        let after = reference.snapshot().expect("snapshot");
        assert_eq!(before.state().recurrent_state(), after.state().recurrent_state());
        assert_eq!(
            before.state().associative_memory(),
            after.state().associative_memory()
        );
        assert_eq!(before.state().vsa_workspace(), after.state().vsa_workspace());
    }

    #[test]
    fn a3_requires_state_aligned_external_memory_widths() {
        let error = A3Reference::new(
            a3_parameters(),
            AssociativeMemoryLayout::new(4, 2).expect("memory layout"),
            17,
            0.5,
            VsaWorkspaceLayout::new(3).expect("VSA layout"),
            29,
            0.5,
        )
        .expect_err("mismatched VSA width must reject");
        assert_eq!(
            error,
            A3ReferenceError::VsaWorkspaceWidthMismatch {
                state_width: 2,
                vsa_width: 3,
            }
        );
    }

    #[test]
    fn a3_accounting_includes_vsa_and_concurrent_peak_scratch() {
        let reference = a3(0.5, 0.5);
        let accounting = reference.memory_accounting().expect("accounting");
        accounting
            .validate_for_arm(ReferenceArm::A3)
            .expect("valid A3 accounting");
        assert_eq!(accounting.recurrent_state().get(), 128);
        assert!(accounting.associative_payload().get() > 0);
        assert!(accounting.associative_metadata().get() > 0);
        assert_eq!(accounting.vsa_workspace().get(), 128);
        // 2 recurrent candidate values + 2 VSA retrieval/bundle-working values.
        assert_eq!(accounting.temporary_working().get(), 256);
    }

    #[test]
    fn snapshot_captures_all_three_persistent_a3_components() {
        let mut reference = a3(0.5, 0.5);
        reference
            .step(&[0.25, -0.5], 3, Some(3))
            .expect("write state");
        let snapshot = reference.snapshot().expect("snapshot");
        assert_eq!(snapshot.arm(), ReferenceArm::A3);
        assert_eq!(snapshot.state().recurrent_state(), reference.state());
        assert_eq!(
            snapshot.state().associative_memory(),
            reference.associative_memory()
        );
        assert_eq!(snapshot.state().vsa_workspace(), reference.vsa_workspace());
    }

    #[test]
    fn reset_clears_recurrent_associative_and_vsa_state() {
        let mut reference = a3(0.5, 0.5);
        reference
            .step(&[0.25, -0.5], 3, Some(3))
            .expect("write state");
        reference.reset();
        assert_eq!(reference.state(), &[0.0, 0.0]);
        assert!(reference.vsa_workspace().components().iter().all(|v| *v == 0.0));
        assert!(matches!(
            reference.associative_memory().read(3),
            crate::associative_memory::AssociativeRead::Empty { .. }
        ));
    }
}
