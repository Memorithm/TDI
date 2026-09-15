#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi21_anf_search::{
    AnfSearchError, DevelopmentSet, LabeledAssignment, SearchEnvelope, ValidationSet,
    evaluate_validation, fit_sparse_anf,
};

fn label_nontrivial(assignment: u64) -> bool {
    let x0 = assignment & 0b001 != 0;
    let x1 = assignment & 0b010 != 0;
    let x2 = assignment & 0b100 != 0;
    x0 ^ (x1 & x2)
}

fn full_three_variable_set() -> Vec<LabeledAssignment> {
    (0..8)
        .map(|assignment| LabeledAssignment {
            assignment,
            expected: label_nontrivial(assignment),
        })
        .collect()
}

fn reference_envelope() -> SearchEnvelope {
    SearchEnvelope {
        variable_count: 3,
        max_degree: 2,
        max_terms: 2,
        max_candidates: 64,
    }
}

#[test]
fn exhaustive_search_recovers_nontrivial_sparse_zhegalkin_relation() {
    let development = DevelopmentSet::new(3, &full_three_variable_set()).unwrap();
    let result = fit_sparse_anf(&development, reference_envelope()).unwrap();

    assert_eq!(result.variable_count(), 3);
    assert!(!result.constant());
    assert_eq!(result.monomials(), &[0b001, 0b110]);
    assert_eq!(result.development_mismatches(), 0);
    assert_eq!(result.max_degree(), 2);

    // Six monomials of degree <= 2 over three variables. With at most two
    // terms and both constants, exactly 2*(C(6,0)+C(6,1)+C(6,2))=44 candidates.
    assert_eq!(result.work().candidates_evaluated, 44);
    assert_eq!(result.work().case_evaluations, 44 * 8);
    // Per constant: C(6,1)*1 + C(6,2)*2 = 36 term evaluations per row.
    assert_eq!(result.work().monomial_evaluations, 2 * 36 * 8);
}

#[test]
fn selected_program_round_trips_through_exact_anf_synthesizer() {
    let development = DevelopmentSet::new(3, &full_three_variable_set()).unwrap();
    let result = fit_sparse_anf(&development, reference_envelope()).unwrap();
    let program = result.to_anf_program().unwrap();
    assert_eq!(program.variable_count(), 3);
    assert!(!program.constant());
    let masks: Vec<_> = program.terms().iter().map(|term| term.variables).collect();
    assert_eq!(masks, vec![0b001, 0b110]);
    for assignment in 0..8 {
        assert_eq!(
            program.evaluate(assignment),
            Ok(label_nontrivial(assignment))
        );
    }
}

#[test]
fn validation_is_post_selection_and_cannot_change_the_fitted_program() {
    let development = DevelopmentSet::new(3, &full_three_variable_set()).unwrap();
    let result = fit_sparse_anf(&development, reference_envelope()).unwrap();
    let before = result.clone();

    let validation_good = ValidationSet::new(
        3,
        &[
            LabeledAssignment {
                assignment: 3,
                expected: label_nontrivial(3),
            },
            LabeledAssignment {
                assignment: 7,
                expected: label_nontrivial(7),
            },
        ],
    )
    .unwrap();
    let validation_bad = ValidationSet::new(
        3,
        &[
            LabeledAssignment {
                assignment: 3,
                expected: !label_nontrivial(3),
            },
            LabeledAssignment {
                assignment: 7,
                expected: !label_nontrivial(7),
            },
        ],
    )
    .unwrap();

    let good = evaluate_validation(&result, &validation_good).unwrap();
    let bad = evaluate_validation(&result, &validation_bad).unwrap();
    assert_eq!(good.mismatches, 0);
    assert_eq!(bad.mismatches, 2);
    assert_eq!(result, before);
}

#[test]
fn tie_breaking_prefers_the_simplest_canonical_program() {
    let development = DevelopmentSet::new(
        2,
        &[LabeledAssignment {
            assignment: 0,
            expected: false,
        }],
    )
    .unwrap();
    let result = fit_sparse_anf(
        &development,
        SearchEnvelope {
            variable_count: 2,
            max_degree: 2,
            max_terms: 2,
            max_candidates: 32,
        },
    )
    .unwrap();
    assert_eq!(result.variable_count(), 2);
    assert!(!result.constant());
    assert!(result.monomials().is_empty());
    assert_eq!(result.development_mismatches(), 0);
}

#[test]
fn candidate_space_is_rejected_before_unbounded_enumeration() {
    let development = DevelopmentSet::new(
        6,
        &[LabeledAssignment {
            assignment: 0,
            expected: false,
        }],
    )
    .unwrap();
    let envelope = SearchEnvelope {
        variable_count: 6,
        max_degree: 3,
        max_terms: 4,
        max_candidates: 65_536,
    };
    assert!(matches!(
        fit_sparse_anf(&development, envelope),
        Err(AnfSearchError::CandidateSpaceExceedsBudget { .. })
    ));
}

#[test]
fn datasets_reject_duplicate_and_out_of_range_assignments() {
    assert_eq!(
        DevelopmentSet::new(
            2,
            &[
                LabeledAssignment {
                    assignment: 1,
                    expected: false,
                },
                LabeledAssignment {
                    assignment: 1,
                    expected: true,
                },
            ],
        ),
        Err(AnfSearchError::DuplicateAssignment { assignment: 1 })
    );
    assert_eq!(
        ValidationSet::new(
            2,
            &[LabeledAssignment {
                assignment: 4,
                expected: false,
            }],
        ),
        Err(AnfSearchError::AssignmentOutOfRange {
            index: 0,
            assignment: 4,
        })
    );
}

#[test]
fn variable_count_drift_fails_closed_for_fit_and_validation() {
    let development = DevelopmentSet::new(
        2,
        &[LabeledAssignment {
            assignment: 0,
            expected: false,
        }],
    )
    .unwrap();
    assert_eq!(
        fit_sparse_anf(&development, reference_envelope()),
        Err(AnfSearchError::VariableCountMismatch {
            expected: 3,
            actual: 2,
        })
    );

    let result = fit_sparse_anf(
        &DevelopmentSet::new(3, &full_three_variable_set()).unwrap(),
        reference_envelope(),
    )
    .unwrap();
    let validation = ValidationSet::new(
        2,
        &[LabeledAssignment {
            assignment: 0,
            expected: false,
        }],
    )
    .unwrap();
    assert_eq!(
        evaluate_validation(&result, &validation),
        Err(AnfSearchError::VariableCountMismatch {
            expected: 3,
            actual: 2,
        })
    );
    assert_eq!(result.variable_count(), 3);
}

#[test]
fn sparse_search_can_recover_three_bit_parity_with_degree_one() {
    let cases: Vec<_> = (0..8)
        .map(|assignment| LabeledAssignment {
            assignment,
            expected: assignment.count_ones() % 2 == 1,
        })
        .collect();
    let result = fit_sparse_anf(
        &DevelopmentSet::new(3, &cases).unwrap(),
        SearchEnvelope {
            variable_count: 3,
            max_degree: 1,
            max_terms: 3,
            max_candidates: 32,
        },
    )
    .unwrap();
    assert_eq!(result.variable_count(), 3);
    assert!(!result.constant());
    assert_eq!(result.monomials(), &[0b001, 0b010, 0b100]);
    assert_eq!(result.development_mismatches(), 0);
}
