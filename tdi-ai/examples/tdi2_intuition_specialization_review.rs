//! Development review of a specialization driven by a refuted state.
use tdi_ai::experimental::tdi2_intuition::{BooleanState, PredicateId, Template, TemplateId};
use tdi_ai::experimental::tdi2_intuition_consolidation::{
    ClausePolarity, SpecializationProposal, evaluate_specialization_proposal,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let template = Template::new(
        TemplateId::new(900_002),
        vec![PredicateId::new(1)],
        Vec::new(),
        Vec::new(),
    )?;
    let proposal = SpecializationProposal {
        template_id: template.id(),
        predicate: PredicateId::new(8),
        polarity: ClausePolarity::Forbidden,
    };
    let positives = [
        BooleanState::new(vec![PredicateId::new(1)]),
        BooleanState::new(vec![PredicateId::new(1), PredicateId::new(9)]),
    ];
    let negatives = [BooleanState::new(vec![
        PredicateId::new(1),
        PredicateId::new(8),
    ])];
    let evaluation = evaluate_specialization_proposal(&template, proposal, &positives, &negatives)?;
    assert!(evaluation.passes_supplied_evidence());
    println!(
        "tdi2.1-specialization-review-v1;positive_total={};positive_admitted={};negative_total={};negative_admitted={};accepted={}",
        evaluation.positive_total,
        evaluation.positive_admitted,
        evaluation.negative_total,
        evaluation.negative_admitted,
        evaluation.passes_supplied_evidence()
    );
    Ok(())
}
