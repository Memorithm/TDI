//! Non-executing TDI-11.2 model-observation rejection / provenance qualification.
//!
//! These are software-contract tests only. They do not pin freeze fields, load a
//! model, or authorize model/final execution.

#[path = "../src/hallucination_model_observation_rejections.rs"]
mod hallucination_model_observation_rejections;
#[allow(dead_code)]
#[path = "../src/hallucination_observation.rs"]
mod hallucination_observation;
#[allow(dead_code)]
#[path = "../src/hallucination_observation_adapter.rs"]
mod hallucination_observation_adapter;

use hallucination_model_observation_rejections::{
    ALL_MODEL_OBSERVATION_REJECTION_CODES, CANDIDATE_PROVENANCE_SCHEMA,
    CANDIDATE_TYPED_REJECTION_VOCABULARY, MODEL_OBSERVATION_PROVENANCE_SCHEMA,
    MODEL_OBSERVATION_REJECTION_SCHEMA, ModelObservationDomain, ModelObservationRejectionCode,
    ModelObservationRejectionRecord, ModelObservationTraceProvenance,
};
use hallucination_observation::{ObservableSignal, ObservationContractError, SignalChannel};
use hallucination_observation_adapter::{
    AdapterObservationPacket, DeclaredSignalChannel, GuardedObservationAdapter,
    ObservationAdapterError, ObservationAdapterManifest, ObservationSourceClass,
};

fn channel(name: &str) -> SignalChannel {
    SignalChannel::new(name).expect("valid channel")
}

fn signal(name: &str, value: f64) -> ObservableSignal {
    ObservableSignal::new(channel(name), value).expect("finite signal")
}

fn manifest() -> ObservationAdapterManifest {
    ObservationAdapterManifest::new(
        "local-open-model-adapter",
        "dev-v1",
        "model-artifact:fixture-001",
        "tdi11-controlled-prompt-v1",
        vec![
            DeclaredSignalChannel::new(
                channel("decoder.entropy"),
                ObservationSourceClass::DecoderStatistics,
            ),
            DeclaredSignalChannel::new(
                channel("visible.support"),
                ObservationSourceClass::VisibleEvidenceSummary,
            ),
        ],
    )
    .expect("valid non-final manifest")
}

#[test]
fn candidate_identifiers_are_stable_and_non_pinning() {
    assert_eq!(
        CANDIDATE_TYPED_REJECTION_VOCABULARY,
        "ModelObservationRejectionCode"
    );
    assert_eq!(
        CANDIDATE_PROVENANCE_SCHEMA,
        "ModelObservationTraceProvenance"
    );
    assert_eq!(
        MODEL_OBSERVATION_REJECTION_SCHEMA,
        "tdi11.2-model-observation-rejection-v0"
    );
    assert_eq!(
        MODEL_OBSERVATION_PROVENANCE_SCHEMA,
        "tdi11.2-model-observation-provenance-v0"
    );
}

#[test]
fn source_class_mismatch_maps_to_stable_rejection_code() {
    let mut adapter = GuardedObservationAdapter::new(manifest());
    let packet = AdapterObservationPacket::new(
        1,
        ObservationSourceClass::HiddenStateSummary,
        vec![signal("decoder.entropy", 0.25)],
    )
    .expect("non-empty packet");
    let err = adapter.ingest(packet).expect_err("source mismatch");
    assert!(matches!(
        err,
        ObservationAdapterError::SourceClassMismatch { .. }
    ));
    let record = ModelObservationRejectionRecord::from_adapter_error(err, &manifest());
    assert_eq!(
        record.code(),
        ModelObservationRejectionCode::SourceClassMismatch
    );
    assert_eq!(record.code().numeric(), 0x0301);
    assert_eq!(record.adapter_name(), "local-open-model-adapter");
    assert_eq!(record.adapter_version(), "dev-v1");
    assert!(matches!(
        record.error(),
        ObservationAdapterError::SourceClassMismatch { .. }
    ));
    assert!(
        record
            .canonical_record()
            .contains(MODEL_OBSERVATION_REJECTION_SCHEMA)
    );
}

#[test]
fn timing_non_monotonic_maps_through_adapter_timing_wrapper() {
    let code = ModelObservationRejectionCode::from_timing_error(
        &ObservationContractError::NonMonotonicEventIndex {
            previous: 3,
            attempted: 2,
        },
    );
    assert_eq!(
        code,
        ModelObservationRejectionCode::TimingNonMonotonicEventIndex
    );
    assert_eq!(code.numeric(), 0x0404);

    let wrapped = ObservationAdapterError::Timing(ObservationContractError::EmptySignalVector);
    assert_eq!(
        ModelObservationRejectionCode::from_adapter_error(&wrapped),
        ModelObservationRejectionCode::TimingEmptySignalVector
    );
}

#[test]
fn undeclared_channel_is_packet_rejection_not_quality_outcome() {
    let mut adapter = GuardedObservationAdapter::new(manifest());
    let packet = AdapterObservationPacket::new(
        1,
        ObservationSourceClass::DecoderStatistics,
        vec![signal("decoder.unknown", 0.1)],
    )
    .expect("packet");
    let err = adapter.ingest(packet).expect_err("undeclared");
    assert_eq!(err, ObservationAdapterError::UndeclaredChannel);
    let record = ModelObservationRejectionRecord::from_adapter_error(err, &manifest());
    assert_eq!(
        record.code(),
        ModelObservationRejectionCode::PacketUndeclaredChannel
    );
    assert_eq!(record.code().numeric(), 0x0203);
}

#[test]
fn rejection_numeric_codes_are_exact_unique_and_stable() {
    let expected: &[(ModelObservationRejectionCode, u16)] = &[
        (ModelObservationRejectionCode::AdapterEmptyName, 0x0101),
        (ModelObservationRejectionCode::AdapterEmptyVersion, 0x0102),
        (
            ModelObservationRejectionCode::AdapterEmptyModelArtifactIdentity,
            0x0103,
        ),
        (
            ModelObservationRejectionCode::AdapterEmptyPromptSerializerVersion,
            0x0104,
        ),
        (
            ModelObservationRejectionCode::AdapterEmptyChannelManifest,
            0x0105,
        ),
        (
            ModelObservationRejectionCode::AdapterDuplicateDeclaredChannel,
            0x0106,
        ),
        (ModelObservationRejectionCode::PacketEmpty, 0x0201),
        (
            ModelObservationRejectionCode::PacketDuplicateChannel,
            0x0202,
        ),
        (
            ModelObservationRejectionCode::PacketUndeclaredChannel,
            0x0203,
        ),
        (ModelObservationRejectionCode::SourceClassMismatch, 0x0301),
        (
            ModelObservationRejectionCode::TimingEmptySignalChannel,
            0x0401,
        ),
        (ModelObservationRejectionCode::TimingNonFiniteSignal, 0x0402),
        (
            ModelObservationRejectionCode::TimingEmptySignalVector,
            0x0403,
        ),
        (
            ModelObservationRejectionCode::TimingNonMonotonicEventIndex,
            0x0404,
        ),
        (
            ModelObservationRejectionCode::TimingObservationAtAssertionBoundary,
            0x0405,
        ),
        (
            ModelObservationRejectionCode::TimingAssertionBoundaryAlreadySet,
            0x0406,
        ),
        (
            ModelObservationRejectionCode::TimingAssertionBoundaryNotAfterHistory,
            0x0407,
        ),
        (
            ModelObservationRejectionCode::TimingAssertionBoundaryUnknown,
            0x0408,
        ),
        (ModelObservationRejectionCode::ProvenanceEmptyField, 0x0501),
        (
            ModelObservationRejectionCode::ProvenanceMissingRequiredField,
            0x0502,
        ),
        (
            ModelObservationRejectionCode::ProvenanceForbiddenDomain,
            0x0503,
        ),
        (
            ModelObservationRejectionCode::ProvenanceForbiddenSurfaceToken,
            0x0504,
        ),
    ];
    assert_eq!(ALL_MODEL_OBSERVATION_REJECTION_CODES.len(), expected.len());
    let mut seen = std::collections::BTreeSet::new();
    for (idx, (code, numeric)) in expected.iter().enumerate() {
        assert_eq!(ALL_MODEL_OBSERVATION_REJECTION_CODES[idx], *code);
        assert_eq!(code.numeric(), *numeric);
        assert!(seen.insert(*numeric), "duplicate numeric {numeric:#06x}");
        assert!(!code.as_str().is_empty());
    }
}

#[test]
fn provenance_scaffold_accepts_dev_val_and_frames_canonical_record() {
    let provenance = ModelObservationTraceProvenance::from_manifest(
        ModelObservationDomain::Development,
        &manifest(),
        2,
        Some(3),
        None,
    )
    .expect("valid scaffold provenance");
    assert_eq!(provenance.domain(), ModelObservationDomain::Development);
    assert_eq!(provenance.adapter_name(), "local-open-model-adapter");
    assert_eq!(provenance.adapter_version(), "dev-v1");
    assert_eq!(provenance.event_count(), 2);
    assert_eq!(provenance.first_assertion_event(), Some(3));
    assert_eq!(provenance.rejection_code(), None);
    assert!(
        provenance
            .declared_source_classes()
            .contains(&"decoder_statistics")
    );
    let record = provenance.canonical_record();
    assert!(record.starts_with(MODEL_OBSERVATION_PROVENANCE_SCHEMA));
    assert!(record.contains("domain:Development"));
    assert!(record.contains("decoder_statistics"));
    assert!(record.contains("rejection_code:none"));
}

#[test]
fn provenance_scaffold_rejects_forbidden_surface_tokens_and_empty_fields() {
    assert_eq!(
        ModelObservationTraceProvenance::new(
            ModelObservationDomain::Validation,
            "ok",
            "",
            vec![],
            0,
            None,
            None,
        ),
        Err(ModelObservationRejectionCode::ProvenanceEmptyField)
    );
    assert_eq!(
        ModelObservationTraceProvenance::new(
            ModelObservationDomain::Validation,
            "runner-with-final_seed_list",
            "dev-v1",
            vec![],
            0,
            None,
            None,
        ),
        Err(ModelObservationRejectionCode::ProvenanceForbiddenSurfaceToken)
    );
    assert_eq!(
        ModelObservationDomain::parse("Final"),
        Err(ModelObservationRejectionCode::ProvenanceForbiddenDomain)
    );
}

#[test]
fn adapter_identity_failures_map_to_0x01xx_range() {
    let cases = [
        (
            ObservationAdapterError::EmptyAdapterName,
            ModelObservationRejectionCode::AdapterEmptyName,
            0x0101u16,
        ),
        (
            ObservationAdapterError::EmptyAdapterVersion,
            ModelObservationRejectionCode::AdapterEmptyVersion,
            0x0102,
        ),
        (
            ObservationAdapterError::EmptyModelArtifactIdentity,
            ModelObservationRejectionCode::AdapterEmptyModelArtifactIdentity,
            0x0103,
        ),
        (
            ObservationAdapterError::EmptyPromptSerializerVersion,
            ModelObservationRejectionCode::AdapterEmptyPromptSerializerVersion,
            0x0104,
        ),
        (
            ObservationAdapterError::EmptyChannelManifest,
            ModelObservationRejectionCode::AdapterEmptyChannelManifest,
            0x0105,
        ),
        (
            ObservationAdapterError::DuplicateDeclaredChannel,
            ModelObservationRejectionCode::AdapterDuplicateDeclaredChannel,
            0x0106,
        ),
    ];
    for (error, expected, numeric) in cases {
        let code = ModelObservationRejectionCode::from_adapter_error(&error);
        assert_eq!(code, expected);
        assert_eq!(code.numeric(), numeric);
    }
}
