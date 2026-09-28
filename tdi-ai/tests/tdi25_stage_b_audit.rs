#![cfg(feature = "experimental")]

//! Cross-contract leakage/adversarial audit for the TDI-25 Stage-B data surface.
//!
//! These tests qualify software semantics only. They do not train or evaluate a
//! model and do not authorize protected/final data, performance, or scientific
//! claims.

use tdi_ai::experimental::tdi25_tasks::{
    CHIRAL_REFLECTION_TASK_CONTRACT, DIFFICULTY_STRATA_CONTRACT, DataSplit,
    MIXED_GEOMETRY_TASK_CONTRACT, NEUTRAL_CONTROL_TASK_CONTRACT, POSITION_GEOMETRY_ARM_CONTRACT,
    PROTECTED_LABEL_CONTRACT, SEED_CASE_CANONICALIZATION_CONTRACT, SPLIT_MANIFEST_CONTRACT,
    SeedDomain, TORSOR_TRANSPORT_TASK_CONTRACT, assert_seed_domain_disjointness,
    canonicalize_chiral_reflection_input, canonicalize_mixed_geometry_input,
    canonicalize_neutral_control_input, canonicalize_torsor_transport_input,
    chiral_reflection_pair, chiral_reflection_pair_in_split, difficulty_stratum,
    mixed_geometry_pair, neutral_control_pair, register_seed, run_inference_callback,
    seal_chiral_reflection, seal_mixed_geometry, seal_neutral_control, seal_torsor_transport,
    torsor_transport_pair,
};
use tdi_ai::experimental::tdi25_torsor_chiral::TaskFamily;

#[test]
fn inference_callbacks_cannot_observe_sealed_oracles() {
    let pair = torsor_transport_pair(9).unwrap();
    let labeled = seal_torsor_transport(pair.original, pair.oracle);
    let rendered = run_inference_callback(&labeled, |input| format!("{input:?}"));
    assert!(!rendered.contains("expected_score"));
    assert!(!rendered.contains("oracle"));
    assert_eq!(labeled.label_contract(), PROTECTED_LABEL_CONTRACT);
}

#[test]
fn declared_seed_domains_remain_disjoint_for_all_phase_b_families() {
    let families = [
        TaskFamily::TorsorFavorable,
        TaskFamily::ChiralFavorable,
        TaskFamily::Mixed,
        TaskFamily::Neutral,
    ];
    let mut seeds = Vec::new();
    for family in families {
        for local in 0..128u64 {
            seeds.push(register_seed(SeedDomain::Development, family, local));
            seeds.push(register_seed(SeedDomain::Validation, family, local));
        }
    }
    assert_eq!(assert_seed_domain_disjointness(&seeds), Ok(()));
}

#[test]
fn canonical_digests_are_stable_and_split_sensitive_without_oracles() {
    let development = seal_chiral_reflection(
        chiral_reflection_pair(4).unwrap().right,
        chiral_reflection_pair(4).unwrap().right_oracle,
    );
    let validation = {
        let pair = chiral_reflection_pair_in_split(4, DataSplit::Validation).unwrap();
        seal_chiral_reflection(pair.right, pair.right_oracle)
    };
    let left = canonicalize_chiral_reflection_input(development.inference_input());
    let right = canonicalize_chiral_reflection_input(validation.inference_input());
    assert_eq!(
        left,
        canonicalize_chiral_reflection_input(development.inference_input())
    );
    assert_ne!(left.digest, right.digest);
    assert!(!left.record.contains("expected_score"));
    assert!(!left.record.contains("handedness"));
    assert_eq!(left.contract, SEED_CASE_CANONICALIZATION_CONTRACT);

    let _ = canonicalize_mixed_geometry_input(&mixed_geometry_pair(1).unwrap().base);
    let _ = canonicalize_neutral_control_input(&neutral_control_pair(1).unwrap().class_a);
    let _ = canonicalize_torsor_transport_input(&torsor_transport_pair(1).unwrap().original);
}

#[test]
fn phase_b_contract_pins_and_fail_closed_split_labels_are_explicit() {
    assert_eq!(
        TORSOR_TRANSPORT_TASK_CONTRACT,
        "tdi25-torsor-transport-task-v1"
    );
    assert_eq!(
        CHIRAL_REFLECTION_TASK_CONTRACT,
        "tdi25-chiral-reflection-task-v1"
    );
    assert_eq!(MIXED_GEOMETRY_TASK_CONTRACT, "tdi25-mixed-geometry-task-v1");
    assert_eq!(
        NEUTRAL_CONTROL_TASK_CONTRACT,
        "tdi25-neutral-control-task-v1"
    );
    assert_eq!(
        POSITION_GEOMETRY_ARM_CONTRACT,
        "tdi25-position-geometry-arm-v1"
    );
    assert_eq!(DIFFICULTY_STRATA_CONTRACT, "tdi25-difficulty-strata-v1");
    assert_eq!(SPLIT_MANIFEST_CONTRACT, "tdi25-split-manifest-v1");
    assert_eq!(PROTECTED_LABEL_CONTRACT, "tdi25-protected-label-api-v1");
    assert_eq!(
        SEED_CASE_CANONICALIZATION_CONTRACT,
        "tdi25-seed-case-canonicalization-v1"
    );

    assert!(DataSplit::parse("protected").is_err());
    assert!(DataSplit::parse("final").is_err());
    assert!(SeedDomain::parse("protected").is_err());
    assert!(SeedDomain::parse("final").is_err());

    let _ = seal_mixed_geometry(
        mixed_geometry_pair(2).unwrap().base,
        mixed_geometry_pair(2).unwrap().base_oracle,
    );
    let _ = seal_neutral_control(
        neutral_control_pair(2).unwrap().class_a,
        neutral_control_pair(2).unwrap().class_a_oracle,
    );
    assert!(difficulty_stratum(0).level.level <= 3);
}

#[test]
fn stage_b_freeze_manifest_remains_non_authorizing() {
    let manifest = include_str!("../../docs/tdi25-stage-b-freeze.yaml");
    assert!(manifest.contains("stage: B"));
    assert!(manifest.contains("status: candidate_pending_exact_head_qualification_and_merge"));
    assert!(manifest.contains("protected_or_final_access: false"));
    assert!(manifest.contains("scientific_claim: false"));
    assert!(manifest.contains("training: false"));
    assert!(manifest.contains("tdi25-protected-label-api-v1"));
    assert!(manifest.contains("tdi25-seed-case-canonicalization-v1"));
    assert!(manifest.contains("tdi25-split-manifest-v1"));
}
