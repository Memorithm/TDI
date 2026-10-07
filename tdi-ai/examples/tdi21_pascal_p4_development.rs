//! Emit canonical TDI-21 Pascal P4 Development evidence.
//!
//! Development-only executable. It has no Validation or protected/final path.

use tdi_ai::tdi21_pascal_p4::{canonical_header, run_development_matrix};

fn main() {
    let cells = run_development_matrix().expect("frozen P4 Development matrix");
    assert_eq!(cells.len(), 108);
    println!("{}", canonical_header());
    for cell in cells {
        println!("{}", cell.canonical_record());
    }
}