use core::fmt;

/// Construction errors for a finite real symmetric tridiagonal matrix.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum JacobiError {
    DimensionMismatch { diagonal: usize, off_diagonal: usize },
    NonFiniteDiagonal { index: usize, value: f64 },
    NonFiniteOffDiagonal { index: usize, value: f64 },
}

impl fmt::Display for JacobiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::DimensionMismatch {
                diagonal,
                off_diagonal,
            } => write!(
                f,
                "Jacobi dimensions are inconsistent: diagonal={diagonal}, off_diagonal={off_diagonal}"
            ),
            Self::NonFiniteDiagonal { index, value } => {
                write!(f, "diagonal[{index}] is not finite: {value:e}")
            }
            Self::NonFiniteOffDiagonal { index, value } => {
                write!(f, "off_diagonal[{index}] is not finite: {value:e}")
            }
        }
    }
}

impl std::error::Error for JacobiError {}

/// Finite real symmetric tridiagonal/Jacobi matrix.
///
/// The matrix is represented by diagonal entries `a_i` and first
/// off-diagonal entries `b_i`, so `b_i` couples rows `i` and `i+1`.
#[derive(Clone, Debug, PartialEq)]
pub struct JacobiMatrix {
    diagonal: Vec<f64>,
    off_diagonal: Vec<f64>,
}

impl JacobiMatrix {
    /// Construct a finite Jacobi matrix after exact dimension and finiteness checks.
    pub fn new(diagonal: Vec<f64>, off_diagonal: Vec<f64>) -> Result<Self, JacobiError> {
        let expected = diagonal.len().saturating_sub(1);
        if off_diagonal.len() != expected {
            return Err(JacobiError::DimensionMismatch {
                diagonal: diagonal.len(),
                off_diagonal: off_diagonal.len(),
            });
        }
        for (index, value) in diagonal.iter().copied().enumerate() {
            if !value.is_finite() {
                return Err(JacobiError::NonFiniteDiagonal { index, value });
            }
        }
        for (index, value) in off_diagonal.iter().copied().enumerate() {
            if !value.is_finite() {
                return Err(JacobiError::NonFiniteOffDiagonal { index, value });
            }
        }
        Ok(Self {
            diagonal,
            off_diagonal,
        })
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.diagonal.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.diagonal.is_empty()
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
