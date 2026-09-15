#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi21_anf_synthesis::synthesize_anf;
use tdi_ai::experimental::tdi21_distributional_objective::{
    AdmissionOutcomeSample, DevelopmentAdmissionSet, DistributionalObjectiveError,
    MAX_DISTRIBUTIONAL_SAMPLES, ValidationAdmissionSet, score_development_policy,
    score_validation_policy,
};

fn constant_program(
    variable_count: u8,
    value: bool,
) -> tdi_ai::experimental::tdi21_anf_synthesis::AnfProgram {
    synthesize_anf(variable_count, &vec![value; 1usize << variable_count]).unwrap()
}

#[test]
fn v1_conflict_pair_has_an_exact_one_of_two_deterministic_failure_floor() {
    let assignment = 0b011001;
    let development = DevelopmentAdmissionSet::new(
        6,
        &[
            AdmissionOutcomeSample {
                assignment,
                admit_succeeds: false,
                inhibit_succeeds: true,
            },
            AdmissionOutcomeSample {
                assignment,
                admit_succeeds: true,
                inhibit_succeeds: false,
            },
        ],
    )
    .unwrap();

    assert_eq!(development.sample_count(), 2);
    assert_eq!(development.states().len(), 1);
    let row = development.states()[0];
    assert_eq!(row.assignment(), assignment);
    assert_eq!(row.outcomes().admit_only(), 1);
    assert_eq!(row.outcomes().inhibit_only(), 1);
    assert_eq!(row.outcomes().both_succeed(), 0);
    assert_eq!(row.outcomes().neither_succeeds(), 0);
    assert_eq!(row.outcomes().deterministic_conflict_penalty(), 1);

    for program in [constant_program(6, false), constant_program(6, true)] {
        let score = score_development_policy(&program, &development).unwrap();
        assert_eq!(score.samples(), 2);
        assert_eq!(score.unique_states(), 1);
        assert_eq!(score.selected_failures(), 1);
        assert_eq!(score.selected_successes(), 1);
        assert_eq!(score.hindsight_oracle_failures(), 0);
        assert_eq!(score.deterministic_conflict_lower_bound(), 1);
        assert_eq!(score.deterministic_state_oracle_failures(), 1);
        assert_eq!(score.policy_excess_failures(), 0);
        assert_eq!(score.runtime_anf_term_evaluations(), 0);
    }
}

#[test]
fn four_counterfactual_outcome_classes_are_preserved_exactly() {
    let assignment = 3;
    let development = DevelopmentAdmissionSet::new(
        2,
        &[
            AdmissionOutcomeSample {
                assignment,
                admit_succeeds: true,
                inhibit_succeeds: true,
            },
            AdmissionOutcomeSample {
                assignment,
                admit_succeeds: true,
                inhibit_succeeds: false,
            },
            AdmissionOutcomeSample {
                assignment,
                admit_succeeds: false,
                inhibit_succeeds: true,
            },
            AdmissionOutcomeSample {
                assignment,
                admit_succeeds: false,
                inhibit_succeeds: false,
            },
        ],
    )
    .unwrap();
    let counts = development.states()[0].outcomes();
    assert_eq!(counts.both_succeed(), 1);
    assert_eq!(counts.admit_only(), 1);
    assert_eq!(counts.inhibit_only(), 1);
    assert_eq!(counts.neither_succeeds(), 1);
    assert_eq!(counts.observations(), 4);
    assert_eq!(counts.failures_if_admit(), 2);
    assert_eq!(counts.failures_if_inhibit(), 2);
    assert_eq!(counts.hindsight_oracle_failures(), 1);
    assert_eq!(counts.deterministic_conflict_penalty(), 1);
}

#[test]
fn a_simple_anf_can_match_the_best_action_on_two_identifiable_states() {
    let development = DevelopmentAdmissionSet::new(
        1,
        &[
            AdmissionOutcomeSample {
                assignment: 0,
                admit_succeeds: false,
                inhibit_succeeds: true,
            },
            AdmissionOutcomeSample {
                assignment: 1,
                admit_succeeds: true,
                inhibit_succeeds: false,
            },
        ],
    )
    .unwrap();
    let x0 = synthesize_anf(1, &[false, true]).unwrap();
    let score = score_development_policy(&x0, &development).unwrap();
    assert_eq!(score.selected_failures(), 0);
    assert_eq!(score.hindsight_oracle_failures(), 0);
    assert_eq!(score.deterministic_state_oracle_failures(), 0);
    assert_eq!(score.deterministic_conflict_lower_bound(), 0);
    assert_eq!(score.policy_excess_failures(), 0);
    assert_eq!(score.admit_selected_samples(), 1);
    assert_eq!(score.inhibit_selected_samples(), 1);
    assert_eq!(score.runtime_anf_term_evaluations(), 2);
}

#[test]
fn policy_form_error_is_separated_from_feature_conflict() {
    let development = DevelopmentAdmissionSet::new(
        1,
        &[
            AdmissionOutcomeSample {
                assignment: 0,
                admit_succeeds: false,
                inhibit_succeeds: true,
            },
            AdmissionOutcomeSample {
                assignment: 1,
                admit_succeeds: true,
                inhibit_succeeds: false,
            },
        ],
    )
    .unwrap();
    let always_inhibit = constant_program(1, false);
    let score = score_development_policy(&always_inhibit, &development).unwrap();
    assert_eq!(score.selected_failures(), 1);
    assert_eq!(score.deterministic_state_oracle_failures(), 0);
    assert_eq!(score.deterministic_conflict_lower_bound(), 0);
    assert_eq!(score.policy_excess_failures(), 1);
}

#[test]
fn repeated_assignments_are_aggregated_not_rejected_or_overwritten() {
    let samples = vec![
        AdmissionOutcomeSample {
            assignment: 1,
            admit_succeeds: true,
            inhibit_succeeds: false,
        };
        7
    ];
    let set = DevelopmentAdmissionSet::new(2, &samples).unwrap();
    assert_eq!(set.sample_count(), 7);
    assert_eq!(set.states().len(), 1);
    assert_eq!(set.states()[0].outcomes().admit_only(), 7);
}

#[test]
fn validation_is_a_distinct_post_selection_dataset() {
    let development = DevelopmentAdmissionSet::new(
        1,
        &[AdmissionOutcomeSample {
            assignment: 0,
            admit_succeeds: false,
            inhibit_succeeds: true,
        }],
    )
    .unwrap();
    let validation = ValidationAdmissionSet::new(
        1,
        &[AdmissionOutcomeSample {
            assignment: 1,
            admit_succeeds: true,
            inhibit_succeeds: false,
        }],
    )
    .unwrap();
    let x0 = synthesize_anf(1, &[false, true]).unwrap();
    assert_eq!(
        score_development_policy(&x0, &development)
            .unwrap()
            .selected_failures(),
        0
    );
    assert_eq!(
        score_validation_policy(&x0, &validation)
            .unwrap()
            .selected_failures(),
        0
    );
}

#[test]
fn program_and_dataset_arity_must_match() {
    let set = DevelopmentAdmissionSet::new(
        2,
        &[AdmissionOutcomeSample {
            assignment: 0,
            admit_succeeds: true,
            inhibit_succeeds: false,
        }],
    )
    .unwrap();
    let one_bit = constant_program(1, false);
    assert_eq!(
        score_development_policy(&one_bit, &set),
        Err(DistributionalObjectiveError::ProgramArityMismatch {
            program: 1,
            dataset: 2,
        })
    );
}

#[test]
fn input_bounds_fail_closed() {
    assert_eq!(
        DevelopmentAdmissionSet::new(0, &[]),
        Err(DistributionalObjectiveError::ZeroVariables)
    );
    assert_eq!(
        DevelopmentAdmissionSet::new(
            7,
            &[AdmissionOutcomeSample {
                assignment: 0,
                admit_succeeds: true,
                inhibit_succeeds: true,
            }],
        ),
        Err(DistributionalObjectiveError::TooManyVariables)
    );
    assert_eq!(
        DevelopmentAdmissionSet::new(1, &[]),
        Err(DistributionalObjectiveError::EmptySamples)
    );
    assert_eq!(
        DevelopmentAdmissionSet::new(
            1,
            &[AdmissionOutcomeSample {
                assignment: 2,
                admit_succeeds: true,
                inhibit_succeeds: false,
            }],
        ),
        Err(DistributionalObjectiveError::AssignmentOutOfRange {
            index: 0,
            assignment: 2,
        })
    );
    let oversized = vec![
        AdmissionOutcomeSample {
            assignment: 0,
            admit_succeeds: true,
            inhibit_succeeds: true,
        };
        MAX_DISTRIBUTIONAL_SAMPLES + 1
    ];
    assert_eq!(
        ValidationAdmissionSet::new(1, &oversized),
        Err(DistributionalObjectiveError::TooManySamples)
    );
}
