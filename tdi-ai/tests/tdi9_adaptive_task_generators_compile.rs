#[allow(dead_code)]
#[path = "../src/adaptive_task_generators.rs"]
mod adaptive_task_generators;

use adaptive_task_generators::{
    AdaptiveTaskFamily, DifficultyStratum, EvaluatorOracle, EvaluatorTarget, ForkEvent, P1Config,
    P2Config, P3Config, PolicyTask, generate_p1, generate_p2, generate_p3,
};

#[test]
fn p1_preserves_the_requested_irreversible_step_across_many_non_final_seeds() {
    for evidence_count in (3u32..=21).step_by(2) {
        let minimum = evidence_count.div_ceil(2);
        for decisive_step in minimum..=evidence_count {
            let config = P1Config::new(evidence_count, decisive_step).expect("valid P1 config");
            for seed in 0u64..64 {
                let generated = generate_p1(config, DifficultyStratum::Intermediate, seed)
                    .expect("deterministic P1 generation");
                assert_eq!(
                    generated.evaluator().oracle(),
                    EvaluatorOracle::P1 {
                        earliest_decisive_step: decisive_step,
                    }
                );
                assert_eq!(generated.policy().event_count(), evidence_count as usize);
            }
        }
    }
}

#[test]
fn p2_policy_constraints_have_exactly_one_solution_without_storing_target() {
    let generated = generate_p2(
        P2Config::new(7).expect("valid P2 config"),
        DifficultyStratum::Shallow,
        0x1234_5678,
    )
    .expect("deterministic P2 generation");

    assert_eq!(
        generated.policy().family(),
        AdaptiveTaskFamily::VerificationSensitiveInference
    );
    let (target, width) = match generated.evaluator().target() {
        EvaluatorTarget::P2 { bits, width } => (bits, width),
        other => panic!("unexpected target: {other:?}"),
    };
    let constraints = match generated.policy() {
        PolicyTask::P2(task) => task.constraints(),
        other => panic!("unexpected policy task: {other:?}"),
    };

    let solutions = (0u64..(1u64 << width))
        .filter(|candidate| {
            constraints.iter().all(|constraint| {
                ((candidate & constraint.mask()).count_ones() % 2 == 1) == constraint.parity()
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(solutions, vec![target]);
}

#[test]
fn p3_exposes_ordinary_contradiction_but_keeps_hidden_labels_in_evaluator_record() {
    let generated = generate_p3(
        P3Config::new(5, 4).expect("valid P3 config"),
        DifficultyStratum::Deep,
        0xfeed_beef,
    )
    .expect("deterministic P3 generation");

    let (target, contradiction_index, decoy) = match (
        generated.evaluator().target(),
        generated.evaluator().oracle(),
    ) {
        (
            EvaluatorTarget::P3(target),
            EvaluatorOracle::P3 {
                contradiction_event_index,
                decoy_branch,
            },
        ) => (target, contradiction_event_index, decoy_branch),
        other => panic!("unexpected evaluator record: {other:?}"),
    };
    assert_eq!(target.opposite(), decoy);

    let events = match generated.policy() {
        PolicyTask::P3(task) => task.events(),
        other => panic!("unexpected policy task: {other:?}"),
    };
    assert_eq!(events.first(), Some(&ForkEvent::ChoicePoint));
    assert_eq!(
        events[usize::try_from(contradiction_index).expect("bounded index")],
        ForkEvent::EliminateBranch { branch: decoy }
    );
}

#[test]
fn policy_and_evaluator_surfaces_remain_structurally_separate() {
    let generated = generate_p1(
        P1Config::new(9, 7).expect("valid P1 config"),
        DifficultyStratum::Deep,
        77,
    )
    .expect("deterministic P1 generation");
    let (policy, evaluator) = generated.into_parts();

    assert_eq!(
        policy.family(),
        AdaptiveTaskFamily::StagedEvidenceAccumulation
    );
    assert_eq!(evaluator.stratum(), DifficultyStratum::Deep);
    assert_eq!(evaluator.seed(), 77);
    assert!(matches!(evaluator.target(), EvaluatorTarget::P1(_)));
}
