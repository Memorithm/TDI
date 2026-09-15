#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi21_anf_search::SearchEnvelope;
use tdi_ai::experimental::tdi21_distributional_objective::{
    AdmissionOutcomeSample, DevelopmentAdmissionSet, ValidationAdmissionSet,
};
use tdi_ai::experimental::tdi21_distributional_search::{
    DistributionalSearchError, evaluate_distributional_validation, fit_distributional_anf,
};

fn sample(
    assignment: u64,
    admit_succeeds: bool,
    inhibit_succeeds: bool,
) -> AdmissionOutcomeSample {
    AdmissionOutcomeSample {
        assignment,
        admit_succeeds,
        inhibit_succeeds,
    }
}

fn envelope(variable_count: u8, max_terms: usize, max_candidates: u64) -> SearchEnvelope {
    SearchEnvelope {
        variable_count,
        max_degree: variable_count.min(2),
        max_terms,
        max_candidates,
    }
}

#[test]
fn exact_v1_conflict_preserves_one_of_two_failure_floor_during_search() {
    let assignment = 0b011001;
    let development = DevelopmentAdmissionSet::new(
        6,
        &[
            sample(assignment, false, true),
            sample(assignment, true, false),
        ],
    )
    .unwrap();
    let result = fit_distributional_anf(&development, envelope(6, 0, 2)).unwrap();

    // Both constants fail one episode. The declared deterministic tie-break
    // chooses constant=false (INHIBIT) rather than inventing perfect labels.
    assert!(!result.program().constant());
    assert!(result.program().terms().is_empty());
    assert_eq!(result.development_selected_failures(), 1);
    assert_eq!(result.development_state_oracle_failures(), 1);
    assert_eq!(result.development_conflict_lower_bound(), 1);
    assert_eq!(result.development_policy_excess_failures(), 0);
    assert_eq!(result.work().candidates_evaluated, 2);
    assert_eq!(result.work().state_evaluations, 2);
    assert_eq!(result.work().monomial_evaluations, 0);
}

#[test]
fn repeated_samples_retain_unit_weight_in_development_selection() {
    let mut samples = vec![sample(0, true, false); 3];
    samples.push(sample(0, false, true));
    let development = DevelopmentAdmissionSet::new(1, &samples).unwrap();
    let result = fit_distributional_anf(&development, envelope(1, 0, 2)).unwrap();

    // Same observable state, 3:1 preference for ADMIT. A state-unweighted
    // implementation would lose this multiplicity information.
    assert!(result.program().constant());
    assert!(result.program().terms().is_empty());
    assert_eq!(result.development_selected_failures(), 1);
    assert_eq!(result.development_state_oracle_failures(), 1);
    assert_eq!(result.development_conflict_lower_bound(), 1);
    assert_eq!(result.development_policy_excess_failures(), 0);
}

#[test]
fn identifiable_two_state_distribution_recovers_x0_policy() {
    let development = DevelopmentAdmissionSet::new(
        1,
        &[
            sample(0, false, true),
            sample(1, true, false),
        ],
    )
    .unwrap();
    let result = fit_distributional_anf(&development, envelope(1, 1, 4)).unwrap();

    assert!(!result.program().constant());
    let masks: Vec<_> = result
        .program()
        .terms()
        .iter()
        .map(|term| term.variables)
        .collect();
    assert_eq!(masks, vec![1]);
    assert_eq!(result.development_selected_failures(), 0);
    assert_eq!(result.development_state_oracle_failures(), 0);
    assert_eq!(result.development_conflict_lower_bound(), 0);
    assert_eq!(result.development_policy_excess_failures(), 0);
}

#[test]
fn restricted_policy_family_exposes_policy_form_error_above_feature_floor() {
    // Desired action is XOR(x0,x1). Constants only cannot express it.
    let development = DevelopmentAdmissionSet::new(
        2,
        &[
            sample(0b00, false, true),
            sample(0b01, true, false),
            sample(0b10, true, false),
            sample(0b11, false, true),
        ],
    )
    .unwrap();
    let restricted = fit_distributional_anf(&development, envelope(2, 0, 2)).unwrap();
    assert_eq!(restricted.development_state_oracle_failures(), 0);
    assert_eq!(restricted.development_conflict_lower_bound(), 0);
    assert_eq!(restricted.development_selected_failures(), 2);
    assert_eq!(restricted.development_policy_excess_failures(), 2);

    let expressive = fit_distributional_anf(
        &development,
        SearchEnvelope {
            variable_count: 2,
            max_degree: 1,
            max_terms: 2,
            max_candidates: 8,
        },
    )
    .unwrap();
    assert_eq!(expressive.development_selected_failures(), 0);
    assert_eq!(expressive.development_policy_excess_failures(), 0);
    let masks: Vec<_> = expressive
        .program()
        .terms()
        .iter()
        .map(|term| term.variables)
        .collect();
    assert_eq!(masks, vec![0b01, 0b10]);
}

#[test]
fn validation_is_post_selection_and_cannot_refit_the_policy() {
    let development = DevelopmentAdmissionSet::new(
        1,
        &[
            sample(0, false, true),
            sample(1, true, false),
        ],
    )
    .unwrap();
    let result = fit_distributional_anf(&development, envelope(1, 1, 4)).unwrap();
    let before = result.clone();

    let agreeing = ValidationAdmissionSet::new(
        1,
        &[
            sample(0, false, true),
            sample(1, true, false),
        ],
    )
    .unwrap();
    let reversed = ValidationAdmissionSet::new(
        1,
        &[
            sample(0, true, false),
            sample(1, false, true),
        ],
    )
    .unwrap();

    let agreeing_score = evaluate_distributional_validation(&result, &agreeing).unwrap();
    let reversed_score = evaluate_distributional_validation(&result, &reversed).unwrap();
    assert_eq!(agreeing_score.selected_failures(), 0);
    assert_eq!(reversed_score.selected_failures(), 2);
    assert_eq!(result, before);
}

#[test]
fn validation_arity_mismatch_fails_closed() {
    let development = DevelopmentAdmissionSet::new(2, &[sample(0, false, true)]).unwrap();
    let result = fit_distributional_anf(&development, envelope(2, 0, 2)).unwrap();
    let validation = ValidationAdmissionSet::new(1, &[sample(0, false, true)]).unwrap();
    assert_eq!(
        evaluate_distributional_validation(&result, &validation),
        Err(DistributionalSearchError::VariableCountMismatch {
            envelope: 2,
            dataset: 1,
        })
    );
}

#[test]
fn candidate_budget_is_checked_before_enumeration() {
    let development = DevelopmentAdmissionSet::new(3, &[sample(0, false, true)]).unwrap();
    let too_small = SearchEnvelope {
        variable_count: 3,
        max_degree: 2,
        max_terms: 2,
        max_candidates: 43,
    };
    assert_eq!(
        fit_distributional_anf(&development, too_small),
        Err(DistributionalSearchError::CandidateSpaceExceedsBudget {
            required: 44,
            budget: 43,
        })
    );
}

#[test]
fn search_work_counts_unique_states_not_hidden_sample_multiplicity() {
    let mut samples = vec![sample(0, true, false); 10];
    samples.extend(vec![sample(1, false, true); 10]);
    let development = DevelopmentAdmissionSet::new(1, &samples).unwrap();
    let result = fit_distributional_anf(&development, envelope(1, 1, 4)).unwrap();

    // Candidate family over one degree-1 monomial: {}, {x0}, each with two
    // constants = 4 candidates. Search evaluates two aggregated states each.
    assert_eq!(result.work().candidates_evaluated, 4);
    assert_eq!(result.work().state_evaluations, 8);
    // Two candidates carry one monomial, each evaluated on two states.
    assert_eq!(result.work().monomial_evaluations, 4);
}
