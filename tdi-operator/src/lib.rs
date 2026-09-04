//! TDI-10.x generic operator / resolvent research primitives.
//!
//! Scientific scope: real finite tridiagonal (Jacobi) matrices and exact
//! finite-dimensional shifted resolvent identities. This crate deliberately
//! contains no Riemann-specific coefficients, parity sectors, spectral-crossing
//! interpretation, or hypothesis-specific normalization.

pub mod cavity;
pub mod green;
pub mod jacobi;
pub mod resolvent;

pub use cavity::SchurCavities;
pub use green::GreenBands;
pub use jacobi::{JacobiError, JacobiMatrix};
pub use resolvent::{ResolventError, ShiftedLdl};
