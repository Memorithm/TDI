#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi21::{BooleanState, MemoryRead};
use tdi_ai::experimental::tdi21_anf_search::SearchEnvelope;
use tdi_ai::experimental::tdi21_distributional_objective::DevelopmentAdmissionSet;
use tdi_ai::experimental::tdi21_distributional_search::fit_distributional_anf;
use tdi_ai::experimental::tdi21_predicate_identifiability::{
    AdmissionAuditCase, FutureRecallProbe, IdentifiabilityError,
};
use tdi_ai::experimental::tdi21_sequence_materializer::{
    SequenceMaterializationError, materialize_admission_cases,
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

fn case(decision: Event, probe_key: u64, expected: u64) -> AdmissionAuditCase {
    AdmissionAuditCase {
        prefix: common_prefix(),
        decision,
        probe: FutureRecallProbe {
            key: probe_key,
            expected: hit(expected),
        },
    }
}

#[test]
fn exact_v1_conflict_is_materialized_without_manual_labels() {
    let materialized = materialize_admission_cases(
        config(MemoryMode::TwoWay),
        &[
            case(write(9, 99, 1), 1, 11),
            case(write(9, 99, 1), 9, 99),
        ],
    )
    .unwrap();

    assert_eq!(materialized.len(), 2);
    assert_eq!(materialized.samples()[0].assignment, 0b011001);
    assert_eq!(materialized.samples()[1].assignment, 0b011001);
    assert!(!materialized.samples()[0].admit_succeeds);
    assert!(materialized.samples()[0].inhibit_succeeds);
    assert!(materialized.samples()[1].admit_succeeds);
    assert!(!materialized.samples()[1].inhibit_succeeds);

    let summary = materialized.summary();
    assert_eq!(summary.cases, 2);
    assert_eq!(summary.both_succeed, 0);
    assert_eq!(summary.admit_only, 1);
    assert_eq!(summary.inhibit_only, 1);
    assert_eq!(summary.neither_succeeds, 0);
}

#[test]
fn all_four_counterfactual_classes_are_derived_by_execution() {
    let materialized = materialize_admission_cases(
        config(MemoryMode::TwoWay),
        &[
            // Both: updating key 5 or inhibiting it both preserve key 1 = 11.
            case(write(5, 77, 1), 1, 11),
            // Admit only: only admitting new key 9 makes key 9 = 99 recallable.
            case(write(9, 99, 1), 9, 99),
            // Inhibit only: admitting key 9 evicts key 1; inhibiting preserves it.
            case(write(9, 99, 1), 1, 11),
            // Neither: neither action changes key 1 into the deliberately wrong 99.
            case(write(5, 77, 1), 1, 99),
        ],
    )
    .unwrap();

    let summary = materialized.summary();
    assert_eq!(summary.cases, 4);
    assert_eq!(summary.both_succeed, 1);
    assert_eq!(summary.admit_only, 1);
    assert_eq!(summary.inhibit_only, 1);
    assert_eq!(summary.neither_succeeds, 1);
}

#[test]
fn materialized_conflict_flows_into_distributional_search_unchanged() {
    let materialized = materialize_admission_cases(
        config(MemoryMode::TwoWay),
        &[
            case(write(9, 99, 1), 1, 11),
            case(write(9, 99, 1), 9, 99),
        ],
    )
    .unwrap();
    let development = DevelopmentAdmissionSet::new(6, materialized.samples()).unwrap();
    let result = fit_distributional_anf(
        &development,
        SearchEnvelope {
            variable_count: 6,
            max_degree: 1,
            max_terms: 0,
            max_candidates: 2,
        },
    )
    .unwrap();

    assert_eq!(result.development_samples(), 2);
    assert_eq!(result.development_unique_states(), 1);
    assert_eq!(result.development_selected_failures(), 1);
    assert_eq!(result.development_conflict_lower_bound(), 1);
    assert_eq!(result.development_policy_excess_failures(), 0);
}

#[test]
fn batch_errors_report_the_exact_case_index() {
    let good = case(write(9, 99, 1), 9, 99);
    let mut invalid = good.clone();
    invalid.decision = write(9, 99, 3);

    assert_eq!(
        materialize_admission_cases(config(MemoryMode::TwoWay), &[good, invalid]),
        Err(SequenceMaterializationError::Case {
            index: 1,
            source: IdentifiabilityError::DecisionMustBeAdmittedWrite,
        })
    );
}

#[test]
fn empty_and_oversized_batches_fail_before_materialization() {
    assert_eq!(
        materialize_admission_cases(config(MemoryMode::TwoWay), &[]),
        Err(SequenceMaterializationError::EmptyCases)
    );

    let repeated = case(write(9, 99, 1), 9, 99);
    let oversized = vec![repeated; 4097];
    assert_eq!(
        materialize_admission_cases(config(MemoryMode::TwoWay), &oversized),
        Err(SequenceMaterializationError::TooManyCases)
    );
}
