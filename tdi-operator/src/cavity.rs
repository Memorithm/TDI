use crate::jacobi::JacobiMatrix;
use crate::resolvent::{ResolventError, checked_pivot, checked_shift};

/// Exact finite left/right Schur cavity denominators for `K + t I`.
///
/// Status: EXACT finite algebra, conditional only on successful positive pivots.
#[derive(Clone, Debug, PartialEq)]
pub struct SchurCavities {
    left: Vec<f64>,
    right: Vec<f64>,
}

impl SchurCavities {
    pub fn compute(matrix: &JacobiMatrix, shift: f64) -> Result<Self, ResolventError> {
        checked_shift(shift)?;
        if matrix.is_empty() {
            return Ok(Self {
                left: Vec::new(),
                right: Vec::new(),
            });
        }

        let n = matrix.len();
        let mut left = vec![0.0; n];
        let mut right = vec![0.0; n];

        left[0] = checked_pivot(0, matrix.diagonal()[0] + shift)?;
        for i in 1..n {
            let edge = matrix.off_diagonal()[i - 1];
            let denominator = matrix.diagonal()[i] + shift - edge * edge / left[i - 1];
            left[i] = checked_pivot(i, denominator)?;
        }

        let last = n - 1;
        right[last] = checked_pivot(last, matrix.diagonal()[last] + shift)?;
        for i in (0..last).rev() {
            let edge = matrix.off_diagonal()[i];
            let denominator = matrix.diagonal()[i] + shift - edge * edge / right[i + 1];
            right[i] = checked_pivot(i, denominator)?;
        }

        Ok(Self { left, right })
    }

    #[inline]
    pub fn left(&self) -> &[f64] {
        &self.left
    }

    #[inline]
    pub fn right(&self) -> &[f64] {
        &self.right
    }
}
