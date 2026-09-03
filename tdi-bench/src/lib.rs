#![forbid(unsafe_code)]

//! Reusable deterministic benchmark fixtures and TDI-7 research evaluators.
//!
//! The TDI-7 modules factor protocol-faithful non-holdout mechanics and
//! predictive evaluation so follow-up stages can share one audited
//! implementation rather than duplicating scientific logic.

pub mod attention_v7;
pub mod predictive_v7;
