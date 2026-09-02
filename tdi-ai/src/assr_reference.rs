//! Deterministic bounded A1/A2 reference semantics for TDI-8.1.
//!
//! This module implements a transparent recurrent-state-only A1 reference and
//! an A2 reference that adds the bounded associative-memory primitive. It does
//! not freeze experimental dimensions, seeds or budgets and it contains no
//! TDI-8.2 surface.

use core::fmt;

use crate::associative_memory::{
    AssociativeMemoryError, AssociativeMemoryLayout, AssociativeRead, AssociativeWriteOutcome,
    DirectMappedAssociativeMemory,
};
use crate::{
    MemoryAccounting, MemoryAccountingError, ReferenceArm, ReferenceSnapshot, StorageBits,
};

const BITS_PER_F64: u128 = 64;
const RECURRENT_LAYOUT_STATIC_BITS: u128 = 128;
const FUSION_GAIN_STATIC_BITS: u128 = 64;

/// Platform-independent vector dimensions for the deterministic recurrent core.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RecurrentLayout {
    input_width: u64,
    state_width: u64,
}

impl RecurrentLayout {
    /// Construct a non-empty recurrent layout.
    pub fn new(input_width: u64, state_width: u64) -> Result<Self, RecurrentReferenceError> {
        if input_width == 0 {
            return Err(RecurrentReferenceError::ZeroInputWidth);
        }
        if state_width == 0 {
            return Err(RecurrentReferenceError::ZeroStateWidth);
        }
        Ok(Self {
            input_width,
            state_width,
        })
    }

    /// Number of binary64 input coordinates consumed per step.
    #[must_use]
    pub const fn input_width(self) -> u64 {
        self.input_width
    }

    /// Number of persistent binary64 recurrent-state coordinates.
    #[must_use]
    pub const fn state_width(self) -> u64 {
        self.state_width
    }

    fn host_dimensions(self) -> Result<(usize, usize), RecurrentReferenceError> {
        let input_width = usize::try_from(self.input_width).map_err(|_| {
            RecurrentReferenceError::HostDimensionTooLarge {
                component: "input_width",
                value: self.input_width,
            }
        })?;
        let state_width = usize::try_from(self.state_width).map_err(|_| {
            RecurrentReferenceError::HostDimensionTooLarge {
                component: "state_width",
                value: self.state_width,
            }
        })?;
        Ok((input_width, state_width))
    }

    fn recurrent_parameter_lengths(self) -> Result<(usize, usize, usize), RecurrentReferenceError> {
        let (input_width, state_width) = self.host_dimensions()?;
        let input_matrix = state_width.checked_mul(input_width).ok_or(
            RecurrentReferenceError::HostLengthOverflow {
                component: "input_to_state",
            },
        )?;
        let recurrent_matrix = state_width.checked_mul(state_width).ok_or(
            RecurrentReferenceError::HostLengthOverflow {
                component: "recurrent_to_state",
            },
        )?;
        Ok((input_matrix, recurrent_matrix, state_width))
    }
}

/// Frozen-by-construction parameter tensors for the bounded recurrent core.
///
/// Matrices use row-major order. The recurrence uses a fixed left-to-right
/// accumulation order and a deterministic hard-tanh activation.
#[derive(Clone, Debug, PartialEq)]
pub struct RecurrentParameters {
    layout: RecurrentLayout,
    input_to_state: Vec<f64>,
    recurrent_to_state: Vec<f64>,
    bias: Vec<f64>,
}

impl RecurrentParameters {
    /// Validate shapes and finite binary64 parameter values.
    pub fn new(
        layout: RecurrentLayout,
        input_to_state: Vec<f64>,
        recurrent_to_state: Vec<f64>,
        bias: Vec<f64>,
    ) -> Result<Self, RecurrentReferenceError> {
        let (expected_input, expected_recurrent, expected_bias) =
            layout.recurrent_parameter_lengths()?;
        validate_length("input_to_state", expected_input, input_to_state.len())?;
        validate_length(
            "recurrent_to_state",
            expected_recurrent,
            recurrent_to_state.len(),
        )?;
        validate_length("bias", expected_bias, bias.len())?;
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

    fn static_parameter_bits(&self) -> Result<StorageBits, RecurrentReferenceError> {
        let value_count = self
            .input_to_state
            .len()
            .checked_add(self.recurrent_to_state.len())
            .and_then(|count| count.checked_add(self.bias.len()))
            .ok_or(RecurrentReferenceError::AccountingOverflow)?;
        let value_bits = (value_count as u128)
            .checked_mul(BITS_PER_F64)
            .ok_or(RecurrentReferenceError::AccountingOverflow)?;
        let total = value_bits
            .checked_add(RECURRENT_LAYOUT_STATIC_BITS)
            .ok_or(RecurrentReferenceError::AccountingOverflow)?;
        Ok(StorageBits::new(total))
    }
}

/// Fail-closed errors for the deterministic A1/A2 reference mechanisms.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecurrentReferenceError {
    /// A recurrent input vector must have at least one coordinate.
    ZeroInputWidth,
    /// A recurrent state must have at least one coordinate.
    ZeroStateWidth,
    /// A platform-independent dimension cannot be represented by this host.
    HostDimensionTooLarge {
        /// Name of the offending component.
        component: &'static str,
        /// Declared dimension.
        value: u64,
    },
    /// A derived vector/matrix length overflowed the host index representation.
    HostLengthOverflow {
        /// Name of the offending component.
        component: &'static str,
    },
    /// The host allocator refused a recurrent working/state vector.
    HostAllocationFailed {
        /// Requested number of binary64 values.
        elements: usize,
    },
    /// A parameter tensor had an unexpected flattened length.
    ParameterLengthMismatch {
        /// Parameter tensor name.
        component: &'static str,
        /// Required length.
        expected: usize,
        /// Supplied length.
        actual: usize,
    },
    /// A parameter tensor contained a non-finite value.
    NonFiniteParameter {
        /// Parameter tensor name.
        component: &'static str,
        /// Invalid flattened coordinate.
        index: usize,
    },
    /// A step input had the wrong width.
    InputWidthMismatch {
        /// Required width.
        expected: usize,
        /// Supplied width.
        actual: usize,
    },
    /// A step input contained a non-finite value.
    NonFiniteInput {
        /// Invalid input coordinate.
        index: usize,
    },
    /// A deterministic accumulation produced a non-finite intermediate.
    NonFiniteIntermediate {
        /// State coordinate being computed.
        state_index: usize,
    },
    /// A2 memory payload width must equal recurrent-state width for direct
    /// coordinate-wise fusion.
    AssociativePayloadWidthMismatch {
        /// Recurrent-state width.
        state_width: u64,
        /// Associative payload width.
        payload_width: u64,
    },
    /// A2 fusion gain must be finite.
    NonFiniteFusionGain,
    /// Exact architecture-level bit accounting overflowed.
    AccountingOverflow,
    /// Underlying associative-memory failure.
    AssociativeMemory(AssociativeMemoryError),
    /// Common TDI-8 memory-accounting validation failed.
    MemoryAccounting(MemoryAccountingError),
}

impl fmt::Display for RecurrentReferenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroInputWidth => formatter.write_str("recurrent input width must be non-zero"),
            Self::ZeroStateWidth => formatter.write_str("recurrent state width must be non-zero"),
            Self::HostDimensionTooLarge { component, value } => write!(
                formatter,
                "{component}={value} does not fit the host index type"
            ),
            Self::HostLengthOverflow { component } => {
                write!(
                    formatter,
                    "{component} length overflows the host index type"
                )
            }
            Self::HostAllocationFailed { elements } => write!(
                formatter,
                "host allocation failed for recurrent vector with {elements} elements"
            ),
            Self::ParameterLengthMismatch {
                component,
                expected,
                actual,
            } => write!(
                formatter,
                "{component} length mismatch: expected {expected}, got {actual}"
            ),
            Self::NonFiniteParameter { component, index } => {
                write!(formatter, "{component}[{index}] is not finite")
            }
            Self::InputWidthMismatch { expected, actual } => write!(
                formatter,
                "recurrent input width mismatch: expected {expected}, got {actual}"
            ),
            Self::NonFiniteInput { index } => {
                write!(
                    formatter,
                    "recurrent input coordinate {index} is not finite"
                )
            }
            Self::NonFiniteIntermediate { state_index } => write!(
                formatter,
                "recurrent state coordinate {state_index} became non-finite"
            ),
            Self::AssociativePayloadWidthMismatch {
                state_width,
                payload_width,
            } => write!(
                formatter,
                "A2 associative payload width {payload_width} must equal recurrent state width {state_width}"
            ),
            Self::NonFiniteFusionGain => formatter.write_str("A2 fusion gain must be finite"),
            Self::AccountingOverflow => formatter.write_str("A1/A2 bit accounting overflow"),
            Self::AssociativeMemory(error) => write!(formatter, "associative memory: {error}"),
            Self::MemoryAccounting(error) => write!(formatter, "memory accounting: {error}"),
        }
    }
}

impl std::error::Error for RecurrentReferenceError {}

impl From<AssociativeMemoryError> for RecurrentReferenceError {
    fn from(error: AssociativeMemoryError) -> Self {
        Self::AssociativeMemory(error)
    }
}

impl From<MemoryAccountingError> for RecurrentReferenceError {
    fn from(error: MemoryAccountingError) -> Self {
        Self::MemoryAccounting(error)
    }
}

fn validate_length(
    component: &'static str,
    expected: usize,
    actual: usize,
) -> Result<(), RecurrentReferenceError> {
    if expected == actual {
        Ok(())
    } else {
        Err(RecurrentReferenceError::ParameterLengthMismatch {
            component,
            expected,
            actual,
        })
    }
}

fn validate_finite(component: &'static str, values: &[f64]) -> Result<(), RecurrentReferenceError> {
    if let Some(index) = values.iter().position(|value| !value.is_finite()) {
        Err(RecurrentReferenceError::NonFiniteParameter { component, index })
    } else {
        Ok(())
    }
}

fn allocate_zeroed(elements: usize) -> Result<Vec<f64>, RecurrentReferenceError> {
    let bytes = elements
        .checked_mul(core::mem::size_of::<f64>())
        .ok_or(RecurrentReferenceError::HostAllocationFailed { elements })?;
    if bytes > isize::MAX as usize {
        return Err(RecurrentReferenceError::HostAllocationFailed { elements });
    }
    let mut values = Vec::new();
    values
        .try_reserve_exact(elements)
        .map_err(|_| RecurrentReferenceError::HostAllocationFailed { elements })?;
    values.resize(elements, 0.0);
    Ok(values)
}

fn storage_bits_for_values(values: u64) -> Result<StorageBits, RecurrentReferenceError> {
    let bits = u128::from(values)
        .checked_mul(BITS_PER_F64)
        .ok_or(RecurrentReferenceError::AccountingOverflow)?;
    Ok(StorageBits::new(bits))
}

fn hard_tanh(value: f64) -> f64 {
    value.clamp(-1.0, 1.0)
}

/// Deterministic bounded recurrent core used by A1 and A2.
#[derive(Clone, Debug, PartialEq)]
pub struct BoundedRecurrentCore {
    parameters: RecurrentParameters,
    state: Vec<f64>,
}

impl BoundedRecurrentCore {
    /// Construct the zero-state recurrent core.
    pub fn new(parameters: RecurrentParameters) -> Result<Self, RecurrentReferenceError> {
        let (_, state_width) = parameters.layout.host_dimensions()?;
        let state = allocate_zeroed(state_width)?;
        Ok(Self { parameters, state })
    }

    /// Current recurrent state.
    #[must_use]
    pub fn state(&self) -> &[f64] {
        &self.state
    }

    /// Recurrent layout.
    #[must_use]
    pub const fn layout(&self) -> RecurrentLayout {
        self.parameters.layout
    }

    /// Reset only the persistent recurrent state to zero.
    pub fn reset(&mut self) {
        self.state.fill(0.0);
    }

    fn compute_next(&self, input: &[f64]) -> Result<Vec<f64>, RecurrentReferenceError> {
        let (input_width, state_width) = self.parameters.layout.host_dimensions()?;
        if input.len() != input_width {
            return Err(RecurrentReferenceError::InputWidthMismatch {
                expected: input_width,
                actual: input.len(),
            });
        }
        if let Some(index) = input.iter().position(|value| !value.is_finite()) {
            return Err(RecurrentReferenceError::NonFiniteInput { index });
        }

        let mut next = allocate_zeroed(state_width)?;
        for (row, next_value) in next.iter_mut().enumerate() {
            let mut accumulator = self.parameters.bias[row];

            let recurrent_row = row.checked_mul(state_width).ok_or(
                RecurrentReferenceError::HostLengthOverflow {
                    component: "recurrent_to_state row",
                },
            )?;
            for column in 0..state_width {
                accumulator +=
                    self.parameters.recurrent_to_state[recurrent_row + column] * self.state[column];
                if !accumulator.is_finite() {
                    return Err(RecurrentReferenceError::NonFiniteIntermediate {
                        state_index: row,
                    });
                }
            }

            let input_row = row.checked_mul(input_width).ok_or(
                RecurrentReferenceError::HostLengthOverflow {
                    component: "input_to_state row",
                },
            )?;
            for (column, input_value) in input.iter().enumerate() {
                accumulator += self.parameters.input_to_state[input_row + column] * *input_value;
                if !accumulator.is_finite() {
                    return Err(RecurrentReferenceError::NonFiniteIntermediate {
                        state_index: row,
                    });
                }
            }
            *next_value = hard_tanh(accumulator);
        }
        Ok(next)
    }

    fn commit(&mut self, next: Vec<f64>) {
        self.state = next;
    }

    fn recurrent_state_bits(&self) -> Result<StorageBits, RecurrentReferenceError> {
        storage_bits_for_values(self.parameters.layout.state_width)
    }

    fn temporary_working_bits(&self) -> Result<StorageBits, RecurrentReferenceError> {
        storage_bits_for_values(self.parameters.layout.state_width)
    }

    fn static_parameter_bits(&self) -> Result<StorageBits, RecurrentReferenceError> {
        self.parameters.static_parameter_bits()
    }
}

/// Bounded recurrent-state-only A1 reference.
#[derive(Clone, Debug, PartialEq)]
pub struct A1Reference {
    core: BoundedRecurrentCore,
}

impl A1Reference {
    /// Construct a zero-state A1 reference.
    pub fn new(parameters: RecurrentParameters) -> Result<Self, RecurrentReferenceError> {
        Ok(Self {
            core: BoundedRecurrentCore::new(parameters)?,
        })
    }

    /// Current recurrent state.
    #[must_use]
    pub fn state(&self) -> &[f64] {
        self.core.state()
    }

    /// Advance one deterministic recurrent step.
    pub fn step(&mut self, input: &[f64]) -> Result<&[f64], RecurrentReferenceError> {
        let next = self.core.compute_next(input)?;
        self.core.commit(next);
        Ok(self.core.state())
    }

    /// Reset the persistent recurrent state to zero.
    pub fn reset(&mut self) {
        self.core.reset();
    }

    /// Exact architecture-level memory accounting for this A1 instance.
    pub fn memory_accounting(&self) -> Result<MemoryAccounting, RecurrentReferenceError> {
        let accounting = MemoryAccounting::zero()
            .with_recurrent_state(self.core.recurrent_state_bits()?)
            .with_temporary_working(self.core.temporary_working_bits()?)
            .with_static_parameters(self.core.static_parameter_bits()?);
        accounting.validate_for_arm(ReferenceArm::A1)?;
        Ok(accounting)
    }

    /// Clone a validated framework-independent A1 snapshot.
    pub fn snapshot(&self) -> Result<ReferenceSnapshot<Vec<f64>>, RecurrentReferenceError> {
        Ok(ReferenceSnapshot::new(
            ReferenceArm::A1,
            self.core.state.clone(),
            self.memory_accounting()?,
        )?)
    }
}

/// Owned read-status record emitted by one A2 step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum A2ReadStatus {
    /// The projected associative slot was empty.
    Empty {
        /// Direct-mapped address.
        address: u64,
    },
    /// The requested association was resident.
    Hit {
        /// Direct-mapped address.
        address: u64,
    },
    /// The projected slot was occupied by another key.
    CollisionMiss {
        /// Direct-mapped address.
        address: u64,
        /// Resident colliding key.
        resident_key: u64,
    },
}

/// Deterministic observable outcomes from one A2 step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct A2StepReport {
    read: A2ReadStatus,
    write: Option<AssociativeWriteOutcome>,
}

impl A2StepReport {
    /// Associative lookup status observed before any optional write.
    #[must_use]
    pub const fn read(self) -> A2ReadStatus {
        self.read
    }

    /// Optional write/replacement outcome after recurrent-memory fusion.
    #[must_use]
    pub const fn write(self) -> Option<AssociativeWriteOutcome> {
        self.write
    }
}

/// Complete A2 state used by TDI snapshots and later interventions.
#[derive(Clone, Debug, PartialEq)]
pub struct A2StateSnapshot {
    recurrent_state: Vec<f64>,
    associative_memory: DirectMappedAssociativeMemory,
}

impl A2StateSnapshot {
    /// Persistent recurrent coordinates.
    #[must_use]
    pub fn recurrent_state(&self) -> &[f64] {
        &self.recurrent_state
    }

    /// Complete bounded associative table state.
    #[must_use]
    pub const fn associative_memory(&self) -> &DirectMappedAssociativeMemory {
        &self.associative_memory
    }
}

/// Bounded A2 recurrent + associative-memory reference.
///
/// Each step performs lookup-before-write. A hit is fused coordinate-wise into
/// the candidate recurrent state as `hard_tanh(candidate + gain * payload)`.
/// An optional write then stores the fused recurrent state under `write_key`.
#[derive(Clone, Debug, PartialEq)]
pub struct A2Reference {
    core: BoundedRecurrentCore,
    memory: DirectMappedAssociativeMemory,
    fusion_gain: f64,
}

impl A2Reference {
    /// Construct a zero-state A2 reference with an empty bounded table.
    pub fn new(
        parameters: RecurrentParameters,
        memory_layout: AssociativeMemoryLayout,
        projection_seed: u64,
        fusion_gain: f64,
    ) -> Result<Self, RecurrentReferenceError> {
        if memory_layout.payload_width() != parameters.layout.state_width() {
            return Err(RecurrentReferenceError::AssociativePayloadWidthMismatch {
                state_width: parameters.layout.state_width(),
                payload_width: memory_layout.payload_width(),
            });
        }
        if !fusion_gain.is_finite() {
            return Err(RecurrentReferenceError::NonFiniteFusionGain);
        }
        Ok(Self {
            core: BoundedRecurrentCore::new(parameters)?,
            memory: DirectMappedAssociativeMemory::new(memory_layout, projection_seed)?,
            fusion_gain,
        })
    }

    /// Current recurrent state.
    #[must_use]
    pub fn state(&self) -> &[f64] {
        self.core.state()
    }

    /// Bounded associative memory used by A2.
    #[must_use]
    pub const fn associative_memory(&self) -> &DirectMappedAssociativeMemory {
        &self.memory
    }

    /// Advance one deterministic lookup-before-write A2 step.
    pub fn step(
        &mut self,
        input: &[f64],
        read_key: u64,
        write_key: Option<u64>,
    ) -> Result<A2StepReport, RecurrentReferenceError> {
        let mut next = self.core.compute_next(input)?;

        let read_status = match self.memory.read(read_key) {
            AssociativeRead::Empty { address } => A2ReadStatus::Empty { address },
            AssociativeRead::CollisionMiss {
                address,
                resident_key,
            } => A2ReadStatus::CollisionMiss {
                address,
                resident_key,
            },
            AssociativeRead::Hit { address, payload } => {
                for (index, (next_value, memory_value)) in
                    next.iter_mut().zip(payload.iter()).enumerate()
                {
                    let fused = *next_value + self.fusion_gain * *memory_value;
                    if !fused.is_finite() {
                        return Err(RecurrentReferenceError::NonFiniteIntermediate {
                            state_index: index,
                        });
                    }
                    *next_value = hard_tanh(fused);
                }
                A2ReadStatus::Hit { address }
            }
        };

        let write = if let Some(key) = write_key {
            Some(self.memory.write(key, &next)?)
        } else {
            None
        };
        self.core.commit(next);
        Ok(A2StepReport {
            read: read_status,
            write,
        })
    }

    /// Reset recurrent and associative persistent state while preserving static
    /// parameters, layout, projection seed and fusion gain.
    pub fn reset(&mut self) {
        self.core.reset();
        self.memory.clear();
    }

    /// Exact architecture-level memory accounting for this A2 instance.
    pub fn memory_accounting(&self) -> Result<MemoryAccounting, RecurrentReferenceError> {
        let associative = self.memory.storage_accounting()?;
        let recurrent_static = self.core.static_parameter_bits()?.get();
        let static_parameters = recurrent_static
            .checked_add(associative.static_parameter_bits().get())
            .and_then(|bits| bits.checked_add(FUSION_GAIN_STATIC_BITS))
            .ok_or(RecurrentReferenceError::AccountingOverflow)?;
        let accounting = MemoryAccounting::zero()
            .with_recurrent_state(self.core.recurrent_state_bits()?)
            .with_associative_payload(associative.payload_bits())
            .with_associative_metadata(associative.metadata_bits())
            .with_temporary_working(self.core.temporary_working_bits()?)
            .with_static_parameters(StorageBits::new(static_parameters));
        accounting.validate_for_arm(ReferenceArm::A2)?;
        Ok(accounting)
    }

    /// Clone a validated framework-independent A2 snapshot.
    pub fn snapshot(&self) -> Result<ReferenceSnapshot<A2StateSnapshot>, RecurrentReferenceError> {
        let state = A2StateSnapshot {
            recurrent_state: self.core.state.clone(),
            associative_memory: self.memory.clone(),
        };
        Ok(ReferenceSnapshot::new(
            ReferenceArm::A2,
            state,
            self.memory_accounting()?,
        )?)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        A1Reference, A2ReadStatus, A2Reference, RecurrentLayout, RecurrentParameters,
        RecurrentReferenceError,
    };
    use crate::ReferenceArm;
    use crate::associative_memory::{AssociativeMemoryLayout, AssociativeWriteOutcome};

    fn identity_parameters() -> RecurrentParameters {
        let layout = RecurrentLayout::new(2, 2).expect("valid fixture layout");
        RecurrentParameters::new(
            layout,
            vec![1.0, 0.0, 0.0, 1.0],
            vec![0.0, 0.0, 0.0, 0.0],
            vec![0.0, 0.0],
        )
        .expect("valid fixture parameters")
    }

    #[test]
    fn recurrent_core_is_exactly_deterministic_for_identical_inputs() {
        let mut left = A1Reference::new(identity_parameters()).expect("A1");
        let mut right = left.clone();

        left.step(&[0.25, -0.75]).expect("left step");
        right.step(&[0.25, -0.75]).expect("right step");

        let left_bits: Vec<_> = left.state().iter().map(|value| value.to_bits()).collect();
        let right_bits: Vec<_> = right.state().iter().map(|value| value.to_bits()).collect();
        assert_eq!(left_bits, right_bits);
    }

    #[test]
    fn a1_rejected_input_cannot_mutate_persistent_state() {
        let mut a1 = A1Reference::new(identity_parameters()).expect("A1");
        a1.step(&[0.5, -0.5]).expect("baseline step");
        let before = a1.state().to_vec();

        assert_eq!(
            a1.step(&[f64::NAN, 0.0]),
            Err(RecurrentReferenceError::NonFiniteInput { index: 0 })
        );
        assert_eq!(a1.state(), before.as_slice());
    }

    #[test]
    fn a1_accounting_is_valid_for_recurrent_only_arm() {
        let a1 = A1Reference::new(identity_parameters()).expect("A1");
        let accounting = a1.memory_accounting().expect("accounting");
        accounting
            .validate_for_arm(ReferenceArm::A1)
            .expect("valid A1 accounting");
        assert_eq!(accounting.recurrent_state().get(), 128);
        assert_eq!(accounting.temporary_working().get(), 128);
        assert_eq!(accounting.associative_payload().get(), 0);
    }

    #[test]
    fn a2_requires_associative_payload_width_equal_to_state_width() {
        let layout = AssociativeMemoryLayout::new(4, 1).expect("memory layout");
        assert!(matches!(
            A2Reference::new(identity_parameters(), layout, 7, 1.0),
            Err(RecurrentReferenceError::AssociativePayloadWidthMismatch {
                state_width: 2,
                payload_width: 1,
            })
        ));
    }

    #[test]
    fn a2_write_then_read_hit_fuses_resident_state() {
        let memory_layout = AssociativeMemoryLayout::new(8, 2).expect("memory layout");
        let mut a2 = A2Reference::new(identity_parameters(), memory_layout, 11, 1.0).expect("A2");

        let first = a2
            .step(&[0.5, -0.5], 7, Some(7))
            .expect("write association");
        assert!(matches!(
            first.write(),
            Some(AssociativeWriteOutcome::Inserted { .. })
        ));

        let second = a2.step(&[0.0, 0.0], 7, None).expect("read association");
        assert!(matches!(second.read(), A2ReadStatus::Hit { .. }));
        assert_eq!(a2.state(), &[0.5, -0.5]);
    }

    #[test]
    fn a2_lookup_happens_before_optional_write() {
        let memory_layout = AssociativeMemoryLayout::new(1, 2).expect("memory layout");
        let mut a2 = A2Reference::new(identity_parameters(), memory_layout, 13, 1.0).expect("A2");
        a2.step(&[0.25, 0.5], 1, Some(1)).expect("seed key 1");

        let report = a2
            .step(&[0.0, 0.0], 2, Some(2))
            .expect("colliding replacement");
        assert!(matches!(
            report.read(),
            A2ReadStatus::CollisionMiss {
                resident_key: 1,
                ..
            }
        ));
        assert!(matches!(
            report.write(),
            Some(AssociativeWriteOutcome::ReplacedCollision { evicted_key: 1, .. })
        ));
    }

    #[test]
    fn a2_accounting_includes_associative_payload_metadata_and_static_constants() {
        let memory_layout = AssociativeMemoryLayout::new(3, 2).expect("memory layout");
        let a2 = A2Reference::new(identity_parameters(), memory_layout, 17, 0.5).expect("A2");
        let accounting = a2.memory_accounting().expect("accounting");
        accounting
            .validate_for_arm(ReferenceArm::A2)
            .expect("valid A2 accounting");
        assert_eq!(accounting.recurrent_state().get(), 128);
        assert_eq!(accounting.associative_payload().get(), 384);
        assert_eq!(accounting.associative_metadata().get(), 344);
        assert!(accounting.static_parameters().get() > 0);
    }

    #[test]
    fn snapshots_capture_complete_persistent_reference_state() {
        let a1 = A1Reference::new(identity_parameters()).expect("A1");
        let a1_snapshot = a1.snapshot().expect("A1 snapshot");
        assert_eq!(a1_snapshot.arm(), ReferenceArm::A1);

        let memory_layout = AssociativeMemoryLayout::new(2, 2).expect("memory layout");
        let mut a2 = A2Reference::new(identity_parameters(), memory_layout, 19, 1.0).expect("A2");
        a2.step(&[0.5, 0.25], 9, Some(9)).expect("A2 step");
        let a2_snapshot = a2.snapshot().expect("A2 snapshot");
        assert_eq!(a2_snapshot.arm(), ReferenceArm::A2);
        assert_eq!(a2_snapshot.state().recurrent_state(), a2.state());
    }
}
