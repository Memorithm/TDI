//! Deterministic bounded A3 / ASSR-H reference semantics for TDI-8.1.
//!
//! A3 composes the merged A2 recurrent+associative reference with the merged
//! bounded VSA workspace oracle. The integration rule is explicit and
//! deterministic but does not freeze any experimental dimension, seed, budget
//! or task parameter, and it creates no TDI-8.2 surface.

use core::fmt;

use crate::associative_memory::AssociativeMemoryLayout;
use crate::assr_reference::{
    A2Reference, A2StateSnapshot, A2StepReport, RecurrentParameters, RecurrentReferenceError,
};
use crate::vsa_workspace::{BoundedVsaWorkspace, VsaWorkspaceError, VsaWorkspaceLayout};
use crate::{
    MemoryAccounting, MemoryAccountingError, ReferenceArm, ReferenceSnapshot, StorageBits,
};

const VSA_FUSION_GAIN_STATIC_BITS: u128 = 64;

/// Fail-closed errors from the bounded A3 reference mechanism.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum A3ReferenceError {
    /// The VSA workspace width must equal the recurrent input width because the
    /// A3 integration fuses the VSA readout coordinate-wise into the A2 input.
    VsaInputWidthMismatch {
        /// Recurrent input width.
        input_width: u64,
        /// VSA workspace width.
        vsa_width: u64,
    },
    /// The A3 VSA-to-input fusion gain must be finite.
    NonFiniteVsaFusionGain,
    /// A runtime input vector has the wrong width.
    InputWidthMismatch {
        /// Required width.
        expected: usize,
        /// Supplied width.
        actual: usize,
    },
    /// A runtime input contains a non-finite coordinate.
    NonFiniteInput {
        /// Invalid coordinate.
        index: usize,
    },
    /// Coordinate-wise VSA/input fusion produced a non-finite value.
    NonFiniteFusedInput {
        /// Invalid coordinate.
        index: usize,
    },
    /// Exact A3 accounting overflowed `u128`.
    AccountingOverflow,
    /// Underlying A2 recurrent/associative failure.
    A2(RecurrentReferenceError),
    /// Underlying bounded VSA workspace failure.
    Vsa(VsaWorkspaceError),
    /// Common TDI-8 memory-accounting validation failure.
    MemoryAccounting(MemoryAccountingError),
}

impl fmt::Display for A3ReferenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::VsaInputWidthMismatch {
                input_width,
                vsa_width,
            } => write!(
                formatter,
                "A3 VSA width {vsa_width} must equal recurrent input width {input_width}"
            ),
            Self::NonFiniteVsaFusionGain => {
                formatter.write_str("A3 VSA fusion gain must be finite")
            }
            Self::InputWidthMismatch { expected, actual } => write!(
                formatter,
                "A3 input width mismatch: expected {expected}, got {actual}"
            ),
            Self::NonFiniteInput { index } => {
                write!(formatter, "A3 input coordinate {index} is not finite")
            }
            Self::NonFiniteFusedInput { index } => write!(
                formatter,
                "A3 fused input coordinate {index} became non-finite"
            ),
            Self::AccountingOverflow => formatter.write_str("A3 bit accounting overflow"),
            Self::A2(error) => write!(formatter, "A2 reference: {error}"),
            Self::Vsa(error) => write!(formatter, "VSA workspace: {error}"),
            Self::MemoryAccounting(error) => write!(formatter, "memory accounting: {error}"),
        }
    }
}

impl std::error::Error for A3ReferenceError {}

impl From<RecurrentReferenceError> for A3ReferenceError {
    fn from(error: RecurrentReferenceError) -> Self {
        Self::A2(error)
    }
}

impl From<VsaWorkspaceError> for A3ReferenceError {
    fn from(error: VsaWorkspaceError) -> Self {
        Self::Vsa(error)
    }
}

impl From<MemoryAccountingError> for A3ReferenceError {
    fn from(error: MemoryAccountingError) -> Self {
        Self::MemoryAccounting(error)
    }
}

fn checked_add_bits(
    left: StorageBits,
    right: StorageBits,
) -> Result<StorageBits, A3ReferenceError> {
    let bits = left
        .get()
        .checked_add(right.get())
        .ok_or(A3ReferenceError::AccountingOverflow)?;
    Ok(StorageBits::new(bits))
}

/// Complete persistent A3 state used by TDI snapshots and later interventions.
#[derive(Clone, Debug, PartialEq)]
pub struct A3StateSnapshot {
    a2: A2StateSnapshot,
    vsa_workspace: Vec<f64>,
}

impl A3StateSnapshot {
    /// Complete recurrent+associative A2 persistent state.
    #[must_use]
    pub const fn a2(&self) -> &A2StateSnapshot {
        &self.a2
    }

    /// Persistent VSA superposition coordinates.
    #[must_use]
    pub fn vsa_workspace(&self) -> &[f64] {
        &self.vsa_workspace
    }
}

/// Explicit VSA-read routing for one bounded A3 transition.
///
/// The associative A2 read key is supplied independently to [`A3Reference::step_routed`].
/// `Skip` means the persistent VSA workspace must not influence this transition;
/// `Key` performs the existing deterministic unbind/fuse operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum A3VsaReadRoute {
    /// Do not read or fuse the VSA workspace for this transition.
    Skip,
    /// Unbind the VSA workspace with this logical key before the A2 step.
    Key(u64),
}

/// Bounded A3 reference: A2 plus one explicit VSA workspace.
///
/// The legacy [`Self::step`] method preserves the original software-oracle
/// behavior where one `read_key` drives both VSA retrieval and the A2 lookup.
/// [`Self::step_routed`] exposes those two routing decisions separately so later
/// bounded task adapters do not have to create accidental VSA cross-talk merely
/// to preserve an independently qualified A2 read policy.
///
/// The VSA workspace is written only through [`Self::store_vsa`]. That operation
/// remains deliberately separate from transition execution. The later bounded
/// evaluator owns the task-level policy deciding when and what to store.
#[derive(Clone, Debug, PartialEq)]
pub struct A3Reference {
    a2: A2Reference,
    workspace: BoundedVsaWorkspace,
    vsa_fusion_gain: f64,
}

impl A3Reference {
    /// Construct a zero-state A3 reference from explicit, still-unfrozen
    /// mechanism parameters.
    pub fn new(
        recurrent_parameters: RecurrentParameters,
        associative_layout: AssociativeMemoryLayout,
        associative_projection_seed: u64,
        associative_fusion_gain: f64,
        vsa_layout: VsaWorkspaceLayout,
        vsa_role_seed: u64,
        vsa_fusion_gain: f64,
    ) -> Result<Self, A3ReferenceError> {
        let input_width = recurrent_parameters.layout().input_width();
        if vsa_layout.width() != input_width {
            return Err(A3ReferenceError::VsaInputWidthMismatch {
                input_width,
                vsa_width: vsa_layout.width(),
            });
        }
        if !vsa_fusion_gain.is_finite() {
            return Err(A3ReferenceError::NonFiniteVsaFusionGain);
        }

        Ok(Self {
            a2: A2Reference::new(
                recurrent_parameters,
                associative_layout,
                associative_projection_seed,
                associative_fusion_gain,
            )?,
            workspace: BoundedVsaWorkspace::new(vsa_layout, vsa_role_seed)?,
            vsa_fusion_gain,
        })
    }

    /// Embedded A2 recurrent+associative reference.
    #[must_use]
    pub const fn a2(&self) -> &A2Reference {
        &self.a2
    }

    /// Current recurrent state after A3 integration.
    #[must_use]
    pub fn state(&self) -> &[f64] {
        self.a2.state()
    }

    /// Bounded persistent VSA workspace.
    #[must_use]
    pub const fn workspace(&self) -> &BoundedVsaWorkspace {
        &self.workspace
    }

    /// VSA-to-input fusion gain.
    #[must_use]
    pub const fn vsa_fusion_gain(&self) -> f64 {
        self.vsa_fusion_gain
    }

    /// Execute the original integrated A3 read/fuse/A2 step.
    ///
    /// This compatibility wrapper preserves the original contract exactly: the
    /// same `read_key` drives VSA unbinding and the A2 associative lookup.
    pub fn step(
        &mut self,
        input: &[f64],
        read_key: u64,
        write_key: Option<u64>,
    ) -> Result<A2StepReport, A3ReferenceError> {
        self.step_routed(input, A3VsaReadRoute::Key(read_key), read_key, write_key)
    }

    /// Execute one A3 transition with independent VSA and A2 read routing.
    ///
    /// Input shape and finiteness are always validated first. With
    /// [`A3VsaReadRoute::Skip`], the VSA workspace is neither unbound nor fused
    /// and the exact external input is passed to the unchanged A2 reference.
    /// With [`A3VsaReadRoute::Key`], VSA retrieval is read-only and all fallible
    /// allocation/fusion checks complete before the mutating A2 step begins.
    pub fn step_routed(
        &mut self,
        input: &[f64],
        vsa_read: A3VsaReadRoute,
        a2_read_key: u64,
        a2_write_key: Option<u64>,
    ) -> Result<A2StepReport, A3ReferenceError> {
        let expected = self.workspace.components().len();
        if input.len() != expected {
            return Err(A3ReferenceError::InputWidthMismatch {
                expected,
                actual: input.len(),
            });
        }
        if let Some(index) = input.iter().position(|value| !value.is_finite()) {
            return Err(A3ReferenceError::NonFiniteInput { index });
        }

        match vsa_read {
            A3VsaReadRoute::Skip => Ok(self.a2.step(input, a2_read_key, a2_write_key)?),
            A3VsaReadRoute::Key(vsa_read_key) => {
                let mut fused_input = self.workspace.unbind(vsa_read_key)?;
                for (index, (fused, input_value)) in
                    fused_input.iter_mut().zip(input.iter()).enumerate()
                {
                    let value = *input_value + self.vsa_fusion_gain * *fused;
                    if !value.is_finite() {
                        return Err(A3ReferenceError::NonFiniteFusedInput { index });
                    }
                    *fused = value;
                }
                Ok(self
                    .a2
                    .step(&fused_input, a2_read_key, a2_write_key)?)
            }
        }
    }

    /// Atomically bind and superpose one finite payload under one deterministic
    /// VSA role. This operation does not mutate A2 state.
    pub fn store_vsa(&mut self, key: u64, payload: &[f64]) -> Result<(), A3ReferenceError> {
        self.workspace.bundle(key, payload)?;
        Ok(())
    }

    /// Read the current VSA superposition without mutating A3 state.
    pub fn retrieve_vsa(&self, key: u64) -> Result<Vec<f64>, A3ReferenceError> {
        Ok(self.workspace.unbind(key)?)
    }

    /// Reset recurrent, associative and VSA persistent state while preserving
    /// every declared static parameter and layout.
    pub fn reset(&mut self) {
        self.a2.reset();
        self.workspace.clear();
    }

    /// Exact architecture-level memory accounting for this A3 instance.
    ///
    /// The integrated VSA-read path keeps one VSA-width readout/fused-input
    /// vector alive while A2 computes its state-width candidate vector, so the
    /// declared temporary component remains the maximum across admissible routed
    /// steps: A2 temporary storage plus one standalone VSA temporary vector.
    /// Static accounting additionally records the explicit A3 fusion gain.
    pub fn memory_accounting(&self) -> Result<MemoryAccounting, A3ReferenceError> {
        let a2 = self.a2.memory_accounting()?;
        let vsa = self.workspace.storage_accounting()?;
        let temporary = checked_add_bits(a2.temporary_working(), vsa.temporary_working_bits())?;
        let static_parameters =
            checked_add_bits(a2.static_parameters(), vsa.static_parameter_bits())?;
        let static_parameters = checked_add_bits(
            static_parameters,
            StorageBits::new(VSA_FUSION_GAIN_STATIC_BITS),
        )?;

        let accounting = MemoryAccounting::zero()
            .with_recurrent_state(a2.recurrent_state())
            .with_associative_payload(a2.associative_payload())
            .with_associative_metadata(a2.associative_metadata())
            .with_vsa_workspace(vsa.workspace_bits())
            .with_temporary_working(temporary)
            .with_static_parameters(static_parameters);
        accounting.validate_for_arm(ReferenceArm::A3)?;
        Ok(accounting)
    }

    /// Clone a validated complete A3 persistent-state snapshot.
    pub fn snapshot(&self) -> Result<ReferenceSnapshot<A3StateSnapshot>, A3ReferenceError> {
        let a2_snapshot = self.a2.snapshot()?;
        let state = A3StateSnapshot {
            a2: a2_snapshot.state().clone(),
            vsa_workspace: self.workspace.components().to_vec(),
        };
        Ok(ReferenceSnapshot::new(
            ReferenceArm::A3,
            state,
            self.memory_accounting()?,
        )?)
    }
}

#[cfg(test)]
mod tests {
    use super::{A3Reference, A3ReferenceError, A3VsaReadRoute};
    use crate::associative_memory::AssociativeMemoryLayout;
    use crate::assr_reference::{
        A1Reference, A2ReadStatus, A2Reference, RecurrentLayout, RecurrentParameters,
    };
    use crate::vsa_workspace::VsaWorkspaceLayout;
    use crate::{MatchedDynamicBudget, ReferenceArm};

    fn parameters(input_width: u64, state_width: u64) -> RecurrentParameters {
        let layout =
            RecurrentLayout::new(input_width, state_width).expect("valid recurrent layout");
        let input_width = usize::try_from(input_width).expect("fixture input width");
        let state_width = usize::try_from(state_width).expect("fixture state width");
        RecurrentParameters::new(
            layout,
            vec![0.0; state_width * input_width],
            vec![0.0; state_width * state_width],
            vec![0.0; state_width],
        )
        .expect("finite zero parameters")
    }

    fn identity_parameters() -> RecurrentParameters {
        let layout = RecurrentLayout::new(2, 2).expect("valid fixture layout");
        RecurrentParameters::new(layout, vec![1.0, 0.0, 0.0, 1.0], vec![0.0; 4], vec![0.0; 2])
            .expect("identity parameters")
    }

    fn a3() -> A3Reference {
        A3Reference::new(
            identity_parameters(),
            AssociativeMemoryLayout::new(8, 2).expect("associative layout"),
            11,
            1.0,
            VsaWorkspaceLayout::new(2).expect("VSA layout"),
            23,
            1.0,
        )
        .expect("A3")
    }

    #[test]
    fn constructor_requires_input_vsa_width_match_and_finite_gain() {
        assert!(matches!(
            A3Reference::new(
                identity_parameters(),
                AssociativeMemoryLayout::new(8, 2).expect("associative layout"),
                11,
                1.0,
                VsaWorkspaceLayout::new(3).expect("VSA layout"),
                23,
                1.0,
            ),
            Err(A3ReferenceError::VsaInputWidthMismatch {
                input_width: 2,
                vsa_width: 3,
            })
        ));
        assert!(matches!(
            A3Reference::new(
                identity_parameters(),
                AssociativeMemoryLayout::new(8, 2).expect("associative layout"),
                11,
                1.0,
                VsaWorkspaceLayout::new(2).expect("VSA layout"),
                23,
                f64::NAN,
            ),
            Err(A3ReferenceError::NonFiniteVsaFusionGain)
        ));
    }

    #[test]
    fn empty_vsa_workspace_preserves_a2_step_semantics_bit_exactly() {
        let memory_layout = AssociativeMemoryLayout::new(8, 2).expect("associative layout");
        let mut a2 = A2Reference::new(identity_parameters(), memory_layout, 11, 1.0).expect("A2");
        let mut a3 = A3Reference::new(
            identity_parameters(),
            memory_layout,
            11,
            1.0,
            VsaWorkspaceLayout::new(2).expect("VSA layout"),
            23,
            0.75,
        )
        .expect("A3");

        let a2_report = a2.step(&[0.25, -0.5], 7, Some(7)).expect("A2 step");
        let a3_report = a3.step(&[0.25, -0.5], 7, Some(7)).expect("A3 step");
        assert_eq!(a3_report, a2_report);
        let a2_bits: Vec<_> = a2.state().iter().map(|value| value.to_bits()).collect();
        let a3_bits: Vec<_> = a3.state().iter().map(|value| value.to_bits()).collect();
        assert_eq!(a3_bits, a2_bits);
    }

    #[test]
    fn legacy_step_matches_explicit_same_key_route_bit_exactly() {
        let mut legacy = a3();
        let mut routed = a3();
        legacy.store_vsa(7, &[0.5, -0.25]).expect("legacy VSA store");
        routed.store_vsa(7, &[0.5, -0.25]).expect("routed VSA store");

        let legacy_report = legacy.step(&[0.25, 0.0], 7, Some(7)).expect("legacy step");
        let routed_report = routed
            .step_routed(&[0.25, 0.0], A3VsaReadRoute::Key(7), 7, Some(7))
            .expect("routed step");
        assert_eq!(legacy_report, routed_report);
        assert_eq!(
            legacy.snapshot().expect("legacy snapshot").state(),
            routed.snapshot().expect("routed snapshot").state()
        );
    }

    #[test]
    fn routed_skip_ignores_nonempty_vsa_and_preserves_a2_semantics_bit_exactly() {
        let memory_layout = AssociativeMemoryLayout::new(8, 2).expect("associative layout");
        let mut a2 = A2Reference::new(identity_parameters(), memory_layout, 11, 1.0).expect("A2");
        let mut a3 = A3Reference::new(
            identity_parameters(),
            memory_layout,
            11,
            1.0,
            VsaWorkspaceLayout::new(2).expect("VSA layout"),
            23,
            1.0,
        )
        .expect("A3");
        a3.store_vsa(7, &[0.75, -0.5]).expect("nonempty VSA");

        let a2_report = a2.step(&[0.25, -0.5], 99, Some(3)).expect("A2 step");
        let a3_report = a3
            .step_routed(&[0.25, -0.5], A3VsaReadRoute::Skip, 99, Some(3))
            .expect("routed skip");
        assert_eq!(a3_report, a2_report);
        let a2_bits: Vec<_> = a2.state().iter().map(|value| value.to_bits()).collect();
        let a3_bits: Vec<_> = a3.state().iter().map(|value| value.to_bits()).collect();
        assert_eq!(a3_bits, a2_bits);
        assert_ne!(a3.workspace().components(), &[0.0, 0.0]);
    }

    #[test]
    fn routed_vsa_key_and_a2_read_key_are_independent() {
        let mut model = a3();
        model.store_vsa(7, &[0.5, -0.25]).expect("VSA store");
        let a2_read_key = 99;
        let expected_address = model.a2().associative_memory().address_for(a2_read_key);

        let report = model
            .step_routed(
                &[0.0, 0.0],
                A3VsaReadRoute::Key(7),
                a2_read_key,
                None,
            )
            .expect("independently routed step");
        assert_eq!(report.read(), A2ReadStatus::Empty { address: expected_address });
        assert_eq!(model.state(), &[0.5, -0.25]);
    }

    #[test]
    fn vsa_readout_changes_the_integrated_a3_recurrent_input() {
        let mut model = a3();
        model.store_vsa(7, &[0.5, -0.25]).expect("VSA store");
        model.step(&[0.0, 0.0], 7, None).expect("A3 read/fuse step");
        assert_eq!(model.state(), &[0.5, -0.25]);
    }

    #[test]
    fn rejected_integrated_step_cannot_mutate_a2_or_vsa_state() {
        let mut model = a3();
        model.store_vsa(7, &[0.5, -0.25]).expect("VSA store");
        let before = model.snapshot().expect("snapshot before rejection");

        assert_eq!(
            model.step(&[f64::NAN, 0.0], 7, Some(7)),
            Err(A3ReferenceError::NonFiniteInput { index: 0 })
        );
        let after = model.snapshot().expect("snapshot after rejection");
        assert_eq!(before.state(), after.state());
    }

    #[test]
    fn rejected_routed_step_cannot_mutate_a2_or_vsa_state() {
        let mut model = a3();
        model.store_vsa(7, &[0.5, -0.25]).expect("VSA store");
        let before = model.snapshot().expect("snapshot before rejection");

        assert_eq!(
            model.step_routed(
                &[f64::NAN, 0.0],
                A3VsaReadRoute::Key(7),
                99,
                Some(3),
            ),
            Err(A3ReferenceError::NonFiniteInput { index: 0 })
        );
        let after = model.snapshot().expect("snapshot after rejection");
        assert_eq!(before.state(), after.state());
    }

    #[test]
    fn rejected_vsa_store_is_atomic_and_does_not_touch_a2() {
        let mut model = a3();
        model.store_vsa(3, &[0.25, 0.5]).expect("seed VSA state");
        let before = model.snapshot().expect("snapshot before rejection");
        assert!(model.store_vsa(9, &[f64::INFINITY, 0.0]).is_err());
        let after = model.snapshot().expect("snapshot after rejection");
        assert_eq!(before.state(), after.state());
    }

    #[test]
    fn a3_accounting_reports_integrated_vsa_and_peak_temporary_storage() {
        let model = a3();
        let a2 = model.a2().memory_accounting().expect("A2 accounting");
        let vsa = model
            .workspace()
            .storage_accounting()
            .expect("VSA accounting");
        let a3 = model.memory_accounting().expect("A3 accounting");

        assert_eq!(a3.vsa_workspace(), vsa.workspace_bits());
        assert_eq!(
            a3.temporary_working().get(),
            a2.temporary_working().get() + vsa.temporary_working_bits().get()
        );
        assert_eq!(
            a3.static_parameters().get(),
            a2.static_parameters().get() + vsa.static_parameter_bits().get() + 64
        );
        a3.validate_for_arm(ReferenceArm::A3)
            .expect("valid A3 accounting");
    }

    #[test]
    fn synthetic_partitions_can_match_a1_a2_a3_dynamic_budget_exactly() {
        // This is an accounting oracle only, not an experimental configuration.
        // In 64-bit units the three dynamic totals are exactly 56:
        // A1 = state 28 + temporary 28;
        // A2 = state 2 + payload 32 + metadata 20 + temporary 2;
        // A3 = state 1 + payload 16 + metadata 20 + VSA 9 + temporary 10.
        let a1 = A1Reference::new(parameters(9, 28)).expect("A1");
        let a2 = A2Reference::new(
            parameters(9, 2),
            AssociativeMemoryLayout::new(16, 2).expect("A2 memory"),
            101,
            1.0,
        )
        .expect("A2");
        let a3 = A3Reference::new(
            parameters(9, 1),
            AssociativeMemoryLayout::new(16, 1).expect("A3 memory"),
            103,
            1.0,
            VsaWorkspaceLayout::new(9).expect("A3 VSA"),
            107,
            0.5,
        )
        .expect("A3");

        let matched = MatchedDynamicBudget::validate(
            a1.memory_accounting().expect("A1 accounting"),
            a2.memory_accounting().expect("A2 accounting"),
            a3.memory_accounting().expect("A3 accounting"),
        )
        .expect("exact matched synthetic budget");
        assert_eq!(matched.bits().get(), 56 * 64);
    }

    #[test]
    fn snapshot_and_reset_cover_both_a2_and_vsa_persistent_state() {
        let mut model = a3();
        model.store_vsa(5, &[0.5, 0.25]).expect("VSA store");
        model.step(&[0.25, -0.5], 3, Some(3)).expect("A3 step");
        let snapshot = model.snapshot().expect("A3 snapshot");
        assert_eq!(snapshot.arm(), ReferenceArm::A3);
        assert_eq!(snapshot.state().a2().recurrent_state(), model.state());
        assert_eq!(
            snapshot.state().vsa_workspace(),
            model.workspace().components()
        );

        let layout = model.workspace().layout();
        let role_seed = model.workspace().role_seed();
        model.reset();
        assert_eq!(model.state(), &[0.0, 0.0]);
        assert_eq!(model.workspace().components(), &[0.0, 0.0]);
        assert_eq!(model.workspace().layout(), layout);
        assert_eq!(model.workspace().role_seed(), role_seed);
    }
}
