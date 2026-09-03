#![forbid(unsafe_code)]

//! Reusable deterministic benchmark fixtures and TDI research evaluators.
//!
//! TDI-7 modules factor protocol-faithful non-holdout mechanics, predictive
//! evaluation, bounded information decomposition, and diagnostic source-subspace
//! audits. TDI-8 modules transcribe frozen bounded-evaluator rules that belong in
//! benchmark/evidence processing rather than architecture primitives.

pub mod attention_v7;
pub mod decision_v8;
#[path = "gaussian_mmi_v7_stable.rs"]
pub mod gaussian_mmi_v7;
pub mod paired_resampling_v8;
pub mod predictive_v7;
pub mod subspace_v7;
pub mod task_adapter_v8;
