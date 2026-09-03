#[path = "../../task_encoding.rs"]
pub mod task_encoding;
#[path = "../../task_readout.rs"]
pub mod task_readout;

pub use tdi_ai::{associative_memory, task_generators};

use core::fmt;
use std::error::Error;

use task_encoding::{
    A0_TASK_KEY_WIDTH, A0_TASK_VALUE_WIDTH, LosslessTaskEncoder, MIN_TASK_INPUT_WIDTH,
    TaskEncodingError, TaskInputLayout, a0_association_item, a0_association_query_key,
    a0_distractor_item, a0_payload_item, a0_payload_query_key,
};
use task_readout::{
    ExactStatePrediction, ExactStateReadoutLayout, ExactStateSymbolReadout, TaskReadoutError,
    decode_exact_symbol_coordinates,
};
use tdi_ai::ReferenceArm;
use tdi_ai::assr_reference::{
    A1Reference, RecurrentLayout, RecurrentParameters, RecurrentReferenceError,
};
use tdi_ai::full_history_reference::{A0Reference, A0ReferenceError, FullHistoryLayout};
use tdi_ai::task_execution::{SymbolicTaskAdapter, TaskPrediction, execute_symbolic_task};
use tdi_ai::task_generators::{T1Config, T2Config, TaskSymbol, generate_t1, generate_t2};

#[derive(Debug)]
enum AdapterError {
    A0(A0ReferenceError),
    Encoding(TaskEncodingError),
    Recurrent(RecurrentReferenceError),
    Readout(TaskReadoutError),
    ReadoutStateWidthMismatch { recurrent: u64, readout: u64 },
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

struct A0Adapter {
    reference: A0Reference,
    next_payload_position: u64,
}

impl A0Adapter {
    fn new() -> Result<Self, AdapterError> {
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

struct A1Adapter {
    reference: A1Reference,
    encoder: LosslessTaskEncoder,
    readout: ExactStateSymbolReadout,
}

impl A1Adapter {
    fn new(
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
    .expect("finite key-echo fixture")
}

fn invalid_output_parameters() -> RecurrentParameters {
    let input_width = MIN_TASK_INPUT_WIDTH as usize;
    let state_width = 2usize;
    RecurrentParameters::new(
        RecurrentLayout::new(MIN_TASK_INPUT_WIDTH, state_width as u64).expect("layout"),
        vec![0.0; input_width * state_width],
        vec![0.0; state_width * state_width],
        vec![0.1, 0.0],
    )
    .expect("finite invalid-output fixture")
}

fn exact_readout() -> ExactStateSymbolReadout {
    ExactStateSymbolReadout::new(ExactStateReadoutLayout::new(2, 0, 1).expect("readout layout"))
}

fn main() -> Result<(), Box<dyn Error>> {
    let t1 = generate_t1(17, T1Config::new(5, 3, 2)?)?;
    let t2 = generate_t2(29, T2Config::new(3, 4)?)?;

    let mut a0 = A0Adapter::new()?;
    let a0_t1 = execute_symbolic_task(&t1, &mut a0)?;
    if !a0_t1.all_queries_exact() || a0_t1.invalid_predictions() != 0 {
        return Err("A0 failed exact T1 adapter preflight".into());
    }
    let a0_t2 = execute_symbolic_task(&t2, &mut a0)?;
    if !a0_t2.all_queries_exact() || a0_t2.invalid_predictions() != 0 {
        return Err("A0 failed exact T2 adapter preflight".into());
    }

    let mut a1 = A1Adapter::new(key_echo_parameters(), exact_readout())?;
    let a1_t1 = execute_symbolic_task(&t1, &mut a1)?;
    if a1_t1.queries().len() != 2 || a1_t1.invalid_predictions() != 0 {
        return Err("A1 valid-symbol adapter path failed".into());
    }

    let mut invalid_a1 = A1Adapter::new(invalid_output_parameters(), exact_readout())?;
    let invalid_record = execute_symbolic_task(&t1, &mut invalid_a1)?;
    if invalid_record.invalid_predictions() != invalid_record.queries().len()
        || invalid_record.failed_queries() != invalid_record.queries().len()
    {
        return Err("A1 invalid readout was not retained as evaluated failure".into());
    }

    println!("TDI-8.1 A0/A1 adapter preflight: PASS");
    println!("scope=bounded_preflight_only");
    println!("a0_t1_exact=PASS");
    println!("a0_t2_exact=PASS");
    println!("a1_valid_symbol_path=PASS_NO_QUALITY_CLAIM");
    println!("a1_invalid_readout=COUNTED_AS_FAILURE");
    println!("a2_a3_adapter_policy=NOT_SELECTED");
    println!("final_holdout=DOES_NOT_EXIST");
    println!("tdi8_2_surface=ABSENT");
    Ok(())
}
