#![forbid(unsafe_code)]

//! Noyau mathématique et algorithmique du benchmark TDI-1.

mod action;
mod explorer;
mod signature;
mod state;
mod system;

pub use action::Action;
pub use explorer::{ExploreError, ReachabilityReport, explore};
pub use signature::{ExactRatio, SignatureError, TdiSignature};
pub use state::{State, StateError};
pub use system::{TableSystem, TableSystemError, TransitionSystem};
