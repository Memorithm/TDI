#![cfg(feature = "experimental")]

//! Cross-contract leakage/adversarial audit for the TDI-24 Stage-B data surface.
//!
//! These tests qualify software semantics only. They do not train or evaluate a
//! model and do not authorize protected/final data, performance, or scientific
//! claims.

use tdi_ai::experimental::tdi24_tasks::{
    DATASET_CANONICALIZATION_CONTRACT, DIFFICULTY_STRATA_CONTRACT, DIRECTION_REVERSAL_CONTRACT,
    DataSplit, NON_CHIRAL_CONTROL_CONTRACT, PROTECTED_LABEL_CONTRACT,
    REFLECTION_DISCRIMINATIVE_CONTRACT, REFLECTION_NUISANCE_CONTRACT, SEED_REGISTRY_CONTRACT,
    SPLIT_MANIFEST_CONTRACT, SeedDomain, TaskFamily, assert_seed_domain_disjointness,
    canonicalize_inference_view, difficulty_stratum, direction_reversal_pair,
    non_chiral_control_case, reflection_discriminative_pair, reflection_nuisance_pair,
    register_seed, run_inference_callback, seal_direction_reversal, seal_non_chiral_control,
    seal_reflection_discriminative, seal_reflection_nuisance,
};

#[test]
fn inference_callbacks_cannot_observe_sealed_targets() {
    let labeled = seal_reflection_discriminative(&reflection_discriminative_pair(9).unwrap().right);
    let rendered = run_inference_callback(&labeled, |view| format!("{view:?}"));
    assert!(!rendered.contains("target"));
    assert!(!rendered.contains("Right"));
    assert!(!rendered.contains("Left"));
    assert_eq!(
        labeled.inference_view().label_contract,
        PROTECTED_LABEL_CONTRACT
    );
}

#[test]
fn declared_seed_domains_remain_disjoint_for_all_phase_b_families() {
    let families = [
        TaskFamily::ReflectionDiscriminative,
        TaskFamily::ReflectionNuisance,
        TaskFamily::DirectionReversal,
        TaskFamily::NonChiralControl,
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
    let development = seal_direction_reversal(&direction_reversal_pair(4).unwrap().forward);
    let validation = {
        use tdi_ai::experimental::tdi24_tasks::direction_reversal_pair_in_split;
        seal_direction_reversal(
            &direction_reversal_pair_in_split(4, DataSplit::Validation)
                .unwrap()
                .forward,
        )
    };
    let left = canonicalize_inference_view(development.inference_view());
    let right = canonicalize_inference_view(validation.inference_view());
    assert_eq!(
        left,
        canonicalize_inference_view(development.inference_view())
    );
    assert_ne!(left.digest, right.digest);
    assert!(!left.record.contains("target"));
    assert_eq!(left.contract, DATASET_CANONICALIZATION_CONTRACT);
}

#[test]
fn phase_b_contract_pins_and_fail_closed_split_labels_are_explicit() {
    assert_eq!(
        REFLECTION_DISCRIMINATIVE_CONTRACT,
        "tdi24-reflection-discriminative-generator-v1"
    );
    assert_eq!(
        REFLECTION_NUISANCE_CONTRACT,
        "tdi24-reflection-nuisance-generator-v1"
    );
    assert_eq!(
        DIRECTION_REVERSAL_CONTRACT,
        "tdi24-direction-reversal-generator-v1"
    );
    assert_eq!(
        NON_CHIRAL_CONTROL_CONTRACT,
        "tdi24-non-chiral-control-generator-v1"
    );
    assert_eq!(DIFFICULTY_STRATA_CONTRACT, "tdi24-difficulty-strata-v1");
    assert_eq!(SPLIT_MANIFEST_CONTRACT, "tdi24-split-manifest-v1");
    assert_eq!(PROTECTED_LABEL_CONTRACT, "tdi24-protected-label-api-v1");
    assert_eq!(SEED_REGISTRY_CONTRACT, "tdi24-seed-registry-v1");
    assert_eq!(
        DATASET_CANONICALIZATION_CONTRACT,
        "tdi24-dataset-canonicalization-v1"
    );

    assert!(DataSplit::parse("protected").is_err());
    assert!(DataSplit::parse("final").is_err());
    assert!(SeedDomain::parse("protected").is_err());
    assert!(SeedDomain::parse("final").is_err());

    let _ = reflection_nuisance_pair(0).unwrap();
    let _ = non_chiral_control_case(0).unwrap();
    let _ = seal_reflection_nuisance(&reflection_nuisance_pair(1).unwrap().canonical);
    let _ = seal_non_chiral_control(&non_chiral_control_case(1).unwrap());
    assert!(difficulty_stratum(0).level.level <= 3);
}

#[test]
fn stage_b_freeze_manifest_remains_non_authorizing() {
    let manifest = include_str!("../../docs/tdi24-stage-b-freeze.yaml");
    assert!(manifest.contains("stage: B"));
    assert!(manifest.contains("status: candidate_pending_exact_head_qualification_and_merge"));
    assert!(manifest.contains("protected_or_final_access: false"));
    assert!(manifest.contains("scientific_claim: false"));
    assert!(manifest.contains("training: false"));
    assert!(manifest.contains("tdi24-protected-label-api-v1"));
    assert!(manifest.contains("tdi24-seed-registry-v1"));
    assert!(manifest.contains("tdi24-dataset-canonicalization-v1"));
}
