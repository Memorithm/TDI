//! Reusable qualified TDI-8.1 symbolic task adapters.
//!
//! This module preserves the reviewed A0/A1 adapter policies while leaving
//! fixture-only recurrent parameters and readout layouts to preflight clients.

use core::fmt;
use std::error::Error;

use crate::ReferenceArm;
use crate::assr_reference::{A1Reference, RecurrentParameters, RecurrentReferenceError};
use crate::full_history_reference::{A0Reference, A0ReferenceError, FullHistoryLayout};
use crate::task_encoding::{
    A0_TASK_KEY_WIDTH, A0_TASK_VALUE_WIDTH, LosslessTaskEncoder, TaskEncodingError,
    TaskInputLayout, a0_association_item, a0_association_query_key, a0_distractor_item,
    a0_payload_item, a0_payload_query_key,
};
use crate::task_execution::{SymbolicTaskAdapter, TaskPrediction};
use crate::task_generators::TaskSymbol;
use crate::task_readout::{
    ExactStatePrediction, ExactStateSymbolReadout, TaskReadoutError,
    decode_exact_symbol_coordinates,
};

/// Technical failures raised by the qualified A0/A1 task adapters.
#[derive(Debug)]
pub enum AdapterError {
    /// A0 full-history reference failure.
    A0(A0ReferenceError),
    /// Leakage-safe task encoding failure.
    Encoding(TaskEncodingError),
    /// A1 recurrent reference failure.
    Recurrent(RecurrentReferenceError),
    /// Exact target-blind readout failure.
    Readout(TaskReadoutError),
    /// A1 recurrent/readout state widths disagree.
    ReadoutStateWidthMismatch {
        /// Recurrent state width.
        recurrent: u64,
        /// Readout state width.
        readout: u64,
    },
    /// A0 payload position counter overflowed.
    PayloadPositionOverflow,
}

impl fmt::Display for AdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::A0(error) => write!(formatter, "A0 reference: {error}"),
            Self::Encoding(error) => write!(formatter, "task encoding: {error}"),
            Self::Recurrent(error) => write!(formatter, "A1 recurrent reference: {error}"),
            Self::Readout(error) => write!(formatter, "exact readout: {error}"),
            Self::ReadoutStateWidthMismatch { recurrent, readout } => write!(
                formatter,
                "A1 recurrent state width {recurrent} does not match readout state width {readout}"
            ),
            Self::PayloadPositionOverflow => {
                formatter.write_str("A0 payload position counter overflow")
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
