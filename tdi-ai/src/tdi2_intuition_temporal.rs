//! Temporal Boolean structure for TDI-2.1 experiential templates.

use super::tdi2_intuition::{BooleanState, NumericState, PredicateId};

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

/// Ordered Boolean observations forming one temporal experience template.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BooleanSequence {
    frames: Vec<BooleanState>,
}

impl BooleanSequence {
    /// Construct an ordered sequence. Empty sequences are allowed for explicit boundary tests.
    #[must_use]
    pub fn new(frames: Vec<BooleanState>) -> Self {
        Self { frames }
    }

    /// Ordered Boolean frames.
    #[must_use]
    pub fn frames(&self) -> &[BooleanState] {
        &self.frames
    }

    /// Number of frames.
    #[must_use]
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    /// Whether the sequence has no frames.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }
}

/// Exact frame-by-frame temporal match.
#[must_use]
pub fn exact_sequence_match(template: &BooleanSequence, query: &BooleanSequence) -> bool {
    template == query
}

/// Fraction of same-position frames that match exactly.
///
/// Returns `None` when sequence lengths differ or both are empty, preserving the
/// distinction between incompatible horizons and observed partial agreement.
#[must_use]
pub fn frame_match_fraction(
    template: &BooleanSequence,
    query: &BooleanSequence,
) -> Option<f64> {
    if template.len() != query.len() || template.is_empty() {
        return None;
    }
    let matched = template
        .frames()
        .iter()
        .zip(query.frames())
        .filter(|(left, right)| left == right)
        .count();
    Some(matched as f64 / template.len() as f64)
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

/// Encode all true temporal predicates into one canonical Boolean state.
pub fn encode_temporal(
    previous: &NumericState,
    current: &NumericState,
    predicates: &[DeltaPredicate],
) -> Result<BooleanState, TemporalError> {
    let mut active = Vec::new();
    for predicate in predicates {
        if predicate.evaluate(previous, current)? {
            active.push(predicate.id);
        }
    }
    Ok(BooleanState::new(active))
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
    use super::{
        BooleanSequence, DeltaDirection, DeltaPredicate, encode_temporal, exact_sequence_match,
        frame_match_fraction,
    };
    use crate::experimental::tdi2_intuition::{BooleanState, NumericState, PredicateId};

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

    #[test]
    fn temporal_encoder_emits_only_true_delta_predicates() {
        let previous = NumericState::new(vec![1.0, 4.0]).expect("state");
        let current = NumericState::new(vec![3.0, 4.1]).expect("state");
        let predicates = [
            DeltaPredicate::new(PredicateId::new(1), 0, DeltaDirection::Increase, 1.0)
                .expect("predicate"),
            DeltaPredicate::new(PredicateId::new(2), 1, DeltaDirection::Stable, 0.2)
                .expect("predicate"),
        ];
        let encoded = encode_temporal(&previous, &current, &predicates).expect("encode");
        assert_eq!(encoded.predicates(), &[PredicateId::new(1), PredicateId::new(2)]);
    }

    #[test]
    fn boolean_sequence_preserves_temporal_order() {
        let first = BooleanState::new(vec![PredicateId::new(1)]);
        let second = BooleanState::new(vec![PredicateId::new(2)]);
        let sequence = BooleanSequence::new(vec![first.clone(), second.clone()]);
        assert_eq!(sequence.frames(), &[first, second]);
    }

    #[test]
    fn sequence_matching_respects_frame_order() {
        let first = BooleanState::new(vec![PredicateId::new(1)]);
        let second = BooleanState::new(vec![PredicateId::new(2)]);
        let template = BooleanSequence::new(vec![first.clone(), second.clone()]);
        let reversed = BooleanSequence::new(vec![second, first]);
        assert!(!exact_sequence_match(&template, &reversed));
        assert_eq!(frame_match_fraction(&template, &reversed), Some(0.0));
    }
}
