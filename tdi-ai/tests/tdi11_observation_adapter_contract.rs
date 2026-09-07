#[allow(dead_code)]
#[path = "../src/hallucination_observation.rs"]
mod hallucination_observation;
#[path = "../src/hallucination_observation_adapter.rs"]
mod hallucination_observation_adapter;

use hallucination_observation::{ObservableSignal, PrecursorEligibility, SignalChannel};
use hallucination_observation_adapter::{
    AdapterObservationPacket, DeclaredSignalChannel, GuardedObservationAdapter,
    OBSERVATION_ADAPTER_SCHEMA, ObservationAdapterError, ObservationAdapterManifest,
    ObservationSourceClass,
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
                channel("hidden.norm"),
                ObservationSourceClass::HiddenStateSummary,
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
fn all_source_classes_have_stable_non_oracle_names() {
    let cases = [
        (
            ObservationSourceClass::DecoderStatistics,
            "decoder_statistics",
        ),
        (
            ObservationSourceClass::HiddenStateSummary,
            "hidden_state_summary",
        ),
        (
            ObservationSourceClass::VisibleEvidenceSummary,
            "visible_evidence_summary",
        ),
        (ObservationSourceClass::VerifierResult, "verifier_result"),
        (
            ObservationSourceClass::RetrievalToolResult,
            "retrieval_tool_result",
        ),
        (ObservationSourceClass::ActionHistory, "action_history"),
        (
            ObservationSourceClass::RuntimeResourceSummary,
            "runtime_resource_summary",
        ),
        (ObservationSourceClass::ResampleSummary, "resample_summary"),
    ];

    for (source, expected) in cases {
        assert_eq!(source.as_str(), expected);
        assert!(!source.as_str().contains("truth"));
        assert!(!source.as_str().contains("label"));
        assert!(!source.as_str().contains("future"));
    }
}

#[test]
fn manifest_requires_complete_identity_and_nonempty_channels() {
    let declared = vec![DeclaredSignalChannel::new(
        channel("decoder.entropy"),
        ObservationSourceClass::DecoderStatistics,
    )];

    assert_eq!(
        ObservationAdapterManifest::new("", "v1", "model", "prompt", declared.clone()),
        Err(ObservationAdapterError::EmptyAdapterName)
    );
    assert_eq!(
        ObservationAdapterManifest::new("adapter", "", "model", "prompt", declared.clone()),
        Err(ObservationAdapterError::EmptyAdapterVersion)
    );
    assert_eq!(
        ObservationAdapterManifest::new("adapter", "v1", "", "prompt", declared.clone()),
        Err(ObservationAdapterError::EmptyModelArtifactIdentity)
    );
    assert_eq!(
        ObservationAdapterManifest::new("adapter", "v1", "model", "", declared),
        Err(ObservationAdapterError::EmptyPromptSerializerVersion)
    );
    assert_eq!(
        ObservationAdapterManifest::new("adapter", "v1", "model", "prompt", Vec::new()),
        Err(ObservationAdapterError::EmptyChannelManifest)
    );
}

#[test]
fn manifest_rejects_duplicate_channel_names_even_across_source_classes() {
    let duplicate = channel("shared.signal");
    let result = ObservationAdapterManifest::new(
        "adapter",
        "v1",
        "model",
        "prompt",
        vec![
            DeclaredSignalChannel::new(
                duplicate.clone(),
                ObservationSourceClass::DecoderStatistics,
            ),
            DeclaredSignalChannel::new(duplicate, ObservationSourceClass::HiddenStateSummary),
        ],
    );
    assert_eq!(
        result,
        Err(ObservationAdapterError::DuplicateDeclaredChannel)
    );
}

#[test]
fn manifest_is_sorted_and_provenance_is_deterministic() {
    let manifest = manifest();
    assert_eq!(manifest.adapter_name(), "local-open-model-adapter");
    assert_eq!(manifest.adapter_version(), "dev-v1");
    assert_eq!(
        manifest.model_artifact_identity(),
        "model-artifact:fixture-001"
    );
    assert_eq!(
        manifest.prompt_serializer_version(),
        "tdi11-controlled-prompt-v1"
    );
    assert_eq!(manifest.declared_channels().len(), 3);
    assert_eq!(
        manifest.declared_channels()[0].channel().as_str(),
        "decoder.entropy"
    );
    assert_eq!(
        manifest.declared_channels()[0].source_class(),
        ObservationSourceClass::DecoderStatistics
    );

    let provenance = manifest.provenance_record();
    assert!(provenance.starts_with(OBSERVATION_ADAPTER_SCHEMA));
    assert!(provenance.contains("adapter=local-open-model-adapter"));
    assert!(provenance.contains("model=model-artifact:fixture-001"));
    assert!(provenance.contains("decoder.entropy:decoder_statistics"));
}

#[test]
fn packet_requires_nonempty_unique_channels_and_exposes_no_eligibility_flag() {
    assert_eq!(
        AdapterObservationPacket::new(1, ObservationSourceClass::DecoderStatistics, Vec::new()),
        Err(ObservationAdapterError::EmptyPacket)
    );

    let packet = AdapterObservationPacket::new(
        2,
        ObservationSourceClass::DecoderStatistics,
        vec![
            signal("decoder.entropy", 0.75),
            signal("decoder.entropy", 0.25),
        ],
    );
    assert_eq!(packet, Err(ObservationAdapterError::DuplicatePacketChannel));

    let packet = AdapterObservationPacket::new(
        3,
        ObservationSourceClass::DecoderStatistics,
        vec![signal("decoder.entropy", 0.5)],
    )
    .expect("valid packet");
    assert_eq!(packet.event_index(), 3);
    assert_eq!(
        packet.source_class(),
        ObservationSourceClass::DecoderStatistics
    );
    assert_eq!(packet.signals().len(), 1);
}

#[test]
fn guarded_adapter_rejects_undeclared_and_wrong_source_channels() {
    let mut adapter = GuardedObservationAdapter::new(manifest());
    assert_eq!(
        adapter.manifest().adapter_name(),
        "local-open-model-adapter"
    );

    let undeclared = AdapterObservationPacket::new(
        1,
        ObservationSourceClass::DecoderStatistics,
        vec![signal("decoder.margin", 0.3)],
    )
    .expect("packet shape is valid");
    assert_eq!(
        adapter.ingest(undeclared),
        Err(ObservationAdapterError::UndeclaredChannel)
    );

    let wrong_source = AdapterObservationPacket::new(
        1,
        ObservationSourceClass::HiddenStateSummary,
        vec![signal("decoder.entropy", 0.3)],
    )
    .expect("packet shape is valid");
    assert_eq!(
        adapter.ingest(wrong_source),
        Err(ObservationAdapterError::SourceClassMismatch {
            declared: ObservationSourceClass::DecoderStatistics,
            packet: ObservationSourceClass::HiddenStateSummary,
        })
    );
    assert!(adapter.timeline().observations().is_empty());
}

#[test]
fn guarded_adapter_preserves_prospective_timing_and_posthoc_boundary() {
    let mut adapter = GuardedObservationAdapter::new(manifest());
    adapter
        .ingest(
            AdapterObservationPacket::new(
                1,
                ObservationSourceClass::DecoderStatistics,
                vec![signal("decoder.entropy", 0.9)],
            )
            .expect("packet"),
        )
        .expect("declared pre-claim signal");
    adapter
        .ingest(
            AdapterObservationPacket::new(
                2,
                ObservationSourceClass::HiddenStateSummary,
                vec![signal("hidden.norm", 1.25)],
            )
            .expect("packet"),
        )
        .expect("declared pre-claim signal");
    adapter.mark_first_assertion(3).expect("claim boundary");
    adapter
        .ingest(
            AdapterObservationPacket::new(
                4,
                ObservationSourceClass::VisibleEvidenceSummary,
                vec![signal("visible.support", 0.4)],
            )
            .expect("packet"),
        )
        .expect("post-hoc signal is retained");

    let timing = adapter.timeline().timing_records().expect("boundary known");
    assert_eq!(timing.len(), 3);
    assert_eq!(
        timing[0].eligibility(),
        PrecursorEligibility::PrimaryEligible
    );
    assert_eq!(
        timing[1].eligibility(),
        PrecursorEligibility::PrimaryEligible
    );
    assert_eq!(timing[2].eligibility(), PrecursorEligibility::PostHocOnly);
    assert_eq!(
        adapter
            .timeline()
            .primary_precursor_events()
            .expect("boundary known")
            .len(),
        2
    );
}

#[test]
fn adapter_provenance_excludes_observed_signal_values() {
    let mut adapter = GuardedObservationAdapter::new(manifest());
    adapter
        .ingest(
            AdapterObservationPacket::new(
                7,
                ObservationSourceClass::DecoderStatistics,
                vec![signal("decoder.entropy", 123.456)],
            )
            .expect("packet"),
        )
        .expect("declared signal");
    adapter.mark_first_assertion(8).expect("claim boundary");

    let provenance = adapter.provenance_record().expect("timing known");
    assert!(provenance.contains("first_assertion_event=8"));
    assert!(provenance.contains("observation_events=7"));
    assert!(!provenance.contains("123.456"));
}

#[test]
fn timing_errors_are_preserved_as_typed_adapter_errors() {
    let mut adapter = GuardedObservationAdapter::new(manifest());
    adapter
        .ingest(
            AdapterObservationPacket::new(
                2,
                ObservationSourceClass::DecoderStatistics,
                vec![signal("decoder.entropy", 0.5)],
            )
            .expect("packet"),
        )
        .expect("first event");

    let repeated = AdapterObservationPacket::new(
        2,
        ObservationSourceClass::DecoderStatistics,
        vec![signal("decoder.entropy", 0.4)],
    )
    .expect("packet");
    assert!(matches!(
        adapter.ingest(repeated),
        Err(ObservationAdapterError::Timing(_))
    ));
    assert!(matches!(
        adapter.provenance_record(),
        Err(ObservationAdapterError::Timing(_))
    ));
}
