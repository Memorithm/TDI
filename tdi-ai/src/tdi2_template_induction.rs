//! Label-free observation and episode IR for TDI-2.2 template induction.
//!
//! The induction surface deliberately contains no expected template id, target
//! class, correct role mapping, evaluator verdict, or latency field.

use super::tdi2_intuition::{BooleanState, NumericState};

/// Stable identifier for one observed experience episode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EpisodeId(u64);

impl EpisodeId {
    /// Construct an episode identifier.
    #[must_use]
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// Canonical integer representation.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// One ordered observation frame available to the induction algorithm.
#[derive(Clone, Debug, PartialEq)]
pub struct ObservationFrame {
    ordinal: u32,
    numeric: NumericState,
    observed_predicates: BooleanState,
}

impl ObservationFrame {
    /// Construct one frame from observable state only.
    #[must_use]
    pub fn new(ordinal: u32, numeric: NumericState, observed_predicates: BooleanState) -> Self {
        Self {
            ordinal,
            numeric,
            observed_predicates,
        }
    }

    /// Position within the episode.
    #[must_use]
    pub const fn ordinal(&self) -> u32 {
        self.ordinal
    }

    /// Numeric observation available before induction.
    #[must_use]
    pub const fn numeric(&self) -> &NumericState {
        &self.numeric
    }

    /// Predicate observations available before induction.
    #[must_use]
    pub const fn observed_predicates(&self) -> &BooleanState {
        &self.observed_predicates
    }
}

/// One label-free experience episode.
#[derive(Clone, Debug, PartialEq)]
pub struct ExperienceEpisode {
    id: EpisodeId,
    frames: Vec<ObservationFrame>,
}

/// Episode construction failures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EpisodeError {
    /// An episode must contain at least one observation.
    EmptyEpisode,
    /// Observation ordinals must increase strictly.
    NonIncreasingOrdinal,
}

impl ExperienceEpisode {
    /// Construct a validated label-free episode.
    pub fn new(id: EpisodeId, frames: Vec<ObservationFrame>) -> Result<Self, EpisodeError> {
        if frames.is_empty() {
            return Err(EpisodeError::EmptyEpisode);
        }
        if frames
            .windows(2)
            .any(|pair| pair[0].ordinal() >= pair[1].ordinal())
        {
            return Err(EpisodeError::NonIncreasingOrdinal);
        }
        Ok(Self { id, frames })
    }

    /// Episode identity. It is provenance, not a prediction target.
    #[must_use]
    pub const fn id(&self) -> EpisodeId {
        self.id
    }

    /// Ordered observable frames.
    #[must_use]
    pub fn frames(&self) -> &[ObservationFrame] {
        &self.frames
    }

    /// Number of observation frames.
    #[must_use]
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    /// Episodes are validated non-empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        false
    }
}

impl core::fmt::Display for EpisodeError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyEpisode => formatter.write_str("experience episode must not be empty"),
            Self::NonIncreasingOrdinal => {
                formatter.write_str("experience observation ordinals must increase strictly")
            }
        }
    }
}

impl std::error::Error for EpisodeError {}

#[cfg(test)]
mod tests {
    use super::{EpisodeError, EpisodeId, ExperienceEpisode, ObservationFrame};
    use crate::experimental::tdi2_intuition::{BooleanState, NumericState, PredicateId};

    fn frame(ordinal: u32, value: f64) -> ObservationFrame {
        ObservationFrame::new(
            ordinal,
            NumericState::new(vec![value]).expect("finite state"),
            BooleanState::new(vec![PredicateId::new(ordinal + 1)]),
        )
    }

    #[test]
    fn episode_contains_observations_but_no_target_label_surface() {
        let episode = ExperienceEpisode::new(EpisodeId::new(7), vec![frame(0, 1.0), frame(1, 2.0)])
            .expect("episode");
        assert_eq!(episode.id().raw(), 7);
        assert_eq!(episode.len(), 2);
        assert_eq!(episode.frames()[1].numeric().values(), &[2.0]);
    }

    #[test]
    fn episode_rejects_empty_or_reordered_observations() {
        assert_eq!(
            ExperienceEpisode::new(EpisodeId::new(1), Vec::new()),
            Err(EpisodeError::EmptyEpisode)
        );
        assert_eq!(
            ExperienceEpisode::new(EpisodeId::new(1), vec![frame(2, 1.0), frame(2, 2.0)]),
            Err(EpisodeError::NonIncreasingOrdinal)
        );
    }
}
