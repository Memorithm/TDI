#[path = "../../reference_operation_accounting.rs"]
mod reference_operation_accounting;

pub use tdi_ai::{associative_memory, assr_h_reference, assr_reference, full_history_reference};

use reference_operation_accounting::ReferenceOperationAccounting;
use tdi_ai::associative_memory::AssociativeMemoryLayout;
use tdi_ai::assr_h_reference::A3VsaReadRoute;
use tdi_ai::assr_reference::{A2Reference, RecurrentLayout, RecurrentParameters};
use tdi_ai::full_history_reference::FullHistoryLayout;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let recurrent_layout = RecurrentLayout::new(2, 2)?;
    let parameters = RecurrentParameters::new(
        recurrent_layout,
        vec![1.0, 0.0, 0.0, 1.0],
        vec![0.0; 4],
        vec![0.0; 2],
    )?;
    let mut a2 = A2Reference::new(
        parameters,
        AssociativeMemoryLayout::new(8, 2)?,
        7,
        1.0,
    )?;
    a2.step(&[0.25, -0.25], 99, Some(17))?;
    let report = a2.step(&[0.0, 0.0], 17, Some(17))?;

    let a2_count = ReferenceOperationAccounting::a2_step(recurrent_layout, report)?;
    let a3_read = ReferenceOperationAccounting::a3_routed_step(
        recurrent_layout,
        A3VsaReadRoute::Key(17),
        report,
    )?;
    let a3_store =
        ReferenceOperationAccounting::a3_skip_and_store_step(recurrent_layout, report)?;
    let a0_layout = FullHistoryLayout::new(3, 2)?;
    let a0_read = ReferenceOperationAccounting::a0_read(a0_layout, 4)?;

    assert_eq!(a2_count.total()?, 18);
    assert_eq!(a3_read.total()?, 22);
    assert_eq!(a3_store.total()?, 22);
    assert_eq!(a0_read.total()?, 16);

    println!("TDI-8.1 semantic operation accounting preflight: PASS");
    println!("A2 hit+write semantic units: {}", a2_count.total()?);
    println!("A3 keyed-read semantic units: {}", a3_read.total()?);
    println!("A3 atomic-store semantic units: {}", a3_store.total()?);
    println!("A0 four-item/width-three read semantic units: {}", a0_read.total()?);
    println!("hardware performance claim: ABSENT");
    println!("TDI-8.2 surface: ABSENT");
    Ok(())
}
