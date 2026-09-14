//! Structural scaling guards for TDI-13 candidates.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ScalingCounters {
    pub sequence_items: u64,
    pub pairwise_comparisons: u64,
}

impl ScalingCounters {
    pub const fn candidate_is_structurally_pairwise_free(self) -> bool {
        self.pairwise_comparisons == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidate_guard_rejects_pairwise_work() {
        assert!(ScalingCounters {
            sequence_items: 1024,
            pairwise_comparisons: 0,
        }
        .candidate_is_structurally_pairwise_free());

        assert!(!ScalingCounters {
            sequence_items: 1024,
            pairwise_comparisons: 1,
        }
        .candidate_is_structurally_pairwise_free());
    }
}
