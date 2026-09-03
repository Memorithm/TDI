#[path = "../../task_encoding.rs"]
pub mod task_encoding;
#[path = "../../task_readout.rs"]
pub mod task_readout;
#[path = "../../a2_task_policy.rs"]
pub mod a2_task_policy;

pub use tdi_ai::{associative_memory, task_generators};

use core::fmt;
use std::error::Error;

use a2_task_policy::{A2TaskRoute, A2TaskRouting};
use associative_memory::{AssociativeMemoryLayout, AssociativeWriteOutcome};
use task_encoding::{
    LosslessTaskEncoder, MIN_TASK_INPUT_WIDTH, ProjectionAudit, TaskEncodingError, TaskInputLayout,
    audit_associative_projection,
};
use task_readout::{
    ExactStatePrediction, ExactStateReadoutLayout, ExactStateSymbolReadout, TaskReadoutError,
};
use tdi_ai::ReferenceArm;
use tdi_ai::assr_reference::{
    A2ReadStatus, A2Reference, A2StepReport, RecurrentLayout, RecurrentParameters,
    RecurrentReferenceError,
};
use tdi_ai::task_execution::{
    SymbolicTaskAdapter, TaskPrediction, execute_symbolic_task,
};
use tdi_ai::task_generators::{
    T2Config, T3Config, TaskInstance, TaskSymbol, generate_t2, generate_t3,
};

#[derive(Debug)]
enum AdapterError {
    Encoding(TaskEncodingError),
    Recurrent(RecurrentReferenceError),
    Readout(TaskReadoutError),
    ReadoutStateWidthMismatch { recurrent: u64, readout: u64 },
}

impl fmt::Display for AdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Encoding(error) => write!(formatter, "task encoding: {error}"),
            Self::Recurrent(error) => write!(formatter, "A2 reference: {error}"),
            Self::Readout(error) => write!(formatter, "exact readout: {error}"),
            Self::ReadoutStateWidthMismatch { recurrent, readout } => write!(
                formatter,
                "A2 recurrent state width {recurrent} does not match readout state width {readout}"
            ),
        }
    }
}

impl Error for AdapterError {}

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
struct A2ObservedRouting {
    writes: u64,
    replacement_collisions: u64,
    query_hits: u64,
    query_collision_misses: u64,
    query_empty: u64,
}

impl A2ObservedRouting {
    fn record_write(&mut self, report: A2StepReport) {
        if let Some(outcome) = report.write() {
            self.writes += 1;
            if matches!(outcome, AssociativeWriteOutcome::ReplacedCollision { .. }) {
                self.replacement_collisions += 1;
            }
        }
    }

    fn record_query(&mut self, report: A2StepReport) {
        match report.read() {
            A2ReadStatus::Hit { .. } => self.query_hits += 1,
            A2ReadStatus::CollisionMiss { .. } => self.query_collision_misses += 1,
            A2ReadStatus::Empty { .. } => self.query_empty += 1,
        }
    }

    fn matches_audit(self, audit: ProjectionAudit) -> bool {
        self.writes == audit.planned_writes()
            && self.replacement_collisions == audit.physical_replacement_collisions()
            && self.query_hits == audit.query_hits()
            && self.query_collision_misses == audit.query_collision_misses()
            && self.query_empty == audit.query_empty()
    }
}

struct A2Adapter {
    reference: A2Reference,
    encoder: LosslessTaskEncoder,
    readout: ExactStateSymbolReadout,
    routing: A2TaskRouting,
    observed: A2ObservedRouting,
}

impl A2Adapter {
    fn new(
        instance: &TaskInstance,
        parameters: RecurrentParameters,
        memory_layout: AssociativeMemoryLayout,
        projection_seed: u64,
        fusion_gain: f64,
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
            reference: A2Reference::new(parameters, memory_layout, projection_seed, fusion_gain)?,
            encoder: LosslessTaskEncoder::new(TaskInputLayout::new(
                recurrent_layout.input_width(),
            )?),
            readout,
            routing: A2TaskRouting::for_instance(instance)?,
            observed: A2ObservedRouting::default(),
        })
    }

    fn step(&mut self, input: Vec<f64>, route: A2TaskRoute) -> Result<A2StepReport, AdapterError> {
        Ok(self
            .reference
            .step(&input, route.read_key(), route.write_key())?)
    }

    fn prediction(&self) -> Result<TaskPrediction, AdapterError> {
        Ok(match self.readout.decode_state(self.reference.state())? {
            ExactStatePrediction::Symbol(symbol) => TaskPrediction::Symbol(symbol),
            ExactStatePrediction::InvalidEncoding => TaskPrediction::Invalid,
        })
    }

    fn projection_audit(&self, instance: &TaskInstance) -> Result<ProjectionAudit, AdapterError> {
        Ok(audit_associative_projection(
            instance,
            self.reference.associative_memory(),
        )?)
    }
}

impl SymbolicTaskAdapter for A2Adapter {
    type Error = AdapterError;

    fn arm(&self) -> ReferenceArm {
        ReferenceArm::A2
    }

    fn reset(&mut self) -> Result<(), Self::Error> {
        self.reference.reset();
        self.routing.reset();
        self.observed = A2ObservedRouting::default();
        Ok(())
    }

    fn associate(&mut self, key_code: u64, value: TaskSymbol) -> Result<(), Self::Error> {
        let route = self.routing.association(key_code);
        let input = self.encoder.association(key_code, value)?;
        let report = self.step(input, route)?;
        self.observed.record_write(report);
        Ok(())
    }

    fn payload(&mut self, value: TaskSymbol) -> Result<(), Self::Error> {
        let (route, next_routing) = self.routing.prospective_payload()?;
        let input = self.encoder.payload(value)?;
        let report = self.step(input, route)?;
        self.observed.record_write(report);
        self.routing = next_routing;
        Ok(())
    }

    fn distractor(&mut self, token: TaskSymbol) -> Result<(), Self::Error> {
        let route = self.routing.distractor();
        let input = self.encoder.distractor(token)?;
        let _ = self.step(input, route)?;
        Ok(())
    }

    fn query_association(&mut self, key_code: u64) -> Result<TaskPrediction, Self::Error> {
        let route = self.routing.query_association(key_code);
        let input = self.encoder.query_association(key_code)?;
        let report = self.step(input, route)?;
        self.observed.record_query(report);
        self.prediction()
    }

    fn query_payload(&mut self, position: u64) -> Result<TaskPrediction, Self::Error> {
        let route = self.routing.query_payload(position);
        let input = self.encoder.query_payload(position)?;
        let report = self.step(input, route)?;
        self.observed.record_query(report);
        self.prediction()
    }
}

fn constant_parameters(value: f64) -> RecurrentParameters {
    let state_width = 2usize;
    let input_width = MIN_TASK_INPUT_WIDTH as usize;
    let layout = RecurrentLayout::new(MIN_TASK_INPUT_WIDTH, state_width as u64).expect("layout");
    RecurrentParameters::new(
        layout,
        vec![0.0; state_width * input_width],
        vec![0.0; state_width * state_width],
        vec![value; state_width],
    )
    .expect("finite software-oracle parameters")
}

fn exact_readout() -> ExactStateSymbolReadout {
    ExactStateSymbolReadout::new(ExactStateReadoutLayout::new(2, 0, 1).expect("readout layout"))
}

fn verify_instance(instance: &TaskInstance, memory_slots: u64) -> Result<(), Box<dyn Error>> {
    let mut adapter = A2Adapter::new(
        instance,
        constant_parameters(0.1),
        AssociativeMemoryLayout::new(memory_slots, 2)?,
        31,
        0.5,
        exact_readout(),
    )?;
    let audit = adapter.projection_audit(instance)?;
    let record = execute_symbolic_task(instance, &mut adapter)?;

    if !adapter.observed.matches_audit(audit) {
        return Err("observed A2 routing/collision outcomes diverged from projection audit".into());
    }
    if record.queries().len() as u64 != instance.query_count() {
        return Err("A2 adapter dropped a declared query".into());
    }
    if record.successful_queries() + record.failed_queries() != record.queries().len() {
        return Err("A2 query denominator accounting is inconsistent".into());
    }
    let _ = adapter.reference.memory_accounting()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let t2 = generate_t2(82, T2Config::new(4, 3)?)?;
    let t3 = generate_t3(83, T3Config::new(6, 3, 3, 48, 2)?)?;

    verify_instance(&t2, 2)?;
    verify_instance(&t3, 2)?;

    println!("TDI-8.1 A2 task-policy preflight: PASS");
    println!("scope=bounded_software_oracle_only");
    println!("association_route=READ_SAME_KEY_THEN_WRITE_SAME_KEY");
    println!("payload_route=CALL_ORDER_DOMAIN_KEY_READ_THEN_WRITE");
    println!("distractor_route=INSTANCE_SAFE_READ_ONLY_KEY");
    println!("query_route=MATCHING_LOGICAL_KEY_READ_ONLY");
    println!("payload_cursor=COMMIT_AFTER_SUCCESSFUL_A2_STEP");
    println!("physical_collision_accounting=MATCHES_PROJECTION_AUDIT");
    println!("generator_collision_class=NOT_ARM_INPUT");
    println!("invalid_predictions=COUNTED_AS_FAILURES");
    println!("a3_vsa_policy=NOT_SELECTED");
    println!("h8_a_h8_b_result=NOT_COMPUTED");
    println!("final_holdout=DOES_NOT_EXIST");
    println!("tdi8_2_surface=ABSENT");
    Ok(())
}
