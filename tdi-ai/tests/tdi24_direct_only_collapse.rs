#![cfg(feature = "experimental")]

//! TDI-24 slice 33: direct-only collapse (Phase D).
//!
//! Zeroes both extra C6 channels (mirror-even `beta`, parity-odd `gamma`) of the
//! matched C6 reference and reruns the bounded Stage-C case stream on synthetic
//! Development/Validation cases. The collapsed C6 path must reproduce the matched
//! V6 score semantics (`score_V = s = q^T k`) bit-for-bit on every case. Because
//! the matched reference already carries `beta=0`, the collapse has the same
//! weights as the slice-31 `gamma=0` ablation; the report records that
//! explicitly. These tests qualify software semantics and exact identities only:
//! no training, no protected/final access, no attribution or scientific claim.

use tdi_ai::experimental::tdi24_chiral::{
    Chiral6, ChiralScoreWeights, chiral_score, observables, vector_score,
};
use tdi_ai::experimental::tdi24_eval::{
    C6_REFERENCE_WEIGHTS, DIRECT_ONLY_COLLAPSE_CONTRACT, DIRECT_ONLY_V6_MATCH_TOLERANCE,
    DirectOnlyCollapseReport, EvalArm, EvalError, EvalOutcome, MAX_PREFLIGHT_PAIRS_PER_FAMILY,
    STAGE_C_PREFLIGHT_FAMILIES, StageCPreflightBudget, TrainableCapacity, direct_only_weights,
    gamma_zero_weights, run_direct_only_collapse, run_direct_only_collapse_for_label,
    run_gamma_zero_ablation, run_stage_c_preflight, validate_direct_only_collapse_report,
};
use tdi_ai::experimental::tdi24_tasks::DataSplit;
use tdi_ai::experimental::tdi24_vector::{Vector6, vector6_score};

#[test]
fn direct_only_zeroes_both_extra_channels_and_keeps_alpha() {
    assert_eq!(
        DIRECT_ONLY_COLLAPSE_CONTRACT,
        "tdi24-direct-only-collapse-v1"
    );
    assert_eq!(DIRECT_ONLY_V6_MATCH_TOLERANCE.to_bits(), 0.0_f64.to_bits());
    let collapsed = direct_only_weights(C6_REFERENCE_WEIGHTS);
    assert_eq!(collapsed.alpha, C6_REFERENCE_WEIGHTS.alpha);
    assert_eq!(collapsed.alpha, 1.0);
    assert_eq!(collapsed.beta, 0.0);
    assert_eq!(collapsed.gamma, 0.0);
    // Degeneracy recorded: the reference already omits the mirror-even channel,
    // so the collapse coincides with the slice-31 gamma=0 ablation weights.
    assert_eq!(C6_REFERENCE_WEIGHTS.beta, 0.0);
    assert_eq!(collapsed, gamma_zero_weights(C6_REFERENCE_WEIGHTS));
    assert_ne!(collapsed, C6_REFERENCE_WEIGHTS);
}

#[test]
fn direct_only_collapse_matches_v6_generically_and_needs_unit_alpha() {
    // Off-campaign algebra check with non-zero beta and gamma.
    let even = [1.0, -0.5, 2.0];
    let odd = [0.25, 1.5, -1.0];
    let key_even = [-0.75, 1.0, 0.5];
    let key_odd = [2.0, -0.25, 1.25];
    let query = Chiral6::new(even, odd).unwrap();
    let key = Chiral6::new(key_even, key_odd).unwrap();
    let weights = ChiralScoreWeights::new(1.0, 0.5, 1.0).unwrap();
    let collapsed = chiral_score(query, key, direct_only_weights(weights)).unwrap();
    let channels = observables(query, key).unwrap();
    assert_eq!(
        chiral_score(query, key, weights).unwrap(),
        collapsed + weights.beta * channels.mirrored + weights.gamma * channels.chiral
    );
    let v6 = vector6_score(
        Vector6::new(query.as_array()).unwrap(),
        Vector6::new(key.as_array()).unwrap(),
    )
    .unwrap();
    assert_eq!(collapsed.to_bits(), v6.to_bits());
    assert_eq!(
        collapsed.to_bits(),
        vector_score(query, key).unwrap().to_bits()
    );
    // A non-unit alpha would not be V6 score semantics.
    let scaled = ChiralScoreWeights::new(2.0, 0.5, 1.0).unwrap();
    let scaled_collapse = chiral_score(query, key, direct_only_weights(scaled)).unwrap();
    assert_ne!(scaled_collapse, v6);
    assert_eq!(scaled_collapse, 2.0 * v6);
}

#[test]
fn direct_only_collapse_matches_v6_on_both_non_final_splits() {
    for split in [DataSplit::Development, DataSplit::Validation] {
        let budget = StageCPreflightBudget::bounded(4, 3);
        let report = run_direct_only_collapse(split, budget).unwrap();
        validate_direct_only_collapse_report(&report).unwrap();
        assert_eq!(report.split, split);
        assert_eq!(report.cases.len(), 32);
        assert!(report.collapse_coincides_with_gamma_zero);
        assert_eq!(report.reference_capacity, TrainableCapacity::reference_c6());
        assert_eq!(report.reference_capacity, report.collapsed_capacity);
        assert_eq!(report.v6_capacity, TrainableCapacity::reference_v6());
        assert_eq!(report.collapsed_capacity.trainable_parameters, 0);
        assert_eq!(
            report.collapsed_capacity.carrier_width,
            report.v6_capacity.carrier_width
        );
        assert!(!report.protected_or_final_access);
        assert!(!report.training_executed);
        assert!(!report.scientific_claim);
        assert!(report.experimental_non_final);
        for case in &report.cases {
            assert_eq!(case.collapsed_score.to_bits(), case.v6_score.to_bits());
            assert_eq!(case.collapsed_correct, case.v6_correct);
            assert_eq!(
                case.reference_score,
                case.collapsed_score + C6_REFERENCE_WEIGHTS.gamma * case.parity_odd_channel
            );
        }
        assert!(
            report
                .cases
                .iter()
                .any(|case| case.reference_score != case.collapsed_score),
            "the collapse is not the identity on the stream"
        );
        assert_eq!(report.families.len(), STAGE_C_PREFLIGHT_FAMILIES.len());
        for (summary, family) in report.families.iter().zip(STAGE_C_PREFLIGHT_FAMILIES) {
            assert_eq!(summary.family, *family);
            assert_eq!(summary.n_cases, 8);
            assert_eq!(summary.collapsed_correct, summary.v6_correct);
        }
    }
}

#[test]
fn both_sides_reproduce_the_stage_c_evaluators_exactly() {
    let budget = StageCPreflightBudget::bounded(3, 1);
    let collapse = run_direct_only_collapse(DataSplit::Development, budget).unwrap();
    let preflight = run_stage_c_preflight(DataSplit::Development, budget).unwrap();
    assert_eq!(collapse.cases.len(), preflight.c6_records.len());
    assert_eq!(collapse.cases.len(), preflight.v6_records.len());
    for ((case, c6), v6) in collapse
        .cases
        .iter()
        .zip(&preflight.c6_records)
        .zip(&preflight.v6_records)
    {
        assert_eq!(c6.arm, EvalArm::C6);
        assert_eq!(v6.arm, EvalArm::V6);
        assert_eq!((case.family, case.case_id), (c6.family, c6.case_id));
        assert_eq!((case.family, case.case_id), (v6.family, v6.case_id));
        match (c6.outcome, v6.outcome) {
            (
                EvalOutcome::Scored {
                    score: c6_score,
                    correct: c6_correct,
                },
                EvalOutcome::Scored {
                    score: v6_score,
                    correct: v6_correct,
                },
            ) => {
                assert_eq!(c6_score.to_bits(), case.reference_score.to_bits());
                assert_eq!(c6_correct, case.reference_correct);
                assert_eq!(v6_score.to_bits(), case.v6_score.to_bits());
                assert_eq!(v6_score.to_bits(), case.collapsed_score.to_bits());
                assert_eq!(v6_correct, case.collapsed_correct);
            }
            other => panic!("unexpected outcomes {other:?}"),
        }
    }
}

#[test]
fn collapse_coincides_with_the_gamma_zero_ablation_bit_for_bit() {
    let budget = StageCPreflightBudget::bounded(MAX_PREFLIGHT_PAIRS_PER_FAMILY, 0);
    let collapse = run_direct_only_collapse(DataSplit::Validation, budget).unwrap();
    let gamma_zero = run_gamma_zero_ablation(DataSplit::Validation, budget).unwrap();
    assert_eq!(collapse.collapsed_weights, gamma_zero.ablated_weights);
    for (case, ablated) in collapse.cases.iter().zip(&gamma_zero.cases) {
        assert_eq!(
            (case.family, case.case_id),
            (ablated.family, ablated.case_id)
        );
        assert_eq!(
            case.collapsed_score.to_bits(),
            ablated.ablated_score.to_bits()
        );
        assert_eq!(case.collapsed_correct, ablated.ablated_correct);
    }
}

#[test]
fn direct_only_collapse_is_deterministic_and_bounded() {
    let budget = StageCPreflightBudget::bounded(2, 0);
    assert_eq!(
        run_direct_only_collapse(DataSplit::Validation, budget).unwrap(),
        run_direct_only_collapse(DataSplit::Validation, budget).unwrap()
    );
    let full = StageCPreflightBudget::bounded(MAX_PREFLIGHT_PAIRS_PER_FAMILY, 0);
    assert_eq!(
        run_direct_only_collapse(DataSplit::Development, full)
            .unwrap()
            .cases
            .len(),
        64
    );
    assert_eq!(
        run_direct_only_collapse(
            DataSplit::Development,
            StageCPreflightBudget::bounded(MAX_PREFLIGHT_PAIRS_PER_FAMILY + 1, 0)
        ),
        Err(EvalError::StageCPreflightInvalid {
            reason: "budget_exceeds_case_cap"
        })
    );
}

#[test]
fn protected_or_final_labels_never_generate_a_collapse_case() {
    for label in ["protected", "final", "holdout", ""] {
        assert_eq!(
            run_direct_only_collapse_for_label(label, StageCPreflightBudget::bounded(1, 0)),
            Err(EvalError::ProtectedOrFinalSplit)
        );
    }
}

#[test]
fn drifted_collapse_reports_fail_closed() {
    let report =
        run_direct_only_collapse(DataSplit::Development, StageCPreflightBudget::bounded(1, 0))
            .unwrap();
    let active = report
        .cases
        .iter()
        .position(|case| case.parity_odd_channel != 0.0)
        .expect("the stream exercises a non-zero parity-odd observable");
    let reject = |mutate: &dyn Fn(&mut DirectOnlyCollapseReport), reason: &'static str| {
        let mut tampered = report.clone();
        mutate(&mut tampered);
        assert_eq!(
            validate_direct_only_collapse_report(&tampered),
            Err(EvalError::DirectOnlyCollapseInvalid { reason })
        );
    };
    reject(
        &|r| r.collapse_contract = "tdi24-direct-only-collapse-v0",
        "contract_drift",
    );
    reject(
        &|r| r.reference_weights.gamma = 0.5,
        "reference_weights_drift",
    );
    reject(
        &|r| r.collapsed_weights.gamma = 1.0,
        "collapsed_weights_drift",
    );
    reject(
        &|r| r.collapsed_weights.beta = 0.5,
        "collapsed_weights_drift",
    );
    reject(
        &|r| r.collapsed_weights.alpha = 2.0,
        "collapsed_weights_drift",
    );
    reject(&|r| r.v6_match_tolerance = 1e-12, "tolerance_drift");
    reject(&|r| r.v6_match_tolerance = -0.0, "tolerance_drift");
    reject(
        &|r| r.collapse_coincides_with_gamma_zero = false,
        "degeneracy_flag_drift",
    );
    reject(
        &|r| r.collapsed_capacity.trainable_parameters = 1,
        "capacity_mismatch",
    );
    reject(&|r| r.v6_capacity.carrier_width = 5, "capacity_mismatch");
    reject(
        &|r| r.v6_capacity = TrainableCapacity::reference_c6(),
        "capacity_mismatch",
    );
    reject(
        &|r| {
            r.cases.pop();
        },
        "case_count",
    );
    reject(
        &|r| r.cases[active].collapsed_score += 1.0,
        "removed_channel_residual",
    );
    reject(
        &|r| r.cases[active].parity_odd_channel = 0.0,
        "removed_channel_residual",
    );
    reject(
        &|r| r.cases[0].v6_score = f64::from_bits(r.cases[0].v6_score.to_bits() ^ 1),
        "v6_score_mismatch",
    );
    reject(
        &|r| r.cases[0].v6_correct = !r.cases[0].v6_correct,
        "v6_score_mismatch",
    );
    reject(&|r| r.families[0].v6_correct += 1, "family_summary_drift");
    reject(
        &|r| r.protected_or_final_access = true,
        "protected_or_final_access",
    );
    reject(&|r| r.training_executed = true, "training_executed");
    reject(&|r| r.scientific_claim = true, "scientific_claim");
    reject(
        &|r| r.experimental_non_final = false,
        "experimental_non_final",
    );
}
