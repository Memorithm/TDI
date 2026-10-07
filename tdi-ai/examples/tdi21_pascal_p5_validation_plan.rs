//! Emit the frozen TDI-21 Pascal P5 Validation case plan.
//!
//! This preflight executable emits plan bytes only. It performs no timing and
//! has no protected/final path.

use tdi_ai::tdi21_pascal_p4::Split;
use tdi_ai::tdi21_pascal_p5::{canonical_plan_header, canonical_plan_records};

fn main() {
    let records = canonical_plan_records(Split::Validation).expect("frozen P5 Validation plan");
    assert_eq!(records.len(), 54);
    println!("{}", canonical_plan_header());
    for record in records {
        println!("{}", record);
    }
}
