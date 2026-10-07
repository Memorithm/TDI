//! Emit canonical TDI-21 Pascal P3 Development evidence.
//!
//! Development-only executable. It does not execute the frozen Validation
//! matrix and has no protected/final execution path.

use tdi_ai::tdi21_pascal_p3::{canonical_header, run_pascal_p3_development_matrix};

fn main() {
    let cells = run_pascal_p3_development_matrix().expect("frozen P3 Development matrix");
    assert_eq!(cells.len(), 108);
    println!("{}", canonical_header());
    for cell in cells {
        println!("{}", cell.canonical_record());
    }
}
