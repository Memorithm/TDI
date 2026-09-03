//! Deterministic TDI-9.1 adaptive-inference contracts and resource accounting.
//!
//! This module intentionally stops before task generators, solver semantics,
//! verifier semantics or learned/search policies. It defines only the frozen
//! policy-arm identities, action vocabulary, leakage-safe observation carrier
//! and exact bounded resource meter required by TDI-9.0.

use core::fmt;

/// Frozen TDI-9 policy ladder.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PolicyArm {
    /// C0: fixed compute schedule.
    C0FixedCompute,
    /// C1: pre-inference static allocation using only allowed static inputs.
    C1StaticPreallocation,
    /// C2: trajectory-conditioned CONTINUE/STOP.
    C2AdaptiveStopping,
    /// C3: trajectory-conditioned CONTINUE/VERIFY/BACKTRACK/STOP.
    C3VerificationRecovery,
}

impl PolicyArm {
    /// Whether the frozen arm is allowed to issue an action at runtime.
    #[must_use]
    pub const fn allows(self, action: InferenceAction) -> bool {
        match self {
            Self::C0FixedCompute | Self::C1StaticPreallocation | Self::C2AdaptiveStopping => {
                matches!(action, InferenceAction::Continue | InferenceAction::Stop)
            }
            Self::C3VerificationRecovery => true,
        }
    }

    /// Whether trajectory observations may alter the compute schedule.
    #[must_use]
    pub const fn is_trajectory_adaptive(self) -> bool {
        matches!(
            self,
            Self::C2AdaptiveStopping | Self::C3VerificationRecovery
        )
    }
}

/// Frozen adaptive-inference action vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InferenceAction {
    /// Execute the next solver transition.
    Continue,
    /// Execute the independently frozen verifier.
    Verify,
    /// Restore one eligible checkpoint and resume from it.
    Backtrack,
    /// Emit the current candidate and terminate the trajectory.
    Stop,
}

/// Result exposed by a verifier invocation to a C3 policy.
///
/// The verifier signal describes constraint satisfaction only. It is not the
/// evaluator-owned task target or a success/failure label for the final answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VerifierSignal {
    /// Frozen verifier accepted all checked constraints.
    Satisfied,
    /// Frozen verifier found at least one violated constraint.
    Violated,
    /// Frozen verifier could not decide under its bounded semantics.
    Indeterminate,
}

/// Leakage-safe trajectory observation available to an adaptive policy.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PolicyObservation {
    step_index: u64,
    remaining_compute_ops: u64,
    state_delta: f64,
    residual: f64,
    score_margin: f64,
    prior_action_count: u64,
    verifier_signal: Option<VerifierSignal>,
    available_checkpoints: u32,
}

impl PolicyObservation {
    /// Construct an observation from current/past trajectory information only.
    ///
    /// `state_delta` and `residual` are non-negative magnitudes. `score_margin`
    /// may be signed but must be finite. No target/future/evaluator field exists
    /// in this type by construction.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        step_index: u64,
        remaining_compute_ops: u64,
        state_delta: f64,
        residual: f64,
        score_margin: f64,
        prior_action_count: u64,
        verifier_signal: Option<VerifierSignal>,
        available_checkpoints: u32,
    ) -> Result<Self, AdaptiveInferenceError> {
        if !state_delta.is_finite() {
            return Err(AdaptiveInferenceError::NonFiniteObservation {
                field: ObservationField::StateDelta,
            });
        }
        if state_delta < 0.0 {
            return Err(AdaptiveInferenceError::NegativeObservationMagnitude {
                field: ObservationField::StateDelta,
            });
        }
        if !residual.is_finite() {
            return Err(AdaptiveInferenceError::NonFiniteObservation {
                field: ObservationField::Residual,
            });
        }
        if residual < 0.0 {
            return Err(AdaptiveInferenceError::NegativeObservationMagnitude {
                field: ObservationField::Residual,
            });
        }
        if !score_margin.is_finite() {
            return Err(AdaptiveInferenceError::NonFiniteObservation {
                field: ObservationField::ScoreMargin,
            });
        }
        Ok(Self {
            step_index,
            remaining_compute_ops,
            state_delta,
            residual,
            score_margin,
            prior_action_count,
            verifier_signal,
            available_checkpoints,
        })
    }

    /// Verify arm-specific information boundaries.
    pub fn validate_for_arm(self, arm: PolicyArm) -> Result<Self, AdaptiveInferenceError> {
        if arm != PolicyArm::C3VerificationRecovery && self.verifier_signal.is_some() {
            return Err(AdaptiveInferenceError::VerifierSignalForbidden { arm });
        }
        if arm != PolicyArm::C3VerificationRecovery && self.available_checkpoints != 0 {
            return Err(AdaptiveInferenceError::CheckpointMetadataForbidden { arm });
        }
        Ok(self)
    }

    #[must_use]
    pub const fn step_index(self) -> u64 {
        self.step_index
    }

    #[must_use]
    pub const fn remaining_compute_ops(self) -> u64 {
        self.remaining_compute_ops
    }

    #[must_use]
    pub const fn state_delta(self) -> f64 {
        self.state_delta
    }

    #[must_use]
    pub const fn residual(self) -> f64 {
        self.residual
    }

    #[must_use]
    pub const fn score_margin(self) -> f64 {
        self.score_margin
    }

    #[must_use]
    pub const fn prior_action_count(self) -> u64 {
        self.prior_action_count
    }

    #[must_use]
    pub const fn verifier_signal(self) -> Option<VerifierSignal> {
        self.verifier_signal
    }

    #[must_use]
    pub const fn available_checkpoints(self) -> u32 {
        self.available_checkpoints
    }
}

/// Named numeric field rejected by observation validation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ObservationField {
    StateDelta,
    Residual,
    ScoreMargin,
}

/// Common maximum resource envelope for one primary-cell arm execution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ResourceEnvelope {
    max_compute_ops: u64,
    max_memory_bits: u64,
}

impl ResourceEnvelope {
    /// Construct a strictly positive compute and memory envelope.
    pub fn new(max_compute_ops: u64, max_memory_bits: u64) -> Result<Self, AdaptiveInferenceError> {
        if max_compute_ops == 0 {
            return Err(AdaptiveInferenceError::ZeroComputeEnvelope);
        }
        if max_memory_bits == 0 {
            return Err(AdaptiveInferenceError::ZeroMemoryEnvelope);
        }
        Ok(Self {
            max_compute_ops,
            max_memory_bits,
        })
    }

    #[must_use]
    pub const fn max_compute_ops(self) -> u64 {
        self.max_compute_ops
    }

    #[must_use]
    pub const fn max_memory_bits(self) -> u64 {
        self.max_memory_bits
    }
}

/// Explicit compute component required by the TDI-9.0 accounting contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ComputeComponent {
    Solver,
    Verifier,
    PolicyDecision,
    Checkpoint,
    Replay,
}

/// Exact accumulated reference resource usage.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ResourceUsage {
    solver_ops: u64,
    verifier_ops: u64,
    policy_ops: u64,
    checkpoint_ops: u64,
    replay_ops: u64,
    persistent_memory_peak_bits: u64,
    policy_memory_peak_bits: u64,
    checkpoint_memory_peak_bits: u64,
    temporary_peak_bits: u64,
    total_memory_peak_bits: u64,
}

impl ResourceUsage {
    #[must_use]
    pub const fn solver_ops(self) -> u64 {
        self.solver_ops
    }

    #[must_use]
    pub const fn verifier_ops(self) -> u64 {
        self.verifier_ops
    }

    #[must_use]
    pub const fn policy_ops(self) -> u64 {
        self.policy_ops
    }

    #[must_use]
    pub const fn checkpoint_ops(self) -> u64 {
        self.checkpoint_ops
    }

    #[must_use]
    pub const fn replay_ops(self) -> u64 {
        self.replay_ops
    }

    pub fn total_compute_ops(self) -> Result<u64, AdaptiveInferenceError> {
        [
            self.solver_ops,
            self.verifier_ops,
            self.policy_ops,
            self.checkpoint_ops,
            self.replay_ops,
        ]
        .into_iter()
        .try_fold(0u64, |total, value| {
            total
                .checked_add(value)
                .ok_or(AdaptiveInferenceError::ResourceAccountingOverflow)
        })
    }

    #[must_use]
    pub const fn persistent_memory_bits(self) -> u64 {
        self.persistent_memory_peak_bits
    }

    #[must_use]
    pub const fn policy_memory_bits(self) -> u64 {
        self.policy_memory_peak_bits
    }

    #[must_use]
    pub const fn checkpoint_memory_bits(self) -> u64 {
        self.checkpoint_memory_peak_bits
    }

    #[must_use]
    pub const fn temporary_peak_bits(self) -> u64 {
        self.temporary_peak_bits
    }

    /// Exact high-water mark of simultaneous declared memory.
    #[must_use]
    pub const fn total_memory_bits(self) -> u64 {
        self.total_memory_peak_bits
    }
}

/// Fail-closed mutable meter for one arm execution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceMeter {
    envelope: ResourceEnvelope,
    usage: ResourceUsage,
}

impl ResourceMeter {
    #[must_use]
    pub const fn new(envelope: ResourceEnvelope) -> Self {
        Self {
            envelope,
            usage: ResourceUsage {
                solver_ops: 0,
                verifier_ops: 0,
                policy_ops: 0,
                checkpoint_ops: 0,
                replay_ops: 0,
                persistent_memory_peak_bits: 0,
                policy_memory_peak_bits: 0,
                checkpoint_memory_peak_bits: 0,
                temporary_peak_bits: 0,
                total_memory_peak_bits: 0,
            },
        }
    }

    #[must_use]
    pub const fn envelope(self) -> ResourceEnvelope {
        self.envelope
    }

    #[must_use]
    pub const fn usage(self) -> ResourceUsage {
        self.usage
    }

    /// Charge compute atomically. A rejected charge leaves usage unchanged.
    pub fn charge_compute(
        &mut self,
        component: ComputeComponent,
        ops: u64,
    ) -> Result<(), AdaptiveInferenceError> {
        let mut candidate = self.usage;
        let target = match component {
            ComputeComponent::Solver => &mut candidate.solver_ops,
            ComputeComponent::Verifier => &mut candidate.verifier_ops,
            ComputeComponent::PolicyDecision => &mut candidate.policy_ops,
            ComputeComponent::Checkpoint => &mut candidate.checkpoint_ops,
            ComputeComponent::Replay => &mut candidate.replay_ops,
        };
        *target = target
            .checked_add(ops)
            .ok_or(AdaptiveInferenceError::ResourceAccountingOverflow)?;
        let requested = candidate.total_compute_ops()?;
        if requested > self.envelope.max_compute_ops {
            return Err(AdaptiveInferenceError::ComputeEnvelopeExceeded {
                maximum: self.envelope.max_compute_ops,
                requested,
            });
        }
        self.usage = candidate;
        Ok(())
    }

    /// Account one simultaneous-memory state atomically and retain exact
    /// component and total high-water marks for the complete trajectory.
    pub fn set_memory(
        &mut self,
        persistent_memory_bits: u64,
        policy_memory_bits: u64,
        checkpoint_memory_bits: u64,
        temporary_memory_bits: u64,
    ) -> Result<(), AdaptiveInferenceError> {
        let requested = [
            persistent_memory_bits,
            policy_memory_bits,
            checkpoint_memory_bits,
            temporary_memory_bits,
        ]
        .into_iter()
        .try_fold(0u64, |total, value| {
            total
                .checked_add(value)
                .ok_or(AdaptiveInferenceError::ResourceAccountingOverflow)
        })?;
        if requested > self.envelope.max_memory_bits {
            return Err(AdaptiveInferenceError::MemoryEnvelopeExceeded {
                maximum: self.envelope.max_memory_bits,
                requested,
            });
        }

        let mut candidate = self.usage;
        candidate.persistent_memory_peak_bits = candidate
            .persistent_memory_peak_bits
            .max(persistent_memory_bits);
        candidate.policy_memory_peak_bits = candidate.policy_memory_peak_bits.max(policy_memory_bits);
        candidate.checkpoint_memory_peak_bits = candidate
            .checkpoint_memory_peak_bits
            .max(checkpoint_memory_bits);
        candidate.temporary_peak_bits = candidate.temporary_peak_bits.max(temporary_memory_bits);
        candidate.total_memory_peak_bits = candidate.total_memory_peak_bits.max(requested);
        self.usage = candidate;
        Ok(())
    }
}

/// Validate that an action belongs to the frozen arm vocabulary.
pub fn validate_action(
    arm: PolicyArm,
    action: InferenceAction,
) -> Result<(), AdaptiveInferenceError> {
    if arm.allows(action) {
        Ok(())
    } else {
        Err(AdaptiveInferenceError::ActionForbidden { arm, action })
    }
}

/// Typed TDI-9.1 contract/accounting failures.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AdaptiveInferenceError {
    NonFiniteObservation {
        field: ObservationField,
    },
    NegativeObservationMagnitude {
        field: ObservationField,
    },
    VerifierSignalForbidden {
        arm: PolicyArm,
    },
    CheckpointMetadataForbidden {
        arm: PolicyArm,
    },
    ActionForbidden {
        arm: PolicyArm,
        action: InferenceAction,
    },
    ZeroComputeEnvelope,
    ZeroMemoryEnvelope,
    ResourceAccountingOverflow,
    ComputeEnvelopeExceeded {
        maximum: u64,
        requested: u64,
    },
    MemoryEnvelopeExceeded {
        maximum: u64,
        requested: u64,
    },
}

impl fmt::Display for AdaptiveInferenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for AdaptiveInferenceError {}

#[cfg(test)]
mod tests {
    use super::{
        AdaptiveInferenceError, ComputeComponent, InferenceAction, PolicyArm, PolicyObservation,
        ResourceEnvelope, ResourceMeter, VerifierSignal, validate_action,
    };

    #[test]
    fn frozen_action_ladder_blocks_verify_and_backtrack_before_c3() {
        for arm in [
            PolicyArm::C0FixedCompute,
            PolicyArm::C1StaticPreallocation,
            PolicyArm::C2AdaptiveStopping,
        ] {
            assert!(validate_action(arm, InferenceAction::Continue).is_ok());
            assert!(validate_action(arm, InferenceAction::Stop).is_ok());
            assert!(matches!(
                validate_action(arm, InferenceAction::Verify),
                Err(AdaptiveInferenceError::ActionForbidden { .. })
            ));
            assert!(matches!(
                validate_action(arm, InferenceAction::Backtrack),
                Err(AdaptiveInferenceError::ActionForbidden { .. })
            ));
        }
        for action in [
            InferenceAction::Continue,
            InferenceAction::Verify,
            InferenceAction::Backtrack,
            InferenceAction::Stop,
        ] {
            assert!(validate_action(PolicyArm::C3VerificationRecovery, action).is_ok());
        }
    }

    #[test]
    fn observation_rejects_nonfinite_or_negative_magnitudes() {
        assert!(PolicyObservation::new(0, 10, f64::NAN, 0.0, 0.0, 0, None, 0).is_err());
        assert!(PolicyObservation::new(0, 10, 0.0, -1.0, 0.0, 0, None, 0).is_err());
        assert!(PolicyObservation::new(0, 10, 0.0, 0.0, f64::INFINITY, 0, None, 0).is_err());
    }

    #[test]
    fn verifier_and_checkpoint_metadata_are_c3_only() {
        let observation = PolicyObservation::new(
            3,
            100,
            0.25,
            0.5,
            -0.1,
            4,
            Some(VerifierSignal::Violated),
            2,
        )
        .expect("finite observation");
        assert!(matches!(
            observation.validate_for_arm(PolicyArm::C2AdaptiveStopping),
            Err(AdaptiveInferenceError::VerifierSignalForbidden { .. })
        ));
        assert!(
            observation
                .validate_for_arm(PolicyArm::C3VerificationRecovery)
                .is_ok()
        );
    }

    #[test]
    fn resource_meter_counts_all_compute_components_and_rejects_atomically() {
        let envelope = ResourceEnvelope::new(20, 1_000).expect("positive envelope");
        let mut meter = ResourceMeter::new(envelope);
        meter
            .charge_compute(ComputeComponent::Solver, 7)
            .expect("solver");
        meter
            .charge_compute(ComputeComponent::PolicyDecision, 2)
            .expect("policy");
        meter
            .charge_compute(ComputeComponent::Verifier, 3)
            .expect("verifier");
        meter
            .charge_compute(ComputeComponent::Checkpoint, 2)
            .expect("checkpoint");
        meter
            .charge_compute(ComputeComponent::Replay, 4)
            .expect("replay");
        assert_eq!(meter.usage().total_compute_ops().expect("sum"), 18);

        let before = meter.usage();
        assert!(matches!(
            meter.charge_compute(ComputeComponent::Solver, 3),
            Err(AdaptiveInferenceError::ComputeEnvelopeExceeded {
                maximum: 20,
                requested: 21,
            })
        ));
        assert_eq!(meter.usage(), before);
    }

    #[test]
    fn memory_meter_retains_component_and_simultaneous_high_water_marks() {
        let envelope = ResourceEnvelope::new(100, 64).expect("positive envelope");
        let mut meter = ResourceMeter::new(envelope);
        meter.set_memory(24, 8, 16, 16).expect("exact envelope");
        assert_eq!(meter.usage().total_memory_bits(), 64);

        meter
            .set_memory(20, 8, 8, 4)
            .expect("smaller later state is valid");
        assert_eq!(meter.usage().persistent_memory_bits(), 24);
        assert_eq!(meter.usage().policy_memory_bits(), 8);
        assert_eq!(meter.usage().checkpoint_memory_bits(), 16);
        assert_eq!(meter.usage().temporary_peak_bits(), 16);
        assert_eq!(meter.usage().total_memory_bits(), 64);

        let before = meter.usage();
        assert!(matches!(
            meter.set_memory(24, 8, 17, 16),
            Err(AdaptiveInferenceError::MemoryEnvelopeExceeded {
                maximum: 64,
                requested: 65,
            })
        ));
        assert_eq!(meter.usage(), before);
    }
}
