//! Label-free support/counterexample accounting for TDI-2.2 predicate candidates.
//!
//! This slice deliberately separates candidate-generation provenance from the
//! batch on which a candidate is accounted. That permits later Development →
//! Validation evaluation without relabelling the candidate's origin. A
//! `support` is only an observed Boolean `true` for the candidate predicate; a
//! `counterexample` is an observed `false`; missing/inapplicable numeric inputs
//! remain explicit `unknown`. None of these counts is an expected task label.

use super::tdi2_candidate_identity::{
    CanonicalPredicateCandidate, MAX_CANONICAL_CANDIDATES, PredicateCandidateIdentity,
};
use super::tdi2_induction_input::InductionBatch;
use super::tdi2_induction_provenance::canonical_batch_record;
use super::tdi2_induction_split::InductionDomain;
use super::tdi2_numeric_thresholds::ThresholdDirection;
use super::tdi2_pairwise_relations::PairwiseOperator;
use super::tdi2_temporal_delta::TemporalDeltaOperator;

/// Versioned schema for deterministic candidate evidence records.
pub const CANDIDATE_EVIDENCE_SCHEMA: &str = "tdi2.2-candidate-evidence-v1";

/// Exact label-free accounting for one candidate on one observation batch.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CandidateEvidenceCounts {
    opportunities: u64,
    support: u64,
    counterexamples: u64,
    unknown: u64,
    episodes_with_support: u64,
    episodes_with_counterexample: u64,
    episodes_with_unknown: u64,
}

impl CandidateEvidenceCounts {
    #[must_use]
    pub const fn opportunities(self) -> u64 {
        self.opportunities
    }

    #[must_use]
    pub const fn support(self) -> u64 {
        self.support
    }

    #[must_use]
    pub const fn counterexamples(self) -> u64 {
        self.counterexamples
    }

    #[must_use]
    pub const fn unknown(self) -> u64 {
        self.unknown
    }

    #[must_use]
    pub const fn episodes_with_support(self) -> u64 {
        self.episodes_with_support
    }

    #[must_use]
    pub const fn episodes_with_counterexample(self) -> u64 {
        self.episodes_with_counterexample
    }

    #[must_use]
    pub const fn episodes_with_unknown(self) -> u64 {
        self.episodes_with_unknown
    }

    fn observe(&mut self, verdict: Option<bool>) -> Result<(), CandidateEvidenceError> {
        self.opportunities = self
            .opportunities
            .checked_add(1)
            .ok_or(CandidateEvidenceError::CountOverflow)?;
        let slot = match verdict {
            Some(true) => &mut self.support,
            Some(false) => &mut self.counterexamples,
            None => &mut self.unknown,
        };
        *slot = slot
            .checked_add(1)
            .ok_or(CandidateEvidenceError::CountOverflow)?;
        Ok(())
    }

    fn observe_episode(
        &mut self,
        support: bool,
        counterexample: bool,
        unknown: bool,
    ) -> Result<(), CandidateEvidenceError> {
        if support {
            self.episodes_with_support = self
                .episodes_with_support
                .checked_add(1)
                .ok_or(CandidateEvidenceError::CountOverflow)?;
        }
        if counterexample {
            self.episodes_with_counterexample = self
                .episodes_with_counterexample
                .checked_add(1)
                .ok_or(CandidateEvidenceError::CountOverflow)?;
        }
        if unknown {
            self.episodes_with_unknown = self
                .episodes_with_unknown
                .checked_add(1)
                .ok_or(CandidateEvidenceError::CountOverflow)?;
        }
        Ok(())
    }
}

/// One canonical candidate plus its exact observation-only evidence counts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateEvidence {
    domain: InductionDomain,
    evaluation_batch_record: String,
    candidate: CanonicalPredicateCandidate,
    counts: CandidateEvidenceCounts,
}

impl CandidateEvidence {
    #[must_use]
    pub const fn domain(&self) -> InductionDomain {
        self.domain
    }

    /// Exact canonical observation-only batch on which these counts were computed.
    #[must_use]
    pub fn evaluation_batch_record(&self) -> &str {
        &self.evaluation_batch_record
    }

    #[must_use]
    pub const fn candidate(&self) -> &CanonicalPredicateCandidate {
        &self.candidate
    }

    #[must_use]
    pub const fn counts(&self) -> CandidateEvidenceCounts {
        self.counts
    }

    /// Deterministic machine-readable record retaining candidate provenance.
    ///
    /// This record is explanatory evidence only. It is not a task-label record,
    /// selection decision, novelty fingerprint, or scientific verdict.
    #[must_use]
    pub fn canonical_record(&self) -> String {
        let domain = match self.domain {
            InductionDomain::Development => "development",
            InductionDomain::Validation => "validation",
        };
        let mut sources = String::new();
        for (index, source) in self.candidate.provenance().sources().iter().enumerate() {
            if index != 0 {
                sources.push(',');
            }
            sources.push_str(&format!(
                "{}:{}",
                source.episode().raw(),
                source.frame_ordinal()
            ));
        }
        let mut batch_hex = String::with_capacity(self.evaluation_batch_record.len() * 2);
        append_hex(&mut batch_hex, self.evaluation_batch_record.as_bytes());
        format!(
            "{CANDIDATE_EVIDENCE_SCHEMA};domain={domain};evaluation_batch_hex={batch_hex};candidate={};sources={sources};opportunities={};support={};counterexamples={};unknown={};episodes_with_support={};episodes_with_counterexample={};episodes_with_unknown={}",
            self.candidate.identity().canonical_key(),
            self.counts.opportunities,
            self.counts.support,
            self.counts.counterexamples,
            self.counts.unknown,
            self.counts.episodes_with_support,
            self.counts.episodes_with_counterexample,
            self.counts.episodes_with_unknown,
        )
    }
}

/// Fail-closed accounting errors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CandidateEvidenceError {
    CandidateLimitExceeded { actual: usize, maximum: usize },
    DuplicateCandidate { canonical_key: String },
    CountOverflow,
}

/// Account support, counterexamples and unknowns for a canonical candidate set.
///
/// Candidates are sorted by exact identity before evaluation. Duplicate
/// identities are rejected rather than silently double-weighted. Scalar and
/// pairwise candidates use each frame as one opportunity; temporal candidates
/// use each adjacent frame transition as one opportunity. An episode can appear
/// in more than one episode-level category when it contains mixed evidence.
pub fn account_candidate_evidence(
    batch: &InductionBatch,
    candidates: &[CanonicalPredicateCandidate],
) -> Result<Vec<CandidateEvidence>, CandidateEvidenceError> {
    if candidates.len() > MAX_CANONICAL_CANDIDATES {
        return Err(CandidateEvidenceError::CandidateLimitExceeded {
            actual: candidates.len(),
            maximum: MAX_CANONICAL_CANDIDATES,
        });
    }
    let evaluation_batch_record = canonical_batch_record(batch);
    let mut ordered = candidates.iter().collect::<Vec<_>>();
    ordered.sort_unstable_by_key(|candidate| candidate.identity());
    if let Some(candidate) = ordered
        .windows(2)
        .find_map(|pair| (pair[0].identity() == pair[1].identity()).then_some(pair[0]))
    {
        return Err(CandidateEvidenceError::DuplicateCandidate {
            canonical_key: candidate.identity().canonical_key(),
        });
    }

    ordered
        .into_iter()
        .map(|candidate| {
            let counts = account_one(batch, candidate.identity())?;
            Ok(CandidateEvidence {
                domain: batch.domain(),
                evaluation_batch_record: evaluation_batch_record.clone(),
                candidate: candidate.clone(),
                counts,
            })
        })
        .collect()
}

fn account_one(
    batch: &InductionBatch,
    identity: &PredicateCandidateIdentity,
) -> Result<CandidateEvidenceCounts, CandidateEvidenceError> {
    let mut counts = CandidateEvidenceCounts::default();

    for episode in batch.episodes() {
        let mut episode_support = false;
        let mut episode_counterexample = false;
        let mut episode_unknown = false;

        match identity {
            PredicateCandidateIdentity::TemporalDelta { .. } => {
                for frames in episode.frames().windows(2) {
                    let verdict = evaluate_transition(
                        identity,
                        frames[0].numeric().values(),
                        frames[1].numeric().values(),
                    );
                    classify_episode_verdict(
                        verdict,
                        &mut episode_support,
                        &mut episode_counterexample,
                        &mut episode_unknown,
                    );
                    counts.observe(verdict)?;
                }
            }
            _ => {
                for frame in episode.frames() {
                    let verdict = evaluate_frame(identity, frame.numeric().values());
                    classify_episode_verdict(
                        verdict,
                        &mut episode_support,
                        &mut episode_counterexample,
                        &mut episode_unknown,
                    );
                    counts.observe(verdict)?;
                }
            }
        }

        counts.observe_episode(episode_support, episode_counterexample, episode_unknown)?;
    }

    debug_assert_eq!(
        counts.opportunities,
        counts.support + counts.counterexamples + counts.unknown
    );
    Ok(counts)
}

fn classify_episode_verdict(
    verdict: Option<bool>,
    support: &mut bool,
    counterexample: &mut bool,
    unknown: &mut bool,
) {
    match verdict {
        Some(true) => *support = true,
        Some(false) => *counterexample = true,
        None => *unknown = true,
    }
}

fn evaluate_frame(identity: &PredicateCandidateIdentity, values: &[f64]) -> Option<bool> {
    match identity {
        PredicateCandidateIdentity::ScalarThreshold {
            feature_index,
            threshold_bits,
            direction,
        } => {
            let value = *values.get(*feature_index as usize)?;
            let threshold = f64::from_bits(*threshold_bits);
            if !value.is_finite() || !threshold.is_finite() {
                return None;
            }
            Some(match direction {
                ThresholdDirection::LessEqual => value <= threshold,
                ThresholdDirection::GreaterEqual => value >= threshold,
            })
        }
        PredicateCandidateIdentity::PairwiseRelation {
            left_feature,
            right_feature,
            operator,
        } => {
            let left = *values.get(*left_feature as usize)?;
            let right = *values.get(*right_feature as usize)?;
            if !left.is_finite() || !right.is_finite() {
                return None;
            }
            Some(match operator {
                PairwiseOperator::Less => left < right,
                PairwiseOperator::Equal => left == right,
                PairwiseOperator::Greater => left > right,
            })
        }
        PredicateCandidateIdentity::TemporalDelta { .. } => None,
    }
}

fn evaluate_transition(
    identity: &PredicateCandidateIdentity,
    previous: &[f64],
    current: &[f64],
) -> Option<bool> {
    let PredicateCandidateIdentity::TemporalDelta {
        feature_index,
        operator,
    } = identity
    else {
        return None;
    };
    let previous = *previous.get(*feature_index as usize)?;
    let current = *current.get(*feature_index as usize)?;
    if !previous.is_finite() || !current.is_finite() {
        return None;
    }
    Some(match operator {
        TemporalDeltaOperator::Falling => current < previous,
        TemporalDeltaOperator::Stable => current == previous,
        TemporalDeltaOperator::Rising => current > previous,
    })
}

fn append_hex(output: &mut String, bytes: &[u8]) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in bytes {
        output.push(char::from(HEX[(byte >> 4) as usize]));
        output.push(char::from(HEX[(byte & 0x0f) as usize]));
    }
}

impl core::fmt::Display for CandidateEvidenceError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::CandidateLimitExceeded { actual, maximum } => write!(
                formatter,
                "candidate evidence count {actual} exceeds bounded maximum {maximum}"
            ),
            Self::DuplicateCandidate { canonical_key } => {
                write!(
                    formatter,
                    "candidate identity is duplicated: {canonical_key}"
                )
            }
            Self::CountOverflow => formatter.write_str("candidate evidence count overflow"),
        }
    }
}

impl std::error::Error for CandidateEvidenceError {}

#[cfg(test)]
mod tests {
    use super::{CandidateEvidenceError, account_candidate_evidence};
    use crate::experimental::tdi2_candidate_identity::{
        CanonicalPredicateCandidate, MAX_CANONICAL_CANDIDATES, deduplicate_candidates,
    };
    use crate::experimental::tdi2_induction_input::InductionBatch;
    use crate::experimental::tdi2_induction_provenance::canonical_batch_record;
    use crate::experimental::tdi2_induction_split::{DEVELOPMENT_START, InductionDomain};
    use crate::experimental::tdi2_intuition::{BooleanState, NumericState};
    use crate::experimental::tdi2_numeric_thresholds::{
        ThresholdDirection, generate_numeric_threshold_candidates,
    };
    use crate::experimental::tdi2_observation_graph::ObservationGraph;
    use crate::experimental::tdi2_pairwise_relations::{
        PairwiseOperator, generate_pairwise_relation_candidates,
    };
    use crate::experimental::tdi2_template_induction::{
        EpisodeId, ExperienceEpisode, ObservationFrame,
    };
    use crate::experimental::tdi2_temporal_delta::{
        TemporalDeltaOperator, generate_temporal_delta_candidates,
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

    fn graph(id: u64) -> ObservationGraph {
        ObservationGraph::new(EpisodeId::new(id), Vec::new(), Vec::new()).expect("graph")
    }

    fn batch(episodes: Vec<ExperienceEpisode>) -> InductionBatch {
        let graphs = episodes
            .iter()
            .map(|episode| graph(episode.id().raw()))
            .collect();
        InductionBatch::new(InductionDomain::Development, episodes, graphs).expect("batch")
    }

    fn canonical_threshold() -> CanonicalPredicateCandidate {
        let source = batch(vec![episode(DEVELOPMENT_START, &[(0, vec![1.0])])]);
        let raw = generate_numeric_threshold_candidates(&source).expect("thresholds");
        let candidate = raw
            .iter()
            .find(|candidate| candidate.direction() == ThresholdDirection::LessEqual)
            .expect("<= candidate");
        CanonicalPredicateCandidate::from_threshold(candidate)
    }

    #[test]
    fn scalar_accounting_separates_support_counterexamples_and_unknowns() {
        let eval = batch(vec![
            episode(DEVELOPMENT_START + 1, &[(0, vec![0.5])]),
            episode(DEVELOPMENT_START + 2, &[(0, vec![2.0])]),
            episode(DEVELOPMENT_START + 3, &[(0, Vec::new())]),
        ]);
        let evidence = account_candidate_evidence(&eval, &[canonical_threshold()])
            .expect("accounting")
            .pop()
            .expect("one candidate");
        let counts = evidence.counts();
        assert_eq!(counts.opportunities(), 3);
        assert_eq!(counts.support(), 1);
        assert_eq!(counts.counterexamples(), 1);
        assert_eq!(counts.unknown(), 1);
        assert_eq!(counts.episodes_with_support(), 1);
        assert_eq!(counts.episodes_with_counterexample(), 1);
        assert_eq!(counts.episodes_with_unknown(), 1);
        assert_eq!(
            evidence.evaluation_batch_record(),
            canonical_batch_record(&eval)
        );
        let record = evidence.canonical_record();
        assert!(record.contains("evaluation_batch_hex="));
        assert!(record.contains("support=1"));
        assert!(record.contains("counterexamples=1"));
        assert!(!record.contains("expected_label"));

        let same_counts_other_batch = batch(vec![
            episode(DEVELOPMENT_START + 4, &[(0, vec![0.25])]),
            episode(DEVELOPMENT_START + 5, &[(0, vec![3.0])]),
            episode(DEVELOPMENT_START + 6, &[(0, Vec::new())]),
        ]);
        let other = account_candidate_evidence(&same_counts_other_batch, &[canonical_threshold()])
            .expect("other accounting")[0]
            .canonical_record();
        assert_ne!(record, other);
    }

    #[test]
    fn pairwise_accounting_uses_each_frame_as_one_opportunity() {
        let source = batch(vec![episode(DEVELOPMENT_START, &[(0, vec![1.0, 2.0])])]);
        let raw = generate_pairwise_relation_candidates(&source).expect("pairwise");
        let less = raw
            .iter()
            .find(|candidate| candidate.operator() == PairwiseOperator::Less)
            .expect("less");
        let canonical = CanonicalPredicateCandidate::from_pairwise(less);
        let eval = batch(vec![episode(
            DEVELOPMENT_START + 1,
            &[(0, vec![1.0, 2.0]), (1, vec![2.0, 1.0]), (2, vec![1.0])],
        )]);
        let counts =
            account_candidate_evidence(&eval, &[canonical]).expect("accounting")[0].counts();
        assert_eq!(counts.opportunities(), 3);
        assert_eq!(counts.support(), 1);
        assert_eq!(counts.counterexamples(), 1);
        assert_eq!(counts.unknown(), 1);
        assert_eq!(counts.episodes_with_support(), 1);
        assert_eq!(counts.episodes_with_counterexample(), 1);
        assert_eq!(counts.episodes_with_unknown(), 1);
    }

    #[test]
    fn temporal_accounting_uses_adjacent_transitions_only() {
        let source = batch(vec![episode(
            DEVELOPMENT_START,
            &[(0, vec![1.0]), (1, vec![2.0])],
        )]);
        let raw = generate_temporal_delta_candidates(&source).expect("temporal");
        let rising = raw
            .iter()
            .find(|candidate| candidate.operator() == TemporalDeltaOperator::Rising)
            .expect("rising");
        let canonical = CanonicalPredicateCandidate::from_temporal(rising);
        let eval = batch(vec![episode(
            DEVELOPMENT_START + 1,
            &[
                (0, vec![1.0]),
                (1, vec![2.0]),
                (2, vec![1.0]),
                (3, Vec::new()),
            ],
        )]);
        let counts =
            account_candidate_evidence(&eval, &[canonical]).expect("accounting")[0].counts();
        assert_eq!(counts.opportunities(), 3);
        assert_eq!(counts.support(), 1);
        assert_eq!(counts.counterexamples(), 1);
        assert_eq!(counts.unknown(), 1);
    }

    #[test]
    fn accounting_rejects_candidate_catalogues_above_the_declared_bound() {
        let eval = batch(vec![episode(DEVELOPMENT_START + 1, &[(0, vec![0.5])])]);
        let candidates = vec![canonical_threshold(); MAX_CANONICAL_CANDIDATES + 1];
        let error = account_candidate_evidence(&eval, &candidates).expect_err("bounded catalogue");
        assert_eq!(
            error,
            CandidateEvidenceError::CandidateLimitExceeded {
                actual: MAX_CANONICAL_CANDIDATES + 1,
                maximum: MAX_CANONICAL_CANDIDATES,
            }
        );
    }

    #[test]
    fn accounting_is_identity_sorted_and_rejects_duplicate_weighting() {
        let source = batch(vec![episode(
            DEVELOPMENT_START,
            &[(0, vec![1.0, 2.0]), (1, vec![2.0, 1.0])],
        )]);
        let thresholds = generate_numeric_threshold_candidates(&source).expect("thresholds");
        let pairwise = generate_pairwise_relation_candidates(&source).expect("pairwise");
        let temporal = generate_temporal_delta_candidates(&source).expect("temporal");
        let mut records = Vec::new();
        records.extend(
            thresholds
                .iter()
                .map(CanonicalPredicateCandidate::from_threshold),
        );
        records.extend(
            pairwise
                .iter()
                .map(CanonicalPredicateCandidate::from_pairwise),
        );
        records.extend(
            temporal
                .iter()
                .map(CanonicalPredicateCandidate::from_temporal),
        );
        let catalog = deduplicate_candidates(&source, records).expect("catalog");
        let mut reversed = catalog.clone();
        reversed.reverse();
        let forward = account_candidate_evidence(&source, &catalog).expect("forward");
        let reverse = account_candidate_evidence(&source, &reversed).expect("reverse");
        assert_eq!(forward, reverse);

        let duplicate = vec![catalog[0].clone(), catalog[0].clone()];
        let error = account_candidate_evidence(&source, &duplicate).expect_err("duplicate");
        assert!(matches!(
            error,
            CandidateEvidenceError::DuplicateCandidate { .. }
        ));
    }
}
