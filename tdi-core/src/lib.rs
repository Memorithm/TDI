#![forbid(unsafe_code)]

//! Noyau mathématique et algorithmique de TDI.
//!
//! Le crate expose les primitives exactes de dynamique prospective ainsi que
//! l'API canonique permettant de construire et comparer des signatures TDI.

mod action;
mod baseline;
mod branching_baseline;
mod branching_distribution;
mod branching_recovery;
mod dynamics;
mod explorer;
mod primitive;
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
pub use branching_distribution::{
    BranchingDistributionError, DistributionMathError, distribution_overlap,
    uniform_branching_state_distribution,
};
pub use branching_recovery::{
    BranchingRecoveryAnalysis, BranchingRecoveryError, analyze_branching_recovery,
};
pub use dynamics::{OrbitAnalysis, OrbitError, analyze_orbit};
pub use explorer::{ExploreError, ReachabilityReport, explore};
pub use primitive::{
    InterventionComparison, PrimitiveError, compare_intervention, prospective_signature,
};
pub use recovery::{RecoveryAnalysis, RecoveryError, analyze_recovery};
pub use signature::{ExactRatio, SignatureError, TdiSignature};
pub use state::{State, StateError};
pub use system::{TableSystem, TableSystemError, TransitionSystem};
