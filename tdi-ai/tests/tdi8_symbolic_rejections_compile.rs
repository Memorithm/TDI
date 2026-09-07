#[path = "../src/task_rejections.rs"]
mod task_rejections;

pub use tdi_ai::{ReferenceArm, task_execution, task_generators};

use core::fmt;

use task_rejections::{
    RecordedSymbolicTaskOutcome, SymbolicRejectionCode, execute_symbolic_task_recorded,
};
use tdi_ai::task_execution::{SymbolicTaskAdapter, TaskExecutionError, TaskPrediction};
use tdi_ai::task_generators::{T1Config, TaskFamily, TaskSymbol, generate_t1};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FakeAdapterError {
    Reset,
    Event,
}

impl fmt::Display for FakeAdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reset => formatter.write_str("fake reset failure"),
            Self::Event => formatter.write_str("fake event failure"),
        }
    }
}

impl std::error::Error for FakeAdapterError {}

struct FakeAdapter {
    arm: ReferenceArm,
    fail_reset: bool,
    fail_event: bool,
    invalid_queries: bool,
}

impl FakeAdapter {
    const fn invalid(arm: ReferenceArm) -> Self {
        Self {
            arm,
            fail_reset: false,
            fail_event: false,
            invalid_queries: true,
        }
    }

    const fn event_failure(arm: ReferenceArm) -> Self {
        Self {
            arm,
            fail_reset: false,
            fail_event: true,
            invalid_queries: false,
        }
    }

    const fn reset_failure(arm: ReferenceArm) -> Self {
        Self {
            arm,
            fail_reset: true,
            fail_event: false,
            invalid_queries: false,
        }
    }

    fn prediction(&self) -> TaskPrediction {
        if self.invalid_queries {
            TaskPrediction::Invalid
        } else {
            TaskPrediction::Symbol(TaskSymbol::new(0))
        }
    }
}

impl SymbolicTaskAdapter for FakeAdapter {
    type Error = FakeAdapterError;

    fn arm(&self) -> ReferenceArm {
        self.arm
    }

    fn reset(&mut self) -> Result<(), Self::Error> {
        if self.fail_reset {
            Err(FakeAdapterError::Reset)
        } else {
            Ok(())
        }
    }

    fn associate(&mut self, _key_code: u64, _value: TaskSymbol) -> Result<(), Self::Error> {
        if self.fail_event {
            Err(FakeAdapterError::Event)
        } else {
            Ok(())
        }
    }

    fn payload(&mut self, _value: TaskSymbol) -> Result<(), Self::Error> {
        Ok(())
    }

    fn distractor(&mut self, _token: TaskSymbol) -> Result<(), Self::Error> {
        Ok(())
    }

    fn query_association(&mut self, _key_code: u64) -> Result<TaskPrediction, Self::Error> {
        Ok(self.prediction())
    }

    fn query_payload(&mut self, _position: u64) -> Result<TaskPrediction, Self::Error> {
        Ok(self.prediction())
    }
}

fn numeric(error: TaskExecutionError<FakeAdapterError>) -> u16 {
    SymbolicRejectionCode::from_error(&error).numeric()
}

#[test]
fn rejection_numeric_codes_are_exact_and_stable() {
    assert_eq!(
        numeric(TaskExecutionError::QueryCountTooLarge { query_count: 9 }),
        0x0101
    );
    assert_eq!(
        numeric(TaskExecutionError::QueryRecordAllocationFailed { records: 9 }),
        0x0102
    );
    assert_eq!(
        numeric(TaskExecutionError::AdapterReset(FakeAdapterError::Reset)),
        0x0201
    );
    assert_eq!(
        numeric(TaskExecutionError::AdapterEvent {
            event_index: 3,
            error: FakeAdapterError::Event,
        }),
        0x0202
    );
    assert_eq!(
        numeric(TaskExecutionError::ArmChanged {
            event_index: Some(3),
            expected: ReferenceArm::A1,
            observed: ReferenceArm::A2,
        }),
        0x0301
    );
    assert_eq!(
        numeric(TaskExecutionError::QueryCountMismatch {
            declared: 2,
            observed: 1,
        }),
        0x0302
    );
}

#[test]
fn invalid_prediction_remains_completed_quality_failure() {
    let instance = generate_t1(17, T1Config::new(2, 1, 1).expect("T1 config")).expect("T1");
    let mut adapter = FakeAdapter::invalid(ReferenceArm::A1);

    match execute_symbolic_task_recorded(&instance, &mut adapter) {
        RecordedSymbolicTaskOutcome::Completed(record) => {
            assert_eq!(record.arm(), ReferenceArm::A1);
            assert_eq!(record.family(), TaskFamily::AssociativeRecall);
            assert_eq!(record.generator_seed(), 17);
            assert_eq!(record.queries().len(), 1);
            assert_eq!(record.invalid_predictions(), 1);
            assert_eq!(record.failed_queries(), 1);
            assert_eq!(record.successful_queries(), 0);
        }
        RecordedSymbolicTaskOutcome::Rejected(_) => {
            panic!("invalid symbolic prediction must not become technical rejection")
        }
    }
}

#[test]
fn adapter_event_rejection_retains_typed_error_and_provenance() {
    let instance = generate_t1(29, T1Config::new(2, 1, 1).expect("T1 config")).expect("T1");
    let mut adapter = FakeAdapter::event_failure(ReferenceArm::A2);

    match execute_symbolic_task_recorded(&instance, &mut adapter) {
        RecordedSymbolicTaskOutcome::Completed(_) => panic!("event failure must reject"),
        RecordedSymbolicTaskOutcome::Rejected(rejection) => {
            assert_eq!(rejection.arm(), ReferenceArm::A2);
            assert_eq!(rejection.family(), TaskFamily::AssociativeRecall);
            assert_eq!(rejection.generator_seed(), 29);
            assert_eq!(rejection.code(), SymbolicRejectionCode::AdapterEvent);
            assert_eq!(rejection.code().numeric(), 0x0202);
            assert!(matches!(
                rejection.error(),
                TaskExecutionError::AdapterEvent {
                    event_index: 0,
                    error: FakeAdapterError::Event,
                }
            ));
        }
    }
}

#[test]
fn reset_failure_is_rejection_not_quality_record() {
    let instance = generate_t1(31, T1Config::new(2, 1, 1).expect("T1 config")).expect("T1");
    let mut adapter = FakeAdapter::reset_failure(ReferenceArm::A3);

    match execute_symbolic_task_recorded(&instance, &mut adapter) {
        RecordedSymbolicTaskOutcome::Completed(_) => panic!("reset failure must reject"),
        RecordedSymbolicTaskOutcome::Rejected(rejection) => {
            assert_eq!(rejection.arm(), ReferenceArm::A3);
            assert_eq!(rejection.code(), SymbolicRejectionCode::AdapterReset);
            assert!(matches!(
                rejection.error(),
                TaskExecutionError::AdapterReset(FakeAdapterError::Reset)
            ));
        }
    }
}
