use tdi_ai::ReferenceArm;
use tdi_ai::associative_memory::AssociativeMemoryLayout;
use tdi_ai::assr_h_reference::A3Reference;
use tdi_ai::assr_reference::{RecurrentLayout, RecurrentParameters};
use tdi_ai::vsa_workspace::VsaWorkspaceLayout;

#[test]
fn downstream_crate_can_build_and_use_the_bounded_a3_reference() {
    let layout = RecurrentLayout::new(1, 1).expect("recurrent layout");
    let parameters = RecurrentParameters::new(layout, vec![1.0], vec![0.0], vec![0.0])
        .expect("recurrent parameters");
    let memory = AssociativeMemoryLayout::new(1, 1).expect("associative layout");
    let vsa = VsaWorkspaceLayout::new(1).expect("VSA layout");
    let mut model = A3Reference::new(parameters, memory, 11, 1.0, vsa, 13, 0.5).expect("A3");

    model.store_vsa(7, &[0.5]).expect("VSA store");
    model.step(&[0.25], 7, Some(7)).expect("A3 step");

    let accounting = model.memory_accounting().expect("A3 accounting");
    accounting
        .validate_for_arm(ReferenceArm::A3)
        .expect("valid A3 accounting");
    assert!(accounting.vsa_workspace().get() > 0);

    let snapshot = model.snapshot().expect("A3 snapshot");
    assert_eq!(snapshot.arm(), ReferenceArm::A3);
    assert_eq!(snapshot.state().a2().recurrent_state(), model.state());
    assert_eq!(
        snapshot.state().vsa_workspace(),
        model.workspace().components()
    );
}
