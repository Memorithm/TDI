#![forbid(unsafe_code)]

//! Framework-independent research contracts for applying TDI-style intervention
//! and recovery analysis to attention, memory and sequence-mixing systems.
//!
//! This crate intentionally sits above `tdi-core`. It does not alter the frozen
//! finite-state semantics or any historical TDI experiment. Instead it provides
//! generic contracts that can be instantiated by toy mechanisms, model probes,
//! FLAT-ATTENTION reference semantics, or future adapters.

mod static_diagnostics;
pub mod tdi7;
mod toy_attention;

use tdi_core::{BranchingRecoveryAnalysis, ExactRatio};

pub use static_diagnostics::{
    StaticAttentionDiagnostics, StaticAttentionError, analyze_static_attention,
};
pub use toy_attention::{
    BalancedTokenShift, FixedAttentionMixer, FullStateObservable, ReciprocalLInfRecovery,
    ToyAttentionError, ToyAttentionState, ToyRecoveryMetricError,
};

/// One recovery measurement at one strictly positive downstream depth.
#[derive(Clone, Debug, PartialEq)]
pub struct RecoveryPoint<S> {
    depth: usize,
    overlap: S,
}

impl<S> RecoveryPoint<S> {
    /// Create a recovery point.
    #[must_use]
    pub fn new(depth: usize, overlap: S) -> Self {
        Self { depth, overlap }
    }

    /// Downstream depth at which the measurement was taken.
    #[must_use]
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Overlap/recovery score produced by the selected metric.
    #[must_use]
    pub fn overlap(&self) -> &S {
        &self.overlap
    }
}

/// Ordered trajectory of recovery measurements after an intervention.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RecoveryProfile<S> {
    points: Vec<RecoveryPoint<S>>,
}

impl<S> RecoveryProfile<S> {
    /// Build a profile from already ordered points.
    #[must_use]
    pub fn new(points: Vec<RecoveryPoint<S>>) -> Self {
        Self { points }
    }

    /// Build the conventional TDI profile for depths `1..=n` from overlap values.
    #[must_use]
    pub fn from_overlaps(overlaps: impl IntoIterator<Item = S>) -> Self {
        let points = overlaps
            .into_iter()
            .enumerate()
            .map(|(index, overlap)| RecoveryPoint::new(index + 1, overlap))
            .collect();
        Self { points }
    }

    /// Number of downstream depths measured.
    #[must_use]
    pub fn horizon(&self) -> usize {
        self.points.len()
    }

    /// Whether no downstream depth was requested.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Ordered recovery points.
    #[must_use]
    pub fn points(&self) -> &[RecoveryPoint<S>] {
        &self.points
    }

    /// Recovery score at the final measured depth.
    #[must_use]
    pub fn final_overlap(&self) -> Option<&S> {
        self.points.last().map(RecoveryPoint::overlap)
    }
}

/// Deterministic or explicitly reproducible dynamics under study.
pub trait ReferenceDynamics {
    type State: Clone;
    type Error;
    fn advance(&self, state: &Self::State) -> Result<Self::State, Self::Error>;
}

/// Controlled perturbation applied once to the initial state.
pub trait Intervention<S> {
    type Error;
    fn apply(&self, reference: &S) -> Result<S, Self::Error>;
}

/// Observable extracted from a downstream state before comparison.
pub trait FutureObservable<S> {
    type Output;
    type Error;
    fn observe(&self, state: &S, depth: usize) -> Result<Self::Output, Self::Error>;
}

/// Comparison between two future observables.
pub trait FutureOverlap<O> {
    type Score;
    type Error;
    fn overlap(&self, reference: &O, perturbed: &O) -> Result<Self::Score, Self::Error>;
}

/// Failure stage for a generic intervention/recovery run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecoveryError<DynamicsError, InterventionError, ObservableError, OverlapError> {
    Intervention(InterventionError),
    ReferenceDynamics(DynamicsError),
    PerturbedDynamics(DynamicsError),
    ReferenceObservable(ObservableError),
    PerturbedObservable(ObservableError),
    Overlap(OverlapError),
}

/// Run the generic TDI-AI recovery protocol.
pub fn analyze_intervention_recovery<D, I, O, M>(
    dynamics: &D,
    intervention: &I,
    observable: &O,
    overlap: &M,
    initial_state: &D::State,
    horizon: usize,
) -> Result<RecoveryProfile<M::Score>, RecoveryError<D::Error, I::Error, O::Error, M::Error>>
where
    D: ReferenceDynamics,
    I: Intervention<D::State>,
    O: FutureObservable<D::State>,
    M: FutureOverlap<O::Output>,
{
    let mut reference_state = initial_state.clone();
    let mut perturbed_state = intervention.apply(initial_state).map_err(RecoveryError::Intervention)?;
    let mut points = Vec::with_capacity(horizon);

    for depth in 1..=horizon {
        reference_state = dynamics.advance(&reference_state).map_err(RecoveryError::ReferenceDynamics)?;
        perturbed_state = dynamics.advance(&perturbed_state).map_err(RecoveryError::PerturbedDynamics)?;
        let reference_observable = observable.observe(&reference_state, depth).map_err(RecoveryError::ReferenceObservable)?;
        let perturbed_observable = observable.observe(&perturbed_state, depth).map_err(RecoveryError::PerturbedObservable)?;
        let score = overlap.overlap(&reference_observable, &perturbed_observable).map_err(RecoveryError::Overlap)?;
        points.push(RecoveryPoint::new(depth, score));
    }

    Ok(RecoveryProfile::new(points))
}

/// Convert the exact finite-state branching oracle without numerical conversion.
#[must_use]
pub fn from_exact_branching_analysis(analysis: &BranchingRecoveryAnalysis) -> RecoveryProfile<ExactRatio> {
    RecoveryProfile::from_overlaps(analysis.overlap_profile().iter().cloned())
}

#[cfg(test)]
mod tests {
    use super::{FutureObservable, FutureOverlap, Intervention, RecoveryPoint, ReferenceDynamics, analyze_intervention_recovery, from_exact_branching_analysis};
    use tdi_core::{Action, ExactRatio, State, TableSystem, analyze_branching_recovery};

    #[derive(Clone, Copy)] struct Increment;
    impl ReferenceDynamics for Increment { type State=i32; type Error=core::convert::Infallible; fn advance(&self,state:&i32)->Result<i32,Self::Error>{Ok(*state+1)} }
    #[derive(Clone, Copy)] struct Shift(i32);
    impl Intervention<i32> for Shift { type Error=core::convert::Infallible; fn apply(&self,reference:&i32)->Result<i32,Self::Error>{Ok(*reference+self.0)} }
    #[derive(Clone, Copy)] struct IdentityObservable;
    impl FutureObservable<i32> for IdentityObservable { type Output=i32; type Error=core::convert::Infallible; fn observe(&self,state:&i32,_:usize)->Result<i32,Self::Error>{Ok(*state)} }
    #[derive(Clone, Copy)] struct ReciprocalDistance;
    impl FutureOverlap<i32> for ReciprocalDistance { type Score=f64; type Error=core::convert::Infallible; fn overlap(&self,reference:&i32,perturbed:&i32)->Result<f64,Self::Error>{Ok(1.0/(1.0+f64::from((*reference-*perturbed).abs())))} }

    #[test] fn generic_protocol_applies_intervention_once_and_preserves_depth_order(){let p=analyze_intervention_recovery(&Increment,&Shift(2),&IdentityObservable,&ReciprocalDistance,&0,3).unwrap();assert_eq!(p.horizon(),3);assert_eq!(p.points()[0],RecoveryPoint::new(1,1.0/3.0));}
    #[test] fn zero_horizon_does_not_require_future_observations(){let p=analyze_intervention_recovery(&Increment,&Shift(2),&IdentityObservable,&ReciprocalDistance,&0,0).unwrap();assert!(p.is_empty());}
    #[test] fn exact_branching_oracle_maps_without_numerical_conversion(){let zero=State::new(0,2).unwrap();let one=State::new(1,2).unwrap();let two=State::new(2,2).unwrap();let three=State::new(3,2).unwrap();let mut s=TableSystem::new(2).unwrap();s.insert(zero,Action::Noop,vec![two,three]).unwrap();s.insert(one,Action::Noop,vec![three]).unwrap();let e=analyze_branching_recovery(&s,zero,Action::Flip{node:0},Action::Noop,1).unwrap();let g=from_exact_branching_analysis(&e);assert_eq!(g.final_overlap(),Some(&ExactRatio::new(1,2).unwrap()));}
}
