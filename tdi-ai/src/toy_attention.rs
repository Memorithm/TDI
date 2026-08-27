use core::convert::Infallible;

use crate::{FutureObservable, FutureOverlap, Intervention, ReferenceDynamics};

const ROW_SUM_TOLERANCE: f64 = 1.0e-12;

/// Validation/runtime failures for the deterministic attention fixture.
#[derive(Clone, Debug, PartialEq)]
pub enum ToyAttentionError {
    /// A mixing operator must contain at least one row/token.
    EmptyOperator,
    /// A toy attention state must contain at least one token value.
    EmptyState,
    /// Every mixing row must have exactly as many entries as there are rows.
    NonSquare {
        row: usize,
        expected: usize,
        actual: usize,
    },
    /// A matrix coefficient is not finite.
    NonFiniteWeight { row: usize, column: usize },
    /// This first fixture deliberately restricts itself to non-negative weights.
    NegativeWeight { row: usize, column: usize },
    /// A row does not sum to one within the frozen construction tolerance.
    RowNotNormalized { row: usize, sum: f64 },
    /// A state component is not finite.
    NonFiniteState { index: usize },
    /// The state and mixer dimensions do not agree.
    DimensionMismatch { expected: usize, actual: usize },
    /// An intervention addressed a token outside the current state.
    TokenOutOfBounds { index: usize, len: usize },
    /// A balanced shift must move mass between two distinct token locations.
    IdenticalInterventionTokens { index: usize },
    /// The intervention magnitude must be finite.
    NonFiniteIntervention,
    /// A numerically non-finite mixer output was produced.
    NonFiniteOutput { index: usize },
}

/// Failures specific to the toy recovery metric.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToyRecoveryMetricError {
    /// Reference and perturbed observables must describe the same token count.
    LengthMismatch { reference: usize, perturbed: usize },
    /// The reference observable contains a non-finite component.
    NonFiniteReference { index: usize },
    /// The perturbed observable contains a non-finite component.
    NonFinitePerturbed { index: usize },
}

/// Scalar token state used by the first deterministic attention fixture.
///
/// This is intentionally tiny and framework-independent. It is not a neural
/// tensor type and is not intended to become the public FLAT-ATTENTION state.
#[derive(Clone, Debug, PartialEq)]
pub struct ToyAttentionState {
    values: Vec<f64>,
}

impl ToyAttentionState {
    /// Construct a finite-valued toy state.
    pub fn new(values: Vec<f64>) -> Result<Self, ToyAttentionError> {
        if values.is_empty() {
            return Err(ToyAttentionError::EmptyState);
        }

        if let Some(index) = values.iter().position(|value| !value.is_finite()) {
            return Err(ToyAttentionError::NonFiniteState { index });
        }

        Ok(Self { values })
    }

    /// Construct an all-zero state with the requested token count.
    pub fn zeros(len: usize) -> Result<Self, ToyAttentionError> {
        if len == 0 {
            return Err(ToyAttentionError::EmptyState);
        }
        Ok(Self {
            values: vec![0.0; len],
        })
    }

    /// Token values in sequence order.
    #[must_use]
    pub fn values(&self) -> &[f64] {
        &self.values
    }

    /// Number of scalar token positions.
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Whether the state is empty. Valid constructed states are never empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Sum of all token values, useful for balanced-intervention controls.
    #[must_use]
    pub fn sum(&self) -> f64 {
        self.values.iter().sum()
    }
}

/// A fixed, non-negative, row-stochastic sequence mixer.
///
/// The fixture intentionally does not use learned Q/K projections or softmax.
/// Its role is narrower: provide an analytically tractable attention-like
/// operator through the same TDI-AI intervention/recovery contracts that later
/// adapters will use for real attention semantics.
#[derive(Clone, Debug, PartialEq)]
pub struct FixedAttentionMixer {
    weights: Vec<Vec<f64>>,
}

impl FixedAttentionMixer {
    /// Validate and construct a square row-stochastic mixer.
    pub fn new(weights: Vec<Vec<f64>>) -> Result<Self, ToyAttentionError> {
        if weights.is_empty() {
            return Err(ToyAttentionError::EmptyOperator);
        }

        let dimension = weights.len();
        for (row_index, row) in weights.iter().enumerate() {
            if row.len() != dimension {
                return Err(ToyAttentionError::NonSquare {
                    row: row_index,
                    expected: dimension,
                    actual: row.len(),
                });
            }

            let mut row_sum = 0.0;
            for (column_index, weight) in row.iter().copied().enumerate() {
                if !weight.is_finite() {
                    return Err(ToyAttentionError::NonFiniteWeight {
                        row: row_index,
                        column: column_index,
                    });
                }
                if weight < 0.0 {
                    return Err(ToyAttentionError::NegativeWeight {
                        row: row_index,
                        column: column_index,
                    });
                }
                row_sum += weight;
            }

            if (row_sum - 1.0).abs() > ROW_SUM_TOLERANCE {
                return Err(ToyAttentionError::RowNotNormalized {
                    row: row_index,
                    sum: row_sum,
                });
            }
        }

        Ok(Self { weights })
    }

    /// Token dimension expected by this mixer.
    #[must_use]
    pub fn dimension(&self) -> usize {
        self.weights.len()
    }

    /// Frozen row-stochastic coefficients.
    #[must_use]
    pub fn weights(&self) -> &[Vec<f64>] {
        &self.weights
    }
}

impl ReferenceDynamics for FixedAttentionMixer {
    type State = ToyAttentionState;
    type Error = ToyAttentionError;

    fn advance(&self, state: &Self::State) -> Result<Self::State, Self::Error> {
        let dimension = self.dimension();
        if state.len() != dimension {
            return Err(ToyAttentionError::DimensionMismatch {
                expected: dimension,
                actual: state.len(),
            });
        }

        let mut next = Vec::with_capacity(dimension);
        for (row_index, row) in self.weights.iter().enumerate() {
            let value = row
                .iter()
                .zip(state.values())
                .map(|(weight, state_value)| weight * state_value)
                .sum::<f64>();

            if !value.is_finite() {
                return Err(ToyAttentionError::NonFiniteOutput { index: row_index });
            }
            next.push(value);
        }

        ToyAttentionState::new(next)
    }
}

/// One-shot perturbation that adds `amount` at one token and subtracts the
/// same amount at another token.
///
/// This preserves the total scalar content of the state and therefore isolates
/// redistribution/recovery from a trivial change in the global sum.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BalancedTokenShift {
    add_to: usize,
    subtract_from: usize,
    amount: f64,
}

impl BalancedTokenShift {
    /// Construct a balanced intervention between two distinct token locations.
    pub fn new(
        add_to: usize,
        subtract_from: usize,
        amount: f64,
    ) -> Result<Self, ToyAttentionError> {
        if add_to == subtract_from {
            return Err(ToyAttentionError::IdenticalInterventionTokens { index: add_to });
        }
        if !amount.is_finite() {
            return Err(ToyAttentionError::NonFiniteIntervention);
        }

        Ok(Self {
            add_to,
            subtract_from,
            amount,
        })
    }

    /// Token receiving the positive shift.
    #[must_use]
    pub fn add_to(&self) -> usize {
        self.add_to
    }

    /// Token receiving the negative shift.
    #[must_use]
    pub fn subtract_from(&self) -> usize {
        self.subtract_from
    }

    /// Signed intervention magnitude.
    #[must_use]
    pub fn amount(&self) -> f64 {
        self.amount
    }
}

impl Intervention<ToyAttentionState> for BalancedTokenShift {
    type Error = ToyAttentionError;

    fn apply(&self, reference: &ToyAttentionState) -> Result<ToyAttentionState, Self::Error> {
        let len = reference.len();
        if self.add_to >= len {
            return Err(ToyAttentionError::TokenOutOfBounds {
                index: self.add_to,
                len,
            });
        }
        if self.subtract_from >= len {
            return Err(ToyAttentionError::TokenOutOfBounds {
                index: self.subtract_from,
                len,
            });
        }

        let mut values = reference.values().to_vec();
        values[self.add_to] += self.amount;
        values[self.subtract_from] -= self.amount;
        ToyAttentionState::new(values)
    }
}

/// Observe the complete scalar toy state without transformation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FullStateObservable;

impl FutureObservable<ToyAttentionState> for FullStateObservable {
    type Output = Vec<f64>;
    type Error = Infallible;

    fn observe(
        &self,
        state: &ToyAttentionState,
        _depth: usize,
    ) -> Result<Self::Output, Self::Error> {
        Ok(state.values().to_vec())
    }
}

/// Bounded toy recovery score based on reciprocal L-infinity distance.
///
/// For two observables `x` and `y`,
///
/// `R(x,y) = 1 / (1 + ||x-y||_inf)`.
///
/// `R=1` means identical observables. The score approaches zero as the maximum
/// component-wise discrepancy grows. It is a deterministic research metric,
/// **not** a probability and **not** the historical finite-state TDI overlap.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReciprocalLInfRecovery;

impl FutureOverlap<Vec<f64>> for ReciprocalLInfRecovery {
    type Score = f64;
    type Error = ToyRecoveryMetricError;

    fn overlap(
        &self,
        reference: &Vec<f64>,
        perturbed: &Vec<f64>,
    ) -> Result<Self::Score, Self::Error> {
        if reference.len() != perturbed.len() {
            return Err(ToyRecoveryMetricError::LengthMismatch {
                reference: reference.len(),
                perturbed: perturbed.len(),
            });
        }

        let mut max_distance = 0.0_f64;
        for (index, (reference_value, perturbed_value)) in
            reference.iter().zip(perturbed).enumerate()
        {
            if !reference_value.is_finite() {
                return Err(ToyRecoveryMetricError::NonFiniteReference { index });
            }
            if !perturbed_value.is_finite() {
                return Err(ToyRecoveryMetricError::NonFinitePerturbed { index });
            }

            max_distance = max_distance.max((reference_value - perturbed_value).abs());
        }

        Ok(1.0 / (1.0 + max_distance))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BalancedTokenShift, FixedAttentionMixer, FullStateObservable, ReciprocalLInfRecovery,
        ToyAttentionError, ToyAttentionState,
    };
    use crate::{Intervention, ReferenceDynamics, analyze_intervention_recovery};

    fn averaging_mixer() -> FixedAttentionMixer {
        FixedAttentionMixer::new(vec![
            vec![0.5, 0.5, 0.0],
            vec![0.25, 0.5, 0.25],
            vec![0.0, 0.5, 0.5],
        ])
        .expect("hard-coded fixture is row stochastic")
    }

    fn assert_close(actual: f64, expected: f64) {
        let error = (actual - expected).abs();
        assert!(
            error <= 1.0e-14,
            "expected {expected:.17e}, got {actual:.17e}, absolute error {error:.3e}"
        );
    }

    #[test]
    fn rejects_non_stochastic_or_non_square_mixers() {
        assert_eq!(
            FixedAttentionMixer::new(vec![vec![1.0, 0.0], vec![1.0]]),
            Err(ToyAttentionError::NonSquare {
                row: 1,
                expected: 2,
                actual: 1,
            })
        );

        match FixedAttentionMixer::new(vec![vec![0.6, 0.6], vec![0.5, 0.5]]) {
            Err(ToyAttentionError::RowNotNormalized { row, sum }) => {
                assert_eq!(row, 0);
                assert_close(sum, 1.2);
            }
            other => panic!("expected row-normalization error, got {other:?}"),
        }
    }

    #[test]
    fn balanced_shift_preserves_total_content_and_reference_state() {
        let reference = ToyAttentionState::new(vec![3.0, -1.0, 2.0]).expect("finite state");
        let original = reference.clone();
        let shift = BalancedTokenShift::new(0, 2, 0.5).expect("valid intervention");

        let perturbed = shift.apply(&reference).expect("intervention succeeds");

        assert_eq!(reference, original);
        assert_close(reference.sum(), perturbed.sum());
        assert_eq!(perturbed.values(), &[3.5, -1.0, 1.5]);
    }

    #[test]
    fn analytic_antisymmetric_mode_halves_after_each_attention_step() {
        let mixer = averaging_mixer();
        let initial = ToyAttentionState::zeros(3).expect("non-empty state");
        let shift = BalancedTokenShift::new(0, 2, 1.0).expect("valid intervention");

        let perturbed = shift.apply(&initial).expect("intervention succeeds");
        assert_eq!(perturbed.values(), &[1.0, 0.0, -1.0]);

        let first = mixer.advance(&perturbed).expect("mixer succeeds");
        let second = mixer.advance(&first).expect("mixer succeeds");
        let third = mixer.advance(&second).expect("mixer succeeds");

        assert_eq!(first.values(), &[0.5, 0.0, -0.5]);
        assert_eq!(second.values(), &[0.25, 0.0, -0.25]);
        assert_eq!(third.values(), &[0.125, 0.0, -0.125]);
    }

    #[test]
    fn generic_tdi_ai_protocol_recovers_hand_derived_profile() {
        let mixer = averaging_mixer();
        let initial = ToyAttentionState::zeros(3).expect("non-empty state");
        let shift = BalancedTokenShift::new(0, 2, 1.0).expect("valid intervention");

        let profile = analyze_intervention_recovery(
            &mixer,
            &shift,
            &FullStateObservable,
            &ReciprocalLInfRecovery,
            &initial,
            4,
        )
        .expect("deterministic fixture succeeds");

        let expected = [2.0 / 3.0, 4.0 / 5.0, 8.0 / 9.0, 16.0 / 17.0];
        assert_eq!(profile.horizon(), expected.len());

        for (point, expected_score) in profile.points().iter().zip(expected) {
            assert_close(*point.overlap(), expected_score);
        }
    }

    #[test]
    fn identical_observables_have_unit_recovery() {
        use crate::FutureOverlap;

        let score = ReciprocalLInfRecovery
            .overlap(&vec![1.0, -2.0], &vec![1.0, -2.0])
            .expect("finite equal vectors");
        assert_eq!(score, 1.0);
    }
}
