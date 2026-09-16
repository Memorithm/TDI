#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi2_intuition::{
    BooleanState, PredicateId, RoleId, Template, TemplateId,
};
use tdi_ai::experimental::tdi2_intuition_engine::IntuitionEngine;
use tdi_ai::experimental::tdi2_intuition_inference::IntuitionOutcome;
use tdi_ai::experimental::tdi2_intuition_relations::{
    RelationId, RelationalTemplate, RoleRelation,
};
use tdi_ai::experimental::tdi2_intuition_reliability::ReliabilityEvidence;
use tdi_ai::experimental::tdi2_intuition_store::{ExperienceEntry, ExperienceStore};
use tdi_ai::experimental::tdi2_intuition_transfer::{
    EntityId, RoleBinding, RoleMap, transfer_relations,
};

fn entry(id: u64, predicate: u32, successes: u64, failures: u64) -> ExperienceEntry {
    let base = Template::new(
        TemplateId::new(id),
        vec![PredicateId::new(predicate)],
        Vec::new(),
        Vec::new(),
    )
    .expect("valid template");
    ExperienceEntry::new(
        RelationalTemplate::new(base, Vec::new()).expect("valid relational template"),
        ReliabilityEvidence::new(successes, failures),
    )
}

#[test]
fn accumulated_relevant_experience_changes_selected_template() {
    let mut store = ExperienceStore::new(2).expect("valid store");
    store.insert(entry(10, 7, 2, 0)).expect("insert");
    store.insert(entry(20, 7, 20, 0)).expect("insert");

    let state = BooleanState::new(vec![PredicateId::new(7)]);
    let report = IntuitionEngine::default()
        .run(&store, &state)
        .expect("valid evidence");

    assert!(matches!(
        report.outcome(),
        IntuitionOutcome::Selected { template_id, .. } if template_id == TemplateId::new(20)
    ));
    assert_eq!(report.candidates().len(), 2);
}

#[test]
fn uncovered_situation_abstains_instead_of_guessing() {
    let mut store = ExperienceStore::new(1).expect("valid store");
    store.insert(entry(10, 7, 20, 0)).expect("insert");

    let state = BooleanState::new(vec![PredicateId::new(999)]);
    let report = IntuitionEngine::default()
        .run(&store, &state)
        .expect("valid evidence");

    assert_eq!(report.outcome(), IntuitionOutcome::InsufficientExperience);
    assert!(report.candidates().is_empty());
}

#[test]
fn structural_template_transfers_to_novel_entities_without_relation_drift() {
    let base = Template::new(
        TemplateId::new(30),
        vec![PredicateId::new(1)],
        Vec::new(),
        vec![RoleId::new(1), RoleId::new(2)],
    )
    .expect("valid template");
    let template = RelationalTemplate::new(
        base,
        vec![RoleRelation::new(
            RoleId::new(1),
            RelationId::new(77),
            RoleId::new(2),
        )],
    )
    .expect("valid relational template");
    let mapping = RoleMap::for_template(
        &template,
        vec![
            RoleBinding::new(RoleId::new(1), EntityId::new(400)),
            RoleBinding::new(RoleId::new(2), EntityId::new(900)),
        ],
    )
    .expect("valid novel-role mapping");

    let transferred = transfer_relations(&template, &mapping);
    assert_eq!(transferred.len(), 1);
    assert_eq!(transferred[0].left(), EntityId::new(400));
    assert_eq!(transferred[0].relation(), RelationId::new(77));
    assert_eq!(transferred[0].right(), EntityId::new(900));
}
