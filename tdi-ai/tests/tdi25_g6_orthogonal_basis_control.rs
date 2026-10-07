//! TDI-25 slice 36: G6 orthogonal-basis control.
//!
//! Phase-D control on the bounded matched Development/Validation population:
//! query and key scalars are rotated by the four declared TDI-24 slice-36
//! orthogonal probes. The generic G6 score must be basis-invariant; the C6
//! score on the rotated pair is recorded for contrast. No protected/final
//! execution, no training, no scientific claim.
#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi24_eval::{
    LEARNED_BASIS_PROBE_COUNT, LEARNED_BASIS_PROTOTYPE_CONTRACT, learned_basis_probes,
};
use tdi_ai::experimental::tdi25_eval::matched_reference::{
    MATCHED_POPULATION_CONTRACT, MatchedPrimaryRun,
};
use tdi_ai::experimental::tdi25_eval::{
    EvalError, G6_ORTHOGONAL_BASIS_CONTROL_CONTRACT, G6OrthogonalBasisControlReport,
    ParameterReadoutCapacity, REQUIRED_SYNTHESIS_FAMILIES, StageCPreflightBudget,
    g6_rotation_invariant, rotated_generic_and_chiral_scores, run_g6_orthogonal_basis_control,
    run_g6_orthogonal_basis_control_for_label, validate_g6_orthogonal_basis_control_report,
};
use tdi_ai::experimental::tdi25_tasks::DataSplit;
use tdi_ai::experimental::tdi25_torsor_chiral::{GENERIC6_CONTRACT, TaskFamily};

fn smoke(split: DataSplit) -> G6OrthogonalBasisControlReport {
    run_g6_orthogonal_basis_control(split, StageCPreflightBudget::smoke()).unwrap()
}

fn rejection(report: &G6OrthogonalBasisControlReport) -> &'static str {
    match validate_g6_orthogonal_basis_control_report(report) {
        Err(EvalError::G6OrthogonalBasisControlInvalid { reason }) => reason,
        other => panic!("expected a G6 orthogonal-basis rejection, got {other:?}"),
    }
}

#[test]
fn control_consumes_the_upstream_probe_set_unchanged() {
    assert_eq!(
        G6_ORTHOGONAL_BASIS_CONTROL_CONTRACT,
        "tdi25-g6-orthogonal-basis-control-v1"
    );
    let report = smoke(DataSplit::Development);
    assert_eq!(report.probe_contract, LEARNED_BASIS_PROTOTYPE_CONTRACT);
    assert_eq!(report.probes, learned_basis_probes().to_vec());
    assert_eq!(report.generic_contract, GENERIC6_CONTRACT);
    assert_eq!(report.population_contract, MATCHED_POPULATION_CONTRACT);
}

#[test]
fn smoke_control_covers_every_probe_and_family_on_both_non_final_splits() {
    for split in [DataSplit::Development, DataSplit::Validation] {
        let budget = StageCPreflightBudget::smoke();
        let report = smoke(split);
        assert_eq!(
            report.cases.len() as u64,
            budget.cases_per_arm() * LEARNED_BASIS_PROBE_COUNT as u64
        );
        assert_eq!(
            report.summaries.len(),
            LEARNED_BASIS_PROBE_COUNT * REQUIRED_SYNTHESIS_FAMILIES.len()
        );
        assert_eq!(
            report.reference_capacity,
            ParameterReadoutCapacity::reference_g6()
        );
        assert_eq!(report.rotated_capacity, report.reference_capacity);
        assert!(!report.protected_or_final_access);
        assert!(!report.training_executed);
        assert!(!report.scientific_claim);
        assert!(report.experimental_non_final);
        for case in &report.cases {
            assert!(g6_rotation_invariant(
                case.g6_reference_score,
                case.g6_rotated_score
            ));
        }
        for summary in &report.summaries {
            assert_eq!(summary.g6_reference_matches, summary.g6_rotated_matches);
            if summary.probe_index <= 1 {
                // Identity and gauge probes leave C6 unchanged within tolerance.
                assert_eq!(summary.c6_changed, 0);
            }
        }
        // C6 is not basis-invariant under the non-gauge probes.
        assert!(
            report
                .summaries
                .iter()
                .any(|s| s.probe_index >= 2 && s.c6_changed > 0)
        );
        validate_g6_orthogonal_basis_control_report(&report).unwrap();
    }
}

#[test]
fn reference_side_reproduces_the_matched_c6_primary_exactly() {
    let budget = StageCPreflightBudget::smoke();
    let report = smoke(DataSplit::Validation);
    let mut position = 0usize;
    for family in REQUIRED_SYNTHESIS_FAMILIES {
        for seed_block in 0..budget.seed_blocks {
            let run = MatchedPrimaryRun::evaluate(
                DataSplit::Validation,
                *family,
                seed_block,
                budget.cases_per_block,
            )
            .unwrap();
            for (index, input) in run.inputs().iter().enumerate() {
                for probe in learned_basis_probes() {
                    let case = &report.cases[position];
                    assert_eq!(
                        case.c6_reference_score.to_bits(),
                        run.c6_scores()[index].to_bits()
                    );
                    let (g6, c6) = rotated_generic_and_chiral_scores(input, &probe).unwrap();
                    assert_eq!(case.g6_rotated_score.to_bits(), g6.to_bits());
                    assert_eq!(case.c6_rotated_score.to_bits(), c6.to_bits());
                    position += 1;
                }
            }
        }
    }
}

#[test]
fn tampered_reports_fail_closed() {
    let report = smoke(DataSplit::Development);
    let tamper = |mutate: &dyn Fn(&mut G6OrthogonalBasisControlReport), reason: &str| {
        let mut tampered = report.clone();
        mutate(&mut tampered);
        assert_eq!(rejection(&tampered), reason);
    };
    tamper(&|r| r.control_contract = "legacy", "contract_drift");
    tamper(&|r| r.population_contract = "legacy", "population_drift");
    tamper(&|r| r.generic_contract = "legacy", "generic_contract_drift");
    tamper(&|r| r.probe_contract = "legacy", "probe_contract_drift");
    tamper(&|r| r.probes[3].angles[0] += 0.01, "probe_set_drift");
    tamper(
        &|r| r.rotated_capacity = ParameterReadoutCapacity::reference_c6(),
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
        &|r| r.cases[0].c6_rotated_score += 1.0,
        "identity_probe_drift",
    );
    tamper(
        &|r| r.cases[2].g6_rotated_score += 1.0,
        "g6_rotation_invariance_drift",
    );
    tamper(
        &|r| r.summaries[0].g6_rotated_matches += 1,
        "family_summary",
    );
    tamper(
        &|r| r.cases[2].c6_rotated_score += 1.0,
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
            run_g6_orthogonal_basis_control_for_label(label, StageCPreflightBudget::smoke()),
            Err(EvalError::ProtectedOrFinalSplit)
        );
    }
    for budget in [
        StageCPreflightBudget {
            seed_blocks: 0,
            cases_per_block: 8,
        },
        StageCPreflightBudget {
            seed_blocks: 1,
            cases_per_block: 17,
        },
    ] {
        assert!(matches!(
            run_g6_orthogonal_basis_control(DataSplit::Development, budget),
            Err(EvalError::StageCPreflightInvalid { .. })
        ));
    }
}

#[test]
fn control_is_deterministic() {
    let budget = StageCPreflightBudget {
        seed_blocks: 1,
        cases_per_block: 4,
    };
    assert_eq!(
        run_g6_orthogonal_basis_control(DataSplit::Validation, budget).unwrap(),
        run_g6_orthogonal_basis_control(DataSplit::Validation, budget).unwrap()
    );
    let _ = TaskFamily::Neutral;
}
