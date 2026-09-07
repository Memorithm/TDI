//! Non-final TDI-11.1 leakage-safe model/runtime observation adapter contract.
//!
//! This layer defines the boundary future local/open-model instrumentation must
//! cross before observations may enter the prospective H11-A timeline. It does
//! not load or execute a model, choose a detector, estimate hallucination risk,
//! select a controller, or expose evaluator-owned truth.
//!
//! The only representable source classes are explicitly model/runtime-visible
//! summaries. Complete-world truth, evaluator labels, hidden difficulty, final
//! seed material, future trajectory state, and alternative-arm outcomes have no
//! representation in this contract.

use core::fmt;

use crate::hallucination_observation::{
    ObservableSignal, ObservationContractError, ObservationTimeline, SignalChannel,
};

pub const OBSERVATION_ADAPTER_SCHEMA: &str = "tdi11-observation-adapter-v1";

/// Explicitly permitted classes of model/runtime-visible observations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ObservationSourceClass {
    DecoderStatistics,
    HiddenStateSummary,
    VisibleEvidenceSummary,
    VerifierResult,
    RetrievalToolResult,
    ActionHistory,
    RuntimeResourceSummary,
    ResampleSummary,
}

impl ObservationSourceClass {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DecoderStatistics => "decoder_statistics",
            Self::HiddenStateSummary => "hidden_state_summary",
            Self::VisibleEvidenceSummary => "visible_evidence_summary",
            Self::VerifierResult => "verifier_result",
            Self::RetrievalToolResult => "retrieval_tool_result",
            Self::ActionHistory => "action_history",
            Self::RuntimeResourceSummary => "runtime_resource_summary",
            Self::ResampleSummary => "resample_summary",
        }
    }
}

/// One declared signal channel and its permitted source class.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeclaredSignalChannel {
    channel: SignalChannel,
    source_class: ObservationSourceClass,
}

impl DeclaredSignalChannel {
    #[must_use]
    pub const fn new(channel: SignalChannel, source_class: ObservationSourceClass) -> Self {
        Self {
            channel,
            source_class,
        }
    }

    #[must_use]
    pub fn channel(&self) -> &SignalChannel {
        &self.channel
    }

    #[must_use]
    pub const fn source_class(&self) -> ObservationSourceClass {
        self.source_class
    }
}

/// Versioned identity of one non-final observation adapter configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservationAdapterManifest {
    adapter_name: String,
    adapter_version: String,
    model_artifact_identity: String,
    prompt_serializer_version: String,
    declared_channels: Vec<DeclaredSignalChannel>,
}

impl ObservationAdapterManifest {
    pub fn new(
        adapter_name: impl Into<String>,
        adapter_version: impl Into<String>,
        model_artifact_identity: impl Into<String>,
        prompt_serializer_version: impl Into<String>,
        mut declared_channels: Vec<DeclaredSignalChannel>,
    ) -> Result<Self, ObservationAdapterError> {
        let adapter_name = adapter_name.into();
        let adapter_version = adapter_version.into();
        let model_artifact_identity = model_artifact_identity.into();
        let prompt_serializer_version = prompt_serializer_version.into();

        if adapter_name.trim().is_empty() {
            return Err(ObservationAdapterError::EmptyAdapterName);
        }
        if adapter_version.trim().is_empty() {
            return Err(ObservationAdapterError::EmptyAdapterVersion);
        }
        if model_artifact_identity.trim().is_empty() {
            return Err(ObservationAdapterError::EmptyModelArtifactIdentity);
        }
        if prompt_serializer_version.trim().is_empty() {
            return Err(ObservationAdapterError::EmptyPromptSerializerVersion);
        }
        if declared_channels.is_empty() {
            return Err(ObservationAdapterError::EmptyChannelManifest);
        }

        declared_channels.sort_by(|left, right| {
            left.channel
                .as_str()
                .cmp(right.channel.as_str())
                .then_with(|| left.source_class.cmp(&right.source_class))
        });
        for pair in declared_channels.windows(2) {
            if pair[0].channel.as_str() == pair[1].channel.as_str() {
                return Err(ObservationAdapterError::DuplicateDeclaredChannel);
            }
        }

        Ok(Self {
            adapter_name,
            adapter_version,
            model_artifact_identity,
            prompt_serializer_version,
            declared_channels,
        })
    }

    #[must_use]
    pub fn adapter_name(&self) -> &str {
        &self.adapter_name
    }

    #[must_use]
    pub fn adapter_version(&self) -> &str {
        &self.adapter_version
    }

    #[must_use]
    pub fn model_artifact_identity(&self) -> &str {
        &self.model_artifact_identity
    }

    #[must_use]
    pub fn prompt_serializer_version(&self) -> &str {
        &self.prompt_serializer_version
    }

    #[must_use]
    pub fn declared_channels(&self) -> &[DeclaredSignalChannel] {
        &self.declared_channels
    }

    fn declared_source_for(&self, channel: &SignalChannel) -> Option<ObservationSourceClass> {
        self.declared_channels
            .iter()
            .find(|declared| declared.channel.as_str() == channel.as_str())
            .map(|declared| declared.source_class)
    }

    /// Deterministic configuration provenance. No signal values are included.
    #[must_use]
    pub fn provenance_record(&self) -> String {
        let channels = self
            .declared_channels
            .iter()
            .map(|declared| {
                format!(
                    "{}:{}",
                    declared.channel.as_str(),
                    declared.source_class.as_str()
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{OBSERVATION_ADAPTER_SCHEMA};adapter={};version={};model={};prompt={};channels={channels}",
            self.adapter_name,
            self.adapter_version,
            self.model_artifact_identity,
            self.prompt_serializer_version,
        )
    }
}

/// One adapter-produced observation packet. It cannot carry a truth label or
/// caller-controlled prospective-eligibility flag.
#[derive(Clone, Debug, PartialEq)]
pub struct AdapterObservationPacket {
    event_index: u64,
    source_class: ObservationSourceClass,
    signals: Vec<ObservableSignal>,
}

impl AdapterObservationPacket {
    pub fn new(
        event_index: u64,
        source_class: ObservationSourceClass,
        signals: Vec<ObservableSignal>,
    ) -> Result<Self, ObservationAdapterError> {
        if signals.is_empty() {
            return Err(ObservationAdapterError::EmptyPacket);
        }
        let mut names = signals
            .iter()
            .map(|signal| signal.channel().as_str())
            .collect::<Vec<_>>();
        names.sort_unstable();
        if names.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(ObservationAdapterError::DuplicatePacketChannel);
        }
        Ok(Self {
            event_index,
            source_class,
            signals,
        })
    }

    #[must_use]
    pub const fn event_index(&self) -> u64 {
        self.event_index
    }

    #[must_use]
    pub const fn source_class(&self) -> ObservationSourceClass {
        self.source_class
    }

    #[must_use]
    pub fn signals(&self) -> &[ObservableSignal] {
        &self.signals
    }
}

/// Leakage-safe sink that validates adapter declarations before forwarding
/// observations into the prospective timing contract.
#[derive(Clone, Debug, PartialEq)]
pub struct GuardedObservationAdapter {
    manifest: ObservationAdapterManifest,
    timeline: ObservationTimeline,
}

impl GuardedObservationAdapter {
    #[must_use]
    pub fn new(manifest: ObservationAdapterManifest) -> Self {
        Self {
            manifest,
            timeline: ObservationTimeline::new(),
        }
    }

    #[must_use]
    pub fn manifest(&self) -> &ObservationAdapterManifest {
        &self.manifest
    }

    #[must_use]
    pub fn timeline(&self) -> &ObservationTimeline {
        &self.timeline
    }

    pub fn ingest(
        &mut self,
        packet: AdapterObservationPacket,
    ) -> Result<(), ObservationAdapterError> {
        for signal in &packet.signals {
            let Some(declared_source) = self.manifest.declared_source_for(signal.channel()) else {
                return Err(ObservationAdapterError::UndeclaredChannel);
            };
            if declared_source != packet.source_class {
                return Err(ObservationAdapterError::SourceClassMismatch {
                    declared: declared_source,
                    packet: packet.source_class,
                });
            }
        }
        self.timeline
            .push_observation(packet.event_index, packet.signals)
            .map_err(ObservationAdapterError::Timing)
    }

    pub fn mark_first_assertion(
        &mut self,
        event_index: u64,
    ) -> Result<(), ObservationAdapterError> {
        self.timeline
            .mark_first_assertion(event_index)
            .map_err(ObservationAdapterError::Timing)
    }

    /// Adapter identity plus timing metadata; signal values remain excluded.
    pub fn provenance_record(&self) -> Result<String, ObservationAdapterError> {
        let timing = self
            .timeline
            .timing_provenance()
            .map_err(ObservationAdapterError::Timing)?;
        Ok(format!("{};{timing}", self.manifest.provenance_record()))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObservationAdapterError {
    EmptyAdapterName,
    EmptyAdapterVersion,
    EmptyModelArtifactIdentity,
    EmptyPromptSerializerVersion,
    EmptyChannelManifest,
    DuplicateDeclaredChannel,
    EmptyPacket,
    DuplicatePacketChannel,
    UndeclaredChannel,
    SourceClassMismatch {
        declared: ObservationSourceClass,
        packet: ObservationSourceClass,
    },
    Timing(ObservationContractError),
}

impl fmt::Display for ObservationAdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for ObservationAdapterError {}
