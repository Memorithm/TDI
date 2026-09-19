//! Frozen predicate-discovery ablations and null controls for TDI-2.2.
//!
//! Stage-B slice 16 compares ordering ingredients without opening any protected
//! population and without consuming task labels, outcomes, evaluator state or
//! latency. Every control starts from the exact same eligible candidate set
//! produced by the slice-15 frozen policy. The controls therefore isolate
//! ordering information rather than silently changing admission.
//!
//! These are comparison instruments only. Producing a different ordering is not
//! evidence that one policy is better, novel, or scientifically significant.

use super::tdi2_candidate_scoring::CandidateDescriptionEvidenceScore;
use super::tdi2_candidate_selection::{
    CandidateSelectionError, SelectedCandidate, select_candidates_v1,
};

/// Stable schema for the Stage-B predicate-discovery control family.
pub const PREDICATE_DISCOVERY_CONTROLS_SCHEMA: &str = "tdi2.2-predicate-discovery-controls-v1";

/// Frozen ablation/null-control identities.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PredicateDiscoveryControl {
    /// Evidence ordering without the canonical-description length tie-break.
    EvidenceWithoutDescription,
    /// Support-only ordering: negative-evidence terms are removed from ranking.
    CounterexampleBlind,
    /// Description complexity only; observation evidence does not affect order.
    DescriptionOnly,
    /// Pure deterministic null: exact canonical candidate identity only.
    CanonicalIdentityNull,
}

impl PredicateDiscoveryControl {
    #[must_use]
    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::EvidenceWithoutDescription => "evidence_without_description",
            Self::CounterexampleBlind => "counterexample_blind",
            Self::DescriptionOnly => "description_only",
            Self::CanonicalIdentityNull => "canonical_identity_null",
        }
    }

    /// Immutable machine-readable definition of the control ordering.
    #[must_use]
    pub const fn canonical_record(self) -> &'static str {
        match self {
            Self::EvidenceWithoutDescription => {
                "tdi2.2-predicate-discovery-controls-v1;control=evidence_without_description;eligibility=frozen_v1;order=evidence_margin_desc,counterexamples_asc,support_desc,unknown_asc,candidate_key_asc"
            }
            Self::CounterexampleBlind => {
                "tdi2.2-predicate-discovery-controls-v1;control=counterexample_blind;eligibility=frozen_v1;order=support_desc,unknown_asc,description_bytes_asc,candidate_key_asc"
            }
            Self::DescriptionOnly => {
                "tdi2.2-predicate-discovery-controls-v1;control=description_only;eligibility=frozen_v1;order=description_bytes_asc,candidate_key_asc"
            }
            Self::CanonicalIdentityNull => {
                "tdi2.2-predicate-discovery-controls-v1;control=canonical_identity_null;eligibility=frozen_v1;order=candidate_key_asc"
            }
        }
    }
}

/// One candidate rank emitted by one frozen control.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ControlledCandidateRank {
    rank: u32,
    score: CandidateDescriptionEvidenceScore,
}

impl ControlledCandidateRank {
    #[must_use]
    pub const fn rank(&self) -> u32 {
        self.rank
    }

    #[must_use]
    pub const fn score(&self) -> &CandidateDescriptionEvidenceScore {
        &self.score
    }
}

/// Full report for one control, bound to its immutable policy identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PredicateDiscoveryControlReport {
    control: PredicateDiscoveryControl,
    candidates: Vec<ControlledCandidateRank>,
}

impl PredicateDiscoveryControlReport {
    #[must_use]
    pub const fn control(&self) -> PredicateDiscoveryControl {
        self.control
    }

    #[must_use]
    pub fn policy_record(&self) -> &'static str {
        self.control.canonical_record()
    }

    #[must_use]
    pub fn candidates(&self) -> &[ControlledCandidateRank] {
        &self.candidates
    }

    /// Deterministic report record. This is an ordering artifact, not a result.
    #[must_use]
    pub fn canonical_record(&self) -> String {
        let mut keys = String::new();
        for (index, ranked) in self.candidates.iter().enumerate() {
            if index != 0 {
                keys.push(',');
            }
            keys.push_str(ranked.score.candidate_key());
        }
        format!("{};ranking={keys}", self.control.canonical_record())
    }
}

/// Generate every frozen Stage-B ablation/null-control ranking.
///
/// Calling the slice-15 selector first is deliberate: it applies the frozen
/// bounds, domain consistency, duplicate rejection and eligibility gate once.
/// The resulting candidate membership is then reused unchanged by every control.
pub fn run_predicate_discovery_controls(
    scores: &[CandidateDescriptionEvidenceScore],
) -> Result<Vec<PredicateDiscoveryControlReport>, CandidateSelectionError> {
    let eligible = select_candidates_v1(scores)?
        .into_iter()
        .map(|selected| selected.score().clone())
        .collect::<Vec<_>>();

    const CONTROLS: [PredicateDiscoveryControl; 4] = [
        PredicateDiscoveryControl::EvidenceWithoutDescription,
        PredicateDiscoveryControl::CounterexampleBlind,
        PredicateDiscoveryControl::DescriptionOnly,
        PredicateDiscoveryControl::CanonicalIdentityNull,
    ];

    CONTROLS
        .into_iter()
        .map(|control| rank_control(control, &eligible))
        .collect()
}

fn rank_control(
    control: PredicateDiscoveryControl,
    eligible: &[CandidateDescriptionEvidenceScore],
) -> Result<PredicateDiscoveryControlReport, CandidateSelectionError> {
    let mut ordered = eligible.to_vec();
    ordered.sort_by(|left, right| match control {
        PredicateDiscoveryControl::EvidenceWithoutDescription => right
            .evidence_margin()
            .cmp(&left.evidence_margin())
            .then_with(|| left.counterexamples().cmp(&right.counterexamples()))
            .then_with(|| right.support().cmp(&left.support()))
            .then_with(|| left.unknown().cmp(&right.unknown()))
            .then_with(|| left.candidate_key().cmp(right.candidate_key())),
        PredicateDiscoveryControl::CounterexampleBlind => right
            .support()
            .cmp(&left.support())
            .then_with(|| left.unknown().cmp(&right.unknown()))
            .then_with(|| {
                left.canonical_description_bytes()
                    .cmp(&right.canonical_description_bytes())
            })
            .then_with(|| left.candidate_key().cmp(right.candidate_key())),
        PredicateDiscoveryControl::DescriptionOnly => left
            .canonical_description_bytes()
            .cmp(&right.canonical_description_bytes())
            .then_with(|| left.candidate_key().cmp(right.candidate_key())),
        PredicateDiscoveryControl::CanonicalIdentityNull => {
            left.candidate_key().cmp(right.candidate_key())
        }
    });

    let candidates = ordered
        .into_iter()
        .enumerate()
        .map(|(index, score)| {
            let rank =
                u32::try_from(index + 1).map_err(|_| CandidateSelectionError::RankOverflow)?;
            Ok(ControlledCandidateRank { rank, score })
        })
        .collect::<Result<Vec<_>, CandidateSelectionError>>()?;

    Ok(PredicateDiscoveryControlReport {
        control,
        candidates,
    })
}

/// Extract exact candidate membership from a slice-15 selection for paired
/// control checks without assigning any quality semantics to the order.
#[must_use]
pub fn selected_candidate_keys(selected: &[SelectedCandidate]) -> Vec<&str> {
    let mut keys = selected
        .iter()
        .map(|candidate| candidate.score().candidate_key())
        .collect::<Vec<_>>();
    keys.sort_unstable();
    keys
}

#[cfg(test)]
mod tests {
    use super::{
        PredicateDiscoveryControl, run_predicate_discovery_controls, selected_candidate_keys,
    };
    use crate::experimental::tdi2_candidate_evidence::account_candidate_evidence;
    use crate::experimental::tdi2_candidate_identity::CanonicalPredicateCandidate;
    use crate::experimental::tdi2_candidate_scoring::score_candidate_catalogue;
    use crate::experimental::tdi2_candidate_selection::select_candidates_v1;
    use crate::experimental::tdi2_induction_input::InductionBatch;
    use crate::experimental::tdi2_induction_split::{DEVELOPMENT_START, InductionDomain};
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

    fn batch(episodes: Vec<ExperienceEpisode>) -> InductionBatch {
        let graphs = episodes
            .iter()
            .map(|episode| {
                ObservationGraph::new(episode.id(), Vec::new(), Vec::new()).expect("graph")
            })
            .collect();
        InductionBatch::new(InductionDomain::Development, episodes, graphs).expect("batch")
    }

    fn scores()
    -> Vec<crate::experimental::tdi2_candidate_scoring::CandidateDescriptionEvidenceScore> {
        let source = batch(vec![episode(DEVELOPMENT_START, &[(0, vec![1.0, 2.0])])]);
        let candidates = generate_numeric_threshold_candidates(&source)
            .expect("thresholds")
            .iter()
            .map(CanonicalPredicateCandidate::from_threshold)
            .collect::<Vec<_>>();
        let observed = batch(vec![episode(
            DEVELOPMENT_START + 1,
            &[
                (0, vec![0.5, 2.0]),
                (1, vec![1.5, 1.5]),
                (2, vec![2.5, 0.5]),
            ],
        )]);
        let evidence = account_candidate_evidence(&observed, &candidates).expect("evidence");
        score_candidate_catalogue(&evidence).expect("scores")
    }

    #[test]
    fn controls_are_frozen_and_contain_no_forbidden_information_surface() {
        let reports = run_predicate_discovery_controls(&scores()).expect("controls");
        assert_eq!(reports.len(), 4);
        assert_eq!(
            reports
                .iter()
                .map(|report| report.control())
                .collect::<Vec<_>>(),
            vec![
                PredicateDiscoveryControl::EvidenceWithoutDescription,
                PredicateDiscoveryControl::CounterexampleBlind,
                PredicateDiscoveryControl::DescriptionOnly,
                PredicateDiscoveryControl::CanonicalIdentityNull,
            ]
        );
        for report in reports {
            let policy = report.policy_record();
            assert!(policy.contains("eligibility=frozen_v1"));
            assert!(!policy.contains("latency"));
            assert!(!policy.contains("expected_label"));
            assert!(!policy.contains("primary_holdout"));
            assert!(!policy.contains("outcome"));
        }
    }

    #[test]
    fn every_control_preserves_the_exact_frozen_candidate_membership() {
        let scores = scores();
        let selected = select_candidates_v1(&scores).expect("frozen policy");
        let expected = selected_candidate_keys(&selected);
        let reports = run_predicate_discovery_controls(&scores).expect("controls");
        for report in reports {
            let mut actual = report
                .candidates()
                .iter()
                .map(|ranked| ranked.score().candidate_key())
                .collect::<Vec<_>>();
            actual.sort_unstable();
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn control_rankings_are_deterministic_under_score_input_permutation() {
        let forward_scores = scores();
        let mut reverse_scores = forward_scores.clone();
        reverse_scores.reverse();
        let forward = run_predicate_discovery_controls(&forward_scores).expect("forward");
        let reverse = run_predicate_discovery_controls(&reverse_scores).expect("reverse");
        assert_eq!(forward, reverse);
    }

    #[test]
    fn canonical_identity_null_is_exact_lexicographic_order() {
        let reports = run_predicate_discovery_controls(&scores()).expect("controls");
        let null = reports
            .iter()
            .find(|report| report.control() == PredicateDiscoveryControl::CanonicalIdentityNull)
            .expect("null control");
        let keys = null
            .candidates()
            .iter()
            .map(|ranked| ranked.score().candidate_key())
            .collect::<Vec<_>>();
        assert!(keys.windows(2).all(|pair| pair[0] <= pair[1]));
        assert!(
            null.candidates()
                .iter()
                .enumerate()
                .all(|(index, ranked)| ranked.rank() == u32::try_from(index + 1).unwrap())
        );
    }
}
