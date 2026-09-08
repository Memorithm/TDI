//! Qualified reusable TDI-8.1 A3 symbolic task adapter.
//!
//! The adapter preserves the reviewed atomic A2+VSA write transaction, routed
//! VSA reads, transactional payload cursor, target-blind readouts and evaluator-
//! owned diagnostics. Experimental fixture dimensions, seeds and gains remain
//! caller supplied.

use core::fmt;
use std::error::Error;

use crate::ReferenceArm;
use crate::associative_memory::{
    AssociativeMemoryError, AssociativeMemoryLayout, AssociativeWriteOutcome,
};
use crate::assr_h_reference::{A3Reference, A3ReferenceError, A3VsaReadRoute};
use crate::assr_reference::{A2ReadStatus, A2StepReport, RecurrentParameters};
use crate::task_encoding::{
    LosslessTaskEncoder, PayloadKeyCursor, TaskEncodingError, TaskInputLayout,
    association_memory_key, payload_memory_key,
};
use crate::task_execution::{SymbolicTaskAdapter, TaskPrediction};
use crate::task_generators::TaskSymbol;
use crate::task_readout::{ExactStatePrediction, ExactStateSymbolReadout, TaskReadoutError};
use crate::vsa_workspace::VsaWorkspaceLayout;

/// Technical failures raised by the qualified A3 task adapter.
#[derive(Debug)]
pub enum A3AdapterError {
    /// Associative-memory failure.
    Associative(AssociativeMemoryError),
    /// Leakage-safe task encoding failure.
    Encoding(TaskEncodingError),
    /// A3 reference failure.
    A3(A3ReferenceError),
    /// Exact target-blind readout failure.
    Readout(TaskReadoutError),
    /// Recurrent/readout state widths disagree.
    ReadoutStateWidthMismatch {
        /// Recurrent state width.
        recurrent: u64,
        /// Association readout state width.
        association_readout: u64,
        /// Payload readout state width.
        payload_readout: u64,
    },
    /// Neutral non-query A2 read unexpectedly hit resident memory.
    UnexpectedNeutralReadHit {
        /// Physical address that unexpectedly hit.
        address: u64,
    },
    /// An evaluator-owned diagnostic counter overflowed.
    CounterOverflow,
}

impl fmt::Display for A3AdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Associative(error) => write!(formatter, "associative memory: {error}"),
            Self::Encoding(error) => write!(formatter, "task encoding: {error}"),
            Self::A3(error) => write!(formatter, "A3 reference: {error}"),
            Self::Readout(error) => write!(formatter, "exact readout: {error}"),
            Self::ReadoutStateWidthMismatch {
                recurrent,
                association_readout,
                payload_readout,
            } => write!(
                formatter,
                "A3 recurrent state width {recurrent} does not match association/payload readout widths {association_readout}/{payload_readout}"
            ),
            Self::UnexpectedNeutralReadHit { address } => write!(
                formatter,
                "A3 neutral non-query A2 read unexpectedly hit resident memory at address {address}"
            ),
            Self::CounterOverflow => formatter.write_str("A3 adapter diagnostic counter overflow"),
        }
    }
}

impl Error for A3AdapterError {}

impl From<AssociativeMemoryError> for A3AdapterError {
    fn from(error: AssociativeMemoryError) -> Self {
        Self::Associative(error)
    }
}

impl From<TaskEncodingError> for A3AdapterError {
    fn from(error: TaskEncodingError) -> Self {
        Self::Encoding(error)
    }
}

impl From<A3ReferenceError> for A3AdapterError {
    fn from(error: A3ReferenceError) -> Self {
        Self::A3(error)
    }
}

impl From<TaskReadoutError> for A3AdapterError {
    fn from(error: TaskReadoutError) -> Self {
        Self::Readout(error)
    }
}

/// Evaluator-owned A3 runtime diagnostics retained by the qualified adapter.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct A3Diagnostics {
    query_hits: u64,
    query_collision_misses: u64,
    query_empty: u64,
    inserted_writes: u64,
    updated_writes: u64,
    replacement_writes: u64,
    vsa_stores: u64,
    vsa_queries: u64,
}

impl A3Diagnostics {
    fn increment(value: &mut u64) -> Result<(), A3AdapterError> {
        *value = value.checked_add(1).ok_or(A3AdapterError::CounterOverflow)?;
        Ok(())
    }

    fn observe_query(&mut self, status: A2ReadStatus) -> Result<(), A3AdapterError> {
        match status {
            A2ReadStatus::Hit { .. } => Self::increment(&mut self.query_hits),
            A2ReadStatus::CollisionMiss { .. } => Self::increment(&mut self.query_collision_misses),
            A2ReadStatus::Empty { .. } => Self::increment(&mut self.query_empty),
        }
    }

    fn observe_write(
        &mut self,
        outcome: Option<AssociativeWriteOutcome>,
    ) -> Result<(), A3AdapterError> {
        match outcome {
            Some(AssociativeWriteOutcome::Inserted { .. }) => {
                Self::increment(&mut self.inserted_writes)
            }
            Some(AssociativeWriteOutcome::Updated { .. }) => {
                Self::increment(&mut self.updated_writes)
            }
            Some(AssociativeWriteOutcome::ReplacedCollision { .. }) => {
                Self::increment(&mut self.replacement_writes)
            }
            None => Ok(()),
        }
    }

    /// Number of logical queries that hit their resident A2 key.
    #[must_use]
    pub fn query_hits(self) -> u64 {
        self.query_hits
    }

    /// Number of logical queries rejected by a physical A2 key collision.
    #[must_use]
    pub fn query_collision_misses(self) -> u64 {
        self.query_collision_misses
    }

    /// Number of logical queries that addressed an empty A2 slot.
    #[must_use]
    pub fn query_empty(self) -> u64 {
        self.query_empty
    }

    /// Number of A2 writes inserted into an empty slot.
    #[must_use]
    pub fn inserted_writes(self) -> u64 {
        self.inserted_writes
    }

    /// Number of A2 writes updating an existing logical key.
    #[must_use]
    pub fn updated_writes(self) -> u64 {
        self.updated_writes
    }

    /// Number of A2 writes replacing a colliding logical key.
    #[must_use]
    pub fn replacement_writes(self) -> u64 {
        self.replacement_writes
    }

    /// Number of atomic VSA stores committed.
    #[must_use]
    pub fn vsa_stores(self) -> u64 {
        self.vsa_stores
    }

    /// Number of routed VSA queries executed.
    #[must_use]
    pub fn vsa_queries(self) -> u64 {
        self.vsa_queries
    }
}

/// Qualified A3 recurrent + A2 associative-memory + VSA symbolic task adapter.
pub struct A3Adapter {
    reference: A3Reference,
    encoder: LosslessTaskEncoder,
    association_readout: ExactStateSymbolReadout,
    payload_readout: ExactStateSymbolReadout,
    payload_keys: PayloadKeyCursor,
    neutral_read_key: u64,
    diagnostics: A3Diagnostics,
}

impl A3Adapter {
    /// Construct A3 from caller-supplied reviewed recurrent, memory, VSA and
    /// readout parameters. No experimental defaults are selected here.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        parameters: RecurrentParameters,
        memory_layout: AssociativeMemoryLayout,
        projection_seed: u64,
        associative_fusion_gain: f64,
        vsa_role_seed: u64,
        vsa_fusion_gain: f64,
        association_readout: ExactStateSymbolReadout,
        payload_readout: ExactStateSymbolReadout,
        neutral_read_key: u64,
    ) -> Result<Self, A3AdapterError> {
        let recurrent_layout = parameters.layout();
        let association_layout = association_readout.layout();
        let payload_layout = payload_readout.layout();
        if recurrent_layout.state_width() != association_layout.state_width()
            || recurrent_layout.state_width() != payload_layout.state_width()
        {
            return Err(A3AdapterError::ReadoutStateWidthMismatch {
                recurrent: recurrent_layout.state_width(),
                association_readout: association_layout.state_width(),
                payload_readout: payload_layout.state_width(),
            });
        }
        let input_width = recurrent_layout.input_width();
        let vsa_layout = VsaWorkspaceLayout::new(input_width).map_err(A3ReferenceError::from)?;
        Ok(Self {
            reference: A3Reference::new(
                parameters,
                memory_layout,
                projection_seed,
                associative_fusion_gain,
                vsa_layout,
                vsa_role_seed,
                vsa_fusion_gain,
            )?,
            encoder: LosslessTaskEncoder::new(TaskInputLayout::new(input_width)?),
            association_readout,
            payload_readout,
            payload_keys: PayloadKeyCursor::default(),
            neutral_read_key,
            diagnostics: A3Diagnostics::default(),
        })
    }

    /// Return evaluator-owned A2/VSA runtime diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> A3Diagnostics {
        self.diagnostics
    }

    /// Current ordered-payload position, exposed for transactional qualification.
    #[must_use]
    pub fn payload_position(&self) -> u64 {
        self.payload_keys.next_position()
    }

    /// Current VSA workspace components, exposed read-only for no-mutation qualification.
    #[must_use]
    pub fn vsa_components(&self) -> &[f64] {
        self.reference.workspace().components()
    }

    fn atomic_store_step(
        &mut self,
        input: Vec<f64>,
        logical_key: u64,
    ) -> Result<(), A3AdapterError> {
        let report = self.reference.step_skip_vsa_and_store(
            &input,
            self.neutral_read_key,
            Some(logical_key),
            logical_key,
            &input,
        )?;
        if let A2ReadStatus::Hit { address } = report.read() {
            return Err(A3AdapterError::UnexpectedNeutralReadHit { address });
        }
        self.diagnostics.observe_write(report.write())?;
        A3Diagnostics::increment(&mut self.diagnostics.vsa_stores)?;
        Ok(())
    }

    fn distractor_step(&mut self, input: Vec<f64>) -> Result<(), A3AdapterError> {
        let report = self.reference.step_routed(
            &input,
            A3VsaReadRoute::Skip,
            self.neutral_read_key,
            None,
        )?;
        if let A2ReadStatus::Hit { address } = report.read() {
            return Err(A3AdapterError::UnexpectedNeutralReadHit { address });
        }
        Ok(())
    }

    fn query_step(
        &mut self,
        input: Vec<f64>,
        read_key: u64,
        readout: ExactStateSymbolReadout,
    ) -> Result<TaskPrediction, A3AdapterError> {
        let report: A2StepReport =
            self.reference
                .step_routed(&input, A3VsaReadRoute::Key(read_key), read_key, None)?;
        self.diagnostics.observe_query(report.read())?;
        A3Diagnostics::increment(&mut self.diagnostics.vsa_queries)?;
        Ok(match readout.decode_state(self.reference.state())? {
            ExactStatePrediction::Symbol(symbol) => TaskPrediction::Symbol(symbol),
            ExactStatePrediction::InvalidEncoding => TaskPrediction::Invalid,
        })
    }
}

impl SymbolicTaskAdapter for A3Adapter {
    type Error = A3AdapterError;

    fn arm(&self) -> ReferenceArm {
        ReferenceArm::A3
    }

    fn reset(&mut self) -> Result<(), Self::Error> {
        self.reference.reset();
        self.payload_keys.reset();
        self.diagnostics = A3Diagnostics::default();
        Ok(())
    }

    fn associate(&mut self, key_code: u64, value: TaskSymbol) -> Result<(), Self::Error> {
        let input = self.encoder.association(key_code, value)?;
        self.atomic_store_step(input, association_memory_key(key_code))
    }

    fn payload(&mut self, value: TaskSymbol) -> Result<(), Self::Error> {
        let input = self.encoder.payload(value)?;
        let mut next_payload_keys = self.payload_keys;
        let logical_key = next_payload_keys.next_write_key()?;
        self.atomic_store_step(input, logical_key)?;
        self.payload_keys = next_payload_keys;
        Ok(())
    }

    fn distractor(&mut self, token: TaskSymbol) -> Result<(), Self::Error> {
        let input = self.encoder.distractor(token)?;
        self.distractor_step(input)
    }

    fn query_association(&mut self, key_code: u64) -> Result<TaskPrediction, Self::Error> {
        let input = self.encoder.query_association(key_code)?;
        self.query_step(
            input,
            association_memory_key(key_code),
            self.association_readout,
        )
    }

    fn query_payload(&mut self, position: u64) -> Result<TaskPrediction, Self::Error> {
        let input = self.encoder.query_payload(position)?;
        self.query_step(input, payload_memory_key(position), self.payload_readout)
    }
}
