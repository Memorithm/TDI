//! Bounded paired development execution. No scientific defaults or final runner.
use crate::{RecoveryPoint, RecoveryProfile};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Branch {
    Reference,
    Perturbed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoiseCoupling {
    Deterministic,
    Paired,
    Independent,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StepContext {
    pub depth: usize,
    /// Stream identity, not a random sample. Adapters declare their RNG algorithm.
    pub noise_stream: u64,
}
/// Implementations must fork all mutable caches and RNG state independently.
/// This semantic contract requires adapter qualification; Clone alone is insufficient.
pub trait ReplayAdapter: Sized {
    type Checkpoint;
    type Observation;
    type Error;
    fn fork(&self, checkpoint: &Self::Checkpoint) -> Result<Self, Self::Error>;
    fn checkpoint(&self) -> Result<Self::Checkpoint, Self::Error>;
    fn advance(&mut self, context: StepContext) -> Result<Self::Observation, Self::Error>;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Collection {
    All,
    Every(usize),
    None,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RunLimits {
    horizon: usize,
    max_steps: usize,
    max_points: usize,
    collection: Collection,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfigurationError {
    StepLimit,
    ZeroStride,
    PointLimit,
    Capacity,
}
impl RunLimits {
    pub fn new(
        horizon: usize,
        max_steps: usize,
        max_points: usize,
        collection: Collection,
    ) -> Result<Self, ConfigurationError> {
        if horizon > max_steps {
            return Err(ConfigurationError::StepLimit);
        }
        if collection == Collection::Every(0) {
            return Err(ConfigurationError::ZeroStride);
        }
        let limits = Self {
            horizon,
            max_steps,
            max_points,
            collection,
        };
        if limits.point_count() > max_points {
            return Err(ConfigurationError::PointLimit);
        }
        Ok(limits)
    }
    pub fn horizon(self) -> usize {
        self.horizon
    }
    pub fn max_steps(self) -> usize {
        self.max_steps
    }
    pub fn max_points(self) -> usize {
        self.max_points
    }
    pub fn collection(self) -> Collection {
        self.collection
    }
    fn point_count(self) -> usize {
        match self.collection {
            Collection::All => self.horizon,
            Collection::Every(n) => self.horizon / n,
            Collection::None => 0,
        }
    }
    fn retains(self, depth: usize) -> bool {
        match self.collection {
            Collection::All => true,
            Collection::Every(n) => depth % n == 0,
            Collection::None => false,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Fork,
    Advance,
    Metric,
    Sink,
    Cancelled,
}
#[derive(Clone, Debug, PartialEq)]
pub enum Failure<E, M, W> {
    Adapter(E),
    Metric(M),
    Sink(W),
    Cancelled,
}
#[derive(Clone, Debug, PartialEq)]
pub struct RunFailure<E, M, W> {
    pub depth: usize,
    pub branch: Option<Branch>,
    pub stage: Stage,
    pub error: Failure<E, M, W>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct RunReport<S, E, M, W> {
    pub completed_depth: usize,
    pub profile: RecoveryProfile<S>,
    /// None means the requested horizon completed, never a scientific success verdict.
    pub failure: Option<RunFailure<E, M, W>>,
}
/// Cancellation is checked between steps; a blocking backend requires its own
/// deadline. Sink failures retain only fully accepted points. Disposable sessions
/// are dropped after failure, with no promise of a resumable partial checkpoint.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn run_paired<A: ReplayAdapter, S, M, W>(
    factory: &A,
    reference: &A::Checkpoint,
    perturbed: &A::Checkpoint,
    limits: RunLimits,
    coupling: NoiseCoupling,
    mut metric: impl FnMut(&A::Observation, &A::Observation) -> Result<S, M>,
    mut cancelled: impl FnMut(usize) -> bool,
    mut sink: impl FnMut(usize, &S) -> Result<(), W>,
) -> Result<RunReport<S, A::Error, M, W>, ConfigurationError> {
    let mut points = Vec::new();
    points
        .try_reserve_exact(limits.point_count())
        .map_err(|_| ConfigurationError::Capacity)?;
    let mut report = RunReport {
        completed_depth: 0,
        profile: RecoveryProfile::new(Vec::new()),
        failure: None,
    };
    macro_rules! fail {
        ($d:expr,$b:expr,$s:expr,$e:expr) => {{
            report.profile = RecoveryProfile::new(points);
            report.failure = Some(RunFailure {
                depth: $d,
                branch: $b,
                stage: $s,
                error: $e,
            });
            return Ok(report);
        }};
    }
    if cancelled(0) {
        fail!(0, None, Stage::Cancelled, Failure::Cancelled);
    }
    let mut left = match factory.fork(reference) {
        Ok(v) => v,
        Err(e) => fail!(0, Some(Branch::Reference), Stage::Fork, Failure::Adapter(e)),
    };
    let mut right = match factory.fork(perturbed) {
        Ok(v) => v,
        Err(e) => fail!(0, Some(Branch::Perturbed), Stage::Fork, Failure::Adapter(e)),
    };
    for depth in 1..=limits.horizon {
        if cancelled(depth) {
            fail!(depth, None, Stage::Cancelled, Failure::Cancelled);
        }
        let a = match left.advance(StepContext {
            depth,
            noise_stream: 0,
        }) {
            Ok(v) => v,
            Err(e) => fail!(
                depth,
                Some(Branch::Reference),
                Stage::Advance,
                Failure::Adapter(e)
            ),
        };
        let b = match right.advance(StepContext {
            depth,
            noise_stream: u64::from(coupling == NoiseCoupling::Independent),
        }) {
            Ok(v) => v,
            Err(e) => fail!(
                depth,
                Some(Branch::Perturbed),
                Stage::Advance,
                Failure::Adapter(e)
            ),
        };
        let score = match metric(&a, &b) {
            Ok(v) => v,
            Err(e) => fail!(depth, None, Stage::Metric, Failure::Metric(e)),
        };
        if let Err(e) = sink(depth, &score) {
            fail!(depth, None, Stage::Sink, Failure::Sink(e));
        }
        if limits.retains(depth) {
            points.push(RecoveryPoint::new(depth, score));
        }
        report.completed_depth = depth;
    }
    report.profile = RecoveryProfile::new(points);
    Ok(report)
}
#[derive(Clone, Debug, PartialEq)]
pub enum ConformanceError<E> {
    Adapter(E),
    SourceMutated,
    ReplayMismatch,
    OrderMismatch,
}
/// Deterministic fixture qualification, not a universal proof. Checkpoints must
/// be value snapshots with meaningful equality, not shared mutable handles.
pub fn check_replay_conformance<A>(
    factory: &A,
    contexts: &[StepContext],
) -> Result<(), ConformanceError<A::Error>>
where
    A: ReplayAdapter,
    A::Checkpoint: PartialEq,
    A::Observation: PartialEq,
{
    let original = factory.checkpoint().map_err(ConformanceError::Adapter)?;
    let mut left = factory.fork(&original).map_err(ConformanceError::Adapter)?;
    let mut right = factory.fork(&original).map_err(ConformanceError::Adapter)?;
    for &context in contexts {
        let before = left.checkpoint().map_err(ConformanceError::Adapter)?;
        let a = left.advance(context).map_err(ConformanceError::Adapter)?;
        if factory.checkpoint().map_err(ConformanceError::Adapter)? != original {
            return Err(ConformanceError::SourceMutated);
        }
        let b = right.advance(context).map_err(ConformanceError::Adapter)?;
        if a != b {
            return Err(ConformanceError::OrderMismatch);
        }
        let mut replay = factory.fork(&before).map_err(ConformanceError::Adapter)?;
        if replay.advance(context).map_err(ConformanceError::Adapter)? != a {
            return Err(ConformanceError::ReplayMismatch);
        }
    }
    Ok(())
}
/// Compatibility bridge for qualified immutable legacy dynamics and observables.
#[derive(Clone)]
pub struct DeterministicAdapter<D: crate::ReferenceDynamics, O> {
    dynamics: D,
    observable: O,
    state: D::State,
}
#[derive(Clone, Debug, PartialEq)]
pub enum DeterministicError<D, O> {
    Dynamics(D),
    Observable(O),
}
impl<D: crate::ReferenceDynamics, O> DeterministicAdapter<D, O> {
    pub fn new(dynamics: D, observable: O, state: D::State) -> Self {
        Self {
            dynamics,
            observable,
            state,
        }
    }
}
impl<D, O> ReplayAdapter for DeterministicAdapter<D, O>
where
    D: crate::ReferenceDynamics + Clone,
    O: crate::FutureObservable<D::State> + Clone,
{
    type Checkpoint = D::State;
    type Observation = O::Output;
    type Error = DeterministicError<D::Error, O::Error>;
    fn fork(&self, c: &D::State) -> Result<Self, Self::Error> {
        Ok(Self::new(
            self.dynamics.clone(),
            self.observable.clone(),
            c.clone(),
        ))
    }
    fn checkpoint(&self) -> Result<D::State, Self::Error> {
        Ok(self.state.clone())
    }
    fn advance(&mut self, c: StepContext) -> Result<O::Output, Self::Error> {
        let next = self
            .dynamics
            .advance(&self.state)
            .map_err(DeterministicError::Dynamics)?;
        let out = self
            .observable
            .observe(&next, c.depth)
            .map_err(DeterministicError::Observable)?;
        self.state = next;
        Ok(out)
    }
}
