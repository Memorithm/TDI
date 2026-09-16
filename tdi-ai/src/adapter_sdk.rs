//! Capability and checkpoint-codec extension of the existing replay interface.
//!
//! Generation and intervention prepare checkpoints; execution/observation use
//! [`ReplayAdapter`]; scoring and persistence remain caller-owned closures in
//! [`crate::experiment::run_paired`]. This SDK adds no scheduler or final runner.

use crate::experiment::{ConformanceError, ReplayAdapter, StepContext, check_replay_conformance};

/// Reproducibility class actually declared by an adapter implementation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reproduction {
    /// Exact replay for the declared ordered inputs, build and target.
    ExactBuildTarget,
    /// Numerical comparison needs a caller-frozen tolerance and independent oracle.
    NumericalWithDeclaredTolerance,
    /// Distributional comparison needs a separate caller-owned statistical protocol.
    StatisticalProtocolRequired,
}

/// Adapter semantics and structural bounds, not measured physical resource use.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AdapterContract {
    /// Versioned implementation/capability identifier; deployment also pins code bytes.
    pub implementation: &'static str,
    /// Logical advancement unit (never implicitly seconds).
    pub advancement_unit: &'static str,
    /// Unit and meaning of the returned observation.
    pub observation_unit: &'static str,
    /// Numeric representation and accumulation semantics.
    pub precision: &'static str,
    /// Declared reproducibility scope.
    pub reproduction: Reproduction,
    /// RNG algorithm or `none`; stochastic adapters must checkpoint complete state.
    pub rng: &'static str,
    /// Maximum encoded checkpoint bytes accepted by the codec.
    pub max_checkpoint_bytes: usize,
    /// Maximum logical advances from a fresh checkpoint.
    pub max_steps: usize,
    /// Bound on explicitly owned state scalars, not an RSS measurement.
    pub max_state_scalars: usize,
    /// Input permission boundary.
    pub access: &'static str,
    /// Cancellation/deadline scope and unsupported modes.
    pub execution_limits: &'static str,
}

/// A complete, versioned value-checkpoint codec for a replay adapter.
///
/// Decoding must bound allocation and reject wrong versions, truncation,
/// trailing data, invalid values and incompatible immutable configuration.
/// It must not mutate the source. Serialized caches/RNG are part of the state,
/// not optional hints whose omission can silently change future execution.
pub trait ReplayCodec: ReplayAdapter {
    /// Describe implementation capabilities and explicit limitations.
    fn contract(&self) -> AdapterContract;
    /// Absolute logical depth of the current complete state.
    fn progress(&self) -> usize;
    /// Encode a complete canonical checkpoint without changing the source.
    fn encode_checkpoint(&self) -> Result<Vec<u8>, Self::Error>;
    /// Decode a complete checkpoint compatible with this adapter's configuration.
    fn decode_checkpoint(&self, bytes: &[u8]) -> Result<Self::Checkpoint, Self::Error>;
}

/// A restored paired branch has an incompatible origin or its depth overflows.
#[derive(Debug, PartialEq)]
pub enum RelativeReplayError<E> {
    /// The underlying adapter rejected an operation.
    Adapter(E),
    /// Paired checkpoints must start at the same absolute logical progress.
    OriginMismatch,
    /// Origin plus requested advancement exceeds the adapter's lifetime budget.
    DepthLimit,
}

/// Translate the existing paired runner's relative depths to a restored origin.
///
/// Prepare this factory explicitly from one checkpoint, then pass it and both
/// complete checkpoints to `run_paired`. Every fork validates the same origin.
/// Scores, cancellation callbacks, sink depths and report depths remain relative
/// to this run; add `origin()` when an absolute timeline is needed. Preparation
/// executes no advances, scoring or sink. The original paired API is unchanged.
pub struct RelativeReplay<A: ReplayCodec> {
    adapter: A,
    origin: usize,
}

impl<A: ReplayCodec> RelativeReplay<A> {
    /// Prepare an owned source factory at the checkpoint's absolute origin.
    pub fn new(
        factory: &A,
        checkpoint: &A::Checkpoint,
    ) -> Result<Self, RelativeReplayError<A::Error>> {
        let adapter = factory
            .fork(checkpoint)
            .map_err(RelativeReplayError::Adapter)?;
        let origin = adapter.progress();
        if origin > adapter.contract().max_steps {
            return Err(RelativeReplayError::DepthLimit);
        }
        Ok(Self { adapter, origin })
    }

    /// Absolute logical depth represented by relative run depth zero.
    pub fn origin(&self) -> usize {
        self.origin
    }
}

impl<A: ReplayCodec> ReplayAdapter for RelativeReplay<A> {
    type Checkpoint = A::Checkpoint;
    type Observation = A::Observation;
    type Error = RelativeReplayError<A::Error>;

    fn fork(&self, checkpoint: &A::Checkpoint) -> Result<Self, Self::Error> {
        let adapter = self
            .adapter
            .fork(checkpoint)
            .map_err(RelativeReplayError::Adapter)?;
        if adapter.progress() != self.origin {
            return Err(RelativeReplayError::OriginMismatch);
        }
        Ok(Self {
            adapter,
            origin: self.origin,
        })
    }
    fn checkpoint(&self) -> Result<A::Checkpoint, Self::Error> {
        self.adapter
            .checkpoint()
            .map_err(RelativeReplayError::Adapter)
    }
    fn advance(&mut self, context: StepContext) -> Result<A::Observation, Self::Error> {
        let depth = self
            .origin
            .checked_add(context.depth)
            .ok_or(RelativeReplayError::DepthLimit)?;
        if depth > self.adapter.contract().max_steps {
            return Err(RelativeReplayError::DepthLimit);
        }
        self.adapter
            .advance(StepContext { depth, ..context })
            .map_err(RelativeReplayError::Adapter)
    }
}

/// A failed codec or independent replay qualification.
#[derive(Debug, PartialEq)]
pub enum CodecConformanceError<E> {
    /// An adapter or codec operation failed.
    Adapter(E),
    /// Encoding exceeded its declared bound or failed complete value round-trip.
    CodecMismatch,
    /// The existing branch/source/order/replay qualification failed.
    Replay(ConformanceError<E>),
}

/// Exercise codec round-trip, canonical re-encoding and existing branch checks.
///
/// This bounded fixture check is not a universal proof of an opaque backend.
/// The caller supplies valid ordered contexts and separate domain oracles.
pub fn check_codec_conformance<A>(
    adapter: &A,
    contexts: &[StepContext],
) -> Result<(), CodecConformanceError<A::Error>>
where
    A: ReplayCodec,
    A::Checkpoint: PartialEq,
    A::Observation: PartialEq,
{
    let original = adapter
        .checkpoint()
        .map_err(CodecConformanceError::Adapter)?;
    let bytes = adapter
        .encode_checkpoint()
        .map_err(CodecConformanceError::Adapter)?;
    if bytes.len() > adapter.contract().max_checkpoint_bytes {
        return Err(CodecConformanceError::CodecMismatch);
    }
    let decoded = adapter
        .decode_checkpoint(&bytes)
        .map_err(CodecConformanceError::Adapter)?;
    let restored = adapter
        .fork(&decoded)
        .map_err(CodecConformanceError::Adapter)?;
    if decoded != original
        || restored
            .encode_checkpoint()
            .map_err(CodecConformanceError::Adapter)?
            != bytes
        || adapter
            .checkpoint()
            .map_err(CodecConformanceError::Adapter)?
            != original
    {
        return Err(CodecConformanceError::CodecMismatch);
    }
    check_replay_conformance(adapter, contexts).map_err(CodecConformanceError::Replay)
}
