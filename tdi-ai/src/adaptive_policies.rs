//! Deterministic non-final C0/C1/C2/C3 reference-policy semantics for TDI-9.1.
//!
//! The policy types preserve the information boundaries frozen by TDI-9.0:
//! C0 and C1 never receive a trajectory observation, C1 plans from task-family
//! identity only, C2 receives only the leakage-safe base observation, and C3
//! receives the same observation plus its already-authorized verifier/checkpoint
//! fields. Concrete primary-cell thresholds and schedules remain TDI-9.1
//! development choices and are not frozen by this module.

use core::fmt;

use crate::adaptive_inference::{
    AdaptiveInferenceError, InferenceAction, PolicyArm, PolicyObservation, VerifierSignal,
    validate_action,
};
use crate::adaptive_task_generators::AdaptiveTaskFamily;

// Reference logical-operation model. These are not CPU instructions. C2 always
// evaluates 6 scalar predicates/operations plus 4 boolean-composition ops. C3
// evaluates the same 10-op base vector plus verification-threshold comparison,
// cadence modulo, cadence composition, checkpoint comparison, violation-residual
// comparison, verify-before-stop composition and one action-dispatch operation.
const C0_DECISION_OPS: u64 = 2;
const C0_POLICY_BITS: u64 = 64;
const C1_PLANNING_OPS: u64 = 2;
const C1_PLANNING_BITS: u64 = 192;
const C1_DECISION_OPS: u64 = 2;
const C1_RUNTIME_BITS: u64 = 64;
const C2_DECISION_OPS: u64 = 10;
const C2_POLICY_BITS: u64 = 256;
const C3_DECISION_OPS: u64 = 17;
const C3_POLICY_BITS: u64 = 385;

/// Logical policy-compute and persistent-policy-memory charge.
///
/// Operation counts are reference logical operations, not CPU instructions or
/// wall-clock estimates. Memory is the exact logical bit representation frozen
/// by this software oracle for the policy configuration/state used at that
/// decision point.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PolicyCharge {
    operations: u64,
    memory_bits: u64,
}

impl PolicyCharge {
    #[must_use]
    pub const fn operations(self) -> u64 {
        self.operations
    }

    #[must_use]
    pub const fn memory_bits(self) -> u64 {
        self.memory_bits
    }
}

/// One action selected by a frozen-arm reference policy plus its exact charge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PolicyDecision {
    arm: PolicyArm,
    action: InferenceAction,
    charge: PolicyCharge,
}

impl PolicyDecision {
    fn new(
        arm: PolicyArm,
        action: InferenceAction,
        operations: u64,
        memory_bits: u64,
    ) -> Result<Self, ReferencePolicyError> {
        validate_action(arm, action)?;
        Ok(Self {
            arm,
            action,
            charge: PolicyCharge {
                operations,
                memory_bits,
            },
        })
    }

    #[must_use]
    pub const fn arm(self) -> PolicyArm {
        self.arm
    }

    #[must_use]
    pub const fn action(self) -> InferenceAction {
        self.action
    }

    #[must_use]
    pub const fn charge(self) -> PolicyCharge {
        self.charge
    }
}

/// C0 fixed-compute schedule.
///
/// Runtime decisions receive only the current transition index. No trajectory
/// summary, verifier signal, task target or hidden stratum can enter this API.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct C0FixedPolicy {
    stop_after_steps: u64,
}

impl C0FixedPolicy {
    pub fn new(stop_after_steps: u64) -> Result<Self, ReferencePolicyError> {
        if stop_after_steps == 0 {
            return Err(ReferencePolicyError::ZeroFixedSchedule);
        }
        Ok(Self { stop_after_steps })
    }

    #[must_use]
    pub const fn stop_after_steps(self) -> u64 {
        self.stop_after_steps
    }

    pub fn decide(self, step_index: u64) -> Result<PolicyDecision, ReferencePolicyError> {
        let action = if step_index >= self.stop_after_steps {
            InferenceAction::Stop
        } else {
            InferenceAction::Continue
        };
        PolicyDecision::new(
            PolicyArm::C0FixedCompute,
            action,
            C0_DECISION_OPS,
            C0_POLICY_BITS,
        )
    }
}

/// Pre-inference C1 family-only compute schedule.
///
/// The schedule deliberately accepts no task length, difficulty stratum,
/// generator seed or evaluator record. Selecting a runtime plan therefore uses
/// only the ordinary task-family identity explicitly permitted by TDI-9.0.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct C1StaticPolicy {
    p1_stop_after_steps: u64,
    p2_stop_after_steps: u64,
    p3_stop_after_steps: u64,
}

impl C1StaticPolicy {
    pub fn new(
        p1_stop_after_steps: u64,
        p2_stop_after_steps: u64,
        p3_stop_after_steps: u64,
    ) -> Result<Self, ReferencePolicyError> {
        for (family, steps) in [
            (
                AdaptiveTaskFamily::StagedEvidenceAccumulation,
                p1_stop_after_steps,
            ),
            (
                AdaptiveTaskFamily::VerificationSensitiveInference,
                p2_stop_after_steps,
            ),
            (
                AdaptiveTaskFamily::RecoverableDeceptiveFork,
                p3_stop_after_steps,
            ),
        ] {
            if steps == 0 {
                return Err(ReferencePolicyError::ZeroStaticSchedule { family });
            }
        }
        Ok(Self {
            p1_stop_after_steps,
            p2_stop_after_steps,
            p3_stop_after_steps,
        })
    }

    #[must_use]
    pub const fn planning_charge(self) -> PolicyCharge {
        PolicyCharge {
            operations: C1_PLANNING_OPS,
            memory_bits: C1_PLANNING_BITS,
        }
    }

    #[must_use]
    pub const fn plan(self, family: AdaptiveTaskFamily) -> C1Plan {
        let stop_after_steps = match family {
            AdaptiveTaskFamily::StagedEvidenceAccumulation => self.p1_stop_after_steps,
            AdaptiveTaskFamily::VerificationSensitiveInference => self.p2_stop_after_steps,
            AdaptiveTaskFamily::RecoverableDeceptiveFork => self.p3_stop_after_steps,
        };
        C1Plan { stop_after_steps }
    }
}

/// C1 runtime plan after family-only preallocation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct C1Plan {
    stop_after_steps: u64,
}

impl C1Plan {
    #[must_use]
    pub const fn stop_after_steps(self) -> u64 {
        self.stop_after_steps
    }

    pub fn decide(self, step_index: u64) -> Result<PolicyDecision, ReferencePolicyError> {
        let action = if step_index >= self.stop_after_steps {
            InferenceAction::Stop
        } else {
            InferenceAction::Continue
        };
        PolicyDecision::new(
            PolicyArm::C1StaticPreallocation,
            action,
            C1_DECISION_OPS,
            C1_RUNTIME_BITS,
        )
    }
}

/// Non-final C2 observation-conditioned stopping rule.
///
/// The four caller-supplied values are development parameters, not frozen
/// primary-cell choices. The rule evaluates every predicate in a fixed order so
/// its logical operation count is constant and exact.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct C2AdaptivePolicy {
    minimum_steps: u64,
    max_residual_for_adaptive_stop: f64,
    max_state_delta_for_adaptive_stop: f64,
    min_abs_margin_for_adaptive_stop: f64,
}

impl C2AdaptivePolicy {
    pub fn new(
        minimum_steps: u64,
        max_residual_for_adaptive_stop: f64,
        max_state_delta_for_adaptive_stop: f64,
        min_abs_margin_for_adaptive_stop: f64,
    ) -> Result<Self, ReferencePolicyError> {
        validate_nonnegative_finite(
            PolicyConfigField::MaxResidualForAdaptiveStop,
            max_residual_for_adaptive_stop,
        )?;
        validate_nonnegative_finite(
            PolicyConfigField::MaxStateDeltaForAdaptiveStop,
            max_state_delta_for_adaptive_stop,
        )?;
        validate_nonnegative_finite(
            PolicyConfigField::MinAbsMarginForAdaptiveStop,
            min_abs_margin_for_adaptive_stop,
        )?;
        Ok(Self {
            minimum_steps,
            max_residual_for_adaptive_stop,
            max_state_delta_for_adaptive_stop,
            min_abs_margin_for_adaptive_stop,
        })
    }

    pub fn decide(
        self,
        observation: PolicyObservation,
    ) -> Result<PolicyDecision, ReferencePolicyError> {
        let observation = observation.validate_for_arm(PolicyArm::C2AdaptiveStopping)?;
        let action = if base_should_stop(
            observation,
            self.minimum_steps,
            self.max_residual_for_adaptive_stop,
            self.max_state_delta_for_adaptive_stop,
            self.min_abs_margin_for_adaptive_stop,
        ) {
            InferenceAction::Stop
        } else {
            InferenceAction::Continue
        };
        PolicyDecision::new(
            PolicyArm::C2AdaptiveStopping,
            action,
            C2_DECISION_OPS,
            C2_POLICY_BITS,
        )
    }
}

/// Non-final C3 verification/recovery policy.
///
/// C3 uses the same base adaptive-stop predicate as C2, then adds a fixed
/// verification cadence and verifier-driven recovery. `verify_before_stop`
/// controls whether an otherwise selected adaptive STOP must first be certified
/// by the independent verifier. No target/evaluator value is accepted.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct C3RecoveryPolicy {
    minimum_steps: u64,
    max_residual_for_adaptive_stop: f64,
    max_state_delta_for_adaptive_stop: f64,
    min_abs_margin_for_adaptive_stop: f64,
    minimum_verification_step: u64,
    verify_every_steps: u64,
    verify_before_stop: bool,
}

impl C3RecoveryPolicy {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        minimum_steps: u64,
        max_residual_for_adaptive_stop: f64,
        max_state_delta_for_adaptive_stop: f64,
        min_abs_margin_for_adaptive_stop: f64,
        minimum_verification_step: u64,
        verify_every_steps: u64,
        verify_before_stop: bool,
    ) -> Result<Self, ReferencePolicyError> {
        validate_nonnegative_finite(
            PolicyConfigField::MaxResidualForAdaptiveStop,
            max_residual_for_adaptive_stop,
        )?;
        validate_nonnegative_finite(
            PolicyConfigField::MaxStateDeltaForAdaptiveStop,
            max_state_delta_for_adaptive_stop,
        )?;
        validate_nonnegative_finite(
            PolicyConfigField::MinAbsMarginForAdaptiveStop,
            min_abs_margin_for_adaptive_stop,
        )?;
        if verify_every_steps == 0 {
            return Err(ReferencePolicyError::ZeroVerificationCadence);
        }
        Ok(Self {
            minimum_steps,
            max_residual_for_adaptive_stop,
            max_state_delta_for_adaptive_stop,
            min_abs_margin_for_adaptive_stop,
            minimum_verification_step,
            verify_every_steps,
            verify_before_stop,
        })
    }

    pub fn decide(
        self,
        observation: PolicyObservation,
    ) -> Result<PolicyDecision, ReferencePolicyError> {
        let observation = observation.validate_for_arm(PolicyArm::C3VerificationRecovery)?;
        let base_stop = base_should_stop(
            observation,
            self.minimum_steps,
            self.max_residual_for_adaptive_stop,
            self.max_state_delta_for_adaptive_stop,
            self.min_abs_margin_for_adaptive_stop,
        );
        let verification_threshold_met =
            observation.step_index() >= self.minimum_verification_step;
        let on_verification_cadence = observation.step_index() % self.verify_every_steps == 0;
        let cadence_due = verification_threshold_met & on_verification_cadence;
        let checkpoint_available = observation.available_checkpoints() > 0;
        let violation_has_remaining_work = observation.residual() > 0.0;
        let verify_adaptive_stop = base_stop & self.verify_before_stop;
        let action = match observation.verifier_signal() {
            Some(VerifierSignal::Violated) if checkpoint_available => InferenceAction::Backtrack,
            Some(VerifierSignal::Violated) if violation_has_remaining_work => {
                InferenceAction::Continue
            }
            Some(VerifierSignal::Violated) => {
                return Err(ReferencePolicyError::UnrecoverableVerifiedViolation)
            }
            Some(VerifierSignal::Satisfied) => InferenceAction::Stop,
            Some(VerifierSignal::Indeterminate) => InferenceAction::Continue,
            None if verify_adaptive_stop => InferenceAction::Verify,
            None if base_stop => InferenceAction::Stop,
            None if cadence_due => InferenceAction::Verify,
            None => InferenceAction::Continue,
        };
        PolicyDecision::new(
            PolicyArm::C3VerificationRecovery,
            action,
            C3_DECISION_OPS,
            C3_POLICY_BITS,
        )
    }
}

/// Named non-final policy configuration field.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PolicyConfigField {
    MaxResidualForAdaptiveStop,
    MaxStateDeltaForAdaptiveStop,
    MinAbsMarginForAdaptiveStop,
}

fn base_should_stop(
    observation: PolicyObservation,
    minimum_steps: u64,
    max_residual_for_adaptive_stop: f64,
    max_state_delta_for_adaptive_stop: f64,
    min_abs_margin_for_adaptive_stop: f64,
) -> bool {
    let enough_steps = observation.step_index() >= minimum_steps;
    let terminal = observation.residual() == 0.0;
    let residual_small = observation.residual() <= max_residual_for_adaptive_stop;
    let delta_small = observation.state_delta() <= max_state_delta_for_adaptive_stop;
    let absolute_margin = observation.score_margin().abs();
    let margin_large = absolute_margin >= min_abs_margin_for_adaptive_stop;
    let adaptive_stop = residual_small & delta_small & margin_large;
    enough_steps & (terminal | adaptive_stop)
}

fn validate_nonnegative_finite(
    field: PolicyConfigField,
    value: f64,
) -> Result<(), ReferencePolicyError> {
    if !value.is_finite() {
        return Err(ReferencePolicyError::NonFiniteConfiguration { field });
    }
    if value < 0.0 {
        return Err(ReferencePolicyError::NegativeConfiguration { field });
    }
    Ok(())
}

/// Typed fail-closed reference-policy failure.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ReferencePolicyError {
    AdaptiveInference(AdaptiveInferenceError),
    ZeroFixedSchedule,
    ZeroStaticSchedule { family: AdaptiveTaskFamily },
    NonFiniteConfiguration { field: PolicyConfigField },
    NegativeConfiguration { field: PolicyConfigField },
    ZeroVerificationCadence,
    UnrecoverableVerifiedViolation,
}

impl From<AdaptiveInferenceError> for ReferencePolicyError {
    fn from(value: AdaptiveInferenceError) -> Self {
        Self::AdaptiveInference(value)
    }
}

impl fmt::Display for ReferencePolicyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for ReferencePolicyError {}
