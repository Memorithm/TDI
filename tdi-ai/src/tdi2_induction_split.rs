//! Frozen non-final episode populations for TDI-2.2 induction research.
//!
//! This module exposes Development and Validation only. There is deliberately no
//! protected/final/PrimaryHoldout domain in this campaign surface.

use super::tdi2_template_induction::EpisodeId;

/// Allowed non-final TDI-2.2 population domains.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InductionDomain {
    /// Iterative algorithm development.
    Development,
    /// Frozen disjoint non-final validation.
    Validation,
}

/// First Development episode id reserved for TDI-2.2.
pub const DEVELOPMENT_START: u64 = 100_000;
/// First Validation episode id reserved for TDI-2.2.
pub const VALIDATION_START: u64 = 200_000;
/// Number of episode ids frozen for each non-final domain.
pub const DOMAIN_EPISODES: usize = 256;

/// Frozen checked episode-id range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EpisodePopulation {
    domain: InductionDomain,
    start: u64,
    count: usize,
}

/// Population derivation failures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PopulationError {
    /// Requested count cannot be represented in the id range.
    CountOverflow,
    /// Last id would overflow u64.
    IdOverflow,
}

/// Return the frozen population descriptor for one allowed domain.
#[must_use]
pub const fn frozen_population(domain: InductionDomain) -> EpisodePopulation {
    let start = match domain {
        InductionDomain::Development => DEVELOPMENT_START,
        InductionDomain::Validation => VALIDATION_START,
    };
    EpisodePopulation {
        domain,
        start,
        count: DOMAIN_EPISODES,
    }
}

impl EpisodePopulation {
    /// Frozen population domain.
    #[must_use]
    pub const fn domain(self) -> InductionDomain {
        self.domain
    }

    /// First frozen episode id.
    #[must_use]
    pub const fn start(self) -> u64 {
        self.start
    }

    /// Number of frozen episode ids.
    #[must_use]
    pub const fn count(self) -> usize {
        self.count
    }

    /// Whether an episode belongs to this exact frozen non-final population.
    #[must_use]
    pub fn contains(self, episode: EpisodeId) -> bool {
        let Ok(count) = u64::try_from(self.count) else {
            return false;
        };
        let Some(end_exclusive) = self.start.checked_add(count) else {
            return false;
        };
        episode.raw() >= self.start && episode.raw() < end_exclusive
    }

    /// Materialize deterministic episode ids without hidden RNG state.
    pub fn episode_ids(self) -> Result<Vec<EpisodeId>, PopulationError> {
        let count = u64::try_from(self.count).map_err(|_| PopulationError::CountOverflow)?;
        let _end_exclusive = self
            .start
            .checked_add(count)
            .ok_or(PopulationError::IdOverflow)?;
        Ok((0..count)
            .map(|offset| EpisodeId::new(self.start + offset))
            .collect())
    }
}

impl core::fmt::Display for PopulationError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::CountOverflow => formatter.write_str("episode population count overflows u64"),
            Self::IdOverflow => formatter.write_str("episode population id range overflows u64"),
        }
    }
}

impl std::error::Error for PopulationError {}

#[cfg(test)]
mod tests {
    use super::{
        DEVELOPMENT_START, DOMAIN_EPISODES, InductionDomain, VALIDATION_START, frozen_population,
    };
    use crate::experimental::tdi2_template_induction::EpisodeId;

    #[test]
    fn frozen_populations_are_disjoint_and_equal_sized() {
        let development = frozen_population(InductionDomain::Development);
        let validation = frozen_population(InductionDomain::Validation);
        assert_eq!(development.domain(), InductionDomain::Development);
        assert_eq!(validation.domain(), InductionDomain::Validation);
        assert_eq!(development.start(), DEVELOPMENT_START);
        assert_eq!(validation.start(), VALIDATION_START);
        assert_eq!(development.count(), DOMAIN_EPISODES);
        assert_eq!(validation.count(), DOMAIN_EPISODES);
        assert!(development.start() + development.count() as u64 <= validation.start());
        assert!(development.contains(EpisodeId::new(DEVELOPMENT_START)));
        assert!(development.contains(EpisodeId::new(
            DEVELOPMENT_START + DOMAIN_EPISODES as u64 - 1
        )));
        assert!(!development.contains(EpisodeId::new(VALIDATION_START)));
    }

    #[test]
    fn population_ids_are_deterministic_and_contiguous() {
        let ids = frozen_population(InductionDomain::Development)
            .episode_ids()
            .expect("bounded population");
        assert_eq!(ids.len(), DOMAIN_EPISODES);
        assert_eq!(ids[0].raw(), DEVELOPMENT_START);
        assert_eq!(ids[1].raw(), DEVELOPMENT_START + 1);
    }
}
