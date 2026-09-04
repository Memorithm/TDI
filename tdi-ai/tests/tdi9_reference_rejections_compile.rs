#[allow(dead_code)]
#[path = "../src/adaptive_evaluator.rs"]
mod adaptive_evaluator;
#[allow(dead_code)]
#[path = "../src/adaptive_execution.rs"]
mod adaptive_execution;
#[allow(dead_code)]
#[path = "../src/adaptive_inference.rs"]
mod adaptive_inference;
#[allow(dead_code)]
#[path = "../src/adaptive_policies.rs"]
mod adaptive_policies;
#[allow(dead_code)]
#[path = "../src/adaptive_rejections.rs"]
mod adaptive_rejections;
#[allow(dead_code)]
#[path = "../src/adaptive_task_generators.rs"]
mod adaptive_task_generators;

use adaptive_evaluator::{ReferenceEvaluatorError, ReferencePolicy};
use adaptive_execution::ReferenceExecutionError;
use adaptive_inference::{AdaptiveInferenceError, InferenceAction, PolicyArm, ResourceEnvelope};
use adaptive_policies::{C0FixedPolicy, ReferencePolicyError};
use adaptive_rejections::{
    ReferenceEvaluationOutcome, ReferenceRejectionCode, evaluate_generated_task_recorded,
};
use adaptive_task_generators::{
    DifficultyStratum, GeneratedTask, P1Config, generate_p1,
};

fn p1() -> GeneratedTask {
    generate_p1(
        P1Config::new(5, 3).expect("P1 config"),
        DifficultyStratum::Shallow,
        0x9139,
    )
    .expect("P1 generation")
}

fn roomy_envelope() -> ResourceEnvelope {
    ResourceEnvelope::new(1_000_000, 1_000_000).expect("positive non-final fixture envelope")
}

fn c0() -> ReferencePolicy {
    C0FixedPolicy::new(5).expect("C0 schedule").into()
}

#[test]
fn recorded_completion_remains_a_normal_evaluation() {
    let task = p1();
    let outcome = evaluate_generated_task_recorded(task.clone(), c0(), roomy_envelope(), 32);
    let ReferenceEvaluationOutcome::Completed(record) = outcome else {
        panic!("roomy bounded fixture must complete normally");
    };
    assert_eq!(record.arm(), PolicyArm::C0FixedCompute);
    assert_eq!(record.family(), task.policy().family());
    assert_eq!(record.stratum(), task.evaluator().stratum());
    assert_eq!(record.seed(), task.evaluator().seed());
}

#[test]
fn decision_limit_rejection_retains_evaluator_side_provenance() {
    let task = p1();
    let outcome = evaluate_generated_task_recorded(task.clone(), c0(), roomy_envelope(), 2);
    let ReferenceEvaluationOutcome::Rejected(record) = outcome else {
        panic!("caller technical guard must reject");
    };
    assert_eq!(record.arm(), PolicyArm::C0FixedCompute);
    assert_eq!(record.family(), task.policy().family());
    assert_eq!(record.stratum(), task.evaluator().stratum());
    assert_eq!(record.seed(), task.evaluator().seed());
    assert_eq!(
        record.code(),
        ReferenceRejectionCode::EvaluatorDecisionLimitExceeded
    );
    assert_eq!(record.code().numeric(), 0x0102);
    assert_eq!(
        record.error(),
        ReferenceEvaluatorError::DecisionLimitExceeded { limit: 2 }
    );
}

#[test]
fn resource_exhaustion_is_rejection_not_quality_failure() {
    let task = p1();
    let tiny_memory = ResourceEnvelope::new(1_000_000, 1).expect("positive envelope");
    let outcome = evaluate_generated_task_recorded(task, c0(), tiny_memory, 32);
    let ReferenceEvaluationOutcome::Rejected(record) = outcome else {
        panic!("insufficient declared memory must reject");
    };
    assert_eq!(
        record.code(),
        ReferenceRejectionCode::ExecutionInferenceMemoryEnvelopeExceeded
    );
    assert!(matches!(
        record.error(),
        ReferenceEvaluatorError::Execution(ReferenceExecutionError::AdaptiveInference(
            AdaptiveInferenceError::MemoryEnvelopeExceeded { .. }
        ))
    ));
}

#[test]
fn typed_mapping_keeps_execution_policy_and_inference_paths_distinct() {
    let execution = ReferenceEvaluatorError::Execution(ReferenceExecutionError::SolverExhausted);
    assert_eq!(
        ReferenceRejectionCode::from_error(execution),
        ReferenceRejectionCode::ExecutionSolverExhausted
    );

    let policy = ReferenceEvaluatorError::Policy(ReferencePolicyError::ZeroVerificationCadence);
    assert_eq!(
        ReferenceRejectionCode::from_error(policy),
        ReferenceRejectionCode::PolicyZeroVerificationCadence
    );

    let policy_inference = ReferenceEvaluatorError::Policy(
        ReferencePolicyError::AdaptiveInference(AdaptiveInferenceError::ActionForbidden {
            arm: PolicyArm::C2AdaptiveStopping,
            action: InferenceAction::Verify,
        }),
    );
    assert_eq!(
        ReferenceRejectionCode::from_error(policy_inference),
        ReferenceRejectionCode::PolicyInferenceActionForbidden
    );

    let execution_inference = ReferenceEvaluatorError::Execution(
        ReferenceExecutionError::AdaptiveInference(AdaptiveInferenceError::ActionForbidden {
            arm: PolicyArm::C2AdaptiveStopping,
            action: InferenceAction::Verify,
        }),
    );
    assert_eq!(
        ReferenceRejectionCode::from_error(execution_inference),
        ReferenceRejectionCode::ExecutionInferenceActionForbidden
    );
    assert_ne!(
        ReferenceRejectionCode::from_error(policy_inference),
        ReferenceRejectionCode::from_error(execution_inference)
    );
}

#[test]
fn rejection_numeric_codes_are_exact_and_stable() {
    let expected = [
        (ReferenceRejectionCode::EvaluatorZeroDecisionLimit, 0x0101),
        (
            ReferenceRejectionCode::EvaluatorDecisionLimitExceeded,
            0x0102,
        ),
        (
            ReferenceRejectionCode::EvaluatorDecisionCountOverflow,
            0x0103,
        ),
        (ReferenceRejectionCode::EvaluatorMissingC1Plan, 0x0104),
        (
            ReferenceRejectionCode::EvaluatorGeneratorFamilyMismatch,
            0x0105,
        ),
        (ReferenceRejectionCode::EvaluatorPolicyArmDrift, 0x0106),
        (ReferenceRejectionCode::ExecutionAlreadyStopped, 0x0201),
        (ReferenceRejectionCode::ExecutionSolverExhausted, 0x0202),
        (ReferenceRejectionCode::ExecutionTaskTooLarge, 0x0203),
        (ReferenceRejectionCode::ExecutionTaskStateMismatch, 0x0204),
        (
            ReferenceRejectionCode::ExecutionTaskContractViolation,
            0x0205,
        ),
        (ReferenceRejectionCode::ExecutionArithmeticOverflow, 0x0206),
        (
            ReferenceRejectionCode::ExecutionCheckpointTrafficOverflow,
            0x0207,
        ),
        (
            ReferenceRejectionCode::ExecutionAccountingInvariant,
            0x0208,
        ),
        (
            ReferenceRejectionCode::ExecutionCheckpointUnavailable,
            0x0209,
        ),
        (
            ReferenceRejectionCode::ExecutionCheckpointNotEarlier,
            0x020a,
        ),
        (
            ReferenceRejectionCode::ExecutionBacktrackRequiresViolation,
            0x020b,
        ),
        (
            ReferenceRejectionCode::ExecutionBacktrackUnsupportedForTask,
            0x020c,
        ),
        (
            ReferenceRejectionCode::ExecutionEvaluatorFamilyMismatch,
            0x020d,
        ),
        (
            ReferenceRejectionCode::ExecutionInferenceNonFiniteObservation,
            0x0301,
        ),
        (
            ReferenceRejectionCode::ExecutionInferenceNegativeObservationMagnitude,
            0x0302,
        ),
        (
            ReferenceRejectionCode::ExecutionInferenceVerifierSignalForbidden,
            0x0303,
        ),
        (
            ReferenceRejectionCode::ExecutionInferenceCheckpointMetadataForbidden,
            0x0304,
        ),
        (
            ReferenceRejectionCode::ExecutionInferenceActionForbidden,
            0x0305,
        ),
        (
            ReferenceRejectionCode::ExecutionInferenceZeroComputeEnvelope,
            0x0306,
        ),
        (
            ReferenceRejectionCode::ExecutionInferenceZeroMemoryEnvelope,
            0x0307,
        ),
        (
            ReferenceRejectionCode::ExecutionInferenceResourceAccountingOverflow,
            0x0308,
        ),
        (
            ReferenceRejectionCode::ExecutionInferenceComputeEnvelopeExceeded,
            0x0309,
        ),
        (
            ReferenceRejectionCode::ExecutionInferenceMemoryEnvelopeExceeded,
            0x030a,
        ),
        (ReferenceRejectionCode::PolicyZeroFixedSchedule, 0x0401),
        (ReferenceRejectionCode::PolicyZeroStaticSchedule, 0x0402),
        (
            ReferenceRejectionCode::PolicyNonFiniteConfiguration,
            0x0403,
        ),
        (ReferenceRejectionCode::PolicyNegativeConfiguration, 0x0404),
        (
            ReferenceRejectionCode::PolicyZeroVerificationCadence,
            0x0405,
        ),
        (
            ReferenceRejectionCode::PolicyUnrecoverableVerifiedViolation,
            0x0406,
        ),
        (
            ReferenceRejectionCode::PolicyInferenceNonFiniteObservation,
            0x0501,
        ),
        (
            ReferenceRejectionCode::PolicyInferenceNegativeObservationMagnitude,
            0x0502,
        ),
        (
            ReferenceRejectionCode::PolicyInferenceVerifierSignalForbidden,
            0x0503,
        ),
        (
            ReferenceRejectionCode::PolicyInferenceCheckpointMetadataForbidden,
            0x0504,
        ),
        (
            ReferenceRejectionCode::PolicyInferenceActionForbidden,
            0x0505,
        ),
        (
            ReferenceRejectionCode::PolicyInferenceZeroComputeEnvelope,
            0x0506,
        ),
        (
            ReferenceRejectionCode::PolicyInferenceZeroMemoryEnvelope,
            0x0507,
        ),
        (
            ReferenceRejectionCode::PolicyInferenceResourceAccountingOverflow,
            0x0508,
        ),
        (
            ReferenceRejectionCode::PolicyInferenceComputeEnvelopeExceeded,
            0x0509,
        ),
        (
            ReferenceRejectionCode::PolicyInferenceMemoryEnvelopeExceeded,
            0x050a,
        ),
    ];

    assert_eq!(expected.len(), 45);
    for (code, numeric) in expected {
        assert_eq!(code.numeric(), numeric);
    }
}
