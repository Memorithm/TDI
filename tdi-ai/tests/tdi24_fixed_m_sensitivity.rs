#![cfg(feature = "experimental")]

//! TDI-24 slice 35: fixed-M sensitivity (Phase D).
//!
//! Scores the bounded Stage-C case stream under every fixed mirror basis of
//! the frozen rule (all `C(6,3) = 20` choices of the parity-even sector, in
//! lexicographic order, canonical first) with the unchanged C6 reference
//! weights. These tests qualify software semantics and exact identities only:
//! no training, no basis selection, no protected/final access, no attribution
//! or scientific claim.

use tdi_ai::experimental::tdi24_chiral::{Chiral6, chiral_score};
use tdi_ai::experimental::tdi24_eval::{
    C6_REFERENCE_WEIGHTS, EvalArm, EvalError, EvalOutcome, FIXED_M_SENSITIVITY_CONTRACT,
    FIXED_MIRROR_BASIS_COUNT, FixedMSensitivityReport, FixedMirrorBasis,
    MAX_PREFLIGHT_PAIRS_PER_FAMILY, STAGE_C_PREFLIGHT_FAMILIES, StageCPreflightBudget,
    TrainableCapacity, fixed_mirror_bases, run_fixed_m_sensitivity,
    run_fixed_m_sensitivity_for_label, run_stage_c_preflight, validate_fixed_m_sensitivity_report,
    validate_fixed_mirror_basis,
};
use tdi_ai::experimental::tdi24_tasks::DataSplit;

fn rejection(report: &FixedMSensitivityReport) -> &'static str {
    match validate_fixed_m_sensitivity_report(report) {
        Err(EvalError::FixedMSensitivityInvalid { reason }) => reason,
        other => panic!("expected a fixed-M sensitivity rejection, got {other:?}"),
    }
}

#[test]
fn contract_pin_and_family_size_are_declared() {
    assert_eq!(FIXED_M_SENSITIVITY_CONTRACT, "tdi24-fixed-m-sensitivity-v1");
    // C(6,3): derived from the carrier width, not tuned.
    assert_eq!(FIXED_MIRROR_BASIS_COUNT, 20);
}

#[test]
fn frozen_family_is_complete_ordered_and_algebraic() {
    let bases = fixed_mirror_bases();
    assert!(bases[0].is_canonical());
    assert_eq!(bases[0].even_slots, [0, 1, 2]);
    assert_eq!(bases[0].odd_slots, [3, 4, 5]);
    assert_eq!(bases[19].even_slots, [3, 4, 5]);
    let mut seen = std::collections::BTreeSet::new();
    for (index, basis) in bases.iter().enumerate() {
        assert_eq!(basis.index, index);
        validate_fixed_mirror_basis(basis).unwrap();
        assert!(seen.insert(basis.even_slots));
        // The complement exchanges the sectors.
        let complement = bases[basis.complement_index()];
        assert_eq!(complement.even_slots, basis.odd_slots);
        assert_eq!(complement.odd_slots, basis.even_slots);
        if index > 0 {
            assert!(bases[index - 1].even_slots < basis.even_slots);
        }
    }
    assert_eq!(seen.len(), FIXED_MIRROR_BASIS_COUNT);
}

#[test]
fn non_partitions_and_reordered_bases_are_rejected() {
    let canonical = fixed_mirror_bases()[0];
    let invalid = |basis: FixedMirrorBasis, reason: &str| {
        assert_eq!(
            validate_fixed_mirror_basis(&basis),
            Err(EvalError::FixedMSensitivityInvalid {
                reason: leak(reason)
            })
        );
    };
    invalid(
        FixedMirrorBasis {
            even_slots: [0, 1, 1],
            ..canonical
        },
        "basis_not_a_sector_partition",
    );
    invalid(
        FixedMirrorBasis {
            even_slots: [1, 0, 2],
            ..canonical
        },
        "basis_not_a_sector_partition",
    );
    invalid(
        FixedMirrorBasis {
            index: 1,
            ..canonical
        },
        "basis_order_drift",
    );
    invalid(
        FixedMirrorBasis {
            index: FIXED_MIRROR_BASIS_COUNT,
            ..canonical
        },
        "basis_order_drift",
    );
}

fn leak(reason: &str) -> &'static str {
    Box::leak(reason.to_owned().into_boxed_str())
}

#[test]
fn canonical_basis_reproduces_the_stage_c_c6_evaluator_exactly() {
    let budget = StageCPreflightBudget::bounded(3, 1);
    let report = run_fixed_m_sensitivity(DataSplit::Development, budget).unwrap();
    let preflight = run_stage_c_preflight(DataSplit::Development, budget).unwrap();
    let canonical: Vec<_> = report
        .cases
        .iter()
        .filter(|case| case.basis_index == 0)
        .collect();
    assert_eq!(canonical.len(), preflight.c6_records.len());
    for (case, record) in canonical.iter().zip(&preflight.c6_records) {
        assert_eq!(record.arm, EvalArm::C6);
        assert_eq!((case.family, case.case_id), (record.family, record.case_id));
        match record.outcome {
            EvalOutcome::Scored { score, correct } => {
                assert_eq!(score.to_bits(), case.score.to_bits());
                assert_eq!(correct, case.correct);
            }
            ref other => panic!("unexpected outcome {other:?}"),
        }
    }
}

#[test]
fn hand_calculated_score_under_an_alternative_basis() {
    let query = Chiral6::new([1.0, 0.5, -1.0], [0.5, -1.5, 1.0]).unwrap();
    let key = Chiral6::new([-0.5, 1.0, 1.5], [1.0, 0.5, -2.0]).unwrap();
    // H+ = {0,1,3}, H- = {2,4,5}: rank 1 in the frozen enumeration.
    let basis = fixed_mirror_bases()[1];
    assert_eq!(basis.even_slots, [0, 1, 3]);
    assert_eq!(basis.odd_slots, [2, 4, 5]);
    // Pq = (1, 0.5, 0.5 | -1, -1.5, 1), Pk = (-0.5, 1, 1 | 1.5, 0.5, -2)
    // s = -3.75, chi = (1.5 - 0.5) + (0.25 + 1.5) + (-1 - 1) = 0.75
    let score = chiral_score(basis.apply(query), basis.apply(key), C6_REFERENCE_WEIGHTS).unwrap();
    assert_eq!(score, -3.75 + 0.75);
    assert_eq!(
        basis.apply(query).chiral_pairing(basis.apply(key)).unwrap(),
        0.75
    );
}

#[test]
fn smoke_study_covers_every_basis_and_family_on_both_non_final_splits() {
    let budget = StageCPreflightBudget::bounded(MAX_PREFLIGHT_PAIRS_PER_FAMILY, 0);
    for split in [DataSplit::Development, DataSplit::Validation] {
        let report = run_fixed_m_sensitivity(split, budget).unwrap();
        let per_basis = budget.cases_per_arm().unwrap();
        assert_eq!(report.split, split);
        assert_eq!(report.sensitivity_contract, FIXED_M_SENSITIVITY_CONTRACT);
        assert_eq!(report.bases, fixed_mirror_bases().to_vec());
        assert_eq!(report.weights, C6_REFERENCE_WEIGHTS);
        assert_eq!(report.capacity, TrainableCapacity::reference_c6());
        assert_eq!(
            report.cases.len() as u64,
            per_basis * FIXED_MIRROR_BASIS_COUNT as u64
        );
        assert_eq!(
            report.summaries.len(),
            FIXED_MIRROR_BASIS_COUNT * STAGE_C_PREFLIGHT_FAMILIES.len()
        );
        assert!(!report.protected_or_final_access);
        assert!(!report.training_executed);
        assert!(!report.scientific_claim);
        assert!(report.experimental_non_final);
        for group in report.cases.chunks(FIXED_MIRROR_BASIS_COUNT) {
            for case in group {
                let complement = &group[FIXED_MIRROR_BASIS_COUNT - 1 - case.basis_index];
                assert_eq!(case.chiral, -complement.chiral);
            }
        }
        validate_fixed_m_sensitivity_report(&report).unwrap();
    }
}

#[test]
fn tampered_reports_fail_closed() {
    let report =
        run_fixed_m_sensitivity(DataSplit::Development, StageCPreflightBudget::bounded(2, 0))
            .unwrap();
    let tamper = |mutate: &dyn Fn(&mut FixedMSensitivityReport), reason: &str| {
        let mut tampered = report.clone();
        mutate(&mut tampered);
        assert_eq!(rejection(&tampered), reason);
    };
    tamper(
        &|r| r.sensitivity_contract = "tdi24-fixed-m-sensitivity-v0",
        "contract_drift",
    );
    tamper(
        &|r| {
            r.bases.pop();
        },
        "basis_family_incomplete",
    );
    tamper(&|r| r.bases.swap(0, 1), "basis_order_drift");
    tamper(&|r| r.weights.gamma = 0.0, "weights_drift");
    tamper(
        &|r| r.capacity = TrainableCapacity::reference_v6(),
        "capacity_mismatch",
    );
    tamper(
        &|r| {
            r.cases.pop();
        },
        "case_count",
    );
    tamper(&|r| r.cases.swap(0, 1), "case_order");
    tamper(
        &|r| {
            let chiral = r.cases[1].chiral;
            r.cases[1].chiral = chiral + 1.0;
        },
        "complement_antisymmetry",
    );
    tamper(&|r| r.summaries[0].correct += 1, "summary_drift");
    // A coherent forgery of a score with consistent summaries is still caught
    // by regeneration.
    tamper(
        &|r| {
            r.cases[0].score += 1.0;
        },
        "case_evidence_drift",
    );
    tamper(
        &|r| r.protected_or_final_access = true,
        "protected_or_final_access",
    );
    tamper(&|r| r.training_executed = true, "training_executed");
    tamper(&|r| r.scientific_claim = true, "scientific_claim");
    tamper(
        &|r| r.experimental_non_final = false,
        "experimental_non_final",
    );
}

#[test]
fn protected_or_final_labels_and_bad_budgets_never_score() {
    for label in ["protected", "final", "holdout", ""] {
        assert_eq!(
            run_fixed_m_sensitivity_for_label(label, StageCPreflightBudget::bounded(1, 0)),
            Err(EvalError::ProtectedOrFinalSplit)
        );
    }
    assert_eq!(
        run_fixed_m_sensitivity(
            DataSplit::Development,
            StageCPreflightBudget::bounded(MAX_PREFLIGHT_PAIRS_PER_FAMILY + 1, 0)
        ),
        Err(EvalError::StageCPreflightInvalid {
            reason: "budget_exceeds_case_cap"
        })
    );
    assert_eq!(
        run_fixed_m_sensitivity(DataSplit::Development, StageCPreflightBudget::bounded(0, 0)),
        Err(EvalError::StageCPreflightInvalid {
            reason: "empty_budget"
        })
    );
}

#[test]
fn study_is_deterministic() {
    let budget = StageCPreflightBudget::bounded(2, 0);
    assert_eq!(
        run_fixed_m_sensitivity(DataSplit::Validation, budget).unwrap(),
        run_fixed_m_sensitivity(DataSplit::Validation, budget).unwrap()
    );
}
