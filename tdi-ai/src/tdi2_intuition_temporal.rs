//! Temporal Boolean structure for TDI-2.1 experiential templates.

use super::tdi2_intuition::{NumericState, PredicateId};

/// Direction of a numeric change between two observations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeltaDirection {
    /// Current value exceeds previous value by at least the threshold.
    Increase,
    /// Current value is below previous value by at least the threshold.
    Decrease,
    /// Absolute change is no greater than the threshold.
    Stable,
}

/// Frozen predicate over one feature delta.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DeltaPredicate {
    /// Boolean predicate emitted when the temporal clause is true.
    pub id: PredicateId,
    /// Feature index in both numeric states.
    pub feature: usize,
    /// Temporal direction to test.
    pub direction: DeltaDirection,
    /// Non-negative finite delta threshold.
    pub threshold: f64,
}

/// Temporal predicate validation failures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemporalError {
    /// Threshold is non-finite or negative.
    InvalidThreshold,
    /// Feature is absent from one of the states.
    FeatureOutOfBounds,
}

impl DeltaPredicate {
    /// Construct a validated delta predicate.
    pub fn new(
        id: PredicateId,
        feature: usize,
        direction: DeltaDirection,
        threshold: f64,
    ) -> Result<Self, TemporalError> {
        if !threshold.is_finite() || threshold < 0.0 {
            return Err(TemporalError::InvalidThreshold);
        }
        Ok(Self { id, feature, direction, threshold })
    }

    /// Evaluate the temporal clause between previous and current states.
    pub fn evaluate(
        self,
        previous: &NumericState,
        current: &NumericState,
    ) -> Result<bool, TemporalError> {
        let before = *previous.values().get(self.feature).ok_or(TemporalError::FeatureOutOfBounds)?;
        let after = *current.values().get(self.feature).ok_or(TemporalError::FeatureOutOfBounds)?;
        let delta = after - before;
        Ok(match self.direction {
            DeltaDirection::Increase => delta >= self.threshold,
            DeltaDirection::Decrease => -delta >= self.threshold,
            DeltaDirection::Stable => delta.abs() <= self.threshold,
        })
    }
}

impl core::fmt::Display for TemporalError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidThreshold => formatter.write_str("temporal threshold must be finite and non-negative"),
            Self::FeatureOutOfBounds => formatter.write_str("temporal feature index is out of bounds"),
        }
    }
}
impl std::error::Error for TemporalError {}

#[cfg(test)]
mod tests {
    use super::{DeltaDirection, DeltaPredicate};
    use crate::experimental::tdi2_intuition::{NumericState, PredicateId};

    #[test]
    fn increase_predicate_detects_direction() {
        let previous = NumericState::new(vec![1.0]).expect("state");
        let current = NumericState::new(vec![2.5]).expect("state");
        let predicate = DeltaPredicate::new(
            PredicateId::new(100),
            0,
            DeltaDirection::Increase,
            1.0,
        )
        .expect("predicate");
        assert_eq!(predicate.evaluate(&previous, &current), Ok(true));
    }
}
