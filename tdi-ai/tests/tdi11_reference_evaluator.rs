mod hallucination_world {
    pub use tdi_ai::hallucination_world::*;
}

#[path = "../src/hallucination_generator.rs"]
mod hallucination_generator;
#[path = "../src/hallucination_evaluator.rs"]
mod hallucination_evaluator;

use hallucination_evaluator::{
    ActionCounts, ReferenceProvenanceContext, ReferenceResourceEnvelope, ReferenceResourceUsage,
    aggregate_counts, evaluate_response,
};
use hallucination_generator::{GeneratedTask, SeedDomain, generate_task};
use hallucination_world::{
    Fact, HallucinationRejection, PRIMARY_CELLS, PrimaryTaskFamily, StructuredResponse, SupportLabel,
};

fn cell(family: PrimaryTaskFamily) -> hallucination_world::PrimaryCell {
    PRIMARY_CELLS
        .into_iter()
        .find(|cell| cell.family == family)
        .expect("family exists in primary grid")
}

fn task(family: PrimaryTaskFamily, seed: u64) -> GeneratedTask {
    generate_task(SeedDomain::Development, seed, cell(family)).expect("valid generated task")
}

fn provenance() -> ReferenceProvenanceContext {
    ReferenceProvenanceContext::new(
        "development-head",
        "bbf02543f5661a7f5c029b9e4673e697e19a350a",
        "tdi11-controlled-prompt-v1",
    )
    .with_policy("B0-direct", "no-adaptive-controller")
    .with_verifier("none", "none")
}

fn supported_assertion(task: &GeneratedTask) -> String {
    for object in task.world().policy_view().entities() {
        let fact = Fact::new(
            task.query().subject().clone(),
            task.query().relation().clone(),
            object.clone(),
        );
        let score = task
            .world()
            .score_response(&StructuredResponse::Assert(fact.clone()))
            .expect("known identifiers");
        if matches!(
            score.label(),
            SupportLabel::SupportedExplicit | SupportLabel::SupportedDerived
        ) {
            return format!(
                "ASSERT {} {} {}",
                fact.subject(),
                fact.relation(),
                fact.object()
            );
        }
    }
    panic!("task has no supported answer")
}

fn hidden_assertion(task: &GeneratedTask) -> (String, Fact) {
    for object in task.world().policy_view().entities() {
        let fact = Fact::new(
            task.query().subject().clone(),
            task.query().relation().clone(),
            object.clone(),
        );
        let score = task
            .world()
            .score_response(&StructuredResponse::Assert(fact.clone()))
            .expect("known identifiers");
        if score.label() == SupportLabel::TrueHiddenUnsupported {
            return (
                format!(
                    "ASSERT {} {} {}",
                    fact.subject(),
                    fact.relation(),
                    fact.object()
                ),
                fact,
            );
        }
    }
    panic!("task has no hidden true assertion")
}

#[test]
fn f1_supported_emit_is_answered_success_without_unsupported_flag() {
    let task = task(PrimaryTaskFamily::F1ExplicitSupport, 1);
    let response = supported_assertion(&task);
    let record = evaluate_response(
        &task,
        &response,
        ActionCounts::single_emit(),
        ReferenceResourceUsage::default().with_tokens(12, 4).with_decode_steps(4),
        ReferenceResourceEnvelope::unlimited_for_development(),
        &provenance(),
    );

    assert!(record.is_valid());
    assert_eq!(record.label(), Some(SupportLabel::SupportedExplicit));
    assert!(record.answered());
    assert!(record.task_success());
    assert!(!record.unsupported_emit());
    assert!(!record.false_abstention());
}

#[test]
fn f3_abstention_is_success_but_not_an_answer() {
    let task = task(PrimaryTaskFamily::F3HiddenTruthInsufficiency, 2);
    let record = evaluate_response(
        &task,
        "ABSTAIN",
        ActionCounts::single_abstain(),
        ReferenceResourceUsage::default(),
        ReferenceResourceEnvelope::unlimited_for_development(),
        &provenance(),
    );

    assert!(record.is_valid());
    assert_eq!(record.label(), Some(SupportLabel::Abstained));
    assert!(!record.answered());
    assert!(record.task_success());
    assert!(!record.unsupported_emit());
}

#[test]
fn f1_abstention_is_counted_as_false_abstention() {
    let task = task(PrimaryTaskFamily::F1ExplicitSupport, 3);
    let record = evaluate_response(
        &task,
        "ABSTAIN",
        ActionCounts::single_abstain(),
        ReferenceResourceUsage::default(),
        ReferenceResourceEnvelope::unlimited_for_development(),
        &provenance(),
    );

    assert!(record.is_valid());
    assert!(record.false_abstention());
    assert!(!record.task_success());
}

#[test]
fn hidden_true_emission_remains_unsupported_and_hidden_payload_is_not_in_provenance() {
    let task = task(PrimaryTaskFamily::F3HiddenTruthInsufficiency, 4);
    let (response, hidden_fact) = hidden_assertion(&task);
    let record = evaluate_response(
        &task,
        &response,
        ActionCounts::single_emit(),
        ReferenceResourceUsage::default(),
        ReferenceResourceEnvelope::unlimited_for_development(),
        &provenance(),
    );

    assert!(record.is_valid());
    assert_eq!(record.label(), Some(SupportLabel::TrueHiddenUnsupported));
    assert!(record.unsupported_emit());
    assert!(record.answered());
    assert!(!record.task_success());
    assert!(!record.provenance_record().contains(&hidden_fact.canonical_record()));
}

#[test]
fn malformed_output_is_preserved_as_a_rejected_record() {
    let task = task(PrimaryTaskFamily::F1ExplicitSupport, 5);
    let record = evaluate_response(
        &task,
        "the answer is probably e001",
        ActionCounts::single_emit(),
        ReferenceResourceUsage::default(),
        ReferenceResourceEnvelope::unlimited_for_development(),
        &provenance(),
    );

    assert!(!record.is_valid());
    assert_eq!(record.label(), None);
    assert_eq!(record.rejection(), Some(&HallucinationRejection::MalformedOutput));
    assert!(!record.task_success());
    assert!(record.provenance_record().contains("rejection=MALFORMED_OUTPUT"));
}

#[test]
fn resource_envelope_excess_fails_closed_before_scoring() {
    let task = task(PrimaryTaskFamily::F1ExplicitSupport, 6);
    let response = supported_assertion(&task);
    let mut envelope = ReferenceResourceEnvelope::unlimited_for_development();
    envelope.max_decode_steps = 3;
    let record = evaluate_response(
        &task,
        &response,
        ActionCounts::single_emit(),
        ReferenceResourceUsage::default().with_decode_steps(4),
        envelope,
        &provenance(),
    );

    assert_eq!(
        record.rejection(),
        Some(&HallucinationRejection::ResourceEnvelopeExceeded)
    );
    assert_eq!(record.label(), None);
    assert!(!record.task_success());
}

#[test]
fn action_resource_mismatch_is_contract_drift() {
    let task = task(PrimaryTaskFamily::F1ExplicitSupport, 7);
    let response = supported_assertion(&task);
    let record = evaluate_response(
        &task,
        &response,
        ActionCounts::single_emit().with_verify(1),
        ReferenceResourceUsage::default(),
        ReferenceResourceEnvelope::unlimited_for_development(),
        &provenance(),
    );

    assert_eq!(record.rejection(), Some(&HallucinationRejection::ContractDrift));
}

#[test]
fn action_counter_overflow_is_a_typed_rejection() {
    let task = task(PrimaryTaskFamily::F1ExplicitSupport, 8);
    let response = supported_assertion(&task);
    let actions = ActionCounts::single_emit()
        .with_continue(u64::MAX)
        .with_verify(1);
    let usage = ReferenceResourceUsage::default().with_verifier(1, 0);
    let record = evaluate_response(
        &task,
        &response,
        actions,
        usage,
        ReferenceResourceEnvelope::unlimited_for_development(),
        &provenance(),
    );

    assert_eq!(
        record.rejection(),
        Some(&HallucinationRejection::ResourceAccountingOverflow)
    );
}

#[test]
fn incomplete_provenance_fails_closed() {
    let task = task(PrimaryTaskFamily::F1ExplicitSupport, 9);
    let response = supported_assertion(&task);
    let bad = ReferenceProvenanceContext::new("", "blob", "prompt");
    let record = evaluate_response(
        &task,
        &response,
        ActionCounts::single_emit(),
        ReferenceResourceUsage::default(),
        ReferenceResourceEnvelope::unlimited_for_development(),
        &bad,
    );

    assert_eq!(
        record.rejection(),
        Some(&HallucinationRejection::ProvenanceFailure)
    );
}

#[test]
fn deterministic_replay_produces_identical_record_hash() {
    let task = task(PrimaryTaskFamily::F2DerivedSupport, 10);
    let response = supported_assertion(&task);
    let evaluate = || {
        evaluate_response(
            &task,
            &response,
            ActionCounts::single_emit().with_continue(2),
            ReferenceResourceUsage::default().with_decode_steps(3),
            ReferenceResourceEnvelope::unlimited_for_development(),
            &provenance(),
        )
    };
    let first = evaluate();
    let second = evaluate();

    assert_eq!(first.provenance_record(), second.provenance_record());
    assert_eq!(first.result_record_hash(), second.result_record_hash());
    assert_eq!(first.result_record_hash().len(), 16);
}

#[test]
fn aggregate_counts_keep_rejections_separate_from_valid_risk_denominators() {
    let f1 = task(PrimaryTaskFamily::F1ExplicitSupport, 11);
    let f1_ok = evaluate_response(
        &f1,
        &supported_assertion(&f1),
        ActionCounts::single_emit(),
        ReferenceResourceUsage::default(),
        ReferenceResourceEnvelope::unlimited_for_development(),
        &provenance(),
    );

    let f3 = task(PrimaryTaskFamily::F3HiddenTruthInsufficiency, 12);
    let f3_abstain = evaluate_response(
        &f3,
        "ABSTAIN",
        ActionCounts::single_abstain(),
        ReferenceResourceUsage::default(),
        ReferenceResourceEnvelope::unlimited_for_development(),
        &provenance(),
    );
    let (hidden_response, _) = hidden_assertion(&f3);
    let f3_bad = evaluate_response(
        &f3,
        &hidden_response,
        ActionCounts::single_emit(),
        ReferenceResourceUsage::default(),
        ReferenceResourceEnvelope::unlimited_for_development(),
        &provenance(),
    );

    let rejected = evaluate_response(
        &f1,
        "not structured",
        ActionCounts::single_emit(),
        ReferenceResourceUsage::default(),
        ReferenceResourceEnvelope::unlimited_for_development(),
        &provenance(),
    );

    let summary = aggregate_counts(&[f1_ok, f3_abstain, f3_bad, rejected])
        .expect("small fixture cannot overflow");
    assert_eq!(summary.total_records, 4);
    assert_eq!(summary.valid_records, 3);
    assert_eq!(summary.rejected_records, 1);
    assert_eq!(summary.unsupported_emits, 1);
    assert_eq!(summary.answered, 2);
    assert_eq!(summary.unsupported_answered, 1);
    assert_eq!(summary.task_successes, 2);
}
