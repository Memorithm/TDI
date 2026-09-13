//! Non-executing TDI-11.2 model-observation registry / timing / H11-A eligibility
//! qualification.
//!
//! Software-contract tests only. Do not pin freeze fields, load a model, or
//! authorize model/final execution.

#[path = "../src/hallucination_model_observation_registry.rs"]
mod hallucination_model_observation_registry;
#[allow(dead_code)]
#[path = "../src/hallucination_model_observation_rejections.rs"]
mod hallucination_model_observation_rejections;
#[allow(dead_code)]
#[path = "../src/hallucination_observation.rs"]
mod hallucination_observation;
#[allow(dead_code)]
#[path = "../src/hallucination_observation_adapter.rs"]
mod hallucination_observation_adapter;

use hallucination_model_observation_registry::{
    CANDIDATE_H11A_ELIGIBILITY_RULE, CANDIDATE_OBSERVATION_REGISTRY,
    CANDIDATE_OBSERVATION_TIMING_CONTRACT, INHERITED_OBSERVATION_SOURCE_CLASS_COUNT,
    INHERITED_OBSERVATION_SOURCE_CLASS_KEYS, MODEL_OBSERVATION_H11A_ELIGIBILITY_SCHEMA,
    MODEL_OBSERVATION_REGISTRY_SCHEMA, MODEL_OBSERVATION_TIMING_CONTRACT_SCHEMA,
    ModelObservationChannelEntry, ModelObservationChannelRegistry,
    ModelObservationChannelTimingPolicy, ModelObservationH11AEligibilityRule,
    ModelObservationTimingBinding, ModelObservationTimingContract,
};
use hallucination_model_observation_rejections::{
    ALL_MODEL_OBSERVATION_REJECTION_CODES, CANDIDATE_PROVENANCE_SCHEMA,
    CANDIDATE_TYPED_REJECTION_VOCABULARY, ModelObservationRejectionCode,
};
use hallucination_observation::PrecursorEligibility;
use hallucination_observation_adapter::ObservationSourceClass;

fn sample_registry() -> ModelObservationChannelRegistry {
    let entries = vec![
        ModelObservationChannelEntry::new(
            "decoder_entropy",
            ObservationSourceClass::DecoderStatistics,
        )
        .expect("entry"),
        ModelObservationChannelEntry::new(
            "hidden_norm",
            ObservationSourceClass::HiddenStateSummary,
        )
        .expect("entry"),
        ModelObservationChannelEntry::new(
            "visible_support",
            ObservationSourceClass::VisibleEvidenceSummary,
        )
        .expect("entry"),
    ];
    ModelObservationChannelRegistry::new(entries).expect("registry")
}

#[test]
fn candidate_identifiers_are_stable_distinct_and_non_pinning() {
    assert_eq!(
        CANDIDATE_OBSERVATION_REGISTRY,
        "ModelObservationChannelRegistry"
    );
    assert_eq!(
        CANDIDATE_OBSERVATION_TIMING_CONTRACT,
        "ModelObservationTimingContract"
    );
    assert_eq!(
        CANDIDATE_H11A_ELIGIBILITY_RULE,
        "ModelObservationH11AEligibilityRule"
    );
    assert_eq!(
        MODEL_OBSERVATION_REGISTRY_SCHEMA,
        "tdi11.2-model-observation-registry-v0"
    );
    assert_eq!(
        MODEL_OBSERVATION_TIMING_CONTRACT_SCHEMA,
        "tdi11.2-model-observation-timing-contract-v0"
    );
    assert_eq!(
        MODEL_OBSERVATION_H11A_ELIGIBILITY_SCHEMA,
        "tdi11.2-model-observation-h11a-eligibility-v0"
    );
    // Distinct from #213 / #216 candidates.
    assert_ne!(
        CANDIDATE_OBSERVATION_REGISTRY,
        CANDIDATE_TYPED_REJECTION_VOCABULARY
    );
    assert_ne!(CANDIDATE_OBSERVATION_REGISTRY, CANDIDATE_PROVENANCE_SCHEMA);
    assert_ne!(
        CANDIDATE_OBSERVATION_TIMING_CONTRACT,
        "ModelObservationResourceAccountingContract"
    );
    assert_ne!(
        CANDIDATE_H11A_ELIGIBILITY_RULE,
        CANDIDATE_OBSERVATION_REGISTRY
    );
}

#[test]
fn inherited_source_class_taxonomy_is_exact_eight_keys_matching_prearm() {
    assert_eq!(
        INHERITED_OBSERVATION_SOURCE_CLASS_KEYS.len(),
        INHERITED_OBSERVATION_SOURCE_CLASS_COUNT
    );
    assert_eq!(INHERITED_OBSERVATION_SOURCE_CLASS_COUNT, 8);
    assert_eq!(
        INHERITED_OBSERVATION_SOURCE_CLASS_KEYS,
        &[
            "decoder_statistics",
            "hidden_state_summary",
            "visible_evidence_summary",
            "verifier_result",
            "retrieval_tool_result",
            "action_history",
            "runtime_resource_summary",
            "resample_summary",
        ]
    );
    for class in [
        ObservationSourceClass::DecoderStatistics,
        ObservationSourceClass::HiddenStateSummary,
        ObservationSourceClass::VisibleEvidenceSummary,
        ObservationSourceClass::VerifierResult,
        ObservationSourceClass::RetrievalToolResult,
        ObservationSourceClass::ActionHistory,
        ObservationSourceClass::RuntimeResourceSummary,
        ObservationSourceClass::ResampleSummary,
    ] {
        assert!(INHERITED_OBSERVATION_SOURCE_CLASS_KEYS.contains(&class.as_str()));
    }
}

#[test]
fn registry_construction_and_lookup_are_exact_and_fail_closed() {
    let registry = sample_registry();
    assert_eq!(registry.len(), 3);
    assert!(!registry.is_empty());
    assert_eq!(registry.entries()[0].channel(), "decoder_entropy");
    assert_eq!(
        registry.source_class_of("decoder_entropy"),
        Ok(ObservationSourceClass::DecoderStatistics)
    );
    let framed = registry.canonical_record();
    assert!(framed.starts_with(MODEL_OBSERVATION_REGISTRY_SCHEMA));
    assert!(framed.contains("kind:registry"));
    assert!(framed.contains("entry_count:3"));
    assert!(framed.contains("channel:15decoder_entropy:class:decoder_statistics"));

    let empty = ModelObservationChannelRegistry::new(vec![]);
    assert_eq!(empty, Err(ModelObservationRejectionCode::RegistryEmpty));
    assert_eq!(
        ModelObservationChannelEntry::new(" ", ObservationSourceClass::ActionHistory),
        Err(ModelObservationRejectionCode::RegistryEmptyChannelName)
    );
    let dup = ModelObservationChannelRegistry::new(vec![
        ModelObservationChannelEntry::new("a", ObservationSourceClass::ActionHistory).unwrap(),
        ModelObservationChannelEntry::new("a", ObservationSourceClass::ResampleSummary).unwrap(),
    ]);
    assert_eq!(
        dup,
        Err(ModelObservationRejectionCode::RegistryDuplicateChannel)
    );
    assert_eq!(
        registry.source_class_of("missing"),
        Err(ModelObservationRejectionCode::RegistryUndeclaredChannel)
    );
}

#[test]
fn timing_contract_exact_coverage_and_admission_rules() {
    let registry = sample_registry();
    let bindings = vec![
        ModelObservationTimingBinding::new(
            "decoder_entropy",
            ModelObservationChannelTimingPolicy::RequireStrictlyBeforeAssertion,
        )
        .unwrap(),
        ModelObservationTimingBinding::new(
            "hidden_norm",
            ModelObservationChannelTimingPolicy::AllowPostHocDiagnostic,
        )
        .unwrap(),
        ModelObservationTimingBinding::new(
            "visible_support",
            ModelObservationChannelTimingPolicy::AllowPostHocDiagnostic,
        )
        .unwrap(),
    ];
    let contract =
        ModelObservationTimingContract::covering_registry(&registry, bindings).expect("contract");
    assert_eq!(contract.bindings().len(), 3);
    assert_eq!(contract.bindings()[0].channel(), "decoder_entropy");
    assert_eq!(
        contract.policy_of("decoder_entropy"),
        Ok(ModelObservationChannelTimingPolicy::RequireStrictlyBeforeAssertion)
    );
    let framed = contract.canonical_record();
    assert!(framed.starts_with(MODEL_OBSERVATION_TIMING_CONTRACT_SCHEMA));
    assert!(framed.contains("policy:require_strictly_before_assertion"));

    contract
        .admit_observation("decoder_entropy", 3, Some(10))
        .expect("strict pre");
    assert_eq!(
        contract.admit_observation("decoder_entropy", 11, Some(10)),
        Err(ModelObservationRejectionCode::EligibilityPrimaryRequiresStrictPreAssertion)
    );
    contract
        .admit_observation("hidden_norm", 11, Some(10))
        .expect("post-hoc allowed");
    assert_eq!(
        contract.admit_observation("hidden_norm", 10, Some(10)),
        Err(ModelObservationRejectionCode::TimingObservationAtAssertionBoundary)
    );
    assert_eq!(
        contract.admit_observation("hidden_norm", 1, None),
        Err(ModelObservationRejectionCode::EligibilityAssertionBoundaryUnknown)
    );

    let incomplete = ModelObservationTimingContract::covering_registry(
        &registry,
        vec![
            ModelObservationTimingBinding::new(
                "decoder_entropy",
                ModelObservationChannelTimingPolicy::AllowPostHocDiagnostic,
            )
            .unwrap(),
        ],
    );
    assert_eq!(
        incomplete,
        Err(ModelObservationRejectionCode::TimingContractChannelCoverageIncomplete)
    );
}

#[test]
fn h11a_eligibility_exact_primary_posthoc_and_fail_closed_boundary() {
    assert_eq!(
        ModelObservationH11AEligibilityRule::candidate_id(),
        CANDIDATE_H11A_ELIGIBILITY_RULE
    );
    assert_eq!(
        ModelObservationH11AEligibilityRule::classify(4, Some(10)),
        Ok(PrecursorEligibility::PrimaryEligible)
    );
    assert_eq!(
        ModelObservationH11AEligibilityRule::classify(12, Some(10)),
        Ok(PrecursorEligibility::PostHocOnly)
    );
    assert!(ModelObservationH11AEligibilityRule::admits_primary(
        PrecursorEligibility::PrimaryEligible
    ));
    assert!(!ModelObservationH11AEligibilityRule::admits_primary(
        PrecursorEligibility::PostHocOnly
    ));
    assert_eq!(
        ModelObservationH11AEligibilityRule::classify(10, Some(10)),
        Err(ModelObservationRejectionCode::TimingObservationAtAssertionBoundary)
    );
    assert_eq!(
        ModelObservationH11AEligibilityRule::classify(1, None),
        Err(ModelObservationRejectionCode::EligibilityAssertionBoundaryUnknown)
    );

    let primary = ModelObservationH11AEligibilityRule::canonical_record(
        4,
        Some(10),
        Ok(PrecursorEligibility::PrimaryEligible),
    );
    assert!(primary.starts_with(MODEL_OBSERVATION_H11A_ELIGIBILITY_SCHEMA));
    assert!(primary.contains("eligibility:primary_eligible"));
    assert!(primary.contains("admits_primary:true"));
    assert!(primary.contains(&format!("candidate:{CANDIDATE_H11A_ELIGIBILITY_RULE}")));

    let rejected = ModelObservationH11AEligibilityRule::canonical_record(
        1,
        None,
        Err(ModelObservationRejectionCode::EligibilityAssertionBoundaryUnknown),
    );
    assert!(rejected.contains("eligibility:rejected"));
    assert!(rejected.contains("eligibility_assertion_boundary_unknown:0x070a"));
}

#[test]
fn registry_timing_eligibility_codes_are_present_unique_and_stable() {
    let family = [
        ModelObservationRejectionCode::RegistryEmpty,
        ModelObservationRejectionCode::RegistryEmptyChannelName,
        ModelObservationRejectionCode::RegistryDuplicateChannel,
        ModelObservationRejectionCode::RegistryUnknownSourceClass,
        ModelObservationRejectionCode::RegistryUndeclaredChannel,
        ModelObservationRejectionCode::TimingContractEmpty,
        ModelObservationRejectionCode::TimingContractChannelCoverageIncomplete,
        ModelObservationRejectionCode::TimingContractUnknownChannel,
        ModelObservationRejectionCode::TimingContractPolicyConflict,
        ModelObservationRejectionCode::EligibilityAssertionBoundaryUnknown,
        ModelObservationRejectionCode::EligibilityPrimaryRequiresStrictPreAssertion,
    ];
    for code in family {
        assert!(
            ALL_MODEL_OBSERVATION_REJECTION_CODES.contains(&code),
            "missing {code}"
        );
    }
    let mut numerics: Vec<u16> = ALL_MODEL_OBSERVATION_REJECTION_CODES
        .iter()
        .map(|code| code.numeric())
        .collect();
    let before = numerics.len();
    numerics.sort_unstable();
    numerics.dedup();
    assert_eq!(before, numerics.len(), "duplicate numeric codes");
    assert_eq!(
        ModelObservationRejectionCode::RegistryEmpty.numeric(),
        0x0701
    );
    assert_eq!(
        ModelObservationRejectionCode::EligibilityPrimaryRequiresStrictPreAssertion.numeric(),
        0x070B
    );
}

#[test]
fn inherited_class_coverage_helper_does_not_pin_registry() {
    let partial = sample_registry();
    assert!(!partial.covers_all_inherited_source_classes());
    // Coverage helper is engineering only; candidate id remains non-authorizing.
    assert_eq!(
        CANDIDATE_OBSERVATION_REGISTRY,
        "ModelObservationChannelRegistry"
    );
}
