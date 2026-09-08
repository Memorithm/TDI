pub use tdi_ai::{
    ReferenceArm, assr_reference, full_history_reference, task_encoding, task_execution,
    task_generators, task_readout,
};

use std::error::Error;

use tdi_ai::assr_reference::{RecurrentLayout, RecurrentParameters};
use tdi_ai::task_adapters::{A0Adapter, A1Adapter};
use tdi_ai::task_encoding::MIN_TASK_INPUT_WIDTH;
use tdi_ai::task_execution::execute_symbolic_task;
use tdi_ai::task_generators::{T1Config, T2Config, generate_t1, generate_t2};
use tdi_ai::task_readout::{ExactStateReadoutLayout, ExactStateSymbolReadout};

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
