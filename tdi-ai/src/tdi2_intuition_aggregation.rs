//! Numeric aggregation for multiple applicable TDI-2.1 experiential proposals.

/// One scalar proposal emitted by an applicable template.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WeightedProposal {
    value: f64,
    weight: f64,
}

impl WeightedProposal {
    /// Construct a scalar proposal. Validation is performed by aggregation.
    #[must_use]
    pub const fn new(value: f64, weight: f64) -> Self {
        Self { value, weight }
    }

    /// Proposed scalar value.
    #[must_use]
    pub const fn value(self) -> f64 {
        self.value
    }

    /// Evidence weight attached to the proposal.
    #[must_use]
    pub const fn weight(self) -> f64 {
        self.weight
    }
}

/// Fail-closed aggregation errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AggregationError {
    /// No proposal was supplied.
    Empty,
    /// One proposal value was NaN or infinite.
    NonFiniteValue { index: usize },
    /// One proposal weight was NaN, infinite, or negative.
    InvalidWeight { index: usize },
    /// Every valid proposal had zero weight.
    ZeroTotalWeight,
    /// Floating-point accumulation overflowed to a non-finite value.
    NonFiniteAggregate,
}

impl core::fmt::Display for AggregationError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Empty => formatter.write_str("no intuition proposal to aggregate"),
            Self::NonFiniteValue { index } => {
                write!(formatter, "proposal value at index {index} is not finite")
            }
            Self::InvalidWeight { index } => {
                write!(formatter, "proposal weight at index {index} is invalid")
            }
            Self::ZeroTotalWeight => formatter.write_str("intuition proposal total weight is zero"),
            Self::NonFiniteAggregate => formatter.write_str("intuition aggregate is not finite"),
        }
    }
}

impl std::error::Error for AggregationError {}

/// Compute a deterministic weighted mean over scalar proposals.
pub fn weighted_mean(proposals: &[WeightedProposal]) -> Result<f64, AggregationError> {
    if proposals.is_empty() {
        return Err(AggregationError::Empty);
    }

    let mut weighted_sum = 0.0;
    let mut total_weight = 0.0;
    for (index, proposal) in proposals.iter().copied().enumerate() {
        if !proposal.value().is_finite() {
            return Err(AggregationError::NonFiniteValue { index });
        }
        if !proposal.weight().is_finite() || proposal.weight() < 0.0 {
            return Err(AggregationError::InvalidWeight { index });
        }
        weighted_sum += proposal.value() * proposal.weight();
        total_weight += proposal.weight();
    }

    if total_weight == 0.0 {
        return Err(AggregationError::ZeroTotalWeight);
    }
    if !weighted_sum.is_finite() || !total_weight.is_finite() {
        return Err(AggregationError::NonFiniteAggregate);
    }

    let result = weighted_sum / total_weight;
    if !result.is_finite() {
        return Err(AggregationError::NonFiniteAggregate);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::{AggregationError, WeightedProposal, weighted_mean};

    #[test]
    fn weighted_mean_respects_experience_weights() {
        let result = weighted_mean(&[
            WeightedProposal::new(10.0, 1.0),
            WeightedProposal::new(20.0, 3.0),
        ])
        .expect("valid proposals");
        assert!((result - 17.5).abs() < f64::EPSILON);
    }

    #[test]
    fn invalid_weight_fails_closed() {
        assert_eq!(
            weighted_mean(&[WeightedProposal::new(1.0, -1.0)]),
            Err(AggregationError::InvalidWeight { index: 0 })
        );
    }
}
