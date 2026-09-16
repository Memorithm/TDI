//! Development/Validation campaign harness for TDI-2.1.
//!
//! This module deliberately excludes protected/final holdout access.

use super::tdi2_intuition_evaluation::EvaluationSummary;
use super::tdi2_intuition_inference::IntuitionOutcome;
use super::tdi2_intuition_tasks::SyntheticCase;

/// Allowed non-final experimental domains for this harness.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CampaignDomain {
    /// Iterative engineering and exploratory scientific development.
    Development,
    /// Frozen non-final validation material.
    Validation,
}

/// Evaluate a fixed case list with a supplied inference closure.
///
/// The harness owns only iteration and accounting. It does not mutate memory,
/// tune thresholds, or inspect any protected holdout.
pub fn run_synthetic_campaign<F>(
    _domain: CampaignDomain,
    cases: &[SyntheticCase],
    mut infer: F,
) -> EvaluationSummary
where
    F: FnMut(&SyntheticCase) -> IntuitionOutcome,
{
    let mut summary = EvaluationSummary::default();
    for case in cases {
        summary.observe(case, infer(case));
    }
    summary
}

#[cfg(test)]
mod tests {
    use super::{CampaignDomain, run_synthetic_campaign};
    use crate::experimental::tdi2_intuition_inference::IntuitionOutcome;
    use crate::experimental::tdi2_intuition_tasks::motif_retrieval_case;

    #[test]
    fn campaign_preserves_explicit_abstention() {
        let cases = [motif_retrieval_case(0), motif_retrieval_case(1)];
        let summary = run_synthetic_campaign(CampaignDomain::Development, &cases, |_| {
            IntuitionOutcome::InsufficientExperience
        });
        assert_eq!(summary.total(), 2);
        assert_eq!(summary.abstained(), 2);
        assert_eq!(summary.coverage(), Some(0.0));
    }
}
