#[path = "../../task_encoding.rs"]
pub mod task_encoding;
#[path = "../../task_readout.rs"]
pub mod task_readout;

pub use tdi_ai::{associative_memory, task_generators};

use core::fmt;
use std::error::Error;

use task_encoding::{
    LosslessTaskEncoder, MIN_TASK_INPUT_WIDTH, PayloadKeyCursor, TaskEncodingError,
    TaskInputLayout, association_memory_key, audit_associative_projection,
    distractor_read_key_for_instance, payload_memory_key,
};
use task_readout::{
    ExactStatePrediction, ExactStateReadoutLayout, ExactStateSymbolReadout, TaskReadoutError,
};
use tdi_ai::ReferenceArm;
use tdi_ai::associative_memory::{
    AssociativeMemoryError, AssociativeMemoryLayout, AssociativeWriteOutcome,
    DirectMappedAssociativeMemory,
};
use tdi_ai::assr_reference::{
    A2ReadStatus, A2Reference, A2StepReport, RecurrentLayout, RecurrentParameters,
    RecurrentReferenceError,
};
use tdi_ai::task_execution::{SymbolicTaskAdapter, TaskPrediction, execute_symbolic_task};
use tdi_ai::task_generators::{T1Config, TaskSymbol, generate_t1};

const FIXTURE_SLOTS: u64 = 4_096;
const FIXTURE_PROJECTION_SEED: u64 = 11;
const FIXTURE_FUSION_GAIN: f64 = 1.0;

#[derive(Debug)]
enum AdapterError {
    Associative(AssociativeMemoryError),
    Encoding(TaskEncodingError),
    Recurrent(RecurrentReferenceError),
    Readout(TaskReadoutError),
    ReadoutStateWidthMismatch { recurrent: u64, readout: u64 },
    UnexpectedNeutralReadHit { address: u64 },
    CounterOverflow,
}

impl fmt::Display for AdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Associative(error) => write!(formatter, "associative memory: {error}"),
            Self::Encoding(error) => write!(formatter, "task encoding: {error}"),
            Self::Recurrent(error) => write!(formatter, "A2 reference: {error}"),
            Self::Readout(error) => write!(formatter, "exact readout: {error}"),
            Self::ReadoutStateWidthMismatch { recurrent, readout } => write!(
                formatter,
                "A2 recurrent state width {recurrent} does not match readout state width {readout}"
            ),
            Self::UnexpectedNeutralReadHit { address } => write!(
                formatter,
                "A2 neutral non-query read unexpectedly hit resident memory at address {address}"
            ),
            Self::CounterOverflow => formatter.write_str("A2 adapter diagnostic counter overflow"),
        }
    }
}

impl Error for AdapterError {}

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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct A2Diagnostics {
    query_hits: u64,
    query_collision_misses: u64,
    query_empty: u64,
    inserted_writes: u64,
    updated_writes: u64,
    replacement_writes: u64,
}

impl A2Diagnostics {
    fn increment(value: &mut u64) -> Result<(), AdapterError> {
        *value = value.checked_add(1).ok_or(AdapterError::CounterOverflow)?;
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
}

struct A2Adapter {
    reference: A2Reference,
    encoder: LosslessTaskEncoder,
    readout: ExactStateSymbolReadout,
    payload_keys: PayloadKeyCursor,
    neutral_read_key: u64,
    diagnostics: A2Diagnostics,
}

impl A2Adapter {
    fn new(
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

    fn diagnostics(&self) -> A2Diagnostics {
        self.diagnostics
    }

    fn non_query_step(
        &mut self,
        input: Vec<f64>,
        write_key: Option<u64>,
    ) -> Result<(), AdapterError> {
        let report = self
            .reference
            .step(&input, self.neutral_read_key, write_key)?;
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
        let write_key = self.payload_keys.next_write_key()?;
        self.non_query_step(input, Some(write_key))
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
    .expect("finite T1 value-capture fixture")
}

fn exact_readout() -> ExactStateSymbolReadout {
    ExactStateSymbolReadout::new(ExactStateReadoutLayout::new(2, 0, 1).expect("readout layout"))
}

fn main() -> Result<(), Box<dyn Error>> {
    let instance = generate_t1(17, T1Config::new(5, 3, 2)?)?;
    let neutral_read_key = distractor_read_key_for_instance(&instance)?;
    let memory_layout = AssociativeMemoryLayout::new(FIXTURE_SLOTS, 2)?;

    let audit_memory = DirectMappedAssociativeMemory::new(memory_layout, FIXTURE_PROJECTION_SEED)?;
    let audit = audit_associative_projection(&instance, &audit_memory)?;
    if audit.physical_replacement_collisions() != 0
        || audit.query_collision_misses() != 0
        || audit.query_empty() != 0
        || audit.query_hits() != instance.query_count()
    {
        return Err("A2 collision-free software fixture projection is not collision-free".into());
    }

    let mut adapter = A2Adapter::new(
        t1_value_capture_parameters(),
        memory_layout,
        FIXTURE_PROJECTION_SEED,
        FIXTURE_FUSION_GAIN,
        exact_readout(),
        neutral_read_key,
    )?;
    let record = execute_symbolic_task(&instance, &mut adapter)?;
    let diagnostics = adapter.diagnostics();

    if !record.all_queries_exact() || record.invalid_predictions() != 0 {
        return Err("A2 failed exact bounded T1 associative-recall fixture".into());
    }
    if diagnostics.query_hits != instance.query_count()
        || diagnostics.query_collision_misses != 0
        || diagnostics.query_empty != 0
        || diagnostics.replacement_writes != 0
    {
        return Err("A2 runtime diagnostics drifted from collision-free fixture audit".into());
    }

    println!("TDI-8.1 A2 adapter preflight: PASS");
    println!("scope=bounded_preflight_only");
    println!("non_query_memory_read=NEUTRAL_KEY_NO_HIT");
    println!("query_memory_read=LOGICAL_QUERY_KEY");
    println!("t1_exact_associative_recall=PASS_SOFTWARE_FIXTURE_ONLY");
    println!(
        "physical_replacement_collisions={}",
        diagnostics.replacement_writes
    );
    println!("query_hits={}", diagnostics.query_hits);
    println!("generator_collision_class_used_as_input=NO");
    println!("a3_vsa_policy=NOT_SELECTED");
    println!("final_holdout=DOES_NOT_EXIST");
    println!("tdi8_2_surface=ABSENT");
    Ok(())
}
