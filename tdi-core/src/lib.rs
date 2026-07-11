#![forbid(unsafe_code)]

//! Noyau mathématique et algorithmique du benchmark TDI-1.

mod action;
mod state;
mod system;

pub use action::Action;
pub use state::{State, StateError};
pub use system::TransitionSystem;
