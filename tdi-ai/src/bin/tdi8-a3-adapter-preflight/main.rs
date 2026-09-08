use std::error::Error;

use tdi_ai::associative_memory::{AssociativeMemoryLayout, DirectMappedAssociativeMemory};
use tdi_ai::assr_reference::{RecurrentLayout, RecurrentParameters};
use tdi_ai::task_adapters::a3::{A3Adapter, A3AdapterError, A3Diagnostics};
use tdi_ai::task_encoding::{
    MIN_TASK_INPUT_WIDTH, audit_associative_projection, distractor_read_key_for_instance,
};
use tdi_ai::task_execution::{SymbolicTaskAdapter, TaskPrediction, execute_symbolic_task};
use tdi_ai::task_generators::{T2Config, TaskSymbol, generate_t2};
use tdi_ai::task_readout::{ExactStateReadoutLayout, ExactStateSymbolReadout};

const FIXTURE_SLOTS: u64 = 4_096;
const FIXTURE_PROJECTION_SEED: u64 = 11;
const FIXTURE_ASSOCIATIVE_FUSION_GAIN: f64 = 1.0;
const FIXTURE_VSA_ROLE_SEED: u64 = 23;
const FIXTURE_VSA_FUSION_GAIN: f64 = 1.0;
const FIXTURE_INPUT_WEIGHT: f64 = 0.5;
const FIXTURE_STATE_WIDTH: u64 = 4;

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
) -> Result<A3Adapter, A3AdapterError> {
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
    if diagnostics.query_hits() != instance.query_count()
        || diagnostics.query_collision_misses() != 0
        || diagnostics.query_empty() != 0
        || diagnostics.replacement_writes() != 0
        || diagnostics.vsa_stores() != 1
        || diagnostics.vsa_queries() != 1
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
    println!("vsa_stores={}", diagnostics.vsa_stores());
    println!("vsa_queries={}", diagnostics.vsa_queries());
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
        assert_eq!(diagnostics.vsa_stores(), 1);
        assert_eq!(diagnostics.vsa_queries(), 1);
        assert_eq!(diagnostics.query_hits(), 1);
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
        assert_eq!(diagnostics.vsa_stores(), 1);
        assert_eq!(diagnostics.vsa_queries(), 1);
        assert_eq!(diagnostics.query_hits(), 1);
    }

    #[test]
    fn distractor_and_query_do_not_mutate_vsa_workspace() {
        let key_code = 7;
        let value = TaskSymbol::new(0x0fed_cba9_8765_4321);
        let mut adapter = adapter_with_parameters(dual_path_parameters(), 99).expect("adapter");
        adapter
            .associate(key_code, value)
            .expect("association store");
        let stored = adapter.vsa_components().to_vec();

        adapter
            .distractor(TaskSymbol::new(0x55aa))
            .expect("distractor step");
        assert_eq!(adapter.vsa_components(), stored.as_slice());

        let _ = adapter
            .query_association(key_code)
            .expect("association query");
        assert_eq!(adapter.vsa_components(), stored.as_slice());
    }

    #[test]
    fn payload_cursor_does_not_advance_when_atomic_a2_step_rejects() {
        let mut adapter =
            adapter_with_parameters(failing_payload_parameters(), 99).expect("failing adapter");
        let before_workspace = adapter.vsa_components().to_vec();
        assert_eq!(adapter.payload_position(), 0);
        assert!(adapter.payload(TaskSymbol::new(u64::MAX)).is_err());
        assert_eq!(adapter.payload_position(), 0);
        assert_eq!(adapter.vsa_components(), before_workspace.as_slice());
        assert_eq!(adapter.diagnostics().vsa_stores(), 0);
    }
}
