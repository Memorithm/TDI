mod cell;
mod enums;
mod format;

pub use cell::Cell;
pub use enums::{Error, Schedule, Split};
pub use format::canonical_header;

pub const SEMANTICS: &str = "tdi21-pascal-p3-v1";
pub(crate) const GATES: usize = 8;
