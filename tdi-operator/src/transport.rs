use core::fmt;

/// Fail-closed validation errors for one exact cavity-error transport step.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CavityTransportError {
    NonFiniteInput { name: &'static str, value: f64 },
    NonPositiveNeighborCavity { value: f64 },
    NonPositiveReference { name: &'static str, value: f64 },
    NonPositiveCurrentCavity { value: f64 },
    NonFiniteDerivedQuantity,
}

impl fmt::Display for CavityTransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::NonFiniteInput { name, value } => {
                write!(f, "{name} is not finite: {value:e}")
            }
            Self::NonPositiveNeighborCavity { value } => {
                write!(f, "neighbor cavity is not strictly positive: {value:e}")
            }
            Self::NonPositiveReference { name, value } => {
                write!(f, "{name} reference cavity is not strictly positive: {value:e}")
            }
            Self::NonPositiveCurrentCavity { value } => {
                write!(f, "derived current cavity is not strictly positive: {value:e}")
            }
            Self::NonFiniteDerivedQuantity => {
                write!(f, "cavity transport produced a non-finite derived quantity")
            }
        }
    }
}

impl std::error::Error for CavityTransportError {}

/// Exact decomposition of one finite Schur-cavity error relative to arbitrary
/// positive reference cavities.
///
/// Let the finite recurrence be
///
/// `C_i = a_i - e^2 / C_j`,
///
/// where `j=i-1` for a left cavity step or `j=i+1` for a right cavity step.
/// For any positive reference values `q_j` and `q_i`, define
///
/// `E_j = C_j - q_j`, `E_i = C_i - q_i`.
///
/// Then exactly
///
/// `E_i = alpha E_j + delta`,
///
/// with
///
/// `alpha = e^2 / (C_j q_j)`
///
/// and
///
/// `delta = a_i - e^2 / q_j - q_i`.
///
/// Scientific status: EXACT finite algebra under the declared positivity and
/// finite-value conditions. The reference sequence is supplied by the caller;
/// this type does not claim that any particular local frozen model approximates
/// a variable-coefficient operator.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CavityTransportStep {
    neighbor_cavity: f64,
    current_cavity: f64,
    neighbor_reference: f64,
    current_reference: f64,
    neighbor_error: f64,
    current_error: f64,
    transport_factor: f64,
    drift: f64,
}

impl CavityTransportStep {
    /// Build one left-cavity transport step from `L_{i-1}` to `L_i`.
    pub fn left(
        shifted_diagonal: f64,
        incoming_edge: f64,
        previous_cavity: f64,
        previous_reference: f64,
        current_reference: f64,
    ) -> Result<Self, CavityTransportError> {
        Self::build(
            shifted_diagonal,
            incoming_edge,
            previous_cavity,
            previous_reference,
            current_reference,
        )
    }

    /// Build one right-cavity transport step from `R_{i+1}` to `R_i`.
    pub fn right(
        shifted_diagonal: f64,
        outgoing_edge: f64,
        next_cavity: f64,
        next_reference: f64,
        current_reference: f64,
    ) -> Result<Self, CavityTransportError> {
        Self::build(
            shifted_diagonal,
            outgoing_edge,
            next_cavity,
            next_reference,
            current_reference,
        )
    }

    fn build(
        shifted_diagonal: f64,
        edge: f64,
        neighbor_cavity: f64,
        neighbor_reference: f64,
        current_reference: f64,
    ) -> Result<Self, CavityTransportError> {
        for (name, value) in [
            ("shifted diagonal", shifted_diagonal),
            ("edge", edge),
            ("neighbor cavity", neighbor_cavity),
            ("neighbor reference", neighbor_reference),
            ("current reference", current_reference),
        ] {
            if !value.is_finite() {
                return Err(CavityTransportError::NonFiniteInput { name, value });
            }
        }
        if neighbor_cavity <= 0.0 {
            return Err(CavityTransportError::NonPositiveNeighborCavity {
                value: neighbor_cavity,
            });
        }
        if neighbor_reference <= 0.0 {
            return Err(CavityTransportError::NonPositiveReference {
                name: "neighbor",
                value: neighbor_reference,
            });
        }
        if current_reference <= 0.0 {
            return Err(CavityTransportError::NonPositiveReference {
                name: "current",
                value: current_reference,
            });
        }

        let edge_squared = edge * edge;
        let current_cavity = shifted_diagonal - edge_squared / neighbor_cavity;
        let neighbor_error = neighbor_cavity - neighbor_reference;
        let current_error = current_cavity - current_reference;
        let transport_factor = edge_squared / (neighbor_cavity * neighbor_reference);
        let drift = shifted_diagonal - edge_squared / neighbor_reference - current_reference;

        if !edge_squared.is_finite()
            || !current_cavity.is_finite()
            || !neighbor_error.is_finite()
            || !current_error.is_finite()
            || !transport_factor.is_finite()
            || !drift.is_finite()
        {
            return Err(CavityTransportError::NonFiniteDerivedQuantity);
        }
        if current_cavity <= 0.0 {
            return Err(CavityTransportError::NonPositiveCurrentCavity {
                value: current_cavity,
            });
        }

        Ok(Self {
            neighbor_cavity,
            current_cavity,
            neighbor_reference,
            current_reference,
            neighbor_error,
            current_error,
            transport_factor,
            drift,
        })
    }

    #[inline]
    pub fn neighbor_cavity(self) -> f64 {
        self.neighbor_cavity
    }

    #[inline]
    pub fn current_cavity(self) -> f64 {
        self.current_cavity
    }

    #[inline]
    pub fn neighbor_reference(self) -> f64 {
        self.neighbor_reference
    }

    #[inline]
    pub fn current_reference(self) -> f64 {
        self.current_reference
    }

    #[inline]
    pub fn neighbor_error(self) -> f64 {
        self.neighbor_error
    }

    #[inline]
    pub fn current_error(self) -> f64 {
        self.current_error
    }

    #[inline]
    pub fn transport_factor(self) -> f64 {
        self.transport_factor
    }

    #[inline]
    pub fn drift(self) -> f64 {
        self.drift
    }

    #[inline]
    pub fn reconstructed_error(self) -> f64 {
        self.transport_factor * self.neighbor_error + self.drift
    }
}
