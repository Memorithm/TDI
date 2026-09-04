use core::fmt;

use crate::transport::CavityTransportStep;

/// Fail-closed errors for the exact TDI-10.3 factorization layer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CavityFactorizationError {
    NonFiniteReferenceEdge { value: f64 },
    NonFiniteDerivedQuantity,
}

impl fmt::Display for CavityFactorizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::NonFiniteReferenceEdge { value } => {
                write!(f, "local reference edge is not finite: {value:e}")
            }
            Self::NonFiniteDerivedQuantity => {
                write!(
                    f,
                    "cavity factorization produced a non-finite derived quantity"
                )
            }
        }
    }
}

impl std::error::Error for CavityFactorizationError {}

/// Exact algebraic factorization of one TDI-10.2 transport step.
///
/// In addition to the positive references `q_j,q_i` already carried by the
/// transport step, the caller supplies a finite local reference edge `b_i`.
/// No rule for constructing `b_i` is assumed.
///
/// The TDI-10.2 drift
///
/// `delta = a_i - e^2/q_j - q_i`
///
/// is decomposed exactly as
///
/// `delta = reference_defect + edge_drift + reference_drift`,
///
/// where
///
/// `reference_defect = a_i - b_i^2/q_i - q_i`,
///
/// `edge_drift = (b_i^2-e^2)/q_i`,
///
/// `reference_drift = e^2(1/q_i-1/q_j)`.
///
/// The exact transport multiplier is also factored as
///
/// `alpha = normalized_edge_square * cavity_correction`,
///
/// with
///
/// `normalized_edge_square = (e/q_j)^2`
///
/// and
///
/// `cavity_correction = q_j/C_j`.
///
/// Scientific status: EXACT algebra when all represented quantities are
/// finite. None of these factors is asserted to be less than one or uniformly
/// bounded over an operator family.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CavityDriftFactorization {
    local_reference_edge: f64,
    reference_defect: f64,
    edge_drift: f64,
    reference_drift: f64,
    normalized_edge_square: f64,
    cavity_correction: f64,
}

impl CavityDriftFactorization {
    pub fn new(
        step: CavityTransportStep,
        local_reference_edge: f64,
    ) -> Result<Self, CavityFactorizationError> {
        if !local_reference_edge.is_finite() {
            return Err(CavityFactorizationError::NonFiniteReferenceEdge {
                value: local_reference_edge,
            });
        }

        let reference_edge_squared = local_reference_edge * local_reference_edge;
        let edge_squared = step.edge() * step.edge();
        let q_neighbor = step.neighbor_reference();
        let q_current = step.current_reference();

        let reference_defect =
            step.shifted_diagonal() - reference_edge_squared / q_current - q_current;
        let edge_drift = (reference_edge_squared - edge_squared) / q_current;
        let reference_drift = edge_squared * (1.0 / q_current - 1.0 / q_neighbor);
        let normalized_edge = step.edge() / q_neighbor;
        let normalized_edge_square = normalized_edge * normalized_edge;
        let cavity_correction = q_neighbor / step.neighbor_cavity();

        if !reference_edge_squared.is_finite()
            || !edge_squared.is_finite()
            || !reference_defect.is_finite()
            || !edge_drift.is_finite()
            || !reference_drift.is_finite()
            || !normalized_edge_square.is_finite()
            || !cavity_correction.is_finite()
        {
            return Err(CavityFactorizationError::NonFiniteDerivedQuantity);
        }

        Ok(Self {
            local_reference_edge,
            reference_defect,
            edge_drift,
            reference_drift,
            normalized_edge_square,
            cavity_correction,
        })
    }

    #[inline]
    pub fn local_reference_edge(self) -> f64 {
        self.local_reference_edge
    }

    #[inline]
    pub fn reference_defect(self) -> f64 {
        self.reference_defect
    }

    #[inline]
    pub fn edge_drift(self) -> f64 {
        self.edge_drift
    }

    #[inline]
    pub fn reference_drift(self) -> f64 {
        self.reference_drift
    }

    #[inline]
    pub fn normalized_edge_square(self) -> f64 {
        self.normalized_edge_square
    }

    #[inline]
    pub fn cavity_correction(self) -> f64 {
        self.cavity_correction
    }

    #[inline]
    pub fn reconstructed_drift(self) -> f64 {
        self.reference_defect + self.edge_drift + self.reference_drift
    }

    #[inline]
    pub fn reconstructed_transport_factor(self) -> f64 {
        self.normalized_edge_square * self.cavity_correction
    }
}
