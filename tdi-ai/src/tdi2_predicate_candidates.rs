//! Predicate-candidate provenance and admissibility for TDI-2.2.
//!
//! Candidate provenance can name only observable episode material. There is no
//! representation for expected labels, expected templates or expected role maps.

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
}

impl CandidateProvenance {
    /// Validate and canonicalize observable-only candidate provenance.
    pub fn new(
        family: PredicateCandidateFamily,
        mut sources: Vec<CandidateSource>,
        episodes: &[ExperienceEpisode],
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
        Ok(Self { family, sources })
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
        }
    }
}

impl std::error::Error for CandidateProvenanceError {}

#[cfg(test)]
mod tests {
    use super::{
        CandidateProvenance, CandidateProvenanceError, CandidateSource, PredicateCandidateFamily,
    };
    use crate::experimental::tdi2_intuition::{BooleanState, NumericState};
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
        let episodes = [episode(2, &[1]), episode(9, &[4])];
        let late = CandidateSource::new(EpisodeId::new(9), 4);
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
