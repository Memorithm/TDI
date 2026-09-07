#![cfg(feature = "experimental")]
use tdi_ai::experimental::{
    adaptive_evaluator::*, adaptive_inference::*, adaptive_policies::*, adaptive_rejections::*,
    adaptive_task_generators::*, development_campaign::*,
};
use tdi_ai::provenance::*;
fn manifest() -> ExperimentManifest {
    ExperimentManifest::new(
        [
            ("source_commit", "0000000000000000000000000000000000000000"),
            ("dependency_identity", "fixture-lock"),
            ("backend_identity", "deterministic-reference"),
            ("configuration", "P1(5,3)/Shallow"),
            ("generator_identity", "p1-fixture-v1"),
            ("metric_identity", "exact-target"),
            ("accounting_identity", "logical-reference-v1"),
        ]
        .map(|(k, v)| (k.to_string(), v.to_string())),
    )
    .unwrap()
}
fn task(seed: u64) -> Result<GeneratedTask, AdaptiveTaskError> {
    generate_p1(P1Config::new(5, 3)?, DifficultyStratum::Shallow, seed)
}
fn plan() -> CampaignPlan {
    CampaignPlan::new(
        manifest(),
        DevelopmentDomain::Development,
        vec![0, 1, 2],
        vec![
            PolicySpec {
                id: "fixed".into(),
                policy: C0FixedPolicy::new(5).unwrap().into(),
            },
            PolicySpec {
                id: "short".into(),
                policy: C0FixedPolicy::new(2).unwrap().into(),
            },
        ],
        ResourceEnvelope::new(10000, 10000).unwrap(),
        20,
        6,
    )
    .unwrap()
}
#[test]
fn resume_matches_uninterrupted_trial_order_and_outcomes() {
    let plan = plan();
    let mut full = plan.start().unwrap();
    plan.run(&mut full, task, || false).unwrap();
    let mut partial = plan.start().unwrap();
    let mut calls = 0;
    plan.run(&mut partial, task, || {
        calls += 1;
        calls > 1
    })
    .unwrap();
    assert_eq!(partial.trials().len(), 1);
    assert!(!partial.is_complete());
    plan.run(&mut partial, task, || false).unwrap();
    assert_eq!(partial, full);
    assert!(partial.is_complete());
}
#[test]
fn failed_generation_and_wrong_seed_are_retained() {
    let plan = plan();
    let mut report = plan.start::<&str>().unwrap();
    plan.run(
        &mut report,
        |seed| {
            if seed == 0 {
                Err("generator failure")
            } else {
                Ok(task(seed + 1).unwrap())
            }
        },
        || false,
    )
    .unwrap();
    assert!(report.is_complete());
    assert_eq!(
        report.trials()[0].outcome,
        TrialOutcome::GeneratorRejected("generator failure")
    );
    assert_eq!(
        report.trials()[1].outcome,
        TrialOutcome::SeedMismatch { actual: 2 }
    );
}
#[test]
fn resume_rejects_changed_plan_without_losing_prefix() {
    let original = plan();
    let mut report = original.start::<AdaptiveTaskError>().unwrap();
    original.run(&mut report, task, || false).unwrap();
    let before = report.clone();
    let changed = CampaignPlan::new(
        manifest(),
        DevelopmentDomain::Validation,
        vec![0, 1, 2],
        vec![PolicySpec {
            id: "fixed".into(),
            policy: C0FixedPolicy::new(5).unwrap().into(),
        }],
        ResourceEnvelope::new(10000, 10000).unwrap(),
        20,
        6,
    )
    .unwrap();
    assert_eq!(
        changed.run(&mut report, task, || false),
        Err(CampaignError::PlanMismatch)
    );
    assert_eq!(before, report);
    assert_ne!(original.canonical_record(), changed.canonical_record());
    assert_eq!(DevelopmentDomain::Development.seed(1), Some(1));
    assert_eq!(DevelopmentDomain::Validation.seed(1), Some((1 << 63) + 1));
    assert!(DevelopmentDomain::Development.seed(1 << 63).is_none());
}
#[test]
fn rejection_diagnostics_preserve_compatibility_and_consumed_work() {
    let policy = ReferencePolicy::from(C0FixedPolicy::new(5).unwrap());
    let envelope = ResourceEnvelope::new(10000, 10000).unwrap();
    let expected = evaluate_generated_task(task(0).unwrap(), policy, envelope, 2).unwrap_err();
    let ReferenceEvaluationOutcome::Rejected(rejected) =
        evaluate_generated_task_recorded(task(0).unwrap(), policy, envelope, 2)
    else {
        panic!("expected rejection")
    };
    assert_eq!(rejected.error(), expected);
    let progress = rejected.progress().unwrap();
    assert_eq!(progress.step_index, 2);
    assert_eq!(progress.completed_decisions, 2);
    assert!(progress.accounting.usage().total_compute_ops().unwrap() > 0);
    let ReferenceEvaluationOutcome::Rejected(rejected) =
        evaluate_generated_task_recorded(task(0).unwrap(), policy, envelope, 0)
    else {
        panic!("expected rejection")
    };
    assert!(rejected.progress().is_none());
    let complete = evaluate_generated_task(task(0).unwrap(), policy, envelope, 20).unwrap();
    assert_eq!(
        evaluate_generated_task_recorded(task(0).unwrap(), policy, envelope, 20),
        ReferenceEvaluationOutcome::Completed(complete)
    );
}
#[test]
fn canonical_manifest_is_order_independent_and_parameters_are_exact() {
    let manifest = manifest();
    assert_eq!(
        manifest.canonical_record(),
        ExperimentManifest::new(
            manifest
                .fields()
                .iter()
                .rev()
                .map(|(k, v)| (k.clone(), v.clone()))
        )
        .unwrap()
        .canonical_record()
    );
    assert_ne!(
        C2AdaptivePolicy::new(1, 0.0, 0.0, 0.0)
            .unwrap()
            .canonical_record(),
        C2AdaptivePolicy::new(1, -0.0, 0.0, 0.0)
            .unwrap()
            .canonical_record()
    );
    assert!(
        ExperimentManifest::new(vec![("x".into(), "v".into()), ("x".into(), "w".into())]).is_err()
    );
    let mut fields = manifest.fields().clone();
    fields.insert("source_commit".into(), "main".into());
    assert_eq!(
        ExperimentManifest::new(fields),
        Err(ManifestError::InvalidCommit)
    );
}
