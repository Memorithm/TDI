//! Predicate specifications for converting numeric observations into Boolean structure.

use super::tdi2_intuition::{NumericState, PredicateId};

/// Direction of a scalar threshold predicate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Comparison {
    /// Feature is greater than or equal to the threshold.
    GreaterOrEqual,
    /// Feature is less than or equal to the threshold.
    LessOrEqual,
}

/// One frozen scalar predicate definition.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScalarPredicate {
    /// Stable predicate id emitted when the clause is true.
    pub id: PredicateId,
    /// Numeric feature index.
    pub feature: usize,
    /// Comparison direction.
    pub comparison: Comparison,
    /// Frozen finite threshold.
    pub threshold: f64,
}

/// Predicate-spec validation errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PredicateError {
    /// Threshold is NaN or infinite.
    NonFiniteThreshold,
    /// Feature index is outside the input state.
    FeatureOutOfBounds,
}

impl ScalarPredicate {
    /// Validate one scalar predicate.
    pub fn new(
        id: PredicateId,
        feature: usize,
        comparison: Comparison,
        threshold: f64,
    ) -> Result<Self, PredicateError> {
        if !threshold.is_finite() {
            return Err(PredicateError::NonFiniteThreshold);
        }
        Ok(Self { id, feature, comparison, threshold })
    }

    /// Evaluate the predicate against a validated numeric state.
    pub fn evaluate(self, state: &NumericState) -> Result<bool, PredicateError> {
        let value = *state.values().get(self.feature).ok_or(PredicateError::FeatureOutOfBounds)?;
        Ok(match self.comparison {
            Comparison::GreaterOrEqual => value >= self.threshold,
            Comparison::LessOrEqual => value <= self.threshold,
        })
    }
}

impl core::fmt::Display for PredicateError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NonFiniteThreshold => formatter.write_str("predicate threshold must be finite"),
            Self::FeatureOutOfBounds => formatter.write_str("predicate feature index is out of bounds"),
        }
    }
}
impl std::error::Error for PredicateError {}

#[cfg(test)]
mod tests {
    use super::{Comparison, ScalarPredicate};
    use crate::experimental::tdi2_intuition::{NumericState, PredicateId};

    #[test]
    fn scalar_predicate_is_deterministic() {
        let state = NumericState::new(vec![1.0, 3.0]).expect("state");
        let predicate = ScalarPredicate::new(PredicateId::new(7), 1, Comparison::GreaterOrEqual, 2.0)
            .expect("predicate");
        assert_eq!(predicate.evaluate(&state), Ok(true));
    }
}
