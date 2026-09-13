//! Non-executing TDI-11.2 model-observation rejection + provenance scaffolding.
//!
//! This layer adds a closed, stable machine vocabulary over the already-qualified
//! TDI-11.1 observation timing and leakage-safe adapter contracts. It does **not**:
//!
//! - load or execute a concrete model;
//! - pin any of the twelve TDI-11.2 freeze fields;
//! - set `model_execution_authorized` / `final_execution_authorized`;
//! - transfer TDI-8.1 `SymbolicRejectionCode` or TDI-9.1 `ReferenceRejectionCode`;
//! - invent model/adapter/tokenizer/decoding identities.
//!
//! Technical adapter / timing / accounting / registry / eligibility failures remain typed
//! rejections and are never reinterpreted as hallucination-quality outcomes.

use core::fmt;

use super::hallucination_observation::ObservationContractError;
use super::hallucination_observation_adapter::{
    ObservationAdapterError, ObservationAdapterManifest, ObservationSourceClass,
};

/// Scaffold schema marker for model-observation rejection records.
pub const MODEL_OBSERVATION_REJECTION_SCHEMA: &str = "tdi11.2-model-observation-rejection-v0";

/// Scaffold schema marker for model-observation trace provenance records.
pub const MODEL_OBSERVATION_PROVENANCE_SCHEMA: &str = "tdi11.2-model-observation-provenance-v0";

/// Non-authorizing candidate identifier for the typed-rejection freeze field.
///
/// Matching this string in docs/CI does **not** pin
/// `typed_rejection_contract`. A later reviewed freeze must still replace
/// `unresolved_blocking` explicitly.
pub const CANDIDATE_TYPED_REJECTION_VOCABULARY: &str = "ModelObservationRejectionCode";

/// Non-authorizing candidate identifier for the provenance freeze field.
pub const CANDIDATE_PROVENANCE_SCHEMA: &str = "ModelObservationTraceProvenance";

/// Stable non-final machine code for every currently represented
/// model-observation rejection path.
///
/// Numeric values are intentionally explicit. Existing values must never be
/// renumbered; future rejection reasons must consume previously unused values.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum ModelObservationRejectionCode {
    // 0x01xx — adapter identity / manifest construction
    AdapterEmptyName = 0x0101,
    AdapterEmptyVersion = 0x0102,
    AdapterEmptyModelArtifactIdentity = 0x0103,
    AdapterEmptyPromptSerializerVersion = 0x0104,
    AdapterEmptyChannelManifest = 0x0105,
    AdapterDuplicateDeclaredChannel = 0x0106,

    // 0x02xx — packet / channel declaration failures
    PacketEmpty = 0x0201,
    PacketDuplicateChannel = 0x0202,
    PacketUndeclaredChannel = 0x0203,

    // 0x03xx — source-class contract
    SourceClassMismatch = 0x0301,

    // 0x04xx — prospective timing / observation contract
    TimingEmptySignalChannel = 0x0401,
    TimingNonFiniteSignal = 0x0402,
    TimingEmptySignalVector = 0x0403,
    TimingNonMonotonicEventIndex = 0x0404,
    TimingObservationAtAssertionBoundary = 0x0405,
    TimingAssertionBoundaryAlreadySet = 0x0406,
    TimingAssertionBoundaryNotAfterHistory = 0x0407,
    TimingAssertionBoundaryUnknown = 0x0408,

    // 0x05xx — provenance scaffolding failures
    ProvenanceEmptyField = 0x0501,
    ProvenanceMissingRequiredField = 0x0502,
    ProvenanceForbiddenDomain = 0x0503,
    ProvenanceForbiddenSurfaceToken = 0x0504,

    // 0x06xx — resource accounting scaffolding failures
    AccountingOverflow = 0x0601,
    AccountingEnvelopeExceeded = 0x0602,

    // 0x07xx — registry / timing-contract / H11-A eligibility scaffolding failures
    RegistryEmpty = 0x0701,
    RegistryEmptyChannelName = 0x0702,
    RegistryDuplicateChannel = 0x0703,
    RegistryUnknownSourceClass = 0x0704,
    RegistryUndeclaredChannel = 0x0705,
    TimingContractEmpty = 0x0706,
    TimingContractChannelCoverageIncomplete = 0x0707,
    TimingContractUnknownChannel = 0x0708,
    TimingContractPolicyConflict = 0x0709,
    EligibilityAssertionBoundaryUnknown = 0x070A,
    EligibilityPrimaryRequiresStrictPreAssertion = 0x070B,
}

impl ModelObservationRejectionCode {
    /// Exact stable numeric representation for machine-readable records.
    #[must_use]
    pub const fn numeric(self) -> u16 {
        self as u16
    }

    /// Stable snake-case identifier (docs / CI surface).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AdapterEmptyName => "adapter_empty_name",
            Self::AdapterEmptyVersion => "adapter_empty_version",
            Self::AdapterEmptyModelArtifactIdentity => "adapter_empty_model_artifact_identity",
            Self::AdapterEmptyPromptSerializerVersion => "adapter_empty_prompt_serializer_version",
            Self::AdapterEmptyChannelManifest => "adapter_empty_channel_manifest",
            Self::AdapterDuplicateDeclaredChannel => "adapter_duplicate_declared_channel",
            Self::PacketEmpty => "packet_empty",
            Self::PacketDuplicateChannel => "packet_duplicate_channel",
            Self::PacketUndeclaredChannel => "packet_undeclared_channel",
            Self::SourceClassMismatch => "source_class_mismatch",
            Self::TimingEmptySignalChannel => "timing_empty_signal_channel",
            Self::TimingNonFiniteSignal => "timing_non_finite_signal",
            Self::TimingEmptySignalVector => "timing_empty_signal_vector",
            Self::TimingNonMonotonicEventIndex => "timing_non_monotonic_event_index",
            Self::TimingObservationAtAssertionBoundary => {
                "timing_observation_at_assertion_boundary"
            }
            Self::TimingAssertionBoundaryAlreadySet => "timing_assertion_boundary_already_set",
            Self::TimingAssertionBoundaryNotAfterHistory => {
                "timing_assertion_boundary_not_after_history"
            }
            Self::TimingAssertionBoundaryUnknown => "timing_assertion_boundary_unknown",
            Self::ProvenanceEmptyField => "provenance_empty_field",
            Self::ProvenanceMissingRequiredField => "provenance_missing_required_field",
            Self::ProvenanceForbiddenDomain => "provenance_forbidden_domain",
            Self::ProvenanceForbiddenSurfaceToken => "provenance_forbidden_surface_token",
            Self::AccountingOverflow => "accounting_overflow",
            Self::AccountingEnvelopeExceeded => "accounting_envelope_exceeded",
            Self::RegistryEmpty => "registry_empty",
            Self::RegistryEmptyChannelName => "registry_empty_channel_name",
            Self::RegistryDuplicateChannel => "registry_duplicate_channel",
            Self::RegistryUnknownSourceClass => "registry_unknown_source_class",
            Self::RegistryUndeclaredChannel => "registry_undeclared_channel",
            Self::TimingContractEmpty => "timing_contract_empty",
            Self::TimingContractChannelCoverageIncomplete => {
                "timing_contract_channel_coverage_incomplete"
            }
            Self::TimingContractUnknownChannel => "timing_contract_unknown_channel",
            Self::TimingContractPolicyConflict => "timing_contract_policy_conflict",
            Self::EligibilityAssertionBoundaryUnknown => "eligibility_assertion_boundary_unknown",
            Self::EligibilityPrimaryRequiresStrictPreAssertion => {
                "eligibility_primary_requires_strict_pre_assertion"
            }
        }
    }

    /// Lossless categorical map from adapter errors (timing nested).
    #[must_use]
    pub const fn from_adapter_error(error: &ObservationAdapterError) -> Self {
        match error {
            ObservationAdapterError::EmptyAdapterName => Self::AdapterEmptyName,
            ObservationAdapterError::EmptyAdapterVersion => Self::AdapterEmptyVersion,
            ObservationAdapterError::EmptyModelArtifactIdentity => {
                Self::AdapterEmptyModelArtifactIdentity
            }
            ObservationAdapterError::EmptyPromptSerializerVersion => {
                Self::AdapterEmptyPromptSerializerVersion
            }
            ObservationAdapterError::EmptyChannelManifest => Self::AdapterEmptyChannelManifest,
            ObservationAdapterError::DuplicateDeclaredChannel => {
                Self::AdapterDuplicateDeclaredChannel
            }
            ObservationAdapterError::EmptyPacket => Self::PacketEmpty,
            ObservationAdapterError::DuplicatePacketChannel => Self::PacketDuplicateChannel,
            ObservationAdapterError::UndeclaredChannel => Self::PacketUndeclaredChannel,
            ObservationAdapterError::SourceClassMismatch { .. } => Self::SourceClassMismatch,
            ObservationAdapterError::Timing(timing) => Self::from_timing_error(timing),
        }
    }

    /// Lossless categorical map from prospective timing contract errors.
    #[must_use]
    pub const fn from_timing_error(error: &ObservationContractError) -> Self {
        match error {
            ObservationContractError::EmptySignalChannel => Self::TimingEmptySignalChannel,
            ObservationContractError::NonFiniteSignal => Self::TimingNonFiniteSignal,
            ObservationContractError::EmptySignalVector => Self::TimingEmptySignalVector,
            ObservationContractError::NonMonotonicEventIndex { .. } => {
                Self::TimingNonMonotonicEventIndex
            }
            ObservationContractError::ObservationAtAssertionBoundary { .. } => {
                Self::TimingObservationAtAssertionBoundary
            }
            ObservationContractError::AssertionBoundaryAlreadySet { .. } => {
                Self::TimingAssertionBoundaryAlreadySet
            }
            ObservationContractError::AssertionBoundaryNotAfterHistory { .. } => {
                Self::TimingAssertionBoundaryNotAfterHistory
            }
            ObservationContractError::AssertionBoundaryUnknown => {
                Self::TimingAssertionBoundaryUnknown
            }
        }
    }
}

impl fmt::Display for ModelObservationRejectionCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{:#06x}", self.as_str(), self.numeric())
    }
}

/// Immutable rejection record retaining the original typed adapter error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelObservationRejectionRecord {
    code: ModelObservationRejectionCode,
    error: ObservationAdapterError,
    adapter_name: String,
    adapter_version: String,
}

impl ModelObservationRejectionRecord {
    #[must_use]
    pub fn from_adapter_error(
        error: ObservationAdapterError,
        manifest: &ObservationAdapterManifest,
    ) -> Self {
        Self {
            code: ModelObservationRejectionCode::from_adapter_error(&error),
            error,
            adapter_name: manifest.adapter_name().to_string(),
            adapter_version: manifest.adapter_version().to_string(),
        }
    }

    #[must_use]
    pub const fn code(&self) -> ModelObservationRejectionCode {
        self.code
    }

    #[must_use]
    pub const fn error(&self) -> &ObservationAdapterError {
        &self.error
    }

    #[must_use]
    pub fn adapter_name(&self) -> &str {
        &self.adapter_name
    }

    #[must_use]
    pub fn adapter_version(&self) -> &str {
        &self.adapter_version
    }

    /// Byte-length framed machine-readable record (scaffold only).
    #[must_use]
    pub fn canonical_record(&self) -> String {
        format!(
            "{MODEL_OBSERVATION_REJECTION_SCHEMA}\ncode:{}\nnumeric:{:#06x}\nadapter_name:{}{}\nadapter_version:{}{}\nerror:{:?}",
            self.code.as_str(),
            self.code.numeric(),
            self.adapter_name.len(),
            self.adapter_name,
            self.adapter_version.len(),
            self.adapter_version,
            self.error
        )
    }
}

/// Allowed Development/Validation domain labels for provenance scaffolding.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ModelObservationDomain {
    Development,
    Validation,
}

impl ModelObservationDomain {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Development => "Development",
            Self::Validation => "Validation",
        }
    }

    pub fn parse(raw: &str) -> Result<Self, ModelObservationRejectionCode> {
        match raw {
            "Development" => Ok(Self::Development),
            "Validation" => Ok(Self::Validation),
            _ => Err(ModelObservationRejectionCode::ProvenanceForbiddenDomain),
        }
    }
}

/// Fail-closed scaffold for machine-readable model-observation trace provenance.
///
/// Required keys are integrity scaffolding only. Presence of this type does
/// **not** pin `provenance_contract` and does not authorize model execution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelObservationTraceProvenance {
    domain: ModelObservationDomain,
    adapter_name: String,
    adapter_version: String,
    declared_source_classes: Vec<&'static str>,
    event_count: u64,
    first_assertion_event: Option<u64>,
    rejection_code: Option<ModelObservationRejectionCode>,
}

impl ModelObservationTraceProvenance {
    pub fn new(
        domain: ModelObservationDomain,
        adapter_name: impl Into<String>,
        adapter_version: impl Into<String>,
        declared_source_classes: Vec<ObservationSourceClass>,
        event_count: u64,
        first_assertion_event: Option<u64>,
        rejection_code: Option<ModelObservationRejectionCode>,
    ) -> Result<Self, ModelObservationRejectionCode> {
        let adapter_name = adapter_name.into();
        let adapter_version = adapter_version.into();
        if adapter_name.trim().is_empty() || adapter_version.trim().is_empty() {
            return Err(ModelObservationRejectionCode::ProvenanceEmptyField);
        }
        for forbidden in [
            "final_seed_list",
            "final_dataset",
            "final_runner",
            "final_result_payload",
            "concrete_model_runner",
            "complete_world_hidden_truth",
            "evaluator_labels",
        ] {
            if adapter_name.contains(forbidden) || adapter_version.contains(forbidden) {
                return Err(ModelObservationRejectionCode::ProvenanceForbiddenSurfaceToken);
            }
        }
        let mut classes: Vec<&'static str> = declared_source_classes
            .into_iter()
            .map(ObservationSourceClass::as_str)
            .collect();
        classes.sort_unstable();
        classes.dedup();
        Ok(Self {
            domain,
            adapter_name,
            adapter_version,
            declared_source_classes: classes,
            event_count,
            first_assertion_event,
            rejection_code,
        })
    }

    /// Build provenance from a non-final adapter manifest (scaffold helper).
    pub fn from_manifest(
        domain: ModelObservationDomain,
        manifest: &ObservationAdapterManifest,
        event_count: u64,
        first_assertion_event: Option<u64>,
        rejection_code: Option<ModelObservationRejectionCode>,
    ) -> Result<Self, ModelObservationRejectionCode> {
        let classes = manifest
            .declared_channels()
            .iter()
            .map(|declared| declared.source_class())
            .collect();
        Self::new(
            domain,
            manifest.adapter_name(),
            manifest.adapter_version(),
            classes,
            event_count,
            first_assertion_event,
            rejection_code,
        )
    }

    #[must_use]
    pub const fn domain(&self) -> ModelObservationDomain {
        self.domain
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
    pub fn declared_source_classes(&self) -> &[&'static str] {
        &self.declared_source_classes
    }

    #[must_use]
    pub const fn event_count(&self) -> u64 {
        self.event_count
    }

    #[must_use]
    pub const fn first_assertion_event(&self) -> Option<u64> {
        self.first_assertion_event
    }

    #[must_use]
    pub const fn rejection_code(&self) -> Option<ModelObservationRejectionCode> {
        self.rejection_code
    }

    /// Byte-length framed machine-readable provenance record.
    #[must_use]
    pub fn canonical_record(&self) -> String {
        let mut out = String::from(MODEL_OBSERVATION_PROVENANCE_SCHEMA);
        out.push('\n');
        out.push_str(&format!("domain:{}\n", self.domain.as_str()));
        out.push_str(&format!(
            "adapter_name:{}{}\n",
            self.adapter_name.len(),
            self.adapter_name
        ));
        out.push_str(&format!(
            "adapter_version:{}{}\n",
            self.adapter_version.len(),
            self.adapter_version
        ));
        out.push_str(&format!("event_count:{}\n", self.event_count));
        match self.first_assertion_event {
            Some(index) => out.push_str(&format!("first_assertion_event:{index}\n")),
            None => out.push_str("first_assertion_event:none\n"),
        }
        out.push_str("declared_source_classes:");
        out.push_str(&self.declared_source_classes.join(","));
        out.push('\n');
        match self.rejection_code {
            Some(code) => out.push_str(&format!(
                "rejection_code:{}:{:#06x}\n",
                code.as_str(),
                code.numeric()
            )),
            None => out.push_str("rejection_code:none\n"),
        }
        out
    }
}

/// Exhaustive list of currently minted numeric codes (CI stability surface).
pub const ALL_MODEL_OBSERVATION_REJECTION_CODES: &[ModelObservationRejectionCode] = &[
    ModelObservationRejectionCode::AdapterEmptyName,
    ModelObservationRejectionCode::AdapterEmptyVersion,
    ModelObservationRejectionCode::AdapterEmptyModelArtifactIdentity,
    ModelObservationRejectionCode::AdapterEmptyPromptSerializerVersion,
    ModelObservationRejectionCode::AdapterEmptyChannelManifest,
    ModelObservationRejectionCode::AdapterDuplicateDeclaredChannel,
    ModelObservationRejectionCode::PacketEmpty,
    ModelObservationRejectionCode::PacketDuplicateChannel,
    ModelObservationRejectionCode::PacketUndeclaredChannel,
    ModelObservationRejectionCode::SourceClassMismatch,
    ModelObservationRejectionCode::TimingEmptySignalChannel,
    ModelObservationRejectionCode::TimingNonFiniteSignal,
    ModelObservationRejectionCode::TimingEmptySignalVector,
    ModelObservationRejectionCode::TimingNonMonotonicEventIndex,
    ModelObservationRejectionCode::TimingObservationAtAssertionBoundary,
    ModelObservationRejectionCode::TimingAssertionBoundaryAlreadySet,
    ModelObservationRejectionCode::TimingAssertionBoundaryNotAfterHistory,
    ModelObservationRejectionCode::TimingAssertionBoundaryUnknown,
    ModelObservationRejectionCode::ProvenanceEmptyField,
    ModelObservationRejectionCode::ProvenanceMissingRequiredField,
    ModelObservationRejectionCode::ProvenanceForbiddenDomain,
    ModelObservationRejectionCode::ProvenanceForbiddenSurfaceToken,
    ModelObservationRejectionCode::AccountingOverflow,
    ModelObservationRejectionCode::AccountingEnvelopeExceeded,
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
