//! Evidence record for deterministic TDI-13 development runs.

use crate::tdi13::ResourceCounters;
use crate::tdi13_manifest::RunManifest;
use crate::tdi13_scaling::ScalingCounters;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EvidenceRecord {
    pub manifest: RunManifest,
    pub correct: bool,
    pub resources: ResourceCounters,
    pub scaling: ScalingCounters,
}

impl EvidenceRecord {
    pub const fn candidate_structurally_valid(self) -> bool {
        self.scaling.candidate_is_structurally_pairwise_free()
    }
}
