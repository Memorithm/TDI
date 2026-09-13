//! Non-executing TDI-11.2 model-observation resource accounting qualification.
//!
//! Software-contract tests only. Do not pin freeze fields, load a model, or
//! authorize model/final execution.

#[path = "../src/hallucination_model_observation_accounting.rs"]
mod hallucination_model_observation_accounting;
#[allow(dead_code)]
#[path = "../src/hallucination_model_observation_rejections.rs"]
mod hallucination_model_observation_rejections;
#[allow(dead_code)]
#[path = "../src/hallucination_observation.rs"]
mod hallucination_observation;
#[allow(dead_code)]
#[path = "../src/hallucination_observation_adapter.rs"]
mod hallucination_observation_adapter;

use hallucination_model_observation_accounting::{
    CANDIDATE_RESOURCE_ACCOUNTING_CONTRACT, MODEL_OBSERVATION_RESOURCE_ACCOUNTING_SCHEMA,
    ModelObservationAccountingRejection, ModelObservationResourceEnvelope,
    ModelObservationResourceUsage, RESOURCE_ACCOUNTING_COMPONENT_COUNT,
    RESOURCE_ACCOUNTING_COMPONENT_KEYS,
};
use hallucination_model_observation_rejections::{
    ALL_MODEL_OBSERVATION_REJECTION_CODES, CANDIDATE_PROVENANCE_SCHEMA,
    CANDIDATE_TYPED_REJECTION_VOCABULARY, ModelObservationRejectionCode,
};

#[test]
fn candidate_identifier_is_stable_and_non_pinning() {
    assert_eq!(
        CANDIDATE_RESOURCE_ACCOUNTING_CONTRACT,
        "ModelObservationResourceAccountingContract"
    );
    assert_eq!(
        MODEL_OBSERVATION_RESOURCE_ACCOUNTING_SCHEMA,
        "tdi11.2-model-observation-resource-accounting-v0"
    );
    // Distinct from rejection/provenance candidates; none of these pin.
    assert_ne!(
        CANDIDATE_RESOURCE_ACCOUNTING_CONTRACT,
        CANDIDATE_TYPED_REJECTION_VOCABULARY
    );
    assert_ne!(
        CANDIDATE_RESOURCE_ACCOUNTING_CONTRACT,
        CANDIDATE_PROVENANCE_SCHEMA
    );
}

#[test]
fn component_taxonomy_is_exact_eleven_keys() {
    assert_eq!(
        RESOURCE_ACCOUNTING_COMPONENT_KEYS.len(),
        RESOURCE_ACCOUNTING_COMPONENT_COUNT
    );
    assert_eq!(RESOURCE_ACCOUNTING_COMPONENT_COUNT, 11);
    assert_eq!(
        RESOURCE_ACCOUNTING_COMPONENT_KEYS,
        &[
            "model_input_tokens",
            "model_output_tokens",
            "model_decode_steps",
            "adapter_ingest_events",
            "adapter_declared_channel_emissions",
            "observation_packet_bytes",
            "timing_contract_checks",
            "provenance_frame_bytes",
            "verifier_calls",
            "retrieval_tool_calls",
            "resamples",
        ]
    );
}

#[test]
fn charge_helpers_accumulate_exactly_and_frame_canonical_usage() {
    let mut usage = ModelObservationResourceUsage::zero();
    usage.charge_model_tokens(3, 5).expect("tokens");
    usage.charge_decode_steps(7).expect("decode");
    usage.charge_adapter_ingest(2, 64).expect("adapter ingest");
    usage.charge_timing_contract_check().expect("timing");
    usage
        .charge_provenance_frame_bytes(128)
        .expect("provenance");
    usage.charge_secondary(1, 1, 2).expect("secondary");

    assert_eq!(usage.model_input_tokens(), 3);
    assert_eq!(usage.model_output_tokens(), 5);
    assert_eq!(usage.model_decode_steps(), 7);
    assert_eq!(usage.adapter_ingest_events(), 1);
    assert_eq!(usage.adapter_declared_channel_emissions(), 2);
    assert_eq!(usage.observation_packet_bytes(), 64);
    assert_eq!(usage.timing_contract_checks(), 1);
    assert_eq!(usage.provenance_frame_bytes(), 128);
    assert_eq!(usage.verifier_calls(), 1);
    assert_eq!(usage.retrieval_tool_calls(), 1);
    assert_eq!(usage.resamples(), 2);
    assert_eq!(
        usage.total(),
        Ok(3 + 5 + 7 + 1 + 2 + 64 + 1 + 128 + 1 + 1 + 2)
    );

    let record = usage.canonical_record();
    assert!(record.starts_with(MODEL_OBSERVATION_RESOURCE_ACCOUNTING_SCHEMA));
    assert!(record.contains("kind:usage"));
    assert!(record.contains("model_decode_steps:7"));
    assert!(record.contains("observation_packet_bytes:64"));
}

#[test]
fn overflow_fail_closes_to_0x0601() {
    let mut usage = ModelObservationResourceUsage::zero();
    usage.charge_decode_steps(u64::MAX).expect("max steps");
    let err = usage
        .charge_decode_steps(1)
        .expect_err("overflow must fail closed");
    assert_eq!(err, ModelObservationRejectionCode::AccountingOverflow);
    assert_eq!(err.numeric(), 0x0601);
    assert_eq!(err.as_str(), "accounting_overflow");
    let rejection = ModelObservationAccountingRejection::overflow();
    assert_eq!(
        rejection.code(),
        ModelObservationRejectionCode::AccountingOverflow
    );
    assert_eq!(rejection.component(), None);
    assert!(
        rejection
            .canonical_record()
            .contains("accounting_overflow:0x0601")
    );
}

#[test]
fn envelope_admission_fail_closes_to_0x0602() {
    let envelope =
        ModelObservationResourceEnvelope::new(10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10);
    let mut usage = ModelObservationResourceUsage::zero();
    usage.charge_model_tokens(11, 0).expect("tokens");
    let err = envelope.admit(usage).expect_err("exceeded");
    assert_eq!(
        err,
        ModelObservationRejectionCode::AccountingEnvelopeExceeded
    );
    assert_eq!(err.numeric(), 0x0602);

    let rejection = ModelObservationAccountingRejection::envelope_exceeded("model_input_tokens");
    assert_eq!(
        rejection.code(),
        ModelObservationRejectionCode::AccountingEnvelopeExceeded
    );
    assert_eq!(rejection.component(), Some("model_input_tokens"));
    let framed = rejection.canonical_record();
    assert!(framed.contains("kind:rejection"));
    assert!(framed.contains("accounting_envelope_exceeded:0x0602"));
    assert!(framed.contains("component:model_input_tokens"));
}

#[test]
fn unbounded_caller_supplied_admits_finite_usage_but_is_not_a_pin() {
    let envelope = ModelObservationResourceEnvelope::unbounded_caller_supplied();
    let mut usage = ModelObservationResourceUsage::zero();
    usage.charge_model_tokens(1_000, 2_000).expect("tokens");
    usage.charge_decode_steps(50).expect("decode");
    envelope.admit(usage).expect("unbounded admits finite");
    let record = envelope.canonical_record();
    assert!(record.contains("kind:envelope"));
    assert!(record.contains(&format!("max_model_decode_steps:{}", u64::MAX)));
    // Candidate id must remain the non-authorizing string, not a numeric freeze.
    assert_eq!(
        CANDIDATE_RESOURCE_ACCOUNTING_CONTRACT,
        "ModelObservationResourceAccountingContract"
    );
}

#[test]
fn accounting_codes_are_present_unique_and_stable_in_rejection_vocabulary() {
    let accounting = [
        ModelObservationRejectionCode::AccountingOverflow,
        ModelObservationRejectionCode::AccountingEnvelopeExceeded,
    ];
    for code in accounting {
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
        ModelObservationRejectionCode::AccountingOverflow.numeric(),
        0x0601
    );
    assert_eq!(
        ModelObservationRejectionCode::AccountingEnvelopeExceeded.numeric(),
        0x0602
    );
}

#[test]
fn tdi11_1_unlimited_development_label_does_not_transfer_as_11_2_pin() {
    // Explicit non-transfer: the 11.2 candidate id is not the 11.1 helper name.
    assert_ne!(
        CANDIDATE_RESOURCE_ACCOUNTING_CONTRACT,
        "unlimited_for_development"
    );
    assert_ne!(
        CANDIDATE_RESOURCE_ACCOUNTING_CONTRACT,
        "ReferenceResourceEnvelope"
    );
    assert_ne!(
        CANDIDATE_RESOURCE_ACCOUNTING_CONTRACT,
        "tdi11-reference-accounting-v1"
    );
}
