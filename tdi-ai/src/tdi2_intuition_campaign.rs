//! Development/Validation campaign harness for TDI-2.1.
//!
//! This module deliberately excludes protected/final holdout access.

use super::tdi2_intuition_analogy_tasks::{AnalogyCase, analogy_case};
use super::tdi2_intuition_evaluation::{EvaluationSummary, PairedComparison};
use super::tdi2_intuition_inference::IntuitionOutcome;
use super::tdi2_intuition_tasks::{
    SyntheticCase, TemporalCase, context_reversal_pair, context_template_pair,
    motif_retrieval_case, temporal_trend_case,
};

/// Allowed non-final experimental domains for this harness.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CampaignDomain {
    /// Iterative engineering and exploratory scientific development.
    Development,
    /// Frozen non-final validation material.
    Validation,
}

/// Frozen campaign family identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CampaignFamily {
    /// Static motif retrieval with distractors.
    MotifRetrieval,
    /// Context-conditioned reversal.
    ContextReversal,
    /// Reusable context templates over novel case identities.
    ContextTemplateTransfer,
    /// Ordered temporal trend.
    TemporalTrend,
    /// Novel-identity structural transfer.
    AnalogyTransfer,
}

/// Frozen motif Development population start.
pub const MOTIF_DEVELOPMENT_START: u32 = 1_000;
/// Frozen motif Validation population start.
pub const MOTIF_VALIDATION_START: u32 = 2_000;
/// Number of motif cases in each non-final population.
pub const MOTIF_DOMAIN_CASES: usize = 34;

/// Return the frozen motif manifest for one non-final domain.
#[must_use]
pub fn frozen_motif_manifest(domain: CampaignDomain) -> CampaignManifest {
    let start_id = match domain {
        CampaignDomain::Development => MOTIF_DEVELOPMENT_START,
        CampaignDomain::Validation => MOTIF_VALIDATION_START,
    };
    CampaignManifest::new(
        domain,
        CampaignFamily::MotifRetrieval,
        start_id,
        MOTIF_DOMAIN_CASES,
        0,
        0.0,
    )
    .expect("frozen motif manifest constants are valid")
}

/// Frozen transferable-context Development pair-id start.
pub const CONTEXT_DEVELOPMENT_START: u32 = 3_000;
/// Frozen transferable-context Validation pair-id start.
pub const CONTEXT_VALIDATION_START: u32 = 4_000;
/// Number of pair ids per non-final context domain.
pub const CONTEXT_DOMAIN_PAIRS: usize = 26;

/// Return the frozen context-template manifest for one non-final domain.
#[must_use]
pub fn frozen_context_manifest(domain: CampaignDomain) -> CampaignManifest {
    let start_id = match domain {
        CampaignDomain::Development => CONTEXT_DEVELOPMENT_START,
        CampaignDomain::Validation => CONTEXT_VALIDATION_START,
    };
    CampaignManifest::new(
        domain,
        CampaignFamily::ContextTemplateTransfer,
        start_id,
        CONTEXT_DOMAIN_PAIRS,
        0,
        0.0,
    )
    .expect("frozen context manifest constants are valid")
}

/// Frozen analogy Development case-id start.
pub const ANALOGY_DEVELOPMENT_START: u32 = 7_000;
/// Frozen analogy Validation case-id start.
pub const ANALOGY_VALIDATION_START: u32 = 8_000;
/// Number of analogy cases per non-final domain.
pub const ANALOGY_DOMAIN_CASES: usize = 32;

/// Return the frozen analogy manifest for one non-final domain.
#[must_use]
pub fn frozen_analogy_manifest(domain: CampaignDomain) -> CampaignManifest {
    let start_id = match domain {
        CampaignDomain::Development => ANALOGY_DEVELOPMENT_START,
        CampaignDomain::Validation => ANALOGY_VALIDATION_START,
    };
    CampaignManifest::new(
        domain,
        CampaignFamily::AnalogyTransfer,
        start_id,
        ANALOGY_DOMAIN_CASES,
        0,
        0.0,
    )
    .expect("frozen analogy manifest constants are valid")
}

/// Canonical non-final campaign manifest.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CampaignManifest {
    /// Development or Validation only.
    pub domain: CampaignDomain,
    /// Frozen task family.
    pub family: CampaignFamily,
    /// First deterministic case identifier.
    pub start_id: u32,
    /// Number of case ids requested.
    pub count: usize,
    /// Seed reserved for matched stochastic controls.
    pub control_seed: u64,
    /// External minimum experience-weight threshold.
    pub min_weight: f64,
}

impl CampaignManifest {
    /// Validate and construct a campaign manifest.
    pub fn new(
        domain: CampaignDomain,
        family: CampaignFamily,
        start_id: u32,
        count: usize,
        control_seed: u64,
        min_weight: f64,
    ) -> Result<Self, CampaignError> {
        if !min_weight.is_finite() || min_weight < 0.0 {
            return Err(CampaignError::InvalidMinimumWeight);
        }
        Ok(Self {
            domain,
            family,
            start_id,
            count,
            control_seed,
            min_weight,
        })
    }

    /// Stable record suitable for hashing before campaign execution.
    #[must_use]
    pub fn canonical_record(self) -> String {
        let domain = match self.domain {
            CampaignDomain::Development => "development",
            CampaignDomain::Validation => "validation",
        };
        let family = match self.family {
            CampaignFamily::MotifRetrieval => "motif-retrieval",
            CampaignFamily::ContextReversal => "context-reversal",
            CampaignFamily::ContextTemplateTransfer => "context-template-transfer",
            CampaignFamily::TemporalTrend => "temporal-trend",
            CampaignFamily::AnalogyTransfer => "analogy-transfer",
        };
        format!(
            "tdi2.1-campaign-manifest-v1;domain={domain};family={family};start={};count={};control_seed={};min_weight_bits={:016x}",
            self.start_id,
            self.count,
            self.control_seed,
            self.min_weight.to_bits()
        )
    }
}

/// Canonical post-run artifact binding a manifest to observed denominators and outcomes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CampaignResultArtifact {
    /// Frozen pre-run manifest.
    pub manifest: CampaignManifest,
    /// Observed aggregate evaluation.
    pub summary: EvaluationSummary,
}

impl CampaignResultArtifact {
    /// Construct a result artifact from a frozen manifest and completed summary.
    #[must_use]
    pub const fn new(manifest: CampaignManifest, summary: EvaluationSummary) -> Self {
        Self { manifest, summary }
    }

    /// Stable result record. The manifest is included verbatim as a nested canonical record.
    #[must_use]
    pub fn canonical_record(self) -> String {
        format!(
            "tdi2.1-campaign-result-v1;manifest=[{}];total={};selected={};correct={};incorrect={};abstained={}",
            self.manifest.canonical_record(),
            self.summary.total(),
            self.summary.selected(),
            self.summary.correct(),
            self.summary.incorrect(),
            self.summary.abstained()
        )
    }
}

/// Campaign-construction failures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CampaignError {
    /// Requested case-id range exceeds the representable u32 domain.
    CaseIdOverflow,
    /// Requested output case count overflows usize.
    CaseCountOverflow,
    /// External inference threshold must be finite and non-negative.
    InvalidMinimumWeight,
}

impl core::fmt::Display for CampaignError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::CaseIdOverflow => {
                formatter.write_str("TDI-2.1 campaign case-id range overflows u32")
            }
            Self::CaseCountOverflow => {
                formatter.write_str("TDI-2.1 campaign case count overflows usize")
            }
            Self::InvalidMinimumWeight => {
                formatter.write_str("TDI-2.1 minimum weight must be finite and non-negative")
            }
        }
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

fn outcome_is_correct(case: &SyntheticCase, outcome: IntuitionOutcome) -> bool {
    matches!(
        outcome,
        IntuitionOutcome::Selected { template_id, .. } if template_id == case.expected_template()
    )
}

/// Compare intuition and a baseline on the identical ordered case list.
pub fn run_paired_campaign<FI, FB>(
    _domain: CampaignDomain,
    cases: &[SyntheticCase],
    mut intuition: FI,
    mut baseline: FB,
) -> PairedComparison
where
    FI: FnMut(&SyntheticCase) -> IntuitionOutcome,
    FB: FnMut(&SyntheticCase) -> IntuitionOutcome,
{
    let mut comparison = PairedComparison::default();
    for case in cases {
        comparison.observe(
            outcome_is_correct(case, intuition(case)),
            outcome_is_correct(case, baseline(case)),
        );
    }
    comparison
}

/// Materialize a deterministic contiguous motif-retrieval campaign.
pub fn motif_cases(start: u32, count: usize) -> Result<Vec<SyntheticCase>, CampaignError> {
    let mut cases = Vec::with_capacity(count);
    for offset in 0..count {
        let offset = u32::try_from(offset).map_err(|_| CampaignError::CaseIdOverflow)?;
        let id = start
            .checked_add(offset)
            .ok_or(CampaignError::CaseIdOverflow)?;
        cases.push(motif_retrieval_case(id));
    }
    Ok(cases)
}

/// Materialize paired context-reversal cases for a checked contiguous id range.
pub fn context_cases(start: u32, pair_count: usize) -> Result<Vec<SyntheticCase>, CampaignError> {
    let capacity = pair_count
        .checked_mul(2)
        .ok_or(CampaignError::CaseCountOverflow)?;
    let mut cases = Vec::with_capacity(capacity);
    for offset in 0..pair_count {
        let offset = u32::try_from(offset).map_err(|_| CampaignError::CaseIdOverflow)?;
        let id = start
            .checked_add(offset)
            .ok_or(CampaignError::CaseIdOverflow)?;
        cases.extend(context_reversal_pair(id));
    }
    Ok(cases)
}

/// Materialize reusable context-template pairs for a checked id range.
pub fn transferable_context_cases(
    start: u32,
    pair_count: usize,
) -> Result<Vec<SyntheticCase>, CampaignError> {
    let capacity = pair_count
        .checked_mul(2)
        .ok_or(CampaignError::CaseCountOverflow)?;
    let mut cases = Vec::with_capacity(capacity);
    for offset in 0..pair_count {
        let offset = u32::try_from(offset).map_err(|_| CampaignError::CaseIdOverflow)?;
        let id = start
            .checked_add(offset)
            .ok_or(CampaignError::CaseIdOverflow)?;
        cases.extend(context_template_pair(id));
    }
    Ok(cases)
}

/// Materialize deterministic ordered temporal cases for a checked id range.
pub fn temporal_cases(start: u32, count: usize) -> Result<Vec<TemporalCase>, CampaignError> {
    let mut cases = Vec::with_capacity(count);
    for offset in 0..count {
        let offset = u32::try_from(offset).map_err(|_| CampaignError::CaseIdOverflow)?;
        let id = start
            .checked_add(offset)
            .ok_or(CampaignError::CaseIdOverflow)?;
        cases.push(temporal_trend_case(id));
    }
    Ok(cases)
}

/// Materialize deterministic novel-identity analogy cases for a checked id range.
pub fn analogy_cases(start: u32, count: usize) -> Result<Vec<AnalogyCase>, CampaignError> {
    let mut cases = Vec::with_capacity(count);
    for offset in 0..count {
        let offset = u32::try_from(offset).map_err(|_| CampaignError::CaseIdOverflow)?;
        let id = start
            .checked_add(offset)
            .ok_or(CampaignError::CaseIdOverflow)?;
        cases.push(analogy_case(id));
    }
    Ok(cases)
}

#[cfg(test)]
mod tests {
    use super::{
        CampaignDomain, CampaignFamily, CampaignManifest, CampaignResultArtifact,
        MOTIF_DEVELOPMENT_START, MOTIF_DOMAIN_CASES, MOTIF_VALIDATION_START, analogy_cases,
        context_cases, frozen_motif_manifest, motif_cases, run_paired_campaign,
        run_synthetic_campaign, temporal_cases,
    };
    use crate::experimental::tdi2_intuition_analogy_tasks::analogy_case;
    use crate::experimental::tdi2_intuition_inference::IntuitionOutcome;
    use crate::experimental::tdi2_intuition_tasks::{
        context_reversal_pair, motif_retrieval_case, temporal_trend_case,
    };

    #[test]
    fn campaign_manifest_binds_external_threshold_bits() {
        let manifest = CampaignManifest::new(
            CampaignDomain::Validation,
            CampaignFamily::MotifRetrieval,
            10,
            20,
            7,
            0.5,
        )
        .expect("manifest");
        let record = manifest.canonical_record();
        assert!(record.contains("domain=validation"));
        assert!(record.contains("min_weight_bits=3fe0000000000000"));
    }

    #[test]
    fn result_artifact_keeps_abstentions_in_denominator() {
        let manifest = CampaignManifest::new(
            CampaignDomain::Development,
            CampaignFamily::MotifRetrieval,
            0,
            2,
            0,
            0.0,
        )
        .expect("manifest");
        let cases = motif_cases(0, 2).expect("cases");
        let summary = run_synthetic_campaign(CampaignDomain::Development, &cases, |_| {
            IntuitionOutcome::InsufficientExperience
        });
        let record = CampaignResultArtifact::new(manifest, summary).canonical_record();
        assert!(record.contains("total=2"));
        assert!(record.contains("abstained=2"));
    }

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
    fn paired_campaign_preserves_case_pairing() {
        let cases = motif_cases(0, 2).expect("range");
        let comparison = run_paired_campaign(
            CampaignDomain::Development,
            &cases,
            |case| IntuitionOutcome::Selected {
                template_id: case.expected_template(),
                weight: 1.0,
                support: 1,
            },
            |_| IntuitionOutcome::InsufficientExperience,
        );
        assert_eq!(comparison.intuition_only, 2);
        assert_eq!(comparison.baseline_only, 0);
    }

    #[test]
    fn motif_range_is_deterministic_and_contiguous() {
        let cases = motif_cases(10, 3).expect("bounded range");
        assert_eq!(cases[0], motif_retrieval_case(10));
        assert_eq!(cases[2], motif_retrieval_case(12));
    }

    #[test]
    fn context_range_preserves_pair_adjacency() {
        let cases = context_cases(4, 2).expect("bounded range");
        let first = context_reversal_pair(4);
        assert_eq!(cases.len(), 4);
        assert_eq!(cases[0], first[0]);
        assert_eq!(cases[1], first[1]);
    }

    #[test]
    fn temporal_range_is_deterministic_and_contiguous() {
        let cases = temporal_cases(7, 2).expect("bounded range");
        assert_eq!(cases[0], temporal_trend_case(7));
        assert_eq!(cases[1], temporal_trend_case(8));
    }

    #[test]
    fn analogy_range_is_deterministic_and_contiguous() {
        let cases = analogy_cases(3, 2).expect("bounded range");
        assert_eq!(cases[0], analogy_case(3));
        assert_eq!(cases[1], analogy_case(4));
    }

    #[test]
    fn frozen_motif_populations_are_disjoint_and_equal_sized() {
        let development = frozen_motif_manifest(CampaignDomain::Development);
        let validation = frozen_motif_manifest(CampaignDomain::Validation);
        assert_eq!(development.start_id, MOTIF_DEVELOPMENT_START);
        assert_eq!(validation.start_id, MOTIF_VALIDATION_START);
        assert_eq!(development.count, MOTIF_DOMAIN_CASES);
        assert_eq!(validation.count, MOTIF_DOMAIN_CASES);
        assert!(development.start_id + development.count as u32 <= validation.start_id);
    }

    #[test]
    fn frozen_context_populations_are_disjoint_and_cover_repeated_classes() {
        let development = super::frozen_context_manifest(CampaignDomain::Development);
        let validation = super::frozen_context_manifest(CampaignDomain::Validation);
        assert_eq!(development.count, super::CONTEXT_DOMAIN_PAIRS);
        assert_eq!(validation.count, super::CONTEXT_DOMAIN_PAIRS);
        assert!(development.start_id + development.count as u32 <= validation.start_id);
        let cases = super::transferable_context_cases(development.start_id, development.count)
            .expect("cases");
        assert_eq!(cases.len(), 52);
    }

    #[test]
    fn frozen_analogy_populations_are_disjoint() {
        let development = super::frozen_analogy_manifest(CampaignDomain::Development);
        let validation = super::frozen_analogy_manifest(CampaignDomain::Validation);
        assert_eq!(development.count, super::ANALOGY_DOMAIN_CASES);
        assert_eq!(validation.count, super::ANALOGY_DOMAIN_CASES);
        assert!(development.start_id + development.count as u32 <= validation.start_id);
    }
}
