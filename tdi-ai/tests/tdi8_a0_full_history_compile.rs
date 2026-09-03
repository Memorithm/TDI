use tdi_ai::ReferenceArm;
use tdi_ai::full_history_reference::{A0Reference, FullHistoryLayout};

#[test]
fn downstream_crate_can_build_and_use_the_a0_full_history_reference() {
    let layout = FullHistoryLayout::new(2, 1).expect("full-history layout");
    let mut model = A0Reference::new(layout).expect("A0 reference");

    model
        .append(&[1.0, 0.0], &[11.0])
        .expect("append first item");
    model
        .append(&[0.0, 1.0], &[22.0])
        .expect("append second item");

    let readout = model.read(&[0.1, 0.9]).expect("content read");
    assert_eq!(readout.selected_index(), 1);
    assert_eq!(readout.coefficients(), &[0.0, 1.0]);
    assert_eq!(readout.value(), &[22.0]);

    let accounting = model.memory_accounting().expect("A0 accounting");
    accounting
        .validate_for_arm(ReferenceArm::A0)
        .expect("valid A0 accounting");
    assert!(accounting.cumulative_history().get() > 0);

    let snapshot = model.snapshot().expect("A0 snapshot");
    assert_eq!(snapshot.arm(), ReferenceArm::A0);
    assert_eq!(snapshot.state().item_count(), 2);
}
