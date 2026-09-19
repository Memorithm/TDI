//! Label-leakage boundary for TDI-2.2 induction algorithms.
//!
//! Future template inductors receive `InductionBatch` rather than evaluator
//! cases. The batch contains observed episodes and graphs only; expected labels,
//! expected template identities and expected role mappings have no field here.

use super::tdi2_induction_split::{InductionDomain, frozen_population};
use super::tdi2_observation_graph::ObservationGraph;
use super::tdi2_template_induction::{EpisodeId, ExperienceEpisode};

/// Observation-only batch passed into template induction.
#[derive(Clone, Debug, PartialEq)]
pub struct InductionBatch {
    domain: InductionDomain,
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
    /// Validation content is not yet bound to an immutable manifest/generator.
    ValidationPopulationUnbound,
    /// An episode falls outside the selected frozen non-final population.
    EpisodeOutsidePopulation {
        domain: InductionDomain,
        episode: EpisodeId,
    },
    /// A relational graph is bound to a different episode than its paired frames.
    GraphEpisodeMismatch {
        episode: EpisodeId,
        graph_episode: EpisodeId,
    },
}

impl InductionBatch {
    /// Construct a validated observation-only induction batch.
    pub fn new(
        domain: InductionDomain,
        mut episodes: Vec<ExperienceEpisode>,
        graphs: Vec<ObservationGraph>,
    ) -> Result<Self, InductionInputError> {
        if episodes.is_empty() {
            return Err(InductionInputError::EmptyBatch);
        }
        if episodes.len() != graphs.len() {
            return Err(InductionInputError::MismatchedCardinality);
        }
        // Merely reserving Validation IDs does not freeze Validation contents. Until
        // this campaign binds those IDs to an immutable content manifest or a
        // deterministic generator, accepting caller-supplied Validation episodes
        // would permit post-observation replacement. Keep that boundary fail-closed.
        if domain == InductionDomain::Validation {
            return Err(InductionInputError::ValidationPopulationUnbound);
        }
        let population = frozen_population(domain);
        for episode in &episodes {
            if !population.contains(episode.id()) {
                return Err(InductionInputError::EpisodeOutsidePopulation {
                    domain,
                    episode: episode.id(),
                });
            }
        }
        for (episode, graph) in episodes.iter().zip(&graphs) {
            if graph.episode() != episode.id() {
                return Err(InductionInputError::GraphEpisodeMismatch {
                    episode: episode.id(),
                    graph_episode: graph.episode(),
                });
            }
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
        episodes.shrink_to_fit();
        Ok(Self {
            domain,
            episodes,
            graphs,
        })
    }

    /// Frozen non-final population represented by this batch.
    #[must_use]
    pub const fn domain(&self) -> InductionDomain {
        self.domain
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
            Self::ValidationPopulationUnbound => formatter.write_str(
                "validation population contents are not yet bound to an immutable manifest or deterministic generator",
            ),
            Self::DuplicateEpisode { episode } => {
                write!(
                    formatter,
                    "episode {} appears more than once",
                    episode.raw()
                )
            }
            Self::EpisodeOutsidePopulation { domain, episode } => write!(
                formatter,
                "episode {} is outside the frozen {domain:?} population",
                episode.raw()
            ),
            Self::GraphEpisodeMismatch {
                episode,
                graph_episode,
            } => write!(
                formatter,
                "episode {} is paired with graph for episode {}",
                episode.raw(),
                graph_episode.raw()
            ),
        }
    }
}

impl std::error::Error for InductionInputError {}

#[cfg(test)]
mod tests {
    use super::{InductionBatch, InductionInputError};
    use crate::experimental::tdi2_induction_split::{
        DEVELOPMENT_START, InductionDomain, VALIDATION_START,
    };
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

    fn graph(episode: u64) -> ObservationGraph {
        ObservationGraph::new(EpisodeId::new(episode), Vec::new(), Vec::new())
            .expect("empty graph is observable")
    }

    #[test]
    fn batch_surface_contains_only_bound_observations() {
        let first = DEVELOPMENT_START;
        let second = DEVELOPMENT_START + 1;
        let batch = InductionBatch::new(
            InductionDomain::Development,
            vec![episode(first), episode(second)],
            vec![graph(first), graph(second)],
        )
        .expect("batch");
        assert_eq!(batch.domain(), InductionDomain::Development);
        assert_eq!(batch.len(), 2);
        assert_eq!(batch.episodes()[0].id(), EpisodeId::new(first));
        assert_eq!(batch.graphs()[0].episode(), EpisodeId::new(first));
        assert!(batch.graphs()[0].relations().is_empty());
    }

    #[test]
    fn swapped_graphs_are_rejected() {
        let first = DEVELOPMENT_START;
        let second = DEVELOPMENT_START + 1;
        assert_eq!(
            InductionBatch::new(
                InductionDomain::Development,
                vec![episode(first), episode(second)],
                vec![graph(second), graph(first)],
            ),
            Err(InductionInputError::GraphEpisodeMismatch {
                episode: EpisodeId::new(first),
                graph_episode: EpisodeId::new(second),
            })
        );
    }

    #[test]
    fn validation_is_fail_closed_until_contents_are_immutably_frozen() {
        assert_eq!(
            InductionBatch::new(
                InductionDomain::Validation,
                vec![episode(VALIDATION_START)],
                vec![graph(VALIDATION_START)],
            ),
            Err(InductionInputError::ValidationPopulationUnbound)
        );
    }

    #[test]
    fn development_rejects_validation_population_ids() {
        assert_eq!(
            InductionBatch::new(
                InductionDomain::Development,
                vec![episode(DEVELOPMENT_START), episode(VALIDATION_START)],
                vec![graph(DEVELOPMENT_START), graph(VALIDATION_START)],
            ),
            Err(InductionInputError::EpisodeOutsidePopulation {
                domain: InductionDomain::Development,
                episode: EpisodeId::new(VALIDATION_START),
            })
        );
    }

    #[test]
    fn duplicate_episode_cannot_gain_hidden_weight() {
        let id = DEVELOPMENT_START;
        assert_eq!(
            InductionBatch::new(
                InductionDomain::Development,
                vec![episode(id), episode(id)],
                vec![graph(id), graph(id)],
            ),
            Err(InductionInputError::DuplicateEpisode {
                episode: EpisodeId::new(id)
            })
        );
    }
}
