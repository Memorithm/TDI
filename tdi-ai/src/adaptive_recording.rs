//! Machine-readable non-final TDI-9.1 evaluation outcome recording.
//!
//! This layer is intentionally additive: the already-qualified
//! `evaluate_generated_task` API keeps returning its typed `Result`, while this
//! module records either a completed evaluator result or a lossless technical
//! rejection. Rejections are never reinterpreted as task-quality failures.

use crate::adaptive_evaluator::{
    ReferenceEvaluationRecord, ReferenceEvaluatorError, ReferencePolicy, evaluate_generated_task,
};
use crate::adaptive_execution::ReferenceExecutionError;
use crate::adaptive_inference::{
    AdaptiveInferenceError, InferenceAction, ObservationField, PolicyArm, ResourceEnvelope,
};
use crate::adaptive_policies::{PolicyConfigField, ReferencePolicyError};
use crate::adaptive_task_generators::{
    AdaptiveTaskFamily, DifficultyStratum, GeneratedTask,
};

/// Lossless machine-readable form of one adaptive-inference contract rejection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AdaptiveInferenceRejectionCode {
    NonFiniteObservation { field: ObservationField },
    NegativeObservationMagnitude { field: ObservationField },
    VerifierSignalForbidden { arm: PolicyArm },
    CheckpointMetadataForbidden { arm: PolicyArm },
    ActionForbidden { arm: PolicyArm, action: InferenceAction },
    ZeroComputeEnvelope,
    ZeroMemoryEnvelope,
    ResourceAccountingOverflow,
    ComputeEnvelopeExceeded { maximum: u64, requested: u64 },
    MemoryEnvelopeExceeded { maximum: u64, requested: u64 },
}

impl From<AdaptiveInferenceError> for AdaptiveInferenceRejectionCode {
    fn from(value: AdaptiveInferenceError) -> Self {
        match value {
            AdaptiveInferenceError::NonFiniteObservation { field } => {
                Self::NonFiniteObservation { field }
            }
            AdaptiveInferenceError::NegativeObservationMagnitude { field } => {
                Self::NegativeObservationMagnitude { field }
            }
            AdaptiveInferenceError::VerifierSignalForbidden { arm } => {
                Self::VerifierSignalForbidden { arm }
            }
            AdaptiveInferenceError::CheckpointMetadataForbidden { arm } => {
                Self::CheckpointMetadataForbidden { arm }
            }
            AdaptiveInferenceError::ActionForbidden { arm, action } => {
                Self::ActionForbidden { arm, action }
            }
            AdaptiveInferenceError::ZeroComputeEnvelope => Self::ZeroComputeEnvelope,
            AdaptiveInferenceError::ZeroMemoryEnvelope => Self::ZeroMemoryEnvelope,
            AdaptiveInferenceError::ResourceAccountingOverflow => Self::ResourceAccountingOverflow,
            AdaptiveInferenceError::ComputeEnvelopeExceeded { maximum, requested } => {
                Self::ComputeEnvelopeExceeded { maximum, requested }
            }
            AdaptiveInferenceError::MemoryEnvelopeExceeded { maximum, requested } => {
                Self::MemoryEnvelopeExceeded { maximum, requested }
            }
        }
    }
}

/// Lossless machine-readable form of one deterministic execution rejection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ReferenceExecutionRejectionCode {
    AdaptiveInference(AdaptiveInferenceRejectionCode),
    AlreadyStopped,
    SolverExhausted,
    TaskTooLarge,
    TaskStateMismatch,
    TaskContractViolation,
    ArithmeticOverflow,
    CheckpointTrafficOverflow,
    AccountingInvariant,
    CheckpointUnavailable,
    CheckpointNotEarlier,
    BacktrackRequiresViolation,
    BacktrackUnsupportedForTask,
    EvaluatorFamilyMismatch,
}

impl From<ReferenceExecutionError> for ReferenceExecutionRejectionCode {
    fn from(value: ReferenceExecutionError) -> Self {
        match value {
            ReferenceExecutionError::AdaptiveInference(error) => {
                Self::AdaptiveInference(error.into())
            }
            ReferenceExecutionError::AlreadyStopped => Self::AlreadyStopped,
            ReferenceExecutionError::SolverExhausted => Self::SolverExhausted,
            ReferenceExecutionError::TaskTooLarge => Self::TaskTooLarge,
            ReferenceExecutionError::TaskStateMismatch => Self::TaskStateMismatch,
            ReferenceExecutionError::TaskContractViolation => Self::TaskContractViolation,
            ReferenceExecutionError::ArithmeticOverflow => Self::ArithmeticOverflow,
            ReferenceExecutionError::CheckpointTrafficOverflow => Self::CheckpointTrafficOverflow,
            ReferenceExecutionError::AccountingInvariant => Self::AccountingInvariant,
            ReferenceExecutionError::CheckpointUnavailable => Self::CheckpointUnavailable,
            ReferenceExecutionError::CheckpointNotEarlier => Self::CheckpointNotEarlier,
            ReferenceExecutionError::BacktrackRequiresViolation => {
                Self::BacktrackRequiresViolation
            }
            ReferenceExecutionError::BacktrackUnsupportedForTask => {
                Self::BacktrackUnsupportedForTask
            }
            ReferenceExecutionError::EvaluatorFamilyMismatch => Self::EvaluatorFamilyMismatch,
        }
    }
}

/// Lossless machine-readable form of one reference-policy rejection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ReferencePolicyRejectionCode {
    AdaptiveInference(AdaptiveInferenceRejectionCode),
    ZeroFixedSchedule,
    ZeroStaticSchedule { family: AdaptiveTaskFamily },
    NonFiniteConfiguration { field: PolicyConfigField },
    NegativeConfiguration { field: PolicyConfigField },
    ZeroVerificationCadence,
    UnrecoverableVerifiedViolation,
}

impl From<ReferencePolicyError> for ReferencePolicyRejectionCode {
    fn from(value: ReferencePolicyError) -> Self {
        match value {
            ReferencePolicyError::AdaptiveInference(error) => {
                Self::AdaptiveInference(error.into())
            }
            ReferencePolicyError::ZeroFixedSchedule => Self::ZeroFixedSchedule,
            ReferencePolicyError::ZeroStaticSchedule { family } => {
                Self::ZeroStaticSchedule { family }
            }
            ReferencePolicyError::NonFiniteConfiguration { field } => {
                Self::NonFiniteConfiguration { field }
            }
            ReferencePolicyError::NegativeConfiguration { field } => {
                Self::NegativeConfiguration { field }
            }
            ReferencePolicyError::ZeroVerificationCadence => Self::ZeroVerificationCadence,
            ReferencePolicyError::UnrecoverableVerifiedViolation => {
                Self::UnrecoverableVerifiedViolation
            }
        }
    }
}

/// Lossless machine-readable code for every current evaluator rejection path.
///
/// The enum preserves the layer where a failure originated. It deliberately has
/// no Beneficial/Equivalent/Harmful/Incorrect interpretation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ReferenceRejectionCode {
    Execution(ReferenceExecutionRejectionCode),
    Policy(ReferencePolicyRejectionCode),
    ZeroDecisionLimit,
    DecisionLimitExceeded { limit: u64 },
    DecisionCountOverflow,
    MissingC1Plan,
    GeneratorFamilyMismatch,
    PolicyArmDrift { expected: PolicyArm, actual: PolicyArm },
}

impl From<ReferenceEvaluatorError> for ReferenceRejectionCode {
    fn from(value: ReferenceEvaluatorError) -> Self {
        match value {
            ReferenceEvaluatorError::Execution(error) => Self::Execution(error.into()),
            ReferenceEvaluatorError::Policy(error) => Self::Policy(error.into()),
            ReferenceEvaluatorError::ZeroDecisionLimit => Self::ZeroDecisionLimit,
            ReferenceEvaluatorError::DecisionLimitExceeded { limit } => {
                Self::DecisionLimitExceeded { limit }
            }
            ReferenceEvaluatorError::DecisionCountOverflow => Self::DecisionCountOverflow,
            ReferenceEvaluatorError::MissingC1Plan => Self::MissingC1Plan,
            ReferenceEvaluatorError::GeneratorFamilyMismatch => Self::GeneratorFamilyMismatch,
            ReferenceEvaluatorError::PolicyArmDrift { expected, actual } => {
                Self::PolicyArmDrift { expected, actual }
            }
        }
    }
}

/// Evaluator-owned provenance for one rejected trajectory.
///
/// Seed and hidden stratum are copied from the evaluator record only by this
/// post-decision recording layer. They are never inserted into `PolicyTask`,
/// `PolicyObservation`, `ReferenceExecution`, or any policy decision API.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ReferenceRejectionRecord {
    arm: PolicyArm,
    family: AdaptiveTaskFamily,
    stratum: DifficultyStratum,
    seed: u64,
    code: ReferenceRejectionCode,
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
}

/// One recorded non-final evaluator outcome.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ReferenceRecordedOutcome {
    Completed(ReferenceEvaluationRecord),
    Rejected(ReferenceRejectionRecord),
}

/// Execute the already-qualified evaluator and retain technical rejections.
///
/// This function does not change evaluation semantics. It only captures
/// evaluator-side provenance before ownership of `GeneratedTask` moves into the
/// existing `evaluate_generated_task` entry point.
#[must_use]
pub fn evaluate_generated_task_recorded(
    generated: GeneratedTask,
    policy: ReferencePolicy,
    envelope: ResourceEnvelope,
    runtime_decision_limit: u64,
) -> ReferenceRecordedOutcome {
    let arm = policy.arm();
    let evaluator = generated.evaluator();
    let family = generated.policy().family();

    match evaluate_generated_task(generated, policy, envelope, runtime_decision_limit) {
        Ok(record) => ReferenceRecordedOutcome::Completed(record),
        Err(error) => ReferenceRecordedOutcome::Rejected(ReferenceRejectionRecord {
            arm,
            family,
            stratum: evaluator.stratum(),
            seed: evaluator.seed(),
            code: error.into(),
        }),
    }
}
