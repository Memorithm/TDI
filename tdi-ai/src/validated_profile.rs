//! Validated recovery profiles; historical raw constructors remain compatible.
use crate::RecoveryProfile;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProfileError {
    NonIncreasingDepth { previous: usize, current: usize },
    NonFiniteScore { depth: usize },
    ScoreOutOfRange { depth: usize },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScoreDomain {
    Finite,
    UnitInterval,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ValidatedProfile<S>(RecoveryProfile<S>);
impl<S> ValidatedProfile<S> {
    pub fn new(profile: RecoveryProfile<S>) -> Result<Self, ProfileError> {
        let mut previous = 0;
        for point in profile.points() {
            if point.depth() <= previous {
                return Err(ProfileError::NonIncreasingDepth {
                    previous,
                    current: point.depth(),
                });
            }
            previous = point.depth();
        }
        Ok(Self(profile))
    }
    pub fn observation_count(&self) -> usize {
        self.0.points().len()
    }
    pub fn last_depth(&self) -> Option<usize> {
        self.0.points().last().map(|p| p.depth())
    }
    pub fn profile(&self) -> &RecoveryProfile<S> {
        &self.0
    }
    pub fn into_profile(self) -> RecoveryProfile<S> {
        self.0
    }
}
impl ValidatedProfile<f64> {
    pub fn validate_scores(self, domain: ScoreDomain) -> Result<Self, ProfileError> {
        for point in self.0.points() {
            if !point.overlap().is_finite() {
                return Err(ProfileError::NonFiniteScore {
                    depth: point.depth(),
                });
            }
            if domain == ScoreDomain::UnitInterval && !(0.0..=1.0).contains(point.overlap()) {
                return Err(ProfileError::ScoreOutOfRange {
                    depth: point.depth(),
                });
            }
        }
        Ok(self)
    }
}
