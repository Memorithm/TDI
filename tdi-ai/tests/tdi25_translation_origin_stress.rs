//! TDI-25 slice 43: translation/origin stress suite (Phase E).
//!
//! Declared torsor-relevant transformations (rigid origin shift, key
//! re-reduction, query re-reduction) at declared offsets, drawn from the
//! contract seed independently of any model output and applied identically
//! to T6 and C6 on the bounded matched Development/Validation population.
//! No training, no protected/final access, no scientific claim.
#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi22_torsor::{Torsor3, Twist3, Vec3, direct_pairing};
use tdi_ai::experimental::tdi25_eval::matched_reference::{
    MATCHED_POPULATION_CONTRACT, MatchedInput, MatchedPrimaryRun,
};
use tdi_ai::experimental::tdi25_eval::{
    EvalError, ORIGIN_STRESS_OFFSETS, ORIGIN_STRESS_TRANSFORMS, OriginStressTransform,
    REQUIRED_SYNTHESIS_FAMILIES, SEQUENCE_SCALING_ARMS, StageCPreflightBudget,
    TRANSLATION_ORIGIN_STRESS_CONTRACT, TranslationOriginStressReport, origin_stress_direction,
    run_translation_origin_stress, run_translation_origin_stress_for_label, stress_matched_input,
    translation_origin_stress_seed, validate_translation_origin_stress_report,
};
use tdi_ai::experimental::tdi25_tasks::DataSplit;
use tdi_ai::experimental::tdi25_torsor_chiral::{ComparisonArm, TaskFamily};

fn study(split: DataSplit) -> TranslationOriginStressReport {
    run_translation_origin_stress(split, StageCPreflightBudget::smoke()).unwrap()
}

fn rejection(report: &TranslationOriginStressReport) -> &'static str {
    match validate_translation_origin_stress_report(report) {
        Err(EvalError::TranslationOriginStressInvalid { reason }) => reason,
        other => panic!("expected a translation/origin stress rejection, got {other:?}"),
    }
}

fn physical_pairing(input: &MatchedInput) -> f64 {
    let v = |a: [f64; 6], o: usize| Vec3::new(a[o], a[o + 1], a[o + 2]).unwrap();
    let p = |a: [f64; 3]| Vec3::new(a[0], a[1], a[2]).unwrap();
    let query = Twist3::new(v(input.query(), 0), v(input.query(), 3)).unwrap();
    let key = Torsor3::new(
        v(input.key(), 0),
        v(input.key(), 3),
        p(input.key_position()),
    )
    .unwrap();
    direct_pairing(query, key, p(input.query_position())).unwrap()
}

#[test]
fn grid_and_seed_are_declared() {
    assert_eq!(
        TRANSLATION_ORIGIN_STRESS_CONTRACT,
        "tdi25-translation-origin-stress-v1"
    );
    assert_eq!(
        ORIGIN_STRESS_TRANSFORMS,
        [
            OriginStressTransform::OriginShift,
            OriginStressTransform::KeyReduction,
            OriginStressTransform::QueryReduction
        ]
    );
    assert_eq!(ORIGIN_STRESS_OFFSETS, [1.0, 1e3, 1e6]);
    let fnv = TRANSLATION_ORIGIN_STRESS_CONTRACT
        .bytes()
        .fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
            (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
        });
    assert_eq!(translation_origin_stress_seed(), fnv);
}

#[test]
fn transformations_preserve_the_physical_pairing_and_touch_declared_fields() {
    let run =
        MatchedPrimaryRun::evaluate(DataSplit::Development, TaskFamily::TorsorFavorable, 0, 8)
            .unwrap();
    let seed = translation_origin_stress_seed();
    for (index, input) in run.inputs().iter().enumerate() {
        let case_key = index as u64;
        let direction = origin_stress_direction(seed, case_key).unwrap();
        let norm: f64 = direction.iter().map(|x| x * x).sum::<f64>().sqrt();
        assert!((norm - 1.0).abs() < 1e-12);
        assert_eq!(direction, origin_stress_direction(seed, case_key).unwrap());
        let reference = physical_pairing(input);
        for offset in ORIGIN_STRESS_OFFSETS {
            let shifted = stress_matched_input(
                input,
                OriginStressTransform::OriginShift,
                offset,
                seed,
                case_key,
            )
            .unwrap();
            assert_eq!(shifted.query(), input.query());
            assert_eq!(shifted.key(), input.key());
            let key_moved = stress_matched_input(
                input,
                OriginStressTransform::KeyReduction,
                offset,
                seed,
                case_key,
            )
            .unwrap();
            assert_eq!(key_moved.query(), input.query());
            assert_eq!(key_moved.query_position(), input.query_position());
            assert_eq!(key_moved.key()[..3], input.key()[..3]);
            let query_moved = stress_matched_input(
                input,
                OriginStressTransform::QueryReduction,
                offset,
                seed,
                case_key,
            )
            .unwrap();
            assert_eq!(query_moved.key(), input.key());
            assert_eq!(query_moved.query()[3..], input.query()[3..]);
            // Invariant in real arithmetic; rounding grows with the offset.
            let tolerance = 1e-9 * offset * (1.0 + reference.abs());
            for moved in [&shifted, &key_moved, &query_moved] {
                assert!((physical_pairing(moved) - reference).abs() <= tolerance);
            }
        }
    }
    assert_eq!(
        stress_matched_input(
            &run.inputs()[0],
            OriginStressTransform::OriginShift,
            2.0,
            seed,
            0
        ),
        Err(EvalError::TranslationOriginStressInvalid {
            reason: "offset_not_registered"
        })
    );
}

#[test]
fn smoke_study_covers_the_full_grid_with_paired_counts() {
    for split in [DataSplit::Development, DataSplit::Validation] {
        let report = study(split);
        validate_translation_origin_stress_report(&report).unwrap();
        assert_eq!(report.population_contract, MATCHED_POPULATION_CONTRACT);
        assert_eq!(report.stress_seed, translation_origin_stress_seed());
        let budget = StageCPreflightBudget::smoke();
        assert_eq!(
            report.cells.len() as u64,
            REQUIRED_SYNTHESIS_FAMILIES.len() as u64
                * budget.seed_blocks
                * (ORIGIN_STRESS_TRANSFORMS.len()
                    * ORIGIN_STRESS_OFFSETS.len()
                    * SEQUENCE_SCALING_ARMS.len()) as u64
        );
        for cell in &report.cells {
            assert_eq!(cell.n_cases, budget.cases_per_block);
            if cell.arm == ComparisonArm::C6 && cell.transform == OriginStressTransform::OriginShift
            {
                // C6 reads no position: bit-for-bit unchanged.
                assert_eq!(cell.max_abs_score_change, 0.0);
                assert_eq!(cell.flips, 0);
            }
            if cell.family == TaskFamily::TorsorFavorable && cell.arm == ComparisonArm::T6 {
                assert_eq!(cell.clean_matches, cell.n_cases);
                // The physical pairing is invariant; at offsets up to 1e3 the
                // rounding stays inside the shared match tolerance.
                if cell.offset <= 1e3 {
                    assert_eq!(cell.flips, 0);
                    assert!(cell.max_abs_target_change < 1e-11);
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

    let mut drifted = report.clone();
    drifted.stress_contract = "tdi25-translation-origin-stress-v0";
    assert_eq!(rejection(&drifted), "contract_drift");

    let mut drifted = report.clone();
    drifted.population_contract = "tdi25-legacy-population";
    assert_eq!(rejection(&drifted), "population_drift");

    let mut drifted = report.clone();
    drifted.stress_seed ^= 1;
    assert_eq!(rejection(&drifted), "seed_drift");

    let mut drifted = report.clone();
    drifted.t6_capacity = drifted.c6_capacity;
    assert_eq!(rejection(&drifted), "capacity_mismatch");

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
    drifted.cells[0].max_abs_target_change = f64::NAN;
    assert_eq!(rejection(&drifted), "score_change_drift");

    // Cell 1: TorsorFavorable, block 0, OriginShift, offset 1, C6.
    let mut drifted = report.clone();
    assert_eq!(drifted.cells[1].arm, ComparisonArm::C6);
    drifted.cells[1].max_abs_score_change = 1e-15;
    assert_eq!(rejection(&drifted), "c6_origin_shift_drift");

    let mut drifted = report.clone();
    drifted.cells[0].max_abs_score_change += 1.0;
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
            run_translation_origin_stress_for_label(label, StageCPreflightBudget::smoke()),
            Err(EvalError::ProtectedOrFinalSplit)
        );
    }
    let mut bad = StageCPreflightBudget::smoke();
    bad.cases_per_block = 0;
    assert!(run_translation_origin_stress(DataSplit::Development, bad).is_err());
}

#[test]
fn study_is_deterministic() {
    assert_eq!(study(DataSplit::Development), study(DataSplit::Development));
    assert_ne!(study(DataSplit::Development), study(DataSplit::Validation));
}
