use tdi_ai::associative_memory::AssociativeMemoryLayout;
use tdi_ai::assr_h_reference::A3VsaReadRoute;
use tdi_ai::assr_reference::{A2Reference, RecurrentLayout, RecurrentParameters};
use tdi_ai::full_history_reference::FullHistoryLayout;
use tdi_ai::reference_operation_accounting::ReferenceOperationAccounting;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let recurrent_layout = RecurrentLayout::new(2, 2)?;
    let parameters = RecurrentParameters::new(
        recurrent_layout,
        vec![1.0, 0.0, 0.0, 1.0],
        vec![0.0; 4],
        vec![0.0; 2],
    )?;
    let mut a2 = A2Reference::new(parameters, AssociativeMemoryLayout::new(8, 2)?, 7, 1.0)?;
    a2.step(&[0.25, -0.25], 99, Some(17))?;
    let report = a2.step(&[0.0, 0.0], 17, Some(17))?;

    let a2_count = ReferenceOperationAccounting::a2_step(recurrent_layout, report)?;
    let a3_read = ReferenceOperationAccounting::a3_routed_step(
        recurrent_layout,
        A3VsaReadRoute::Key(17),
        report,
    )?;
    let a3_store = ReferenceOperationAccounting::a3_skip_and_store_step(recurrent_layout, report)?;
    let a0_layout = FullHistoryLayout::new(3, 2)?;
    let a0_append = ReferenceOperationAccounting::a0_append(a0_layout)?;
    let a0_read = ReferenceOperationAccounting::a0_read(a0_layout, 4)?;
    let a0_combined = a0_append.checked_add(a0_read)?;

    assert_eq!(a2_count.recurrent_mac_terms(), 8);
    assert_eq!(a2_count.activation_terms(), 4);
    assert_eq!(a2_count.associative_address_projections(), 2);
    assert_eq!(a2_count.associative_payload_fusions(), 2);
    assert_eq!(a2_count.associative_payload_stores(), 2);
    assert_eq!(a2_count.vsa_bind_terms(), 0);
    assert_eq!(a2_count.vsa_bundle_terms(), 0);
    assert_eq!(a2_count.vsa_unbind_terms(), 0);
    assert_eq!(a2_count.vsa_input_fusions(), 0);
    assert_eq!(a2_count.history_distance_terms(), 0);
    assert_eq!(a2_count.history_selection_comparisons(), 0);
    assert_eq!(a2_count.history_scalar_stores(), 0);
    assert_eq!(a2_count.total()?, 18);

    assert_eq!(a3_read.vsa_unbind_terms(), 2);
    assert_eq!(a3_read.vsa_input_fusions(), 2);
    assert_eq!(a3_read.total()?, 22);

    assert_eq!(a3_store.vsa_bind_terms(), 2);
    assert_eq!(a3_store.vsa_bundle_terms(), 2);
    assert_eq!(a3_store.total()?, 22);

    assert_eq!(a0_append.history_scalar_stores(), 5);
    assert_eq!(a0_append.total()?, 5);
    assert_eq!(a0_read.history_distance_terms(), 12);
    assert_eq!(a0_read.history_selection_comparisons(), 4);
    assert_eq!(a0_read.total()?, 16);
    assert_eq!(a0_combined.total()?, 21);

    println!("TDI-8.1 semantic operation accounting preflight: PASS");
    println!("A2 hit+write semantic units: {}", a2_count.total()?);
    println!("A3 keyed-read semantic units: {}", a3_read.total()?);
    println!("A3 atomic-store semantic units: {}", a3_store.total()?);
    println!("A0 append semantic units: {}", a0_append.total()?);
    println!(
        "A0 four-item/width-three read semantic units: {}",
        a0_read.total()?
    );
    println!("A0 append+read semantic units: {}", a0_combined.total()?);
    println!("hardware performance claim: ABSENT");
    println!("TDI-8.2 surface: ABSENT");
    Ok(())
}
