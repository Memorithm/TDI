#[cfg(test)]
mod guard_tests {
    use crate::tdi13_contract::*;

    #[test]
    fn candidate_contract_forbids_attention_primitives() {
        assert!(CANDIDATE_FORBIDS_QKV);
        assert!(CANDIDATE_FORBIDS_PAIRWISE_SCORES);
        assert!(CANDIDATE_FORBIDS_SOFTMAX);
        assert!(CANDIDATE_FORBIDS_DENSE_RELATION_MATRIX);
    }
}
