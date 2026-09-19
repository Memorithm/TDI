//! Deterministic temporal-delta predicate candidates for TDI-2.2.
//!
//! Generation uses only ordered numeric observations in `InductionBatch`.
//! Expected labels, outcomes and evaluator state are not inputs.

use std::collections::BTreeMap;

use super::tdi2_induction_input::InductionBatch;
use super::tdi2_predicate_candidates::{
    CandidateProvenance, CandidateProvenanceError, CandidateSource, PredicateCandidateFamily,
};

/// Maximum numeric width accepted by this bounded temporal generator.
pub const MAX_TEMPORAL_FEATURES: usize = 128;
/// Maximum canonical temporal candidates emitted by one batch.
pub const MAX_TEMPORAL_CANDIDATES: usize = 4_096;
/// Maximum raw source references retained before canonicalization.
pub const MAX_TEMPORAL_SOURCE_REFERENCES: usize = 65_536;

/// Direction of a one-feature temporal delta.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TemporalDeltaOperator {
    Falling,
    Stable,
    Rising,
}

impl TemporalDeltaOperator {
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Falling => "falling",
            Self::Stable => "stable",
            Self::Rising => "rising",
        }
    }
}

/// Canonical one-feature temporal predicate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemporalDeltaCandidate {
    feature_index: u32,
    operator: TemporalDeltaOperator,
    provenance: CandidateProvenance,
}

impl TemporalDeltaCandidate {
    #[must_use]
    pub const fn feature_index(&self) -> u32 {
        self.feature_index
    }

    #[must_use]
    pub const fn operator(&self) -> TemporalDeltaOperator {
        self.operator
    }

    #[must_use]
    pub const fn provenance(&self) -> &CandidateProvenance {
        &self.provenance
    }

    /// Evaluate one transition; missing/non-finite values remain unknown.
    #[must_use]
    pub fn evaluate(&self, previous: &[f64], current: &[f64]) -> Option<bool> {
        let previous = *previous.get(self.feature_index as usize)?;
        let current = *current.get(self.feature_index as usize)?;
        if !previous.is_finite() || !current.is_finite() {
            return None;
        }
        Some(match self.operator {
            TemporalDeltaOperator::Falling => current < previous,
            TemporalDeltaOperator::Stable => current == previous,
            TemporalDeltaOperator::Rising => current > previous,
        })
    }
}

/// Fail-closed bounded-generation errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemporalDeltaError {
    SourceLimitExceeded { maximum: usize },
    FeatureWidthExceeded { actual: usize, maximum: usize },
    CandidateLimitExceeded { maximum: usize },
    Provenance(CandidateProvenanceError),
}

impl From<CandidateProvenanceError> for TemporalDeltaError {
    fn from(error: CandidateProvenanceError) -> Self {
        Self::Provenance(error)
    }
}

/// Generate falling/stable/rising predicates for features observed in at least
/// one adjacent frame pair. Each contributing episode contributes both source
/// frames, satisfying the temporal-provenance contract explicitly.
pub fn generate_temporal_delta_candidates(
    batch: &InductionBatch,
) -> Result<Vec<TemporalDeltaCandidate>, TemporalDeltaError> {
    // Bound raw references before allocation, even when all identities repeat.
    let mut references = 0_usize;
    for episode in batch.episodes() {
        for frames in episode.frames().windows(2) {
            let actual = frames[0].numeric().len().max(frames[1].numeric().len());
            if actual > MAX_TEMPORAL_FEATURES {
                return Err(TemporalDeltaError::FeatureWidthExceeded {
                    actual,
                    maximum: MAX_TEMPORAL_FEATURES,
                });
            }
            references = frames[0]
                .numeric()
                .len()
                .min(frames[1].numeric().len())
                .checked_mul(6)
                .and_then(|count| references.checked_add(count))
                .filter(|count| *count <= MAX_TEMPORAL_SOURCE_REFERENCES)
                .ok_or(TemporalDeltaError::SourceLimitExceeded {
                    maximum: MAX_TEMPORAL_SOURCE_REFERENCES,
                })?;
        }
    }
    type Key = (u32, TemporalDeltaOperator);
    let mut sources: BTreeMap<Key, Vec<CandidateSource>> = BTreeMap::new();
    for episode in batch.episodes() {
        for frames in episode.frames().windows(2) {
            let width = frames[0].numeric().len().min(frames[1].numeric().len());
            if frames[0].numeric().len() > MAX_TEMPORAL_FEATURES
                || frames[1].numeric().len() > MAX_TEMPORAL_FEATURES
            {
                return Err(TemporalDeltaError::FeatureWidthExceeded {
                    actual: frames[0].numeric().len().max(frames[1].numeric().len()),
                    maximum: MAX_TEMPORAL_FEATURES,
                });
            }
            let previous = CandidateSource::new(episode.id(), frames[0].ordinal());
            let current = CandidateSource::new(episode.id(), frames[1].ordinal());
            for feature in 0..width {
                let feature = u32::try_from(feature).map_err(|_| {
                    TemporalDeltaError::FeatureWidthExceeded {
                        actual: width,
                        maximum: MAX_TEMPORAL_FEATURES,
                    }
                })?;
                for operator in [
                    TemporalDeltaOperator::Falling,
                    TemporalDeltaOperator::Stable,
                    TemporalDeltaOperator::Rising,
                ] {
                    let entry = sources.entry((feature, operator)).or_default();
                    entry.push(previous);
                    entry.push(current);
                    if sources.len() > MAX_TEMPORAL_CANDIDATES {
                        return Err(TemporalDeltaError::CandidateLimitExceeded {
                            maximum: MAX_TEMPORAL_CANDIDATES,
                        });
                    }
                }
            }
        }
    }

    sources
        .into_iter()
        .map(|((feature_index, operator), sources)| {
            let provenance = CandidateProvenance::new_in_batch(
                PredicateCandidateFamily::TemporalDelta,
                sources,
                batch,
            )?;
            Ok(TemporalDeltaCandidate {
                feature_index,
                operator,
                provenance,
            })
        })
        .collect()
}

impl core::fmt::Display for TemporalDeltaError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::SourceLimitExceeded { maximum } => write!(
                formatter,
                "temporal source references exceed bounded maximum {maximum}"
            ),
            Self::FeatureWidthExceeded { actual, maximum } => write!(
                formatter,
                "temporal feature width {actual} exceeds bounded maximum {maximum}"
            ),
            Self::CandidateLimitExceeded { maximum } => write!(
                formatter,
                "temporal candidate count exceeds bounded maximum {maximum}"
            ),
            Self::Provenance(error) => write!(formatter, "invalid temporal provenance: {error}"),
        }
    }
}
impl std::error::Error for TemporalDeltaError {}

#[cfg(test)]
mod tests {
    use super::{
        MAX_TEMPORAL_FEATURES, TemporalDeltaError, TemporalDeltaOperator,
        generate_temporal_delta_candidates,
    };
    use crate::experimental::tdi2_induction_input::InductionBatch;
    use crate::experimental::tdi2_induction_split::{DEVELOPMENT_START, InductionDomain};
    use crate::experimental::tdi2_intuition::{BooleanState, NumericState};
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

    fn graph(id: u64) -> ObservationGraph {
        ObservationGraph::new(EpisodeId::new(id), Vec::new(), Vec::new()).expect("graph")
    }

    #[test]
    fn two_frames_emit_three_candidates_per_shared_feature() {
        let id = DEVELOPMENT_START;
        let batch = InductionBatch::new(
            InductionDomain::Development,
            vec![episode(id, &[(0, vec![1.0, 2.0]), (1, vec![2.0, 2.0])])],
            vec![graph(id)],
        )
        .expect("batch");
        let candidates = generate_temporal_delta_candidates(&batch).expect("candidates");
        assert_eq!(candidates.len(), 6);
        assert!(
            candidates
                .iter()
                .all(|candidate| candidate.provenance().sources().len() == 2)
        );
    }

    #[test]
    fn repeated_identities_cannot_exhaust_provenance_storage() {
        let id = DEVELOPMENT_START;
        let frames: Vec<_> = (0..11_000).map(|ordinal| (ordinal, vec![1.0])).collect();
        let batch = InductionBatch::new(
            InductionDomain::Development,
            vec![episode(id, &frames)],
            vec![graph(id)],
        )
        .expect("batch");
        assert_eq!(
            generate_temporal_delta_candidates(&batch),
            Err(TemporalDeltaError::SourceLimitExceeded {
                maximum: super::MAX_TEMPORAL_SOURCE_REFERENCES
            })
        );
    }

    #[test]
    fn multiple_pairs_canonicalize_sources_without_labels() {
        let id = DEVELOPMENT_START;
        let batch = InductionBatch::new(
            InductionDomain::Development,
            vec![episode(
                id,
                &[(0, vec![1.0]), (1, vec![2.0]), (2, vec![3.0])],
            )],
            vec![graph(id)],
        )
        .expect("batch");
        let candidates = generate_temporal_delta_candidates(&batch).expect("candidates");
        assert_eq!(candidates.len(), 3);
        assert!(
            candidates
                .iter()
                .all(|candidate| candidate.provenance().sources().len() == 3)
        );
    }
    #[test]
    fn evaluation_is_explicit_and_unknown_for_bad_input() {
        let id = DEVELOPMENT_START;
        let batch = InductionBatch::new(
            InductionDomain::Development,
            vec![episode(id, &[(0, vec![1.0]), (1, vec![2.0])])],
            vec![graph(id)],
        )
        .expect("batch");
        let candidates = generate_temporal_delta_candidates(&batch).expect("candidates");
        let rising = candidates
            .iter()
            .find(|candidate| candidate.operator() == TemporalDeltaOperator::Rising)
            .expect("rising candidate");
        assert_eq!(rising.evaluate(&[1.0], &[2.0]), Some(true));
        assert_eq!(rising.evaluate(&[], &[2.0]), None);
        assert_eq!(rising.evaluate(&[1.0], &[f64::INFINITY]), None);
    }

    #[test]
    fn excessive_width_fails_closed() {
        let id = DEVELOPMENT_START;
        let values = vec![0.0; MAX_TEMPORAL_FEATURES + 1];
        let batch = InductionBatch::new(
            InductionDomain::Development,
            vec![episode(id, &[(0, values.clone()), (1, values)])],
            vec![graph(id)],
        )
        .expect("batch");
        assert_eq!(
            generate_temporal_delta_candidates(&batch),
            Err(TemporalDeltaError::FeatureWidthExceeded {
                actual: MAX_TEMPORAL_FEATURES + 1,
                maximum: MAX_TEMPORAL_FEATURES,
            })
        );
    }
}
