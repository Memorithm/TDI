//! Leakage-safe symbolic task execution contract for bounded TDI-8.1.
//!
//! The TDI-8.0 task generators freeze architecture-independent symbolic events
//! before arm-specific binary64 encoding is selected. This module provides the
//! next boundary: one generated [`TaskInstance`] is executed in exact event
//! order through an arm adapter while evaluation-only target and provenance
//! metadata remain owned by the runner.
//!
//! In particular, an adapter never receives a query target, an association
//! source index, or a generator-side collision-class label. It receives only the
//! symbolic stimulus required by the task. This prevents an arm adapter from
//! obtaining exact-answer or interference annotations through the evaluator API.
//!
//! No vector encoding, architecture dimension, memory budget, horizon, deficit
//! function, interval method or TDI-8.2 surface is selected here.

use core::fmt;

use crate::ReferenceArm;
use crate::task_generators::{TaskEvent, TaskFamily, TaskInstance, TaskKey, TaskSymbol};

/// Arm-side symbolic interface consumed by the architecture-neutral executor.
///
/// The method signatures intentionally exclude evaluation-only fields present in
/// [`TaskEvent`]. Concrete A0/A1/A2/A3 adapters may later encode the exposed
/// symbolic stimuli into binary64 vectors only after that encoding policy is
/// reviewed and frozen on non-final TDI-8.1 data.
pub trait SymbolicTaskAdapter {
    /// Adapter-specific fail-closed error.
    type Error;

    /// Fixed TDI-8 reference arm represented by this adapter.
    fn arm(&self) -> ReferenceArm;

    /// Reset all task-dependent persistent state before one generator-level run.
    fn reset(&mut self) -> Result<(), Self::Error>;

    /// Introduce one symbolic key/value association.
    ///
    /// Only the stable key code is exposed. `TaskKey::collision_class()` and the
    /// generator-side source index remain hidden from the adapter.
    fn associate(&mut self, key_code: u64, value: TaskSymbol) -> Result<(), Self::Error>;

    /// Introduce one ordered T2 payload value.
    ///
    /// Payload order is conveyed only by call order; the generator-side source
    /// position is not supplied as an additional feature.
    fn payload(&mut self, value: TaskSymbol) -> Result<(), Self::Error>;

    /// Process one irrelevant symbolic distractor.
    fn distractor(&mut self, token: TaskSymbol) -> Result<(), Self::Error>;

    /// Predict the value associated with one queried symbolic key.
    ///
    /// The exact target and source index are not supplied to the adapter.
    fn query_association(&mut self, key_code: u64) -> Result<TaskSymbol, Self::Error>;

    /// Predict the requested T2 payload position.
    ///
    /// The requested position is part of the symbolic query. The exact target is
    /// retained by the runner and is not supplied to the adapter.
    fn query_payload(&mut self, position: u64) -> Result<TaskSymbol, Self::Error>;
}

/// Runner-owned identity of one exact-target query.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TaskQueryIdentity {
    /// T1/T3 keyed association query.
    Association {
        /// Stable task key code actually supplied to the adapter.
        key_code: u64,
        /// Generator-side collision/interference class retained for analysis.
        /// This field is never passed to [`SymbolicTaskAdapter`].
        collision_class: u64,
        /// Generator-side association index retained for recent/old analysis.
        /// This field is never passed to [`SymbolicTaskAdapter`].
        source_index: u64,
    },
    /// T2 ordered payload query.
    Payload {
        /// Requested payload position supplied to the adapter.
        position: u64,
    },
}

/// One immutable exact-target query result owned by the evaluator.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TaskQueryRecord {
    event_index: usize,
    identity: TaskQueryIdentity,
    target: TaskSymbol,
    prediction: TaskSymbol,
}

impl TaskQueryRecord {
    /// Zero-based event index in the original generator-owned sequence.
    #[must_use]
    pub const fn event_index(self) -> usize {
        self.event_index
    }

    /// Runner-owned query identity and analysis metadata.
    #[must_use]
    pub const fn identity(self) -> TaskQueryIdentity {
        self.identity
    }

    /// Exact generator-owned target. It was never exposed to the adapter.
    #[must_use]
    pub const fn target(self) -> TaskSymbol {
        self.target
    }

    /// Symbol predicted by the arm adapter.
    #[must_use]
    pub const fn prediction(self) -> TaskSymbol {
        self.prediction
    }

    /// Exact discrete task success for this query.
    #[must_use]
    pub const fn exact_success(self) -> bool {
        self.prediction.code() == self.target.code()
    }
}

/// Complete architecture-neutral result of one symbolic task execution.
///
/// This record contains exact discrete predictions only. It deliberately does
/// not define the later TDI-8 late-retrieval deficit or uncertainty interval.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskExecutionRecord {
    arm: ReferenceArm,
    family: TaskFamily,
    generator_seed: u64,
    event_count: usize,
    declared_query_count: u64,
    queries: Vec<TaskQueryRecord>,
}

impl TaskExecutionRecord {
    /// Reference arm that executed the task.
    #[must_use]
    pub const fn arm(&self) -> ReferenceArm {
        self.arm
    }

    /// Frozen task family from the generator-owned instance.
    #[must_use]
    pub const fn family(&self) -> TaskFamily {
        self.family
    }

    /// Exact generator seed from the source task instance.
    #[must_use]
    pub const fn generator_seed(&self) -> u64 {
        self.generator_seed
    }

    /// Number of source events processed in their original order.
    #[must_use]
    pub const fn event_count(&self) -> usize {
        self.event_count
    }

    /// Query count declared by the source task instance.
    #[must_use]
    pub const fn declared_query_count(&self) -> u64 {
        self.declared_query_count
    }

    /// Ordered exact-target query records.
    #[must_use]
    pub fn queries(&self) -> &[TaskQueryRecord] {
        &self.queries
    }

    /// Number of exact discrete query successes.
    #[must_use]
    pub fn successful_queries(&self) -> usize {
        self.queries
            .iter()
            .filter(|record| record.exact_success())
            .count()
    }

    /// Number of exact discrete query failures.
    #[must_use]
    pub fn failed_queries(&self) -> usize {
        self.queries.len() - self.successful_queries()
    }

    /// Whether every declared exact-target query was predicted exactly.
    #[must_use]
    pub fn all_queries_exact(&self) -> bool {
        self.failed_queries() == 0
            && u64::try_from(self.queries.len()).ok() == Some(self.declared_query_count)
    }
}

/// Fail-closed symbolic executor errors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TaskExecutionError<E> {
    /// The generated task's query count does not fit the current host index type.
    QueryCountTooLarge {
        /// Platform-independent declared count.
        query_count: u64,
    },
    /// The host allocator refused exact query-record reservation.
    QueryRecordAllocationFailed {
        /// Requested number of records.
        records: usize,
    },
    /// Adapter reset failed before any task event was processed.
    AdapterReset(E),
    /// One adapter event failed closed.
    AdapterEvent {
        /// Zero-based source event index.
        event_index: usize,
        /// Adapter-specific error.
        error: E,
    },
    /// Adapter identity changed during one task execution.
    ArmChanged {
        /// `None` means the drift was observed immediately after reset. `Some`
        /// identifies the event after which the drift was observed.
        event_index: Option<usize>,
        /// Arm captured before reset.
        expected: ReferenceArm,
        /// Arm reported after reset/event processing.
        observed: ReferenceArm,
    },
    /// The number of query events observed in the immutable event sequence does
    /// not match the generator-owned declaration.
    QueryCountMismatch {
        /// Generator-owned declaration.
        declared: u64,
        /// Query events actually processed.
        observed: u64,
    },
}

impl<E: fmt::Display> fmt::Display for TaskExecutionError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::QueryCountTooLarge { query_count } => write!(
                formatter,
                "TDI-8 symbolic query count {query_count} does not fit the host index type"
            ),
            Self::QueryRecordAllocationFailed { records } => write!(
                formatter,
                "host allocation failed for {records} TDI-8 query records"
            ),
            Self::AdapterReset(error) => write!(formatter, "TDI-8 adapter reset failed: {error}"),
            Self::AdapterEvent { event_index, error } => write!(
                formatter,
                "TDI-8 adapter failed at source event {event_index}: {error}"
            ),
            Self::ArmChanged {
                event_index,
                expected,
                observed,
            } => write!(
                formatter,
                "TDI-8 adapter arm changed after event {event_index:?}: expected {expected:?}, observed {observed:?}"
            ),
            Self::QueryCountMismatch { declared, observed } => write!(
                formatter,
                "TDI-8 symbolic query count mismatch: declared {declared}, observed {observed}"
            ),
        }
    }
}

impl<E> std::error::Error for TaskExecutionError<E> where E: std::error::Error + 'static {}

fn reserve_query_records<E>(
    query_count: u64,
) -> Result<Vec<TaskQueryRecord>, TaskExecutionError<E>> {
    let records = usize::try_from(query_count)
        .map_err(|_| TaskExecutionError::QueryCountTooLarge { query_count })?;
    let mut queries = Vec::new();
    queries
        .try_reserve_exact(records)
        .map_err(|_| TaskExecutionError::QueryRecordAllocationFailed { records })?;
    Ok(queries)
}

fn ensure_arm<E>(
    adapter: &impl SymbolicTaskAdapter<Error = E>,
    expected: ReferenceArm,
    event_index: Option<usize>,
) -> Result<(), TaskExecutionError<E>> {
    let observed = adapter.arm();
    if observed == expected {
        Ok(())
    } else {
        Err(TaskExecutionError::ArmChanged {
            event_index,
            expected,
            observed,
        })
    }
}

fn adapter_event<E, T>(
    event_index: usize,
    result: Result<T, E>,
) -> Result<T, TaskExecutionError<E>> {
    result.map_err(|error| TaskExecutionError::AdapterEvent { event_index, error })
}

fn association_identity(key: TaskKey, source_index: u64) -> TaskQueryIdentity {
    TaskQueryIdentity::Association {
        key_code: key.code(),
        collision_class: key.collision_class(),
        source_index,
    }
}

/// Execute one immutable symbolic task instance through one reset arm adapter.
///
/// Event order, task family, generator seed, query identities and exact targets
/// are owned by `instance`. The adapter receives only the leakage-safe method
/// arguments declared by [`SymbolicTaskAdapter`]. A query record is created by
/// the runner after the adapter returns a prediction; the adapter cannot supply
/// or rewrite its target/provenance metadata through this API.
pub fn execute_symbolic_task<A>(
    instance: &TaskInstance,
    adapter: &mut A,
) -> Result<TaskExecutionRecord, TaskExecutionError<A::Error>>
where
    A: SymbolicTaskAdapter,
{
    let expected_arm = adapter.arm();
    let mut queries = reserve_query_records(instance.query_count())?;

    adapter.reset().map_err(TaskExecutionError::AdapterReset)?;
    ensure_arm(adapter, expected_arm, None)?;

    for (event_index, event) in instance.events().iter().copied().enumerate() {
        match event {
            TaskEvent::Associate { key, value, .. } => {
                adapter_event(event_index, adapter.associate(key.code(), value))?;
            }
            TaskEvent::Payload { value, .. } => {
                adapter_event(event_index, adapter.payload(value))?;
            }
            TaskEvent::Distractor { token } => {
                adapter_event(event_index, adapter.distractor(token))?;
            }
            TaskEvent::QueryAssociation {
                key,
                target,
                source_index,
            } => {
                let prediction = adapter_event(event_index, adapter.query_association(key.code()))?;
                queries.push(TaskQueryRecord {
                    event_index,
                    identity: association_identity(key, source_index),
                    target,
                    prediction,
                });
            }
            TaskEvent::QueryPayload { position, target } => {
                let prediction = adapter_event(event_index, adapter.query_payload(position))?;
                queries.push(TaskQueryRecord {
                    event_index,
                    identity: TaskQueryIdentity::Payload { position },
                    target,
                    prediction,
                });
            }
        }
        ensure_arm(adapter, expected_arm, Some(event_index))?;
    }

    let observed =
        u64::try_from(queries.len()).map_err(|_| TaskExecutionError::QueryCountMismatch {
            declared: instance.query_count(),
            observed: u64::MAX,
        })?;
    if observed != instance.query_count() {
        return Err(TaskExecutionError::QueryCountMismatch {
            declared: instance.query_count(),
            observed,
        });
    }

    Ok(TaskExecutionRecord {
        arm: expected_arm,
        family: instance.family(),
        generator_seed: instance.seed(),
        event_count: instance.event_count(),
        declared_query_count: instance.query_count(),
        queries,
    })
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::{
        SymbolicTaskAdapter, TaskExecutionError, TaskQueryIdentity, execute_symbolic_task,
    };
    use crate::ReferenceArm;
    use crate::task_generators::{
        T1Config, T2Config, T3Config, TaskEvent, TaskInstance, TaskSymbol, generate_t1,
        generate_t2, generate_t3,
    };

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum SyntheticError {
        MissingPrediction,
        ForcedFailure,
    }

    impl core::fmt::Display for SyntheticError {
        fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(formatter, "{self:?}")
        }
    }

    impl std::error::Error for SyntheticError {}

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum ObservedStimulus {
        Associate { key_code: u64, value: TaskSymbol },
        Payload { value: TaskSymbol },
        Distractor { token: TaskSymbol },
        QueryAssociation { key_code: u64 },
        QueryPayload { position: u64 },
    }

    #[derive(Debug)]
    struct ScriptedAdapter {
        arm: ReferenceArm,
        reset_count: usize,
        calls: Vec<ObservedStimulus>,
        predictions: VecDeque<TaskSymbol>,
        fail_on_call: Option<usize>,
        drift_on_call: Option<usize>,
    }

    impl ScriptedAdapter {
        fn new(arm: ReferenceArm, predictions: Vec<TaskSymbol>) -> Self {
            Self {
                arm,
                reset_count: 0,
                calls: Vec::new(),
                predictions: predictions.into(),
                fail_on_call: None,
                drift_on_call: None,
            }
        }

        fn before_call(&mut self) -> Result<(), SyntheticError> {
            let call_index = self.calls.len();
            if self.fail_on_call == Some(call_index) {
                return Err(SyntheticError::ForcedFailure);
            }
            if self.drift_on_call == Some(call_index) {
                self.arm = match self.arm {
                    ReferenceArm::A0 => ReferenceArm::A1,
                    _ => ReferenceArm::A0,
                };
            }
            Ok(())
        }

        fn prediction(&mut self) -> Result<TaskSymbol, SyntheticError> {
            self.predictions
                .pop_front()
                .ok_or(SyntheticError::MissingPrediction)
        }
    }

    impl SymbolicTaskAdapter for ScriptedAdapter {
        type Error = SyntheticError;

        fn arm(&self) -> ReferenceArm {
            self.arm
        }

        fn reset(&mut self) -> Result<(), Self::Error> {
            self.reset_count += 1;
            self.calls.clear();
            Ok(())
        }

        fn associate(&mut self, key_code: u64, value: TaskSymbol) -> Result<(), Self::Error> {
            self.before_call()?;
            self.calls
                .push(ObservedStimulus::Associate { key_code, value });
            Ok(())
        }

        fn payload(&mut self, value: TaskSymbol) -> Result<(), Self::Error> {
            self.before_call()?;
            self.calls.push(ObservedStimulus::Payload { value });
            Ok(())
        }

        fn distractor(&mut self, token: TaskSymbol) -> Result<(), Self::Error> {
            self.before_call()?;
            self.calls.push(ObservedStimulus::Distractor { token });
            Ok(())
        }

        fn query_association(&mut self, key_code: u64) -> Result<TaskSymbol, Self::Error> {
            self.before_call()?;
            self.calls
                .push(ObservedStimulus::QueryAssociation { key_code });
            self.prediction()
        }

        fn query_payload(&mut self, position: u64) -> Result<TaskSymbol, Self::Error> {
            self.before_call()?;
            self.calls.push(ObservedStimulus::QueryPayload { position });
            self.prediction()
        }
    }

    fn query_targets(instance: &TaskInstance) -> Vec<TaskSymbol> {
        instance
            .events()
            .iter()
            .filter_map(|event| match event {
                TaskEvent::QueryAssociation { target, .. }
                | TaskEvent::QueryPayload { target, .. } => Some(*target),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn runner_preserves_exact_event_order_and_targets_without_exposing_labels() {
        let instance = generate_t1(17, T1Config::new(5, 3, 2).expect("T1 config")).expect("T1");
        let targets = query_targets(&instance);
        let mut adapter = ScriptedAdapter::new(ReferenceArm::A2, targets.clone());
        let record = execute_symbolic_task(&instance, &mut adapter).expect("execution");

        assert_eq!(adapter.reset_count, 1);
        assert_eq!(adapter.calls.len(), instance.event_count());
        assert_eq!(record.arm(), ReferenceArm::A2);
        assert_eq!(record.family(), instance.family());
        assert_eq!(record.generator_seed(), instance.seed());
        assert_eq!(record.event_count(), instance.event_count());
        assert_eq!(record.queries().len(), 2);
        assert_eq!(record.successful_queries(), 2);
        assert_eq!(record.failed_queries(), 0);
        assert!(record.all_queries_exact());
        assert_eq!(record.queries()[0].target(), targets[0]);

        let first_query_event = instance
            .events()
            .iter()
            .enumerate()
            .find_map(|(index, event)| match event {
                TaskEvent::QueryAssociation {
                    key, source_index, ..
                } => Some((index, *key, *source_index)),
                _ => None,
            })
            .expect("query event");
        let query_record = record.queries()[0];
        assert_eq!(query_record.event_index(), first_query_event.0);
        assert_eq!(
            query_record.identity(),
            TaskQueryIdentity::Association {
                key_code: first_query_event.1.code(),
                collision_class: first_query_event.1.collision_class(),
                source_index: first_query_event.2,
            }
        );
        assert!(matches!(
            adapter.calls[first_query_event.0],
            ObservedStimulus::QueryAssociation { key_code }
                if key_code == first_query_event.1.code()
        ));
    }

    #[test]
    fn delayed_copy_exposes_query_position_but_not_exact_target() {
        let instance = generate_t2(29, T2Config::new(3, 4).expect("T2 config")).expect("T2");
        let targets = query_targets(&instance);
        let mut adapter = ScriptedAdapter::new(ReferenceArm::A1, targets);
        let record = execute_symbolic_task(&instance, &mut adapter).expect("execution");

        assert_eq!(record.queries().len(), 3);
        assert!(record.all_queries_exact());
        for query in record.queries() {
            assert!(matches!(
                query.identity(),
                TaskQueryIdentity::Payload { .. }
            ));
            assert!(matches!(
                adapter.calls[query.event_index()],
                ObservedStimulus::QueryPayload { .. }
            ));
        }
    }

    #[test]
    fn t3_collision_class_and_source_index_remain_runner_owned_metadata() {
        let instance =
            generate_t3(41, T3Config::new(8, 3, 4, 20, 3).expect("T3 config")).expect("T3");
        let targets = query_targets(&instance);
        let mut adapter = ScriptedAdapter::new(ReferenceArm::A3, targets);
        let record = execute_symbolic_task(&instance, &mut adapter).expect("execution");

        for query in record.queries() {
            let TaskQueryIdentity::Association {
                key_code,
                collision_class,
                source_index,
            } = query.identity()
            else {
                panic!("T3 query must be associative");
            };
            assert!(collision_class < 3);
            assert!(source_index < 8);
            assert!(matches!(
                adapter.calls[query.event_index()],
                ObservedStimulus::QueryAssociation { key_code: observed }
                    if observed == key_code
            ));
        }
    }

    #[test]
    fn adapter_failure_is_typed_with_exact_source_event_index() {
        let instance = generate_t1(5, T1Config::new(3, 2, 1).expect("T1 config")).expect("T1");
        let mut adapter = ScriptedAdapter::new(ReferenceArm::A0, query_targets(&instance));
        adapter.fail_on_call = Some(2);

        assert_eq!(
            execute_symbolic_task(&instance, &mut adapter),
            Err(TaskExecutionError::AdapterEvent {
                event_index: 2,
                error: SyntheticError::ForcedFailure,
            })
        );
    }

    #[test]
    fn adapter_arm_identity_cannot_drift_mid_instance() {
        let instance = generate_t2(7, T2Config::new(2, 2).expect("T2 config")).expect("T2");
        let mut adapter = ScriptedAdapter::new(ReferenceArm::A1, query_targets(&instance));
        adapter.drift_on_call = Some(1);

        assert_eq!(
            execute_symbolic_task(&instance, &mut adapter),
            Err(TaskExecutionError::ArmChanged {
                event_index: Some(1),
                expected: ReferenceArm::A1,
                observed: ReferenceArm::A0,
            })
        );
    }
}
