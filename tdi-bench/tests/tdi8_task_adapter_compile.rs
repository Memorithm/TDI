use tdi_ai::associative_memory::{AssociativeMemoryLayout, DirectMappedAssociativeMemory};
use tdi_ai::task_generators::{T1Config, T3Config, generate_t1, generate_t3};
use tdi_bench::task_adapter_v8::{
    ExactU64Binary64, MIN_TASK_EVENT_INPUT_WIDTH, TaskAdapterLayout,
    audit_associative_projection, build_task_adapter_plan,
};

#[test]
fn downstream_crate_can_build_lossless_shared_task_schedule() {
    let instance = generate_t1(17, T1Config::new(5, 3, 2).expect("T1 config")).expect("T1");
    let layout = TaskAdapterLayout::new(MIN_TASK_EVENT_INPUT_WIDTH).expect("adapter layout");
    let plan = build_task_adapter_plan(&instance, layout).expect("adapter plan");

    assert_eq!(plan.events().len(), instance.events().len());
    assert!(plan.events().iter().all(|event| {
        event.recurrent_input().len() == MIN_TASK_EVENT_INPUT_WIDTH as usize
            && event.recurrent_input().iter().all(|value| value.is_finite())
    }));
    assert_eq!(
        ExactU64Binary64::decode(ExactU64Binary64::encode(u64::MAX).coordinates())
            .expect("lossless codec"),
        u64::MAX
    );
}

#[test]
fn downstream_crate_can_measure_physical_projection_separately_from_t3_classes() {
    let instance =
        generate_t3(41, T3Config::new(8, 3, 4, 20, 3).expect("T3 config")).expect("T3");
    let plan = build_task_adapter_plan(
        &instance,
        TaskAdapterLayout::new(MIN_TASK_EVENT_INPUT_WIDTH).expect("adapter layout"),
    )
    .expect("adapter plan");
    let memory = DirectMappedAssociativeMemory::new(
        AssociativeMemoryLayout::new(1, 2).expect("memory layout"),
        11,
    )
    .expect("memory");
    let audit = audit_associative_projection(&plan, &memory).expect("projection audit");

    assert_eq!(audit.generator_class_reuses(), 5);
    assert_eq!(audit.physical_replacement_collisions(), 7);
    assert_eq!(audit.distinct_occupied_addresses(), 1);
}
