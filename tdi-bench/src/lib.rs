#![forbid(unsafe_code)]

//! Reusable deterministic benchmark fixtures and TDI-7 research evaluators.
//!
//! The TDI-7 modules factor protocol-faithful non-holdout mechanics,
//! predictive evaluation, bounded information decomposition, and diagnostic
//! source-subspace audits so follow-up stages can share audited implementations
//! rather than duplicating scientific logic.

pub mod attention_v7;
#[path = "gaussian_mmi_v7_stable.rs"]
pub mod gaussian_mmi_v7;
pub mod predictive_v7;
pub mod subspace_v7;
