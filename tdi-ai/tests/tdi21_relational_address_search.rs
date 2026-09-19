#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi21_relational_address_search::{
    AddressSample, AddressSearchError, DevelopmentAddressSet, UnaryAddressRule,
    ValidationAddressSet, development_from_relational_tasks, evaluate_address_validation,
    evaluate_cross_namespace_aliases, fit_relational_address, validation_from_relational_tasks,
};
use tdi_ai::experimental::tdi21_relational_tasks::{
    DevelopmentRelationalSet, ValidationRelationalSet,
};

#[test]
fn sparse_task_development_fits_but_fails_on_renamed_validation_bits() {
    let development = development_from_relational_tasks(&DevelopmentRelationalSet::v1()).unwrap();
    let validation = validation_from_relational_tasks(&ValidationRelationalSet::v1()).unwrap();

    assert_eq!(development.len(), 5);
    let selected = fit_relational_address(&development).unwrap();
    assert_eq!(selected.development_samples(), 5);
    assert_eq!(selected.development_bit_mismatches(), 0);
    assert_eq!(selected.work().candidate_rules_evaluated, 8 * 18);
    assert_eq!(selected.work().sample_bit_evaluations, 8 * 18 * 5);

    let rules = selected.rules();
    assert_eq!(
        rules[3],
        UnaryAddressRule::Constant(false),
        "Development never activates entity bit 3"
    );
    assert_eq!(
        rules[7],
        UnaryAddressRule::Constant(false),
        "Development never activates relation bit 3"
    );
    assert_eq!(
        rules[4],
        UnaryAddressRule::Literal {
            variable: 0,
            inverted: false,
        },
        "Development aliases relation bit 0 with subject bit 0"
    );
    assert_eq!(
        rules[5],
        UnaryAddressRule::Literal {
            variable: 1,
            inverted: false,
        },
        "Development aliases relation bit 1 with subject bit 1"
    );

    let evidence = evaluate_address_validation(&selected, &validation).unwrap();
    assert_eq!(evidence.samples, 5);
    assert_eq!(evidence.address_mismatches, 5);
    assert_eq!(evidence.bit_mismatches, 10);
    assert_eq!(evidence.prediction_aliases, 0);
    assert_eq!(evidence.rule_evaluations, 5 * 8);
}

#[test]
fn basis_coverage_recovers_exact_identity_and_transfers_to_full_domain() {
    let mut development_cases = vec![AddressSample {
        input: 0,
        expected: 0,
    }];
    for bit in 0..8 {
        let value = 1u8 << bit;
        development_cases.push(AddressSample {
            input: value,
            expected: value,
        });
    }
    let development = DevelopmentAddressSet::new(&development_cases).unwrap();
    let selected = fit_relational_address(&development).unwrap();

    assert_eq!(selected.development_bit_mismatches(), 0);
    for (bit, rule) in selected.rules().iter().copied().enumerate() {
        assert_eq!(
            rule,
            UnaryAddressRule::Literal {
                variable: bit as u8,
                inverted: false,
            }
        );
    }

    let validation_cases: Vec<_> = (0u16..=u8::MAX as u16)
        .map(|value| AddressSample {
            input: value as u8,
            expected: value as u8,
        })
        .collect();
    let validation = ValidationAddressSet::new(&validation_cases).unwrap();
    let evidence = evaluate_address_validation(&selected, &validation).unwrap();
    assert_eq!(evidence.samples, 256);
    assert_eq!(evidence.address_mismatches, 0);
    assert_eq!(evidence.bit_mismatches, 0);
    assert_eq!(evidence.prediction_aliases, 0);
    assert_eq!(evidence.rule_evaluations, 256 * 8);
}

#[test]
fn combined_namespaces_expose_five_encoder_aliases_under_sparse_fit() {
    let development = development_from_relational_tasks(&DevelopmentRelationalSet::v1()).unwrap();
    let validation = validation_from_relational_tasks(&ValidationRelationalSet::v1()).unwrap();
    let selected = fit_relational_address(&development).unwrap();

    let evidence = evaluate_cross_namespace_aliases(&selected, &development, &validation).unwrap();
    assert_eq!(evidence.development_samples, 5);
    assert_eq!(evidence.validation_samples, 5);
    assert_eq!(evidence.prediction_aliases, 5);
}

#[test]
fn validation_evaluation_cannot_mutate_the_selected_rules() {
    let development = development_from_relational_tasks(&DevelopmentRelationalSet::v1()).unwrap();
    let validation = validation_from_relational_tasks(&ValidationRelationalSet::v1()).unwrap();
    let selected = fit_relational_address(&development).unwrap();
    let before = selected.clone();

    let _ = evaluate_address_validation(&selected, &validation).unwrap();
    assert_eq!(selected, before);
}

#[test]
fn duplicate_or_empty_training_inputs_fail_closed() {
    assert_eq!(
        DevelopmentAddressSet::new(&[]),
        Err(AddressSearchError::EmptySamples)
    );
    let duplicated = [
        AddressSample {
            input: 7,
            expected: 7,
        },
        AddressSample {
            input: 7,
            expected: 7,
        },
    ];
    assert_eq!(
        DevelopmentAddressSet::new(&duplicated),
        Err(AddressSearchError::DuplicateInput { input: 7 })
    );
}

[executed on device: tarek (fa986a59-0105-42b8-b1db-7cddadcd871f)]