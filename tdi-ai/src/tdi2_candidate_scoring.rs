//! Deterministic description/evidence score components for TDI-2.2 candidates.
//!
//! Stage-B slice 14 intentionally does **not** freeze a selection policy.  It
//! exposes reproducible score components only: exact canonical-description
//! length, observed support/counterexamples, explicit unknown evidence, known
//! evidence mass, and a signed support-minus-counterexample margin.  Latency,
//! evaluator labels, outcomes and protected material are not inputs.
//!
//! The canonical identity byte length is a versioned description-length proxy,
//! not a claim of an information-theoretically minimal code.  Slice 15 owns any
//! later frozen ordering/selection rule and must keep that distinction explicit.

use super::tdi2_candidate_evidence::{CandidateEvidence, CandidateEvidenceCounts};
use super::tdi2_induction_split::InductionDomain;

/// Versioned schema for score-component evidence.
pub const CANDIDATE_SCORE_SCHEMA: &str = "tdi2.2-candidate-description-evidence-v1";

/// Reproducible score components for one already-accounted candidate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateDescriptionEvidenceScore {
    domain: InductionDomain,
    evaluation_batch_record: String,
    candidate_key: String,
    canonical_description_bytes: u32,
    opportunities: u64,
    known_observations: u64,
    support: u64,
    counterexamples: u64,
    unknown: u64,
    evidence_margin: i128,
    source_evidence_record: String,
}

impl CandidateDescriptionEvidenceScore {
    /// Exact observation population shared by all scores in a matched ranking.
    #[must_use]
    pub fn evaluation_batch_record(&self) -> &str {
        &self.evaluation_batch_record
    }

    #[must_use]
    pub const fn domain(&self) -> InductionDomain {
        self.domain
    }

    #[must_use]
    pub fn candidate_key(&self) -> &str {
        &self.candidate_key
    }

    /// UTF-8 byte length of the versioned canonical candidate identity.
    ///
    /// This is a deterministic complexity proxy only.  It is not a claim that
    /// the identity uses a minimum prefix code or achieves Kolmogorov/MDL
    /// optimality.
    #[must_use]
    pub const fn canonical_description_bytes(&self) -> u32 {
        self.canonical_description_bytes
    }

    #[must_use]
    pub const fn opportunities(&self) -> u64 {
        self.opportunities
    }

    #[must_use]
    pub const fn known_observations(&self) -> u64 {
        self.known_observations
    }

    #[must_use]
    pub const fn support(&self) -> u64 {
        self.support
    }

    #[must_use]
    pub const fn counterexamples(&self) -> u64 {
        self.counterexamples
    }

    #[must_use]
    pub const fn unknown(&self) -> u64 {
        self.unknown
    }

    /// Signed observation-only margin `support - counterexamples`.
    ///
    /// Unknown evidence is intentionally not silently converted to either
    /// support or a counterexample.  This field is not a task-correctness score.
    #[must_use]
    pub const fn evidence_margin(&self) -> i128 {
        self.evidence_margin
    }

    /// Exact canonical evidence record from which this score was derived.
    #[must_use]
    pub fn source_evidence_record(&self) -> &str {
        &self.source_evidence_record
    }

    /// Stable machine-readable score-component record.
    ///
    /// No total ordering or acceptance threshold is implied by this record.
    #[must_use]
    pub fn canonical_record(&self) -> String {
        let domain = match self.domain {
            InductionDomain::Development => "development",
            InductionDomain::Validation => "validation",
        };
        let mut source_hex = String::with_capacity(self.source_evidence_record.len() * 2);
        append_hex(&mut source_hex, self.source_evidence_record.as_bytes());
        format!(
            "{CANDIDATE_SCORE_SCHEMA};domain={domain};candidate={};description_bytes={};opportunities={};known={};support={};counterexamples={};unknown={};evidence_margin={};source_evidence_hex={source_hex}",
            self.candidate_key,
            self.canonical_description_bytes,
            self.opportunities,
            self.known_observations,
            self.support,
            self.counterexamples,
            self.unknown,
            self.evidence_margin,
        )
    }
}

/// Fail-closed score construction errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CandidateScoringError {
    DescriptionLengthOverflow,
    EvidenceCountOverflow,
    InconsistentEvidenceCounts,
}

/// Derive description/evidence score components from one validated evidence record.
pub fn score_candidate_evidence(
    evidence: &CandidateEvidence,
) -> Result<CandidateDescriptionEvidenceScore, CandidateScoringError> {
    let key = evidence.candidate().identity().canonical_key();
    let canonical_description_bytes =
        u32::try_from(key.len()).map_err(|_| CandidateScoringError::DescriptionLengthOverflow)?;
    let counts = evidence.counts();
    validate_counts(counts)?;
    let known_observations = counts
        .support()
        .checked_add(counts.counterexamples())
        .ok_or(CandidateScoringError::EvidenceCountOverflow)?;
    let evidence_margin = i128::from(counts.support()) - i128::from(counts.counterexamples());

    Ok(CandidateDescriptionEvidenceScore {
        domain: evidence.domain(),
        evaluation_batch_record: evidence.evaluation_batch_record().to_owned(),
        candidate_key: key,
        canonical_description_bytes,
        opportunities: counts.opportunities(),
        known_observations,
        support: counts.support(),
        counterexamples: counts.counterexamples(),
        unknown: counts.unknown(),
        evidence_margin,
        source_evidence_record: evidence.canonical_record(),
    })
}

/// Score a complete evidence catalogue without changing its deterministic order.
pub fn score_candidate_catalogue(
    evidence: &[CandidateEvidence],
) -> Result<Vec<CandidateDescriptionEvidenceScore>, CandidateScoringError> {
    evidence.iter().map(score_candidate_evidence).collect()
}

fn validate_counts(counts: CandidateEvidenceCounts) -> Result<(), CandidateScoringError> {
    let accounted = counts
        .support()
        .checked_add(counts.counterexamples())
        .and_then(|value| value.checked_add(counts.unknown()))
        .ok_or(CandidateScoringError::EvidenceCountOverflow)?;
    if accounted != counts.opportunities() {
        return Err(CandidateScoringError::InconsistentEvidenceCounts);
    }
    Ok(())
}

fn append_hex(output: &mut String, bytes: &[u8]) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in bytes {
        output.push(char::from(HEX[(byte >> 4) as usize]));
        output.push(char::from(HEX[(byte & 0x0f) as usize]));
    }
}

impl core::fmt::Display for CandidateScoringError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::DescriptionLengthOverflow => {
                formatter.write_str("candidate canonical description length exceeds u32")
            }
            Self::EvidenceCountOverflow => formatter.write_str("candidate evidence count overflow"),
            Self::InconsistentEvidenceCounts => formatter.write_str(
                "candidate evidence opportunities do not equal support + counterexamples + unknown",
            ),
        }
    }
}

impl std::error::Error for CandidateScoringError {}

#[cfg(test)]
mod tests {
    use super::{score_candidate_catalogue, score_candidate_evidence};
    use crate::experimental::tdi2_candidate_evidence::account_candidate_evidence;
    use crate::experimental::tdi2_candidate_identity::CanonicalPredicateCandidate;
    use crate::experimental::tdi2_induction_input::InductionBatch;
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

    #[test]
    fn score_keeps_unknown_explicit_and_uses_no_task_label() {
        let source = batch(vec![episode(DEVELOPMENT_START, &[(0, vec![1.0])])]);
        let threshold = generate_numeric_threshold_candidates(&source)
            .expect("thresholds")
            .into_iter()
            .find(|candidate| candidate.direction() == ThresholdDirection::LessEqual)
            .expect("<= candidate");
        let canonical = CanonicalPredicateCandidate::from_threshold(&threshold);
        let observed = batch(vec![episode(
            DEVELOPMENT_START + 1,
            &[(0, vec![0.5]), (1, vec![2.0]), (2, Vec::new())],
        )]);
        let evidence = account_candidate_evidence(&observed, &[canonical])
            .expect("evidence")
            .pop()
            .expect("one candidate");
        let score = score_candidate_evidence(&evidence).expect("score");

        assert_eq!(score.opportunities(), 3);
        assert_eq!(score.known_observations(), 2);
        assert_eq!(score.support(), 1);
        assert_eq!(score.counterexamples(), 1);
        assert_eq!(score.unknown(), 1);
        assert_eq!(score.evidence_margin(), 0);
        assert_eq!(score.source_evidence_record(), evidence.canonical_record());
        let record = score.canonical_record();
        assert!(record.contains("description_bytes="));
        assert!(record.contains("unknown=1"));
        assert!(record.contains("evidence_margin=0"));
        assert!(!record.contains("latency"));
        assert!(!record.contains("expected_label"));
    }

    #[test]
    fn canonical_description_length_is_identity_bound_not_runtime_cost() {
        let source = batch(vec![episode(DEVELOPMENT_START, &[(0, vec![1.0, 2.0])])]);
        let threshold = generate_numeric_threshold_candidates(&source)
            .expect("thresholds")
            .into_iter()
            .find(|candidate| candidate.direction() == ThresholdDirection::GreaterEqual)
            .expect(">= candidate");
        let relation = generate_pairwise_relation_candidates(&source)
            .expect("pairwise")
            .into_iter()
            .find(|candidate| candidate.operator() == PairwiseOperator::Less)
            .expect("less candidate");
        let candidates = [
            CanonicalPredicateCandidate::from_threshold(&threshold),
            CanonicalPredicateCandidate::from_pairwise(&relation),
        ];
        let evidence = account_candidate_evidence(&source, &candidates).expect("evidence");
        let scores = score_candidate_catalogue(&evidence).expect("scores");

        assert_eq!(scores.len(), 2);
        for score in &scores {
            assert_eq!(
                score.canonical_description_bytes() as usize,
                score.candidate_key().len()
            );
        }
        assert_ne!(
            scores[0].canonical_description_bytes(),
            scores[1].canonical_description_bytes()
        );
    }

    #[test]
    fn catalogue_scoring_preserves_exact_candidate_order() {
        let source = batch(vec![episode(DEVELOPMENT_START, &[(0, vec![1.0, 2.0])])]);
        let raw = generate_pairwise_relation_candidates(&source).expect("pairwise");
        let candidates = raw
            .iter()
            .map(CanonicalPredicateCandidate::from_pairwise)
            .collect::<Vec<_>>();
        let evidence = account_candidate_evidence(&source, &candidates).expect("evidence");
        let scores = score_candidate_catalogue(&evidence).expect("scores");
        let evidence_keys = evidence
            .iter()
            .map(|entry| entry.candidate().identity().canonical_key())
            .collect::<Vec<_>>();
        let score_keys = scores
            .iter()
            .map(|score| score.candidate_key().to_owned())
            .collect::<Vec<_>>();
        assert_eq!(score_keys, evidence_keys);
    }
}
