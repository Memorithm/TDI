//! Emit canonical TDI-21 Pascal P3 Validation evidence.
//!
//! This executable exists prospectively so Development and Validation use the
//! same committed implementation identity. The P3 protocol requires frozen
//! Development evidence before this binary is executed.

use tdi_ai::tdi21_pascal_p3::{canonical_header, run_pascal_p3_validation_matrix};

fn main() {
    let cells = run_pascal_p3_validation_matrix().expect("frozen P3 Validation matrix");
    assert_eq!(cells.len(), 108);
    println!("{}", canonical_header());
    for cell in cells {
        println!("{}", cell.canonical_record());
    }
}
