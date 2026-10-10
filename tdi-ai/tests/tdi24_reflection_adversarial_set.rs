#![cfg(feature = "experimental")]

//! TDI-24 slice 43: reflection adversarial set (Phase E).
//!
//! Hard mirrored pairs built from declared geometry (dominance ratio of the
//! direct channel over the parity-odd channel, chirality scale, orthogonal
//! nuisance) and registered seeds only, independently of any model output.
//! C6 and its direct-only arm score the identical pairs. No training, no
//! protected/final access, no scientific claim.

use tdi_ai::experimental::tdi24_chiral::observables;
use tdi_ai::experimental::tdi24_eval::{
    ADVERSARIAL_ARMS, ADVERSARIAL_CHIRALITY_SCALES, ADVERSARIAL_DOMINANCE_RATIOS, EvalError,
    REFLECTION_ADVERSARIAL_CONTRACT, ReflectionAdversarialReport, SequenceArm,
    StageCPreflightBudget, TrainableCapacity, reflection_adversarial_pairs,
    reflection_adversarial_seed, run_reflection_adversarial_set,
    run_reflection_adversarial_set_for_label, validate_reflection_adversarial_report,
};
use tdi_ai::experimental::tdi24_tasks::DataSplit;

fn study(split: DataSplit) -> ReflectionAdversarialReport {
    run_reflection_adversarial_set(split, StageCPreflightBudget::bounded(4, 0)).unwrap()
}

fn rejection(report: &ReflectionAdversarialReport) -> &'static str {
    match validate_reflection_adversarial_report(report) {
        Err(EvalError::ReflectionAdversarialInvalid { reason }) => reason,
        other => panic!("expected a reflection-adversarial rejection, got {other:?}"),
    }
}

#[test]
fn grid_is_declared() {
    assert_eq!(
        REFLECTION_ADVERSARIAL_CONTRACT,
        "tdi24-reflection-adversarial-set-v1"
    );
    assert_eq!(ADVERSARIAL_DOMINANCE_RATIOS, [0.5, 0.9, 0.99, 1.01, 2.0]);
    assert_eq!(ADVERSARIAL_CHIRALITY_SCALES, [1.0, 1e-3]);
    assert_eq!(ADVERSARIAL_ARMS, [SequenceArm::C6, SequenceArm::DirectOnly]);
}

#[test]
fn pairs_are_exact_mirrors_with_declared_channels() {
    for split in [DataSplit::Development, DataSplit::Validation] {
        let budget = StageCPreflightBudget::bounded(4, 0);
        let pairs = reflection_adversarial_pairs(split, budget).unwrap();
        assert_eq!(
            pairs.len(),
            ADVERSARIAL_CHIRALITY_SCALES.len() * ADVERSARIAL_DOMINANCE_RATIOS.len() * 4
        );
        assert_eq!(pairs, reflection_adversarial_pairs(split, budget).unwrap());
        for pair in &pairs {
            assert_eq!(pair.left_query, pair.right_query.mirror());
            assert_eq!(pair.left_key, pair.right_key.mirror());
            let right = observables(pair.right_query, pair.right_key).unwrap();
            let left = observables(pair.left_query, pair.left_key).unwrap();
            // Mirroring keeps the even channels bit-for-bit and negates chi.
            assert_eq!(right.direct.to_bits(), left.direct.to_bits());
            assert_eq!(right.mirrored.to_bits(), left.mirrored.to_bits());
            assert_eq!(right.chiral.to_bits(), (-left.chiral).to_bits());
            assert!(right.chiral > 0.0);
            let declared = f64::from(pair.direct_sign) * pair.ratio;
            assert!((right.direct / right.chiral - declared).abs() < 1e-6);
            // The orthogonal nuisance dominates the key norm.
            let key_norm = pair.right_key.dot(pair.right_key).unwrap().sqrt();
            let query_norm = pair.right_query.dot(pair.right_query).unwrap().sqrt();
            assert!(key_norm >= 1.9 * query_norm);
        }
    }
    let development =
        reflection_adversarial_pairs(DataSplit::Development, StageCPreflightBudget::bounded(2, 0))
            .unwrap();
    let validation =
        reflection_adversarial_pairs(DataSplit::Validation, StageCPreflightBudget::bounded(2, 0))
            .unwrap();
    assert_ne!(development[0].right_query, validation[0].right_query);
}

#[test]
fn smoke_study_covers_the_grid_and_marks_the_decision_boundary() {
    for split in [DataSplit::Development, DataSplit::Validation] {
        boundary_holds(&study(split));
    }
}

fn boundary_holds(report: &ReflectionAdversarialReport) {
    validate_reflection_adversarial_report(report).unwrap();
    assert_eq!(report.generator_seed, reflection_adversarial_seed());
    assert_eq!(report.capacity, TrainableCapacity::reference_c6());
    assert_eq!(report.cells.len(), 2 * 5 * 2);
    for cell in &report.cells {
        assert_eq!(cell.n_pairs, 4);
        match cell.arm {
            // Identical even channels: one member right per pair, never both.
            SequenceArm::DirectOnly => {
                assert_eq!(cell.members_correct, 4);
                assert_eq!(cell.pairs_discriminated, 0);
            }
            // C6 separates a mirrored pair iff chi dominates |s|.
            SequenceArm::C6 if cell.ratio < 1.0 => {
                assert_eq!(cell.members_correct, 8);
                assert_eq!(cell.pairs_discriminated, 4);
            }
            SequenceArm::C6 => {
                assert_eq!(cell.members_correct, 4);
                assert_eq!(cell.pairs_discriminated, 0);
            }
        }
    }
    assert!(!report.protected_or_final_access);
    assert!(!report.training_executed);
    assert!(!report.scientific_claim);
    assert!(report.experimental_non_final);
}

#[test]
fn tampered_reports_fail_closed() {
    let report = study(DataSplit::Validation);

    let mut drifted = report.clone();
    drifted.adversarial_contract = "tdi24-reflection-adversarial-set-v0";
    assert_eq!(rejection(&drifted), "contract_drift");

    let mut drifted = report.clone();
    drifted.generator_seed ^= 1;
    assert_eq!(rejection(&drifted), "seed_drift");

    let mut drifted = report.clone();
    drifted.capacity.trainable_parameters += 1;
    assert_eq!(rejection(&drifted), "capacity_mismatch");

    let mut drifted = report.clone();
    drifted.cells.pop();
    assert_eq!(rejection(&drifted), "cell_count");

    let mut drifted = report.clone();
    drifted.cells.swap(0, 1);
    assert_eq!(rejection(&drifted), "grid_drift");

    let mut drifted = report.clone();
    drifted.cells[0].members_correct = 9;
    assert_eq!(rejection(&drifted), "paired_count_drift");

    let mut drifted = report.clone();
    drifted.cells[1].members_correct = 8;
    drifted.cells[1].pairs_discriminated = 4;
    assert_eq!(rejection(&drifted), "direct_only_discrimination");

    let mut drifted = report.clone();
    drifted.pair_digest ^= 1;
    assert_eq!(rejection(&drifted), "pair_digest_drift");

    let mut drifted = report.clone();
    drifted.cells[8].members_correct -= 1;
    assert_eq!(rejection(&drifted), "case_evidence_drift");

    for (flag, reason) in [
        (0, "protected_or_final_access"),
        (1, "training_executed"),
        (2, "scientific_claim"),
        (3, "experimental_non_final"),
    ] {
        let mut drifted = report.clone();
        match flag {
            0 => drifted.protected_or_final_access = true,
            1 => drifted.training_executed = true,
            2 => drifted.scientific_claim = true,
            _ => drifted.experimental_non_final = false,
        }
        assert_eq!(rejection(&drifted), reason);
    }
}

#[test]
fn protected_or_final_labels_and_bad_budgets_never_score() {
    for label in ["protected", "final", "holdout", ""] {
        assert_eq!(
            run_reflection_adversarial_set_for_label(label, StageCPreflightBudget::bounded(2, 0)),
            Err(EvalError::ProtectedOrFinalSplit)
        );
    }
    assert!(
        run_reflection_adversarial_set(
            DataSplit::Development,
            StageCPreflightBudget::bounded(0, 0)
        )
        .is_err()
    );
    assert!(
        run_reflection_adversarial_set(
            DataSplit::Development,
            StageCPreflightBudget::bounded(9, 0)
        )
        .is_err()
    );
}

#[test]
fn study_is_deterministic() {
    assert_eq!(study(DataSplit::Development), study(DataSplit::Development));
    assert_ne!(
        study(DataSplit::Development).pair_digest,
        study(DataSplit::Validation).pair_digest
    );
}
