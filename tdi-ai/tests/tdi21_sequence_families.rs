#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi21_anf_search::SearchEnvelope;
use tdi_ai::experimental::tdi21_distributional_search::{
    evaluate_distributional_validation, fit_distributional_anf,
};
use tdi_ai::experimental::tdi21_sequence_families::{
    DevelopmentSequenceFamily, FAMILY_CASE_COUNT, FamilyScenario, SequenceSplit,
    ValidationSequenceFamily, materialize_development_family, materialize_validation_family,
    validate_split_disjointness,
};
use tdi_ai::experimental::tdi21_stream::{MemoryMode, StreamConfig};

fn config() -> StreamConfig {
    StreamConfig {
        mode: MemoryMode::TwoWay,
        slots: 4,
        payload_bits: 8,
        max_events: 64,
        route_salt: 0,
    }
}

#[test]
fn v1_families_have_fixed_typed_case_identity_and_exact_split_disjointness() {
    let development = DevelopmentSequenceFamily::v1();
    let validation = ValidationSequenceFamily::v1();

    assert_eq!(development.cases().len(), FAMILY_CASE_COUNT);
    assert_eq!(validation.cases().len(), FAMILY_CASE_COUNT);
    validate_split_disjointness(&development, &validation).unwrap();

    for case in development.cases() {
        assert_eq!(case.id().split, SequenceSplit::Development);
    }
    for case in validation.cases() {
        assert_eq!(case.id().split, SequenceSplit::Validation);
    }

    let development_scenarios: Vec<_> = development
        .cases()
        .iter()
        .map(|case| case.id().scenario)
        .collect();
    assert_eq!(
        development_scenarios,
        vec![
            FamilyScenario::PreserveFirstOldFact,
            FamilyScenario::RetainNewFact,
            FamilyScenario::PreserveSecondOldFact,
            FamilyScenario::ExactUpdate,
            FamilyScenario::PreserveUnrelatedDuringUpdate,
        ]
    );

    for development_case in development.cases() {
        for validation_case in validation.cases() {
            assert_ne!(development_case.audit(), validation_case.audit());
        }
    }
}

#[test]
fn outcomes_are_derived_from_execution_and_preserve_declared_strata() {
    let development = DevelopmentSequenceFamily::v1();
    let materialized = materialize_development_family(config(), &development).unwrap();

    assert_eq!(materialized.ids().len(), FAMILY_CASE_COUNT);
    assert_eq!(materialized.dataset().sample_count(), FAMILY_CASE_COUNT as u64);
    assert_eq!(materialized.dataset().states().len(), 2);

    let summary = materialized.summary();
    assert_eq!(summary.cases, 5);
    assert_eq!(summary.both_succeed, 2);
    assert_eq!(summary.admit_only, 2);
    assert_eq!(summary.inhibit_only, 1);
    assert_eq!(summary.neither_succeeds, 0);
}

#[test]
fn validation_is_episode_disjoint_but_preserves_the_observable_state_support() {
    let development =
        materialize_development_family(config(), &DevelopmentSequenceFamily::v1()).unwrap();
    let validation =
        materialize_validation_family(config(), &ValidationSequenceFamily::v1()).unwrap();

    let development_assignments: Vec<_> = development
        .dataset()
        .states()
        .iter()
        .map(|state| state.assignment())
        .collect();
    let validation_assignments: Vec<_> = validation
        .dataset()
        .states()
        .iter()
        .map(|state| state.assignment())
        .collect();

    assert_eq!(development_assignments, validation_assignments);
    assert_eq!(development.summary(), validation.summary());
}

#[test]
fn development_search_transfers_to_validation_without_refitting() {
    let development =
        materialize_development_family(config(), &DevelopmentSequenceFamily::v1()).unwrap();
    let validation =
        materialize_validation_family(config(), &ValidationSequenceFamily::v1()).unwrap();

    let selected = fit_distributional_anf(
        development.dataset(),
        SearchEnvelope {
            variable_count: 6,
            max_degree: 1,
            max_terms: 1,
            max_candidates: 14,
        },
    )
    .unwrap();

    assert_eq!(selected.development_samples(), 5);
    assert_eq!(selected.development_unique_states(), 2);
    assert_eq!(selected.development_hindsight_oracle_failures(), 0);
    assert_eq!(selected.development_conflict_lower_bound(), 1);
    assert_eq!(selected.development_state_oracle_failures(), 1);
    assert_eq!(selected.development_selected_failures(), 1);
    assert_eq!(selected.development_policy_excess_failures(), 0);

    let validation_score =
        evaluate_distributional_validation(&selected, validation.dataset()).unwrap();
    assert_eq!(validation_score.samples(), 5);
    assert_eq!(validation_score.unique_states(), 2);
    assert_eq!(validation_score.deterministic_conflict_lower_bound(), 1);
    assert_eq!(validation_score.deterministic_state_oracle_failures(), 1);
    assert_eq!(validation_score.selected_failures(), 1);
    assert_eq!(validation_score.policy_excess_failures(), 0);
}
