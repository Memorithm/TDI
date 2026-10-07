//! P5 Validation-only preflight. No realized timings are produced.

use tdi_ai::tdi21_pascal_p4::Split;
use tdi_ai::tdi21_pascal_p5::{
    canonical_plan_records, declared_cell_count, declared_widths, verify_declared_semantics,
};

#[test]
fn validation_geometry_remains_frozen() {
    assert_eq!(declared_widths(Split::Validation), &[15]);
    assert_eq!(declared_cell_count(Split::Validation), 54);
    let records = canonical_plan_records(Split::Validation).expect("Validation plan");
    assert_eq!(records.len(), 54);
    assert!(
        records
            .iter()
            .all(|record| record.starts_with("validation\t15\t"))
    );
}

#[test]
fn validation_semantics_are_exact_before_timing() {
    assert_eq!(
        verify_declared_semantics(Split::Validation).expect("Validation semantic preflight"),
        54
    );
}
