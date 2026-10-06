mod cell;
mod enums;
mod format;

pub use cell::Cell;
pub use enums::{Error, Schedule, Split};

pub const SEMANTICS: &str = "tdi21-pascal-p3-v1";
pub(crate) const GATES: usize = 8;
