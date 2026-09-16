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
    pub const fn total(self) -> u64 {
        self.total
    }
    /// Number of non-abstained predictions.
    #[must_use]
    pub const fn selected(self) -> u64 {
        self.selected
    }
    /// Number of correct selected predictions.
    #[must_use]
    pub const fn correct(self) -> u64 {
        self.correct
    }
    /// Number of incorrect selected predictions.
    #[must_use]
    pub const fn incorrect(self) -> u64 {
        self.incorrect
    }
    /// Number of abstentions.
    #[must_use]
    pub const fn abstained(self) -> u64 {
        self.abstained
    }
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

/// Matched correctness accounting for intuition and one baseline on identical cases.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PairedComparison {
    /// Both arms correct.
    pub both_correct: u64,
    /// Only intuition correct.
    pub intuition_only: u64,
    /// Only baseline correct.
    pub baseline_only: u64,
    /// Neither arm correct.
    pub neither: u64,
}

impl PairedComparison {
    /// Record one paired case.
    pub fn observe(&mut self, intuition_correct: bool, baseline_correct: bool) {
        match (intuition_correct, baseline_correct) {
            (true, true) => self.both_correct += 1,
            (true, false) => self.intuition_only += 1,
            (false, true) => self.baseline_only += 1,
            (false, false) => self.neither += 1,
        }
    }

    /// Number of paired cases.
    #[must_use]
    pub const fn total(self) -> u64 {
        self.both_correct + self.intuition_only + self.baseline_only + self.neither
    }

    /// Net paired advantage in case counts: intuition-only minus baseline-only.
    #[must_use]
    pub fn net_advantage(self) -> i128 {
        i128::from(self.intuition_only) - i128::from(self.baseline_only)
    }

    /// Stable record for paired Development/Validation evidence.
    #[must_use]
    pub fn canonical_record(self) -> String {
        format!(
            "tdi2.1-paired-comparison-v1;total={};both_correct={};intuition_only={};baseline_only={};neither={};net_advantage={}",
            self.total(),
            self.both_correct,
            self.intuition_only,
            self.baseline_only,
            self.neither,
            self.net_advantage()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{EvaluationSummary, PairedComparison};
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

    #[test]
    fn paired_comparison_preserves_direction_of_disagreements() {
        let mut comparison = PairedComparison::default();
        comparison.observe(true, false);
        comparison.observe(true, false);
        comparison.observe(false, true);
        comparison.observe(true, true);
        assert_eq!(comparison.total(), 4);
        assert_eq!(comparison.intuition_only, 2);
        assert_eq!(comparison.baseline_only, 1);
        assert_eq!(comparison.net_advantage(), 1);
    }
}
