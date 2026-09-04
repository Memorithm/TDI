use core::fmt;

/// Errors constructing a strictly positive constant two-sided Jacobi symbol.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FrozenToeplitzError {
    NonFiniteDiagonal { value: f64 },
    NonFiniteEdge { value: f64 },
    NonPositiveSymbol { diagonal: f64, edge: f64 },
}

impl fmt::Display for FrozenToeplitzError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::NonFiniteDiagonal { value } => {
                write!(f, "frozen diagonal coefficient is not finite: {value:e}")
            }
            Self::NonFiniteEdge { value } => {
                write!(f, "frozen edge coefficient is not finite: {value:e}")
            }
            Self::NonPositiveSymbol { diagonal, edge } => write!(
                f,
                "constant Jacobi symbol is not strictly positive: diagonal={diagonal:e}, edge={edge:e}; require diagonal > 2*abs(edge)"
            ),
        }
    }
}

impl std::error::Error for FrozenToeplitzError {}

/// Exact positive fixed-point and Green data for a constant two-sided Jacobi operator.
///
/// For the constant operator with diagonal `a` and signed edge `b`, strict
/// positivity of the Fourier symbol `a + 2 b cos(theta)` is equivalent to
/// `a > 2 |b|`. Under that assumption:
///
/// `q = (a + sqrt(a^2 - 4 b^2)) / 2`
///
/// is the positive stable fixed point of `F(x)=a-b^2/x`, while the selected
/// two-sided Green entries are
///
/// `G_00 = 1 / sqrt(a^2 - 4 b^2)`
///
/// and
///
/// `G_01 = -b G_00 / q`.
///
/// Scientific status: EXACT for this constant infinite model. This type makes
/// no statement about finite-section convergence or slowly varying operators.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrozenToeplitzCavity {
    diagonal: f64,
    edge: f64,
    discriminant: f64,
    cavity: f64,
    green_diagonal: f64,
    green_off_diagonal: f64,
    contraction: f64,
}

impl FrozenToeplitzCavity {
    pub fn new(diagonal: f64, edge: f64) -> Result<Self, FrozenToeplitzError> {
        if !diagonal.is_finite() {
            return Err(FrozenToeplitzError::NonFiniteDiagonal { value: diagonal });
        }
        if !edge.is_finite() {
            return Err(FrozenToeplitzError::NonFiniteEdge { value: edge });
        }
        if diagonal <= 2.0 * edge.abs() {
            return Err(FrozenToeplitzError::NonPositiveSymbol { diagonal, edge });
        }

        // The positivity check above implies a strictly positive exact-real
        // discriminant. The product form is slightly better behaved close to
        // the edge than subtracting two nearly equal squared quantities.
        let discriminant = (diagonal - 2.0 * edge.abs()) * (diagonal + 2.0 * edge.abs());
        let root = discriminant.sqrt();
        let cavity = 0.5 * (diagonal + root);
        let green_diagonal = 1.0 / root;
        let green_off_diagonal = -edge * green_diagonal / cavity;
        let edge_ratio = edge / cavity;
        let contraction = edge_ratio * edge_ratio;

        Ok(Self {
            diagonal,
            edge,
            discriminant,
            cavity,
            green_diagonal,
            green_off_diagonal,
            contraction,
        })
    }

    #[inline]
    pub fn diagonal(self) -> f64 {
        self.diagonal
    }

    #[inline]
    pub fn edge(self) -> f64 {
        self.edge
    }

    #[inline]
    pub fn discriminant(self) -> f64 {
        self.discriminant
    }

    #[inline]
    pub fn cavity(self) -> f64 {
        self.cavity
    }

    #[inline]
    pub fn green_diagonal(self) -> f64 {
        self.green_diagonal
    }

    #[inline]
    pub fn green_off_diagonal(self) -> f64 {
        self.green_off_diagonal
    }

    /// Exact derivative `F'(q)=b^2/q^2` of the frozen cavity map.
    #[inline]
    pub fn contraction(self) -> f64 {
        self.contraction
    }
}
