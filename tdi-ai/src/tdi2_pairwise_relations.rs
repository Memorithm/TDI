//! Deterministic pairwise numeric predicate candidates for TDI-2.2.
//!
//! Generation consumes only observable numeric values from `InductionBatch`.
//! Expected labels, outcomes and evaluator state are outside this surface.

use std::collections::BTreeMap;

use super::tdi2_induction_input::InductionBatch;
use super::tdi2_predicate_candidates::{
    CandidateProvenance, CandidateProvenanceError, CandidateSource, PredicateCandidateFamily,
};

/// Maximum numeric feature width accepted by this bounded generator.
pub const MAX_PAIRWISE_FEATURES: usize = 64;
/// Maximum canonical pairwise candidates emitted by one batch.
pub const MAX_PAIRWISE_CANDIDATES: usize = 8_192;
/// Maximum raw provenance contributions accepted before candidate accumulation.
///
/// This independently bounds repeated-frame growth even when the number of
/// distinct pair/operator identities remains small.
pub const MAX_PAIRWISE_SOURCE_REFERENCES: usize = 262_144;

/// Boolean comparison represented by one pairwise candidate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PairwiseOperator {
    Less,
    Equal,
    Greater,
}

impl PairwiseOperator {
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Less => "lt",
            Self::Equal => "eq",
            Self::Greater => "gt",
        }
    }
}

/// Canonical comparison between two distinct numeric feature positions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PairwiseRelationCandidate {
    left_feature: u32,
    right_feature: u32,
    operator: PairwiseOperator,
    provenance: CandidateProvenance,
}

impl PairwiseRelationCandidate {
    #[must_use]
    pub const fn left_feature(&self) -> u32 {
        self.left_feature
    }

    #[must_use]
    pub const fn right_feature(&self) -> u32 {
        self.right_feature
    }

    #[must_use]
    pub const fn operator(&self) -> PairwiseOperator {
        self.operator
    }

    #[must_use]
    pub const fn provenance(&self) -> &CandidateProvenance {
        &self.provenance
    }

    /// Evaluate against one observable numeric vector.
    #[must_use]
    pub fn evaluate(&self, values: &[f64]) -> Option<bool> {
        let left = *values.get(self.left_feature as usize)?;
        let right = *values.get(self.right_feature as usize)?;
        if !left.is_finite() || !right.is_finite() {
            return None;
        }
        Some(match self.operator {
            PairwiseOperator::Less => left < right,
            PairwiseOperator::Equal => left == right,
            PairwiseOperator::Greater => left > right,
        })
    }
}

/// Fail-closed bounded-generation errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PairwiseRelationError {
    FeatureWidthExceeded { actual: usize, maximum: usize },
    CandidateLimitExceeded { maximum: usize },
    SourceReferenceLimitExceeded { maximum: usize },
    Provenance(CandidateProvenanceError),
}

impl From<CandidateProvenanceError> for PairwiseRelationError {
    fn from(error: CandidateProvenanceError) -> Self {
        Self::Provenance(error)
    }
}
/// Generate `<`, `==`, and `>` predicates for every observed feature pair.
pub fn generate_pairwise_relation_candidates(
    batch: &InductionBatch,
) -> Result<Vec<PairwiseRelationCandidate>, PairwiseRelationError> {
    type Key = (u32, u32, PairwiseOperator);

    // Preflight the raw provenance budget before allocating per-candidate source
    // vectors. For width n each frame contributes C(n, 2) * 3 source references.
    // Repeated frames otherwise leave the unique-candidate count unchanged while
    // growing memory and later sort work without bound.
    let mut source_references = 0usize;
    for episode in batch.episodes() {
        for frame in episode.frames() {
            let width = frame.numeric().len();
            if width > MAX_PAIRWISE_FEATURES {
                return Err(PairwiseRelationError::FeatureWidthExceeded {
                    actual: width,
                    maximum: MAX_PAIRWISE_FEATURES,
                });
            }
            let pairs = width
                .checked_mul(width.saturating_sub(1))
                .and_then(|value| value.checked_div(2))
                .ok_or(PairwiseRelationError::SourceReferenceLimitExceeded {
                    maximum: MAX_PAIRWISE_SOURCE_REFERENCES,
                })?;
            let additional = pairs.checked_mul(3).ok_or(
                PairwiseRelationError::SourceReferenceLimitExceeded {
                    maximum: MAX_PAIRWISE_SOURCE_REFERENCES,
                },
            )?;
            source_references = source_references.checked_add(additional).ok_or(
                PairwiseRelationError::SourceReferenceLimitExceeded {
                    maximum: MAX_PAIRWISE_SOURCE_REFERENCES,
                },
            )?;
            if source_references > MAX_PAIRWISE_SOURCE_REFERENCES {
                return Err(PairwiseRelationError::SourceReferenceLimitExceeded {
                    maximum: MAX_PAIRWISE_SOURCE_REFERENCES,
                });
            }
        }
    }

    let mut sources: BTreeMap<Key, Vec<CandidateSource>> = BTreeMap::new();
    for episode in batch.episodes() {
        for frame in episode.frames() {
            let width = frame.numeric().len();
            let source = CandidateSource::new(episode.id(), frame.ordinal());
            for left in 0..width {
                for right in (left + 1)..width {
                    let left = u32::try_from(left).map_err(|_| {
                        PairwiseRelationError::FeatureWidthExceeded {
                            actual: width,
                            maximum: MAX_PAIRWISE_FEATURES,
                        }
                    })?;
                    let right = u32::try_from(right).map_err(|_| {
                        PairwiseRelationError::FeatureWidthExceeded {
                            actual: width,
                            maximum: MAX_PAIRWISE_FEATURES,
                        }
                    })?;
                    for operator in [
                        PairwiseOperator::Less,
                        PairwiseOperator::Equal,
                        PairwiseOperator::Greater,
                    ] {
                        sources
                            .entry((left, right, operator))
                            .or_default()
                            .push(source);
                        if sources.len() > MAX_PAIRWISE_CANDIDATES {
                            return Err(PairwiseRelationError::CandidateLimitExceeded {
                                maximum: MAX_PAIRWISE_CANDIDATES,
                            });
                        }
                    }
                }
            }
        }
    }

    sources
        .into_iter()
        .map(|((left_feature, right_feature, operator), sources)| {
            let provenance = CandidateProvenance::new_in_batch(
                PredicateCandidateFamily::PairwiseRelation,
                sources,
                batch,
            )?;
            Ok(PairwiseRelationCandidate {
                left_feature,
                right_feature,
                operator,
                provenance,
            })
        })
        .collect()
}

impl core::fmt::Display for PairwiseRelationError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::FeatureWidthExceeded { actual, maximum } => write!(
                formatter,
                "pairwise feature width {actual} exceeds bounded maximum {maximum}"
            ),
            Self::CandidateLimitExceeded { maximum } => write!(
                formatter,
                "pairwise candidate count exceeds bounded maximum {maximum}"
            ),
            Self::SourceReferenceLimitExceeded { maximum } => write!(
                formatter,
                "pairwise source-reference count exceeds bounded maximum {maximum}"
            ),
            Self::Provenance(error) => write!(formatter, "invalid pairwise provenance: {error}"),
        }
    }
}

impl std::error::Error for PairwiseRelationError {}

#[cfg(test)]
mod tests {
    use super::{
        MAX_PAIRWISE_FEATURES, MAX_PAIRWISE_SOURCE_REFERENCES, PairwiseOperator,
        PairwiseRelationError, generate_pairwise_relation_candidates,
    };
    use crate::experimental::tdi2_induction_input::InductionBatch;
    use crate::experimental::tdi2_induction_split::{DEVELOPMENT_START, InductionDomain};
    use crate::experimental::tdi2_intuition::{BooleanState, NumericState};
    use crate::experimental::tdi2_observation_graph::ObservationGraph;
    use crate::experimental::tdi2_template_induction::{
        EpisodeId, ExperienceEpisode, ObservationFrame,
    };

    fn episode(id: u64, values: Vec<f64>) -> ExperienceEpisode {
        ExperienceEpisode::new(
            EpisodeId::new(id),
            vec![ObservationFrame::new(
                0,
                NumericState::new(values).expect("finite"),
                BooleanState::default(),
            )],
        )
        .expect("episode")
    }

    fn graph(id: u64) -> ObservationGraph {
        ObservationGraph::new(EpisodeId::new(id), Vec::new(), Vec::new()).expect("graph")
    }

    #[test]
    fn two_features_emit_three_label_free_relations() {
        let id = DEVELOPMENT_START;
        let batch = InductionBatch::new(
            InductionDomain::Development,
            vec![episode(id, vec![1.0, 2.0])],
            vec![graph(id)],
        )
        .expect("batch");
        let candidates = generate_pairwise_relation_candidates(&batch).expect("candidates");
        assert_eq!(candidates.len(), 3);
        assert!(candidates.iter().all(|candidate| {
            candidate.left_feature() == 0
                && candidate.right_feature() == 1
                && candidate.provenance().sources().len() == 1
        }));
    }

    #[test]
    fn repeated_pair_identity_accumulates_provenance() {
        let first = DEVELOPMENT_START;
        let second = DEVELOPMENT_START + 1;
        let batch = InductionBatch::new(
            InductionDomain::Development,
            vec![
                episode(second, vec![2.0, 1.0]),
                episode(first, vec![1.0, 2.0]),
            ],
            vec![graph(second), graph(first)],
        )
        .expect("batch");
        let candidates = generate_pairwise_relation_candidates(&batch).expect("candidates");
        assert_eq!(candidates.len(), 3);
        assert!(
            candidates
                .iter()
                .all(|candidate| candidate.provenance().sources().len() == 2)
        );
    }

    #[test]
    fn evaluation_fails_closed_on_bad_input() {
        let id = DEVELOPMENT_START;
        let batch = InductionBatch::new(
            InductionDomain::Development,
            vec![episode(id, vec![1.0, 2.0])],
            vec![graph(id)],
        )
        .expect("batch");
        let candidates = generate_pairwise_relation_candidates(&batch).expect("candidates");
        let less = candidates
            .iter()
            .find(|candidate| candidate.operator() == PairwiseOperator::Less)
            .expect("less candidate");
        assert_eq!(less.evaluate(&[1.0, 2.0]), Some(true));
        assert_eq!(less.evaluate(&[1.0]), None);
        assert_eq!(less.evaluate(&[f64::NAN, 2.0]), None);
    }

    #[test]
    fn repeated_frames_cannot_bypass_pairwise_provenance_bound() {
        let id = DEVELOPMENT_START;
        let per_frame = (MAX_PAIRWISE_FEATURES * (MAX_PAIRWISE_FEATURES - 1) / 2) * 3;
        let frame_count = MAX_PAIRWISE_SOURCE_REFERENCES / per_frame + 1;
        let frames = (0..frame_count)
            .map(|ordinal| {
                ObservationFrame::new(
                    u32::try_from(ordinal).expect("bounded ordinal"),
                    NumericState::new(vec![1.0; MAX_PAIRWISE_FEATURES]).expect("finite"),
                    BooleanState::default(),
                )
            })
            .collect();
        let episode = ExperienceEpisode::new(EpisodeId::new(id), frames).expect("episode");
        let batch =
            InductionBatch::new(InductionDomain::Development, vec![episode], vec![graph(id)])
                .expect("batch");

        assert_eq!(
            generate_pairwise_relation_candidates(&batch),
            Err(PairwiseRelationError::SourceReferenceLimitExceeded {
                maximum: MAX_PAIRWISE_SOURCE_REFERENCES,
            })
        );
    }

    #[test]
    fn excessive_width_is_rejected_before_candidate_explosion() {
        let id = DEVELOPMENT_START;
        let batch = InductionBatch::new(
            InductionDomain::Development,
            vec![episode(id, vec![0.0; MAX_PAIRWISE_FEATURES + 1])],
            vec![graph(id)],
        )
        .expect("batch");
        assert_eq!(
            generate_pairwise_relation_candidates(&batch),
            Err(PairwiseRelationError::FeatureWidthExceeded {
                actual: MAX_PAIRWISE_FEATURES + 1,
                maximum: MAX_PAIRWISE_FEATURES,
            })
        );
    }
}
