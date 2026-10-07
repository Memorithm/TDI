//! Prospective TDI-21 Pascal P5 realized-cost benchmark support.
//!
//! P5 is Development/Validation only. This module freezes timing-order and
//! summary semantics before any scientific timing execution. The matched
//! semantic workload remains owned by the already-frozen P4 harness.

use crate::tdi21_pascal_p4::Split;

pub const SEMANTICS: &str = "tdi21-pascal-p5-realized-cost-v1";
pub const WARMUP_ROUNDS: usize = 1;
pub const MEASURED_ROUNDS: usize = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arm {
    Pascal,
    Direct,
    Generic,
}

impl Arm {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Pascal => "pascal",
            Self::Direct => "direct",
            Self::Generic => "generic",
        }
    }
}

pub const fn timing_order(round: usize) -> [Arm; 3] {
    match round % 3 {
        0 => [Arm::Pascal, Arm::Direct, Arm::Generic],
        1 => [Arm::Direct, Arm::Generic, Arm::Pascal],
        _ => [Arm::Generic, Arm::Pascal, Arm::Direct],
    }
}

pub fn median_ns_six(mut samples: [u128; MEASURED_ROUNDS]) -> u128 {
    samples.sort_unstable();
    (samples[2] + samples[3]) / 2
}

pub const fn declared_cell_count(split: Split) -> usize {
    match split {
        Split::Development => 108,
        Split::Validation => 54,
    }
}

pub const fn declared_widths(split: Split) -> &'static [u8] {
    match split {
        Split::Development => &[10, 12],
        Split::Validation => &[15],
    }
}
