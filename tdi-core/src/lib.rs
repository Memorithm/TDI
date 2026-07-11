#![forbid(unsafe_code)]

//! Noyau mathématique et algorithmique du benchmark TDI-1.

mod action;
mod explorer;
mod state;
mod system;

pub use action::Action;
pub use explorer::{ExploreError, ReachabilityReport, explore};
pub use state::{State, StateError};
pub use system::{TableSystem, TableSystemError, TransitionSystem};
