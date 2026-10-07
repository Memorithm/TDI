//! Emit the frozen TDI-21 Pascal P5 Development case plan.
//!
//! This executable constructs no Validation or protected/final evidence.

use tdi_ai::tdi21_pascal_p4::Split;
use tdi_ai::tdi21_pascal_p5::{canonical_plan_header, canonical_plan_records};

fn main() {
    let records = canonical_plan_records(Split::Development).expect("frozen P5 Development plan");
    assert_eq!(records.len(), 108);
    println!("{}", canonical_plan_header());
    for record in records {
        println!("{}", record);
    }
}
