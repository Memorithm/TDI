#![forbid(unsafe_code)]

use tdi_ai::associative_memory::AssociativeMemoryLayout;
use tdi_ai::assr_reference::{A1Reference, A2Reference, RecurrentLayout, RecurrentParameters};
use tdi_ai::ReferenceArm;

fn identity_parameters() -> RecurrentParameters {
    let layout = RecurrentLayout::new(2, 2).expect("valid public layout");
    RecurrentParameters::new(
        layout,
        vec![1.0, 0.0, 0.0, 1.0],
        vec![0.0; 4],
        vec![0.0; 2],
    )
    .expect("valid public parameters")
}

#[test]
fn downstream_code_can_construct_and_snapshot_a1_a2() {
    let a1 = A1Reference::new(identity_parameters()).expect("public A1");
    assert_eq!(a1.snapshot().expect("A1 snapshot").arm(), ReferenceArm::A1);

    let memory = AssociativeMemoryLayout::new(2, 2).expect("public memory layout");
    let a2 = A2Reference::new(identity_parameters(), memory, 17, 1.0).expect("public A2");
    assert_eq!(a2.snapshot().expect("A2 snapshot").arm(), ReferenceArm::A2);
}
