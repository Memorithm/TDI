#![forbid(unsafe_code)]

//! Noyau mathématique et algorithmique du benchmark TDI-1.

mod action;
mod baseline;
mod branching_baseline;
mod dynamics;
mod explorer;
mod recovery;
mod signature;
mod state;
mod system;

pub use action::Action;
pub use baseline::{
    BaselineError, uniform_future_block_distribution, uniform_future_block_entropy_bits,
};
pub use branching_baseline::{
    BranchingBaselineError, uniform_branching_path_distribution,
    uniform_branching_path_entropy_bits,
};
pub use dynamics::{OrbitAnalysis, OrbitError, analyze_orbit};
pub use explorer::{ExploreError, ReachabilityReport, explore};
pub use recovery::{RecoveryAnalysis, RecoveryError, analyze_recovery};
pub use signature::{ExactRatio, SignatureError, TdiSignature};
pub use state::{State, StateError};
pub use system::{TableSystem, TableSystemError, TransitionSystem};
