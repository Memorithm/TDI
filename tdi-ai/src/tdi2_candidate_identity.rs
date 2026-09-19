//! Canonical identity and exact de-duplication for TDI-2.2 predicate candidates.
//!
//! This layer is deliberately observation-only. It canonicalizes candidates
//! already generated from a validated [`InductionBatch`], preserves every
//! observable provenance source, and revalidates the merged provenance against
//! the target batch. Expected labels, outcomes, evaluator state and latency are
//! not inputs.

use std::collections::BTreeMap;

use super::tdi2_induction_input::InductionBatch;
use super::tdi2_numeric_thresholds::{
    MAX_SCALAR_THRESHOLD_SOURCE_REFERENCES, NumericThresholdCandidate, ThresholdDirection,
};
use super::tdi2_pairwise_relations::{
    MAX_PAIRWISE_SOURCE_REFERENCES, PairwiseOperator, PairwiseRelationCandidate,
};
use super::tdi2_predicate_candidates::{
    CandidateProvenance, CandidateProvenanceError, CandidateSource, PredicateCandidateFamily,
};
use super::tdi2_temporal_delta::{
    MAX_TEMPORAL_SOURCE_REFERENCES, TemporalDeltaCandidate, TemporalDeltaOperator,
};

/// Versioned schema label for stable candidate identities emitted by this slice.
pub const CANDIDATE_IDENTITY_SCHEMA: &str = "tdi2.2-predicate-candidate-id-v1";

/// The bounded aggregate maximum of the three Stage-B generators available at
/// this point in TDI-2.2.
pub const MAX_CANONICAL_CANDIDATES: usize = 16_384;

/// Aggregate raw provenance-reference budget accepted by de-duplication.
///
/// This is the sum of the bounded source budgets of the three Stage-B
/// generators. It also protects the public `Clone` surface from multiplying a
/// valid candidate's provenance without increasing the identity-map size.
pub const MAX_CANONICAL_SOURCE_REFERENCES: usize = MAX_SCALAR_THRESHOLD_SOURCE_REFERENCES
    + MAX_PAIRWISE_SOURCE_REFERENCES
    + MAX_TEMPORAL_SOURCE_REFERENCES;

/// Exact, provenance-independent identity of one predicate candidate.
///
/// The enum variant namespaces otherwise similar numeric indices so identities
/// from different predicate families cannot collide. Floating thresholds are
/// represented by the canonical bits produced by the threshold generator.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PredicateCandidateIdentity {
    ScalarThreshold {
        feature_index: u32,
        threshold_bits: u64,
        direction: ThresholdDirection,
    },
    PairwiseRelation {
        left_feature: u32,
        right_feature: u32,
        operator: PairwiseOperator,
    },
    TemporalDelta {
        feature_index: u32,
        operator: TemporalDeltaOperator,
    },
}

impl PredicateCandidateIdentity {
    /// Broad predicate family implied by this exact identity.
    #[must_use]
    pub const fn family(&self) -> PredicateCandidateFamily {
        match self {
            Self::ScalarThreshold { .. } => PredicateCandidateFamily::ScalarThreshold,
            Self::PairwiseRelation { .. } => PredicateCandidateFamily::PairwiseRelation,
            Self::TemporalDelta { .. } => PredicateCandidateFamily::TemporalDelta,
        }
    }

    /// Versioned, deterministic textual key suitable for manifests and diffs.
    ///
    /// This is an identity key, not a novelty fingerprint or cryptographic
    /// attestation.
    #[must_use]
    pub fn canonical_key(&self) -> String {
        match self {
            Self::ScalarThreshold {
                feature_index,
                threshold_bits,
                direction,
            } => format!(
                "{CANDIDATE_IDENTITY_SCHEMA}:scalar:{feature_index}:{threshold_bits:016x}:{}",
                direction.key()
            ),
            Self::PairwiseRelation {
                left_feature,
                right_feature,
                operator,
            } => format!(
                "{CANDIDATE_IDENTITY_SCHEMA}:pair:{left_feature}:{right_feature}:{}",
                operator.key()
            ),
            Self::TemporalDelta {
                feature_index,
                operator,
            } => format!(
                "{CANDIDATE_IDENTITY_SCHEMA}:temporal:{feature_index}:{}",
                operator.key()
            ),
        }
    }
}

impl From<&NumericThresholdCandidate> for PredicateCandidateIdentity {
    fn from(candidate: &NumericThresholdCandidate) -> Self {
        Self::ScalarThreshold {
            feature_index: candidate.feature_index(),
            threshold_bits: candidate.threshold_bits(),
            direction: candidate.direction(),
        }
    }
}

impl From<&PairwiseRelationCandidate> for PredicateCandidateIdentity {
    fn from(candidate: &PairwiseRelationCandidate) -> Self {
        Self::PairwiseRelation {
            left_feature: candidate.left_feature(),
            right_feature: candidate.right_feature(),
            operator: candidate.operator(),
        }
    }
}

impl From<&TemporalDeltaCandidate> for PredicateCandidateIdentity {
    fn from(candidate: &TemporalDeltaCandidate) -> Self {
        Self::TemporalDelta {
            feature_index: candidate.feature_index(),
            operator: candidate.operator(),
        }
    }
}

/// One canonical identity paired with validated observable provenance.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalPredicateCandidate {
    identity: PredicateCandidateIdentity,
    provenance: CandidateProvenance,
}

impl CanonicalPredicateCandidate {
    #[must_use]
    pub fn from_threshold(candidate: &NumericThresholdCandidate) -> Self {
        Self {
            identity: candidate.into(),
            provenance: candidate.provenance().clone(),
        }
    }

    #[must_use]
    pub fn from_pairwise(candidate: &PairwiseRelationCandidate) -> Self {
        Self {
            identity: candidate.into(),
            provenance: candidate.provenance().clone(),
        }
    }

    #[must_use]
    pub fn from_temporal(candidate: &TemporalDeltaCandidate) -> Self {
        Self {
            identity: candidate.into(),
            provenance: candidate.provenance().clone(),
        }
    }

    #[must_use]
    pub const fn identity(&self) -> &PredicateCandidateIdentity {
        &self.identity
    }

    #[must_use]
    pub const fn provenance(&self) -> &CandidateProvenance {
        &self.provenance
    }
}

/// Fail-closed errors from exact candidate de-duplication.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CandidateDeduplicationError {
    CandidateLimitExceeded { maximum: usize },
    SourceReferenceLimitExceeded { maximum: usize },
    IdentityEvidenceMismatch { source: CandidateSource },
    Provenance(CandidateProvenanceError),
}

impl From<CandidateProvenanceError> for CandidateDeduplicationError {
    fn from(error: CandidateProvenanceError) -> Self {
        Self::Provenance(error)
    }
}

/// Merge exact duplicate identities and return deterministic identity order.
///
/// All sources attached to duplicate records are retained, canonicalized and
/// revalidated against `batch`. A record referencing material outside the batch
/// is rejected rather than silently retained.
pub fn deduplicate_candidates(
    batch: &InductionBatch,
    candidates: impl IntoIterator<Item = CanonicalPredicateCandidate>,
) -> Result<Vec<CanonicalPredicateCandidate>, CandidateDeduplicationError> {
    deduplicate_candidates_with_limits(
        batch,
        candidates,
        MAX_CANONICAL_CANDIDATES,
        MAX_CANONICAL_SOURCE_REFERENCES,
    )
}

fn deduplicate_candidates_with_limit(
    batch: &InductionBatch,
    candidates: impl IntoIterator<Item = CanonicalPredicateCandidate>,
    maximum: usize,
) -> Result<Vec<CanonicalPredicateCandidate>, CandidateDeduplicationError> {
    deduplicate_candidates_with_limits(batch, candidates, maximum, MAX_CANONICAL_SOURCE_REFERENCES)
}

fn deduplicate_candidates_with_limits(
    batch: &InductionBatch,
    candidates: impl IntoIterator<Item = CanonicalPredicateCandidate>,
    maximum_candidates: usize,
    maximum_source_references: usize,
) -> Result<Vec<CanonicalPredicateCandidate>, CandidateDeduplicationError> {
    let mut sources_by_identity: BTreeMap<PredicateCandidateIdentity, Vec<CandidateSource>> =
        BTreeMap::new();
    let mut raw_candidates = 0_usize;
    let mut raw_source_references = 0_usize;

    for candidate in candidates {
        raw_candidates = raw_candidates
            .checked_add(1)
            .filter(|count| *count <= maximum_candidates)
            .ok_or(CandidateDeduplicationError::CandidateLimitExceeded {
                maximum: maximum_candidates,
            })?;
        raw_source_references = raw_source_references
            .checked_add(candidate.provenance.sources().len())
            .filter(|count| *count <= maximum_source_references)
            .ok_or(CandidateDeduplicationError::SourceReferenceLimitExceeded {
                maximum: maximum_source_references,
            })?;
        let entry = sources_by_identity.entry(candidate.identity).or_default();
        entry.extend_from_slice(candidate.provenance.sources());
        if sources_by_identity.len() > maximum_candidates {
            return Err(CandidateDeduplicationError::CandidateLimitExceeded {
                maximum: maximum_candidates,
            });
        }
    }

    sources_by_identity
        .into_iter()
        .map(|(identity, sources)| {
            let provenance = CandidateProvenance::new_in_batch(identity.family(), sources, batch)?;
            validate_identity_evidence(&identity, &provenance, batch)?;
            Ok(CanonicalPredicateCandidate {
                identity,
                provenance,
            })
        })
        .collect()
}

fn validate_identity_evidence(
    identity: &PredicateCandidateIdentity,
    provenance: &CandidateProvenance,
    batch: &InductionBatch,
) -> Result<(), CandidateDeduplicationError> {
    for &source in provenance.sources() {
        let episode = batch
            .episodes()
            .iter()
            .find(|episode| episode.id() == source.episode())
            .expect("provenance episode validated");
        let index = episode
            .frames()
            .binary_search_by_key(&source.frame_ordinal(), |frame| frame.ordinal())
            .expect("provenance frame validated");
        let frame = &episode.frames()[index];
        let valid = match *identity {
            PredicateCandidateIdentity::ScalarThreshold {
                feature_index,
                threshold_bits,
                ..
            } => frame
                .numeric()
                .values()
                .get(feature_index as usize)
                .is_some_and(|value| {
                    let bits = if *value == 0.0 {
                        0.0_f64.to_bits()
                    } else {
                        value.to_bits()
                    };
                    bits == threshold_bits
                }),
            PredicateCandidateIdentity::PairwiseRelation {
                left_feature,
                right_feature,
                ..
            } => left_feature < right_feature && (right_feature as usize) < frame.numeric().len(),
            PredicateCandidateIdentity::TemporalDelta { feature_index, .. } => {
                (feature_index as usize) < frame.numeric().len()
                    && [index.checked_sub(1), index.checked_add(1)]
                        .into_iter()
                        .flatten()
                        .filter_map(|neighbor| episode.frames().get(neighbor))
                        .any(|neighbor| {
                            (feature_index as usize) < neighbor.numeric().len()
                                && provenance
                                    .sources()
                                    .binary_search(&CandidateSource::new(
                                        episode.id(),
                                        neighbor.ordinal(),
                                    ))
                                    .is_ok()
                        })
            }
        };
        if !valid {
            return Err(CandidateDeduplicationError::IdentityEvidenceMismatch { source });
        }
    }
    Ok(())
}

impl core::fmt::Display for CandidateDeduplicationError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::IdentityEvidenceMismatch { source } => write!(
                formatter,
                "candidate identity is unsupported by source {source:?}"
            ),
            Self::CandidateLimitExceeded { maximum } => write!(
                formatter,
                "canonical predicate candidate count exceeds bounded maximum {maximum}"
            ),
            Self::SourceReferenceLimitExceeded { maximum } => write!(
                formatter,
                "candidate provenance source references exceed bounded maximum {maximum}"
            ),
            Self::Provenance(error) => {
                write!(
                    formatter,
                    "invalid de-duplicated candidate provenance: {error}"
                )
            }
        }
    }
}

impl std::error::Error for CandidateDeduplicationError {}

#[cfg(test)]
mod tests {
    use super::{
        CandidateDeduplicationError, CanonicalPredicateCandidate, PredicateCandidateIdentity,
        deduplicate_candidates, deduplicate_candidates_with_limit,
        deduplicate_candidates_with_limits,
    };
    use crate::experimental::tdi2_induction_input::InductionBatch;
    use crate::experimental::tdi2_induction_split::{DEVELOPMENT_START, InductionDomain};
    use crate::experimental::tdi2_intuition::{BooleanState, NumericState};
    use crate::experimental::tdi2_numeric_thresholds::{
        ThresholdDirection, generate_numeric_threshold_candidates,
    };
    use crate::experimental::tdi2_observation_graph::ObservationGraph;
    use crate::experimental::tdi2_pairwise_relations::generate_pairwise_relation_candidates;
    use crate::experimental::tdi2_predicate_candidates::CandidateProvenanceError;
    use crate::experimental::tdi2_template_induction::{
        EpisodeId, ExperienceEpisode, ObservationFrame,
    };
    use crate::experimental::tdi2_temporal_delta::generate_temporal_delta_candidates;

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

    fn all_records(batch: &InductionBatch) -> Vec<CanonicalPredicateCandidate> {
        let mut records = Vec::new();
        records.extend(
            generate_numeric_threshold_candidates(batch)
                .expect("thresholds")
                .iter()
                .map(CanonicalPredicateCandidate::from_threshold),
        );
        records.extend(
            generate_pairwise_relation_candidates(batch)
                .expect("pairwise")
                .iter()
                .map(CanonicalPredicateCandidate::from_pairwise),
        );
        records.extend(
            generate_temporal_delta_candidates(batch)
                .expect("temporal")
                .iter()
                .map(CanonicalPredicateCandidate::from_temporal),
        );
        records
    }

    #[test]
    fn canonical_catalog_is_order_independent_and_sorted() {
        let id = DEVELOPMENT_START;
        let batch = batch(vec![episode(
            id,
            &[(0, vec![1.0, 2.0]), (1, vec![2.0, 1.0])],
        )]);
        let forward = deduplicate_candidates(&batch, all_records(&batch)).expect("catalog");
        let mut reverse_input = all_records(&batch);
        reverse_input.reverse();
        let reverse = deduplicate_candidates(&batch, reverse_input).expect("catalog");
        assert_eq!(forward, reverse);
        assert!(
            forward
                .windows(2)
                .all(|pair| pair[0].identity() < pair[1].identity())
        );
        assert!(forward.iter().all(|candidate| {
            candidate
                .identity()
                .canonical_key()
                .starts_with("tdi2.2-predicate-candidate-id-v1:")
        }));
    }

    #[test]
    fn exact_duplicates_merge_all_observable_sources() {
        let first = DEVELOPMENT_START;
        let second = DEVELOPMENT_START + 1;
        let first_batch = batch(vec![episode(first, &[(0, vec![3.0])])]);
        let second_batch = batch(vec![episode(second, &[(0, vec![3.0])])]);
        let combined = batch(vec![
            episode(second, &[(0, vec![3.0])]),
            episode(first, &[(0, vec![3.0])]),
        ]);

        let pick = |batch: &InductionBatch| {
            generate_numeric_threshold_candidates(batch)
                .expect("thresholds")
                .into_iter()
                .find(|candidate| {
                    candidate.feature_index() == 0
                        && candidate.threshold() == 3.0
                        && candidate.direction() == ThresholdDirection::LessEqual
                })
                .map(|candidate| CanonicalPredicateCandidate::from_threshold(&candidate))
                .expect("candidate")
        };

        let dedup =
            deduplicate_candidates(&combined, vec![pick(&second_batch), pick(&first_batch)])
                .expect("dedup");
        assert_eq!(dedup.len(), 1);
        assert_eq!(dedup[0].provenance().sources().len(), 2);
    }

    #[test]
    fn families_have_disjoint_identity_namespaces() {
        let id = DEVELOPMENT_START;
        let batch = batch(vec![episode(id, &[(0, vec![1.0]), (1, vec![2.0])])]);
        let records = all_records(&batch);
        let scalar = records
            .iter()
            .find(|candidate| {
                matches!(
                    candidate.identity(),
                    PredicateCandidateIdentity::ScalarThreshold {
                        feature_index: 0,
                        ..
                    }
                )
            })
            .expect("scalar")
            .identity();
        let temporal = records
            .iter()
            .find(|candidate| {
                matches!(
                    candidate.identity(),
                    PredicateCandidateIdentity::TemporalDelta {
                        feature_index: 0,
                        ..
                    }
                )
            })
            .expect("temporal")
            .identity();
        assert_ne!(scalar, temporal);
        assert_ne!(scalar.canonical_key(), temporal.canonical_key());
    }

    #[test]
    fn merged_provenance_is_revalidated_against_target_batch() {
        let source_id = DEVELOPMENT_START;
        let target_id = DEVELOPMENT_START + 1;
        let source = batch(vec![episode(source_id, &[(0, vec![1.0])])]);
        let target = batch(vec![episode(target_id, &[(0, vec![1.0])])]);
        let record = generate_numeric_threshold_candidates(&source)
            .expect("thresholds")
            .first()
            .map(CanonicalPredicateCandidate::from_threshold)
            .expect("record");
        assert_eq!(
            deduplicate_candidates(&target, vec![record]),
            Err(CandidateDeduplicationError::Provenance(
                CandidateProvenanceError::UnknownEpisode {
                    episode: EpisodeId::new(source_id),
                }
            ))
        );
    }

    #[test]
    fn candidate_count_bound_fails_closed() {
        let id = DEVELOPMENT_START;
        let batch = batch(vec![episode(id, &[(0, vec![1.0])])]);
        let candidates = generate_numeric_threshold_candidates(&batch).expect("thresholds");
        let records: Vec<_> = candidates
            .iter()
            .map(CanonicalPredicateCandidate::from_threshold)
            .collect();
        assert_eq!(
            deduplicate_candidates_with_limit(&batch, records, 1),
            Err(CandidateDeduplicationError::CandidateLimitExceeded { maximum: 1 })
        );
    }

    #[test]
    fn duplicate_inputs_cannot_bypass_the_raw_candidate_bound() {
        let id = DEVELOPMENT_START;
        let batch = batch(vec![episode(id, &[(0, vec![1.0])])]);
        let record = generate_numeric_threshold_candidates(&batch)
            .expect("thresholds")
            .first()
            .map(CanonicalPredicateCandidate::from_threshold)
            .expect("record");
        assert_eq!(
            deduplicate_candidates_with_limit(&batch, vec![record.clone(), record], 1),
            Err(CandidateDeduplicationError::CandidateLimitExceeded { maximum: 1 })
        );
    }

    #[test]
    fn duplicate_provenance_is_bounded_before_map_extension() {
        let id = DEVELOPMENT_START;
        let batch = batch(vec![episode(id, &[(0, vec![1.0])])]);
        let record = generate_numeric_threshold_candidates(&batch)
            .expect("thresholds")
            .first()
            .map(CanonicalPredicateCandidate::from_threshold)
            .expect("record");
        assert_eq!(
            deduplicate_candidates_with_limits(&batch, vec![record.clone(), record], 2, 1),
            Err(CandidateDeduplicationError::SourceReferenceLimitExceeded { maximum: 1 })
        );
    }

    #[test]
    fn reused_source_ids_do_not_authorize_changed_identity_evidence() {
        let id = DEVELOPMENT_START;
        let original = batch(vec![episode(
            id,
            &[(0, vec![1.0, 2.0, 3.0]), (2, vec![2.0, 3.0, 4.0])],
        )]);
        let changed = batch(vec![episode(
            id,
            &[(0, vec![9.0, 8.0]), (1, vec![]), (2, vec![8.0, 9.0])],
        )]);
        for record in all_records(&original) {
            let must_reject = match record.identity() {
                PredicateCandidateIdentity::ScalarThreshold { .. }
                | PredicateCandidateIdentity::TemporalDelta { .. } => true,
                PredicateCandidateIdentity::PairwiseRelation { right_feature, .. } => {
                    *right_feature == 2
                }
            };
            if must_reject {
                assert!(matches!(
                    deduplicate_candidates(&changed, vec![record]),
                    Err(CandidateDeduplicationError::IdentityEvidenceMismatch { .. })
                ));
            }
        }
    }
}
