#![forbid(unsafe_code)]

//! Reusable deterministic benchmark fixtures and TDI-7 research evaluators.
//!
//! The TDI-7 modules factor protocol-faithful non-holdout mechanics,
//! predictive evaluation, and bounded information decomposition so follow-up
//! stages can share audited implementations rather than duplicating scientific
//! logic.

pub mod attention_v7;
#[path = "gaussian_mmi_v7_impl.rs"]
pub mod gaussian_mmi_v7;
pub mod predictive_v7;
