//! Non-executing TDI-11.2 Development/Validation population-derivation
//! qualification.
//!
//! Software-contract tests only. Do not pin freeze fields, load a model, or
//! authorize model/final execution.

#[path = "../src/hallucination_model_observation_population.rs"]
mod hallucination_model_observation_population;
#[allow(dead_code)]
#[path = "../src/hallucination_model_observation_rejections.rs"]
mod hallucination_model_observation_rejections;
#[allow(dead_code)]
#[path = "../src/hallucination_observation.rs"]
mod hallucination_observation;
#[allow(dead_code)]
#[path = "../src/hallucination_observation_adapter.rs"]
mod hallucination_observation_adapter;

use hallucination_model_observation_population::{
    AUTHORIZED_POPULATION_DOMAIN_COUNT, AUTHORIZED_POPULATION_DOMAIN_KEYS,
    CANDIDATE_POPULATION_DERIVATION_CONTRACT, FORBIDDEN_POPULATION_SURFACE_TOKENS,
    MODEL_OBSERVATION_POPULATION_DERIVATION_SCHEMA,
    MODEL_OBSERVATION_POPULATION_SEED_COMMITMENT_SCHEMA,
    ModelObservationPopulationDerivationContract, ModelObservationPopulationStratum,
    authorized_population_domains_match_prearm,
};
use hallucination_model_observation_rejections::{
    ALL_MODEL_OBSERVATION_REJECTION_CODES, CANDIDATE_PROVENANCE_SCHEMA,
    CANDIDATE_TYPED_REJECTION_VOCABULARY, ModelObservationDomain, ModelObservationRejectionCode,
};

fn sample_contract() -> ModelObservationPopulationDerivationContract {
    let strata = vec![
        ModelObservationPopulationStratum::new(
            ModelObservationDomain::Development,
            "dev_f1_shallow",
            "dev_seed_space_a",
        )
        .expect("dev stratum"),
        ModelObservationPopulationStratum::new(
            ModelObservationDomain::Validation,
            "val_f1_shallow",
            "val_seed_space_a",
        )
        .expect("val stratum"),
    ];
    ModelObservationPopulationDerivationContract::new(strata).expect("contract")
}

#[test]
fn candidate_identifier_is_stable_distinct_and_non_pinning() {
    assert_eq!(
        CANDIDATE_POPULATION_DERIVATION_CONTRACT,
        "ModelObservationPopulationDerivationContract"
    );
    assert_eq!(
        MODEL_OBSERVATION_POPULATION_DERIVATION_SCHEMA,
        "tdi11.2-model-observation-population-derivation-v0"
    );
    assert_eq!(
        MODEL_OBSERVATION_POPULATION_SEED_COMMITMENT_SCHEMA,
        "tdi11.2-model-observation-population-seed-commitment-v0"
    );
    assert_ne!(
        CANDIDATE_POPULATION_DERIVATION_CONTRACT,
        CANDIDATE_TYPED_REJECTION_VOCABULARY
    );
    assert_ne!(
        CANDIDATE_POPULATION_DERIVATION_CONTRACT,
        CANDIDATE_PROVENANCE_SCHEMA
    );
    assert_ne!(
        CANDIDATE_POPULATION_DERIVATION_CONTRACT,
        "ModelObservationChannelRegistry"
    );
    assert_ne!(
        CANDIDATE_POPULATION_DERIVATION_CONTRACT,
        "ModelObservationResourceAccountingContract"
    );
}

#[test]
fn authorized_domain_taxonomy_is_exact_two_keys_matching_prearm() {
    assert!(authorized_population_domains_match_prearm());
    assert_eq!(AUTHORIZED_POPULATION_DOMAIN_COUNT, 2);
    assert_eq!(
        AUTHORIZED_POPULATION_DOMAIN_KEYS,
        &["Development", "Validation"]
    );
    assert_eq!(
        ModelObservationPopulationDerivationContract::parse_domain("Development").unwrap(),
        ModelObservationDomain::Development
    );
    assert_eq!(
        ModelObservationPopulationDerivationContract::parse_domain("Validation").unwrap(),
        ModelObservationDomain::Validation
    );
    assert_eq!(
        ModelObservationPopulationDerivationContract::parse_domain("Final"),
        Err(ModelObservationRejectionCode::PopulationForbiddenDomain)
    );
    assert_eq!(
        ModelObservationPopulationDerivationContract::parse_domain("Holdout"),
        Err(ModelObservationRejectionCode::PopulationForbiddenDomain)
    );
}

#[test]
fn contract_construction_and_membership_are_exact_and_fail_closed() {
    let contract = sample_contract();
    assert_eq!(contract.len(), 2);
    assert!(!contract.is_empty());
    assert_eq!(contract.strata().len(), 2);
    assert_eq!(
        contract.covered_domain_keys(),
        ["Development", "Validation"]
    );
    let dev = contract
        .require_membership(ModelObservationDomain::Development, "dev_f1_shallow")
        .expect("dev membership");
    assert_eq!(dev.seed_space_key(), "dev_seed_space_a");
    assert_eq!(
        contract.require_membership(ModelObservationDomain::Validation, "dev_f1_shallow"),
        Err(ModelObservationRejectionCode::PopulationDomainMismatch)
    );
    assert_eq!(
        contract.lookup_stratum("missing"),
        Err(ModelObservationRejectionCode::PopulationUnknownStratum)
    );

    assert_eq!(
        ModelObservationPopulationDerivationContract::new(vec![]),
        Err(ModelObservationRejectionCode::PopulationEmpty)
    );
    assert_eq!(
        ModelObservationPopulationStratum::new(ModelObservationDomain::Development, "   ", "seed"),
        Err(ModelObservationRejectionCode::PopulationEmptyStratumId)
    );
    assert_eq!(
        ModelObservationPopulationStratum::new(ModelObservationDomain::Development, "stratum", ""),
        Err(ModelObservationRejectionCode::PopulationEmptySeedSpaceKey)
    );

    let only_dev = vec![
        ModelObservationPopulationStratum::new(
            ModelObservationDomain::Development,
            "dev_only",
            "dev_space",
        )
        .unwrap(),
    ];
    assert_eq!(
        ModelObservationPopulationDerivationContract::new(only_dev),
        Err(ModelObservationRejectionCode::PopulationDomainCoverageIncomplete)
    );

    let dup_stratum = vec![
        ModelObservationPopulationStratum::new(
            ModelObservationDomain::Development,
            "same",
            "dev_space",
        )
        .unwrap(),
        ModelObservationPopulationStratum::new(
            ModelObservationDomain::Validation,
            "same",
            "val_space",
        )
        .unwrap(),
    ];
    assert_eq!(
        ModelObservationPopulationDerivationContract::new(dup_stratum),
        Err(ModelObservationRejectionCode::PopulationDuplicateStratum)
    );

    let dup_seed = vec![
        ModelObservationPopulationStratum::new(
            ModelObservationDomain::Development,
            "dev_a",
            "shared_space",
        )
        .unwrap(),
        ModelObservationPopulationStratum::new(
            ModelObservationDomain::Development,
            "dev_b",
            "shared_space",
        )
        .unwrap(),
        ModelObservationPopulationStratum::new(
            ModelObservationDomain::Validation,
            "val_a",
            "val_space",
        )
        .unwrap(),
    ];
    assert_eq!(
        ModelObservationPopulationDerivationContract::new(dup_seed),
        Err(ModelObservationRejectionCode::PopulationDuplicateSeedSpace)
    );
}

#[test]
fn forbidden_tokens_and_final_material_leak_fail_closed() {
    assert!(FORBIDDEN_POPULATION_SURFACE_TOKENS.contains(&"final_seed_list"));
    assert!(FORBIDDEN_POPULATION_SURFACE_TOKENS.contains(&"final_population"));
    assert_eq!(
        ModelObservationPopulationStratum::new(
            ModelObservationDomain::Development,
            "uses_final_seed_list",
            "ok_space",
        ),
        Err(ModelObservationRejectionCode::PopulationForbiddenSurfaceToken)
    );
    let contract = sample_contract();
    assert_eq!(
        contract.refuse_final_material("final_dataset"),
        Err(ModelObservationRejectionCode::PopulationFinalMaterialLeak)
    );
    assert_eq!(
        contract.refuse_final_material("Final"),
        Err(ModelObservationRejectionCode::PopulationFinalMaterialLeak)
    );
    assert!(contract.refuse_final_material("dev_only_token").is_ok());
}

#[test]
fn seed_commitment_is_deterministic_domain_separated_and_fail_closed() {
    let contract = sample_contract();
    let a = contract
        .seed_commitment(
            ModelObservationDomain::Development,
            "dev_f1_shallow",
            "instance-1",
        )
        .expect("commitment");
    let b = contract
        .seed_commitment(
            ModelObservationDomain::Development,
            "dev_f1_shallow",
            "instance-1",
        )
        .expect("commitment");
    assert_eq!(a, b);
    assert!(a.starts_with(MODEL_OBSERVATION_POPULATION_SEED_COMMITMENT_SCHEMA));
    assert!(a.contains("domain:Development"));
    assert!(a.contains("stratum_id:14dev_f1_shallow"));
    assert!(a.contains("instance_key:10instance-1"));

    let val = contract
        .seed_commitment(
            ModelObservationDomain::Validation,
            "val_f1_shallow",
            "instance-1",
        )
        .expect("val commitment");
    assert_ne!(a, val);
    assert!(val.contains("domain:Validation"));

    assert_eq!(
        contract.seed_commitment(ModelObservationDomain::Development, "dev_f1_shallow", "   "),
        Err(ModelObservationRejectionCode::PopulationEmptySeedSpaceKey)
    );
    assert_eq!(
        contract.seed_commitment(
            ModelObservationDomain::Development,
            "dev_f1_shallow",
            "leaks_final_runner"
        ),
        Err(ModelObservationRejectionCode::PopulationForbiddenSurfaceToken)
    );
}

#[test]
fn canonical_record_frames_contract_and_display_names_candidate() {
    let contract = sample_contract();
    let record = contract.canonical_record();
    assert!(record.starts_with(MODEL_OBSERVATION_POPULATION_DERIVATION_SCHEMA));
    assert!(record.contains("stratum_count:2"));
    assert!(record.contains("stratum:14dev_f1_shallow:Development:"));
    assert!(record.contains("stratum:14val_f1_shallow:Validation:"));
    assert_eq!(
        format!("{contract}"),
        "ModelObservationPopulationDerivationContract[2]"
    );
}

#[test]
fn population_codes_are_present_unique_and_stable() {
    let expected = [
        (ModelObservationRejectionCode::PopulationEmpty, 0x0801),
        (
            ModelObservationRejectionCode::PopulationEmptyStratumId,
            0x0802,
        ),
        (
            ModelObservationRejectionCode::PopulationEmptySeedSpaceKey,
            0x0803,
        ),
        (
            ModelObservationRejectionCode::PopulationDuplicateStratum,
            0x0804,
        ),
        (
            ModelObservationRejectionCode::PopulationDuplicateSeedSpace,
            0x0805,
        ),
        (
            ModelObservationRejectionCode::PopulationDomainCoverageIncomplete,
            0x0806,
        ),
        (
            ModelObservationRejectionCode::PopulationForbiddenDomain,
            0x0807,
        ),
        (
            ModelObservationRejectionCode::PopulationForbiddenSurfaceToken,
            0x0808,
        ),
        (
            ModelObservationRejectionCode::PopulationUnknownStratum,
            0x0809,
        ),
        (
            ModelObservationRejectionCode::PopulationDomainMismatch,
            0x080A,
        ),
        (
            ModelObservationRejectionCode::PopulationFinalMaterialLeak,
            0x080B,
        ),
    ];
    for (code, numeric) in expected {
        assert!(ALL_MODEL_OBSERVATION_REJECTION_CODES.contains(&code));
        assert_eq!(code.numeric(), numeric);
        assert!(!code.as_str().is_empty());
        assert!(format!("{code}").contains(&format!("{numeric:#06x}")));
    }
}
