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
#[path = "../src/adaptive_task_generators.rs"]
mod adaptive_task_generators;

use adaptive_evaluator::{
    ReferenceEvaluationRecord, ReferenceEvaluatorError, ReferencePolicy, evaluate_generated_task,
};
use adaptive_inference::{PolicyArm, ResourceEnvelope};
use adaptive_policies::{C0FixedPolicy, C1StaticPolicy, C2AdaptivePolicy, C3RecoveryPolicy};
use adaptive_task_generators::{
    DifficultyStratum, GeneratedTask, P1Config, P2Config, P3Config, generate_p1, generate_p2,
    generate_p3,
};

fn roomy_envelope() -> ResourceEnvelope {
    ResourceEnvelope::new(1_000_000, 1_000_000).expect("valid non-final software fixture envelope")
}

fn p1() -> GeneratedTask {
    generate_p1(
        P1Config::new(5, 3).expect("P1 config"),
        DifficultyStratum::Shallow,
        0x9101,
    )
    .expect("P1 generation")
}

fn p2() -> GeneratedTask {
    generate_p2(
        P2Config::new(4).expect("P2 config"),
        DifficultyStratum::Intermediate,
        0x9202,
    )
    .expect("P2 generation")
}

fn p3() -> GeneratedTask {
    generate_p3(
        P3Config::new(4, 3).expect("P3 config"),
        DifficultyStratum::Deep,
        0xabc0_1234,
    )
    .expect("P3 generation")
}

fn evaluate(
    task: GeneratedTask,
    policy: impl Into<ReferencePolicy>,
) -> ReferenceEvaluationRecord {
    evaluate_generated_task(task, policy.into(), roomy_envelope(), 128)
        .expect("bounded integrated reference evaluation")
}

#[test]
fn c0_and_c1_integrate_without_observation_adaptation() {
    for task in [p1(), p2(), p3()] {
        let event_count = u64::try_from(task.policy().event_count()).expect("small fixture task");
        let c0 = evaluate(
            task.clone(),
            C0FixedPolicy::new(event_count).expect("C0 schedule"),
        );
        assert_eq!(c0.arm(), PolicyArm::C0FixedCompute);
        assert_eq!(c0.family(), task.policy().family());
        assert_eq!(c0.seed(), task.evaluator().seed());
        assert_eq!(c0.stratum(), task.evaluator().stratum());
        assert_eq!(c0.stop_step(), event_count);
        assert!(c0.usage().policy_ops() > 0);
        assert_eq!(c0.usage().verifier_ops(), 0);
        assert_eq!(c0.usage().checkpoint_ops(), 0);

        let c1 = evaluate(
            task.clone(),
            C1StaticPolicy::new(5, 4, 9).expect("C1 family schedules"),
        );
        assert_eq!(c1.arm(), PolicyArm::C1StaticPreallocation);
        assert_eq!(c1.family(), task.policy().family());
        assert!(c1.usage().policy_ops() > c0.usage().policy_ops());
        assert!(c1.usage().policy_memory_bits() >= 192);
        assert_eq!(c1.usage().verifier_ops(), 0);
    }
}

#[test]
fn c2_full_forward_evaluation_uses_post_stop_target_only() {
    let task = p1();
    let record = evaluate(
        task.clone(),
        C2AdaptivePolicy::new(0, 0.0, 0.0, f64::MAX).expect("C2 full-forward policy"),
    );
    assert_eq!(record.arm(), PolicyArm::C2AdaptiveStopping);
    assert_eq!(record.family(), task.policy().family());
    assert!(record.success());
    assert_eq!(record.stop_step(), 5);
    assert!(record.usage().solver_ops() > 0);
    assert!(record.usage().policy_ops() > 0);
    assert_eq!(record.usage().verifier_ops(), 0);
    assert_eq!(record.usage().checkpoint_ops(), 0);
}

#[test]
fn p3_c2_c3_contrast_survives_complete_evaluator_integration() {
    let task = p3();
    let c2 = evaluate(
        task.clone(),
        C2AdaptivePolicy::new(0, 0.0, 0.0, f64::MAX).expect("C2 full-forward policy"),
    );
    assert!(!c2.success());
    assert_eq!(c2.usage().verifier_ops(), 0);
    assert_eq!(c2.usage().checkpoint_ops(), 0);
    assert_eq!(c2.usage().replay_ops(), 0);

    let c3 = evaluate(
        task,
        C3RecoveryPolicy::new(0, 0.0, 0.0, f64::MAX, 1, 1, true)
            .expect("C3 recovery fixture policy"),
    );
    assert!(c3.success());
    assert_eq!(c3.arm(), PolicyArm::C3VerificationRecovery);
    assert!(c3.usage().verifier_ops() > 0);
    assert!(c3.usage().checkpoint_ops() > 0);
    assert!(c3.usage().replay_ops() > 0);
    assert!(c3.checkpoint_traffic().store_bytes() > 0);
    assert!(c3.checkpoint_traffic().restore_bytes() > 0);
}

#[test]
fn caller_supplied_decision_guard_rejects_without_evaluating() {
    let error = evaluate_generated_task(
        p1(),
        C0FixedPolicy::new(5).expect("C0 schedule").into(),
        roomy_envelope(),
        2,
    )
    .expect_err("technical decision guard must reject");
    assert_eq!(
        error,
        ReferenceEvaluatorError::DecisionLimitExceeded { limit: 2 }
    );

    let zero = evaluate_generated_task(
        p1(),
        C0FixedPolicy::new(5).expect("C0 schedule").into(),
        roomy_envelope(),
        0,
    )
    .expect_err("zero guard must reject");
    assert_eq!(zero, ReferenceEvaluatorError::ZeroDecisionLimit);
}
