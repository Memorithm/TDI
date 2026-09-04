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
use tdi_ai::assr_h_reference::{A3Reference, A3ReferenceError, A3VsaReadRoute};
use tdi_ai::assr_reference::{A2ReadStatus, A2StepReport, RecurrentLayout, RecurrentParameters};
use tdi_ai::task_execution::{SymbolicTaskAdapter, TaskPrediction, execute_symbolic_task};
use tdi_ai::task_generators::{T2Config, TaskSymbol, generate_t2};
use tdi_ai::vsa_workspace::VsaWorkspaceLayout;

const FIXTURE_SLOTS: u64 = 4_096;
const FIXTURE_PROJECTION_SEED: u64 = 11;
const FIXTURE_ASSOCIATIVE_FUSION_GAIN: f64 = 1.0;
const FIXTURE_VSA_ROLE_SEED: u64 = 23;
const FIXTURE_VSA_FUSION_GAIN: f64 = 1.0;
const FIXTURE_INPUT_WEIGHT: f64 = 0.5;
const FIXTURE_STATE_WIDTH: u64 = 4;

#[derive(Debug)]
enum AdapterError {
    Associative(AssociativeMemoryError),
    Encoding(TaskEncodingError),
    A3(A3ReferenceError),
    Readout(TaskReadoutError),
    ReadoutStateWidthMismatch {
        recurrent: u64,
        association_readout: u64,
        payload_readout: u64,
    },
    UnexpectedNeutralReadHit {
        address: u64,
    },
    CounterOverflow,
}

impl fmt::Display for AdapterError {
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

impl From<A3ReferenceError> for AdapterError {
    fn from(error: A3ReferenceError) -> Self {
        Self::A3(error)
    }
}

impl From<TaskReadoutError> for AdapterError {
    fn from(error: TaskReadoutError) -> Self {
        Self::Readout(error)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct A3Diagnostics {
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

struct A3Adapter {
    reference: A3Reference,
    encoder: LosslessTaskEncoder,
    association_readout: ExactStateSymbolReadout,
    payload_readout: ExactStateSymbolReadout,
    payload_keys: PayloadKeyCursor,
    neutral_read_key: u64,
    diagnostics: A3Diagnostics,
}

impl A3Adapter {
    #[allow(clippy::too_many_arguments)]
    fn new(
        parameters: RecurrentParameters,
        memory_layout: AssociativeMemoryLayout,
        projection_seed: u64,
        associative_fusion_gain: f64,
        vsa_role_seed: u64,
        vsa_fusion_gain: f64,
        association_readout: ExactStateSymbolReadout,
        payload_readout: ExactStateSymbolReadout,
        neutral_read_key: u64,
    ) -> Result<Self, AdapterError> {
        let recurrent_layout = parameters.layout();
        let association_layout = association_readout.layout();
        let payload_layout = payload_readout.layout();
        if recurrent_layout.state_width() != association_layout.state_width()
            || recurrent_layout.state_width() != payload_layout.state_width()
        {
            return Err(AdapterError::ReadoutStateWidthMismatch {
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

    fn diagnostics(&self) -> A3Diagnostics {
        self.diagnostics
    }

    #[cfg(test)]
    fn payload_position(&self) -> u64 {
        self.payload_keys.next_position()
    }

    fn atomic_store_step(&mut self, input: Vec<f64>, logical_key: u64) -> Result<(), AdapterError> {
        let report = self.reference.step_skip_vsa_and_store(
            &input,
            self.neutral_read_key,
            Some(logical_key),
            logical_key,
            &input,
        )?;
        if let A2ReadStatus::Hit { address } = report.read() {
            return Err(AdapterError::UnexpectedNeutralReadHit { address });
        }
        self.diagnostics.observe_write(report.write())?;
        A3Diagnostics::increment(&mut self.diagnostics.vsa_stores)?;
        Ok(())
    }

    fn distractor_step(&mut self, input: Vec<f64>) -> Result<(), AdapterError> {
        let report = self.reference.step_routed(
            &input,
            A3VsaReadRoute::Skip,
            self.neutral_read_key,
            None,
        )?;
        if let A2ReadStatus::Hit { address } = report.read() {
            return Err(AdapterError::UnexpectedNeutralReadHit { address });
        }
        Ok(())
    }

    fn query_step(
        &mut self,
        input: Vec<f64>,
        read_key: u64,
        readout: ExactStateSymbolReadout,
    ) -> Result<TaskPrediction, AdapterError> {
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
    type Error = AdapterError;

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

fn dual_path_parameters() -> RecurrentParameters {
    let input_width = MIN_TASK_INPUT_WIDTH as usize;
    let state_width = FIXTURE_STATE_WIDTH as usize;
    let layout = RecurrentLayout::new(MIN_TASK_INPUT_WIDTH, FIXTURE_STATE_WIDTH).expect("layout");
    let mut input_to_state = vec![0.0; input_width * state_width];

    // Association symbols occupy input coordinates 3/4; ordered payload symbols
    // occupy 1/2. Each path contributes exactly one half before the A2 memory
    // payload contributes the other half at query time.
    input_to_state[3] = FIXTURE_INPUT_WEIGHT;
    input_to_state[input_width + 4] = FIXTURE_INPUT_WEIGHT;
    input_to_state[2 * input_width + 1] = FIXTURE_INPUT_WEIGHT;
    input_to_state[3 * input_width + 2] = FIXTURE_INPUT_WEIGHT;

    RecurrentParameters::new(
        layout,
        input_to_state,
        vec![0.0; state_width * state_width],
        vec![0.0; state_width],
    )
    .expect("finite dual-path fixture parameters")
}

#[cfg(test)]
fn failing_payload_parameters() -> RecurrentParameters {
    let input_width = MIN_TASK_INPUT_WIDTH as usize;
    let state_width = FIXTURE_STATE_WIDTH as usize;
    let layout = RecurrentLayout::new(MIN_TASK_INPUT_WIDTH, FIXTURE_STATE_WIDTH).expect("layout");
    let mut input_to_state = vec![0.0; input_width * state_width];
    // Both u32 limbs of u64::MAX encode to positive finite values just below 1.
    // Two independent products by f64::MAX are therefore individually finite
    // but their fixed-order sum overflows, forcing the A2 transaction to reject.
    input_to_state[1] = f64::MAX;
    input_to_state[2] = f64::MAX;
    RecurrentParameters::new(
        layout,
        input_to_state,
        vec![0.0; state_width * state_width],
        vec![0.0; state_width],
    )
    .expect("finite forced-failure fixture parameters")
}

fn association_readout() -> ExactStateSymbolReadout {
    ExactStateSymbolReadout::new(
        ExactStateReadoutLayout::new(FIXTURE_STATE_WIDTH, 0, 1).expect("association readout"),
    )
}

fn payload_readout() -> ExactStateSymbolReadout {
    ExactStateSymbolReadout::new(
        ExactStateReadoutLayout::new(FIXTURE_STATE_WIDTH, 2, 3).expect("payload readout"),
    )
}

fn adapter_with_parameters(
    parameters: RecurrentParameters,
    neutral_read_key: u64,
) -> Result<A3Adapter, AdapterError> {
    A3Adapter::new(
        parameters,
        AssociativeMemoryLayout::new(FIXTURE_SLOTS, FIXTURE_STATE_WIDTH)?,
        FIXTURE_PROJECTION_SEED,
        FIXTURE_ASSOCIATIVE_FUSION_GAIN,
        FIXTURE_VSA_ROLE_SEED,
        FIXTURE_VSA_FUSION_GAIN,
        association_readout(),
        payload_readout(),
        neutral_read_key,
    )
}

fn run_t2_preflight() -> Result<A3Diagnostics, Box<dyn Error>> {
    let instance = generate_t2(29, T2Config::new(1, 3)?)?;
    let neutral_read_key = distractor_read_key_for_instance(&instance)?;
    let memory_layout = AssociativeMemoryLayout::new(FIXTURE_SLOTS, FIXTURE_STATE_WIDTH)?;

    let audit_memory = DirectMappedAssociativeMemory::new(memory_layout, FIXTURE_PROJECTION_SEED)?;
    let audit = audit_associative_projection(&instance, &audit_memory)?;
    if audit.physical_replacement_collisions() != 0
        || audit.query_collision_misses() != 0
        || audit.query_empty() != 0
        || audit.query_hits() != instance.query_count()
    {
        return Err("A3 T2 software fixture projection is not collision-free".into());
    }

    let mut adapter = adapter_with_parameters(dual_path_parameters(), neutral_read_key)?;
    let record = execute_symbolic_task(&instance, &mut adapter)?;
    let diagnostics = adapter.diagnostics();

    if !record.all_queries_exact() || record.invalid_predictions() != 0 {
        return Err("A3 failed exact bounded T2 dual-path delayed-copy fixture".into());
    }
    if diagnostics.query_hits != instance.query_count()
        || diagnostics.query_collision_misses != 0
        || diagnostics.query_empty != 0
        || diagnostics.replacement_writes != 0
        || diagnostics.vsa_stores != 1
        || diagnostics.vsa_queries != 1
    {
        return Err("A3 runtime diagnostics drifted from single-payload dual-path fixture".into());
    }
    Ok(diagnostics)
}

fn main() -> Result<(), Box<dyn Error>> {
    let diagnostics = run_t2_preflight()?;

    println!("TDI-8.1 A3 adapter preflight: PASS");
    println!("scope=bounded_preflight_only");
    println!("write_event_vsa_read=SKIP");
    println!("write_event_a2_read=NEUTRAL_KEY_NO_HIT");
    println!("write_key_shared_between_a2_and_vsa=YES");
    println!("query_vsa_read=LOGICAL_QUERY_KEY");
    println!("query_a2_read=LOGICAL_QUERY_KEY");
    println!("query_write=NO");
    println!("vsa_cleanup_rule=NONE");
    println!("fixture_a2_fusion_gain={FIXTURE_ASSOCIATIVE_FUSION_GAIN}");
    println!("fixture_vsa_fusion_gain={FIXTURE_VSA_FUSION_GAIN}");
    println!("fixture_input_weight={FIXTURE_INPUT_WEIGHT}");
    println!("t2_single_payload_exact_dual_path_recall=PASS_SOFTWARE_FIXTURE_ONLY");
    println!("vsa_stores={}", diagnostics.vsa_stores);
    println!("vsa_queries={}", diagnostics.vsa_queries);
    println!("generator_collision_class_used_as_input=NO");
    println!("final_holdout=DOES_NOT_EXIST");
    println!("tdi8_2_surface=ABSENT");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_t2_dual_path_preflight_is_exact() {
        let diagnostics = run_t2_preflight().expect("bounded A3 T2 preflight");
        assert_eq!(diagnostics.vsa_stores, 1);
        assert_eq!(diagnostics.vsa_queries, 1);
        assert_eq!(diagnostics.query_hits, 1);
    }

    #[test]
    fn isolated_association_round_trip_uses_both_a2_and_vsa() {
        let key_code = 42;
        let value = TaskSymbol::new(0x1234_5678_9abc_def0);
        let mut adapter = adapter_with_parameters(dual_path_parameters(), 99).expect("adapter");
        adapter
            .associate(key_code, value)
            .expect("association store");
        let prediction = adapter
            .query_association(key_code)
            .expect("association query");
        assert_eq!(prediction, TaskPrediction::Symbol(value));
        let diagnostics = adapter.diagnostics();
        assert_eq!(diagnostics.vsa_stores, 1);
        assert_eq!(diagnostics.vsa_queries, 1);
        assert_eq!(diagnostics.query_hits, 1);
    }

    #[test]
    fn distractor_and_query_do_not_mutate_vsa_workspace() {
        let key_code = 7;
        let value = TaskSymbol::new(0x0fed_cba9_8765_4321);
        let mut adapter = adapter_with_parameters(dual_path_parameters(), 99).expect("adapter");
        adapter
            .associate(key_code, value)
            .expect("association store");
        let stored = adapter.reference.workspace().components().to_vec();

        adapter
            .distractor(TaskSymbol::new(0x55aa))
            .expect("distractor step");
        assert_eq!(
            adapter.reference.workspace().components(),
            stored.as_slice()
        );

        let _ = adapter
            .query_association(key_code)
            .expect("association query");
        assert_eq!(
            adapter.reference.workspace().components(),
            stored.as_slice()
        );
    }

    #[test]
    fn payload_cursor_does_not_advance_when_atomic_a2_step_rejects() {
        let mut adapter =
            adapter_with_parameters(failing_payload_parameters(), 99).expect("failing adapter");
        let before_workspace = adapter.reference.workspace().components().to_vec();
        assert_eq!(adapter.payload_position(), 0);
        assert!(adapter.payload(TaskSymbol::new(u64::MAX)).is_err());
        assert_eq!(adapter.payload_position(), 0);
        assert_eq!(
            adapter.reference.workspace().components(),
            before_workspace.as_slice()
        );
        assert_eq!(adapter.diagnostics().vsa_stores, 0);
    }
}
