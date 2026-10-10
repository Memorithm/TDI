//! TDI-25 slice 45: mixed adversarial suite (Phase E).
//!
//! Every declared slice-44 mirror/parity transformation composed with every
//! declared slice-43 translation/origin transformation and offset, reusing
//! the slice-43 contract seed, applied identically to T6 and C6 on the
//! bounded matched Development/Validation population under the evaluator's
//! deterministic common-target oracle. No training, no protected/final
//! access, no scientific claim.
#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi25_eval::matched_reference::{
    MATCHED_POPULATION_CONTRACT, MatchedPrimaryRun,
};
use tdi_ai::experimental::tdi25_eval::{
    EvalError, MIRROR_STRESS_TRANSFORMS, MIXED_ADVERSARIAL_SUITE_CONTRACT,
    MixedAdversarialSuiteReport, ORIGIN_STRESS_OFFSETS, ORIGIN_STRESS_TRANSFORMS,
    OriginStressTransform, REQUIRED_SYNTHESIS_FAMILIES, SEQUENCE_SCALING_ARMS,
    StageCPreflightBudget, mirror_stress_matched_input, mixed_stress_matched_input,
    run_mixed_adversarial_suite, run_mixed_adversarial_suite_for_label, stress_matched_input,
    translation_origin_stress_seed, validate_mixed_adversarial_suite_report,
};
use tdi_ai::experimental::tdi25_tasks::DataSplit;
use tdi_ai::experimental::tdi25_torsor_chiral::{ComparisonArm, TaskFamily};

fn study(split: DataSplit) -> MixedAdversarialSuiteReport {
    run_mixed_adversarial_suite(split, StageCPreflightBudget::smoke()).unwrap()
}

fn rejection(report: &MixedAdversarialSuiteReport) -> &'static str {
    match validate_mixed_adversarial_suite_report(report) {
        Err(EvalError::MixedAdversarialStressInvalid { reason }) => reason,
        other => panic!("expected a mixed adversarial rejection, got {other:?}"),
    }
}

#[test]
fn contract_and_reused_grid_are_declared() {
    assert_eq!(
        MIXED_ADVERSARIAL_SUITE_CONTRACT,
        "tdi25-mixed-adversarial-suite-v1"
    );
    let report = study(DataSplit::Validation);
    assert_eq!(report.stress_seed, translation_origin_stress_seed());
    assert_eq!(report.population_contract, MATCHED_POPULATION_CONTRACT);
    let per_block = MIRROR_STRESS_TRANSFORMS.len()
        * ORIGIN_STRESS_TRANSFORMS.len()
        * ORIGIN_STRESS_OFFSETS.len()
        * SEQUENCE_SCALING_ARMS.len();
    assert_eq!(per_block, 54);
    assert_eq!(
        report.cells.len() as u64,
        REQUIRED_SYNTHESIS_FAMILIES.len() as u64
            * StageCPreflightBudget::smoke().seed_blocks
            * per_block as u64
    );
}

#[test]
fn composition_is_mirror_then_origin() {
    let run = MatchedPrimaryRun::evaluate(DataSplit::Validation, TaskFamily::Mixed, 0, 4).unwrap();
    let seed = translation_origin_stress_seed();
    for (index, input) in run.inputs().iter().enumerate() {
        let case_key = index as u64;
        for mirror in MIRROR_STRESS_TRANSFORMS {
            for origin in ORIGIN_STRESS_TRANSFORMS {
                for offset in ORIGIN_STRESS_OFFSETS {
                    let composed =
                        mixed_stress_matched_input(input, mirror, origin, offset, seed, case_key)
                            .unwrap();
                    let expected = stress_matched_input(
                        &mirror_stress_matched_input(input, mirror).unwrap(),
                        origin,
                        offset,
                        seed,
                        case_key,
                    )
                    .unwrap();
                    assert_eq!(composed, expected);
                }
            }
        }
    }
    assert!(
        mixed_stress_matched_input(
            &run.inputs()[0],
            MIRROR_STRESS_TRANSFORMS[0],
            OriginStressTransform::OriginShift,
            2.0,
            seed,
            0
        )
        .is_err()
    );
}

#[test]
fn smoke_suite_reports_every_cell_with_structural_invariants() {
    for split in [DataSplit::Development, DataSplit::Validation] {
        let report = study(split);
        validate_mixed_adversarial_suite_report(&report).unwrap();
        for cell in &report.cells {
            assert_eq!(cell.n_cases, StageCPreflightBudget::smoke().cases_per_block);
            if cell.arm == ComparisonArm::C6 {
                if cell.origin == OriginStressTransform::OriginShift {
                    assert_eq!(cell.max_abs_origin_effect, 0.0);
                }
                if cell.family == TaskFamily::ChiralFavorable {
                    assert_eq!(cell.stressed_matches, cell.n_cases);
                    assert_eq!(cell.flips, 0);
                }
            }
        }
        assert!(!report.protected_or_final_access);
        assert!(!report.training_executed);
        assert!(!report.scientific_claim);
        assert!(report.experimental_non_final);
    }
}

#[test]
fn tampered_reports_fail_closed() {
    let report = study(DataSplit::Validation);
    let c6_shift = report
        .cells
        .iter()
        .position(|cell| {
            cell.arm == ComparisonArm::C6 && cell.origin == OriginStressTransform::OriginShift
        })
        .unwrap();
    let c6_chiral = report
        .cells
        .iter()
        .position(|cell| {
            cell.arm == ComparisonArm::C6 && cell.family == TaskFamily::ChiralFavorable
        })
        .unwrap();

    let mut drifted = report.clone();
    drifted.suite_contract = "tdi25-mixed-adversarial-suite-v0";
    assert_eq!(rejection(&drifted), "contract_drift");

    let mut drifted = report.clone();
    drifted.population_contract = "other";
    assert_eq!(rejection(&drifted), "population_drift");

    let mut drifted = report.clone();
    drifted.stress_seed ^= 1;
    assert_eq!(rejection(&drifted), "seed_drift");

    let mut drifted = report.clone();
    drifted.cells.pop();
    assert_eq!(rejection(&drifted), "cell_count");

    let mut drifted = report.clone();
    drifted.cells.swap(0, 1);
    assert_eq!(rejection(&drifted), "grid_drift");

    let mut drifted = report.clone();
    drifted.cells[0].flips = drifted.cells[0].n_cases + 1;
    assert_eq!(rejection(&drifted), "paired_count_drift");

    let mut drifted = report.clone();
    drifted.cells[0].max_abs_origin_effect = f64::NAN;
    assert_eq!(rejection(&drifted), "score_change_drift");

    let mut drifted = report.clone();
    drifted.cells[c6_shift].max_abs_origin_effect = 1e-9;
    assert_eq!(rejection(&drifted), "c6_origin_shift_drift");

    let mut drifted = report.clone();
    drifted.cells[c6_chiral].stressed_matches -= 1;
    drifted.cells[c6_chiral].flips += 1;
    assert_eq!(rejection(&drifted), "c6_chiral_target_drift");

    let mut drifted = report.clone();
    drifted.cells[0].max_abs_target_change += 1.0;
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
fn protected_or_final_labels_never_generate_a_case() {
    for label in ["protected", "final", "holdout", ""] {
        assert!(matches!(
            run_mixed_adversarial_suite_for_label(label, StageCPreflightBudget::smoke()),
            Err(EvalError::ProtectedOrFinalSplit)
        ));
    }
}

#[test]
fn suite_is_deterministic() {
    assert_eq!(study(DataSplit::Development), study(DataSplit::Development));
}
