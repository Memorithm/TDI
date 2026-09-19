//! Predicate-candidate provenance and admissibility for TDI-2.2.
//!
//! Candidate provenance can name only observable episode material. There is no
//! representation for expected labels, expected templates or expected role maps.

use super::tdi2_induction_input::InductionBatch;
use super::tdi2_observation_graph::ObservationGraph;
use super::tdi2_template_induction::{EpisodeId, ExperienceEpisode};

/// Broad family of a candidate Boolean predicate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PredicateCandidateFamily {
    /// One numeric feature compared with a threshold.
    ScalarThreshold,
    /// Relation between two numeric features or entity-local measurements.
    PairwiseRelation,
    /// Change between ordered observations.
    TemporalDelta,
    /// Unary predicate already present in the declared observation schema.
    ObservedUnary,
    /// Typed relation already present in a relational observation graph.
    ObservedRelation,
}

/// Observable origin of one candidate contribution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CandidateSource {
    episode: EpisodeId,
    frame_ordinal: u32,
}

impl CandidateSource {
    /// Construct observable provenance for one episode frame.
    #[must_use]
    pub const fn new(episode: EpisodeId, frame_ordinal: u32) -> Self {
        Self {
            episode,
            frame_ordinal,
        }
    }

    /// Episode containing the observation.
    #[must_use]
    pub const fn episode(self) -> EpisodeId {
        self.episode
    }

    /// Frame ordinal within that episode.
    #[must_use]
    pub const fn frame_ordinal(self) -> u32 {
        self.frame_ordinal
    }
}

/// Provenance bound to a candidate before any evaluation label is available.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateProvenance {
    family: PredicateCandidateFamily,
    sources: Vec<CandidateSource>,
}

/// Candidate provenance validation failures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CandidateProvenanceError {
    /// Every candidate must be attributable to at least one observation.
    MissingObservableSource,
    /// The episode collection itself must have unambiguous identities.
    DuplicateEpisode { episode: EpisodeId },
    /// A source references no episode in the supplied observation collection.
    UnknownEpisode { episode: EpisodeId },
    /// A source references no frame ordinal in its supplied episode.
    UnknownFrame {
        episode: EpisodeId,
        frame_ordinal: u32,
    },
    /// A source frame does not expose enough numeric observations for this family.
    InsufficientNumericEvidence {
        family: PredicateCandidateFamily,
        episode: EpisodeId,
        frame_ordinal: u32,
        required: usize,
        actual: usize,
    },
    /// Temporal deltas require at least two ordered observations in every cited episode.
    MissingTemporalPair { episode: EpisodeId },
    /// An observed-unary candidate cites a frame with no observed predicate.
    MissingObservedUnary {
        episode: EpisodeId,
        frame_ordinal: u32,
    },
    /// ExperienceEpisode does not by itself bind relational graphs to frame provenance.
    RelationalEvidenceUnbound,
    /// A relation-family source has no observed relation in its bound episode graph.
    MissingObservedRelation { episode: EpisodeId },
}

impl CandidateProvenance {
    /// Validate and canonicalize observable-only candidate provenance.
    pub fn new(
        family: PredicateCandidateFamily,
        sources: Vec<CandidateSource>,
        episodes: &[ExperienceEpisode],
    ) -> Result<Self, CandidateProvenanceError> {
        Self::new_with_graphs(family, sources, episodes, None)
    }

    /// Validate provenance against a fully bound induction batch.
    ///
    /// Relation-family candidates require this constructor because an
    /// `ExperienceEpisode` alone has no relational observation surface.
    pub fn new_in_batch(
        family: PredicateCandidateFamily,
        sources: Vec<CandidateSource>,
        batch: &InductionBatch,
    ) -> Result<Self, CandidateProvenanceError> {
        Self::new_with_graphs(family, sources, batch.episodes(), Some(batch.graphs()))
    }

    fn new_with_graphs(
        family: PredicateCandidateFamily,
        mut sources: Vec<CandidateSource>,
        episodes: &[ExperienceEpisode],
        graphs: Option<&[ObservationGraph]>,
    ) -> Result<Self, CandidateProvenanceError> {
        if sources.is_empty() {
            return Err(CandidateProvenanceError::MissingObservableSource);
        }

        let mut episode_ids: Vec<_> = episodes.iter().map(ExperienceEpisode::id).collect();
        episode_ids.sort_unstable();
        if let Some(episode) = episode_ids
            .windows(2)
            .find_map(|pair| (pair[0] == pair[1]).then_some(pair[0]))
        {
            return Err(CandidateProvenanceError::DuplicateEpisode { episode });
        }

        for source in &sources {
            let episode = episodes
                .iter()
                .find(|episode| episode.id() == source.episode())
                .ok_or(CandidateProvenanceError::UnknownEpisode {
                    episode: source.episode(),
                })?;
            if episode
                .frames()
                .binary_search_by_key(&source.frame_ordinal(), |frame| frame.ordinal())
                .is_err()
            {
                return Err(CandidateProvenanceError::UnknownFrame {
                    episode: source.episode(),
                    frame_ordinal: source.frame_ordinal(),
                });
            }
        }

        sources.sort_unstable();
        sources.dedup();
        Self::validate_family_evidence(family, &sources, episodes, graphs)?;
        Ok(Self { family, sources })
    }

    fn validate_family_evidence(
        family: PredicateCandidateFamily,
        sources: &[CandidateSource],
        episodes: &[ExperienceEpisode],
        graphs: Option<&[ObservationGraph]>,
    ) -> Result<(), CandidateProvenanceError> {
        if family == PredicateCandidateFamily::ObservedRelation {
            let graphs = graphs.ok_or(CandidateProvenanceError::RelationalEvidenceUnbound)?;
            for source in sources {
                let graph = graphs
                    .iter()
                    .find(|graph| graph.episode() == source.episode())
                    .ok_or(CandidateProvenanceError::RelationalEvidenceUnbound)?;
                if graph.relations().is_empty() {
                    return Err(CandidateProvenanceError::MissingObservedRelation {
                        episode: source.episode(),
                    });
                }
            }
            return Ok(());
        }

        for source in sources {
            let episode = episodes
                .iter()
                .find(|episode| episode.id() == source.episode())
                .expect("source episode validated above");
            let frame = episode
                .frames()
                .binary_search_by_key(&source.frame_ordinal(), |frame| frame.ordinal())
                .map(|index| &episode.frames()[index])
                .expect("source frame validated above");
            let required_numeric = match family {
                PredicateCandidateFamily::ScalarThreshold
                | PredicateCandidateFamily::TemporalDelta => 1,
                PredicateCandidateFamily::PairwiseRelation => 2,
                PredicateCandidateFamily::ObservedUnary
                | PredicateCandidateFamily::ObservedRelation => 0,
            };
            if frame.numeric().len() < required_numeric {
                return Err(CandidateProvenanceError::InsufficientNumericEvidence {
                    family,
                    episode: source.episode(),
                    frame_ordinal: source.frame_ordinal(),
                    required: required_numeric,
                    actual: frame.numeric().len(),
                });
            }
            if family == PredicateCandidateFamily::ObservedUnary
                && frame.observed_predicates().is_empty()
            {
                return Err(CandidateProvenanceError::MissingObservedUnary {
                    episode: source.episode(),
                    frame_ordinal: source.frame_ordinal(),
                });
            }
        }

        if family == PredicateCandidateFamily::TemporalDelta {
            let mut index = 0;
            while index < sources.len() {
                let episode = sources[index].episode();
                let start = index;
                while index < sources.len() && sources[index].episode() == episode {
                    index += 1;
                }
                if index - start < 2 {
                    return Err(CandidateProvenanceError::MissingTemporalPair { episode });
                }
            }
        }
        Ok(())
    }

    /// Candidate family.
    #[must_use]
    pub const fn family(&self) -> PredicateCandidateFamily {
        self.family
    }

    /// Canonical observable sources.
    #[must_use]
    pub fn sources(&self) -> &[CandidateSource] {
        &self.sources
    }
}

impl core::fmt::Display for CandidateProvenanceError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MissingObservableSource => {
                formatter.write_str("predicate candidate requires observable provenance")
            }
            Self::DuplicateEpisode { episode } => write!(
                formatter,
                "observation collection contains duplicate episode {}",
                episode.raw()
            ),
            Self::UnknownEpisode { episode } => write!(
                formatter,
                "candidate source references unknown episode {}",
                episode.raw()
            ),
            Self::UnknownFrame {
                episode,
                frame_ordinal,
            } => write!(
                formatter,
                "candidate source references unknown frame {frame_ordinal} in episode {}",
                episode.raw()
            ),
            Self::InsufficientNumericEvidence {
                family,
                episode,
                frame_ordinal,
                required,
                actual,
            } => write!(
                formatter,
                "candidate family {family:?} requires at least {required} numeric observations at frame {frame_ordinal} in episode {}, found {actual}",
                episode.raw()
            ),
            Self::MissingTemporalPair { episode } => write!(
                formatter,
                "temporal candidate has fewer than two ordered sources in episode {}",
                episode.raw()
            ),
            Self::MissingObservedUnary {
                episode,
                frame_ordinal,
            } => write!(
                formatter,
                "observed-unary candidate cites predicate-free frame {frame_ordinal} in episode {}",
                episode.raw()
            ),
            Self::RelationalEvidenceUnbound => formatter.write_str(
                "observed-relation provenance requires an explicit observation-graph binding",
            ),
            Self::MissingObservedRelation { episode } => write!(
                formatter,
                "observed-relation candidate has no relational evidence for episode {}",
                episode.raw()
            ),
        }
    }
}

impl std::error::Error for CandidateProvenanceError {}

#[cfg(test)]
mod tests {
    use super::{
        CandidateProvenance, CandidateProvenanceError, CandidateSource, PredicateCandidateFamily,
    };
    use crate::experimental::tdi2_induction_input::InductionBatch;
    use crate::experimental::tdi2_induction_split::{DEVELOPMENT_START, InductionDomain};
    use crate::experimental::tdi2_intuition::{BooleanState, NumericState};
    use crate::experimental::tdi2_observation_graph::{
        ObservationGraph, ObservedEntity, ObservedEntityId, ObservedRelation, ObservedRelationId,
    };
    use crate::experimental::tdi2_template_induction::{
        EpisodeId, ExperienceEpisode, ObservationFrame,
    };

    fn episode(id: u64, ordinals: &[u32]) -> ExperienceEpisode {
        ExperienceEpisode::new(
            EpisodeId::new(id),
            ordinals
                .iter()
                .copied()
                .map(|ordinal| {
                    ObservationFrame::new(
                        ordinal,
                        NumericState::new(vec![f64::from(ordinal)]).expect("finite"),
                        BooleanState::new(Vec::new()),
                    )
                })
                .collect(),
        )
        .expect("episode")
    }

    #[test]
    fn candidate_provenance_is_observation_only_and_canonical() {
        let episodes = [episode(2, &[1, 4])];
        let late = CandidateSource::new(EpisodeId::new(2), 4);
        let early = CandidateSource::new(EpisodeId::new(2), 1);
        let provenance = CandidateProvenance::new(
            PredicateCandidateFamily::TemporalDelta,
            vec![late, early, late],
            &episodes,
        )
        .expect("provenance");
        assert_eq!(provenance.sources(), &[early, late]);
        assert_eq!(provenance.family(), PredicateCandidateFamily::TemporalDelta);
    }

    #[test]
    fn source_free_candidate_is_rejected() {
        let episodes = [episode(1, &[0])];
        assert_eq!(
            CandidateProvenance::new(
                PredicateCandidateFamily::ScalarThreshold,
                Vec::new(),
                &episodes,
            ),
            Err(CandidateProvenanceError::MissingObservableSource)
        );
    }

    #[test]
    fn stale_or_fabricated_sources_are_rejected() {
        let episodes = [episode(7, &[0, 3])];
        assert_eq!(
            CandidateProvenance::new(
                PredicateCandidateFamily::ObservedUnary,
                vec![CandidateSource::new(EpisodeId::new(8), 0)],
                &episodes,
            ),
            Err(CandidateProvenanceError::UnknownEpisode {
                episode: EpisodeId::new(8)
            })
        );
        assert_eq!(
            CandidateProvenance::new(
                PredicateCandidateFamily::ObservedUnary,
                vec![CandidateSource::new(EpisodeId::new(7), 2)],
                &episodes,
            ),
            Err(CandidateProvenanceError::UnknownFrame {
                episode: EpisodeId::new(7),
                frame_ordinal: 2,
            })
        );
    }

    #[test]
    fn family_specific_evidence_is_required_fail_closed() {
        let empty_numeric = ExperienceEpisode::new(
            EpisodeId::new(1),
            vec![ObservationFrame::new(
                0,
                NumericState::new(Vec::new()).expect("finite"),
                BooleanState::new(Vec::new()),
            )],
        )
        .expect("episode");
        assert!(matches!(
            CandidateProvenance::new(
                PredicateCandidateFamily::ScalarThreshold,
                vec![CandidateSource::new(EpisodeId::new(1), 0)],
                &[empty_numeric],
            ),
            Err(CandidateProvenanceError::InsufficientNumericEvidence {
                required: 1,
                actual: 0,
                ..
            })
        ));

        let one_numeric = [episode(2, &[0])];
        assert!(matches!(
            CandidateProvenance::new(
                PredicateCandidateFamily::PairwiseRelation,
                vec![CandidateSource::new(EpisodeId::new(2), 0)],
                &one_numeric,
            ),
            Err(CandidateProvenanceError::InsufficientNumericEvidence {
                required: 2,
                actual: 1,
                ..
            })
        ));
        assert_eq!(
            CandidateProvenance::new(
                PredicateCandidateFamily::TemporalDelta,
                vec![CandidateSource::new(EpisodeId::new(2), 0)],
                &one_numeric,
            ),
            Err(CandidateProvenanceError::MissingTemporalPair {
                episode: EpisodeId::new(2)
            })
        );
        assert_eq!(
            CandidateProvenance::new(
                PredicateCandidateFamily::ObservedUnary,
                vec![CandidateSource::new(EpisodeId::new(2), 0)],
                &one_numeric,
            ),
            Err(CandidateProvenanceError::MissingObservedUnary {
                episode: EpisodeId::new(2),
                frame_ordinal: 0,
            })
        );
        assert_eq!(
            CandidateProvenance::new(
                PredicateCandidateFamily::ObservedRelation,
                vec![CandidateSource::new(EpisodeId::new(2), 0)],
                &one_numeric,
            ),
            Err(CandidateProvenanceError::RelationalEvidenceUnbound)
        );
    }

    #[test]
    fn relation_candidates_require_relational_evidence_from_bound_batch() {
        let id = DEVELOPMENT_START;
        let observed_episode = ExperienceEpisode::new(
            EpisodeId::new(id),
            vec![ObservationFrame::new(
                0,
                NumericState::new(vec![1.0, 2.0]).expect("finite"),
                BooleanState::default(),
            )],
        )
        .expect("episode");
        let source = CandidateSource::new(EpisodeId::new(id), 0);
        let empty_graph =
            ObservationGraph::new(EpisodeId::new(id), Vec::new(), Vec::new()).expect("empty graph");
        let empty_batch = InductionBatch::new(
            InductionDomain::Development,
            vec![observed_episode.clone()],
            vec![empty_graph],
        )
        .expect("batch");
        assert_eq!(
            CandidateProvenance::new_in_batch(
                PredicateCandidateFamily::ObservedRelation,
                vec![source],
                &empty_batch,
            ),
            Err(CandidateProvenanceError::MissingObservedRelation {
                episode: EpisodeId::new(id)
            })
        );

        let left = ObservedEntityId::new(1);
        let right = ObservedEntityId::new(2);
        let graph = ObservationGraph::new(
            EpisodeId::new(id),
            vec![
                ObservedEntity::new(left, BooleanState::default()),
                ObservedEntity::new(right, BooleanState::default()),
            ],
            vec![ObservedRelation::new(
                left,
                ObservedRelationId::new(7),
                right,
            )],
        )
        .expect("graph");
        let batch = InductionBatch::new(
            InductionDomain::Development,
            vec![observed_episode],
            vec![graph],
        )
        .expect("batch");
        let provenance = CandidateProvenance::new_in_batch(
            PredicateCandidateFamily::ObservedRelation,
            vec![source],
            &batch,
        )
        .expect("relation evidence");
        assert_eq!(provenance.sources(), &[source]);
    }

    #[test]
    fn duplicate_episode_ids_make_provenance_ambiguous() {
        let episodes = [episode(5, &[0]), episode(5, &[1])];
        assert_eq!(
            CandidateProvenance::new(
                PredicateCandidateFamily::ObservedRelation,
                vec![CandidateSource::new(EpisodeId::new(5), 0)],
                &episodes,
            ),
            Err(CandidateProvenanceError::DuplicateEpisode {
                episode: EpisodeId::new(5)
            })
        );
    }
}
