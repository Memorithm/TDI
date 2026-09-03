#![forbid(unsafe_code)]

//! Framework-independent research contracts for applying TDI-style intervention
//! and recovery analysis to attention, memory and sequence-mixing systems.
//!
//! This crate intentionally sits above `tdi-core`. It does not alter the frozen
//! finite-state semantics or any historical TDI experiment. Instead it provides
//! generic contracts that can be instantiated by toy mechanisms, model probes,
//! FLAT-ATTENTION reference semantics, or future adapters.

pub mod associative_memory;
mod assr;
pub mod assr_h_reference;
pub mod assr_reference;
pub mod full_history_reference;
mod static_diagnostics;
mod toy_attention;
pub mod vsa_workspace;

use tdi_core::{BranchingRecoveryAnalysis, ExactRatio};

pub use assr::{
    MatchedDynamicBudget, MemoryAccounting, MemoryAccountingError, MemoryComponent, ReferenceArm,
    ReferenceSnapshot, StorageBits,
};
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

/// Early recovery features restricted to observations before a target depth.
#[derive(Clone, Debug, PartialEq)]
pub struct EarlyRecoveryFeatures {
    target_depth: usize,
    depths: Vec<usize>,
    overlaps: Vec<f64>,
}

impl EarlyRecoveryFeatures {
    /// Target depth whose future observation is deliberately excluded.
    #[must_use]
    pub fn target_depth(&self) -> usize {
        self.target_depth
    }

    /// Early observation depths in ascending order.
    #[must_use]
    pub fn depths(&self) -> &[usize] {
        &self.depths
    }

    /// Recovery values aligned with the depth list.
    #[must_use]
    pub fn overlaps(&self) -> &[f64] {
        &self.overlaps
    }

    /// Stable record suitable for a provenance envelope.
    #[must_use]
    pub fn canonical_record(&self) -> String {
        let depth_record = self
            .depths
            .iter()
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let overlap_record = self
            .overlaps
            .iter()
            .map(|value| format!("{:016x}", value.to_bits()))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "tdi-ai-early-recovery-v1;target_depth={};depths={depth_record};overlap_bits={overlap_record}",
            self.target_depth
        )
    }
}

/// Errors raised while extracting leakage-safe early recovery features.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EarlyRecoveryFeatureError {
    /// The target depth must be strictly positive.
    ZeroTargetDepth,
    /// No profile point precedes the target depth.
    EmptyEarlyWindow {
        /// Target depth that admitted no early observations.
        target_depth: usize,
    },
    /// A profile point has an invalid depth order.
    NonIncreasingDepth {
        /// Previous accepted depth.
        previous: usize,
        /// Current depth that violated ordering.
        current: usize,
    },
    /// An early recovery value is not finite.
    NonFiniteOverlap {
        /// Depth associated with the invalid value.
        depth: usize,
    },
}

impl core::fmt::Display for EarlyRecoveryFeatureError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ZeroTargetDepth => formatter.write_str("target depth must be positive"),
            Self::EmptyEarlyWindow { target_depth } => {
                write!(
                    formatter,
                    "no recovery point precedes target depth {target_depth}"
                )
            }
            Self::NonIncreasingDepth { previous, current } => write!(
                formatter,
                "recovery depths must increase strictly: previous={previous}, current={current}"
            ),
            Self::NonFiniteOverlap { depth } => {
                write!(formatter, "recovery overlap at depth {depth} is not finite")
            }
        }
    }
}

impl std::error::Error for EarlyRecoveryFeatureError {}

/// Extract only observations strictly before target_depth.
///
/// This function is intentionally one-way: points at the target or beyond are
/// not inspected for feature values and therefore cannot leak target-depth
/// information into an early feature vector. The profile prefix must still be
/// strictly ordered, finite, and non-empty.
pub fn extract_early_recovery_features(
    profile: &RecoveryProfile<f64>,
    target_depth: usize,
) -> Result<EarlyRecoveryFeatures, EarlyRecoveryFeatureError> {
    if target_depth == 0 {
        return Err(EarlyRecoveryFeatureError::ZeroTargetDepth);
    }

    let mut previous = 0;
    let mut depths = Vec::new();
    let mut overlaps = Vec::new();
    for point in profile.points() {
        if point.depth() >= target_depth {
            break;
        }
        if point.depth() == 0 || point.depth() <= previous {
            return Err(EarlyRecoveryFeatureError::NonIncreasingDepth {
                previous,
                current: point.depth(),
            });
        }
        if !point.overlap().is_finite() {
            return Err(EarlyRecoveryFeatureError::NonFiniteOverlap {
                depth: point.depth(),
            });
        }
        previous = point.depth();
        depths.push(point.depth());
        overlaps.push(*point.overlap());
    }

    if depths.is_empty() {
        return Err(EarlyRecoveryFeatureError::EmptyEarlyWindow { target_depth });
    }

    Ok(EarlyRecoveryFeatures {
        target_depth,
        depths,
        overlaps,
    })
}

/// Deterministic or explicitly reproducible dynamics under study.
///
/// A model adapter may interpret one `advance` call as a layer, recurrent step,
/// token-generation step, mixer application, or another preregistered unit.
pub trait ReferenceDynamics {
    /// Complete state required to advance the mechanism.
    type State: Clone;

    /// Failure while advancing the reference or perturbed computation.
    type Error;

    /// Advance one declared unit of dynamics.
    fn advance(&self, state: &Self::State) -> Result<Self::State, Self::Error>;
}

/// Controlled perturbation applied once to the initial state.
pub trait Intervention<S> {
    /// Failure while constructing the perturbed state.
    type Error;

    /// Apply the intervention without mutating the reference state.
    fn apply(&self, reference: &S) -> Result<S, Self::Error>;
}

/// Observable extracted from a downstream state before comparison.
pub trait FutureObservable<S> {
    /// Representation compared between reference and perturbed futures.
    type Output;

    /// Failure while extracting the observable.
    type Error;

    /// Observe a state at the declared downstream depth.
    fn observe(&self, state: &S, depth: usize) -> Result<Self::Output, Self::Error>;
}

/// Comparison between two future observables.
///
/// `Score` is intentionally generic. Exact finite-state TDI uses `ExactRatio`;
/// AI adapters may use a bounded similarity, distributional overlap, calibrated
/// divergence transform, task-specific recovery score, or another frozen metric.
pub trait FutureOverlap<O> {
    /// Output score type.
    type Score;

    /// Failure while comparing the two observables.
    type Error;

    /// Compare the reference future with the perturbed future.
    fn overlap(&self, reference: &O, perturbed: &O) -> Result<Self::Score, Self::Error>;
}

/// Failure stage for a generic intervention/recovery run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecoveryError<DynamicsError, InterventionError, ObservableError, OverlapError> {
    /// Initial intervention failed.
    Intervention(InterventionError),
    /// Reference trajectory failed to advance.
    ReferenceDynamics(DynamicsError),
    /// Perturbed trajectory failed to advance.
    PerturbedDynamics(DynamicsError),
    /// Reference observable could not be extracted.
    ReferenceObservable(ObservableError),
    /// Perturbed observable could not be extracted.
    PerturbedObservable(ObservableError),
    /// Recovery/overlap comparison failed.
    Overlap(OverlapError),
}

/// Result type for a generic intervention/recovery analysis.
pub type RecoveryResult<Score, DynamicsError, InterventionError, ObservableError, OverlapError> =
    Result<
        RecoveryProfile<Score>,
        RecoveryError<DynamicsError, InterventionError, ObservableError, OverlapError>,
    >;

/// Run the generic TDI-AI recovery protocol.
///
/// The intervention is applied exactly once at depth zero. The reference and
/// perturbed states then follow the same declared dynamics. At every downstream
/// depth, the same observable and overlap rule are applied to both trajectories.
///
/// No claim is made that a score is probabilistic or bounded unless the selected
/// `FutureOverlap` implementation defines and validates those properties.
#[allow(
    clippy::type_complexity,
    reason = "the public protocol intentionally preserves stage-specific error types"
)]
pub fn analyze_intervention_recovery<D, I, O, M>(
    dynamics: &D,
    intervention: &I,
    observable: &O,
    overlap: &M,
    initial_state: &D::State,
    horizon: usize,
) -> RecoveryResult<M::Score, D::Error, I::Error, O::Error, M::Error>
where
    D: ReferenceDynamics,
    I: Intervention<D::State>,
    O: FutureObservable<D::State>,
    M: FutureOverlap<O::Output>,
{
    let mut reference_state = initial_state.clone();
    let mut perturbed_state = intervention
        .apply(initial_state)
        .map_err(RecoveryError::Intervention)?;
    let mut points = Vec::with_capacity(horizon);

    for depth in 1..=horizon {
        reference_state = dynamics
            .advance(&reference_state)
            .map_err(RecoveryError::ReferenceDynamics)?;
        perturbed_state = dynamics
            .advance(&perturbed_state)
            .map_err(RecoveryError::PerturbedDynamics)?;

        let reference_observable = observable
            .observe(&reference_state, depth)
            .map_err(RecoveryError::ReferenceObservable)?;
        let perturbed_observable = observable
            .observe(&perturbed_state, depth)
            .map_err(RecoveryError::PerturbedObservable)?;

        let score = overlap
            .overlap(&reference_observable, &perturbed_observable)
            .map_err(RecoveryError::Overlap)?;
        points.push(RecoveryPoint::new(depth, score));
    }

    Ok(RecoveryProfile::new(points))
}

/// Convert the existing exact finite-state branching oracle into the generic
/// TDI-AI recovery schema without changing any historical TDI semantics.
#[must_use]
pub fn from_exact_branching_analysis(
    analysis: &BranchingRecoveryAnalysis,
) -> RecoveryProfile<ExactRatio> {
    RecoveryProfile::from_overlaps(analysis.overlap_profile().iter().cloned())
}

#[cfg(test)]
mod tests {
    use super::{
        EarlyRecoveryFeatureError, FutureObservable, FutureOverlap, Intervention, RecoveryPoint,
        RecoveryProfile, ReferenceDynamics, analyze_intervention_recovery,
        extract_early_recovery_features, from_exact_branching_analysis,
    };
    use tdi_core::{Action, ExactRatio, State, TableSystem, analyze_branching_recovery};

    #[derive(Clone, Copy)]
    struct Increment;

    impl ReferenceDynamics for Increment {
        type State = i32;
        type Error = core::convert::Infallible;

        fn advance(&self, state: &Self::State) -> Result<Self::State, Self::Error> {
            Ok(*state + 1)
        }
    }

    #[derive(Clone, Copy)]
    struct Shift(i32);

    impl Intervention<i32> for Shift {
        type Error = core::convert::Infallible;

        fn apply(&self, reference: &i32) -> Result<i32, Self::Error> {
            Ok(*reference + self.0)
        }
    }

    #[derive(Clone, Copy)]
    struct IdentityObservable;

    impl FutureObservable<i32> for IdentityObservable {
        type Output = i32;
        type Error = core::convert::Infallible;

        fn observe(&self, state: &i32, _depth: usize) -> Result<Self::Output, Self::Error> {
            Ok(*state)
        }
    }

    #[derive(Clone, Copy)]
    struct ReciprocalDistance;

    impl FutureOverlap<i32> for ReciprocalDistance {
        type Score = f64;
        type Error = core::convert::Infallible;

        fn overlap(&self, reference: &i32, perturbed: &i32) -> Result<Self::Score, Self::Error> {
            let distance = f64::from((*reference - *perturbed).abs());
            Ok(1.0 / (1.0 + distance))
        }
    }

    #[test]
    fn generic_protocol_applies_intervention_once_and_preserves_depth_order() {
        let profile = analyze_intervention_recovery(
            &Increment,
            &Shift(2),
            &IdentityObservable,
            &ReciprocalDistance,
            &0,
            3,
        )
        .expect("infallible fixture");

        assert_eq!(profile.horizon(), 3);
        assert_eq!(profile.points()[0], RecoveryPoint::new(1, 1.0 / 3.0));
        assert_eq!(profile.points()[1], RecoveryPoint::new(2, 1.0 / 3.0));
        assert_eq!(profile.points()[2], RecoveryPoint::new(3, 1.0 / 3.0));
    }

    #[test]
    fn early_features_stop_strictly_before_target_depth() {
        let profile = RecoveryProfile::from_overlaps([0.25, 0.5, 0.75]);
        let features = extract_early_recovery_features(&profile, 3).expect("early prefix");

        assert_eq!(features.target_depth(), 3);
        assert_eq!(features.depths(), &[1, 2]);
        assert_eq!(features.overlaps(), &[0.25, 0.5]);
        assert!(features.canonical_record().contains("target_depth=3"));
    }

    #[test]
    fn early_features_fail_closed_without_an_early_observation() {
        let profile = RecoveryProfile::from_overlaps([0.5]);
        assert_eq!(
            extract_early_recovery_features(&profile, 1),
            Err(EarlyRecoveryFeatureError::EmptyEarlyWindow { target_depth: 1 })
        );
        assert_eq!(
            extract_early_recovery_features(&profile, 0),
            Err(EarlyRecoveryFeatureError::ZeroTargetDepth)
        );
    }

    #[test]
    fn zero_horizon_does_not_require_future_observations() {
        let profile = analyze_intervention_recovery(
            &Increment,
            &Shift(2),
            &IdentityObservable,
            &ReciprocalDistance,
            &0,
            0,
        )
        .expect("infallible fixture");

        assert!(profile.is_empty());
        assert_eq!(profile.final_overlap(), None);
    }

    #[test]
    fn exact_branching_oracle_maps_without_numerical_conversion() {
        let zero = State::new(0b00, 2).expect("valid state");
        let one = State::new(0b01, 2).expect("valid state");
        let two = State::new(0b10, 2).expect("valid state");
        let three = State::new(0b11, 2).expect("valid state");

        let mut system = TableSystem::new(2).expect("valid system");
        system
            .insert(zero, Action::Noop, vec![two, three])
            .expect("valid transition");
        system
            .insert(one, Action::Noop, vec![three])
            .expect("valid transition");

        let exact =
            analyze_branching_recovery(&system, zero, Action::Flip { node: 0 }, Action::Noop, 1)
                .expect("exact analysis succeeds");
        let generic = from_exact_branching_analysis(&exact);

        assert_eq!(generic.horizon(), 1);
        assert_eq!(
            generic.final_overlap(),
            Some(&ExactRatio::new(1, 2).expect("valid ratio"))
        );
    }
}