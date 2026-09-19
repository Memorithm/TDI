//! Cross-contract adversarial audit for the TDI-24 Stage-A reference surface.
//!
//! These tests qualify software semantics only. They do not train or evaluate a
//! model and do not authorize protected/final data, performance, or scientific
//! claims.

use tdi_ai::experimental::tdi24_accounting::{
    REFERENCE_ACCOUNTING_CONTRACT, ScoreArm, pair_score_accounting, row_accounting,
};
use tdi_ai::experimental::tdi24_attention::{
    MASKING_CONTRACT, NORMALIZER_CONTRACT, MaskPolicy, NormalizerError, masked_softmax,
    normalize_with_policy,
};
use tdi_ai::experimental::tdi24_chiral::{
    CHANNEL_DECOMPOSITION_CONTRACT, CHIRAL_CONTRACT, CHIRAL_WIDTH,
    ENANTIOMORPHIC_SCORE_CONTRACT, PARITY_RECOMBINATION_CONTRACT, Chiral6,
    ChiralScoreWeights, chiral_score, tagged_enantiomorphic_scores,
};
use tdi_ai::experimental::tdi24_vector::{
    VECTOR6_CONTRACT, VECTOR6_WIDTH, Vector6, vector6_score,
};

fn vector(data: [f64; 6]) -> Vector6 {
    Vector6::new(data).expect("finite V6 fixture")
}

fn chiral(data: [f64; 6]) -> Chiral6 {
    Chiral6::from_array(data).expect("finite C6 fixture")
}

fn close(lhs: f64, rhs: f64) {
    let scale = 1.0_f64.max(lhs.abs()).max(rhs.abs());
    assert!(
        (lhs - rhs).abs() <= 128.0 * f64::EPSILON * scale,
        "lhs={lhs:?}, rhs={rhs:?}"
    );
}

#[test]
fn direct_only_c6_is_exactly_v6_before_and_after_shared_normalization() {
    let query = [1.0, -2.0, 3.0, 0.5, -1.5, 2.5];
    let keys = [
        [-4.0, 1.0, 2.0, 3.0, 0.25, -0.75],
        [0.0, 1.0, -1.0, 2.0, 0.5, 4.0],
        [2.0, 0.25, -1.0, 0.5, 6.0, -2.0],
        [1.0; 6],
    ];
    let weights = ChiralScoreWeights::new(1.0, 0.0, 0.0).unwrap();
    let v6 = keys
        .iter()
        .map(|key| vector6_score(vector(query), vector(*key)).unwrap())
        .collect::<Vec<_>>();
    let c6 = keys
        .iter()
        .map(|key| chiral_score(chiral(query), chiral(*key), weights).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(v6, c6);

    for (policy, query_index) in [(MaskPolicy::Full, 0), (MaskPolicy::Causal, 2)] {
        assert_eq!(
            normalize_with_policy(&v6, policy, query_index).unwrap(),
            normalize_with_policy(&c6, policy, query_index).unwrap()
        );
    }
}

#[test]
fn reflection_swaps_enantiomorphic_branches_with_unchanged_contracts() {
    let query = chiral([1.0, 2.0, -3.0, 0.5, -1.5, 2.5]);
    let key = chiral([-4.0, 1.0, 2.0, 3.0, 0.25, -0.75]);
    let weights = ChiralScoreWeights::new(0.7, -0.2, 1.3).unwrap();
    let base = tagged_enantiomorphic_scores(query, key, weights).unwrap();
    let reflected =
        tagged_enantiomorphic_scores(query.mirror(), key.mirror(), weights).unwrap();

    close(reflected.right, base.left);
    close(reflected.left, base.right);
    assert_eq!(base.algebra_contract, CHIRAL_CONTRACT);
    assert_eq!(
        base.decomposition_contract,
        CHANNEL_DECOMPOSITION_CONTRACT
    );
    assert_eq!(base.pair_contract, ENANTIOMORPHIC_SCORE_CONTRACT);
}

#[test]
fn width_storage_mask_and_contract_versions_are_explicitly_frozen() {
    assert_eq!(VECTOR6_WIDTH, CHIRAL_WIDTH);
    assert_eq!(VECTOR6_CONTRACT, "tdi24-matched-vector6-v1");
    assert_eq!(CHIRAL_CONTRACT, "tdi24-mirror-coupled-chiral-v1");
    assert_eq!(
        CHANNEL_DECOMPOSITION_CONTRACT,
        "tdi24-channel-decomposition-v1"
    );
    assert_eq!(
        ENANTIOMORPHIC_SCORE_CONTRACT,
        "tdi24-enantiomorphic-score-pair-v1"
    );
    assert_eq!(
        PARITY_RECOMBINATION_CONTRACT,
        "tdi24-parity-recombination-v1"
    );
    assert_eq!(NORMALIZER_CONTRACT, "tdi24-masked-softmax-reference-v1");
    assert_eq!(MASKING_CONTRACT, "tdi24-attention-mask-reference-v1");
    assert_eq!(
        REFERENCE_ACCOUNTING_CONTRACT,
        "tdi24-reference-accounting-v3"
    );

    let v6 = pair_score_accounting(ScoreArm::V6);
    let c6 = pair_score_accounting(ScoreArm::C6);
    assert_eq!(v6.carrier_bytes, c6.carrier_bytes);
    assert_eq!(v6.query_scalars, c6.query_scalars);
    assert_eq!(v6.key_scalars, c6.key_scalars);

    let row = row_accounting(4, 3).unwrap();
    assert_eq!(row.mask_bytes, 4 * core::mem::size_of::<bool>());
    assert_eq!(
        row.normalizer_scratch_bytes,
        4 * core::mem::size_of::<f64>()
    );
    assert_eq!(row.validity_predicates, 2 * 4 + 3 * 3 + 5);
}

#[test]
fn malformed_or_hidden_nonfinite_values_fail_closed_across_both_arms() {
    let mut invalid = [0.0; 6];
    invalid[4] = f64::NAN;
    assert!(Vector6::new(invalid).is_err());
    assert!(Chiral6::from_array(invalid).is_err());

    let huge = [f64::MAX, 0.0, 0.0, 0.0, 0.0, 0.0];
    assert!(vector6_score(vector(huge), vector(huge)).is_err());
    assert!(chiral_score(
        chiral(huge),
        chiral(huge),
        ChiralScoreWeights::new(1.0, 0.0, 0.0).unwrap(),
    )
    .is_err());

    assert_eq!(
        masked_softmax(&[1.0, f64::NAN], &[true, false]),
        Err(NormalizerError::NonFiniteLogit)
    );
}
