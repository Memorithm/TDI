//! Deterministic software fixture, not a frozen population or scientific result.
use tdi_ai::{
    experimental::{
        adaptive_inference::ResourceEnvelope,
        adaptive_policies::C0FixedPolicy,
        adaptive_task_generators::{DifficultyStratum, P1Config, generate_p1},
        development_campaign::{CampaignPlan, PolicySpec},
    },
    provenance::{DevelopmentDomain, ExperimentManifest},
};
fn main() {
    let source = std::env::args()
        .nth(1)
        .expect("supply the clean checkout's full commit SHA");
    let manifest = ExperimentManifest::new(
        [
            ("source_commit", source),
            (
                "dependency_identity",
                include_str!("../../Cargo.lock").to_owned(),
            ),
            (
                "backend_identity",
                "tdi-ai deterministic Rust reference".into(),
            ),
            (
                "configuration",
                "P1 evidence_count=5 decisive_step=3 stratum=Shallow".into(),
            ),
            (
                "generator_identity",
                "adaptive_task_generators/generate_p1 at source_commit".into(),
            ),
            ("metric_identity", "exact evaluator target equality".into()),
            (
                "accounting_identity",
                "adaptive_execution logical operations and bits at source_commit".into(),
            ),
        ]
        .map(|(k, v)| (k.to_owned(), v)),
    )
    .expect("valid manifest");
    let plan = CampaignPlan::new(
        manifest,
        DevelopmentDomain::Development,
        vec![0, 1, 2],
        vec![PolicySpec {
            id: "C0-five-steps".into(),
            policy: C0FixedPolicy::new(5).unwrap().into(),
        }],
        ResourceEnvelope::new(10_000, 10_000).unwrap(),
        20,
        3,
    )
    .unwrap();
    let mut report = plan.start().unwrap();
    plan.run(
        &mut report,
        |seed| {
            generate_p1(
                P1Config::new(5, 3).unwrap(),
                DifficultyStratum::Shallow,
                seed,
            )
        },
        || false,
    )
    .unwrap();
    println!("{}", plan.canonical_record());
    for trial in report.trials() {
        println!("{trial:?}");
    }
    assert!(report.is_complete());
}
