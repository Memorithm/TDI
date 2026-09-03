#[path = "../../task_encoding.rs"]
pub mod task_encoding;
#[path = "../../task_readout.rs"]
pub mod task_readout;

pub use tdi_ai::{associative_memory, task_generators};

use std::error::Error;

use task_encoding::ExactU64Binary64;
use task_readout::{ExactStateReadoutLayout, ExactStateSymbolReadout};
use tdi_ai::task_generators::TaskSymbol;

fn main() -> Result<(), Box<dyn Error>> {
    let expected = TaskSymbol::new(0x0123_4567_89ab_cdef);
    let encoded = ExactU64Binary64::encode(expected.code()).coordinates();
    let state = [0.25, encoded[0], -0.5, encoded[1], 0.0];
    let readout = ExactStateSymbolReadout::new(ExactStateReadoutLayout::new(5, 1, 3)?);
    if readout.decode_state(&state)? != expected {
        return Err("exact target-blind recurrent readout round-trip failed".into());
    }

    if readout
        .decode_state(&[0.25, 0.1, -0.5, encoded[1], 0.0])
        .is_ok()
    {
        return Err("non-canonical recurrent readout unexpectedly rounded to a symbol".into());
    }

    println!("TDI-8.1 exact readout preflight: PASS");
    println!("scope=bounded_preflight_only");
    println!("readout_input=RECURRENT_STATE_ONLY");
    println!("target_input=ABSENT");
    println!("candidate_vocabulary=ABSENT");
    println!("rounding_or_tolerance=ABSENT");
    println!("readout_indices=CALLER_SUPPLIED_NO_DEFAULT");
    println!("final_holdout=DOES_NOT_EXIST");
    println!("tdi8_2_surface=ABSENT");
    Ok(())
}
