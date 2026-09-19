#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi21_relational_address_search::{
    AddressSample, DevelopmentAddressSet, ValidationAddressSet,
    development_from_relational_tasks, evaluate_address_validation, fit_relational_address,
    validation_from_relational_tasks,
};
use tdi_ai::experimental::tdi21_relational_binding::{RelationalConfig, RelationalRead};
use tdi_ai::experimental::tdi21_relational_learned::LearnedRelationalBinder;
use tdi_ai::experimental::tdi21_relational_tasks::{
    DevelopmentRelationalSet, RELATIONAL_V1_ENTITY_BITS, RELATIONAL_V1_RELATION_BITS,
    ValidationRelationalSet,
};
use tdi_ai::experimental::tdi21_stream::{MemoryMode, StreamConfig};

fn config() -> RelationalConfig {
    RelationalConfig {
        stream: StreamConfig {
            mode: MemoryMode::TwoWay,
            slots: 128,
            payload_bits: RELATIONAL_V1_ENTITY_BITS,
            max_events: 128,
            route_salt: 0x5444_4932_3100_0021,
        },
        entity_bits: RELATIONAL_V1_ENTITY_BITS,
        relation_bits: RELATIONAL_V1_RELATION_BITS,
    }
}

fn sparse_selected() -> tdi_ai::experimental::tdi21_relational_address_search::AddressSearchResult {
    let development =
        development_from_relational_tasks(&DevelopmentRelationalSet::v1()).unwrap();
    fit_relational_address(&development).unwrap()
}

fn basis_selected() -> tdi_ai::experimental::tdi21_relational_address_search::AddressSearchResult {
    let mut cases = vec![AddressSample {
        input: 0,
        expected: 0,
    }];
    for bit in 0..8 {
        let value = 1u8 << bit;
        cases.push(AddressSample {
            input: value,
            expected: value,
        });
    }
    let development = DevelopmentAddressSet::new(&cases).unwrap();
    fit_relational_address(&development).unwrap()
}

#[test]
fn exact_address_mismatch_can_coexist_with_functional_renamed_id_transfer() {
    let selected = sparse_selected();
    let validation_addresses =
        validation_from_relational_tasks(&ValidationRelationalSet::v1()).unwrap();
    let address_evidence =
        evaluate_address_validation(&selected, &validation_addresses).unwrap();

    assert_eq!(address_evidence.address_mismatches, 5);
    assert_eq!(address_evidence.bit_mismatches, 10);
    assert_eq!(address_evidence.rule_evaluations, 5 * 8);

    for episode in ValidationRelationalSet::v1().episodes() {
        let mut binder = LearnedRelationalBinder::new(config(), selected.clone()).unwrap();
        let observed = binder.run_episode(episode).unwrap();
        assert_eq!(observed, episode.expected());
        assert_eq!(binder.counters().work.pairwise_comparisons, 0);
        let expected_predictions = (episode.facts.len() + episode.query.relations.len()) as u64;
        assert_eq!(binder.work().predictions, expected_predictions);
        assert_eq!(binder.work().rule_evaluations, expected_predictions * 8);
    }
}

#[test]
fn sparse_encoder_aliases_corresponding_development_and_validation_ids() {
    let selected = sparse_selected();
    let mut binder = LearnedRelationalBinder::new(config(), selected).unwrap();

    binder.bind(1, 1, 2).unwrap();
    binder.bind(9, 9, 10).unwrap();

    assert_eq!(binder.recall(1, 1).unwrap(), RelationalRead::Hit(10));
    assert_eq!(binder.recall(9, 9).unwrap(), RelationalRead::Hit(10));
    assert_eq!(binder.counters().work.memory_replacements, 0);
}

#[test]
fn basis_covered_identity_encoder_keeps_namespaces_distinct() {
    let selected = basis_selected();

    let full_domain: Vec<_> = (0u16..=u8::MAX as u16)
        .map(|value| AddressSample {
            input: value as u8,
            expected: value as u8,
        })
        .collect();
    let validation = ValidationAddressSet::new(&full_domain).unwrap();
    let evidence = evaluate_address_validation(&selected, &validation).unwrap();
    assert_eq!(evidence.address_mismatches, 0);
    assert_eq!(evidence.bit_mismatches, 0);
    assert_eq!(evidence.rule_evaluations, 256 * 8);

    let mut binder = LearnedRelationalBinder::new(config(), selected).unwrap();
    binder.bind(1, 1, 2).unwrap();
    binder.bind(9, 9, 10).unwrap();
    assert_eq!(binder.recall(1, 1).unwrap(), RelationalRead::Hit(2));
    assert_eq!(binder.recall(9, 9).unwrap(), RelationalRead::Hit(10));
}

#[test]
fn unseen_multi_relation_same_subject_structure_exposes_sparse_alias() {
    let sparse = sparse_selected();
    let mut sparse_binder = LearnedRelationalBinder::new(config(), sparse).unwrap();
    sparse_binder.bind(9, 9, 10).unwrap();
    sparse_binder.bind(10, 9, 11).unwrap();

    // Development never disambiguated relation low bits from subject low bits.
    // Both writes therefore map to the same learned key and the second replaces
    // the first logical fact without a B3 replacement event.
    assert_eq!(sparse_binder.recall(9, 9).unwrap(), RelationalRead::Hit(11));
    assert_eq!(sparse_binder.recall(10, 9).unwrap(), RelationalRead::Hit(11));
    assert_eq!(sparse_binder.counters().work.memory_replacements, 0);

    let identity = basis_selected();
    let mut identity_binder = LearnedRelationalBinder::new(config(), identity).unwrap();
    identity_binder.bind(9, 9, 10).unwrap();
    identity_binder.bind(10, 9, 11).unwrap();
    assert_eq!(identity_binder.recall(9, 9).unwrap(), RelationalRead::Hit(10));
    assert_eq!(identity_binder.recall(10, 9).unwrap(), RelationalRead::Hit(11));
}

#[test]
fn learned_encoder_is_selected_only_from_development_task_addresses() {
    let development =
        development_from_relational_tasks(&DevelopmentRelationalSet::v1()).unwrap();
    let selected = fit_relational_address(&development).unwrap();
    let before = selected.clone();

    for episode in ValidationRelationalSet::v1().episodes() {
        let mut binder = LearnedRelationalBinder::new(config(), selected.clone()).unwrap();
        let _ = binder.run_episode(episode).unwrap();
    }

    assert_eq!(selected, before);
}
