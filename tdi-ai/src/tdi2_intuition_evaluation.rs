//! Evaluation metrics for TDI-2.1 development/validation intuition runs.

use super::tdi2_intuition_inference::IntuitionOutcome;
use super::tdi2_intuition_tasks::SyntheticCase;

/// Aggregate evaluation counts with abstention kept explicit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EvaluationSummary {
    total: u64,
    selected: u64,
    correct: u64,
    incorrect: u64,
    abstained: u64,
}

impl EvaluationSummary {
    /// Observe one labelled case and one inference outcome.
    pub fn observe(&mut self, case: &SyntheticCase, outcome: IntuitionOutcome) {
        self.total += 1;
        match outcome {
            IntuitionOutcome::Selected { template_id, .. } => {
                self.selected += 1;
                if template_id == case.expected_template() {
                    self.correct += 1;
                } else {
                    self.incorrect += 1;
                }
            }
            IntuitionOutcome::InsufficientExperience => self.abstained += 1,
        }
    }

    /// Total number of evaluated cases.
    #[must_use]
    pub const fn total(self) -> u64 { self.total }

    /// Number of non-abstained predictions.
    #[must_use]
    pub const fn selected(self) -> u64 { self.selected }

    /// Number of correct selected predictions.
    #[must_use]
    pub const fn correct(self) -> u64 { self.correct }

    /// Number of incorrect selected predictions.
    #[must_use]
    pub const fn incorrect(self) -> u64 { self.incorrect }

    /// Number of abstentions.
    #[must_use]
    pub const fn abstained(self) -> u64 { self.abstained }

    /// Fraction of cases for which the system produced a prediction.
    #[must_use]
    pub fn coverage(self) -> Option<f64> {
        (self.total > 0).then(|| self.selected as f64 / self.total as f64)
    }

    /// Correct predictions divided by all evaluated cases.
    #[must_use]
    pub fn overall_accuracy(self) -> Option<f64> {
        (self.total > 0).then(|| self.correct as f64 / self.total as f64)
    }

    /// Correct predictions divided by selected cases only.
    #[must_use]
    pub fn selected_accuracy(self) -> Option<f64> {
        (self.selected > 0).then(|| self.correct as f64 / self.selected as f64)
    }

    /// Error rate among selected cases.
    #[must_use]
    pub fn selective_risk(self) -> Option<f64> {
        (self.selected > 0).then(|| self.incorrect as f64 / self.selected as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::EvaluationSummary;
    use crate::experimental::tdi2_intuition::TemplateId;
    use crate::experimental::tdi2_intuition_inference::IntuitionOutcome;
    use crate::experimental::tdi2_intuition_tasks::motif_retrieval_case;

    #[test]
    fn abstention_is_counted_without_becoming_an_error() {
        let mut summary = EvaluationSummary::default();
        let case = motif_retrieval_case(0);
        summary.observe(&case, IntuitionOutcome::InsufficientExperience);
        assert_eq!(summary.abstained(), 1);
        assert_eq!(summary.coverage(), Some(0.0));
        assert_eq!(summary.selected_accuracy(), None);
    }

    #[test]
    fn selected_accuracy_and_risk_are_complements() {
        let mut summary = EvaluationSummary::default();
        let case = motif_retrieval_case(0);
        summary.observe(
            &case,
            IntuitionOutcome::Selected {
                template_id: case.expected_template(),
                weight: 1.0,
                support: 1,
            },
        );
        summary.observe(
            &case,
            IntuitionOutcome::Selected {
                template_id: TemplateId::new(999),
                weight: 1.0,
                support: 1,
            },
        );
        assert_eq!(summary.selected_accuracy(), Some(0.5));
        assert_eq!(summary.selective_risk(), Some(0.5));
    }
}
