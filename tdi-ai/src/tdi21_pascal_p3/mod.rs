//! Exact non-final Pascal P3 harness frozen by the P3 preregistration.

mod core;
mod matrix;
mod types;

pub use core::run_cell;
pub use matrix::{
    run_development_matrix, run_pascal_p3_development_matrix, run_pascal_p3_validation_matrix,
    run_validation_matrix,
};
pub use types::{Cell, Error, SEMANTICS, Schedule, Split, canonical_header};
