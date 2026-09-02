// Compile the bounded A3 source against the public contracts already merged in
// `tdi-ai` without changing the crate's public export surface in the same commit.
// This keeps source/oracle review separable from the later API exposure commit.

pub use tdi_ai::{
    MemoryAccounting, MemoryAccountingError, ReferenceArm, ReferenceSnapshot, StorageBits,
};

pub mod associative_memory {
    pub use tdi_ai::associative_memory::*;
}

pub mod assr_reference {
    pub use tdi_ai::assr_reference::*;
}

pub mod vsa_workspace {
    pub use tdi_ai::vsa_workspace::*;
}

#[path = "../src/a3_reference.rs"]
mod a3_reference;

use a3_reference::{A3RecurrentParameters, A3Reference};
use associative_memory::AssociativeMemoryLayout;
use assr_reference::RecurrentLayout;
use vsa_workspace::VsaWorkspaceLayout;

#[test]
fn a3_source_compiles_and_runs_against_merged_public_contracts() {
    let layout = RecurrentLayout::new(2, 2).expect("valid recurrent layout");
    let parameters = A3RecurrentParameters::new(
        layout,
        vec![1.0, 0.0, 0.0, 1.0],
        vec![0.0, 0.0, 0.0, 0.0],
        vec![0.0, 0.0],
    )
    .expect("valid A3 parameters");
    let mut reference = A3Reference::new(
        parameters,
        AssociativeMemoryLayout::new(4, 2).expect("memory layout"),
        17,
        0.5,
        VsaWorkspaceLayout::new(2).expect("VSA layout"),
        29,
        0.5,
    )
    .expect("A3 reference");

    let report = reference
        .step(&[0.25, -0.5], 7, Some(7))
        .expect("bounded A3 step");
    assert!(report.vsa_bundled());
    reference
        .memory_accounting()
        .expect("A3 accounting")
        .validate_for_arm(ReferenceArm::A3)
        .expect("valid A3 accounting");
}
