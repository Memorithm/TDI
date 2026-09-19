//! Predicate-candidate provenance and admissibility for TDI-2.2.
//!
//! Candidate provenance can name only observable episode material. There is no
//! representation for expected labels, expected templates or expected role maps.

use super::tdi2_template_induction::EpisodeId;

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
}

impl CandidateProvenance {
    /// Validate and canonicalize observable-only candidate provenance.
    pub fn new(
        family: PredicateCandidateFamily,
        mut sources: Vec<CandidateSource>,
    ) -> Result<Self, CandidateProvenanceError> {
        if sources.is_empty() {
            return Err(CandidateProvenanceError::MissingObservableSource);
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
        }
    }
}

impl std::error::Error for CandidateProvenanceError {}

#[cfg(test)]
mod tests {
    use super::{
        CandidateProvenance, CandidateProvenanceError, CandidateSource, PredicateCandidateFamily,
    };
    use crate::experimental::tdi2_template_induction::EpisodeId;

    #[test]
    fn candidate_provenance_is_observation_only_and_canonical() {
        let late = CandidateSource::new(EpisodeId::new(9), 4);
        let early = CandidateSource::new(EpisodeId::new(2), 1);
        let provenance = CandidateProvenance::new(
            PredicateCandidateFamily::TemporalDelta,
            vec![late, early, late],
        )
        .expect("provenance");
        assert_eq!(provenance.sources(), &[early, late]);
        assert_eq!(provenance.family(), PredicateCandidateFamily::TemporalDelta);
    }

    #[test]
    fn source_free_candidate_is_rejected() {
        assert_eq!(
            CandidateProvenance::new(PredicateCandidateFamily::ScalarThreshold, Vec::new()),
            Err(CandidateProvenanceError::MissingObservableSource)
        );
    }
}
