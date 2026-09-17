//! Development review of a tempting but unsafe generalization.
use tdi_ai::experimental::tdi2_intuition::{BooleanState, PredicateId, Template, TemplateId};
use tdi_ai::experimental::tdi2_intuition_consolidation::{
    ClausePolarity, GeneralizationProposal, evaluate_generalization_proposal,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let template = Template::new(
        TemplateId::new(900_001),
        vec![PredicateId::new(1), PredicateId::new(2)],
        Vec::new(),
        Vec::new(),
    )?;
    let proposal = GeneralizationProposal {
        template_id: template.id(),
        predicate: PredicateId::new(2),
        polarity: ClausePolarity::Required,
    };
    let positives = [
        BooleanState::new(vec![PredicateId::new(1), PredicateId::new(2)]),
        BooleanState::new(vec![
            PredicateId::new(1),
            PredicateId::new(2),
            PredicateId::new(99),
        ]),
        BooleanState::new(vec![PredicateId::new(1)]),
    ];
    let negatives = [BooleanState::new(vec![
        PredicateId::new(1),
        PredicateId::new(8),
    ])];
    let evaluation = evaluate_generalization_proposal(&template, proposal, &positives, &negatives)?;
    assert!(evaluation.preserves_all_positives());
    assert_eq!(evaluation.negative_admitted, 1);
    assert!(!evaluation.passes_supplied_evidence());
    println!(
        "tdi2.1-generalization-review-v1;positive_total={};positive_admitted={};negative_total={};negative_admitted={};accepted={}",
        evaluation.positive_total,
        evaluation.positive_admitted,
        evaluation.negative_total,
        evaluation.negative_admitted,
        evaluation.passes_supplied_evidence()
    );
    Ok(())
}
