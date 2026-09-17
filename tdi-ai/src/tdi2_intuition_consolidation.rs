//! Explicit post-outcome consolidation for TDI-2.1 experiential evidence.
//!
//! Inference never calls this module implicitly. A real outcome must first be
//! observed and classified by the experimental protocol.

use super::tdi2_intuition::{BooleanState, PredicateId, Template, TemplateId};
use super::tdi2_intuition_reliability::{ReliabilityError, ReliabilityEvidence};

/// Empirical validation result for one applied template.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValidationOutcome {
    /// The preregistered consequence criterion was satisfied.
    Confirmed,
    /// The preregistered consequence criterion was not satisfied.
    Refuted,
}

/// Immutable record describing one evidence update.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConsolidationUpdate {
    template_id: TemplateId,
    before: ReliabilityEvidence,
    after: ReliabilityEvidence,
    outcome: ValidationOutcome,
}

impl ConsolidationUpdate {
    /// Template whose evidence changed.
    #[must_use]
    pub const fn template_id(self) -> TemplateId {
        self.template_id
    }
    /// Evidence before validation.
    #[must_use]
    pub const fn before(self) -> ReliabilityEvidence {
        self.before
    }
    /// Evidence after validation.
    #[must_use]
    pub const fn after(self) -> ReliabilityEvidence {
        self.after
    }
    /// Observed validation classification.
    #[must_use]
    pub const fn outcome(self) -> ValidationOutcome {
        self.outcome
    }

    /// Canonical post-outcome record suitable for TDI artifact binding.
    #[must_use]
    pub fn canonical_record(self) -> String {
        let outcome = match self.outcome {
            ValidationOutcome::Confirmed => "confirmed",
            ValidationOutcome::Refuted => "refuted",
        };
        format!(
            "tdi2.1-consolidation-v1;template={};before={}:{};after={}:{};outcome={outcome}",
            self.template_id.raw(),
            self.before.successes(),
            self.before.failures(),
            self.after.successes(),
            self.after.failures(),
        )
    }
}

/// Update success/failure evidence after a separately observed outcome.
pub fn consolidate_validation(
    template_id: TemplateId,
    evidence: ReliabilityEvidence,
    outcome: ValidationOutcome,
) -> Result<ConsolidationUpdate, ReliabilityError> {
    let mut after = evidence;
    after.observe(matches!(outcome, ValidationOutcome::Confirmed))?;
    Ok(ConsolidationUpdate {
        template_id,
        before: evidence,
        after,
        outcome,
    })
}

/// Structural evolution is deliberately a proposal, not an automatic mutation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StructuralReview {
    /// Repeated confirmations may justify testing removal of a condition.
    ConsiderGeneralization { template_id: TemplateId },
    /// Counterexamples may justify testing an additional discriminating condition.
    ConsiderSpecialization { template_id: TemplateId },
    /// No existing template explains the validated experience sufficiently.
    ConsiderCreation,
}

/// Polarity of one Boolean clause in a template.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClausePolarity {
    /// Predicate must be present.
    Required,
    /// Predicate must be absent.
    Forbidden,
}

/// One review-only candidate that removes exactly one Boolean condition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GeneralizationProposal {
    /// Existing template under review.
    pub template_id: TemplateId,
    /// Predicate proposed for removal.
    pub predicate: PredicateId,
    /// Clause polarity in the source template.
    pub polarity: ClausePolarity,
}

/// One review-only candidate that adds a Boolean condition excluding one counterexample.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpecializationProposal {
    /// Existing template under review.
    pub template_id: TemplateId,
    /// Predicate proposed as a discriminator.
    pub predicate: PredicateId,
    /// Proposed polarity that would exclude the supplied counterexample.
    pub polarity: ClausePolarity,
}

/// Enumerate one-literal generalizations without mutating the template.
#[must_use]
pub fn one_literal_generalizations(template: &Template) -> Vec<GeneralizationProposal> {
    let total = template.required().len() + template.forbidden().len();
    if total <= 1 {
        return Vec::new();
    }
    template
        .required()
        .iter()
        .copied()
        .map(|predicate| GeneralizationProposal {
            template_id: template.id(),
            predicate,
            polarity: ClausePolarity::Required,
        })
        .chain(
            template
                .forbidden()
                .iter()
                .copied()
                .map(|predicate| GeneralizationProposal {
                    template_id: template.id(),
                    predicate,
                    polarity: ClausePolarity::Forbidden,
                }),
        )
        .collect()
}

/// Enumerate single-predicate specializations that would exclude one refuted state.
#[must_use]
pub fn counterexample_specializations(
    template: &Template,
    counterexample: &BooleanState,
    candidate_predicates: &[PredicateId],
) -> Vec<SpecializationProposal> {
    let mut candidates = candidate_predicates.to_vec();
    candidates.sort_unstable();
    candidates.dedup();
    candidates
        .into_iter()
        .filter(|predicate| {
            template.required().binary_search(predicate).is_err()
                && template.forbidden().binary_search(predicate).is_err()
        })
        .map(|predicate| SpecializationProposal {
            template_id: template.id(),
            predicate,
            polarity: if counterexample.contains(predicate) {
                ClausePolarity::Forbidden
            } else {
                ClausePolarity::Required
            },
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        ClausePolarity, StructuralReview, ValidationOutcome, consolidate_validation,
        counterexample_specializations, one_literal_generalizations,
    };
    use crate::experimental::tdi2_intuition::{BooleanState, PredicateId, Template, TemplateId};
    use crate::experimental::tdi2_intuition_reliability::ReliabilityEvidence;

    #[test]
    fn confirmation_increments_success_only() {
        let update = consolidate_validation(
            TemplateId::new(3),
            ReliabilityEvidence::new(4, 2),
            ValidationOutcome::Confirmed,
        )
        .expect("counter has room");
        assert_eq!(update.after(), ReliabilityEvidence::new(5, 2));
        assert_eq!(update.before(), ReliabilityEvidence::new(4, 2));
    }

    #[test]
    fn refutation_increments_failure_only() {
        let update = consolidate_validation(
            TemplateId::new(3),
            ReliabilityEvidence::new(4, 2),
            ValidationOutcome::Refuted,
        )
        .expect("counter has room");
        assert_eq!(update.after(), ReliabilityEvidence::new(4, 3));
    }

    #[test]
    fn consolidation_record_binds_before_and_after_evidence() {
        let update = consolidate_validation(
            TemplateId::new(3),
            ReliabilityEvidence::new(4, 2),
            ValidationOutcome::Confirmed,
        )
        .expect("update");
        let record = update.canonical_record();
        assert!(record.contains("before=4:2"));
        assert!(record.contains("after=5:2"));
    }

    #[test]
    fn structural_changes_remain_review_proposals() {
        let review = StructuralReview::ConsiderGeneralization {
            template_id: TemplateId::new(8),
        };
        assert_eq!(
            review,
            StructuralReview::ConsiderGeneralization {
                template_id: TemplateId::new(8)
            }
        );
    }

    #[test]
    fn generalization_proposals_remove_only_one_clause_at_a_time() {
        let template = Template::new(
            TemplateId::new(9),
            vec![PredicateId::new(1), PredicateId::new(2)],
            vec![PredicateId::new(7)],
            Vec::new(),
        )
        .expect("template");
        let proposals = one_literal_generalizations(&template);
        assert_eq!(proposals.len(), 3);
        assert_eq!(proposals[0].polarity, ClausePolarity::Required);
    }

    #[test]
    fn specialization_polarity_excludes_counterexample() {
        let template = Template::new(
            TemplateId::new(4),
            vec![PredicateId::new(1)],
            Vec::new(),
            Vec::new(),
        )
        .expect("template");
        let state = BooleanState::new(vec![PredicateId::new(1), PredicateId::new(8)]);
        let proposals = counterexample_specializations(
            &template,
            &state,
            &[PredicateId::new(8), PredicateId::new(9)],
        );
        assert_eq!(proposals.len(), 2);
        assert_eq!(proposals[0].polarity, ClausePolarity::Forbidden);
        assert_eq!(proposals[1].polarity, ClausePolarity::Required);
    }
}

/// Held-out review accounting for one proposed structural mutation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ProposalEvaluation {
    /// Positive examples that should remain admitted.
    pub positive_total: usize,
    /// Positive examples admitted by the proposed structure.
    pub positive_admitted: usize,
    /// Negative/counterexample states that should remain rejected.
    pub negative_total: usize,
    /// Negative states incorrectly admitted by the proposed structure.
    pub negative_admitted: usize,
}

impl ProposalEvaluation {
    /// Whether every supplied positive remains covered.
    #[must_use]
    pub const fn preserves_all_positives(self) -> bool {
        self.positive_admitted == self.positive_total
    }

    /// Whether every supplied negative remains rejected.
    #[must_use]
    pub const fn rejects_all_negatives(self) -> bool {
        self.negative_admitted == 0
    }

    /// A mutation is review-safe only on the explicitly supplied evidence.
    #[must_use]
    pub const fn passes_supplied_evidence(self) -> bool {
        self.preserves_all_positives() && self.rejects_all_negatives()
    }
}

/// Structural-proposal validation errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProposalError {
    /// Proposal was created for another template.
    WrongTemplate,
    /// The proposed removed clause is not present with the declared polarity.
    MissingClause,
    /// A specialization tries to add a clause already represented by the template.
    ExistingClause,
}

impl core::fmt::Display for ProposalError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::WrongTemplate => formatter.write_str("proposal targets a different template"),
            Self::MissingClause => {
                formatter.write_str("generalization clause is absent from template")
            }
            Self::ExistingClause => {
                formatter.write_str("specialization clause already exists in template")
            }
        }
    }
}
impl std::error::Error for ProposalError {}

fn matches_generalized(
    template: &Template,
    state: &BooleanState,
    proposal: GeneralizationProposal,
) -> bool {
    template.required().iter().all(|predicate| {
        (proposal.polarity == ClausePolarity::Required && *predicate == proposal.predicate)
            || state.contains(*predicate)
    }) && template.forbidden().iter().all(|predicate| {
        (proposal.polarity == ClausePolarity::Forbidden && *predicate == proposal.predicate)
            || !state.contains(*predicate)
    })
}

/// Evaluate a one-literal generalization without applying it.
pub fn evaluate_generalization_proposal(
    template: &Template,
    proposal: GeneralizationProposal,
    positives: &[BooleanState],
    negatives: &[BooleanState],
) -> Result<ProposalEvaluation, ProposalError> {
    if proposal.template_id != template.id() {
        return Err(ProposalError::WrongTemplate);
    }
    let present = match proposal.polarity {
        ClausePolarity::Required => template
            .required()
            .binary_search(&proposal.predicate)
            .is_ok(),
        ClausePolarity::Forbidden => template
            .forbidden()
            .binary_search(&proposal.predicate)
            .is_ok(),
    };
    if !present {
        return Err(ProposalError::MissingClause);
    }
    Ok(ProposalEvaluation {
        positive_total: positives.len(),
        positive_admitted: positives
            .iter()
            .filter(|state| matches_generalized(template, state, proposal))
            .count(),
        negative_total: negatives.len(),
        negative_admitted: negatives
            .iter()
            .filter(|state| matches_generalized(template, state, proposal))
            .count(),
    })
}

#[cfg(test)]
mod generalization_evaluation_tests {
    use super::{ClausePolarity, GeneralizationProposal, evaluate_generalization_proposal};
    use crate::experimental::tdi2_intuition::{BooleanState, PredicateId, Template, TemplateId};

    #[test]
    fn generalization_is_rejected_when_it_admits_a_counterexample() {
        let template = Template::new(
            TemplateId::new(7),
            vec![PredicateId::new(1), PredicateId::new(2)],
            Vec::new(),
            Vec::new(),
        )
        .expect("template");
        let proposal = GeneralizationProposal {
            template_id: template.id(),
            predicate: PredicateId::new(2),
            polarity: ClausePolarity::Required,
        };
        let evaluation = evaluate_generalization_proposal(
            &template,
            proposal,
            &[BooleanState::new(vec![
                PredicateId::new(1),
                PredicateId::new(2),
            ])],
            &[BooleanState::new(vec![
                PredicateId::new(1),
                PredicateId::new(9),
            ])],
        )
        .expect("valid proposal");
        assert!(evaluation.preserves_all_positives());
        assert!(!evaluation.rejects_all_negatives());
        assert!(!evaluation.passes_supplied_evidence());
    }
}

use super::tdi2_intuition_matching::match_template;

fn matches_specialized(
    template: &Template,
    state: &BooleanState,
    proposal: SpecializationProposal,
) -> bool {
    if !match_template(template, state).is_exact() {
        return false;
    }
    match proposal.polarity {
        ClausePolarity::Required => state.contains(proposal.predicate),
        ClausePolarity::Forbidden => !state.contains(proposal.predicate),
    }
}

/// Evaluate a one-literal specialization without applying it.
pub fn evaluate_specialization_proposal(
    template: &Template,
    proposal: SpecializationProposal,
    positives: &[BooleanState],
    negatives: &[BooleanState],
) -> Result<ProposalEvaluation, ProposalError> {
    if proposal.template_id != template.id() {
        return Err(ProposalError::WrongTemplate);
    }
    if template
        .required()
        .binary_search(&proposal.predicate)
        .is_ok()
        || template
            .forbidden()
            .binary_search(&proposal.predicate)
            .is_ok()
    {
        return Err(ProposalError::ExistingClause);
    }
    Ok(ProposalEvaluation {
        positive_total: positives.len(),
        positive_admitted: positives
            .iter()
            .filter(|state| matches_specialized(template, state, proposal))
            .count(),
        negative_total: negatives.len(),
        negative_admitted: negatives
            .iter()
            .filter(|state| matches_specialized(template, state, proposal))
            .count(),
    })
}

#[cfg(test)]
mod specialization_evaluation_tests {
    use super::{ClausePolarity, SpecializationProposal, evaluate_specialization_proposal};
    use crate::experimental::tdi2_intuition::{BooleanState, PredicateId, Template, TemplateId};

    #[test]
    fn specialization_can_exclude_counterexample_without_losing_positives() {
        let template = Template::new(
            TemplateId::new(8),
            vec![PredicateId::new(1)],
            Vec::new(),
            Vec::new(),
        )
        .expect("template");
        let proposal = SpecializationProposal {
            template_id: template.id(),
            predicate: PredicateId::new(8),
            polarity: ClausePolarity::Forbidden,
        };
        let evaluation = evaluate_specialization_proposal(
            &template,
            proposal,
            &[
                BooleanState::new(vec![PredicateId::new(1)]),
                BooleanState::new(vec![PredicateId::new(1), PredicateId::new(9)]),
            ],
            &[BooleanState::new(vec![
                PredicateId::new(1),
                PredicateId::new(8),
            ])],
        )
        .expect("valid proposal");
        assert!(evaluation.passes_supplied_evidence());
    }
}
