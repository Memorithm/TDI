use tdi_ai::vsa_workspace::{BoundedVsaWorkspace, VsaWorkspaceLayout};

#[test]
fn vsa_workspace_is_available_through_public_tdi_ai_api() {
    let layout = VsaWorkspaceLayout::new(4).expect("valid public VSA layout");
    let mut workspace =
        BoundedVsaWorkspace::new(layout, 0x1234_5678_9abc_def0).expect("bounded workspace");
    let payload = [1.0, -2.0, 3.0, -4.0];

    workspace.bundle(17, &payload).expect("public bundle");
    let retrieved = workspace.unbind(17).expect("public unbind");
    assert_eq!(retrieved, payload);

    let accounting = workspace.storage_accounting().expect("public accounting");
    assert_eq!(accounting.workspace_bits().get(), 256);
    assert_eq!(accounting.temporary_working_bits().get(), 256);
    assert_eq!(accounting.static_parameter_bits().get(), 320);
}
