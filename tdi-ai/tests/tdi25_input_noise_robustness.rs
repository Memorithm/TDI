//! TDI-25 slice 42: input/noise robustness (Phase E).
//!
//! Declared deterministic perturbation families (isotropic, carrier-only,
//! position-only) at declared amplitudes on the 18 shared input scalars,
//! applied identically to T6 and C6 on the bounded matched
//! Development/Validation population. No training, no protected/final
//! access, no scientific claim.
#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi25_eval::matched_reference::{
    MATCHED_POPULATION_CONTRACT, MatchedPrimaryRun,
};
use tdi_ai::experimental::tdi25_eval::{
    EvalError, INPUT_NOISE_AMPLITUDES, INPUT_NOISE_FAMILIES, INPUT_NOISE_ROBUSTNESS_CONTRACT,
    InputNoiseFamily, InputNoiseRobustnessReport, REQUIRED_SYNTHESIS_FAMILIES,
    SEQUENCE_SCALING_ARMS, StageCPreflightBudget, evaluate_input_noise_robustness,
    input_noise_seed, perturb_matched_input, run_input_noise_robustness,
    run_input_noise_robustness_for_label, validate_input_noise_robustness_report,
};
use tdi_ai::experimental::tdi25_tasks::DataSplit;
use tdi_ai::experimental::tdi25_torsor_chiral::TaskFamily;

fn study(split: DataSplit) -> InputNoiseRobustnessReport {
    run_input_noise_robustness(split, StageCPreflightBudget::smoke()).unwrap()
}

fn rejection(report: &InputNoiseRobustnessReport) -> &'static str {
    match validate_input_noise_robustness_report(report) {
        Err(EvalError::InputNoiseRobustnessInvalid { reason }) => reason,
        other => panic!("expected a noise-robustness rejection, got {other:?}"),
    }
}

#[test]
fn grid_and_seed_are_declared() {
    assert_eq!(
        INPUT_NOISE_ROBUSTNESS_CONTRACT,
        "tdi25-input-noise-robustness-v1"
    );
    assert_eq!(
        INPUT_NOISE_FAMILIES,
        [
            InputNoiseFamily::Isotropic,
            InputNoiseFamily::CarrierOnly,
            InputNoiseFamily::PositionOnly
        ]
    );
    assert_eq!(INPUT_NOISE_AMPLITUDES, [1e-3, 1e-2, 1e-1]);
    let fnv = INPUT_NOISE_ROBUSTNESS_CONTRACT
        .bytes()
        .fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
            (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
        });
    assert_eq!(input_noise_seed(), fnv);
}

#[test]
fn perturbations_are_bounded_masked_and_deterministic() {
    let run =
        MatchedPrimaryRun::evaluate(DataSplit::Development, TaskFamily::TorsorFavorable, 0, 4)
            .unwrap();
    let input = &run.inputs()[1];
    let seed = input_noise_seed();
    for amplitude in INPUT_NOISE_AMPLITUDES {
        let carrier =
            perturb_matched_input(input, InputNoiseFamily::CarrierOnly, amplitude, seed, 1)
                .unwrap();
        assert_eq!(carrier.key_position(), input.key_position());
        assert_eq!(carrier.query_position(), input.query_position());
        assert_ne!(carrier.query(), input.query());
        let position =
            perturb_matched_input(input, InputNoiseFamily::PositionOnly, amplitude, seed, 1)
                .unwrap();
        assert_eq!(position.query(), input.query());
        assert_eq!(position.key(), input.key());
        assert_ne!(position.key_position(), input.key_position());
        let iso =
            perturb_matched_input(input, InputNoiseFamily::Isotropic, amplitude, seed, 1).unwrap();
        let pairs = iso
            .query()
            .into_iter()
            .zip(input.query())
            .chain(iso.key().into_iter().zip(input.key()))
            .chain(iso.key_position().into_iter().zip(input.key_position()))
            .chain(iso.query_position().into_iter().zip(input.query_position()));
        for (a, b) in pairs {
            assert!((a - b).abs() <= amplitude);
        }
        // Isotropic draws coincide slot-by-slot with the masked families.
        assert_eq!(iso.query(), carrier.query());
        assert_eq!(iso.key_position(), position.key_position());
        assert_eq!(
            iso,
            perturb_matched_input(input, InputNoiseFamily::Isotropic, amplitude, seed, 1).unwrap()
        );
    }
    assert_eq!(
        perturb_matched_input(input, InputNoiseFamily::Isotropic, 0.5, seed, 1),
        Err(EvalError::InputNoiseRobustnessInvalid {
            reason: "amplitude_not_registered"
        })
    );
}

#[test]
fn cells_cover_the_grid_and_reproduce_the_clean_primary() {
    let report = study(DataSplit::Development);
    assert_eq!(report.population_contract, MATCHED_POPULATION_CONTRACT);
    let budget = StageCPreflightBudget::smoke();
    assert_eq!(
        report.cells.len() as u64,
        REQUIRED_SYNTHESIS_FAMILIES.len() as u64
            * budget.seed_blocks
            * (INPUT_NOISE_FAMILIES.len()
                * INPUT_NOISE_AMPLITUDES.len()
                * SEQUENCE_SCALING_ARMS.len()) as u64
    );
    for family in REQUIRED_SYNTHESIS_FAMILIES {
        let run = MatchedPrimaryRun::evaluate(DataSplit::Development, *family, 0, 8).unwrap();
        let t6 = run
            .t6_outcomes()
            .iter()
            .filter(|o| o.matches_oracle)
            .count() as u64;
        let c6 = run
            .c6_outcomes()
            .iter()
            .filter(|o| o.matches_oracle)
            .count() as u64;
        let cells = evaluate_input_noise_robustness(
            DataSplit::Development,
            *family,
            0,
            8,
            input_noise_seed(),
        )
        .unwrap();
        for cell in &cells {
            let clean = if cell.arm == SEQUENCE_SCALING_ARMS[0] {
                t6
            } else {
                c6
            };
            assert_eq!(cell.clean_matches, clean);
            assert!(cell.flips >= cell.clean_matches.abs_diff(cell.noisy_matches));
        }
    }
}

#[test]
fn both_splits_validate_and_flags_hold() {
    for split in [DataSplit::Development, DataSplit::Validation] {
        let report = study(split);
        validate_input_noise_robustness_report(&report).unwrap();
        assert!(!report.protected_or_final_access);
        assert!(!report.training_executed);
        assert!(!report.scientific_claim);
        assert!(report.experimental_non_final);
    }
}

#[test]
fn tampered_reports_fail_closed() {
    let report = study(DataSplit::Development);
    let tamper = |mutate: &dyn Fn(&mut InputNoiseRobustnessReport), reason: &str| {
        let mut tampered = report.clone();
        mutate(&mut tampered);
        assert_eq!(rejection(&tampered), reason);
    };
    tamper(&|r| r.robustness_contract = "legacy", "contract_drift");
    tamper(&|r| r.population_contract = "legacy", "population_drift");
    tamper(&|r| r.noise_seed ^= 1, "seed_drift");
    tamper(&|r| r.t6_capacity = r.c6_capacity, "capacity_mismatch");
    tamper(
        &|r| {
            r.cells.pop();
        },
        "cell_count",
    );
    tamper(&|r| r.cells.swap(0, 1), "grid_drift");
    tamper(&|r| r.cells[0].amplitude = 0.5, "grid_drift");
    tamper(&|r| r.cells[0].n_cases += 1, "paired_count_drift");
    tamper(&|r| r.cells[0].flips = 9, "paired_count_drift");
    tamper(
        &|r| r.cells[0].max_abs_score_change = f64::NAN,
        "score_change_drift",
    );
    tamper(
        &|r| r.cells[0].max_abs_score_change += 1.0,
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
    tamper(&|r| r.split = DataSplit::Validation, "case_evidence_drift");
}

#[test]
fn protected_or_final_labels_and_bad_budgets_never_score() {
    for label in ["protected", "final", "holdout", ""] {
        assert_eq!(
            run_input_noise_robustness_for_label(label, StageCPreflightBudget::smoke()),
            Err(EvalError::ProtectedOrFinalSplit)
        );
    }
    assert!(matches!(
        run_input_noise_robustness(
            DataSplit::Development,
            StageCPreflightBudget {
                seed_blocks: 2,
                cases_per_block: 1,
            }
        ),
        Err(EvalError::StageCPreflightInvalid { .. })
    ));
}

#[test]
fn study_is_deterministic() {
    assert_eq!(study(DataSplit::Validation), study(DataSplit::Validation));
}

#[test]
fn position_only_noise_leaves_c6_bit_identical() {
    // C6 never reads the position scalars, so position-only noise cannot move
    // its score; T6 does read them through the transport term.
    let report = study(DataSplit::Development);
    for cell in &report.cells {
        if cell.noise != InputNoiseFamily::PositionOnly {
            continue;
        }
        if cell.arm == SEQUENCE_SCALING_ARMS[1] {
            assert_eq!(cell.max_abs_score_change.to_bits(), 0.0_f64.to_bits());
            assert_eq!(cell.flips, 0);
        } else {
            assert!(cell.max_abs_score_change > 0.0);
        }
    }
}
