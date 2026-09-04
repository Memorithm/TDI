#[allow(dead_code, clippy::double_must_use)]
#[path = "../src/adaptive_inference.rs"]
mod adaptive_inference;

use adaptive_inference::{
    ComputeComponent, InferenceAction, PolicyArm, PolicyObservation, ResourceEnvelope,
    ResourceMeter, VerifierSignal, validate_action,
};

#[test]
fn downstream_fixture_exercises_the_frozen_tdi9_policy_ladder() {
    assert!(validate_action(PolicyArm::C0FixedCompute, InferenceAction::Continue).is_ok());
    assert!(validate_action(PolicyArm::C2AdaptiveStopping, InferenceAction::Stop).is_ok());
    assert!(validate_action(PolicyArm::C2AdaptiveStopping, InferenceAction::Verify).is_err());
    assert!(validate_action(PolicyArm::C3VerificationRecovery, InferenceAction::Verify).is_ok());
    assert!(
        validate_action(
            PolicyArm::C3VerificationRecovery,
            InferenceAction::Backtrack
        )
        .is_ok()
    );
    assert!(PolicyArm::C2AdaptiveStopping.is_trajectory_adaptive());
    assert!(!PolicyArm::C1StaticPreallocation.is_trajectory_adaptive());
}

#[test]
fn observation_and_accounting_contracts_are_bounded_and_c3_scoped() {
    let observation = PolicyObservation::new(
        4,
        90,
        0.125,
        0.25,
        -0.0625,
        5,
        Some(VerifierSignal::Indeterminate),
        1,
    )
    .expect("finite synthetic observation")
    .validate_for_arm(PolicyArm::C3VerificationRecovery)
    .expect("C3 may consume verifier/checkpoint metadata");

    assert_eq!(observation.step_index(), 4);
    assert_eq!(observation.remaining_compute_ops(), 90);
    assert_eq!(observation.state_delta(), 0.125);
    assert_eq!(observation.residual(), 0.25);
    assert_eq!(observation.score_margin(), -0.0625);
    assert_eq!(observation.prior_action_count(), 5);
    assert_eq!(
        observation.verifier_signal(),
        Some(VerifierSignal::Indeterminate)
    );
    assert_eq!(observation.available_checkpoints(), 1);

    let envelope = ResourceEnvelope::new(100, 128).expect("bounded synthetic envelope");
    assert_eq!(envelope.max_compute_ops(), 100);
    assert_eq!(envelope.max_memory_bits(), 128);

    let mut meter = ResourceMeter::new(envelope);
    meter
        .charge_compute(ComputeComponent::Solver, 40)
        .expect("solver charge");
    meter
        .charge_compute(ComputeComponent::PolicyDecision, 5)
        .expect("policy charge");
    meter
        .charge_compute(ComputeComponent::Verifier, 10)
        .expect("verifier charge");
    meter
        .charge_compute(ComputeComponent::Checkpoint, 5)
        .expect("checkpoint charge");
    meter
        .charge_compute(ComputeComponent::Replay, 15)
        .expect("replay charge");
    meter
        .set_memory(48, 16, 32, 24)
        .expect("within shared memory envelope");

    assert_eq!(meter.envelope(), envelope);
    assert_eq!(meter.usage().total_compute_ops().expect("exact sum"), 75);
    assert_eq!(meter.usage().total_memory_bits(), 120);
}

#[test]
fn verifier_metadata_cannot_leak_into_c2() {
    let observation =
        PolicyObservation::new(1, 8, 0.0, 0.0, 0.0, 1, Some(VerifierSignal::Satisfied), 0)
            .expect("finite synthetic observation");

    assert!(
        observation
            .validate_for_arm(PolicyArm::C2AdaptiveStopping)
            .is_err()
    );
}
