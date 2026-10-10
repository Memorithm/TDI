//! TDI-25 slice 47: reference cost study (Phase E), reduced scope.
//!
//! Deterministic source-level operation counts and logical memory for T6 and
//! C6 (C6 through the unchanged TDI-24 slice-09 accounting); the timing
//! harness refuses to measure until a qualified-environment manifest is
//! frozen (timing non qualifie). No training, no protected/final access, no
//! scientific or performance claim.
#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi24_accounting::{
    REFERENCE_ACCOUNTING_CONTRACT, ScoreArm, pair_score_accounting,
};
use tdi_ai::experimental::tdi25_eval::matched_reference::SHARED_INPUT_SCALARS;
use tdi_ai::experimental::tdi25_eval::{
    EvalError, QUALIFIED_TIMING_ENVIRONMENT, REFERENCE_COST_CONTRACT, REFERENCE_TIMING_STATUS,
    REQUIRED_SYNTHESIS_FAMILIES, ReferenceCostReport, StageCPreflightBudget,
    T6_REFERENCE_ACCOUNTING_CONTRACT, c6_pair_accounting, run_qualified_reference_timing,
    run_reference_cost, run_reference_cost_for_label, t6_pair_accounting,
    validate_reference_cost_report,
};
use tdi_ai::experimental::tdi25_tasks::DataSplit;
use tdi_ai::experimental::tdi25_torsor_chiral::ComparisonArm;

fn study(split: DataSplit) -> ReferenceCostReport {
    run_reference_cost(split, StageCPreflightBudget::smoke()).unwrap()
}

fn rejection(report: &ReferenceCostReport) -> &'static str {
    match validate_reference_cost_report(report) {
        Err(EvalError::ReferenceCostInvalid { reason }) => reason,
        other => panic!("expected a reference cost rejection, got {other:?}"),
    }
}

#[test]
fn contracts_and_timing_status_are_declared() {
    assert_eq!(REFERENCE_COST_CONTRACT, "tdi25-reference-cost-v1");
    assert_eq!(
        T6_REFERENCE_ACCOUNTING_CONTRACT,
        "tdi25-t6-reference-accounting-v1"
    );
    assert_eq!(REFERENCE_TIMING_STATUS, "timing_non_qualifie");
    assert!(QUALIFIED_TIMING_ENVIRONMENT.is_none());
}

#[test]
fn t6_counts_follow_the_factorized_pairing_decomposition() {
    // cross (6M + 3A) x2, vector add (3A) x2, dot (3M + 2A) x2, final add 1A.
    let cross = (6, 3);
    let add = (0, 3);
    let dot = (3, 2);
    let mults = 2 * cross.0 + 2 * add.0 + 2 * dot.0;
    let adds = 2 * cross.1 + 2 * add.1 + 2 * dot.1 + 1;
    let t6 = t6_pair_accounting();
    assert_eq!(t6.arm, ComparisonArm::T6);
    assert_eq!((t6.multiplications, t6.additions), (mults, adds));
    assert_eq!((t6.multiplications, t6.additions), (18, 17));
    // query position (3), resultant dual (3), origin moment (3), pairing (1).
    assert_eq!(t6.validity_predicates, 10);
    assert_eq!(t6.read_scalars, SHARED_INPUT_SCALARS as u64);
    assert_eq!(t6.read_bytes, 144);
}

#[test]
fn c6_counts_reuse_the_tdi24_slice_09_accounting() {
    let upstream = pair_score_accounting(ScoreArm::C6);
    let c6 = c6_pair_accounting();
    assert_eq!(c6.arm, ComparisonArm::C6);
    assert_eq!(c6.multiplications, upstream.multiplications as u64);
    assert_eq!(c6.additions, upstream.additions as u64);
    assert_eq!(c6.validity_predicates, upstream.validity_predicates as u64);
    assert_eq!(c6.read_bytes, upstream.carrier_bytes as u64);
    assert_eq!(c6.accounting_contract, REFERENCE_ACCOUNTING_CONTRACT);
    assert_eq!(
        (c6.multiplications, c6.additions, c6.validity_predicates),
        (21, 26, 41)
    );
    assert_eq!((c6.read_scalars, c6.read_bytes), (12, 96));
}

#[test]
fn smoke_report_totals_scale_with_the_scored_population() {
    for split in [DataSplit::Development, DataSplit::Validation] {
        let report = study(split);
        validate_reference_cost_report(&report).unwrap();
        let budget = StageCPreflightBudget::smoke();
        let cases =
            REQUIRED_SYNTHESIS_FAMILIES.len() as u64 * budget.seed_blocks * budget.cases_per_block;
        assert_eq!(report.cases, cases);
        assert_eq!(report.shared_input_bytes_per_case, 144);
        let [t6, c6] = report.totals;
        assert_eq!((t6.arm, c6.arm), (ComparisonArm::T6, ComparisonArm::C6));
        assert_eq!(t6.multiplications, 18 * cases);
        assert_eq!(t6.additions, 17 * cases);
        assert_eq!(c6.multiplications, 21 * cases);
        assert_eq!(c6.additions, 26 * cases);
        assert_eq!(t6.read_bytes, 144 * cases);
        assert_eq!(c6.read_bytes, 96 * cases);
        assert_eq!(report.timing_status, "timing_non_qualifie");
        assert!(!report.timing_measured);
        assert!(!report.performance_claim);
        assert!(!report.protected_or_final_access);
        assert!(!report.training_executed);
        assert!(!report.scientific_claim);
        assert!(report.experimental_non_final);
    }
}

#[test]
fn timing_harness_refuses_without_a_qualified_environment() {
    for split in [DataSplit::Development, DataSplit::Validation] {
        assert_eq!(
            run_qualified_reference_timing(split, StageCPreflightBudget::smoke()),
            Err(EvalError::ReferenceCostInvalid {
                reason: "timing_environment_not_qualified"
            })
        );
    }
}

#[test]
fn tampered_reports_fail_closed() {
    let report = study(DataSplit::Validation);

    let mut drifted = report.clone();
    drifted.cost_contract = "tdi25-reference-cost-v0";
    assert_eq!(rejection(&drifted), "contract_drift");

    let mut drifted = report.clone();
    drifted.population_contract = "other";
    assert_eq!(rejection(&drifted), "population_drift");

    let mut drifted = report.clone();
    drifted.c6.multiplications -= 1;
    assert_eq!(rejection(&drifted), "accounting_drift");

    let mut drifted = report.clone();
    drifted.t6.read_bytes = drifted.c6.read_bytes;
    assert_eq!(rejection(&drifted), "accounting_drift");

    let mut drifted = report.clone();
    drifted.cases -= 1;
    assert_eq!(rejection(&drifted), "population_size_drift");

    let mut drifted = report.clone();
    drifted.totals[1].additions += 1;
    assert_eq!(rejection(&drifted), "totals_drift");

    let mut drifted = report.clone();
    drifted.timing_measured = true;
    assert_eq!(rejection(&drifted), "timing_status_drift");

    let mut drifted = report.clone();
    drifted.timing_status = "qualified";
    assert_eq!(rejection(&drifted), "timing_status_drift");

    for (flag, reason) in [
        (0, "protected_or_final_access"),
        (1, "training_executed"),
        (2, "scientific_claim"),
        (3, "performance_claim"),
        (4, "experimental_non_final"),
    ] {
        let mut drifted = report.clone();
        match flag {
            0 => drifted.protected_or_final_access = true,
            1 => drifted.training_executed = true,
            2 => drifted.scientific_claim = true,
            3 => drifted.performance_claim = true,
            _ => drifted.experimental_non_final = false,
        }
        assert_eq!(rejection(&drifted), reason);
    }
}

#[test]
fn protected_or_final_labels_never_generate_a_case() {
    for label in ["protected", "final", "holdout", ""] {
        assert!(matches!(
            run_reference_cost_for_label(label, StageCPreflightBudget::smoke()),
            Err(EvalError::ProtectedOrFinalSplit)
        ));
    }
    assert_eq!(study(DataSplit::Development), study(DataSplit::Development));
}
