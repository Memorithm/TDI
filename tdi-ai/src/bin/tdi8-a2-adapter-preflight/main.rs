pub use tdi_ai::{
    ReferenceArm, associative_memory, assr_reference, full_history_reference, task_encoding,
    task_execution, task_generators, task_readout,
};
#[path = "../../task_adapters.rs"]
mod task_adapters;

use std::error::Error;

use task_adapters::A2Adapter;
use task_encoding::{
    MIN_TASK_INPUT_WIDTH, audit_associative_projection, distractor_read_key_for_instance,
};
use task_readout::{ExactStateReadoutLayout, ExactStateSymbolReadout};
use tdi_ai::associative_memory::{AssociativeMemoryLayout, DirectMappedAssociativeMemory};
use tdi_ai::assr_reference::{RecurrentLayout, RecurrentParameters};
use tdi_ai::task_execution::execute_symbolic_task;
use tdi_ai::task_generators::{T1Config, generate_t1};

const FIXTURE_SLOTS: u64 = 4_096;
const FIXTURE_PROJECTION_SEED: u64 = 11;
const FIXTURE_FUSION_GAIN: f64 = 1.0;

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
    if diagnostics.query_hits() != instance.query_count()
        || diagnostics.query_collision_misses() != 0
        || diagnostics.query_empty() != 0
        || diagnostics.replacement_writes() != 0
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
        diagnostics.replacement_writes()
    );
    println!("query_hits={}", diagnostics.query_hits());
    println!("generator_collision_class_used_as_input=NO");
    println!("a3_vsa_policy=NOT_SELECTED");
    println!("final_holdout=DOES_NOT_EXIST");
    println!("tdi8_2_surface=ABSENT");
    Ok(())
}
