//! Emit canonical TDI-21 Pascal P4 Validation evidence.
//!
//! This executable is frozen prospectively with the Development implementation.
//! The protocol requires committed Development evidence before it may be run.

use tdi_ai::tdi21_pascal_p4::{canonical_header, run_validation_matrix};

fn main() {
    let cells = run_validation_matrix().expect("frozen P4 Validation matrix");
    assert_eq!(cells.len(), 54);
    println!("{}", canonical_header());
    for cell in cells {
        println!("{}", cell.canonical_record());
    }
}