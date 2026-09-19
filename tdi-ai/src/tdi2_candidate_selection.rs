//! Frozen Stage-B candidate-selection policy for TDI-2.2.
//!
//! This policy is intentionally minimal and non-tunable: every candidate with
//! at least one known observation remains eligible, the upstream canonical
//! candidate bound is reused as the only catalogue cap, and the ordering tuple
//! is fixed in code.  No latency, expected label, outcome, evaluator annotation
//! or protected/final identity participates in selection.
//!
//! The ordering is an engineering policy, not a scientific conclusion.  It
//! prefers observation-only evidence first and shorter canonical descriptions
//! only after the evidence components tie.  Later TDI-2.2 slices may compare or
//! ablate this frozen policy, but Validation must not retune it post hoc.

use std::collections::BTreeSet;

use super::tdi2_candidate_scoring::CandidateDescriptionEvidenceScore;
use super::tdi2_induction_split::InductionDomain;

/// Stable schema for the first frozen TDI-2.2 candidate-selection policy.
pub const FROZEN_CANDIDATE_SELECTION_SCHEMA: &str = "tdi2.2-candidate-selection-v1";
/// A candidate with no known observation is not eligible for this policy.
pub const MINIMUM_KNOWN_OBSERVATIONS: u64 = 1;
/// Frozen catalogue cap. This value equals the slice-12 canonical candidate
/// bound at policy freeze time, but is intentionally copied rather than aliased:
/// a later upstream bound change must not silently retune this policy.
pub const MAX_SELECTED_CANDIDATES: usize = 16_384;

/// Fixed, reviewable policy identity.  This type has no runtime parameters.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrozenCandidateSelectionPolicyV1;

impl FrozenCandidateSelectionPolicyV1 {
    /// Machine-readable preregistration of the immutable ordering/filter rules.
    #[must_use]
    pub const fn canonical_record(self) -> &'static str {
        "tdi2.2-candidate-selection-v1;minimum_known=1;max_selected=16384;order=evidence_margin_desc,counterexamples_asc,support_desc,unknown_asc,description_bytes_asc,candidate_key_asc"
    }
}

/// One candidate emitted by the frozen ordering, with a stable one-based rank.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectedCandidate {
    rank: u32,
    score: CandidateDescriptionEvidenceScore,
}

impl SelectedCandidate {
    #[must_use]
    pub const fn rank(&self) -> u32 {
        self.rank
    }

    #[must_use]
    pub const fn score(&self) -> &CandidateDescriptionEvidenceScore {
        &self.score
    }
}

/// Fail-closed policy input errors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CandidateSelectionError {
    CatalogueLimitExceeded {
        actual: usize,
        maximum: usize,
    },
    MixedDomains {
        expected: InductionDomain,
        actual: InductionDomain,
        candidate_key: String,
    },
    DuplicateCandidate {
        candidate_key: String,
    },
    RankOverflow,
}

/// Apply the frozen candidate-selection policy.
///
/// Candidates with zero known observations are excluded because the policy has
/// no observed Boolean evidence with which to order them.  No other acceptance
/// threshold is introduced.  Negative evidence margins remain visible rather
/// than being silently discarded; subsequent induction and null-control slices
/// can therefore inspect them explicitly.
pub fn select_candidates_v1(
    scores: &[CandidateDescriptionEvidenceScore],
) -> Result<Vec<SelectedCandidate>, CandidateSelectionError> {
    if scores.len() > MAX_SELECTED_CANDIDATES {
        return Err(CandidateSelectionError::CatalogueLimitExceeded {
            actual: scores.len(),
            maximum: MAX_SELECTED_CANDIDATES,
        });
    }

    let expected_domain = scores
        .first()
        .map(CandidateDescriptionEvidenceScore::domain);
    let mut identities = BTreeSet::new();
    for score in scores {
        if let Some(expected) = expected_domain {
            if score.domain() != expected {
                return Err(CandidateSelectionError::MixedDomains {
                    expected,
                    actual: score.domain(),
                    candidate_key: score.candidate_key().to_owned(),
                });
            }
        }
        if !identities.insert(score.candidate_key()) {
            return Err(CandidateSelectionError::DuplicateCandidate {
                candidate_key: score.candidate_key().to_owned(),
            });
        }
    }

    let mut eligible = scores
        .iter()
        .filter(|score| score.known_observations() >= MINIMUM_KNOWN_OBSERVATIONS)
        .cloned()
        .collect::<Vec<_>>();

    eligible.sort_by(|left, right| {
        right
            .evidence_margin()
            .cmp(&left.evidence_margin())
            .then_with(|| left.counterexamples().cmp(&right.counterexamples()))
            .then_with(|| right.support().cmp(&left.support()))
            .then_with(|| left.unknown().cmp(&right.unknown()))
            .then_with(|| {
                left.canonical_description_bytes()
                    .cmp(&right.canonical_description_bytes())
            })
            .then_with(|| left.candidate_key().cmp(right.candidate_key()))
    });

    eligible
        .into_iter()
        .enumerate()
        .map(|(index, score)| {
            let rank =
                u32::try_from(index + 1).map_err(|_| CandidateSelectionError::RankOverflow)?;
            Ok(SelectedCandidate { rank, score })
        })
        .collect()
}

impl core::fmt::Display for CandidateSelectionError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::CatalogueLimitExceeded { actual, maximum } => write!(
                formatter,
                "candidate score catalogue count {actual} exceeds bounded maximum {maximum}"
            ),
            Self::MixedDomains {
                expected,
                actual,
                candidate_key,
            } => write!(
                formatter,
                "candidate {candidate_key} belongs to {actual:?}, expected {expected:?}"
            ),
            Self::DuplicateCandidate { candidate_key } => {
                write!(
                    formatter,
                    "candidate score identity is duplicated: {candidate_key}"
                )
            }
            Self::RankOverflow => formatter.write_str("candidate selection rank exceeds u32"),
        }
    }
}

impl std::error::Error for CandidateSelectionError {}

#[cfg(test)]
mod tests {
    use super::{
        CandidateSelectionError, FrozenCandidateSelectionPolicyV1, MAX_SELECTED_CANDIDATES,
        select_candidates_v1,
    };
    use crate::experimental::tdi2_candidate_evidence::account_candidate_evidence;
    use crate::experimental::tdi2_candidate_identity::{
        CanonicalPredicateCandidate, MAX_CANONICAL_CANDIDATES,
    };
    use crate::experimental::tdi2_candidate_scoring::{
        CandidateDescriptionEvidenceScore, score_candidate_catalogue,
    };
    use crate::experimental::tdi2_induction_input::InductionBatch;
    use crate::experimental::tdi2_induction_split::{
        DEVELOPMENT_START, InductionDomain, VALIDATION_START,
    };
    use crate::experimental::tdi2_intuition::{BooleanState, NumericState};
    use crate::experimental::tdi2_numeric_thresholds::generate_numeric_threshold_candidates;
    use crate::experimental::tdi2_observation_graph::ObservationGraph;
    use crate::experimental::tdi2_template_induction::{
        EpisodeId, ExperienceEpisode, ObservationFrame,
    };

    fn episode(id: u64, frames: &[(u32, Vec<f64>)]) -> ExperienceEpisode {
        ExperienceEpisode::new(
            EpisodeId::new(id),
            frames
                .iter()
                .map(|(ordinal, values)| {
                    ObservationFrame::new(
                        *ordinal,
                        NumericState::new(values.clone()).expect("finite"),
                        BooleanState::default(),
                    )
                })
                .collect(),
        )
        .expect("episode")
    }

    fn batch(domain: InductionDomain, episodes: Vec<ExperienceEpisode>) -> InductionBatch {
        let graphs = episodes
            .iter()
            .map(|episode| {
                ObservationGraph::new(episode.id(), Vec::new(), Vec::new()).expect("graph")
            })
            .collect();
        InductionBatch::new(domain, episodes, graphs).expect("batch")
    }

    fn source_candidates() -> Vec<CanonicalPredicateCandidate> {
        let source = batch(
            InductionDomain::Development,
            vec![episode(DEVELOPMENT_START, &[(0, vec![1.0, 2.0])])],
        );
        generate_numeric_threshold_candidates(&source)
            .expect("thresholds")
            .iter()
            .map(CanonicalPredicateCandidate::from_threshold)
            .collect()
    }

    fn score_on(
        candidate: &CanonicalPredicateCandidate,
        domain: InductionDomain,
        start: u64,
        values: &[Vec<f64>],
    ) -> CandidateDescriptionEvidenceScore {
        let frames = values
            .iter()
            .enumerate()
            .map(|(index, value)| {
                (
                    u32::try_from(index).expect("bounded fixture"),
                    value.clone(),
                )
            })
            .collect::<Vec<_>>();
        let observed = batch(domain, vec![episode(start, &frames)]);
        let evidence = account_candidate_evidence(&observed, core::slice::from_ref(candidate))
            .expect("evidence");
        score_candidate_catalogue(&evidence)
            .expect("scores")
            .remove(0)
    }

    #[test]
    fn frozen_cap_matches_the_slice12_bound_at_freeze_time() {
        assert_eq!(MAX_SELECTED_CANDIDATES, MAX_CANONICAL_CANDIDATES);
    }

    #[test]
    fn policy_identity_is_explicit_and_contains_no_latency_or_label_surface() {
        let record = FrozenCandidateSelectionPolicyV1.canonical_record();
        assert!(record.contains("minimum_known=1"));
        assert!(record.contains("max_selected=16384"));
        assert!(record.contains("evidence_margin_desc"));
        assert!(!record.contains("latency"));
        assert!(!record.contains("expected_label"));
        assert!(!record.contains("primary_holdout"));
    }

    #[test]
    fn selection_is_deterministic_and_filters_only_zero_known_evidence() {
        let candidates = source_candidates();
        assert!(candidates.len() >= 3);
        let supported = score_on(
            &candidates[0],
            InductionDomain::Development,
            DEVELOPMENT_START + 1,
            &[vec![0.5, 2.0], vec![0.75, 2.0]],
        );
        let mixed = score_on(
            &candidates[1],
            InductionDomain::Development,
            DEVELOPMENT_START + 2,
            &[vec![0.5, 2.0], vec![2.0, 2.0]],
        );
        let unknown = score_on(
            &candidates[2],
            InductionDomain::Development,
            DEVELOPMENT_START + 3,
            &[Vec::new()],
        );
        assert_eq!(unknown.known_observations(), 0);

        let forward = select_candidates_v1(&[mixed.clone(), unknown.clone(), supported.clone()])
            .expect("selection");
        let reverse = select_candidates_v1(&[supported, unknown, mixed]).expect("selection");
        assert_eq!(forward, reverse);
        assert_eq!(forward.len(), 2);
        assert_eq!(forward[0].rank(), 1);
        assert!(forward[0].score().evidence_margin() >= forward[1].score().evidence_margin());
    }

    #[test]
    fn mixed_domains_fail_closed() {
        let candidates = source_candidates();
        let development = score_on(
            &candidates[0],
            InductionDomain::Development,
            DEVELOPMENT_START + 1,
            &[vec![0.5, 2.0]],
        );
        let validation = score_on(
            &candidates[1],
            InductionDomain::Validation,
            VALIDATION_START,
            &[vec![0.5, 2.0]],
        );
        let error = select_candidates_v1(&[development, validation]).expect_err("mixed domains");
        assert!(matches!(
            error,
            CandidateSelectionError::MixedDomains { .. }
        ));
    }

    #[test]
    fn duplicate_and_oversized_catalogues_fail_closed() {
        let candidates = source_candidates();
        let score = score_on(
            &candidates[0],
            InductionDomain::Development,
            DEVELOPMENT_START + 1,
            &[vec![0.5, 2.0]],
        );
        let duplicate = vec![score.clone(), score.clone()];
        let duplicate_error = select_candidates_v1(&duplicate).expect_err("duplicate");
        assert!(matches!(
            duplicate_error,
            CandidateSelectionError::DuplicateCandidate { .. }
        ));

        let oversized = vec![score; MAX_SELECTED_CANDIDATES + 1];
        let limit_error = select_candidates_v1(&oversized).expect_err("bounded catalogue");
        assert_eq!(
            limit_error,
            CandidateSelectionError::CatalogueLimitExceeded {
                actual: MAX_SELECTED_CANDIDATES + 1,
                maximum: MAX_SELECTED_CANDIDATES,
            }
        );
    }
}
