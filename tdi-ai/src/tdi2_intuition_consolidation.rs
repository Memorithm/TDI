//! Explicit post-outcome consolidation for TDI-2.1 experiential evidence.
//!
//! Inference never calls this module implicitly. A real outcome must first be
//! observed and classified by the experimental protocol.

use super::tdi2_intuition::{PredicateId, Template, TemplateId};
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

/// Enumerate one-literal generalizations without mutating the template.
///
/// If a template contains only one Boolean condition, no proposal is emitted,
/// because applying it would produce the invalid empty-template case.
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
        .chain(template.forbidden().iter().copied().map(|predicate| {
            GeneralizationProposal {
                template_id: template.id(),
                predicate,
                polarity: ClausePolarity::Forbidden,
            }
        }))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        ClausePolarity, StructuralReview, ValidationOutcome, consolidate_validation,
        one_literal_generalizations,
    };
    use crate::experimental::tdi2_intuition::{PredicateId, Template, TemplateId};
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
}
