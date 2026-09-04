use crate::cavity::SchurCavities;
use crate::jacobi::JacobiMatrix;
use crate::resolvent::{ResolventError, checked_pivot, checked_shift};

/// Exact diagonal and first off-diagonal bands of `(K + t I)^(-1)`.
///
/// Status: EXACT finite algebra, conditional on successful positive Schur pivots.
#[derive(Clone, Debug, PartialEq)]
pub struct GreenBands {
    diagonal: Vec<f64>,
    off_diagonal: Vec<f64>,
}

impl GreenBands {
    pub fn compute(matrix: &JacobiMatrix, shift: f64) -> Result<Self, ResolventError> {
        checked_shift(shift)?;
        if matrix.is_empty() {
            return Ok(Self {
                diagonal: Vec::new(),
                off_diagonal: Vec::new(),
            });
        }

        let cavities = SchurCavities::compute(matrix, shift)?;
        let n = matrix.len();
        let mut diagonal = vec![0.0; n];

        for (i, value) in diagonal.iter_mut().enumerate() {
            let mut denominator = matrix.diagonal()[i] + shift;
            if i > 0 {
                let edge = matrix.off_diagonal()[i - 1];
                denominator -= edge * edge / cavities.left()[i - 1];
            }
            if i + 1 < n {
                let edge = matrix.off_diagonal()[i];
                denominator -= edge * edge / cavities.right()[i + 1];
            }
            *value = 1.0 / checked_pivot(i, denominator)?;
        }

        let mut off_diagonal = vec![0.0; n.saturating_sub(1)];
        for (i, value) in off_diagonal.iter_mut().enumerate() {
            *value = -matrix.off_diagonal()[i] * diagonal[i] / cavities.right()[i + 1];
        }

        Ok(Self {
            diagonal,
            off_diagonal,
        })
    }

    #[inline]
    pub fn diagonal(&self) -> &[f64] {
        &self.diagonal
    }

    #[inline]
    pub fn off_diagonal(&self) -> &[f64] {
        &self.off_diagonal
    }
}
