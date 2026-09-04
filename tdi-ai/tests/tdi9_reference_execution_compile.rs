#[allow(dead_code)]
#[path = "../src/adaptive_execution.rs"]
mod adaptive_execution;
#[allow(dead_code)]
#[path = "../src/adaptive_inference.rs"]
mod adaptive_inference;
#[allow(dead_code)]
#[path = "../src/adaptive_task_generators.rs"]
mod adaptive_task_generators;

use adaptive_execution::{
    P3_CHECKPOINT_BYTES, ReferenceExecution, ReferenceExecutionError, evaluate_stopped,
};
use adaptive_inference::{
    AdaptiveInferenceError, InferenceAction, PolicyArm, ResourceEnvelope, VerifierSignal,
};
use adaptive_task_generators::{
    DifficultyStratum, P1Config, P2Config, P3Config, generate_p1, generate_p2, generate_p3,
};

fn roomy_envelope() -> ResourceEnvelope {
    ResourceEnvelope::new(1_000_000, 1_000_000).expect("valid roomy envelope")
}

#[test]
fn p1_c2_can_stop_from_current_observation_without_evaluator_metadata() {
    let generated = generate_p1(
        P1Config::new(11, 8).expect("valid P1 config"),
        DifficultyStratum::Intermediate,
        0x1020_3040,
    )
    .expect("P1 generation");
    let (policy_task, evaluator) = generated.into_parts();
    let mut execution =
        ReferenceExecution::new(PolicyArm::C2AdaptiveStopping, policy_task, roomy_envelope())
            .expect("reference execution");

    loop {
        let observation = execution.observation().expect("current observation");
        if observation.score_margin().abs() > observation.residual() {
            break;
        }
        execution.continue_step().expect("P1 transition");
    }

    let stopped = execution.stop().expect("STOP");
    assert!(evaluate_stopped(stopped, evaluator).expect("post-STOP evaluation"));
    assert!(stopped.step_index() < 11);
    assert_eq!(stopped.accounting().usage().verifier_ops(), 0);
}

#[test]
fn p2_c3_verifier_is_costed_and_can_certify_a_candidate() {
    let generated = generate_p2(
        P2Config::new(7).expect("valid P2 config"),
        DifficultyStratum::Intermediate,
        0x5566_7788,
    )
    .expect("P2 generation");
    let (policy_task, evaluator) = generated.into_parts();
    let mut execution = ReferenceExecution::new(
        PolicyArm::C3VerificationRecovery,
        policy_task,
        roomy_envelope(),
    )
    .expect("reference execution");

    loop {
        let signal = execution.verify().expect("P2 verifier");
        if signal == VerifierSignal::Satisfied {
            break;
        }
        execution.continue_step().expect("P2 transition");
    }

    let stopped = execution.stop().expect("STOP");
    assert!(evaluate_stopped(stopped, evaluator).expect("post-STOP evaluation"));
    assert!(stopped.accounting().usage().verifier_ops() > 0);
}

#[test]
fn p3_c2_fails_but_c3_verify_backtrack_replay_recovers_without_target_input() {
    let generated = generate_p3(
        P3Config::new(4, 3).expect("valid P3 config"),
        DifficultyStratum::Deep,
        0xabc0_1234,
    )
    .expect("P3 generation");
    let (policy_task, evaluator) = generated.into_parts();

    let mut c2 = ReferenceExecution::new(
        PolicyArm::C2AdaptiveStopping,
        policy_task.clone(),
        roomy_envelope(),
    )
    .expect("C2 execution");
    while c2.observation().expect("C2 observation").residual() > 0.0 {
        c2.continue_step().expect("C2 transition");
    }
    let c2_stopped = c2.stop().expect("C2 STOP");
    assert!(!evaluate_stopped(c2_stopped, evaluator).expect("C2 evaluation"));

    let mut c3 = ReferenceExecution::new(
        PolicyArm::C3VerificationRecovery,
        policy_task,
        roomy_envelope(),
    )
    .expect("C3 execution");
    let violation_step = loop {
        c3.continue_step().expect("C3 transition");
        match c3.verify().expect("C3 verifier") {
            VerifierSignal::Violated => break c3.step_index(),
            VerifierSignal::Satisfied | VerifierSignal::Indeterminate => {}
        }
    };
    assert!(c3.checkpoint_available());
    c3.backtrack().expect("C3 backtrack");
    assert_eq!(c3.step_index(), 0);

    while c3.step_index() < violation_step {
        c3.continue_step().expect("replayed transition");
    }
    while c3.observation().expect("C3 observation").residual() > 0.0 {
        c3.continue_step().expect("post-recovery transition");
    }

    let stopped = c3.stop().expect("C3 STOP");
    assert!(evaluate_stopped(stopped, evaluator).expect("C3 evaluation"));
    let accounting = stopped.accounting();
    assert_eq!(
        accounting.checkpoint_traffic().store_bytes(),
        P3_CHECKPOINT_BYTES
    );
    assert_eq!(
        accounting.checkpoint_traffic().restore_bytes(),
        P3_CHECKPOINT_BYTES
    );
    assert_eq!(
        accounting.usage().checkpoint_ops(),
        2 * P3_CHECKPOINT_BYTES
    );
    assert!(accounting.usage().replay_ops() > 0);
    assert!(accounting.usage().checkpoint_memory_bits() >= 8 * P3_CHECKPOINT_BYTES);
}

#[test]
fn non_c3_arms_cannot_invoke_verify_or_backtrack() {
    let generated = generate_p2(
        P2Config::new(5).expect("valid P2 config"),
        DifficultyStratum::Shallow,
        7,
    )
    .expect("P2 generation");
    let (policy_task, _) = generated.into_parts();
    let mut execution =
        ReferenceExecution::new(PolicyArm::C2AdaptiveStopping, policy_task, roomy_envelope())
            .expect("C2 execution");

    assert!(matches!(
        execution.verify(),
        Err(ReferenceExecutionError::AdaptiveInference(
            AdaptiveInferenceError::ActionForbidden {
                action: InferenceAction::Verify,
                ..
            }
        ))
    ));
    assert!(matches!(
        execution.backtrack(),
        Err(ReferenceExecutionError::AdaptiveInference(
            AdaptiveInferenceError::ActionForbidden {
                action: InferenceAction::Backtrack,
                ..
            }
        ))
    ));
}

#[test]
fn rejected_compute_charge_does_not_partially_store_a_checkpoint() {
    let generated = generate_p3(
        P3Config::new(2, 2).expect("valid P3 config"),
        DifficultyStratum::Shallow,
        12,
    )
    .expect("P3 generation");
    let (policy_task, _) = generated.into_parts();
    let envelope = ResourceEnvelope::new(1, 100_000).expect("valid constrained envelope");
    let mut execution =
        ReferenceExecution::new(PolicyArm::C3VerificationRecovery, policy_task, envelope)
            .expect("initial state fits");
    let before = execution.accounting();

    assert!(matches!(
        execution.continue_step(),
        Err(ReferenceExecutionError::AdaptiveInference(
            AdaptiveInferenceError::ComputeEnvelopeExceeded { .. }
        ))
    ));
    assert_eq!(execution.step_index(), 0);
    assert!(!execution.checkpoint_available());
    assert_eq!(execution.accounting(), before);
}

#[test]
fn rejected_memory_state_does_not_partially_commit_solver_or_checkpoint() {
    let generated = generate_p3(
        P3Config::new(2, 2).expect("valid P3 config"),
        DifficultyStratum::Shallow,
        13,
    )
    .expect("P3 generation");
    let (policy_task, _) = generated.into_parts();
    let envelope = ResourceEnvelope::new(100_000, 700).expect("valid constrained envelope");
    let mut execution =
        ReferenceExecution::new(PolicyArm::C3VerificationRecovery, policy_task, envelope)
            .expect("initial state fits");
    let before = execution.accounting();

    assert!(matches!(
        execution.continue_step(),
        Err(ReferenceExecutionError::AdaptiveInference(
            AdaptiveInferenceError::MemoryEnvelopeExceeded { .. }
        ))
    ));
    assert_eq!(execution.step_index(), 0);
    assert!(!execution.checkpoint_available());
    assert_eq!(execution.accounting(), before);
}

#[test]
fn policy_decision_hook_accounts_compute_and_policy_memory_without_choosing_action() {
    let generated = generate_p1(
        P1Config::new(7, 5).expect("valid P1 config"),
        DifficultyStratum::Shallow,
        19,
    )
    .expect("P1 generation");
    let (policy_task, _) = generated.into_parts();
    let mut execution =
        ReferenceExecution::new(PolicyArm::C2AdaptiveStopping, policy_task, roomy_envelope())
            .expect("C2 execution");

    execution
        .charge_policy_decision(17, 96)
        .expect("policy accounting hook");
    let usage = execution.accounting().usage();
    assert_eq!(usage.policy_ops(), 17);
    assert_eq!(usage.policy_memory_bits(), 96);
    assert_eq!(execution.step_index(), 0);
}
