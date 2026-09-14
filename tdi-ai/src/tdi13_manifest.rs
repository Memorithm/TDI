//! Machine-readable-ish deterministic run manifest primitives for TDI-13 bootstrap.

use crate::tdi13::ArchitectureArm;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunManifest {
    pub arm: ArchitectureArm,
    pub sequence_len: u32,
    pub state_width_bits: u16,
    pub memory_slots: u16,
    pub seed: u64,
}

impl RunManifest {
    pub const fn new(
        arm: ArchitectureArm,
        sequence_len: u32,
        state_width_bits: u16,
        memory_slots: u16,
        seed: u64,
    ) -> Self {
        Self {
            arm,
            sequence_len,
            state_width_bits,
            memory_slots,
            seed,
        }
    }
}
