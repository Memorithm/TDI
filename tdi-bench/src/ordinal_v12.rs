//! Deterministic non-final TDI-12.1 operator-response evaluator.
//!
//! This module reuses the exact finite TDI-10 operator primitives. It does not
//! fit a ranking model, access any protected TDI-7/8/9/10/11 holdout, or make a
//! universality claim. Its role is only to expose reproducible response records
//! that later TDI-12 stages may rank and compare under a preregistered metric.

use tdi_operator::{GreenBands, JacobiMatrix, ResolventError, SchurCavities, ShiftedLdl};

/// Exact finite response observables for one declared Jacobi matrix and shift.
#[derive(Clone, Debug, PartialEq)]
pub struct OperatorResponseV12 {
    /// Matrix dimension.
    pub dimension: usize,
    /// Positive resolvent shift used for all observables.
    pub shift: f64,
    /// Trace of `(K + t I)^(-1)` from selected inversion.
    pub resolvent_trace: f64,
    /// Diagonal Green value at deterministic center index `n / 2`.
    pub center_green: Option<f64>,
    /// Left Schur-cavity denominator at deterministic center index `n / 2`.
    pub center_left_cavity: Option<f64>,
    /// Right Schur-cavity denominator at deterministic center index `n / 2`.
    pub center_right_cavity: Option<f64>,
}

/// Evaluate one finite operator using only generic TDI-10 exact primitives.
pub fn evaluate_operator_response_v12(
    matrix: &JacobiMatrix,
    shift: f64,
) -> Result<OperatorResponseV12, ResolventError> {
    let factor = ShiftedLdl::factor(matrix, shift)?;
    let (resolvent_diagonal, _) = factor.selected_inverse_bands();
    let resolvent_trace = resolvent_diagonal.iter().sum();

    if matrix.is_empty() {
        return Ok(OperatorResponseV12 {
            dimension: 0,
            shift,
            resolvent_trace,
            center_green: None,
            center_left_cavity: None,
            center_right_cavity: None,
        });
    }

    let center = matrix.len() / 2;
    let green = GreenBands::compute(matrix, shift)?;
    let cavities = SchurCavities::compute(matrix, shift)?;

    Ok(OperatorResponseV12 {
        dimension: matrix.len(),
        shift,
        resolvent_trace,
        center_green: Some(green.diagonal()[center]),
        center_left_cavity: Some(cavities.left()[center]),
        center_right_cavity: Some(cavities.right()[center]),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagonal_fixture_matches_exact_responses() {
        let matrix = JacobiMatrix::new(vec![1.0, 2.0, 3.0], vec![0.0, 0.0]).unwrap();
        let response = evaluate_operator_response_v12(&matrix, 1.0).unwrap();

        let expected_trace = 0.5 + (1.0 / 3.0) + 0.25;
        assert_eq!(response.dimension, 3);
        assert_eq!(response.shift, 1.0);
        assert!((response.resolvent_trace - expected_trace).abs() < 1.0e-14);
        assert_eq!(response.center_green, Some(1.0 / 3.0));
        assert_eq!(response.center_left_cavity, Some(3.0));
        assert_eq!(response.center_right_cavity, Some(3.0));
    }

    #[test]
    fn empty_operator_has_no_center_observable() {
        let matrix = JacobiMatrix::new(Vec::new(), Vec::new()).unwrap();
        let response = evaluate_operator_response_v12(&matrix, 1.0).unwrap();

        assert_eq!(response.dimension, 0);
        assert_eq!(response.resolvent_trace, 0.0);
        assert_eq!(response.center_green, None);
        assert_eq!(response.center_left_cavity, None);
        assert_eq!(response.center_right_cavity, None);
    }

    #[test]
    fn invalid_shift_is_rejected_by_tdi10_primitive() {
        let matrix = JacobiMatrix::new(vec![1.0], Vec::new()).unwrap();
        assert!(evaluate_operator_response_v12(&matrix, f64::NAN).is_err());
    }
}
