//! Emit canonical TDI-21 Pascal P5 Development realized-cost evidence.
//!
//! Development-only executable. It has no Validation or protected/final path.

use tdi_ai::tdi21_pascal_p4::Split;
use tdi_ai::tdi21_pascal_p5::{canonical_header, run_timing_matrix};

fn main() {
    let cells = run_timing_matrix(Split::Development).expect("frozen P5 Development matrix");
    assert_eq!(cells.len(), 108);
    println!("{}", canonical_header());
    for cell in cells {
        println!("{}", cell.canonical_record());
    }
}
