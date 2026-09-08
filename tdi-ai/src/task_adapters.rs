//! Reusable qualified TDI-8.1 symbolic task adapters.
//!
//! This module preserves the reviewed A0/A1/A2 adapter policies while leaving
//! fixture-only recurrent parameters, memory sizing, seeds, gains and readout
//! layouts to preflight clients.

use core::fmt;
use std::error::Error;

use crate::ReferenceArm;
use crate::associative_memory::{
    AssociativeMemoryError, AssociativeMemoryLayout, AssociativeWriteOutcome,
};
use crate::assr_reference::{
    A1Reference, A2ReadStatus, A2Reference, A2StepReport, RecurrentParameters,
    RecurrentReferenceError,
};
use crate::full_history_reference::{A0Reference, A0ReferenceError, FullHistoryLayout};
use crate::task_encoding::{
    A0_TASK_KEY_WIDTH, A0_TASK_VALUE_WIDTH, LosslessTaskEncoder, PayloadKeyCursor,
    TaskEncodingError, TaskInputLayout, a0_association_item, a0_association_query_key,
    a0_distractor_item, a0_payload_item, a0_payload_query_key, association_memory_key,
    payload_memory_key,
};
use crate::task_execution::{SymbolicTaskAdapter, TaskPrediction};
use crate::task_generators::TaskSymbol;
use crate::task_readout::{
    ExactStatePrediction, ExactStateSymbolReadout, TaskReadoutError,
    decode_exact_symbol_coordinates,
};

/// Technical failures raised by the qualified A0/A1/A2 task adapters.
#[derive(Debug)]
pub enum AdapterError {
    /// A0 full-history reference failure.
    A0(A0ReferenceError),
    /// Associative-memory failure used by A2.
    Associative(AssociativeMemoryError),
    /// Leakage-safe task encoding failure.
    Encoding(TaskEncodingError),
    /// A1/A2 recurrent reference failure.
    Recurrent(RecurrentReferenceError),
    /// Exact target-blind readout failure.
    Readout(TaskReadoutError),
    /// Recurrent/readout state widths disagree.
    ReadoutStateWidthMismatch {
        /// Recurrent state width.
        recurrent: u64,
        /// Readout state width.
        readout: u64,
    },
    /// A0 payload position counter overflowed.
    PayloadPositionOverflow,
    /// A2 neutral non-query read unexpectedly hit resident memory.
    UnexpectedNeutralReadHit {
        /// Physical address that unexpectedly hit.
        address: u64,
    },
    /// A2 diagnostic counter overflowed.
    DiagnosticCounterOverflow,
}

impl fmt::Display for AdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::A0(error) => write!(formatter, "A0 reference: {error}"),
            Self::Associative(error) => write!(formatter, "associative memory: {error}"),
            Self::Encoding(error) => write!(formatter, "task encoding: {error}"),
            Self::Recurrent(error) => write!(formatter, "recurrent reference: {error}"),
            Self::Readout(error) => write!(formatter, "exact readout: {error}"),
            Self::ReadoutStateWidthMismatch { recurrent, readout } => write!(
                formatter,
                "recurrent state width {recurrent} does not match readout state width {readout}"
            ),
            Self::PayloadPositionOverflow => {
                formatter.write_str("A0 payload position counter overflow")
            }
            Self::UnexpectedNeutralReadHit { address } => write!(
                formatter,
                "A2 neutral non-query read unexpectedly hit resident memory at address {address}"
            ),
            Self::DiagnosticCounterOverflow => {
                formatter.write_str("A2 adapter diagnostic counter overflow")
            }
        }
    }
}

impl Error for AdapterError {}

impl From<A0ReferenceError> for AdapterError {
    fn from(error: A0ReferenceError) -> Self {
        Self::A0(error)
    }
}

impl From<AssociativeMemoryError> for AdapterError {
    fn from(error: AssociativeMemoryError) -> Self {
        Self::Associative(error)
    }
}

impl From<TaskEncodingError> for AdapterError {
    fn from(error: TaskEncodingError) -> Self {
        Self::Encoding(error)
    }
}

impl From<RecurrentReferenceError> for AdapterError {
    fn from(error: RecurrentReferenceError) -> Self {
        Self::Recurrent(error)
    }
}

impl From<TaskReadoutError> for AdapterError {
    fn from(error: TaskReadoutError) -> Self {
        Self::Readout(error)
    }
}

/// Qualified A0 namespaced full-history symbolic task adapter.
pub struct A0Adapter {
    reference: A0Reference,
    next_payload_position: u64,
}

impl A0Adapter {
    /// Construct the reviewed A0 task layout with no experimental dimensions.
    pub fn new() -> Result<Self, AdapterError> {
        Ok(Self {
            reference: A0Reference::new(FullHistoryLayout::new(
                A0_TASK_KEY_WIDTH as u64,
                A0_TASK_VALUE_WIDTH as u64,
            )?)?,
            next_payload_position: 0,
        })
    }

    fn decode_readout(&self, query: &[f64]) -> Result<TaskPrediction, AdapterError> {
        let readout = self.reference.read(query)?;
        let coordinates: [f64; A0_TASK_VALUE_WIDTH] =
            readout
                .value()
                .try_into()
                .map_err(|_| A0ReferenceError::ValueWidthMismatch {
                    expected: A0_TASK_VALUE_WIDTH,
                    actual: readout.value().len(),
                })?;
        Ok(TaskPrediction::Symbol(decode_exact_symbol_coordinates(
            coordinates,
        )?))
    }
}

impl SymbolicTaskAdapter for A0Adapter {
    type Error = AdapterError;

    fn arm(&self) -> ReferenceArm {
        ReferenceArm::A0
    }

    fn reset(&mut self) -> Result<(), Self::Error> {
        self.reference.clear();
        self.next_payload_position = 0;
        Ok(())
    }

    fn associate(&mut self, key_code: u64, value: TaskSymbol) -> Result<(), Self::Error> {
        let item = a0_association_item(key_code, value);
        self.reference.append(&item.key(), &item.value())?;
        Ok(())
    }

    fn payload(&mut self, value: TaskSymbol) -> Result<(), Self::Error> {
        let position = self.next_payload_position;
        self.next_payload_position = self
            .next_payload_position
            .checked_add(1)
            .ok_or(AdapterError::PayloadPositionOverflow)?;
        let item = a0_payload_item(position, value);
        self.reference.append(&item.key(), &item.value())?;
        Ok(())
    }

    fn distractor(&mut self, token: TaskSymbol) -> Result<(), Self::Error> {
        let item = a0_distractor_item(token);
        self.reference.append(&item.key(), &item.value())?;
        Ok(())
    }

    fn query_association(&mut self, key_code: u64) -> Result<TaskPrediction, Self::Error> {
        self.decode_readout(&a0_association_query_key(key_code))
    }

    fn query_payload(&mut self, position: u64) -> Result<TaskPrediction, Self::Error> {
        self.decode_readout(&a0_payload_query_key(position))
    }
}

/// Qualified A1 leakage-safe recurrent symbolic task adapter.
pub struct A1Adapter {
    reference: A1Reference,
    encoder: LosslessTaskEncoder,
    readout: ExactStateSymbolReadout,
}

impl A1Adapter {
    /// Construct A1 from caller-supplied reviewed recurrent and readout parameters.
    pub fn new(
        parameters: RecurrentParameters,
        readout: ExactStateSymbolReadout,
    ) -> Result<Self, AdapterError> {
        let recurrent_layout = parameters.layout();
        let readout_layout = readout.layout();
        if readout_layout.state_width() != recurrent_layout.state_width() {
            return Err(AdapterError::ReadoutStateWidthMismatch {
                recurrent: recurrent_layout.state_width(),
                readout: readout_layout.state_width(),
            });
        }
        Ok(Self {
            reference: A1Reference::new(parameters)?,
            encoder: LosslessTaskEncoder::new(TaskInputLayout::new(
                recurrent_layout.input_width(),
            )?),
            readout,
        })
    }

    fn step(&mut self, input: Vec<f64>) -> Result<(), AdapterError> {
        self.reference.step(&input)?;
        Ok(())
    }

    fn query(&mut self, input: Vec<f64>) -> Result<TaskPrediction, AdapterError> {
        self.reference.step(&input)?;
        Ok(match self.readout.decode_state(self.reference.state())? {
            ExactStatePrediction::Symbol(symbol) => TaskPrediction::Symbol(symbol),
            ExactStatePrediction::InvalidEncoding => TaskPrediction::Invalid,
        })
    }
}

impl SymbolicTaskAdapter for A1Adapter {
    type Error = AdapterError;

    fn arm(&self) -> ReferenceArm {
        ReferenceArm::A1
    }

    fn reset(&mut self) -> Result<(), Self::Error> {
        self.reference.reset();
        Ok(())
    }

    fn associate(&mut self, key_code: u64, value: TaskSymbol) -> Result<(), Self::Error> {
        let input = self.encoder.association(key_code, value)?;
        self.step(input)
    }

    fn payload(&mut self, value: TaskSymbol) -> Result<(), Self::Error> {
        let input = self.encoder.payload(value)?;
        self.step(input)
    }

    fn distractor(&mut self, token: TaskSymbol) -> Result<(), Self::Error> {
        let input = self.encoder.distractor(token)?;
        self.step(input)
    }

    fn query_association(&mut self, key_code: u64) -> Result<TaskPrediction, Self::Error> {
        let input = self.encoder.query_association(key_code)?;
        self.query(input)
    }

    fn query_payload(&mut self, position: u64) -> Result<TaskPrediction, Self::Error> {
        let input = self.encoder.query_payload(position)?;
        self.query(input)
    }
}

/// Evaluator-owned A2 runtime diagnostics retained by the qualified adapter.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct A2Diagnostics {
    query_hits: u64,
    query_collision_misses: u64,
    query_empty: u64,
    inserted_writes: u64,
    updated_writes: u64,
    replacement_writes: u64,
}

impl A2Diagnostics {
    fn increment(value: &mut u64) -> Result<(), AdapterError> {
        *value = value
            .checked_add(1)
            .ok_or(AdapterError::DiagnosticCounterOverflow)?;
        Ok(())
    }

    fn observe_query(&mut self, status: A2ReadStatus) -> Result<(), AdapterError> {
        match status {
            A2ReadStatus::Hit { .. } => Self::increment(&mut self.query_hits),
            A2ReadStatus::CollisionMiss { .. } => Self::increment(&mut self.query_collision_misses),
            A2ReadStatus::Empty { .. } => Self::increment(&mut self.query_empty),
        }
    }

    fn observe_write(
        &mut self,
        outcome: Option<AssociativeWriteOutcome>,
    ) -> Result<(), AdapterError> {
        match outcome {
            Some(AssociativeWriteOutcome::Inserted { .. }) => Self::increment(&mut self.inserted_writes),
            Some(AssociativeWriteOutcome::Updated { .. }) => Self::increment(&mut self.updated_writes),
            Some(AssociativeWriteOutcome::ReplacedCollision { .. }) => {
                Self::increment(&mut self.replacement_writes)
            }
            None => Ok(()),
        }
    }

    /// Number of logical queries that hit their resident key.
    #[must_use]
    pub fn query_hits(self) -> u64 {
        self.query_hits
    }

    /// Number of logical queries rejected by a physical key collision.
    #[must_use]
    pub fn query_collision_misses(self) -> u64 {
        self.query_collision_misses
    }

    /// Number of logical queries that addressed an empty slot.
    #[must_use]
    pub fn query_empty(self) -> u64 {
        self.query_empty
    }

    /// Number of writes that inserted into an empty slot.
    #[must_use]
    pub fn inserted_writes(self) -> u64 {
        self.inserted_writes
    }

    /// Number of writes that updated an existing logical key.
    #[must_use]
    pub fn updated_writes(self) -> u64 {
        self.updated_writes
    }

    /// Number of writes that replaced a colliding logical key.
    #[must_use]
    pub fn replacement_writes(self) -> u64 {
        self.replacement_writes
    }
}

/// Qualified A2 recurrent plus direct-mapped associative-memory task adapter.
pub struct A2Adapter {
    reference: A2Reference,
    encoder: LosslessTaskEncoder,
    readout: ExactStateSymbolReadout,
    payload_keys: PayloadKeyCursor,
    neutral_read_key: u64,
    diagnostics: A2Diagnostics,
}

impl A2Adapter {
    /// Construct A2 from caller-supplied reviewed recurrent/memory parameters.
    pub fn new(
        parameters: RecurrentParameters,
        memory_layout: AssociativeMemoryLayout,
        projection_seed: u64,
        fusion_gain: f64,
        readout: ExactStateSymbolReadout,
        neutral_read_key: u64,
    ) -> Result<Self, AdapterError> {
        let recurrent_layout = parameters.layout();
        let readout_layout = readout.layout();
        if recurrent_layout.state_width() != readout_layout.state_width() {
            return Err(AdapterError::ReadoutStateWidthMismatch {
                recurrent: recurrent_layout.state_width(),
                readout: readout_layout.state_width(),
            });
        }
        Ok(Self {
            reference: A2Reference::new(parameters, memory_layout, projection_seed, fusion_gain)?,
            encoder: LosslessTaskEncoder::new(TaskInputLayout::new(
                recurrent_layout.input_width(),
            )?),
            readout,
            payload_keys: PayloadKeyCursor::default(),
            neutral_read_key,
            diagnostics: A2Diagnostics::default(),
        })
    }

    /// Return evaluator-owned hit/collision/write diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> A2Diagnostics {
        self.diagnostics
    }

    fn non_query_step(
        &mut self,
        input: Vec<f64>,
        write_key: Option<u64>,
    ) -> Result<(), AdapterError> {
        let report = self.reference.step(&input, self.neutral_read_key, write_key)?;
        if let A2ReadStatus::Hit { address } = report.read() {
            return Err(AdapterError::UnexpectedNeutralReadHit { address });
        }
        self.diagnostics.observe_write(report.write())?;
        Ok(())
    }

    fn query_step(
        &mut self,
        input: Vec<f64>,
        read_key: u64,
    ) -> Result<TaskPrediction, AdapterError> {
        let report: A2StepReport = self.reference.step(&input, read_key, None)?;
        self.diagnostics.observe_query(report.read())?;
        Ok(match self.readout.decode_state(self.reference.state())? {
            ExactStatePrediction::Symbol(symbol) => TaskPrediction::Symbol(symbol),
            ExactStatePrediction::InvalidEncoding => TaskPrediction::Invalid,
        })
    }
}

impl SymbolicTaskAdapter for A2Adapter {
    type Error = AdapterError;

    fn arm(&self) -> ReferenceArm {
        ReferenceArm::A2
    }

    fn reset(&mut self) -> Result<(), Self::Error> {
        self.reference.reset();
        self.payload_keys.reset();
        self.diagnostics = A2Diagnostics::default();
        Ok(())
    }

    fn associate(&mut self, key_code: u64, value: TaskSymbol) -> Result<(), Self::Error> {
        let input = self.encoder.association(key_code, value)?;
        self.non_query_step(input, Some(association_memory_key(key_code)))
    }

    fn payload(&mut self, value: TaskSymbol) -> Result<(), Self::Error> {
        let input = self.encoder.payload(value)?;
        let mut next_payload_keys = self.payload_keys;
        let write_key = next_payload_keys.next_write_key()?;
        self.non_query_step(input, Some(write_key))?;
        self.payload_keys = next_payload_keys;
        Ok(())
    }

    fn distractor(&mut self, token: TaskSymbol) -> Result<(), Self::Error> {
        let input = self.encoder.distractor(token)?;
        self.non_query_step(input, None)
    }

    fn query_association(&mut self, key_code: u64) -> Result<TaskPrediction, Self::Error> {
        let input = self.encoder.query_association(key_code)?;
        self.query_step(input, association_memory_key(key_code))
    }

    fn query_payload(&mut self, position: u64) -> Result<TaskPrediction, Self::Error> {
        let input = self.encoder.query_payload(position)?;
        self.query_step(input, payload_memory_key(position))
    }
}
