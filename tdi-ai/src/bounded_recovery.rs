//! Additive migration from the historical recovery API to bounded execution.
use crate::experiment::{
    ConfigurationError, DeterministicAdapter, DeterministicError, NoiseCoupling, RunLimits,
    RunReport, run_paired,
};
use crate::{FutureObservable, FutureOverlap, Intervention, ReferenceDynamics};

#[derive(Clone, Debug, PartialEq)]
pub enum PreparationError<I> {
    Intervention(I),
    Configuration(ConfigurationError),
}

/// Apply the intervention once, then run qualified immutable dynamics through
/// the bounded engine. Unlike the historical API, failures retain an accepted
/// prefix and depth/branch diagnostics. Cancellation is cooperative; external
/// blocking runtimes need the process supervisor's wall-clock deadline.
///
/// Dynamics, observables and states must have independently qualified Clone
/// semantics. Legacy callbacks with observable side effects are not equivalent.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn analyze_bounded<D, I, O, M, W>(
    dynamics: D,
    intervention: &I,
    observable: O,
    overlap: &M,
    initial_state: &D::State,
    limits: RunLimits,
    cancelled: impl FnMut(usize) -> bool,
    sink: impl FnMut(usize, &M::Score) -> Result<(), W>,
) -> Result<
    RunReport<M::Score, DeterministicError<D::Error, O::Error>, M::Error, W>,
    PreparationError<I::Error>,
>
where
    D: ReferenceDynamics + Clone,
    I: Intervention<D::State>,
    O: FutureObservable<D::State> + Clone,
    M: FutureOverlap<O::Output>,
{
    let perturbed = intervention
        .apply(initial_state)
        .map_err(PreparationError::Intervention)?;
    let factory = DeterministicAdapter::new(dynamics, observable, initial_state.clone());
    run_paired(
        &factory,
        initial_state,
        &perturbed,
        limits,
        NoiseCoupling::Deterministic,
        |a, b| overlap.overlap(a, b),
        cancelled,
        sink,
    )
    .map_err(PreparationError::Configuration)
}
