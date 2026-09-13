//! Non-executing TDI-11.2 model-observation resource accounting scaffolding.
//!
//! This layer declares the exact *component taxonomy* and fail-closed
//! overflow / envelope-admission rules for prospective model-observation
//! instrumentation. It does **not**:
//!
//! - invent numeric freeze envelopes (all maxima are caller-supplied);
//! - pin `resource_accounting_contract`;
//! - transfer TDI-11.1 `ReferenceResourceEnvelope::unlimited_for_development`
//!   into a TDI-11.2 freeze value;
//! - load or execute a concrete model;
//! - set `model_execution_authorized` / `final_execution_authorized`.
//!
//! Presence of this scaffold is a non-authorizing candidate only.

use core::fmt;

use super::hallucination_model_observation_rejections::ModelObservationRejectionCode;

/// Scaffold schema marker for model-observation resource accounting records.
pub const MODEL_OBSERVATION_RESOURCE_ACCOUNTING_SCHEMA: &str =
    "tdi11.2-model-observation-resource-accounting-v0";

/// Non-authorizing candidate identifier for the resource-accounting freeze field.
///
/// Matching this string in docs/CI does **not** pin
/// `resource_accounting_contract`. A later reviewed freeze must still replace
/// `unresolved_blocking` with explicit numeric rules.
pub const CANDIDATE_RESOURCE_ACCOUNTING_CONTRACT: &str =
    "ModelObservationResourceAccountingContract";

/// Exact ordered component keys counted by the scaffolding meter.
///
/// This taxonomy is the EXACT software contract surface. Numeric maxima are
/// deliberately absent from the freeze and must remain caller-supplied.
pub const RESOURCE_ACCOUNTING_COMPONENT_KEYS: &[&str] = &[
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
];

/// Fail-closed component-wise usage for one non-final model-observation trace.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ModelObservationResourceUsage {
    model_input_tokens: u64,
    model_output_tokens: u64,
    model_decode_steps: u64,
    adapter_ingest_events: u64,
    adapter_declared_channel_emissions: u64,
    observation_packet_bytes: u64,
    timing_contract_checks: u64,
    provenance_frame_bytes: u64,
    verifier_calls: u64,
    retrieval_tool_calls: u64,
    resamples: u64,
}

impl ModelObservationResourceUsage {
    /// Zero usage.
    #[must_use]
    pub const fn zero() -> Self {
        Self {
            model_input_tokens: 0,
            model_output_tokens: 0,
            model_decode_steps: 0,
            adapter_ingest_events: 0,
            adapter_declared_channel_emissions: 0,
            observation_packet_bytes: 0,
            timing_contract_checks: 0,
            provenance_frame_bytes: 0,
            verifier_calls: 0,
            retrieval_tool_calls: 0,
            resamples: 0,
        }
    }

    fn checked_add_field(current: u64, delta: u64) -> Result<u64, ModelObservationRejectionCode> {
        current
            .checked_add(delta)
            .ok_or(ModelObservationRejectionCode::AccountingOverflow)
    }

    /// Charge model token counters (fail-closed on overflow).
    pub fn charge_model_tokens(
        &mut self,
        input: u64,
        output: u64,
    ) -> Result<(), ModelObservationRejectionCode> {
        self.model_input_tokens = Self::checked_add_field(self.model_input_tokens, input)?;
        self.model_output_tokens = Self::checked_add_field(self.model_output_tokens, output)?;
        Ok(())
    }

    /// Charge decode steps.
    pub fn charge_decode_steps(&mut self, steps: u64) -> Result<(), ModelObservationRejectionCode> {
        self.model_decode_steps = Self::checked_add_field(self.model_decode_steps, steps)?;
        Ok(())
    }

    /// Charge one adapter ingest event and its declared channel emissions / payload.
    pub fn charge_adapter_ingest(
        &mut self,
        declared_channel_emissions: u64,
        packet_bytes: u64,
    ) -> Result<(), ModelObservationRejectionCode> {
        self.adapter_ingest_events = Self::checked_add_field(self.adapter_ingest_events, 1)?;
        self.adapter_declared_channel_emissions = Self::checked_add_field(
            self.adapter_declared_channel_emissions,
            declared_channel_emissions,
        )?;
        self.observation_packet_bytes =
            Self::checked_add_field(self.observation_packet_bytes, packet_bytes)?;
        Ok(())
    }

    /// Charge a prospective timing-contract check.
    pub fn charge_timing_contract_check(&mut self) -> Result<(), ModelObservationRejectionCode> {
        self.timing_contract_checks = Self::checked_add_field(self.timing_contract_checks, 1)?;
        Ok(())
    }

    /// Charge provenance framing work in bytes.
    pub fn charge_provenance_frame_bytes(
        &mut self,
        bytes: u64,
    ) -> Result<(), ModelObservationRejectionCode> {
        self.provenance_frame_bytes = Self::checked_add_field(self.provenance_frame_bytes, bytes)?;
        Ok(())
    }

    /// Charge verifier / retrieval / resample counters.
    pub fn charge_secondary(
        &mut self,
        verifier_calls: u64,
        retrieval_tool_calls: u64,
        resamples: u64,
    ) -> Result<(), ModelObservationRejectionCode> {
        self.verifier_calls = Self::checked_add_field(self.verifier_calls, verifier_calls)?;
        self.retrieval_tool_calls =
            Self::checked_add_field(self.retrieval_tool_calls, retrieval_tool_calls)?;
        self.resamples = Self::checked_add_field(self.resamples, resamples)?;
        Ok(())
    }

    #[must_use]
    pub const fn model_input_tokens(self) -> u64 {
        self.model_input_tokens
    }

    #[must_use]
    pub const fn model_output_tokens(self) -> u64 {
        self.model_output_tokens
    }

    #[must_use]
    pub const fn model_decode_steps(self) -> u64 {
        self.model_decode_steps
    }

    #[must_use]
    pub const fn adapter_ingest_events(self) -> u64 {
        self.adapter_ingest_events
    }

    #[must_use]
    pub const fn adapter_declared_channel_emissions(self) -> u64 {
        self.adapter_declared_channel_emissions
    }

    #[must_use]
    pub const fn observation_packet_bytes(self) -> u64 {
        self.observation_packet_bytes
    }

    #[must_use]
    pub const fn timing_contract_checks(self) -> u64 {
        self.timing_contract_checks
    }

    #[must_use]
    pub const fn provenance_frame_bytes(self) -> u64 {
        self.provenance_frame_bytes
    }

    #[must_use]
    pub const fn verifier_calls(self) -> u64 {
        self.verifier_calls
    }

    #[must_use]
    pub const fn retrieval_tool_calls(self) -> u64 {
        self.retrieval_tool_calls
    }

    #[must_use]
    pub const fn resamples(self) -> u64 {
        self.resamples
    }

    /// Exact checked sum of all component counters.
    pub fn total(self) -> Result<u64, ModelObservationRejectionCode> {
        let mut acc = 0u64;
        for value in [
            self.model_input_tokens,
            self.model_output_tokens,
            self.model_decode_steps,
            self.adapter_ingest_events,
            self.adapter_declared_channel_emissions,
            self.observation_packet_bytes,
            self.timing_contract_checks,
            self.provenance_frame_bytes,
            self.verifier_calls,
            self.retrieval_tool_calls,
            self.resamples,
        ] {
            acc = Self::checked_add_field(acc, value)?;
        }
        Ok(acc)
    }

    /// Byte-length framed machine-readable usage record.
    #[must_use]
    pub fn canonical_record(self) -> String {
        format!(
            "{MODEL_OBSERVATION_RESOURCE_ACCOUNTING_SCHEMA}\nkind:usage\n\
model_input_tokens:{}\nmodel_output_tokens:{}\nmodel_decode_steps:{}\n\
adapter_ingest_events:{}\nadapter_declared_channel_emissions:{}\n\
observation_packet_bytes:{}\ntiming_contract_checks:{}\n\
provenance_frame_bytes:{}\nverifier_calls:{}\nretrieval_tool_calls:{}\nresamples:{}\n",
            self.model_input_tokens,
            self.model_output_tokens,
            self.model_decode_steps,
            self.adapter_ingest_events,
            self.adapter_declared_channel_emissions,
            self.observation_packet_bytes,
            self.timing_contract_checks,
            self.provenance_frame_bytes,
            self.verifier_calls,
            self.retrieval_tool_calls,
            self.resamples,
        )
    }
}

/// Caller-supplied envelope over [`ModelObservationResourceUsage`].
///
/// Constructing an envelope never freezes numeric scientific values. The
/// helper [`Self::unbounded_caller_supplied`] is an engineering convenience
/// for Development/Validation scaffolding only and is **not** a freeze pin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModelObservationResourceEnvelope {
    max_model_input_tokens: u64,
    max_model_output_tokens: u64,
    max_model_decode_steps: u64,
    max_adapter_ingest_events: u64,
    max_adapter_declared_channel_emissions: u64,
    max_observation_packet_bytes: u64,
    max_timing_contract_checks: u64,
    max_provenance_frame_bytes: u64,
    max_verifier_calls: u64,
    max_retrieval_tool_calls: u64,
    max_resamples: u64,
}

impl ModelObservationResourceEnvelope {
    /// Explicit caller-supplied constructor (all maxima required).
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        max_model_input_tokens: u64,
        max_model_output_tokens: u64,
        max_model_decode_steps: u64,
        max_adapter_ingest_events: u64,
        max_adapter_declared_channel_emissions: u64,
        max_observation_packet_bytes: u64,
        max_timing_contract_checks: u64,
        max_provenance_frame_bytes: u64,
        max_verifier_calls: u64,
        max_retrieval_tool_calls: u64,
        max_resamples: u64,
    ) -> Self {
        Self {
            max_model_input_tokens,
            max_model_output_tokens,
            max_model_decode_steps,
            max_adapter_ingest_events,
            max_adapter_declared_channel_emissions,
            max_observation_packet_bytes,
            max_timing_contract_checks,
            max_provenance_frame_bytes,
            max_verifier_calls,
            max_retrieval_tool_calls,
            max_resamples,
        }
    }

    /// Engineering unbounded envelope for non-final scaffolding tests.
    ///
    /// This is **not** a TDI-11.2 freeze value and does **not** transfer from
    /// TDI-11.1 `ReferenceResourceEnvelope::unlimited_for_development`.
    #[must_use]
    pub const fn unbounded_caller_supplied() -> Self {
        Self::new(
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
            u64::MAX,
        )
    }

    /// Fail-closed admission of usage under this envelope.
    pub fn admit(
        self,
        usage: ModelObservationResourceUsage,
    ) -> Result<(), ModelObservationRejectionCode> {
        let pairs = [
            (usage.model_input_tokens(), self.max_model_input_tokens),
            (usage.model_output_tokens(), self.max_model_output_tokens),
            (usage.model_decode_steps(), self.max_model_decode_steps),
            (
                usage.adapter_ingest_events(),
                self.max_adapter_ingest_events,
            ),
            (
                usage.adapter_declared_channel_emissions(),
                self.max_adapter_declared_channel_emissions,
            ),
            (
                usage.observation_packet_bytes(),
                self.max_observation_packet_bytes,
            ),
            (
                usage.timing_contract_checks(),
                self.max_timing_contract_checks,
            ),
            (
                usage.provenance_frame_bytes(),
                self.max_provenance_frame_bytes,
            ),
            (usage.verifier_calls(), self.max_verifier_calls),
            (usage.retrieval_tool_calls(), self.max_retrieval_tool_calls),
            (usage.resamples(), self.max_resamples),
        ];
        for (used, max) in pairs {
            if used > max {
                return Err(ModelObservationRejectionCode::AccountingEnvelopeExceeded);
            }
        }
        Ok(())
    }

    /// Byte-length framed machine-readable envelope record.
    #[must_use]
    pub fn canonical_record(self) -> String {
        format!(
            "{MODEL_OBSERVATION_RESOURCE_ACCOUNTING_SCHEMA}\nkind:envelope\n\
max_model_input_tokens:{}\nmax_model_output_tokens:{}\nmax_model_decode_steps:{}\n\
max_adapter_ingest_events:{}\nmax_adapter_declared_channel_emissions:{}\n\
max_observation_packet_bytes:{}\nmax_timing_contract_checks:{}\n\
max_provenance_frame_bytes:{}\nmax_verifier_calls:{}\nmax_retrieval_tool_calls:{}\nmax_resamples:{}\n",
            self.max_model_input_tokens,
            self.max_model_output_tokens,
            self.max_model_decode_steps,
            self.max_adapter_ingest_events,
            self.max_adapter_declared_channel_emissions,
            self.max_observation_packet_bytes,
            self.max_timing_contract_checks,
            self.max_provenance_frame_bytes,
            self.max_verifier_calls,
            self.max_retrieval_tool_calls,
            self.max_resamples,
        )
    }
}

/// Immutable accounting rejection retaining the exceeded component key when known.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelObservationAccountingRejection {
    code: ModelObservationRejectionCode,
    component: Option<&'static str>,
}

impl ModelObservationAccountingRejection {
    #[must_use]
    pub const fn overflow() -> Self {
        Self {
            code: ModelObservationRejectionCode::AccountingOverflow,
            component: None,
        }
    }

    #[must_use]
    pub const fn envelope_exceeded(component: &'static str) -> Self {
        Self {
            code: ModelObservationRejectionCode::AccountingEnvelopeExceeded,
            component: Some(component),
        }
    }

    #[must_use]
    pub const fn code(&self) -> ModelObservationRejectionCode {
        self.code
    }

    #[must_use]
    pub const fn component(&self) -> Option<&'static str> {
        self.component
    }

    #[must_use]
    pub fn canonical_record(&self) -> String {
        format!(
            "{MODEL_OBSERVATION_RESOURCE_ACCOUNTING_SCHEMA}\nkind:rejection\ncode:{}:{:#06x}\ncomponent:{}\n",
            self.code.as_str(),
            self.code.numeric(),
            self.component.unwrap_or("none"),
        )
    }
}

impl fmt::Display for ModelObservationAccountingRejection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.component {
            Some(component) => write!(formatter, "{} ({component})", self.code),
            None => write!(formatter, "{}", self.code),
        }
    }
}

/// Exact component-count identity used by CI (must match the key table).
pub const RESOURCE_ACCOUNTING_COMPONENT_COUNT: usize = 11;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn component_key_table_is_exact_and_stable() {
        assert_eq!(
            RESOURCE_ACCOUNTING_COMPONENT_KEYS.len(),
            RESOURCE_ACCOUNTING_COMPONENT_COUNT
        );
        assert_eq!(RESOURCE_ACCOUNTING_COMPONENT_KEYS[0], "model_input_tokens");
        assert_eq!(RESOURCE_ACCOUNTING_COMPONENT_KEYS[10], "resamples");
    }
}
