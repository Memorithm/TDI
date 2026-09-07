//! Machine-readable rejection provenance for bounded TDI-8.1 symbolic execution.
//!
//! This qualification-local layer wraps the already-qualified symbolic executor
//! without changing its compatibility API. Technical execution rejection stays
//! structurally separate from completed task quality: in particular,
//! `TaskPrediction::Invalid` remains a completed, evaluated query failure.

use crate::ReferenceArm;
use crate::task_execution::{
    SymbolicTaskAdapter, TaskExecutionError, TaskExecutionRecord, execute_symbolic_task,
};
use crate::task_generators::{TaskFamily, TaskInstance};

/// Stable non-final machine code for every currently represented symbolic
/// executor rejection path.
///
/// Numeric values are explicit and non-renumberable. Future reasons must use
/// previously unused values rather than changing an existing code.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum SymbolicRejectionCode {
    /// The generated query count does not fit the host index type.
    QueryCountTooLarge = 0x0101,
    /// Exact reservation for query records was refused by the host allocator.
    QueryRecordAllocationFailed = 0x0102,

    /// Adapter reset failed before symbolic event execution.
    AdapterReset = 0x0201,
    /// One adapter event failed technically.
    AdapterEvent = 0x0202,

    /// Adapter identity changed during one task execution.
    ArmChanged = 0x0301,
    /// Observed completed query count disagreed with the generated declaration.
    QueryCountMismatch = 0x0302,
}

impl SymbolicRejectionCode {
    /// Exact stable numeric representation for machine-readable records.
    #[must_use]
    pub const fn numeric(self) -> u16 {
        self as u16
    }

    /// Losslessly classify the executor error while the record retains the
    /// original typed error and its adapter-specific payload.
    #[must_use]
    pub const fn from_error<E>(error: &TaskExecutionError<E>) -> Self {
        match error {
            TaskExecutionError::QueryCountTooLarge { .. } => Self::QueryCountTooLarge,
            TaskExecutionError::QueryRecordAllocationFailed { .. } => {
                Self::QueryRecordAllocationFailed
            }
            TaskExecutionError::AdapterReset(_) => Self::AdapterReset,
            TaskExecutionError::AdapterEvent { .. } => Self::AdapterEvent,
            TaskExecutionError::ArmChanged { .. } => Self::ArmChanged,
            TaskExecutionError::QueryCountMismatch { .. } => Self::QueryCountMismatch,
        }
    }
}

/// Immutable evaluator-owned provenance for one technically rejected symbolic
/// trajectory.
///
/// There is deliberately no quality/success field because normal task execution
/// did not complete. The original typed error is retained alongside its stable
/// categorical code.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolicTaskRejectionRecord<E> {
    arm: ReferenceArm,
    family: TaskFamily,
    generator_seed: u64,
    code: SymbolicRejectionCode,
    error: TaskExecutionError<E>,
}

impl<E> SymbolicTaskRejectionRecord<E> {
    /// Arm captured before task reset/execution.
    #[must_use]
    pub const fn arm(&self) -> ReferenceArm {
        self.arm
    }

    /// Generator-owned task family.
    #[must_use]
    pub const fn family(&self) -> TaskFamily {
        self.family
    }

    /// Exact generator seed; no target/oracle value is copied into provenance.
    #[must_use]
    pub const fn generator_seed(&self) -> u64 {
        self.generator_seed
    }

    /// Stable machine-readable rejection category.
    #[must_use]
    pub const fn code(&self) -> SymbolicRejectionCode {
        self.code
    }

    /// Original typed executor error, including the adapter-specific error.
    #[must_use]
    pub const fn error(&self) -> &TaskExecutionError<E> {
        &self.error
    }
}

/// Non-final bounded TDI-8.1 symbolic execution outcome.
///
/// Completed records retain ordinary incorrect/invalid predictions. Only
/// technical executor failures enter `Rejected`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecordedSymbolicTaskOutcome<E> {
    /// Normal architecture-neutral task execution completed.
    Completed(TaskExecutionRecord),
    /// Technical execution could not produce a complete evaluable record.
    Rejected(SymbolicTaskRejectionRecord<E>),
}

/// Execute one immutable symbolic task and retain machine-readable technical
/// rejection provenance without changing [`execute_symbolic_task`].
#[must_use]
pub fn execute_symbolic_task_recorded<A>(
    instance: &TaskInstance,
    adapter: &mut A,
) -> RecordedSymbolicTaskOutcome<A::Error>
where
    A: SymbolicTaskAdapter,
{
    let arm = adapter.arm();
    let family = instance.family();
    let generator_seed = instance.seed();

    match execute_symbolic_task(instance, adapter) {
        Ok(record) => RecordedSymbolicTaskOutcome::Completed(record),
        Err(error) => RecordedSymbolicTaskOutcome::Rejected(SymbolicTaskRejectionRecord {
            arm,
            family,
            generator_seed,
            code: SymbolicRejectionCode::from_error(&error),
            error,
        }),
    }
}
