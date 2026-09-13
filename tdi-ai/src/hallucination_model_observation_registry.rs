//! Non-executing TDI-11.2 model-observation registry / timing / H11-A eligibility
//! scaffolding.
//!
//! This layer declares:
//! - the exact inherited `ObservationSourceClass` taxonomy (classes only);
//! - a fail-closed caller-supplied channel registry candidate;
//! - a fail-closed per-channel timing-policy contract candidate;
//! - an exact H11-A pre-assertion eligibility classifier candidate.
//!
//! It does **not**:
//! - invent scientific channel names or freeze a registry;
//! - pin `exact_observation_registry`, `exact_observation_timing`, or
//!   `primary_pre_assertion_eligibility`;
//! - load or execute a concrete model;
//! - set `model_execution_authorized` / `final_execution_authorized`.
//!
//! Presence of this scaffold is a non-authorizing candidate only. Distinct from
//! rejection/provenance (#213) and resource-accounting (#216) scaffolds.

use core::fmt;

use super::hallucination_model_observation_rejections::ModelObservationRejectionCode;
use super::hallucination_observation::PrecursorEligibility;
use super::hallucination_observation_adapter::ObservationSourceClass;

/// Scaffold schema marker for channel-registry records.
pub const MODEL_OBSERVATION_REGISTRY_SCHEMA: &str = "tdi11.2-model-observation-registry-v0";

/// Scaffold schema marker for timing-contract records.
pub const MODEL_OBSERVATION_TIMING_CONTRACT_SCHEMA: &str =
    "tdi11.2-model-observation-timing-contract-v0";

/// Scaffold schema marker for H11-A eligibility records.
pub const MODEL_OBSERVATION_H11A_ELIGIBILITY_SCHEMA: &str =
    "tdi11.2-model-observation-h11a-eligibility-v0";

/// Non-authorizing candidate identifier for `exact_observation_registry`.
pub const CANDIDATE_OBSERVATION_REGISTRY: &str = "ModelObservationChannelRegistry";

/// Non-authorizing candidate identifier for `exact_observation_timing`.
pub const CANDIDATE_OBSERVATION_TIMING_CONTRACT: &str = "ModelObservationTimingContract";

/// Non-authorizing candidate identifier for `primary_pre_assertion_eligibility`.
pub const CANDIDATE_H11A_ELIGIBILITY_RULE: &str = "ModelObservationH11AEligibilityRule";

/// Exact ordered inherited source-class keys from the TDI-11.2 pre-arm.
///
/// These are *classes*, not an exact channel registry. Selecting scientific
/// channel names remains a later reviewed freeze.
pub const INHERITED_OBSERVATION_SOURCE_CLASS_KEYS: &[&str] = &[
    "decoder_statistics",
    "hidden_state_summary",
    "visible_evidence_summary",
    "verifier_result",
    "retrieval_tool_result",
    "action_history",
    "runtime_resource_summary",
    "resample_summary",
];

/// Exact count of inherited source-class keys.
pub const INHERITED_OBSERVATION_SOURCE_CLASS_COUNT: usize = 8;

/// One caller-supplied registry entry (channel name + inherited source class).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelObservationChannelEntry {
    channel: String,
    source_class: ObservationSourceClass,
}

impl ModelObservationChannelEntry {
    /// Construct a channel entry; empty names fail closed.
    pub fn new(
        channel: impl Into<String>,
        source_class: ObservationSourceClass,
    ) -> Result<Self, ModelObservationRejectionCode> {
        let channel = channel.into();
        if channel.trim().is_empty() {
            return Err(ModelObservationRejectionCode::RegistryEmptyChannelName);
        }
        if !INHERITED_OBSERVATION_SOURCE_CLASS_KEYS.contains(&source_class.as_str()) {
            return Err(ModelObservationRejectionCode::RegistryUnknownSourceClass);
        }
        Ok(Self {
            channel,
            source_class,
        })
    }

    #[must_use]
    pub fn channel(&self) -> &str {
        &self.channel
    }

    #[must_use]
    pub const fn source_class(&self) -> ObservationSourceClass {
        self.source_class
    }
}

/// Fail-closed caller-supplied observation channel registry (candidate only).
///
/// Constructing a registry never freezes scientific channel identities. Channel
/// names remain caller-supplied Development/Validation scaffolding inputs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelObservationChannelRegistry {
    entries: Vec<ModelObservationChannelEntry>,
}

impl ModelObservationChannelRegistry {
    /// Build a non-empty registry with unique channel names.
    pub fn new(
        mut entries: Vec<ModelObservationChannelEntry>,
    ) -> Result<Self, ModelObservationRejectionCode> {
        if entries.is_empty() {
            return Err(ModelObservationRejectionCode::RegistryEmpty);
        }
        entries.sort_by(|left, right| {
            left.channel
                .cmp(&right.channel)
                .then_with(|| left.source_class.cmp(&right.source_class))
        });
        for window in entries.windows(2) {
            if window[0].channel == window[1].channel {
                return Err(ModelObservationRejectionCode::RegistryDuplicateChannel);
            }
        }
        Ok(Self { entries })
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[must_use]
    pub fn entries(&self) -> &[ModelObservationChannelEntry] {
        &self.entries
    }

    /// Look up a declared channel; undeclared names fail closed.
    pub fn source_class_of(
        &self,
        channel: &str,
    ) -> Result<ObservationSourceClass, ModelObservationRejectionCode> {
        self.entries
            .iter()
            .find(|entry| entry.channel == channel)
            .map(ModelObservationChannelEntry::source_class)
            .ok_or(ModelObservationRejectionCode::RegistryUndeclaredChannel)
    }

    /// Whether every inherited source class appears at least once (coverage helper).
    #[must_use]
    pub fn covers_all_inherited_source_classes(&self) -> bool {
        INHERITED_OBSERVATION_SOURCE_CLASS_KEYS.iter().all(|key| {
            self.entries
                .iter()
                .any(|entry| entry.source_class.as_str() == *key)
        })
    }

    /// Byte-length framed machine-readable registry record.
    #[must_use]
    pub fn canonical_record(&self) -> String {
        let mut out = String::from(MODEL_OBSERVATION_REGISTRY_SCHEMA);
        out.push_str("\nkind:registry\n");
        out.push_str(&format!("entry_count:{}\n", self.entries.len()));
        for entry in &self.entries {
            out.push_str(&format!(
                "channel:{}{}:class:{}\n",
                entry.channel.len(),
                entry.channel,
                entry.source_class.as_str()
            ));
        }
        out
    }
}

/// Per-channel timing policy admitted by the scaffolding contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ModelObservationChannelTimingPolicy {
    /// Channel may contribute only to primary H11-A evidence (strict pre-assertion).
    RequireStrictlyBeforeAssertion,
    /// Channel may also appear post-hoc as diagnostic-only (never primary).
    AllowPostHocDiagnostic,
}

impl ModelObservationChannelTimingPolicy {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequireStrictlyBeforeAssertion => "require_strictly_before_assertion",
            Self::AllowPostHocDiagnostic => "allow_post_hoc_diagnostic",
        }
    }
}

/// One timing-policy binding for a registered channel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelObservationTimingBinding {
    channel: String,
    policy: ModelObservationChannelTimingPolicy,
}

impl ModelObservationTimingBinding {
    pub fn new(
        channel: impl Into<String>,
        policy: ModelObservationChannelTimingPolicy,
    ) -> Result<Self, ModelObservationRejectionCode> {
        let channel = channel.into();
        if channel.trim().is_empty() {
            return Err(ModelObservationRejectionCode::RegistryEmptyChannelName);
        }
        Ok(Self { channel, policy })
    }

    #[must_use]
    pub fn channel(&self) -> &str {
        &self.channel
    }

    #[must_use]
    pub const fn policy(&self) -> ModelObservationChannelTimingPolicy {
        self.policy
    }
}

/// Fail-closed per-channel timing contract over a registry (candidate only).
///
/// Must cover every registry channel exactly once. Does **not** pin
/// `exact_observation_timing`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelObservationTimingContract {
    bindings: Vec<ModelObservationTimingBinding>,
}

impl ModelObservationTimingContract {
    /// Build a contract that exactly covers `registry` channels.
    pub fn covering_registry(
        registry: &ModelObservationChannelRegistry,
        mut bindings: Vec<ModelObservationTimingBinding>,
    ) -> Result<Self, ModelObservationRejectionCode> {
        if bindings.is_empty() {
            return Err(ModelObservationRejectionCode::TimingContractEmpty);
        }
        bindings.sort_by(|left, right| left.channel.cmp(&right.channel));
        for window in bindings.windows(2) {
            if window[0].channel == window[1].channel {
                return Err(ModelObservationRejectionCode::TimingContractPolicyConflict);
            }
        }
        if bindings.len() != registry.len() {
            return Err(ModelObservationRejectionCode::TimingContractChannelCoverageIncomplete);
        }
        for binding in &bindings {
            registry.source_class_of(&binding.channel)?;
        }
        for entry in registry.entries() {
            if !bindings
                .iter()
                .any(|binding| binding.channel == entry.channel)
            {
                return Err(ModelObservationRejectionCode::TimingContractChannelCoverageIncomplete);
            }
        }
        Ok(Self { bindings })
    }

    #[must_use]
    pub fn bindings(&self) -> &[ModelObservationTimingBinding] {
        &self.bindings
    }

    /// Look up timing policy; unknown channels fail closed.
    pub fn policy_of(
        &self,
        channel: &str,
    ) -> Result<ModelObservationChannelTimingPolicy, ModelObservationRejectionCode> {
        self.bindings
            .iter()
            .find(|binding| binding.channel == channel)
            .map(ModelObservationTimingBinding::policy)
            .ok_or(ModelObservationRejectionCode::TimingContractUnknownChannel)
    }

    /// Admit an observation event under the channel policy and assertion boundary.
    ///
    /// Exact rule:
    /// - unknown assertion boundary → fail closed;
    /// - observation at the assertion boundary → fail closed;
    /// - `RequireStrictlyBeforeAssertion` admits only `event_index < first_assertion`;
    /// - `AllowPostHocDiagnostic` admits any non-boundary index (primary vs post-hoc
    ///   classification is deferred to the H11-A eligibility rule).
    pub fn admit_observation(
        &self,
        channel: &str,
        event_index: u64,
        first_assertion_event: Option<u64>,
    ) -> Result<(), ModelObservationRejectionCode> {
        let policy = self.policy_of(channel)?;
        let Some(first_assertion) = first_assertion_event else {
            return Err(ModelObservationRejectionCode::EligibilityAssertionBoundaryUnknown);
        };
        if event_index == first_assertion {
            return Err(ModelObservationRejectionCode::TimingObservationAtAssertionBoundary);
        }
        match policy {
            ModelObservationChannelTimingPolicy::RequireStrictlyBeforeAssertion => {
                if event_index < first_assertion {
                    Ok(())
                } else {
                    Err(ModelObservationRejectionCode::EligibilityPrimaryRequiresStrictPreAssertion)
                }
            }
            ModelObservationChannelTimingPolicy::AllowPostHocDiagnostic => Ok(()),
        }
    }

    /// Byte-length framed machine-readable timing-contract record.
    #[must_use]
    pub fn canonical_record(&self) -> String {
        let mut out = String::from(MODEL_OBSERVATION_TIMING_CONTRACT_SCHEMA);
        out.push_str("\nkind:timing_contract\n");
        out.push_str(&format!("binding_count:{}\n", self.bindings.len()));
        for binding in &self.bindings {
            out.push_str(&format!(
                "channel:{}{}:policy:{}\n",
                binding.channel.len(),
                binding.channel,
                binding.policy.as_str()
            ));
        }
        out
    }
}

/// Exact H11-A pre-assertion eligibility classifier (candidate only).
///
/// PrimaryEligible iff `event_index < first_assertion_event`. PostHocOnly when
/// `event_index > first_assertion_event`. Observation at the assertion boundary
/// and unknown boundaries fail closed. Does **not** pin
/// `primary_pre_assertion_eligibility`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ModelObservationH11AEligibilityRule;

impl ModelObservationH11AEligibilityRule {
    #[must_use]
    pub const fn candidate_id() -> &'static str {
        CANDIDATE_H11A_ELIGIBILITY_RULE
    }

    /// Exact eligibility classification under a known assertion boundary.
    pub fn classify(
        event_index: u64,
        first_assertion_event: Option<u64>,
    ) -> Result<PrecursorEligibility, ModelObservationRejectionCode> {
        let Some(first_assertion) = first_assertion_event else {
            return Err(ModelObservationRejectionCode::EligibilityAssertionBoundaryUnknown);
        };
        if event_index == first_assertion {
            return Err(ModelObservationRejectionCode::TimingObservationAtAssertionBoundary);
        }
        if event_index < first_assertion {
            Ok(PrecursorEligibility::PrimaryEligible)
        } else {
            Ok(PrecursorEligibility::PostHocOnly)
        }
    }

    /// Whether a classified eligibility admits primary H11-A use.
    #[must_use]
    pub const fn admits_primary(eligibility: PrecursorEligibility) -> bool {
        matches!(eligibility, PrecursorEligibility::PrimaryEligible)
    }

    /// Byte-length framed machine-readable eligibility decision record.
    #[must_use]
    pub fn canonical_record(
        event_index: u64,
        first_assertion_event: Option<u64>,
        eligibility: Result<PrecursorEligibility, ModelObservationRejectionCode>,
    ) -> String {
        let mut out = String::from(MODEL_OBSERVATION_H11A_ELIGIBILITY_SCHEMA);
        out.push_str("\nkind:h11a_eligibility\n");
        out.push_str(&format!("candidate:{CANDIDATE_H11A_ELIGIBILITY_RULE}\n"));
        out.push_str(&format!("event_index:{event_index}\n"));
        match first_assertion_event {
            Some(index) => out.push_str(&format!("first_assertion_event:{index}\n")),
            None => out.push_str("first_assertion_event:none\n"),
        }
        match eligibility {
            Ok(PrecursorEligibility::PrimaryEligible) => {
                out.push_str("eligibility:primary_eligible\nadmits_primary:true\n");
            }
            Ok(PrecursorEligibility::PostHocOnly) => {
                out.push_str("eligibility:post_hoc_only\nadmits_primary:false\n");
            }
            Err(code) => out.push_str(&format!(
                "eligibility:rejected\nrejection:{}:{:#06x}\n",
                code.as_str(),
                code.numeric()
            )),
        }
        out
    }
}

impl fmt::Display for ModelObservationChannelTimingPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
