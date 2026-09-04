#[allow(dead_code)]
#[path = "../src/adaptive_inference.rs"]
mod adaptive_inference;
#[allow(dead_code)]
#[path = "../src/adaptive_task_generators.rs"]
mod adaptive_task_generators;
#[allow(dead_code)]
#[path = "../src/adaptive_execution.rs"]
mod adaptive_execution;
#[allow(dead_code)]
#[path = "../src/adaptive_policies.rs"]
mod adaptive_policies;

use adaptive_execution::{ReferenceExecution, StoppedCandidate, evaluate_stopped};
use adaptive_inference::{
    AdaptiveInferenceError, InferenceAction, PolicyArm, PolicyObservation, ResourceEnvelope,
    VerifierSignal,
};
use adaptive_policies::{
    C0FixedPolicy, C1StaticPolicy, C2AdaptivePolicy, C3RecoveryPolicy, PolicyDecision,
    ReferencePolicyError,
};
use adaptive_task_generators::{
    DifficultyStratum, P1Config, P3Config, generate_p1, generate_p3,
};

fn roomy_envelope() -> ResourceEnvelope {
    ResourceEnvelope::new(1_000_000, 1_000_000).expect("valid roomy envelope")
}

fn apply_decision(
    execution: &mut ReferenceExecution,
    decision: PolicyDecision,
) -> Option<StoppedCandidate> {
    assert_eq!(execution.arm(), decision.arm());
    let charge = decision.charge();
    execution
        .charge_policy_decision(charge.operations(), charge.memory_bits())
        .expect("policy charge");
    match decision.action() {
        InferenceAction::Continue => {
            execution.continue_step().expect("CONTINUE");
            None
        }
        InferenceAction::Verify => {
            execution.verify().expect("VERIFY");
            None
        }
        InferenceAction::Backtrack => {
            execution.backtrack().expect("BACKTRACK");
            None
        }
        InferenceAction::Stop => Some(execution.stop().expect("STOP")),
    }
}

#[test]
fn c0_is_a_fixed_schedule_without_observation_input() {
    let policy = C0FixedPolicy::new(3).expect("valid C0 schedule");
    let before = policy.decide(2).expect("C0 decision");
    let at_limit = policy.decide(3).expect("C0 decision");

    assert_eq!(before.arm(), PolicyArm::C0FixedCompute);
    assert_eq!(before.action(), InferenceAction::Continue);
    assert_eq!(at_limit.action(), InferenceAction::Stop);
    assert_eq!(before.charge().operations(), 2);
    assert_eq!(before.charge().memory_bits(), 64);
    assert_eq!(before.charge(), at_limit.charge());
}

#[test]
fn c1_plan_depends_on_family_identity_only() {
    let policy = C1StaticPolicy::new(4, 7, 9).expect("valid C1 schedule");
    let short = generate_p1(
        P1Config::new(5, 3).expect("valid short P1"),
        DifficultyStratum::Shallow,
        11,
    )
    .expect("short P1 generation");
    let long = generate_p1(
        P1Config::new(21, 16).expect("valid long P1"),
        DifficultyStratum::Deep,
        999,
    )
    .expect("long P1 generation");

    let short_plan = policy.plan(short.policy().family());
    let long_plan = policy.plan(long.policy().family());
    assert_eq!(short_plan, long_plan);
    assert_eq!(short_plan.stop_after_steps(), 4);
    assert_eq!(policy.planning_charge().operations(), 2);
    assert_eq!(policy.planning_charge().memory_bits(), 192);

    let runtime = short_plan.decide(0).expect("C1 runtime decision");
    assert_eq!(runtime.arm(), PolicyArm::C1StaticPreallocation);
    assert_eq!(runtime.charge().operations(), 2);
    assert_eq!(runtime.charge().memory_bits(), 64);
}

#[test]
fn c2_rejects_c3_only_observation_fields() {
    let policy = C2AdaptivePolicy::new(0, 1.0, 1.0, 1.0).expect("valid C2 config");
    let leaked = PolicyObservation::new(
        1,
        100,
        0.0,
        1.0,
        3.0,
        0,
        Some(VerifierSignal::Satisfied),
        1,
    )
    .expect("finite observation");

    assert!(matches!(
        policy.decide(leaked),
        Err(ReferencePolicyError::AdaptiveInference(
            AdaptiveInferenceError::VerifierSignalForbidden {
                arm: PolicyArm::C2AdaptiveStopping
            }
        ))
    ));
}

#[test]
fn c2_adaptive_stop_has_path_invariant_charge() {
    let policy = C2AdaptivePolicy::new(1, 2.0, 1.0, 3.0).expect("valid C2 config");
    let keep_going = PolicyObservation::new(1, 100, 1.0, 5.0, 4.0, 0, None, 0)
        .expect("valid observation");
    let stop_now = PolicyObservation::new(4, 100, 1.0, 2.0, -3.0, 0, None, 0)
        .expect("valid observation");

    let continue_decision = policy.decide(keep_going).expect("C2 continue");
    let stop_decision = policy.decide(stop_now).expect("C2 stop");
    assert_eq!(continue_decision.action(), InferenceAction::Continue);
    assert_eq!(stop_decision.action(), InferenceAction::Stop);
    assert_eq!(continue_decision.charge(), stop_decision.charge());
    assert_eq!(continue_decision.charge().operations(), 10);
    assert_eq!(continue_decision.charge().memory_bits(), 256);
}

#[test]
fn c3_verifier_state_machine_has_path_invariant_charge() {
    let policy = C3RecoveryPolicy::new(0, 0.0, 0.0, f64::MAX, 2, 2, true)
        .expect("valid C3 config");
    let no_signal_before_cadence =
        PolicyObservation::new(1, 100, 1.0, 5.0, 1.0, 0, None, 1)
            .expect("valid C3 observation");
    let no_signal_on_cadence = PolicyObservation::new(2, 100, 1.0, 4.0, 1.0, 0, None, 1)
        .expect("valid C3 observation");
    let indeterminate = PolicyObservation::new(
        2,
        100,
        1.0,
        4.0,
        1.0,
        0,
        Some(VerifierSignal::Indeterminate),
        1,
    )
    .expect("valid C3 observation");
    let satisfied = PolicyObservation::new(
        2,
        100,
        1.0,
        4.0,
        1.0,
        0,
        Some(VerifierSignal::Satisfied),
        1,
    )
    .expect("valid C3 observation");
    let violated = PolicyObservation::new(
        2,
        100,
        1.0,
        4.0,
        1.0,
        0,
        Some(VerifierSignal::Violated),
        1,
    )
    .expect("valid C3 observation");

    let decisions = [
        policy
            .decide(no_signal_before_cadence)
            .expect("continue decision"),
        policy
            .decide(no_signal_on_cadence)
            .expect("verify decision"),
        policy.decide(indeterminate).expect("continue decision"),
        policy.decide(satisfied).expect("stop decision"),
        policy.decide(violated).expect("backtrack decision"),
    ];
    assert_eq!(decisions[0].action(), InferenceAction::Continue);
    assert_eq!(decisions[1].action(), InferenceAction::Verify);
    assert_eq!(decisions[2].action(), InferenceAction::Continue);
    assert_eq!(decisions[3].action(), InferenceAction::Stop);
    assert_eq!(decisions[4].action(), InferenceAction::Backtrack);
    for decision in decisions {
        assert_eq!(decision.arm(), PolicyArm::C3VerificationRecovery);
        assert_eq!(decision.charge().operations(), 17);
        assert_eq!(decision.charge().memory_bits(), 385);
    }
}

#[test]
fn c3_unrecoverable_terminal_violation_fails_closed() {
    let policy = C3RecoveryPolicy::new(0, 0.0, 0.0, f64::MAX, 1, 1, true)
        .expect("valid C3 config");
    let observation = PolicyObservation::new(
        4,
        100,
        0.0,
        0.0,
        2.0,
        0,
        Some(VerifierSignal::Violated),
        0,
    )
    .expect("valid C3 observation");

    assert_eq!(
        policy.decide(observation),
        Err(ReferencePolicyError::UnrecoverableVerifiedViolation)
    );
}

#[test]
fn invalid_policy_configuration_fails_closed() {
    assert!(C0FixedPolicy::new(0).is_err());
    assert!(C1StaticPolicy::new(1, 0, 1).is_err());
    assert!(C2AdaptivePolicy::new(0, f64::NAN, 1.0, 1.0).is_err());
    assert!(C2AdaptivePolicy::new(0, 1.0, -1.0, 1.0).is_err());
    assert!(C3RecoveryPolicy::new(0, 0.0, 0.0, 1.0, 0, 0, true).is_err());
}

#[test]
fn p3_c2_fails_while_c3_reference_policy_recovers_with_paid_decisions() {
    let generated = generate_p3(
        P3Config::new(4, 3).expect("valid P3 config"),
        DifficultyStratum::Deep,
        0xabc0_1234,
    )
    .expect("P3 generation");
    let (policy_task, evaluator) = generated.into_parts();

    let c2_policy = C2AdaptivePolicy::new(0, 0.0, 0.0, f64::MAX).expect("valid C2 config");
    let mut c2 = ReferenceExecution::new(
        PolicyArm::C2AdaptiveStopping,
        policy_task.clone(),
        roomy_envelope(),
    )
    .expect("C2 execution");
    let c2_stopped = loop {
        let observation = c2.observation().expect("C2 observation");
        let decision = c2_policy.decide(observation).expect("C2 decision");
        if let Some(stopped) = apply_decision(&mut c2, decision) {
            break stopped;
        }
    };
    assert!(!evaluate_stopped(c2_stopped, evaluator).expect("C2 evaluation"));
    assert!(c2_stopped.accounting().usage().policy_ops() > 0);
    assert_eq!(c2_stopped.accounting().usage().verifier_ops(), 0);
    assert_eq!(c2_stopped.accounting().usage().checkpoint_ops(), 0);

    let c3_policy = C3RecoveryPolicy::new(0, 0.0, 0.0, f64::MAX, 1, 1, true)
        .expect("valid C3 config");
    let mut c3 = ReferenceExecution::new(
        PolicyArm::C3VerificationRecovery,
        policy_task,
        roomy_envelope(),
    )
    .expect("C3 execution");
    let mut decision_count = 0u32;
    let c3_stopped = loop {
        assert!(decision_count < 128, "C3 policy did not terminate");
        decision_count += 1;
        let observation = c3.observation().expect("C3 observation");
        let decision = c3_policy.decide(observation).expect("C3 decision");
        if let Some(stopped) = apply_decision(&mut c3, decision) {
            break stopped;
        }
    };

    assert!(evaluate_stopped(c3_stopped, evaluator).expect("C3 evaluation"));
    let usage = c3_stopped.accounting().usage();
    assert!(usage.policy_ops() > 0);
    assert!(usage.verifier_ops() > 0);
    assert!(usage.checkpoint_ops() > 0);
    assert!(usage.replay_ops() > 0);
}
