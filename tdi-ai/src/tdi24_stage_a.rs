//! TDI-24 Slice 10 Stage-A contract freeze surface.
//!
//! This module binds the semantic contracts qualified by slices 01-09 into one
//! explicit Stage-A identity. It does not execute a research population and it
//! does not assert that C6 outperforms V6.

use super::tdi24_accounting::REFERENCE_ACCOUNTING_CONTRACT;
use super::tdi24_attention::{MASKING_CONTRACT, NORMALIZER_CONTRACT};
use super::tdi24_chiral::{
    CHANNEL_DECOMPOSITION_CONTRACT, CHIRAL_CONTRACT, ENANTIOMORPHIC_SCORE_CONTRACT,
    PARITY_RECOMBINATION_CONTRACT,
};
use super::tdi24_vector::VECTOR6_CONTRACT;

/// Versioned identity of the complete TDI-24 Stage-A semantics.
pub const STAGE_A_CONTRACT: &str = "tdi24-stage-a-v1";

/// Immutable identifiers of every semantic surface admitted by Stage A.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StageAContracts {
    /// Stage-A aggregate contract.
    pub stage_a: &'static str,
    /// Mirror-coupled C6 algebra contract.
    pub chiral: &'static str,
    /// Matched V6 vector-reference contract.
    pub vector: &'static str,
    /// Primitive s/m/chi decomposition contract.
    pub decomposition: &'static str,
    /// Right/left enantiomorphic pair contract.
    pub enantiomorphic: &'static str,
    /// Even/odd recombination contract.
    pub recombination: &'static str,
    /// Shared deterministic normalizer contract.
    pub normalizer: &'static str,
    /// Shared causal/full masking contract.
    pub masking: &'static str,
    /// Source-level operation/storage accounting contract.
    pub accounting: &'static str,
}

/// Return the exact contracts admitted by TDI-24 Stage A.
#[must_use]
pub const fn stage_a_contracts() -> StageAContracts {
    StageAContracts {
        stage_a: STAGE_A_CONTRACT,
        chiral: CHIRAL_CONTRACT,
        vector: VECTOR6_CONTRACT,
        decomposition: CHANNEL_DECOMPOSITION_CONTRACT,
        enantiomorphic: ENANTIOMORPHIC_SCORE_CONTRACT,
        recombination: PARITY_RECOMBINATION_CONTRACT,
        normalizer: NORMALIZER_CONTRACT,
        masking: MASKING_CONTRACT,
        accounting: REFERENCE_ACCOUNTING_CONTRACT,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::experimental::tdi24_chiral::CHIRAL_WIDTH;
    use crate::experimental::tdi24_vector::VECTOR6_WIDTH;

    #[test]
    fn stage_a_contract_set_is_exact_and_versioned() {
        let contracts = stage_a_contracts();
        assert_eq!(contracts.stage_a, "tdi24-stage-a-v1");
        assert_eq!(contracts.chiral, "tdi24-mirror-coupled-chiral-v1");
        assert_eq!(contracts.vector, "tdi24-matched-vector6-v1");
        assert_eq!(contracts.decomposition, "tdi24-channel-decomposition-v1");
        assert_eq!(
            contracts.enantiomorphic,
            "tdi24-enantiomorphic-score-pair-v1"
        );
        assert_eq!(contracts.recombination, "tdi24-parity-recombination-v1");
        assert_eq!(contracts.normalizer, "tdi24-masked-softmax-reference-v1");
        assert_eq!(contracts.masking, "tdi24-attention-mask-reference-v1");
        assert_eq!(contracts.accounting, "tdi24-reference-accounting-v1");
    }

    #[test]
    fn stage_a_retains_the_matched_six_component_budget() {
        assert_eq!(VECTOR6_WIDTH, CHIRAL_WIDTH);
        assert_eq!(VECTOR6_WIDTH, 6);
    }
}
