//! TDI-10.x generic operator / resolvent research primitives.
//!
//! Scientific scope: real finite tridiagonal (Jacobi) matrices, exact
//! finite-dimensional shifted resolvent identities, explicitly isolated
//! constant positive frozen-Toeplitz reference models, and exact finite cavity
//! error transport relative to caller-supplied positive reference cavities.
//! This crate deliberately contains no Riemann-specific coefficients, parity
//! sectors, spectral-crossing interpretation, or hypothesis-specific
//! normalization.

pub mod cavity;
pub mod frozen;
pub mod green;
pub mod jacobi;
pub mod resolvent;
pub mod transport;

pub use cavity::SchurCavities;
pub use frozen::{FrozenToeplitzCavity, FrozenToeplitzError};
pub use green::GreenBands;
pub use jacobi::{JacobiError, JacobiMatrix};
pub use resolvent::{ResolventError, ShiftedLdl};
pub use transport::{CavityTransportError, CavityTransportStep};
