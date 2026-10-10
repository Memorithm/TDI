//! TDI-25 slice 44: mirror/parity stress suite (Phase E).
//!
//! Declared chiral-relevant transformations (simultaneous mirror `(Mq, Mk)`,
//! complex structure `(Jq, Jk)`, key-only mirror `(q, Mk)`) applied
//! identically to T6 and C6 on the bounded matched Development/Validation
//! population, independently of any model output. No training, no
//! protected/final access, no scientific claim.
#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi24_chiral::Chiral6;
use tdi_ai::experimental::tdi25_eval::matched_reference::{
    MATCHED_POPULATION_CONTRACT, MatchedPrimaryRun,
};
use tdi_ai::experimental::tdi25_eval::{
    EvalError, MIRROR_PARITY_STRESS_CONTRACT, MIRROR_STRESS_TRANSFORMS, MirrorParityStressReport,
    MirrorStressTransform, REQUIRED_SYNTHESIS_FAMILIES, SEQUENCE_SCALING_ARMS,
    StageCPreflightBudget, mirror_stress_matched_input, run_mirror_parity_stress,
    run_mirror_parity_stress_for_label, validate_mirror_parity_stress_report,
};
use tdi_ai::experimental::tdi25_tasks::DataSplit;
use tdi_ai::experimental::tdi25_torsor_chiral::{ComparisonArm, TaskFamily};

fn study(split: DataSplit) -> MirrorParityStressReport {
    run_mirror_parity_stress(split, StageCPreflightBudget::smoke()).unwrap()
}

fn rejection(report: &MirrorParityStressReport) -> &'static str {
    match validate_mirror_parity_stress_report(report) {
        Err(EvalError::MirrorParityStressInvalid { reason }) => reason,
        other => panic!("expected a mirror/parity stress rejection, got {other:?}"),
    }
}

#[test]
fn grid_is_declared() {
    assert_eq!(
        MIRROR_PARITY_STRESS_CONTRACT,
        "tdi25-mirror-parity-stress-v1"
    );
    assert_eq!(
        MIRROR_STRESS_TRANSFORMS,
        [
            MirrorStressTransform::SimultaneousMirror,
            MirrorStressTransform::ComplexStructure,
            MirrorStressTransform::KeyMirror
        ]
    );
    assert_eq!(
        SEQUENCE_SCALING_ARMS,
        [ComparisonArm::T6, ComparisonArm::C6]
    );
}

#[test]
fn transformations_are_the_upstream_involutions_and_keep_positions() {
    let run = MatchedPrimaryRun::evaluate(DataSplit::Validation, TaskFamily::Mixed, 0, 4).unwrap();
    for input in run.inputs() {
        let q = Chiral6::from_array(input.query()).unwrap();
        let k = Chiral6::from_array(input.key()).unwrap();
        for transform in MIRROR_STRESS_TRANSFORMS {
            let out = mirror_stress_matched_input(input, transform).unwrap();
            let (eq, ek) = match transform {
                MirrorStressTransform::SimultaneousMirror => (q.mirror(), k.mirror()),
                MirrorStressTransform::ComplexStructure => {
                    (q.complex_structure(), k.complex_structure())
                }
                MirrorStressTransform::KeyMirror => (q, k.mirror()),
            };
            assert_eq!(out.query(), eq.as_array());
            assert_eq!(out.key(), ek.as_array());
            assert_eq!(out.key_position(), input.key_position());
            assert_eq!(out.query_position(), input.query_position());
            let oq = Chiral6::from_array(out.query()).unwrap();
            let ok = Chiral6::from_array(out.key()).unwrap();
            match transform {
                MirrorStressTransform::SimultaneousMirror => {
                    assert_eq!(oq.dot(ok).unwrap(), q.dot(k).unwrap());
                    assert_eq!(
                        oq.chiral_pairing(ok).unwrap(),
                        -q.chiral_pairing(k).unwrap()
                    );
                }
                MirrorStressTransform::ComplexStructure => {
                    assert_eq!(oq.dot(ok).unwrap(), q.dot(k).unwrap());
                    assert_eq!(oq.chiral_pairing(ok).unwrap(), q.chiral_pairing(k).unwrap());
                }
                MirrorStressTransform::KeyMirror => {}
            }
        }
    }
}

#[test]
fn smoke_suite_reports_every_cell_on_both_splits() {
    for split in [DataSplit::Development, DataSplit::Validation] {
        let report = study(split);
        validate_mirror_parity_stress_report(&report).unwrap();
        assert_eq!(report.population_contract, MATCHED_POPULATION_CONTRACT);
        let budget = StageCPreflightBudget::smoke();
        assert_eq!(
            report.cells.len() as u64,
            REQUIRED_SYNTHESIS_FAMILIES.len() as u64 * budget.seed_blocks * 3 * 2
        );
        for cell in &report.cells {
            assert_eq!(cell.n_cases, budget.cases_per_block);
            if cell.arm == ComparisonArm::C6 {
                if cell.transform == MirrorStressTransform::ComplexStructure {
                    assert_eq!(cell.max_abs_score_change, 0.0);
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
    let c6_complex = report
        .cells
        .iter()
        .position(|cell| {
            cell.arm == ComparisonArm::C6
                && cell.transform == MirrorStressTransform::ComplexStructure
        })
        .unwrap();
    let c6_chiral = report
        .cells
        .iter()
        .position(|cell| {
            cell.arm == ComparisonArm::C6
                && cell.family == TaskFamily::ChiralFavorable
                && cell.transform == MirrorStressTransform::KeyMirror
        })
        .unwrap();

    let mut drifted = report.clone();
    drifted.stress_contract = "tdi25-mirror-parity-stress-v0";
    assert_eq!(rejection(&drifted), "contract_drift");

    let mut drifted = report.clone();
    drifted.population_contract = "other";
    assert_eq!(rejection(&drifted), "population_drift");

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
    drifted.cells[0].max_abs_score_change = f64::NAN;
    assert_eq!(rejection(&drifted), "score_change_drift");

    let mut drifted = report.clone();
    drifted.cells[c6_complex].max_abs_score_change = 1e-9;
    assert_eq!(rejection(&drifted), "c6_complex_structure_drift");

    let mut drifted = report.clone();
    let cell = &mut drifted.cells[c6_chiral];
    cell.stressed_matches -= 1;
    cell.flips += 1;
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
            run_mirror_parity_stress_for_label(label, StageCPreflightBudget::smoke()),
            Err(EvalError::ProtectedOrFinalSplit)
        ));
    }
    assert!(
        run_mirror_parity_stress_for_label("validation", StageCPreflightBudget::smoke()).is_ok()
    );
}

#[test]
fn suite_is_deterministic() {
    assert_eq!(study(DataSplit::Development), study(DataSplit::Development));
}
