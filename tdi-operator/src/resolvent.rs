use core::fmt;

use crate::jacobi::JacobiMatrix;

/// Errors from positive shifted-LDL / resolvent calculations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ResolventError {
    InvalidShift { value: f64 },
    NonPositivePivot { index: usize, value: f64 },
}

impl fmt::Display for ResolventError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::InvalidShift { value } => {
                write!(f, "resolvent shift must be finite: {value:e}")
            }
            Self::NonPositivePivot { index, value } => write!(
                f,
                "shifted Jacobi LDL/Schur pivot at index {index} is not finite and strictly positive: {value:e}"
            ),
        }
    }
}

impl std::error::Error for ResolventError {}

pub(crate) fn checked_shift(shift: f64) -> Result<(), ResolventError> {
    if shift.is_finite() {
        Ok(())
    } else {
        Err(ResolventError::InvalidShift { value: shift })
    }
}

pub(crate) fn checked_pivot(index: usize, value: f64) -> Result<f64, ResolventError> {
    if value.is_finite() && value > 0.0 {
        Ok(value)
    } else {
        Err(ResolventError::NonPositivePivot { index, value })
    }
}

/// Positive LDLᵀ factorization of `K + t I` for a finite Jacobi matrix.
///
/// Construction is O(m) time and O(m) memory. A successful factorization is
/// also an executable certificate that the shifted matrix has strictly positive
/// leading LDL pivots in this ordering.
#[derive(Clone, Debug, PartialEq)]
pub struct ShiftedLdl {
    shift: f64,
    pivots: Vec<f64>,
    multipliers: Vec<f64>,
}

impl ShiftedLdl {
    pub fn factor(matrix: &JacobiMatrix, shift: f64) -> Result<Self, ResolventError> {
        checked_shift(shift)?;
        if matrix.is_empty() {
            return Ok(Self {
                shift,
                pivots: Vec::new(),
                multipliers: Vec::new(),
            });
        }

        let n = matrix.len();
        let mut pivots = Vec::with_capacity(n);
        let mut multipliers = Vec::with_capacity(n.saturating_sub(1));
        pivots.push(checked_pivot(0, matrix.diagonal()[0] + shift)?);

        for i in 1..n {
            let edge = matrix.off_diagonal()[i - 1];
            let multiplier = edge / pivots[i - 1];
            let pivot = matrix.diagonal()[i] + shift - multiplier * edge;
            multipliers.push(multiplier);
            pivots.push(checked_pivot(i, pivot)?);
        }

        Ok(Self {
            shift,
            pivots,
            multipliers,
        })
    }

    #[inline]
    pub fn shift(&self) -> f64 {
        self.shift
    }

    #[inline]
    pub fn pivots(&self) -> &[f64] {
        &self.pivots
    }

    #[inline]
    pub fn multipliers(&self) -> &[f64] {
        &self.multipliers
    }

    /// Selected inversion of only the diagonal and first off-diagonal bands.
    ///
    /// This is O(m) and does not construct a dense inverse or eigenvectors.
    pub fn selected_inverse_bands(&self) -> (Vec<f64>, Vec<f64>) {
        if self.pivots.is_empty() {
            return (Vec::new(), Vec::new());
        }

        let n = self.pivots.len();
        let mut diagonal = vec![0.0; n];
        let mut off_diagonal = vec![0.0; n.saturating_sub(1)];
        diagonal[n - 1] = 1.0 / self.pivots[n - 1];

        for i in (0..n - 1).rev() {
            let multiplier = self.multipliers[i];
            off_diagonal[i] = -multiplier * diagonal[i + 1];
            diagonal[i] = 1.0 / self.pivots[i] + multiplier * multiplier * diagonal[i + 1];
        }

        (diagonal, off_diagonal)
    }
}
