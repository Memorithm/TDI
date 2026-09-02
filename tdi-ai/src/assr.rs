//! TDI-8 reference-architecture contracts and exact memory accounting.
//!
//! This module deliberately contains no TDI-8.2 surface and no concrete
//! experiment dimensions. It provides the common, deterministic accounting
//! vocabulary needed by the bounded TDI-8.1 A0/A1/A2/A3 reference evaluator.

use core::fmt;

/// Frozen TDI-8 reference architecture ladder.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ReferenceArm {
    /// Competent attention-like full-history contextual reference.
    A0,
    /// Bounded recurrent-state-only reference.
    A1,
    /// Recurrent state plus bounded associative memory (ASSR candidate).
    A2,
    /// A2 plus a bounded VSA/holographic workspace (ASSR-H candidate).
    A3,
}

impl ReferenceArm {
    /// Stable short label used in machine-readable TDI-8 records.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::A0 => "A0",
            Self::A1 => "A1",
            Self::A2 => "A2",
            Self::A3 => "A3",
        }
    }

    /// Whether the arm may contain bounded associative payload/metadata.
    #[must_use]
    pub const fn permits_associative_memory(self) -> bool {
        matches!(self, Self::A2 | Self::A3)
    }

    /// Whether the arm may contain a VSA workspace.
    #[must_use]
    pub const fn permits_vsa_workspace(self) -> bool {
        matches!(self, Self::A3)
    }

    /// Whether cumulative full-history storage belongs to the arm semantics.
    #[must_use]
    pub const fn permits_full_history(self) -> bool {
        matches!(self, Self::A0)
    }
}

/// Exact storage quantity measured in bits.
///
/// `u128` is used so accounting is not silently truncated to host pointer size.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StorageBits(u128);

impl StorageBits {
    /// Exact zero-bit quantity.
    pub const ZERO: Self = Self(0);

    /// Construct an exact bit count.
    #[must_use]
    pub const fn new(bits: u128) -> Self {
        Self(bits)
    }

    /// Raw exact bit count.
    #[must_use]
    pub const fn get(self) -> u128 {
        self.0
    }

    fn checked_add(self, other: Self) -> Result<Self, MemoryAccountingError> {
        self.0
            .checked_add(other.0)
            .map(Self)
            .ok_or(MemoryAccountingError::Overflow)
    }
}

impl From<u64> for StorageBits {
    fn from(value: u64) -> Self {
        Self(u128::from(value))
    }
}

/// Named memory component used by validation diagnostics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryComponent {
    /// Persistent recurrent state.
    RecurrentState,
    /// Values stored in the bounded associative memory.
    AssociativePayload,
    /// Tags, addresses, occupancy and replacement metadata.
    AssociativeMetadata,
    /// Bounded VSA/holographic workspace.
    VsaWorkspace,
    /// Temporary working storage needed by one reference step.
    TemporaryWorking,
    /// Full-history storage retained by A0.
    CumulativeHistory,
    /// Static parameters and constant tables.
    StaticParameters,
}

impl MemoryComponent {
    /// Stable component label for provenance/error records.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::RecurrentState => "recurrent_state",
            Self::AssociativePayload => "associative_payload",
            Self::AssociativeMetadata => "associative_metadata",
            Self::VsaWorkspace => "vsa_workspace",
            Self::TemporaryWorking => "temporary_working",
            Self::CumulativeHistory => "cumulative_history",
            Self::StaticParameters => "static_parameters",
        }
    }
}

/// Fail-closed validation errors for TDI-8 reference memory accounting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryAccountingError {
    /// Summing exact storage components exceeded the accounting representation.
    Overflow,
    /// An architecture omitted storage required by its frozen defining
    /// mechanism.
    RequiredComponentMissing {
        /// Architecture whose accounting was invalid.
        arm: ReferenceArm,
        /// Required component that had zero allocated bits.
        component: MemoryComponent,
    },
    /// An architecture reported storage that its frozen reference semantics do
    /// not permit.
    ComponentNotAllowed {
        /// Architecture whose accounting was invalid.
        arm: ReferenceArm,
        /// Forbidden component.
        component: MemoryComponent,
        /// Non-zero amount that triggered the failure.
        bits: StorageBits,
    },
    /// A1/A2/A3 did not use the same budgeted dynamic-memory total.
    DynamicBudgetMismatch {
        /// A1 budgeted dynamic bits.
        a1: StorageBits,
        /// A2 budgeted dynamic bits.
        a2: StorageBits,
        /// A3 budgeted dynamic bits.
        a3: StorageBits,
    },
}

impl fmt::Display for MemoryAccountingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Overflow => formatter.write_str("TDI-8 memory accounting overflow"),
            Self::RequiredComponentMissing { arm, component } => write!(
                formatter,
                "{} requires non-zero {} storage",
                arm.label(),
                component.label()
            ),
            Self::ComponentNotAllowed {
                arm,
                component,
                bits,
            } => write!(
                formatter,
                "{} does not permit {} storage ({} bits reported)",
                arm.label(),
                component.label(),
                bits.get()
            ),
            Self::DynamicBudgetMismatch { a1, a2, a3 } => write!(
                formatter,
                "matched dynamic-memory budget mismatch: A1={} bits, A2={} bits, A3={} bits",
                a1.get(),
                a2.get(),
                a3.get()
            ),
        }
    }
}

impl std::error::Error for MemoryAccountingError {}

/// Exact, component-wise memory accounting for one reference arm.
///
/// The frozen matched A1/A2/A3 total dynamic-memory budget is the sum of
/// recurrent state, associative payload, associative metadata, VSA workspace
/// and temporary working storage. A0 cumulative history and static parameters
/// are reported separately rather than silently folded into that matched
/// bounded-arm budget.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MemoryAccounting {
    recurrent_state: StorageBits,
    associative_payload: StorageBits,
    associative_metadata: StorageBits,
    vsa_workspace: StorageBits,
    temporary_working: StorageBits,
    cumulative_history: StorageBits,
    static_parameters: StorageBits,
}

impl MemoryAccounting {
    /// Start an accounting record with every component equal to zero.
    #[must_use]
    pub const fn zero() -> Self {
        Self {
            recurrent_state: StorageBits::ZERO,
            associative_payload: StorageBits::ZERO,
            associative_metadata: StorageBits::ZERO,
            vsa_workspace: StorageBits::ZERO,
            temporary_working: StorageBits::ZERO,
            cumulative_history: StorageBits::ZERO,
            static_parameters: StorageBits::ZERO,
        }
    }

    /// Set persistent recurrent-state storage.
    #[must_use]
    pub const fn with_recurrent_state(mut self, bits: StorageBits) -> Self {
        self.recurrent_state = bits;
        self
    }

    /// Set associative payload storage.
    #[must_use]
    pub const fn with_associative_payload(mut self, bits: StorageBits) -> Self {
        self.associative_payload = bits;
        self
    }

    /// Set associative metadata/addressing storage.
    #[must_use]
    pub const fn with_associative_metadata(mut self, bits: StorageBits) -> Self {
        self.associative_metadata = bits;
        self
    }

    /// Set bounded VSA workspace storage.
    #[must_use]
    pub const fn with_vsa_workspace(mut self, bits: StorageBits) -> Self {
        self.vsa_workspace = bits;
        self
    }

    /// Set temporary working storage required by a reference step.
    #[must_use]
    pub const fn with_temporary_working(mut self, bits: StorageBits) -> Self {
        self.temporary_working = bits;
        self
    }

    /// Set cumulative full-history storage (A0 only).
    #[must_use]
    pub const fn with_cumulative_history(mut self, bits: StorageBits) -> Self {
        self.cumulative_history = bits;
        self
    }

    /// Set static parameter/constant-table storage.
    #[must_use]
    pub const fn with_static_parameters(mut self, bits: StorageBits) -> Self {
        self.static_parameters = bits;
        self
    }

    /// Persistent recurrent-state bits.
    #[must_use]
    pub const fn recurrent_state(self) -> StorageBits {
        self.recurrent_state
    }

    /// Associative payload bits.
    #[must_use]
    pub const fn associative_payload(self) -> StorageBits {
        self.associative_payload
    }

    /// Associative metadata/addressing bits.
    #[must_use]
    pub const fn associative_metadata(self) -> StorageBits {
        self.associative_metadata
    }

    /// VSA workspace bits.
    #[must_use]
    pub const fn vsa_workspace(self) -> StorageBits {
        self.vsa_workspace
    }

    /// Temporary working bits.
    #[must_use]
    pub const fn temporary_working(self) -> StorageBits {
        self.temporary_working
    }

    /// A0 cumulative-history bits.
    #[must_use]
    pub const fn cumulative_history(self) -> StorageBits {
        self.cumulative_history
    }

    /// Static parameter/constant-table bits.
    #[must_use]
    pub const fn static_parameters(self) -> StorageBits {
        self.static_parameters
    }

    /// Matched total dynamic-memory budget used by A1/A2/A3.
    pub fn budgeted_dynamic_bits(self) -> Result<StorageBits, MemoryAccountingError> {
        self.recurrent_state
            .checked_add(self.associative_payload)?
            .checked_add(self.associative_metadata)?
            .checked_add(self.vsa_workspace)?
            .checked_add(self.temporary_working)
    }

    /// All live/reference storage reported for the step, excluding static
    /// parameters but including A0 cumulative history.
    pub fn reported_dynamic_bits(self) -> Result<StorageBits, MemoryAccountingError> {
        self.budgeted_dynamic_bits()?
            .checked_add(self.cumulative_history)
    }

    /// Validate both forbidden and defining non-zero components for the frozen
    /// reference arm semantics.
    pub fn validate_for_arm(self, arm: ReferenceArm) -> Result<(), MemoryAccountingError> {
        if !arm.permits_associative_memory() {
            require_zero(
                arm,
                MemoryComponent::AssociativePayload,
                self.associative_payload,
            )?;
            require_zero(
                arm,
                MemoryComponent::AssociativeMetadata,
                self.associative_metadata,
            )?;
        }
        if !arm.permits_vsa_workspace() {
            require_zero(arm, MemoryComponent::VsaWorkspace, self.vsa_workspace)?;
        }
        if !arm.permits_full_history() {
            require_zero(
                arm,
                MemoryComponent::CumulativeHistory,
                self.cumulative_history,
            )?;
        }

        if matches!(arm, ReferenceArm::A1 | ReferenceArm::A2 | ReferenceArm::A3) {
            require_nonzero(arm, MemoryComponent::RecurrentState, self.recurrent_state)?;
        }
        if arm.permits_associative_memory() {
            require_nonzero(
                arm,
                MemoryComponent::AssociativePayload,
                self.associative_payload,
            )?;
        }
        if arm.permits_vsa_workspace() {
            require_nonzero(arm, MemoryComponent::VsaWorkspace, self.vsa_workspace)?;
        }

        Ok(())
    }
}

fn require_zero(
    arm: ReferenceArm,
    component: MemoryComponent,
    bits: StorageBits,
) -> Result<(), MemoryAccountingError> {
    if bits == StorageBits::ZERO {
        Ok(())
    } else {
        Err(MemoryAccountingError::ComponentNotAllowed {
            arm,
            component,
            bits,
        })
    }
}

fn require_nonzero(
    arm: ReferenceArm,
    component: MemoryComponent,
    bits: StorageBits,
) -> Result<(), MemoryAccountingError> {
    if bits == StorageBits::ZERO {
        Err(MemoryAccountingError::RequiredComponentMissing { arm, component })
    } else {
        Ok(())
    }
}

/// Validated common total dynamic-memory budget for the A1/A2/A3 primary
/// contrast.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MatchedDynamicBudget {
    bits: StorageBits,
}

impl MatchedDynamicBudget {
    /// Validate arm-specific accounting and exact A1/A2/A3 budget equality.
    pub fn validate(
        a1: MemoryAccounting,
        a2: MemoryAccounting,
        a3: MemoryAccounting,
    ) -> Result<Self, MemoryAccountingError> {
        a1.validate_for_arm(ReferenceArm::A1)?;
        a2.validate_for_arm(ReferenceArm::A2)?;
        a3.validate_for_arm(ReferenceArm::A3)?;

        let a1_bits = a1.budgeted_dynamic_bits()?;
        let a2_bits = a2.budgeted_dynamic_bits()?;
        let a3_bits = a3.budgeted_dynamic_bits()?;
        if a1_bits != a2_bits || a1_bits != a3_bits {
            return Err(MemoryAccountingError::DynamicBudgetMismatch {
                a1: a1_bits,
                a2: a2_bits,
                a3: a3_bits,
            });
        }

        Ok(Self { bits: a1_bits })
    }

    /// Exact matched budget in bits.
    #[must_use]
    pub const fn bits(self) -> StorageBits {
        self.bits
    }
}

/// Framework-independent state snapshot annotated with its reference arm and
/// exact memory accounting.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceSnapshot<S> {
    arm: ReferenceArm,
    state: S,
    memory: MemoryAccounting,
}

impl<S> ReferenceSnapshot<S> {
    /// Build a snapshot after validating its arm-specific accounting.
    pub fn new(
        arm: ReferenceArm,
        state: S,
        memory: MemoryAccounting,
    ) -> Result<Self, MemoryAccountingError> {
        memory.validate_for_arm(arm)?;
        Ok(Self { arm, state, memory })
    }

    /// Frozen architecture arm associated with the state.
    #[must_use]
    pub const fn arm(&self) -> ReferenceArm {
        self.arm
    }

    /// Mechanism-specific state payload.
    #[must_use]
    pub const fn state(&self) -> &S {
        &self.state
    }

    /// Exact memory-accounting record associated with this state.
    #[must_use]
    pub const fn memory(&self) -> MemoryAccounting {
        self.memory
    }

    /// Consume the snapshot and return the mechanism-specific state.
    #[must_use]
    pub fn into_state(self) -> S {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::{
        MatchedDynamicBudget, MemoryAccounting, MemoryAccountingError, MemoryComponent,
        ReferenceArm, ReferenceSnapshot, StorageBits,
    };

    #[test]
    fn architecture_ladder_has_stable_semantics() {
        assert_eq!(ReferenceArm::A0.label(), "A0");
        assert!(ReferenceArm::A0.permits_full_history());
        assert!(!ReferenceArm::A1.permits_associative_memory());
        assert!(ReferenceArm::A2.permits_associative_memory());
        assert!(!ReferenceArm::A2.permits_vsa_workspace());
        assert!(ReferenceArm::A3.permits_vsa_workspace());
    }

    #[test]
    fn a1_rejects_associative_storage() {
        let memory = MemoryAccounting::zero()
            .with_recurrent_state(StorageBits::new(512))
            .with_associative_payload(StorageBits::new(64));
        assert_eq!(
            memory.validate_for_arm(ReferenceArm::A1),
            Err(MemoryAccountingError::ComponentNotAllowed {
                arm: ReferenceArm::A1,
                component: MemoryComponent::AssociativePayload,
                bits: StorageBits::new(64),
            })
        );
    }

    #[test]
    fn defining_components_must_have_nonzero_storage() {
        let a1 = MemoryAccounting::zero();
        assert_eq!(
            a1.validate_for_arm(ReferenceArm::A1),
            Err(MemoryAccountingError::RequiredComponentMissing {
                arm: ReferenceArm::A1,
                component: MemoryComponent::RecurrentState,
            })
        );

        let a2 = MemoryAccounting::zero().with_recurrent_state(StorageBits::new(64));
        assert_eq!(
            a2.validate_for_arm(ReferenceArm::A2),
            Err(MemoryAccountingError::RequiredComponentMissing {
                arm: ReferenceArm::A2,
                component: MemoryComponent::AssociativePayload,
            })
        );

        let a3 = MemoryAccounting::zero()
            .with_recurrent_state(StorageBits::new(32))
            .with_associative_payload(StorageBits::new(32));
        assert_eq!(
            a3.validate_for_arm(ReferenceArm::A3),
            Err(MemoryAccountingError::RequiredComponentMissing {
                arm: ReferenceArm::A3,
                component: MemoryComponent::VsaWorkspace,
            })
        );
    }

    #[test]
    fn matched_budget_accepts_different_partitions_of_the_same_bits() {
        let a1 = MemoryAccounting::zero().with_recurrent_state(StorageBits::new(1024));
        let a2 = MemoryAccounting::zero()
            .with_recurrent_state(StorageBits::new(512))
            .with_associative_payload(StorageBits::new(448))
            .with_associative_metadata(StorageBits::new(64));
        let a3 = MemoryAccounting::zero()
            .with_recurrent_state(StorageBits::new(384))
            .with_associative_payload(StorageBits::new(384))
            .with_associative_metadata(StorageBits::new(64))
            .with_vsa_workspace(StorageBits::new(192));

        let matched = MatchedDynamicBudget::validate(a1, a2, a3).expect("matched budget");
        assert_eq!(matched.bits(), StorageBits::new(1024));
    }

    #[test]
    fn temporary_working_storage_participates_in_matched_budget() {
        let a1 = MemoryAccounting::zero()
            .with_recurrent_state(StorageBits::new(64))
            .with_temporary_working(StorageBits::new(8));
        let a2 = MemoryAccounting::zero()
            .with_recurrent_state(StorageBits::new(32))
            .with_associative_payload(StorageBits::new(32));
        let a3 = MemoryAccounting::zero()
            .with_recurrent_state(StorageBits::new(32))
            .with_associative_payload(StorageBits::new(16))
            .with_vsa_workspace(StorageBits::new(16));

        assert_eq!(
            MatchedDynamicBudget::validate(a1, a2, a3),
            Err(MemoryAccountingError::DynamicBudgetMismatch {
                a1: StorageBits::new(72),
                a2: StorageBits::new(64),
                a3: StorageBits::new(64),
            })
        );
    }

    #[test]
    fn matched_budget_rejects_unequal_totals() {
        let a1 = MemoryAccounting::zero().with_recurrent_state(StorageBits::new(64));
        let a2 = MemoryAccounting::zero()
            .with_recurrent_state(StorageBits::new(32))
            .with_associative_payload(StorageBits::new(32));
        let a3 = MemoryAccounting::zero()
            .with_recurrent_state(StorageBits::new(32))
            .with_associative_payload(StorageBits::new(16))
            .with_vsa_workspace(StorageBits::new(8));

        assert_eq!(
            MatchedDynamicBudget::validate(a1, a2, a3),
            Err(MemoryAccountingError::DynamicBudgetMismatch {
                a1: StorageBits::new(64),
                a2: StorageBits::new(64),
                a3: StorageBits::new(56),
            })
        );
    }

    #[test]
    fn temporary_static_and_history_are_reported_without_double_counting() {
        let a0 = MemoryAccounting::zero()
            .with_recurrent_state(StorageBits::new(64))
            .with_temporary_working(StorageBits::new(32))
            .with_cumulative_history(StorageBits::new(2048))
            .with_static_parameters(StorageBits::new(4096));
        a0.validate_for_arm(ReferenceArm::A0)
            .expect("A0 may retain history");
        assert_eq!(
            a0.budgeted_dynamic_bits().expect("finite sum"),
            StorageBits::new(96)
        );
        assert_eq!(
            a0.reported_dynamic_bits().expect("finite sum"),
            StorageBits::new(2144)
        );
        assert_eq!(a0.static_parameters(), StorageBits::new(4096));
    }

    #[test]
    fn bounded_arms_reject_full_history_storage() {
        let memory = MemoryAccounting::zero()
            .with_recurrent_state(StorageBits::new(64))
            .with_associative_payload(StorageBits::new(64))
            .with_cumulative_history(StorageBits::new(64));
        assert_eq!(
            memory.validate_for_arm(ReferenceArm::A2),
            Err(MemoryAccountingError::ComponentNotAllowed {
                arm: ReferenceArm::A2,
                component: MemoryComponent::CumulativeHistory,
                bits: StorageBits::new(64),
            })
        );
    }

    #[test]
    fn exact_accounting_fails_closed_on_u128_overflow() {
        let memory = MemoryAccounting::zero()
            .with_recurrent_state(StorageBits::new(u128::MAX))
            .with_associative_payload(StorageBits::new(1));
        assert_eq!(
            memory.budgeted_dynamic_bits(),
            Err(MemoryAccountingError::Overflow)
        );
    }

    #[test]
    fn snapshot_validates_arm_accounting_before_exposing_state() {
        let memory = MemoryAccounting::zero()
            .with_recurrent_state(StorageBits::new(64))
            .with_associative_payload(StorageBits::new(64))
            .with_vsa_workspace(StorageBits::new(64));
        let snapshot =
            ReferenceSnapshot::new(ReferenceArm::A3, vec![1.0, -1.0], memory).expect("A3 snapshot");
        assert_eq!(snapshot.arm(), ReferenceArm::A3);
        assert_eq!(snapshot.state(), &[1.0, -1.0]);
        assert_eq!(snapshot.memory(), memory);
    }
}
