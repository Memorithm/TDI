//! Emit canonical TDI-21 Pascal P5 Validation realized-cost evidence.
//!
//! Operational wrapper registered after Development and before any Validation
//! timing. It invokes the already-frozen P5 timing implementation unchanged.
//! No protected/final population is represented.

use tdi_ai::tdi21_pascal_p4::Split;
use tdi_ai::tdi21_pascal_p5::{canonical_header, run_timing_matrix};

fn main() {
    let cells = run_timing_matrix(Split::Validation).expect("frozen P5 Validation matrix");
    assert_eq!(cells.len(), 54);
    println!("{}", canonical_header());
    for cell in cells {
        println!("{}", cell.canonical_record());
    }
}
