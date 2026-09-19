//! Label-leakage boundary for TDI-2.2 induction algorithms.
//!
//! Future template inductors receive `InductionBatch` rather than evaluator
//! cases. The batch contains observed episodes and graphs only; expected labels,
//! expected template identities and expected role mappings have no field here.

use super::tdi2_observation_graph::ObservationGraph;
use super::tdi2_template_induction::{EpisodeId, ExperienceEpisode};

/// Observation-only batch passed into template induction.
#[derive(Clone, Debug, PartialEq)]
pub struct InductionBatch {
    episodes: Vec<ExperienceEpisode>,
    graphs: Vec<ObservationGraph>,
}

/// Batch construction failures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InductionInputError {
    /// At least one observed episode is required.
    EmptyBatch,
    /// Every episode must have exactly one corresponding observation graph.
    MismatchedCardinality,
    /// Repeating one episode would silently overweight it.
    DuplicateEpisode { episode: EpisodeId },
}

impl InductionBatch {
    /// Construct a validated observation-only induction batch.
    pub fn new(
        mut episodes: Vec<ExperienceEpisode>,
        graphs: Vec<ObservationGraph>,
    ) -> Result<Self, InductionInputError> {
        if episodes.is_empty() {
            return Err(InductionInputError::EmptyBatch);
        }
        if episodes.len() != graphs.len() {
            return Err(InductionInputError::MismatchedCardinality);
        }
        let mut ids = episodes
            .iter()
            .map(ExperienceEpisode::id)
            .collect::<Vec<_>>();
        ids.sort_unstable();
        if let Some(episode) = ids
            .windows(2)
            .find_map(|pair| (pair[0] == pair[1]).then_some(pair[0]))
        {
            return Err(InductionInputError::DuplicateEpisode { episode });
        }
        // Preserve caller-declared episode/graph pairing while making clear that
        // the batch itself owns the observable material passed to inductors.
        episodes.shrink_to_fit();
        Ok(Self { episodes, graphs })
    }

    /// Observed episodes. There is no target-label accessor.
    #[must_use]
    pub fn episodes(&self) -> &[ExperienceEpisode] {
        &self.episodes
    }

    /// Relational observations paired positionally with `episodes()`.
    #[must_use]
    pub fn graphs(&self) -> &[ObservationGraph] {
        &self.graphs
    }

    /// Number of paired observations.
    #[must_use]
    pub fn len(&self) -> usize {
        self.episodes.len()
    }

    /// Batches are validated non-empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        false
    }
}

/// Common API boundary for algorithms that induce structure without evaluator labels.
pub trait LabelFreeInductor<T> {
    /// Algorithm-specific failure.
    type Error;

    /// Induce one result from observation-only material.
    fn induce(&self, batch: &InductionBatch) -> Result<T, Self::Error>;
}

impl core::fmt::Display for InductionInputError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyBatch => formatter.write_str("induction batch must not be empty"),
            Self::MismatchedCardinality => {
                formatter.write_str("induction episodes and graphs must have equal cardinality")
            }
            Self::DuplicateEpisode { episode } => {
                write!(
                    formatter,
                    "episode {} appears more than once",
                    episode.raw()
                )
            }
        }
    }
}

impl std::error::Error for InductionInputError {}

#[cfg(test)]
mod tests {
    use super::{InductionBatch, InductionInputError};
    use crate::experimental::tdi2_intuition::{BooleanState, NumericState};
    use crate::experimental::tdi2_observation_graph::ObservationGraph;
    use crate::experimental::tdi2_template_induction::{
        EpisodeId, ExperienceEpisode, ObservationFrame,
    };

    fn episode(id: u64) -> ExperienceEpisode {
        ExperienceEpisode::new(
            EpisodeId::new(id),
            vec![ObservationFrame::new(
                0,
                NumericState::new(vec![id as f64]).expect("finite"),
                BooleanState::default(),
            )],
        )
        .expect("episode")
    }

    fn graph() -> ObservationGraph {
        ObservationGraph::new(Vec::new(), Vec::new()).expect("empty graph is observable")
    }

    #[test]
    fn batch_surface_contains_only_observations() {
        let batch = InductionBatch::new(vec![episode(1), episode(2)], vec![graph(), graph()])
            .expect("batch");
        assert_eq!(batch.len(), 2);
        assert_eq!(batch.episodes()[0].id(), EpisodeId::new(1));
        assert!(batch.graphs()[0].relations().is_empty());
    }

    #[test]
    fn duplicate_episode_cannot_gain_hidden_weight() {
        assert_eq!(
            InductionBatch::new(vec![episode(3), episode(3)], vec![graph(), graph()]),
            Err(InductionInputError::DuplicateEpisode {
                episode: EpisodeId::new(3)
            })
        );
    }
}
