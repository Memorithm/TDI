#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi21::{BooleanState, MemoryRead};
use tdi_ai::experimental::tdi21_predicate_identifiability::{
    AdmissionAction, AdmissionAuditCase, FutureRecallProbe, IdentifiabilityError,
    audit_admission_case, conflicting_pair,
};
use tdi_ai::experimental::tdi21_stream::{Event, MemoryMode, StreamConfig};

fn config(mode: MemoryMode) -> StreamConfig {
    StreamConfig {
        mode,
        slots: 4,
        payload_bits: 8,
        max_events: 64,
        route_salt: 0,
    }
}

fn write(key: u64, payload: u64, marker: u64) -> Event {
    Event::Write {
        key,
        payload: BooleanState::from_bits(payload),
        marker: BooleanState::from_bits(marker),
    }
}

fn hit(payload: u64) -> MemoryRead {
    MemoryRead::Hit(BooleanState::from_bits(payload))
}

fn common_prefix() -> Vec<Event> {
    vec![write(1, 11, 1), write(5, 55, 1)]
}

#[test]
fn identical_v1_predicates_require_opposite_hindsight_actions() {
    let preserve_old = AdmissionAuditCase {
        prefix: common_prefix(),
        decision: write(9, 99, 1),
        probe: FutureRecallProbe {
            key: 1,
            expected: hit(11),
        },
    };
    let retrieve_new = AdmissionAuditCase {
        prefix: common_prefix(),
        decision: write(9, 99, 1),
        probe: FutureRecallProbe {
            key: 9,
            expected: hit(99),
        },
    };

    let old_audit = audit_admission_case(config(MemoryMode::TwoWay), &preserve_old).unwrap();
    let new_audit = audit_admission_case(config(MemoryMode::TwoWay), &retrieve_new).unwrap();

    assert_eq!(old_audit.predicates, new_audit.predicates);
    // marker0 + has-any + full; exact=false and victim=first way.
    assert_eq!(old_audit.predicates.assignment(), 0b011001);
    assert_eq!(
        old_audit.unique_hindsight_label,
        Some(AdmissionAction::Inhibit)
    );
    assert_eq!(
        new_audit.unique_hindsight_label,
        Some(AdmissionAction::Admit)
    );

    assert!(!old_audit.admit.succeeds);
    assert!(old_audit.inhibit.succeeds);
    assert!(new_audit.admit.succeeds);
    assert!(!new_audit.inhibit.succeeds);

    let conflict =
        conflicting_pair(old_audit, new_audit).expect("must expose a collision in v1 features");
    assert_eq!(conflict.first_label, AdmissionAction::Inhibit);
    assert_eq!(conflict.second_label, AdmissionAction::Admit);
}

#[test]
fn conflict_is_symmetric_but_requires_opposite_unique_labels() {
    let first = audit_admission_case(
        config(MemoryMode::TwoWay),
        &AdmissionAuditCase {
            prefix: common_prefix(),
            decision: write(9, 99, 1),
            probe: FutureRecallProbe {
                key: 1,
                expected: hit(11),
            },
        },
    )
    .unwrap();
    let second = audit_admission_case(
        config(MemoryMode::TwoWay),
        &AdmissionAuditCase {
            prefix: common_prefix(),
            decision: write(9, 99, 1),
            probe: FutureRecallProbe {
                key: 9,
                expected: hit(99),
            },
        },
    )
    .unwrap();
    assert!(conflicting_pair(first, second).is_some());
    assert!(conflicting_pair(second, first).is_some());
}

#[test]
fn same_future_goal_does_not_create_a_false_conflict() {
    let case = AdmissionAuditCase {
        prefix: common_prefix(),
        decision: write(9, 99, 1),
        probe: FutureRecallProbe {
            key: 1,
            expected: hit(11),
        },
    };
    let first = audit_admission_case(config(MemoryMode::TwoWay), &case).unwrap();
    let second = audit_admission_case(config(MemoryMode::TwoWay), &case).unwrap();
    assert!(conflicting_pair(first, second).is_none());
}

#[test]
fn evaluator_can_identify_an_exact_update_that_should_be_admitted() {
    let case = AdmissionAuditCase {
        prefix: common_prefix(),
        decision: write(5, 77, 1),
        probe: FutureRecallProbe {
            key: 5,
            expected: hit(77),
        },
    };
    let audit = audit_admission_case(config(MemoryMode::TwoWay), &case).unwrap();
    assert!(audit.predicates.assignment() & (1 << 2) != 0); // exact-present
    assert_eq!(audit.unique_hindsight_label, Some(AdmissionAction::Admit));
    assert!(audit.admit.succeeds);
    assert!(!audit.inhibit.succeeds);
}

#[test]
fn audit_rejects_candidate_invalid_or_ambiguous_inputs() {
    let valid_probe = FutureRecallProbe {
        key: 1,
        expected: hit(11),
    };
    assert_eq!(
        audit_admission_case(
            config(MemoryMode::Direct),
            &AdmissionAuditCase {
                prefix: common_prefix(),
                decision: write(9, 99, 1),
                probe: valid_probe,
            },
        ),
        Err(IdentifiabilityError::RequiresTwoWayB3)
    );
    assert_eq!(
        audit_admission_case(
            config(MemoryMode::TwoWay),
            &AdmissionAuditCase {
                prefix: common_prefix(),
                decision: write(9, 99, 3),
                probe: valid_probe,
            },
        ),
        Err(IdentifiabilityError::DecisionMustBeAdmittedWrite)
    );
    assert_eq!(
        audit_admission_case(
            config(MemoryMode::TwoWay),
            &AdmissionAuditCase {
                prefix: common_prefix(),
                decision: Event::Recall { key: 9 },
                probe: valid_probe,
            },
        ),
        Err(IdentifiabilityError::DecisionMustBeAdmittedWrite)
    );
    assert_eq!(
        audit_admission_case(
            config(MemoryMode::TwoWay),
            &AdmissionAuditCase {
                prefix: common_prefix(),
                decision: write(9, 99, 1),
                probe: FutureRecallProbe {
                    key: 123,
                    expected: MemoryRead::Miss,
                },
            },
        ),
        Err(IdentifiabilityError::InvalidProbeExpected)
    );
}

#[test]
fn audit_prefix_is_explicitly_bounded() {
    let prefix = vec![Event::Ignore; 33];
    assert_eq!(
        audit_admission_case(
            config(MemoryMode::TwoWay),
            &AdmissionAuditCase {
                prefix,
                decision: write(9, 99, 1),
                probe: FutureRecallProbe {
                    key: 9,
                    expected: hit(99),
                },
            },
        ),
        Err(IdentifiabilityError::PrefixTooLong)
    );
}
