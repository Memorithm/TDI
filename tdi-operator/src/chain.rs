use core::fmt;

use crate::transport::CavityTransportStep;

/// Fail-closed errors for exact finite-chain composition of TDI-10.2 steps.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CavityChainError {
    EmptyChain,
    DiscontinuousCavity {
        index: usize,
        expected: f64,
        observed: f64,
    },
    DiscontinuousReference {
        index: usize,
        expected: f64,
        observed: f64,
    },
    DiscontinuousError {
        index: usize,
        expected: f64,
        observed: f64,
    },
    NonFiniteAccumulation {
        index: usize,
    },
}

impl fmt::Display for CavityChainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::EmptyChain => write!(f, "cavity transport chain is empty"),
            Self::DiscontinuousCavity {
                index,
                expected,
                observed,
            } => write!(
                f,
                "step {index} neighbor cavity is discontinuous: expected {expected:e}, observed {observed:e}"
            ),
            Self::DiscontinuousReference {
                index,
                expected,
                observed,
            } => write!(
                f,
                "step {index} neighbor reference is discontinuous: expected {expected:e}, observed {observed:e}"
            ),
            Self::DiscontinuousError {
                index,
                expected,
                observed,
            } => write!(
                f,
                "step {index} neighbor error is discontinuous: expected {expected:e}, observed {observed:e}"
            ),
            Self::NonFiniteAccumulation { index } => write!(
                f,
                "finite-chain composition became non-finite after step {index}"
            ),
        }
    }
}

impl std::error::Error for CavityChainError {}

/// Exact affine composition of a contiguous finite sequence of TDI-10.2
/// cavity-error transport steps.
///
/// For a chain
///
/// `E_k = alpha_k E_{k-1} + delta_k`, `k=1,...,n`,
///
/// repeated substitution gives exactly
///
/// `E_n = A_n E_0 + B_n`,
///
/// where
///
/// `A_n = product_{k=1}^n alpha_k`
///
/// and
///
/// `B_n = sum_{k=1}^n delta_k product_{r=k+1}^n alpha_r`.
///
/// The implementation evaluates the equivalent stable recurrence
///
/// `A_k = alpha_k A_{k-1}`, `B_k = alpha_k B_{k-1} + delta_k`,
///
/// with `A_0=1`, `B_0=0`.
///
/// Scientific status: the displayed relations are EXACT finite algebra over
/// real numbers. Floating-point accumulation is only an implementation of that
/// identity and may round or overflow. No factor, product, boundary weight, or
/// drift contribution is asserted to be less than one, decaying, or uniformly
/// bounded over an operator family.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CavityTransportChain {
    steps: usize,
    initial_error: f64,
    observed_final_error: f64,
    cumulative_transport_factor: f64,
    accumulated_drift: f64,
}

impl CavityTransportChain {
    /// Compose a non-empty sequence in propagation order.
    ///
    /// Adjacent steps must carry exactly the preceding step's current cavity,
    /// current reference, and current error as the following step's neighbor
    /// metadata. This bitwise continuity check is a provenance guard for the
    /// floating-point representation; it is not a new analytic hypothesis.
    pub fn from_steps(steps: &[CavityTransportStep]) -> Result<Self, CavityChainError> {
        let first = *steps.first().ok_or(CavityChainError::EmptyChain)?;
        let initial_error = first.neighbor_error();
        let mut cumulative_transport_factor = 1.0;
        let mut accumulated_drift = 0.0;
        let mut previous = None;

        for (index, step) in steps.iter().copied().enumerate() {
            if let Some(previous_step) = previous {
                if step.neighbor_cavity() != previous_step.current_cavity() {
                    return Err(CavityChainError::DiscontinuousCavity {
                        index,
                        expected: previous_step.current_cavity(),
                        observed: step.neighbor_cavity(),
                    });
                }
                if step.neighbor_reference() != previous_step.current_reference() {
                    return Err(CavityChainError::DiscontinuousReference {
                        index,
                        expected: previous_step.current_reference(),
                        observed: step.neighbor_reference(),
                    });
                }
                if step.neighbor_error() != previous_step.current_error() {
                    return Err(CavityChainError::DiscontinuousError {
                        index,
                        expected: previous_step.current_error(),
                        observed: step.neighbor_error(),
                    });
                }
            }

            cumulative_transport_factor *= step.transport_factor();
            accumulated_drift = step.transport_factor() * accumulated_drift + step.drift();
            let reconstructed = cumulative_transport_factor * initial_error + accumulated_drift;

            if !cumulative_transport_factor.is_finite()
                || !accumulated_drift.is_finite()
                || !reconstructed.is_finite()
            {
                return Err(CavityChainError::NonFiniteAccumulation { index });
            }

            previous = Some(step);
        }

        let observed_final_error = previous
            .expect("a non-empty slice has a final transport step")
            .current_error();

        Ok(Self {
            steps: steps.len(),
            initial_error,
            observed_final_error,
            cumulative_transport_factor,
            accumulated_drift,
        })
    }

    #[inline]
    pub fn steps(self) -> usize {
        self.steps
    }

    #[inline]
    pub fn initial_error(self) -> f64 {
        self.initial_error
    }

    #[inline]
    pub fn observed_final_error(self) -> f64 {
        self.observed_final_error
    }

    #[inline]
    pub fn cumulative_transport_factor(self) -> f64 {
        self.cumulative_transport_factor
    }

    #[inline]
    pub fn accumulated_drift(self) -> f64 {
        self.accumulated_drift
    }

    #[inline]
    pub fn reconstructed_final_error(self) -> f64 {
        self.cumulative_transport_factor * self.initial_error + self.accumulated_drift
    }
}
