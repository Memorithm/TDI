//! TDI-10.x generic operator / resolvent research primitives, plus TDI-12.0
//! Stage-0 exact ordinal ranking helpers over those primitives.
//!
//! Scientific scope: real finite tridiagonal (Jacobi) matrices, exact
//! finite-dimensional shifted resolvent identities, explicitly isolated
//! constant positive frozen-Toeplitz reference models, exact finite cavity
//! error transport relative to caller-supplied positive reference cavities,
//! exact algebraic factorization of transport drift and multipliers, exact
//! finite-chain affine composition of cavity-error transport steps, and exact
//! finite ordinal ranking (average ranks / Spearman / Kendall τ-b) for the
//! TDI-12 Stage-0 bootstrap. This crate deliberately contains no
//! Riemann-specific coefficients, parity sectors, spectral-crossing
//! interpretation, or hypothesis-specific normalization. TDI-12 Stage-0 does
//! not authorize confirmatory execution.

pub mod cavity;
pub mod chain;
pub mod factorization;
pub mod frozen;
pub mod green;
pub mod jacobi;
pub mod ordinal;
pub mod resolvent;
pub mod transport;

pub use cavity::SchurCavities;
pub use chain::{CavityChainError, CavityTransportChain};
pub use factorization::{CavityDriftFactorization, CavityFactorizationError};
pub use frozen::{FrozenToeplitzCavity, FrozenToeplitzError};
pub use green::GreenBands;
pub use jacobi::{JacobiError, JacobiMatrix};
pub use ordinal::{
    CandidateResponseObservable, OrdinalError, average_ranks, coefficient_frobenius_norm_key,
    deterministic_shuffle, dimension_only_key, gershgorin_dominance_margin_key,
    identity_ordering_key, kendall_tau_b, spearman_rho, strictly_increasing_affine,
};
pub use resolvent::{ResolventError, ShiftedLdl};
pub use transport::{CavityTransportError, CavityTransportStep};
