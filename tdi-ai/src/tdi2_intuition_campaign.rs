//! Development/Validation campaign harness for TDI-2.1.
//!
//! This module deliberately excludes protected/final holdout access.

use super::tdi2_intuition_evaluation::EvaluationSummary;
use super::tdi2_intuition_inference::IntuitionOutcome;
use super::tdi2_intuition_tasks::{SyntheticCase, motif_retrieval_case};

/// Allowed non-final experimental domains for this harness.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CampaignDomain {
    /// Iterative engineering and exploratory scientific development.
    Development,
    /// Frozen non-final validation material.
    Validation,
}

/// Campaign-construction failures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CampaignError {
    /// Requested case-id range exceeds the representable u32 domain.
    CaseIdOverflow,
}

impl core::fmt::Display for CampaignError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("TDI-2.1 campaign case-id range overflows u32")
    }
}
impl std::error::Error for CampaignError {}

/// Evaluate a fixed case list with a supplied inference closure.
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

/// Materialize a deterministic contiguous motif-retrieval campaign.
pub fn motif_cases(start: u32, count: usize) -> Result<Vec<SyntheticCase>, CampaignError> {
    let mut cases = Vec::with_capacity(count);
    for offset in 0..count {
        let offset = u32::try_from(offset).map_err(|_| CampaignError::CaseIdOverflow)?;
        let id = start.checked_add(offset).ok_or(CampaignError::CaseIdOverflow)?;
        cases.push(motif_retrieval_case(id));
    }
    Ok(cases)
}

#[cfg(test)]
mod tests {
    use super::{CampaignDomain, motif_cases, run_synthetic_campaign};
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

    #[test]
    fn motif_range_is_deterministic_and_contiguous() {
        let cases = motif_cases(10, 3).expect("bounded range");
        assert_eq!(cases[0], motif_retrieval_case(10));
        assert_eq!(cases[2], motif_retrieval_case(12));
    }
}
