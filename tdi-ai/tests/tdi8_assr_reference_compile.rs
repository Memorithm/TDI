#![forbid(unsafe_code)]

pub use tdi_ai::{
    MemoryAccounting, MemoryAccountingError, ReferenceArm, ReferenceSnapshot, StorageBits,
};

pub mod associative_memory {
    pub use tdi_ai::associative_memory::*;
}

#[allow(dead_code)]
#[path = "../src/assr_reference.rs"]
mod assr_reference;
