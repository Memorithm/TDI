#[path = "../src/hallucination_observation.rs"]
mod hallucination_observation;

use hallucination_observation::{
    OBSERVATION_TIMING_SCHEMA, ObservableSignal, ObservationContractError, ObservationTimeline,
    PrecursorEligibility, SignalChannel,
};

fn signal(name: &str, value: f64) -> ObservableSignal {
    ObservableSignal::new(SignalChannel::new(name).expect("valid channel"), value)
        .expect("finite signal")
}

#[test]
fn signal_contract_rejects_empty_channels_and_nonfinite_values() {
    assert_eq!(
        SignalChannel::new("   "),
        Err(ObservationContractError::EmptySignalChannel)
    );

    let channel = SignalChannel::new("token_entropy.mean").expect("valid channel");
    assert_eq!(channel.as_str(), "token_entropy.mean");
    assert_eq!(
        ObservableSignal::new(channel.clone(), f64::NAN),
        Err(ObservationContractError::NonFiniteSignal)
    );
    assert_eq!(
        ObservableSignal::new(channel, f64::INFINITY),
        Err(ObservationContractError::NonFiniteSignal)
    );
}

#[test]
fn observations_are_strictly_monotonic_and_nonempty() {
    let mut timeline = ObservationTimeline::new();
    assert_eq!(
        timeline.push_observation(1, Vec::new()),
        Err(ObservationContractError::EmptySignalVector)
    );

    timeline
        .push_observation(2, vec![signal("margin", 0.4)])
        .expect("first observation");
    assert_eq!(
        timeline.push_observation(2, vec![signal("margin", 0.5)]),
        Err(ObservationContractError::NonMonotonicEventIndex {
            previous: 2,
            attempted: 2,
        })
    );
    assert_eq!(
        timeline.push_observation(1, vec![signal("margin", 0.5)]),
        Err(ObservationContractError::NonMonotonicEventIndex {
            previous: 2,
            attempted: 1,
        })
    );
}

#[test]
fn assertion_boundary_cannot_be_backdated_or_rewritten() {
    let mut timeline = ObservationTimeline::new();
    timeline
        .push_observation(3, vec![signal("residual", 0.2)])
        .expect("observation");

    assert_eq!(
        timeline.mark_first_assertion(3),
        Err(ObservationContractError::AssertionBoundaryNotAfterHistory {
            last_observation: 3,
            attempted: 3,
        })
    );
    assert_eq!(
        timeline.mark_first_assertion(2),
        Err(ObservationContractError::AssertionBoundaryNotAfterHistory {
            last_observation: 3,
            attempted: 2,
        })
    );

    timeline
        .mark_first_assertion(4)
        .expect("boundary after history");
    assert_eq!(timeline.first_assertion_event(), Some(4));
    assert_eq!(
        timeline.mark_first_assertion(5),
        Err(ObservationContractError::AssertionBoundaryAlreadySet {
            existing: 4,
            attempted: 5,
        })
    );
}

#[test]
fn primary_eligibility_is_derived_from_event_order() {
    let mut timeline = ObservationTimeline::new();
    timeline
        .push_observation(1, vec![signal("entropy", 0.8), signal("margin", 0.1)])
        .expect("pre-claim observation");
    timeline
        .push_observation(3, vec![signal("entropy", 0.7)])
        .expect("pre-claim observation");
    timeline
        .mark_first_assertion(5)
        .expect("first assertion boundary");
    timeline
        .push_observation(6, vec![signal("entropy", 0.2)])
        .expect("post-hoc diagnostic");

    let records = timeline.timing_records().expect("boundary is known");
    assert_eq!(records.len(), 3);

    assert_eq!(records[0].event_index(), 1);
    assert_eq!(records[0].first_assertion_event(), 5);
    assert_eq!(
        records[0].eligibility(),
        PrecursorEligibility::PrimaryEligible
    );
    assert!(records[0].observed_before_claim());

    assert_eq!(records[1].event_index(), 3);
    assert_eq!(
        records[1].eligibility(),
        PrecursorEligibility::PrimaryEligible
    );
    assert!(records[1].observed_before_claim());

    assert_eq!(records[2].event_index(), 6);
    assert_eq!(records[2].eligibility(), PrecursorEligibility::PostHocOnly);
    assert!(!records[2].observed_before_claim());

    let primary = timeline
        .primary_precursor_events()
        .expect("boundary is known");
    assert_eq!(primary.len(), 2);
    assert_eq!(primary[0].event_index(), 1);
    assert_eq!(primary[1].event_index(), 3);
}

#[test]
fn observation_at_assertion_boundary_is_rejected() {
    let mut timeline = ObservationTimeline::new();
    timeline
        .push_observation(2, vec![signal("margin", 0.3)])
        .expect("pre-claim observation");
    timeline.mark_first_assertion(4).expect("boundary");

    assert_eq!(
        timeline.push_observation(4, vec![signal("margin", 0.2)]),
        Err(ObservationContractError::ObservationAtAssertionBoundary { event_index: 4 })
    );
}

#[test]
fn timing_queries_fail_closed_until_claim_boundary_is_known() {
    let mut timeline = ObservationTimeline::new();
    timeline
        .push_observation(1, vec![signal("state.delta", 0.05)])
        .expect("observation");

    assert_eq!(
        timeline.timing_records(),
        Err(ObservationContractError::AssertionBoundaryUnknown)
    );
    assert_eq!(
        timeline.primary_precursor_events(),
        Err(ObservationContractError::AssertionBoundaryUnknown)
    );
    assert_eq!(
        timeline.timing_provenance(),
        Err(ObservationContractError::AssertionBoundaryUnknown)
    );
}

#[test]
fn signal_values_are_visible_but_timing_provenance_contains_only_indices() {
    let mut timeline = ObservationTimeline::new();
    timeline
        .push_observation(7, vec![signal("adapter.hidden_state_norm", 123.456)])
        .expect("observation");
    timeline.mark_first_assertion(8).expect("boundary");

    let observations = timeline.observations();
    assert_eq!(observations.len(), 1);
    assert_eq!(observations[0].event_index(), 7);
    assert_eq!(observations[0].signals().len(), 1);
    assert_eq!(
        observations[0].signals()[0].channel().as_str(),
        "adapter.hidden_state_norm"
    );
    assert_eq!(observations[0].signals()[0].value(), 123.456);

    let provenance = timeline.timing_provenance().expect("boundary known");
    assert_eq!(
        provenance,
        format!("{OBSERVATION_TIMING_SCHEMA};first_assertion_event=8;observation_events=7")
    );
    assert!(!provenance.contains("123.456"));
    assert!(!provenance.contains("hidden_state_norm"));
}

#[test]
fn posthoc_events_are_retained_without_becoming_primary_evidence() {
    let mut timeline = ObservationTimeline::new();
    timeline.mark_first_assertion(10).expect("boundary");
    timeline
        .push_observation(11, vec![signal("self_check", 1.0)])
        .expect("post-hoc event");
    timeline
        .push_observation(12, vec![signal("self_check", 0.0)])
        .expect("post-hoc event");

    assert_eq!(timeline.observations().len(), 2);
    assert!(
        timeline
            .primary_precursor_events()
            .expect("boundary known")
            .is_empty()
    );
    assert!(
        timeline
            .timing_records()
            .expect("boundary known")
            .iter()
            .all(|record| record.eligibility() == PrecursorEligibility::PostHocOnly)
    );
}
