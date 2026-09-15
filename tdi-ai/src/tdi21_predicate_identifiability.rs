//! Evaluator-only identifiability audit for the TDI-21 B4 v1 predicate map.
//!
//! The candidate-side predicate adapter remains causal. This module is separate:
//! it is allowed to inspect a declared future probe solely to ask whether two
//! causal states that look identical to the candidate can require opposite
//! hindsight-optimal write actions. Hindsight labels produced here MUST NOT be
//! supplied to a final/holdout candidate and are not runtime observations.

use super::tdi21::{BooleanState, MemoryRead};
use super::tdi21_event_predicates::{
    WritePredicateError, WritePredicateVector, encode_b4_write_predicates,
};
use super::tdi21_stream::{
    BooleanStream, Event, MemoryMode, StepOutput, StreamConfig, StreamError,
};

pub const IDENTIFIABILITY_SEMANTICS: &str = "tdi21-b4-local-predicate-identifiability-v1";
pub const MAX_AUDIT_PREFIX_EVENTS: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdmissionAction {
    Admit,
    Inhibit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FutureRecallProbe {
    pub key: u64,
    pub expected: MemoryRead,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdmissionAuditCase {
    pub prefix: Vec<Event>,
    pub decision: Event,
    pub probe: FutureRecallProbe,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AdmissionOutcome {
    pub action: AdmissionAction,
    pub observed: MemoryRead,
    pub succeeds: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AdmissionAudit {
    pub predicates: WritePredicateVector,
    pub admit: AdmissionOutcome,
    pub inhibit: AdmissionOutcome,
    pub unique_hindsight_label: Option<AdmissionAction>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IdentifiabilityError {
    RequiresTwoWayB3,
    PrefixTooLong,
    DecisionMustBeAdmittedWrite,
    InvalidProbeExpected,
    Stream(StreamError),
    Predicate(WritePredicateError),
    UnexpectedQuietProbe,
}

impl From<StreamError> for IdentifiabilityError {
    fn from(value: StreamError) -> Self {
        Self::Stream(value)
    }
}

impl From<WritePredicateError> for IdentifiabilityError {
    fn from(value: WritePredicateError) -> Self {
        Self::Predicate(value)
    }
}

fn decision_parts(event: Event) -> Result<(u64, BooleanState), IdentifiabilityError> {
    match event {
        Event::Write {
            key,
            payload,
            marker,
        } if marker.bits() == 1 => Ok((key, payload)),
        _ => Err(IdentifiabilityError::DecisionMustBeAdmittedWrite),
    }
}

fn apply_action(
    mut stream: BooleanStream,
    key: u64,
    payload: BooleanState,
    action: AdmissionAction,
    probe: FutureRecallProbe,
) -> Result<AdmissionOutcome, IdentifiabilityError> {
    let marker = match action {
        AdmissionAction::Admit => 1,
        AdmissionAction::Inhibit => 3,
    };
    stream.step(Event::Write {
        key,
        payload,
        marker: BooleanState::from_bits(marker),
    })?;
    let observed = match stream.step(Event::Recall { key: probe.key })? {
        StepOutput::Reply(value) => value,
        StepOutput::Quiet => return Err(IdentifiabilityError::UnexpectedQuietProbe),
    };
    Ok(AdmissionOutcome {
        action,
        observed,
        succeeds: observed == probe.expected,
    })
}

/// Compare Admit and Inhibit from the same causal prefix and current Write.
///
/// The predicate vector is captured before either action by the production-side
/// adapter, which derives the bucket observation from the decision's own key.
/// The declared future recall is evaluator-only and is never encoded into that
/// vector.
pub fn audit_admission_case(
    config: StreamConfig,
    case: &AdmissionAuditCase,
) -> Result<AdmissionAudit, IdentifiabilityError> {
    if config.mode != MemoryMode::TwoWay {
        return Err(IdentifiabilityError::RequiresTwoWayB3);
    }
    if case.prefix.len() > MAX_AUDIT_PREFIX_EVENTS {
        return Err(IdentifiabilityError::PrefixTooLong);
    }
    if matches!(case.probe.expected, MemoryRead::Miss) {
        return Err(IdentifiabilityError::InvalidProbeExpected);
    }
    let (key, payload) = decision_parts(case.decision)?;
    let mut stream = BooleanStream::new(config)?;
    for &event in &case.prefix {
        stream.step(event)?;
    }
    let predicates = encode_b4_write_predicates(&mut stream, case.decision)?;

    let admit = apply_action(
        stream.clone(),
        key,
        payload,
        AdmissionAction::Admit,
        case.probe,
    )?;
    let inhibit = apply_action(stream, key, payload, AdmissionAction::Inhibit, case.probe)?;
    let unique_hindsight_label = match (admit.succeeds, inhibit.succeeds) {
        (true, false) => Some(AdmissionAction::Admit),
        (false, true) => Some(AdmissionAction::Inhibit),
        _ => None,
    };
    Ok(AdmissionAudit {
        predicates,
        admit,
        inhibit,
        unique_hindsight_label,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PredicateConflict {
    pub predicates: WritePredicateVector,
    pub first_label: AdmissionAction,
    pub second_label: AdmissionAction,
}

/// Return a concrete non-identifiability witness when the candidate observes
/// the same predicate vector but the evaluator finds opposite unique labels.
#[must_use]
pub fn conflicting_pair(
    first: AdmissionAudit,
    second: AdmissionAudit,
) -> Option<PredicateConflict> {
    let first_label = first.unique_hindsight_label?;
    let second_label = second.unique_hindsight_label?;
    if first.predicates == second.predicates && first_label != second_label {
        Some(PredicateConflict {
            predicates: first.predicates,
            first_label,
            second_label,
        })
    } else {
        None
    }
}
