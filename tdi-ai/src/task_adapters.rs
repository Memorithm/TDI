//! Reusable leakage-safe A0/A1/A2/A3 symbolic adapters for bounded TDI-8.1.
//!
//! This module promotes the concrete adapter semantics already qualified by the
//! A0/A1, A2 and A3 preflight binaries. It deliberately supplies no experimental
//! dimensions, weights, capacities, projection seeds, gains, horizons, matched
//! budgets, populations, deficit rules, interval parameters or TDI-8.2 surface.
//!
//! Every adapter accumulates the exact reference-semantic operation vocabulary
//! from [`crate::reference_operation_accounting`]. Those counts are not CPU
//! instructions, FLOPs, latency, energy, bandwidth or accelerator claims.

use core::fmt;

use crate::ReferenceArm;
use crate::associative_memory::{
    AssociativeMemoryError, AssociativeMemoryLayout, AssociativeWriteOutcome,
};
use crate::assr_h_reference::{A3Reference, A3ReferenceError, A3VsaReadRoute};
use crate::assr_reference::{
    A1Reference, A2ReadStatus, A2Reference, A2StepReport, RecurrentLayout,
    RecurrentParameters, RecurrentReferenceError,
};
use crate::full_history_reference::{A0Reference, A0ReferenceError, FullHistoryLayout};
use crate::reference_operation_accounting::{
    ReferenceOperationAccounting, ReferenceOperationAccountingError,
};
use crate::task_encoding::{
    A0_TASK_KEY_WIDTH, A0_TASK_VALUE_WIDTH, LosslessTaskEncoder, PayloadKeyCursor,
    TaskEncodingError, TaskInputLayout, a0_association_item, a0_association_query_key,
    a0_distractor_item, a0_payload_item, a0_payload_query_key, association_memory_key,
    payload_memory_key,
};
use crate::task_execution::{SymbolicTaskAdapter, TaskPrediction};
use crate::task_generators::TaskSymbol;
use crate::task_readout::{
    ExactStatePrediction, ExactStateSymbolReadout, TaskReadoutError, decode_exact_symbol_coordinates,
};
use crate::vsa_workspace::VsaWorkspaceLayout;

/// Fail-closed reusable adapter errors.
#[derive(Debug)]
pub enum AssrTaskAdapterError {
    /// A0 full-history reference rejected an operation.
    A0(A0ReferenceError),
    /// Leakage-safe task encoding rejected an operation.
    Encoding(TaskEncodingError),
    /// A1/A2 recurrent reference rejected an operation.
    Recurrent(RecurrentReferenceError),
    /// A3 reference rejected an operation.
    A3(A3ReferenceError),
    /// Exact target-blind readout rejected malformed state.
    Readout(TaskReadoutError),
    /// Exact semantic operation accounting failed closed.
    OperationAccounting(ReferenceOperationAccountingError),
    /// Caller-selected readout width does not match recurrent state width.
    ReadoutStateWidthMismatch {
        /// Adapter arm being constructed.
        arm: ReferenceArm,
        /// Recurrent state width.
        recurrent: u64,
        /// Primary readout state width.
        primary_readout: u64,
        /// Optional second readout state width used by A3.
        secondary_readout: Option<u64>,
    },
    /// A neutral non-query A2 read unexpectedly hit resident memory.
    UnexpectedNeutralReadHit {
        /// A2 or A3 adapter.
        arm: ReferenceArm,
        /// Physical associative address that unexpectedly hit.
        address: u64,
    },
    /// A0 chronological payload position could not advance.
    PayloadPositionOverflow,
    /// One runtime diagnostic counter could not advance.
    DiagnosticCounterOverflow {
        /// Adapter whose diagnostics overflowed.
        arm: ReferenceArm,
    },
}

impl fmt::Display for AssrTaskAdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::A0(error) => write!(formatter, "A0 reference: {error}"),
            Self::Encoding(error) => write!(formatter, "task encoding: {error}"),
            Self::Recurrent(error) => write!(formatter, "recurrent reference: {error}"),
            Self::A3(error) => write!(formatter, "A3 reference: {error}"),
            Self::Readout(error) => write!(formatter, "exact readout: {error}"),
            Self::OperationAccounting(error) => {
                write!(formatter, "semantic operation accounting: {error}")
            }
            Self::ReadoutStateWidthMismatch {
                arm,
                recurrent,
                primary_readout,
                secondary_readout,
            } => write!(
                formatter,
                "{arm:?} recurrent state width {recurrent} does not match readout width(s) {primary_readout}/{secondary_readout:?}"
            ),
            Self::UnexpectedNeutralReadHit { arm, address } => write!(
                formatter,
                "{arm:?} neutral non-query A2 read unexpectedly hit resident memory at address {address}"
            ),
            Self::PayloadPositionOverflow => {
                formatter.write_str("A0 payload position counter overflow")
            }
            Self::DiagnosticCounterOverflow { arm } => {
                write!(formatter, "{arm:?} adapter diagnostic counter overflow")
            }
        }
    }
}

impl std::error::Error for AssrTaskAdapterError {}

impl From<A0ReferenceError> for AssrTaskAdapterError {
    fn from(error: A0ReferenceError) -> Self {
        Self::A0(error)
    }
}

impl From<TaskEncodingError> for AssrTaskAdapterError {
    fn from(error: TaskEncodingError) -> Self {
        Self::Encoding(error)
    }
}

impl From<RecurrentReferenceError> for AssrTaskAdapterError {
    fn from(error: RecurrentReferenceError) -> Self {
        Self::Recurrent(error)
    }
}

impl From<A3ReferenceError> for AssrTaskAdapterError {
    fn from(error: A3ReferenceError) -> Self {
        Self::A3(error)
    }
}

impl From<TaskReadoutError> for AssrTaskAdapterError {
    fn from(error: TaskReadoutError) -> Self {
        Self::Readout(error)
    }
}

impl From<ReferenceOperationAccountingError> for AssrTaskAdapterError {
    fn from(error: ReferenceOperationAccountingError) -> Self {
        Self::OperationAccounting(error)
    }
}

fn checked_operations(
    current: ReferenceOperationAccounting,
    delta: ReferenceOperationAccounting,
) -> Result<ReferenceOperationAccounting, AssrTaskAdapterError> {
    current.checked_add(delta).map_err(Into::into)
}

/// Reusable A0 competent full-history control adapter.
#[derive(Debug)]
pub struct A0TaskAdapter {
    reference: A0Reference,
    next_payload_position: u64,
    operations: ReferenceOperationAccounting,
}

impl A0TaskAdapter {
    /// Construct the exact namespaced A0 task adapter.
    pub fn new() -> Result<Self, AssrTaskAdapterError> {
        Ok(Self {
            reference: A0Reference::new(FullHistoryLayout::new(
                A0_TASK_KEY_WIDTH as u64,
                A0_TASK_VALUE_WIDTH as u64,
            )?)?,
            next_payload_position: 0,
            operations: ReferenceOperationAccounting::zero(),
        })
    }

    /// Exact cumulative reference-semantic work since reset.
    #[must_use]
    pub const fn operations(&self) -> ReferenceOperationAccounting {
        self.operations
    }

    fn append_item(
        &mut self,
        key: &[f64],
        value: &[f64],
    ) -> Result<(), AssrTaskAdapterError> {
        let next_operations = checked_operations(
            self.operations,
            ReferenceOperationAccounting::a0_append(self.reference.layout())?,
        )?;
        self.reference.append(key, value)?;
        self.operations = next_operations;
        Ok(())
    }

    fn decode_readout(&mut self, query: &[f64]) -> Result<TaskPrediction, AssrTaskAdapterError> {
        let item_count = self.reference.item_count();
        let next_operations = checked_operations(
            self.operations,
            ReferenceOperationAccounting::a0_read(self.reference.layout(), item_count)?,
        )?;
        let readout = self.reference.read(query)?;
        self.operations = next_operations;
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

impl SymbolicTaskAdapter for A0TaskAdapter {
    type Error = AssrTaskAdapterError;

    fn arm(&self) -> ReferenceArm {
        ReferenceArm::A0
    }

    fn reset(&mut self) -> Result<(), Self::Error> {
        self.reference.clear();
        self.next_payload_position = 0;
        self.operations = ReferenceOperationAccounting::zero();
        Ok(())
    }

    fn associate(&mut self, key_code: u64, value: TaskSymbol) -> Result<(), Self::Error> {
        let item = a0_association_item(key_code, value);
        self.append_item(&item.key(), &item.value())
    }

    fn payload(&mut self, value: TaskSymbol) -> Result<(), Self::Error> {
        let position = self.next_payload_position;
        let next_position = position
            .checked_add(1)
            .ok_or(AssrTaskAdapterError::PayloadPositionOverflow)?;
        let item = a0_payload_item(position, value);
        self.append_item(&item.key(), &item.value())?;
        self.next_payload_position = next_position;
        Ok(())
    }

    fn distractor(&mut self, token: TaskSymbol) -> Result<(), Self::Error> {
        let item = a0_distractor_item(token);
        self.append_item(&item.key(), &item.value())
    }

    fn query_association(&mut self, key_code: u64) -> Result<TaskPrediction, Self::Error> {
        self.decode_readout(&a0_association_query_key(key_code))
    }

    fn query_payload(&mut self, position: u64) -> Result<TaskPrediction, Self::Error> {
        self.decode_readout(&a0_payload_query_key(position))
    }
}

/// Reusable A1 bounded recurrent-only symbolic adapter.
#[derive(Debug)]
pub struct A1TaskAdapter {
    reference: A1Reference,
    recurrent_layout: RecurrentLayout,
    encoder: LosslessTaskEncoder,
    readout: ExactStateSymbolReadout,
    operations: ReferenceOperationAccounting,
}

impl A1TaskAdapter {
    /// Construct A1 from explicit recurrent parameters and explicit exact readout.
    pub fn new(
        parameters: RecurrentParameters,
        readout: ExactStateSymbolReadout,
    ) -> Result<Self, AssrTaskAdapterError> {
        let recurrent_layout = parameters.layout();
        let readout_layout = readout.layout();
        if readout_layout.state_width() != recurrent_layout.state_width() {
            return Err(AssrTaskAdapterError::ReadoutStateWidthMismatch {
                arm: ReferenceArm::A1,
                recurrent: recurrent_layout.state_width(),
                primary_readout: readout_layout.state_width(),
                secondary_readout: None,
            });
        }
        Ok(Self {
            reference: A1Reference::new(parameters)?,
            recurrent_layout,
            encoder: LosslessTaskEncoder::new(TaskInputLayout::new(
                recurrent_layout.input_width(),
            )?),
            readout,
            operations: ReferenceOperationAccounting::zero(),
        })
    }

    /// Exact cumulative reference-semantic work since reset.
    #[must_use]
    pub const fn operations(&self) -> ReferenceOperationAccounting {
        self.operations
    }

    fn step(&mut self, input: Vec<f64>) -> Result<(), AssrTaskAdapterError> {
        let next_operations = checked_operations(
            self.operations,
            ReferenceOperationAccounting::a1_step(self.recurrent_layout)?,
        )?;
        self.reference.step(&input)?;
        self.operations = next_operations;
        Ok(())
    }

    fn query(&mut self, input: Vec<f64>) -> Result<TaskPrediction, AssrTaskAdapterError> {
        self.step(input)?;
        Ok(match self.readout.decode_state(self.reference.state())? {
            ExactStatePrediction::Symbol(symbol) => TaskPrediction::Symbol(symbol),
            ExactStatePrediction::InvalidEncoding => TaskPrediction::Invalid,
        })
    }
}

impl SymbolicTaskAdapter for A1TaskAdapter {
    type Error = AssrTaskAdapterError;

    fn arm(&self) -> ReferenceArm {
        ReferenceArm::A1
    }

    fn reset(&mut self) -> Result<(), Self::Error> {
        self.reference.reset();
        self.operations = ReferenceOperationAccounting::zero();
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

/// Runtime A2 associative diagnostics, kept separate from task quality.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct A2TaskDiagnostics {
    query_hits: u64,
    query_collision_misses: u64,
    query_empty: u64,
    inserted_writes: u64,
    updated_writes: u64,
    replacement_writes: u64,
}

impl A2TaskDiagnostics {
    /// Exact query hits observed from actual A2 reports.
    #[must_use]
    pub const fn query_hits(self) -> u64 {
        self.query_hits
    }

    /// Exact query collision misses observed from actual A2 reports.
    #[must_use]
    pub const fn query_collision_misses(self) -> u64 {
        self.query_collision_misses
    }

    /// Exact empty query reads observed from actual A2 reports.
    #[must_use]
    pub const fn query_empty(self) -> u64 {
        self.query_empty
    }

    /// Exact inserted writes.
    #[must_use]
    pub const fn inserted_writes(self) -> u64 {
        self.inserted_writes
    }

    /// Exact in-place updates.
    #[must_use]
    pub const fn updated_writes(self) -> u64 {
        self.updated_writes
    }

    /// Exact collision replacements.
    #[must_use]
    pub const fn replacement_writes(self) -> u64 {
        self.replacement_writes
    }

    fn increment(value: &mut u64, arm: ReferenceArm) -> Result<(), AssrTaskAdapterError> {
        *value = value
            .checked_add(1)
            .ok_or(AssrTaskAdapterError::DiagnosticCounterOverflow { arm })?;
        Ok(())
    }

    fn observe_query(
        &mut self,
        arm: ReferenceArm,
        status: A2ReadStatus,
    ) -> Result<(), AssrTaskAdapterError> {
        match status {
            A2ReadStatus::Hit { .. } => Self::increment(&mut self.query_hits, arm),
            A2ReadStatus::CollisionMiss { .. } => {
                Self::increment(&mut self.query_collision_misses, arm)
            }
            A2ReadStatus::Empty { .. } => Self::increment(&mut self.query_empty, arm),
        }
    }

    fn observe_write(
        &mut self,
        arm: ReferenceArm,
        outcome: Option<AssociativeWriteOutcome>,
    ) -> Result<(), AssrTaskAdapterError> {
        match outcome {
            Some(AssociativeWriteOutcome::Inserted { .. }) => {
                Self::increment(&mut self.inserted_writes, arm)
            }
            Some(AssociativeWriteOutcome::Updated { .. }) => {
                Self::increment(&mut self.updated_writes, arm)
            }
            Some(AssociativeWriteOutcome::ReplacedCollision { .. }) => {
                Self::increment(&mut self.replacement_writes, arm)
            }
            None => Ok(()),
        }
    }
}

/// Reusable A2 recurrent + direct-mapped associative symbolic adapter.
#[derive(Debug)]
pub struct A2TaskAdapter {
    reference: A2Reference,
    recurrent_layout: RecurrentLayout,
    encoder: LosslessTaskEncoder,
    readout: ExactStateSymbolReadout,
    payload_keys: PayloadKeyCursor,
    neutral_read_key: u64,
    diagnostics: A2TaskDiagnostics,
    operations: ReferenceOperationAccounting,
}

impl A2TaskAdapter {
    /// Construct A2 from explicit caller-selected reference parameters.
    pub fn new(
        parameters: RecurrentParameters,
        memory_layout: AssociativeMemoryLayout,
        projection_seed: u64,
        fusion_gain: f64,
        readout: ExactStateSymbolReadout,
        neutral_read_key: u64,
    ) -> Result<Self, AssrTaskAdapterError> {
        let recurrent_layout = parameters.layout();
        let readout_layout = readout.layout();
        if recurrent_layout.state_width() != readout_layout.state_width() {
            return Err(AssrTaskAdapterError::ReadoutStateWidthMismatch {
                arm: ReferenceArm::A2,
                recurrent: recurrent_layout.state_width(),
                primary_readout: readout_layout.state_width(),
                secondary_readout: None,
            });
        }
        Ok(Self {
            reference: A2Reference::new(parameters, memory_layout, projection_seed, fusion_gain)?,
            recurrent_layout,
            encoder: LosslessTaskEncoder::new(TaskInputLayout::new(
                recurrent_layout.input_width(),
            )?),
            readout,
            payload_keys: PayloadKeyCursor::default(),
            neutral_read_key,
            diagnostics: A2TaskDiagnostics::default(),
            operations: ReferenceOperationAccounting::zero(),
        })
    }

    /// Runtime associative diagnostics since reset.
    #[must_use]
    pub const fn diagnostics(&self) -> A2TaskDiagnostics {
        self.diagnostics
    }

    /// Exact cumulative reference-semantic work since reset.
    #[must_use]
    pub const fn operations(&self) -> ReferenceOperationAccounting {
        self.operations
    }

    fn record_report(&mut self, report: A2StepReport) -> Result<(), AssrTaskAdapterError> {
        self.operations = checked_operations(
            self.operations,
            ReferenceOperationAccounting::a2_step(self.recurrent_layout, report)?,
        )?;
        Ok(())
    }

    fn non_query_step(
        &mut self,
        input: Vec<f64>,
        write_key: Option<u64>,
    ) -> Result<(), AssrTaskAdapterError> {
        let report = self
            .reference
            .step(&input, self.neutral_read_key, write_key)?;
        self.record_report(report)?;
        if let A2ReadStatus::Hit { address } = report.read() {
            return Err(AssrTaskAdapterError::UnexpectedNeutralReadHit {
                arm: ReferenceArm::A2,
                address,
            });
        }
        self.diagnostics
            .observe_write(ReferenceArm::A2, report.write())?;
        Ok(())
    }

    fn query_step(
        &mut self,
        input: Vec<f64>,
        read_key: u64,
    ) -> Result<TaskPrediction, AssrTaskAdapterError> {
        let report = self.reference.step(&input, read_key, None)?;
        self.record_report(report)?;
        self.diagnostics
            .observe_query(ReferenceArm::A2, report.read())?;
        Ok(match self.readout.decode_state(self.reference.state())? {
            ExactStatePrediction::Symbol(symbol) => TaskPrediction::Symbol(symbol),
            ExactStatePrediction::InvalidEncoding => TaskPrediction::Invalid,
        })
    }
}

impl SymbolicTaskAdapter for A2TaskAdapter {
    type Error = AssrTaskAdapterError;

    fn arm(&self) -> ReferenceArm {
        ReferenceArm::A2
    }

    fn reset(&mut self) -> Result<(), Self::Error> {
        self.reference.reset();
        self.payload_keys.reset();
        self.diagnostics = A2TaskDiagnostics::default();
        self.operations = ReferenceOperationAccounting::zero();
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

/// Runtime A3 associative + VSA diagnostics, kept separate from task quality.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct A3TaskDiagnostics {
    associative: A2TaskDiagnostics,
    vsa_stores: u64,
    vsa_queries: u64,
}

impl A3TaskDiagnostics {
    /// A2-side associative diagnostics observed inside A3.
    #[must_use]
    pub const fn associative(self) -> A2TaskDiagnostics {
        self.associative
    }

    /// Successful atomic A2+VSA stores.
    #[must_use]
    pub const fn vsa_stores(self) -> u64 {
        self.vsa_stores
    }

    /// Keyed VSA query reads.
    #[must_use]
    pub const fn vsa_queries(self) -> u64 {
        self.vsa_queries
    }
}

/// Reusable A3 recurrent + associative + VSA symbolic adapter.
#[derive(Debug)]
pub struct A3TaskAdapter {
    reference: A3Reference,
    recurrent_layout: RecurrentLayout,
    encoder: LosslessTaskEncoder,
    association_readout: ExactStateSymbolReadout,
    payload_readout: ExactStateSymbolReadout,
    payload_keys: PayloadKeyCursor,
    neutral_read_key: u64,
    diagnostics: A3TaskDiagnostics,
    operations: ReferenceOperationAccounting,
}

impl A3TaskAdapter {
    /// Construct A3 from explicit caller-selected reference parameters.
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
    ) -> Result<Self, AssrTaskAdapterError> {
        let recurrent_layout = parameters.layout();
        let association_layout = association_readout.layout();
        let payload_layout = payload_readout.layout();
        if recurrent_layout.state_width() != association_layout.state_width()
            || recurrent_layout.state_width() != payload_layout.state_width()
        {
            return Err(AssrTaskAdapterError::ReadoutStateWidthMismatch {
                arm: ReferenceArm::A3,
                recurrent: recurrent_layout.state_width(),
                primary_readout: association_layout.state_width(),
                secondary_readout: Some(payload_layout.state_width()),
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
            recurrent_layout,
            encoder: LosslessTaskEncoder::new(TaskInputLayout::new(input_width)?),
            association_readout,
            payload_readout,
            payload_keys: PayloadKeyCursor::default(),
            neutral_read_key,
            diagnostics: A3TaskDiagnostics::default(),
            operations: ReferenceOperationAccounting::zero(),
        })
    }

    /// Runtime associative + VSA diagnostics since reset.
    #[must_use]
    pub const fn diagnostics(&self) -> A3TaskDiagnostics {
        self.diagnostics
    }

    /// Exact cumulative reference-semantic work since reset.
    #[must_use]
    pub const fn operations(&self) -> ReferenceOperationAccounting {
        self.operations
    }

    fn atomic_store_step(
        &mut self,
        input: Vec<f64>,
        logical_key: u64,
    ) -> Result<(), AssrTaskAdapterError> {
        let report = self.reference.step_skip_vsa_and_store(
            &input,
            self.neutral_read_key,
            Some(logical_key),
            logical_key,
            &input,
        )?;
        self.operations = checked_operations(
            self.operations,
            ReferenceOperationAccounting::a3_skip_and_store_step(self.recurrent_layout, report)?,
        )?;
        if let A2ReadStatus::Hit { address } = report.read() {
            return Err(AssrTaskAdapterError::UnexpectedNeutralReadHit {
                arm: ReferenceArm::A3,
                address,
            });
        }
        self.diagnostics
            .associative
            .observe_write(ReferenceArm::A3, report.write())?;
        A2TaskDiagnostics::increment(&mut self.diagnostics.vsa_stores, ReferenceArm::A3)?;
        Ok(())
    }

    fn distractor_step(&mut self, input: Vec<f64>) -> Result<(), AssrTaskAdapterError> {
        let report = self.reference.step_routed(
            &input,
            A3VsaReadRoute::Skip,
            self.neutral_read_key,
            None,
        )?;
        self.operations = checked_operations(
            self.operations,
            ReferenceOperationAccounting::a3_routed_step(
                self.recurrent_layout,
                A3VsaReadRoute::Skip,
                report,
            )?,
        )?;
        if let A2ReadStatus::Hit { address } = report.read() {
            return Err(AssrTaskAdapterError::UnexpectedNeutralReadHit {
                arm: ReferenceArm::A3,
                address,
            });
        }
        Ok(())
    }

    fn query_step(
        &mut self,
        input: Vec<f64>,
        read_key: u64,
        readout: ExactStateSymbolReadout,
    ) -> Result<TaskPrediction, AssrTaskAdapterError> {
        let route = A3VsaReadRoute::Key(read_key);
        let report = self
            .reference
            .step_routed(&input, route, read_key, None)?;
        self.operations = checked_operations(
            self.operations,
            ReferenceOperationAccounting::a3_routed_step(self.recurrent_layout, route, report)?,
        )?;
        self.diagnostics
            .associative
            .observe_query(ReferenceArm::A3, report.read())?;
        A2TaskDiagnostics::increment(&mut self.diagnostics.vsa_queries, ReferenceArm::A3)?;
        Ok(match readout.decode_state(self.reference.state())? {
            ExactStatePrediction::Symbol(symbol) => TaskPrediction::Symbol(symbol),
            ExactStatePrediction::InvalidEncoding => TaskPrediction::Invalid,
        })
    }
}

impl SymbolicTaskAdapter for A3TaskAdapter {
    type Error = AssrTaskAdapterError;

    fn arm(&self) -> ReferenceArm {
        ReferenceArm::A3
    }

    fn reset(&mut self) -> Result<(), Self::Error> {
        self.reference.reset();
        self.payload_keys.reset();
        self.diagnostics = A3TaskDiagnostics::default();
        self.operations = ReferenceOperationAccounting::zero();
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

#[cfg(test)]
mod tests {
    use super::{A0TaskAdapter, A1TaskAdapter, A2TaskAdapter, A3TaskAdapter};
    use crate::associative_memory::AssociativeMemoryLayout;
    use crate::assr_reference::{RecurrentLayout, RecurrentParameters};
    use crate::task_encoding::MIN_TASK_INPUT_WIDTH;
    use crate::task_execution::{SymbolicTaskAdapter, TaskPrediction, execute_symbolic_task};
    use crate::task_generators::{T1Config, TaskSymbol, generate_t1};
    use crate::task_readout::{ExactStateReadoutLayout, ExactStateSymbolReadout};

    fn exact_readout(state_width: u64, high: u64, low: u64) -> ExactStateSymbolReadout {
        ExactStateSymbolReadout::new(
            ExactStateReadoutLayout::new(state_width, high, low).expect("readout layout"),
        )
    }

    fn key_echo_parameters() -> RecurrentParameters {
        let input_width = MIN_TASK_INPUT_WIDTH as usize;
        let state_width = 2usize;
        let layout = RecurrentLayout::new(MIN_TASK_INPUT_WIDTH, state_width as u64).expect("layout");
        let mut input_to_state = vec![0.0; input_width * state_width];
        input_to_state[1] = 1.0;
        input_to_state[input_width + 2] = 1.0;
        RecurrentParameters::new(
            layout,
            input_to_state,
            vec![0.0; state_width * state_width],
            vec![0.0; state_width],
        )
        .expect("parameters")
    }

    fn t1_value_capture_parameters() -> RecurrentParameters {
        let input_width = MIN_TASK_INPUT_WIDTH as usize;
        let state_width = 2usize;
        let layout = RecurrentLayout::new(MIN_TASK_INPUT_WIDTH, state_width as u64).expect("layout");
        let mut input_to_state = vec![0.0; input_width * state_width];
        input_to_state[3] = 1.0;
        input_to_state[input_width + 4] = 1.0;
        RecurrentParameters::new(
            layout,
            input_to_state,
            vec![0.0; state_width * state_width],
            vec![0.0; state_width],
        )
        .expect("parameters")
    }

    #[test]
    fn a0_and_a1_accumulate_only_their_declared_semantic_work() {
        let instance = generate_t1(17, T1Config::new(2, 1, 1).expect("T1")).expect("instance");
        let mut a0 = A0TaskAdapter::new().expect("A0");
        let a0_record = execute_symbolic_task(&instance, &mut a0).expect("A0 execution");
        assert!(a0_record.all_queries_exact());
        assert!(a0.operations().history_scalar_stores() > 0);
        assert!(a0.operations().history_distance_terms() > 0);
        assert_eq!(a0.operations().recurrent_mac_terms(), 0);

        let mut a1 = A1TaskAdapter::new(key_echo_parameters(), exact_readout(2, 0, 1)).expect("A1");
        let _ = execute_symbolic_task(&instance, &mut a1).expect("A1 execution");
        assert!(a1.operations().recurrent_mac_terms() > 0);
        assert_eq!(a1.operations().associative_address_projections(), 0);
        assert_eq!(a1.operations().vsa_bind_terms(), 0);
    }

    #[test]
    fn a2_accounting_comes_from_observed_reports() {
        let mut adapter = A2TaskAdapter::new(
            t1_value_capture_parameters(),
            AssociativeMemoryLayout::new(64, 2).expect("memory"),
            11,
            1.0,
            exact_readout(2, 0, 1),
            u64::MAX,
        )
        .expect("A2");
        adapter
            .associate(7, TaskSymbol::new(9))
            .expect("association");
        let _ = adapter.query_association(7).expect("query");
        assert_eq!(adapter.diagnostics().query_hits(), 1);
        assert!(adapter.operations().associative_address_projections() >= 3);
        assert!(adapter.operations().associative_payload_fusions() > 0);
        assert_eq!(adapter.operations().vsa_bind_terms(), 0);
    }

    #[test]
    fn a3_distractor_and_query_do_not_mutate_vsa_workspace() {
        let input_width = MIN_TASK_INPUT_WIDTH as usize;
        let state_width = 4usize;
        let layout = RecurrentLayout::new(MIN_TASK_INPUT_WIDTH, 4).expect("layout");
        let mut input_to_state = vec![0.0; input_width * state_width];
        input_to_state[3] = 0.5;
        input_to_state[input_width + 4] = 0.5;
        input_to_state[2 * input_width + 1] = 0.5;
        input_to_state[3 * input_width + 2] = 0.5;
        let parameters = RecurrentParameters::new(
            layout,
            input_to_state,
            vec![0.0; state_width * state_width],
            vec![0.0; state_width],
        )
        .expect("parameters");
        let mut adapter = A3TaskAdapter::new(
            parameters,
            AssociativeMemoryLayout::new(64, 4).expect("memory"),
            11,
            1.0,
            23,
            1.0,
            exact_readout(4, 0, 1),
            exact_readout(4, 2, 3),
            u64::MAX,
        )
        .expect("A3");
        adapter
            .associate(7, TaskSymbol::new(0x1234_5678_9abc_def0))
            .expect("store");
        let stored = adapter.reference.workspace().components().to_vec();
        adapter
            .distractor(TaskSymbol::new(0x55aa))
            .expect("distractor");
        assert_eq!(adapter.reference.workspace().components(), stored.as_slice());
        let prediction = adapter.query_association(7).expect("query");
        assert!(matches!(prediction, TaskPrediction::Symbol(_)));
        assert_eq!(adapter.reference.workspace().components(), stored.as_slice());
        assert_eq!(adapter.diagnostics().vsa_stores(), 1);
        assert_eq!(adapter.diagnostics().vsa_queries(), 1);
        assert!(adapter.operations().vsa_bind_terms() > 0);
        assert!(adapter.operations().vsa_unbind_terms() > 0);
    }
}