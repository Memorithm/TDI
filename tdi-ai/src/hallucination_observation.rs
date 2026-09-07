//! Non-final TDI-11.1 prospective observation and timing contract.
//!
//! H11-A requires primary precursor evidence to exist before the first event of
//! the scored assertion. This module makes that temporal relation structural:
//! callers may append only monotonically ordered observable signals, the first
//! assertion boundary can be established exactly once, and primary eligibility
//! is derived from event order rather than accepted as a caller-supplied flag.
//!
//! The exact observation vector is intentionally *not* frozen here. Signal
//! channels are opaque versioned names so later non-final adapters can be
//! compared without granting any channel special scientific status. This type
//! contains no complete-world truth, evaluator label, hidden difficulty, final
//! seed material, future state, or alternative-arm outcome.

use core::fmt;

/// Version of the non-final timing carrier, not a final observation-vector hash.
pub const OBSERVATION_TIMING_SCHEMA: &str = "tdi11-prospective-observation-v1";

/// Opaque model/runtime-visible signal name.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SignalChannel(String);

impl SignalChannel {
    /// Construct one non-empty opaque channel identifier.
    pub fn new(name: impl Into<String>) -> Result<Self, ObservationContractError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(ObservationContractError::EmptySignalChannel);
        }
        Ok(Self(name))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One finite scalar observable available at a declared trajectory event.
#[derive(Clone, Debug, PartialEq)]
pub struct ObservableSignal {
    channel: SignalChannel,
    value: f64,
}

impl ObservableSignal {
    pub fn new(channel: SignalChannel, value: f64) -> Result<Self, ObservationContractError> {
        if !value.is_finite() {
            return Err(ObservationContractError::NonFiniteSignal);
        }
        Ok(Self { channel, value })
    }

    #[must_use]
    pub fn channel(&self) -> &SignalChannel {
        &self.channel
    }

    #[must_use]
    pub const fn value(&self) -> f64 {
        self.value
    }
}

/// One ordered observation event. No evaluator-owned outcome is representable.
#[derive(Clone, Debug, PartialEq)]
pub struct ObservationEvent {
    event_index: u64,
    signals: Vec<ObservableSignal>,
}

impl ObservationEvent {
    #[must_use]
    pub const fn event_index(&self) -> u64 {
        self.event_index
    }

    #[must_use]
    pub fn signals(&self) -> &[ObservableSignal] {
        &self.signals
    }
}

/// Timing classification derived after the first assertion boundary is known.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PrecursorEligibility {
    /// All signal inputs existed strictly before the first assertion event.
    PrimaryEligible,
    /// Observation occurred after the first assertion boundary and is diagnostic only.
    PostHocOnly,
}

/// Evaluator-verifiable timing record for one stored observation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ObservationTimingRecord {
    event_index: u64,
    first_assertion_event: u64,
    eligibility: PrecursorEligibility,
}

impl ObservationTimingRecord {
    #[must_use]
    pub const fn event_index(self) -> u64 {
        self.event_index
    }

    #[must_use]
    pub const fn first_assertion_event(self) -> u64 {
        self.first_assertion_event
    }

    #[must_use]
    pub const fn eligibility(self) -> PrecursorEligibility {
        self.eligibility
    }

    /// Frozen TDI-11.0 terminology, derived rather than caller supplied.
    #[must_use]
    pub const fn observed_before_claim(self) -> bool {
        matches!(self.eligibility, PrecursorEligibility::PrimaryEligible)
    }
}

/// Append-only non-final trajectory timing journal.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ObservationTimeline {
    observations: Vec<ObservationEvent>,
    first_assertion_event: Option<u64>,
}

impl ObservationTimeline {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            observations: Vec::new(),
            first_assertion_event: None,
        }
    }

    /// Append one observation with a strictly increasing event index.
    ///
    /// Post-assertion observations remain allowed because TDI-11 preserves
    /// post-hoc diagnostics; they are simply ineligible for H11-A primary use.
    pub fn push_observation(
        &mut self,
        event_index: u64,
        signals: Vec<ObservableSignal>,
    ) -> Result<(), ObservationContractError> {
        if signals.is_empty() {
            return Err(ObservationContractError::EmptySignalVector);
        }
        if let Some(previous) = self.observations.last() {
            if event_index <= previous.event_index {
                return Err(ObservationContractError::NonMonotonicEventIndex {
                    previous: previous.event_index,
                    attempted: event_index,
                });
            }
        }
        if self.first_assertion_event == Some(event_index) {
            return Err(ObservationContractError::ObservationAtAssertionBoundary {
                event_index,
            });
        }
        self.observations.push(ObservationEvent {
            event_index,
            signals,
        });
        Ok(())
    }

    /// Establish the first token/event of the atomic assertion exactly once.
    ///
    /// The boundary cannot be backdated behind already observed events. Once
    /// established it is immutable, preventing result-conditioned retiming.
    pub fn mark_first_assertion(
        &mut self,
        event_index: u64,
    ) -> Result<(), ObservationContractError> {
        if let Some(existing) = self.first_assertion_event {
            return Err(ObservationContractError::AssertionBoundaryAlreadySet {
                existing,
                attempted: event_index,
            });
        }
        if let Some(previous) = self.observations.last() {
            if event_index <= previous.event_index {
                return Err(ObservationContractError::AssertionBoundaryNotAfterHistory {
                    last_observation: previous.event_index,
                    attempted: event_index,
                });
            }
        }
        self.first_assertion_event = Some(event_index);
        Ok(())
    }

    #[must_use]
    pub fn observations(&self) -> &[ObservationEvent] {
        &self.observations
    }

    #[must_use]
    pub const fn first_assertion_event(&self) -> Option<u64> {
        self.first_assertion_event
    }

    /// Derive timing eligibility for every retained observation.
    pub fn timing_records(
        &self,
    ) -> Result<Vec<ObservationTimingRecord>, ObservationContractError> {
        let boundary = self
            .first_assertion_event
            .ok_or(ObservationContractError::AssertionBoundaryUnknown)?;
        Ok(self
            .observations
            .iter()
            .map(|observation| ObservationTimingRecord {
                event_index: observation.event_index,
                first_assertion_event: boundary,
                eligibility: if observation.event_index < boundary {
                    PrecursorEligibility::PrimaryEligible
                } else {
                    PrecursorEligibility::PostHocOnly
                },
            })
            .collect())
    }

    /// Return only H11-A-primary-eligible observations after timing is established.
    pub fn primary_precursor_events(
        &self,
    ) -> Result<Vec<&ObservationEvent>, ObservationContractError> {
        let boundary = self
            .first_assertion_event
            .ok_or(ObservationContractError::AssertionBoundaryUnknown)?;
        Ok(self
            .observations
            .iter()
            .filter(|observation| observation.event_index < boundary)
            .collect())
    }

    /// Deterministic, non-secret timing provenance without signal values.
    ///
    /// Signal values are intentionally omitted so this carrier can be attached
    /// to evaluator provenance without becoming a result payload.
    pub fn timing_provenance(&self) -> Result<String, ObservationContractError> {
        let boundary = self
            .first_assertion_event
            .ok_or(ObservationContractError::AssertionBoundaryUnknown)?;
        let indices = self
            .observations
            .iter()
            .map(|event| event.event_index.to_string())
            .collect::<Vec<_>>()
            .join(",");
        Ok(format!(
            "{OBSERVATION_TIMING_SCHEMA};first_assertion_event={boundary};observation_events={indices}"
        ))
    }
}

/// Fail-closed violations of the prospective timing carrier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObservationContractError {
    EmptySignalChannel,
    NonFiniteSignal,
    EmptySignalVector,
    NonMonotonicEventIndex { previous: u64, attempted: u64 },
    ObservationAtAssertionBoundary { event_index: u64 },
    AssertionBoundaryAlreadySet { existing: u64, attempted: u64 },
    AssertionBoundaryNotAfterHistory { last_observation: u64, attempted: u64 },
    AssertionBoundaryUnknown,
}

impl fmt::Display for ObservationContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for ObservationContractError {}
