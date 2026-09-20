#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi22_torsor::{Torsor3, Vec3};
use tdi_ai::experimental::tdi24_attention::{
    MASKING_CONTRACT, MaskPolicy, NORMALIZER_CONTRACT, NormalizerError,
};
use tdi_ai::experimental::tdi24_chiral::{CHIRAL_CONTRACT, Chiral6};
use tdi_ai::experimental::tdi25_torsor_chiral::{
    CARRIER_ACCOUNTING_CONTRACT, CHIRAL_INVARIANT_BRIDGE_CONTRACT, COMPARISON_RECORD_CONTRACT,
    ComparisonArm, ComparisonBudget, ComparisonFailure, ComparisonOutcome, GENERIC6_CONTRACT,
    MASK_NORMALIZER_BRIDGE_CONTRACT, SCORE_SCALE_CONTRACT, SOURCE_CONTRACT_PIN_VERSION,
    TDI25_CONTRACT, TORSOR_INVARIANT_BRIDGE_CONTRACT, TaskFamily, carrier_accounting,
    comparison_record, normalize_arm_row, source_contracts, transported_torsor_invariants,
    validate_chiral_invariants,
};

fn v(x: f64, y: f64, z: f64) -> Vec3 {
    Vec3::new(x, y, z).unwrap()
}

fn c(data: [f64; 6]) -> Chiral6 {
    Chiral6::from_array(data).unwrap()
}

fn close(lhs: f64, rhs: f64) {
    let scale = 1.0_f64.max(lhs.abs()).max(rhs.abs());
    assert!((lhs - rhs).abs() <= 128.0 * f64::EPSILON * scale);
}

#[test]
fn stage_a_contract_versions_and_accounting_are_explicitly_frozen() {
    assert_eq!(TDI25_CONTRACT, "tdi25-torsor-vs-chiral-v1");
    assert_eq!(SOURCE_CONTRACT_PIN_VERSION, "tdi25-source-contract-pin-v1");
    assert_eq!(GENERIC6_CONTRACT, "tdi25-generic6-control-v1");
    assert_eq!(CARRIER_ACCOUNTING_CONTRACT, "tdi25-carrier-accounting-v1");
    assert_eq!(SCORE_SCALE_CONTRACT, "tdi25-common-score-scale-v1");
    assert_eq!(
        TORSOR_INVARIANT_BRIDGE_CONTRACT,
        "tdi25-torsor-invariant-bridge-v1"
    );
    assert_eq!(
        CHIRAL_INVARIANT_BRIDGE_CONTRACT,
        "tdi25-chiral-invariant-bridge-v1"
    );
    assert_eq!(
        MASK_NORMALIZER_BRIDGE_CONTRACT,
        "tdi25-shared-mask-normalizer-v1"
    );
    assert_eq!(COMPARISON_RECORD_CONTRACT, "tdi25-comparison-record-v1");

    let sources = source_contracts();
    assert_eq!(sources.torsor, "tdi22-torsor-dual-pairing-v1");
    assert_eq!(sources.chiral, CHIRAL_CONTRACT);

    let t6 = carrier_accounting(ComparisonArm::T6);
    let c6 = carrier_accounting(ComparisonArm::C6);
    let g6 = carrier_accounting(ComparisonArm::G6);
    assert_eq!(t6.query_components, 6);
    assert_eq!(c6.query_components, 6);
    assert_eq!(g6.query_components, 6);
    assert_eq!(t6.score_components, 1);
    assert_eq!(c6.score_components, 1);
    assert_eq!(g6.score_components, 1);
    assert_eq!(t6.key_components, 9);
    assert_eq!(c6.key_components, 6);
    assert_eq!(g6.key_components, 6);
    assert_eq!(t6.external_geometry_components, 3);
}

#[test]
fn torsor_and_chiral_invariants_survive_the_tdi25_bridges() {
    let key = Torsor3::new(v(2.0, 3.0, -4.0), v(-5.0, 7.0, 11.0), v(13.0, -17.0, 19.0)).unwrap();
    let (before, after) = transported_torsor_invariants(key, v(-2.0, 5.0, 7.0)).unwrap();
    close(before.resultant_norm_squared, after.resultant_norm_squared);
    close(before.scalar_invariant, after.scalar_invariant);
    close(before.origin_moment.x, after.origin_moment.x);
    close(before.origin_moment.y, after.origin_moment.y);
    close(before.origin_moment.z, after.origin_moment.z);

    let query = c([1.0, 2.0, -3.0, 0.5, -1.5, 2.5]);
    let key = c([-4.0, 1.0, 2.0, 3.0, 0.25, -0.75]);
    let snapshot = validate_chiral_invariants(query, key).unwrap();
    assert_eq!(snapshot.reflected.direct, snapshot.base.direct);
    assert_eq!(snapshot.reflected.mirrored, snapshot.base.mirrored);
    assert_eq!(snapshot.reflected.chiral, -snapshot.base.chiral);
}

#[test]
fn all_arms_share_mask_normalization_and_record_provenance() {
    let logits = [1.0, 2.0, 30.0, 40.0];
    let t6 = normalize_arm_row(ComparisonArm::T6, &logits, MaskPolicy::Causal, 1).unwrap();
    let c6 = normalize_arm_row(ComparisonArm::C6, &logits, MaskPolicy::Causal, 1).unwrap();
    let g6 = normalize_arm_row(ComparisonArm::G6, &logits, MaskPolicy::Causal, 1).unwrap();
    assert_eq!(t6.probabilities, c6.probabilities);
    assert_eq!(c6.probabilities, g6.probabilities);
    assert_eq!(t6.masking_contract, MASKING_CONTRACT);
    assert_eq!(t6.normalizer_contract, NORMALIZER_CONTRACT);
    assert_eq!(t6.bridge_contract, MASK_NORMALIZER_BRIDGE_CONTRACT);

    let record = comparison_record(
        TaskFamily::Neutral,
        ComparisonArm::G6,
        7,
        11,
        ComparisonBudget::new(32, 0).unwrap(),
        ComparisonOutcome::Failure(ComparisonFailure::Task),
    )
    .unwrap();
    assert_eq!(
        record.outcome,
        ComparisonOutcome::Failure(ComparisonFailure::Task)
    );
    assert_eq!(record.source_contracts, source_contracts());
    assert_eq!(record.record_contract, COMPARISON_RECORD_CONTRACT);
}

#[test]
fn malformed_rows_and_records_fail_closed() {
    for arm in [ComparisonArm::T6, ComparisonArm::C6, ComparisonArm::G6] {
        assert_eq!(
            normalize_arm_row(arm, &[], MaskPolicy::Full, 0).unwrap_err(),
            tdi_ai::experimental::tdi25_torsor_chiral::Tdi25Error::Normalizer(
                NormalizerError::EmptyInput
            )
        );
    }

    let budget = ComparisonBudget::new(1, 0).unwrap();
    assert!(
        comparison_record(
            TaskFamily::Mixed,
            ComparisonArm::C6,
            1,
            1,
            budget,
            ComparisonOutcome::Score(f64::NAN),
        )
        .is_err()
    );
}
