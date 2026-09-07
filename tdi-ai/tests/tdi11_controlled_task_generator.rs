mod hallucination_world {
    pub use tdi_ai::hallucination_world::*;
}

#[path = "../src/hallucination_generator.rs"]
mod hallucination_generator;

use hallucination_generator::{SeedDomain, generate_task, supported_answer_count};
use hallucination_world::{
    Fact, PRIMARY_CELLS, PrimaryTaskFamily, StructuredResponse, SupportLabel,
};

fn query_labels(task: &hallucination_generator::GeneratedTask) -> Vec<(Fact, SupportLabel)> {
    task.world()
        .policy_view()
        .entities()
        .iter()
        .map(|object| {
            let fact = Fact::new(
                task.query().subject().clone(),
                task.query().relation().clone(),
                object.clone(),
            );
            let response = StructuredResponse::Assert(fact.clone());
            let label = task
                .world()
                .score_response(&response)
                .expect("generator emits only known identifiers")
                .label();
            (fact, label)
        })
        .collect()
}

#[test]
fn all_twelve_cells_generate_in_both_non_final_domains() {
    for domain in [SeedDomain::Development, SeedDomain::Validation] {
        for cell in PRIMARY_CELLS {
            let task = generate_task(domain, 0x1234_5678_9abc_def0, cell)
                .expect("all frozen primary cells must generate");
            assert_eq!(task.cell(), cell);
            assert_eq!(task.domain(), domain);
            assert_eq!(task.seed(), 0x1234_5678_9abc_def0);
            assert!(!task.model_prompt().contains("family="));
            assert!(!task.model_prompt().contains("stratum="));
            assert!(!task.model_prompt().contains("seed="));
        }
    }
}

#[test]
fn generation_is_reproducible_within_a_domain() {
    for cell in PRIMARY_CELLS {
        let first = generate_task(SeedDomain::Development, 17, cell).expect("first task");
        let second = generate_task(SeedDomain::Development, 17, cell).expect("second task");
        assert_eq!(first.model_prompt(), second.model_prompt());
        assert_eq!(first.provenance_record(), second.provenance_record());
    }
}

#[test]
fn development_and_validation_namespaces_are_domain_separated() {
    for cell in PRIMARY_CELLS {
        let development = generate_task(SeedDomain::Development, 9, cell).expect("development");
        let validation = generate_task(SeedDomain::Validation, 9, cell).expect("validation");
        assert_ne!(development.provenance_record(), validation.provenance_record());
        assert!(development.provenance_record().contains("domain=development"));
        assert!(validation.provenance_record().contains("domain=validation"));
    }
}

#[test]
fn answerable_families_have_one_supported_answer_and_f3_has_none() {
    for cell in PRIMARY_CELLS {
        let task = generate_task(SeedDomain::Development, 41, cell).expect("task");
        let expected = match cell.family {
            PrimaryTaskFamily::F1ExplicitSupport
            | PrimaryTaskFamily::F2DerivedSupport
            | PrimaryTaskFamily::F4ContradictionOverride => 1,
            PrimaryTaskFamily::F3HiddenTruthInsufficiency => 0,
        };
        assert_eq!(supported_answer_count(&task), expected, "cell={cell:?}");
    }
}

#[test]
fn f2_supported_answer_is_derived_not_explicit() {
    for cell in PRIMARY_CELLS
        .into_iter()
        .filter(|cell| cell.family == PrimaryTaskFamily::F2DerivedSupport)
    {
        let task = generate_task(SeedDomain::Validation, 73, cell).expect("F2 task");
        let labels = query_labels(&task);
        assert_eq!(
            labels
                .iter()
                .filter(|(_, label)| *label == SupportLabel::SupportedDerived)
                .count(),
            1
        );
        assert_eq!(
            labels
                .iter()
                .filter(|(_, label)| *label == SupportLabel::SupportedExplicit)
                .count(),
            0
        );
    }
}

#[test]
fn f3_contains_one_hidden_true_answer_but_prompt_does_not_expose_the_fact() {
    for cell in PRIMARY_CELLS
        .into_iter()
        .filter(|cell| cell.family == PrimaryTaskFamily::F3HiddenTruthInsufficiency)
    {
        let task = generate_task(SeedDomain::Development, 101, cell).expect("F3 task");
        let labels = query_labels(&task);
        let hidden = labels
            .iter()
            .filter(|(_, label)| *label == SupportLabel::TrueHiddenUnsupported)
            .collect::<Vec<_>>();
        assert_eq!(hidden.len(), 1);
        assert!(!task.model_prompt().contains(&hidden[0].0.canonical_record()));
    }
}

#[test]
fn f4_exposes_one_current_answer_and_rejects_the_stale_override() {
    for cell in PRIMARY_CELLS
        .into_iter()
        .filter(|cell| cell.family == PrimaryTaskFamily::F4ContradictionOverride)
    {
        let task = generate_task(SeedDomain::Validation, 211, cell).expect("F4 task");
        let labels = query_labels(&task);
        assert_eq!(
            labels
                .iter()
                .filter(|(_, label)| *label == SupportLabel::SupportedExplicit)
                .count(),
            1
        );
        assert_eq!(
            labels
                .iter()
                .filter(|(_, label)| *label == SupportLabel::Contradicted)
                .count(),
            1
        );
    }
}
