#[path = "../../task_encoding.rs"]
pub mod task_encoding;

use std::error::Error;

use tdi_ai::associative_memory::{AssociativeMemoryLayout, DirectMappedAssociativeMemory};
use tdi_ai::task_generators::{T3Config, generate_t3};

use task_encoding::{
    LosslessTaskEncoder, MIN_TASK_INPUT_WIDTH, TaskInputLayout, audit_associative_projection,
};

fn main() -> Result<(), Box<dyn Error>> {
    let encoder = LosslessTaskEncoder::new(TaskInputLayout::new(MIN_TASK_INPUT_WIDTH)?);
    let query = encoder.query_association(17)?;
    if query.len() != MIN_TASK_INPUT_WIDTH as usize || query[3..].iter().any(|value| *value != 0.0)
    {
        return Err("leakage-safe query frame invariant failed".into());
    }

    let instance = generate_t3(41, T3Config::new(8, 3, 4, 20, 3)?)?;
    let memory = DirectMappedAssociativeMemory::new(AssociativeMemoryLayout::new(1, 2)?, 11)?;
    let audit = audit_associative_projection(&instance, &memory)?;
    if audit.generator_class_reuses() == audit.physical_replacement_collisions() {
        return Err("generator metadata unexpectedly collapsed into physical collision count".into());
    }

    println!("TDI-8.1 task encoding preflight: PASS");
    println!("scope=bounded_preflight_only");
    println!("query_target_arm_surface=ABSENT");
    println!("source_index_arm_surface=ABSENT");
    println!("collision_class_arm_feature=ABSENT");
    println!("physical_projection_audit=SEPARATE_RUNNER_DIAGNOSTIC");
    println!("final_holdout=DOES_NOT_EXIST");
    println!("tdi8_2_surface=ABSENT");
    Ok(())
}
