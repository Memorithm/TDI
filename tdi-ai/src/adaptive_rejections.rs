//! Machine-readable non-final TDI-9.1 reference rejection records.
//!
//! This layer wraps the already-qualified bounded evaluator without changing its
//! compatibility API. Technical, execution, policy and inference failures remain
//! typed rejections and are never reinterpreted as task-quality outcomes.

use crate::adaptive_evaluator::{
    ReferenceEvaluationRecord, ReferenceEvaluatorError, ReferencePolicy, evaluate_generated_task,
};
use crate::adaptive_execution::ReferenceExecutionError;
use crate::adaptive_inference::{AdaptiveInferenceError, PolicyArm, ResourceEnvelope};
use crate::adaptive_policies::ReferencePolicyError;
use crate::adaptive_task_generators::{AdaptiveTaskFamily, DifficultyStratum, GeneratedTask};

/// Stable non-final machine code for every currently represented evaluator
/// rejection path.
///
/// Numeric values are intentionally explicit. Existing values must never be
/// renumbered; future rejection reasons must consume previously unused values.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum ReferenceRejectionCode {
    EvaluatorZeroDecisionLimit = 0x0101,
    EvaluatorDecisionLimitExceeded = 0x0102,
    EvaluatorDecisionCountOverflow = 0x0103,
    EvaluatorMissingC1Plan = 0x0104,
    EvaluatorGeneratorFamilyMismatch = 0x0105,
    EvaluatorPolicyArmDrift = 0x0106,

    ExecutionAlreadyStopped = 0x0201,
    ExecutionSolverExhausted = 0x0202,
    ExecutionTaskTooLarge = 0x0203,
    ExecutionTaskStateMismatch = 0x0204,
    ExecutionTaskContractViolation = 0x0205,
    ExecutionArithmeticOverflow = 0x0206,
    ExecutionCheckpointTrafficOverflow = 0x0207,
    ExecutionAccountingInvariant = 0x0208,
    ExecutionCheckpointUnavailable = 0x0209,
    ExecutionCheckpointNotEarlier = 0x020a,
    ExecutionBacktrackRequiresViolation = 0x020b,
    ExecutionBacktrackUnsupportedForTask = 0x020c,
    ExecutionEvaluatorFamilyMismatch = 0x020d,

    ExecutionInferenceNonFiniteObservation = 0x0301,
    ExecutionInferenceNegativeObservationMagnitude = 0x0302,
    ExecutionInferenceVerifierSignalForbidden = 0x0303,
    ExecutionInferenceCheckpointMetadataForbidden = 0x0304,
    ExecutionInferenceActionForbidden = 0x0305,
    ExecutionInferenceZeroComputeEnvelope = 0x0306,
    ExecutionInferenceZeroMemoryEnvelope = 0x0307,
    ExecutionInferenceResourceAccountingOverflow = 0x0308,
    ExecutionInferenceComputeEnvelopeExceeded = 0x0309,
    ExecutionInferenceMemoryEnvelopeExceeded = 0x030a,

    PolicyZeroFixedSchedule = 0x0401,
    PolicyZeroStaticSchedule = 0x0402,
    PolicyNonFiniteConfiguration = 0x0403,
    PolicyNegativeConfiguration = 0x0404,
    PolicyZeroVerificationCadence = 0x0405,
    PolicyUnrecoverableVerifiedViolation = 0x0406,

    PolicyInferenceNonFiniteObservation = 0x0501,
    PolicyInferenceNegativeObservationMagnitude = 0x0502,
    PolicyInferenceVerifierSignalForbidden = 0x0503,
    PolicyInferenceCheckpointMetadataForbidden = 0x0504,
    PolicyInferenceActionForbidden = 0x0505,
    PolicyInferenceZeroComputeEnvelope = 0x0506,
    PolicyInferenceZeroMemoryEnvelope = 0x0507,
    PolicyInferenceResourceAccountingOverflow = 0x0508,
    PolicyInferenceComputeEnvelopeExceeded = 0x0509,
    PolicyInferenceMemoryEnvelopeExceeded = 0x050a,
}

impl ReferenceRejectionCode {
    /// Exact stable numeric representation for machine-readable records.
    #[must_use]
    pub const fn numeric(self) -> u16 {
        self as u16
    }

    /// Losslessly map the typed evaluator error tree to one stable categorical
    /// rejection code while the rejection record retains the original error.
    #[must_use]
    pub const fn from_error(error: ReferenceEvaluatorError) -> Self {
        match error {
            ReferenceEvaluatorError::Execution(error) => Self::from_execution(error),
            ReferenceEvaluatorError::Policy(error) => Self::from_policy(error),
            ReferenceEvaluatorError::ZeroDecisionLimit => Self::EvaluatorZeroDecisionLimit,
            ReferenceEvaluatorError::DecisionLimitExceeded { .. } => {
                Self::EvaluatorDecisionLimitExceeded
            }
            ReferenceEvaluatorError::DecisionCountOverflow => Self::EvaluatorDecisionCountOverflow,
            ReferenceEvaluatorError::MissingC1Plan => Self::EvaluatorMissingC1Plan,
            ReferenceEvaluatorError::GeneratorFamilyMismatch => {
                Self::EvaluatorGeneratorFamilyMismatch
            }
            ReferenceEvaluatorError::PolicyArmDrift { .. } => Self::EvaluatorPolicyArmDrift,
        }
    }

    const fn from_execution(error: ReferenceExecutionError) -> Self {
        match error {
            ReferenceExecutionError::AdaptiveInference(error) => {
                Self::from_execution_inference(error)
            }
            ReferenceExecutionError::AlreadyStopped => Self::ExecutionAlreadyStopped,
            ReferenceExecutionError::SolverExhausted => Self::ExecutionSolverExhausted,
            ReferenceExecutionError::TaskTooLarge => Self::ExecutionTaskTooLarge,
            ReferenceExecutionError::TaskStateMismatch => Self::ExecutionTaskStateMismatch,
            ReferenceExecutionError::TaskContractViolation => Self::ExecutionTaskContractViolation,
            ReferenceExecutionError::ArithmeticOverflow => Self::ExecutionArithmeticOverflow,
            ReferenceExecutionError::CheckpointTrafficOverflow => {
                Self::ExecutionCheckpointTrafficOverflow
            }
            ReferenceExecutionError::AccountingInvariant => Self::ExecutionAccountingInvariant,
            ReferenceExecutionError::CheckpointUnavailable => Self::ExecutionCheckpointUnavailable,
            ReferenceExecutionError::CheckpointNotEarlier => Self::ExecutionCheckpointNotEarlier,
            ReferenceExecutionError::BacktrackRequiresViolation => {
                Self::ExecutionBacktrackRequiresViolation
            }
            ReferenceExecutionError::BacktrackUnsupportedForTask => {
                Self::ExecutionBacktrackUnsupportedForTask
            }
            ReferenceExecutionError::EvaluatorFamilyMismatch => {
                Self::ExecutionEvaluatorFamilyMismatch
            }
        }
    }

    const fn from_policy(error: ReferencePolicyError) -> Self {
        match error {
            ReferencePolicyError::AdaptiveInference(error) => Self::from_policy_inference(error),
            ReferencePolicyError::ZeroFixedSchedule => Self::PolicyZeroFixedSchedule,
            ReferencePolicyError::ZeroStaticSchedule { .. } => Self::PolicyZeroStaticSchedule,
            ReferencePolicyError::NonFiniteConfiguration { .. } => {
                Self::PolicyNonFiniteConfiguration
            }
            ReferencePolicyError::NegativeConfiguration { .. } => Self::PolicyNegativeConfiguration,
            ReferencePolicyError::ZeroVerificationCadence => Self::PolicyZeroVerificationCadence,
            ReferencePolicyError::UnrecoverableVerifiedViolation => {
                Self::PolicyUnrecoverableVerifiedViolation
            }
        }
    }

    const fn from_execution_inference(error: AdaptiveInferenceError) -> Self {
        match error {
            AdaptiveInferenceError::NonFiniteObservation { .. } => {
                Self::ExecutionInferenceNonFiniteObservation
            }
            AdaptiveInferenceError::NegativeObservationMagnitude { .. } => {
                Self::ExecutionInferenceNegativeObservationMagnitude
            }
            AdaptiveInferenceError::VerifierSignalForbidden { .. } => {
                Self::ExecutionInferenceVerifierSignalForbidden
            }
            AdaptiveInferenceError::CheckpointMetadataForbidden { .. } => {
                Self::ExecutionInferenceCheckpointMetadataForbidden
            }
            AdaptiveInferenceError::ActionForbidden { .. } => {
                Self::ExecutionInferenceActionForbidden
            }
            AdaptiveInferenceError::ZeroComputeEnvelope => {
                Self::ExecutionInferenceZeroComputeEnvelope
            }
            AdaptiveInferenceError::ZeroMemoryEnvelope => {
                Self::ExecutionInferenceZeroMemoryEnvelope
            }
            AdaptiveInferenceError::ResourceAccountingOverflow => {
                Self::ExecutionInferenceResourceAccountingOverflow
            }
            AdaptiveInferenceError::ComputeEnvelopeExceeded { .. } => {
                Self::ExecutionInferenceComputeEnvelopeExceeded
            }
            AdaptiveInferenceError::MemoryEnvelopeExceeded { .. } => {
                Self::ExecutionInferenceMemoryEnvelopeExceeded
            }
        }
    }

    const fn from_policy_inference(error: AdaptiveInferenceError) -> Self {
        match error {
            AdaptiveInferenceError::NonFiniteObservation { .. } => {
                Self::PolicyInferenceNonFiniteObservation
            }
            AdaptiveInferenceError::NegativeObservationMagnitude { .. } => {
                Self::PolicyInferenceNegativeObservationMagnitude
            }
            AdaptiveInferenceError::VerifierSignalForbidden { .. } => {
                Self::PolicyInferenceVerifierSignalForbidden
            }
            AdaptiveInferenceError::CheckpointMetadataForbidden { .. } => {
                Self::PolicyInferenceCheckpointMetadataForbidden
            }
            AdaptiveInferenceError::ActionForbidden { .. } => Self::PolicyInferenceActionForbidden,
            AdaptiveInferenceError::ZeroComputeEnvelope => Self::PolicyInferenceZeroComputeEnvelope,
            AdaptiveInferenceError::ZeroMemoryEnvelope => Self::PolicyInferenceZeroMemoryEnvelope,
            AdaptiveInferenceError::ResourceAccountingOverflow => {
                Self::PolicyInferenceResourceAccountingOverflow
            }
            AdaptiveInferenceError::ComputeEnvelopeExceeded { .. } => {
                Self::PolicyInferenceComputeEnvelopeExceeded
            }
            AdaptiveInferenceError::MemoryEnvelopeExceeded { .. } => {
                Self::PolicyInferenceMemoryEnvelopeExceeded
            }
        }
    }
}

/// Immutable evaluator-side provenance for one rejected trajectory.
///
/// The record intentionally has no success/failure field because no normal task
/// evaluation completed. The original typed error is retained alongside the
/// stable code so numeric categorization never discards diagnostic information.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReferenceRejectionRecord {
    arm: PolicyArm,
    family: AdaptiveTaskFamily,
    stratum: DifficultyStratum,
    seed: u64,
    code: ReferenceRejectionCode,
    error: ReferenceEvaluatorError,
}

impl ReferenceRejectionRecord {
    #[must_use]
    pub const fn arm(self) -> PolicyArm {
        self.arm
    }

    #[must_use]
    pub const fn family(self) -> AdaptiveTaskFamily {
        self.family
    }

    #[must_use]
    pub const fn stratum(self) -> DifficultyStratum {
        self.stratum
    }

    #[must_use]
    pub const fn seed(self) -> u64 {
        self.seed
    }

    #[must_use]
    pub const fn code(self) -> ReferenceRejectionCode {
        self.code
    }

    #[must_use]
    pub const fn error(self) -> ReferenceEvaluatorError {
        self.error
    }
}

/// Recorded non-final evaluator outcome. Rejections are structurally distinct
/// from completed quality evaluations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ReferenceEvaluationOutcome {
    Completed(ReferenceEvaluationRecord),
    Rejected(ReferenceRejectionRecord),
}

/// Execute one complete non-final trajectory while retaining typed provenance
/// for technical rejection.
///
/// This is additive: [`evaluate_generated_task`] keeps its existing `Result`
/// contract unchanged for compatibility.
#[must_use]
pub fn evaluate_generated_task_recorded(
    generated: GeneratedTask,
    policy: ReferencePolicy,
    envelope: ResourceEnvelope,
    runtime_decision_limit: u64,
) -> ReferenceEvaluationOutcome {
    let arm = policy.arm();
    let family = generated.policy().family();
    let evaluator = generated.evaluator();
    match evaluate_generated_task(generated, policy, envelope, runtime_decision_limit) {
        Ok(record) => ReferenceEvaluationOutcome::Completed(record),
        Err(error) => ReferenceEvaluationOutcome::Rejected(ReferenceRejectionRecord {
            arm,
            family,
            stratum: evaluator.stratum(),
            seed: evaluator.seed(),
            code: ReferenceRejectionCode::from_error(error),
            error,
        }),
    }
}
