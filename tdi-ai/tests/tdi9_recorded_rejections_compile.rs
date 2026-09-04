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
#[path = "../src/adaptive_recording.rs"]
mod adaptive_recording;
#[allow(dead_code)]
#[path = "../src/adaptive_task_generators.rs"]
mod adaptive_task_generators;

use adaptive_evaluator::{ReferenceEvaluatorError, ReferencePolicy};
use adaptive_inference::{AdaptiveInferenceError, InferenceAction, PolicyArm, ResourceEnvelope};
use adaptive_policies::{C0FixedPolicy, ReferencePolicyError};
use adaptive_recording::{
    AdaptiveInferenceRejectionCode, ReferenceExecutionRejectionCode, ReferencePolicyRejectionCode,
    ReferenceRecordedOutcome, ReferenceRejectionCode, evaluate_generated_task_recorded,
};
use adaptive_task_generators::{
    AdaptiveTaskFamily, DifficultyStratum, P1Config, generate_p1,
};

fn p1(seed: u64, stratum: DifficultyStratum) -> adaptive_task_generators::GeneratedTask {
    generate_p1(P1Config::new(5, 3).expect("P1 config"), stratum, seed).expect("P1 generation")
}

fn roomy_envelope() -> ResourceEnvelope {
    ResourceEnvelope::new(1_000_000, 1_000_000).expect("roomy software fixture envelope")
}

#[test]
fn completed_recording_is_bit_identical_to_normal_evaluator_result() {
    let seed = 0x9911;
    let policy: ReferencePolicy = C0FixedPolicy::new(5).expect("C0 schedule").into();
    let expected = adaptive_evaluator::evaluate_generated_task(
        p1(seed, DifficultyStratum::Shallow),
        policy,
        roomy_envelope(),
        32,
    )
    .expect("normal evaluation");

    let recorded = evaluate_generated_task_recorded(
        p1(seed, DifficultyStratum::Shallow),
        policy,
        roomy_envelope(),
        32,
    );
    assert_eq!(recorded, ReferenceRecordedOutcome::Completed(expected));
}

#[test]
fn zero_decision_limit_is_recorded_as_rejection_with_evaluator_provenance() {
    let seed = 0x9922;
    let outcome = evaluate_generated_task_recorded(
        p1(seed, DifficultyStratum::Deep),
        C0FixedPolicy::new(5).expect("C0 schedule").into(),
        roomy_envelope(),
        0,
    );

    let ReferenceRecordedOutcome::Rejected(record) = outcome else {
        panic!("zero decision limit must not become a completed quality result");
    };
    assert_eq!(record.arm(), PolicyArm::C0FixedCompute);
    assert_eq!(
        record.family(),
        AdaptiveTaskFamily::StagedEvidenceAccumulation
    );
    assert_eq!(record.stratum(), DifficultyStratum::Deep);
    assert_eq!(record.seed(), seed);
    assert_eq!(record.code(), ReferenceRejectionCode::ZeroDecisionLimit);
}

#[test]
fn decision_guard_exhaustion_is_not_reinterpreted_as_stop_or_failure() {
    let outcome = evaluate_generated_task_recorded(
        p1(0x9933, DifficultyStratum::Intermediate),
        C0FixedPolicy::new(5).expect("C0 schedule").into(),
        roomy_envelope(),
        2,
    );
    let ReferenceRecordedOutcome::Rejected(record) = outcome else {
        panic!("technical guard exhaustion must remain rejected");
    };
    assert_eq!(
        record.code(),
        ReferenceRejectionCode::DecisionLimitExceeded { limit: 2 }
    );
}

#[test]
fn memory_envelope_exhaustion_remains_nested_execution_rejection() {
    let tiny_memory = ResourceEnvelope::new(1_000_000, 1).expect("positive tiny envelope");
    let outcome = evaluate_generated_task_recorded(
        p1(0x9944, DifficultyStratum::Shallow),
        C0FixedPolicy::new(5).expect("C0 schedule").into(),
        tiny_memory,
        32,
    );
    let ReferenceRecordedOutcome::Rejected(record) = outcome else {
        panic!("memory-envelope exhaustion must remain rejected");
    };
    match record.code() {
        ReferenceRejectionCode::Execution(ReferenceExecutionRejectionCode::AdaptiveInference(
            AdaptiveInferenceRejectionCode::MemoryEnvelopeExceeded { maximum, requested },
        )) => {
            assert_eq!(maximum, 1);
            assert!(requested > maximum);
        }
        other => panic!("unexpected memory rejection code: {other:?}"),
    }
}

#[test]
fn policy_origin_and_inference_payload_are_preserved_losslessly() {
    let original = ReferenceEvaluatorError::Policy(ReferencePolicyError::AdaptiveInference(
        AdaptiveInferenceError::ActionForbidden {
            arm: PolicyArm::C0FixedCompute,
            action: InferenceAction::Verify,
        },
    ));
    let code: ReferenceRejectionCode = original.into();
    assert_eq!(
        code,
        ReferenceRejectionCode::Policy(ReferencePolicyRejectionCode::AdaptiveInference(
            AdaptiveInferenceRejectionCode::ActionForbidden {
                arm: PolicyArm::C0FixedCompute,
                action: InferenceAction::Verify,
            }
        ))
    );
}
