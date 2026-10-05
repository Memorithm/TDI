//! TDI-25 slice 30: Stage-C bounded preflight.
//!
//! Development/Validation smoke run over slices 21-29 machinery. No
//! protected/final execution, no training, no scientific claim.
#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi25_eval::{
    EvalError, FAILURE_RESOURCE_ACCOUNTING_CONTRACT, FAMILY_STRATIFIED_SYNTHESIS_CONTRACT,
    MAX_PREFLIGHT_CASES_PER_BLOCK, MAX_PREFLIGHT_SEED_BLOCKS, REQUIRED_SYNTHESIS_FAMILIES,
    STAGE_C_PREFLIGHT_CONTRACT, StageCPreflightBudget, StageCPreflightReport,
    run_stage_c_preflight, run_stage_c_preflight_for_label, validate_stage_c_preflight_report,
};
use tdi_ai::experimental::tdi25_matched_matrix::MATCHED_EVALUATOR_MATRIX_CONTRACT;
use tdi_ai::experimental::tdi25_tasks::DataSplit;

#[test]
fn smoke_preflight_exercises_every_family_on_both_non_final_splits() {
    for split in [DataSplit::Development, DataSplit::Validation] {
        let budget = StageCPreflightBudget::smoke();
        let report = run_stage_c_preflight(split, budget).unwrap();
        assert_eq!(report.split, split);
        assert_eq!(report.preflight_contract, STAGE_C_PREFLIGHT_CONTRACT);
        assert_eq!(report.matrix_contract, MATCHED_EVALUATOR_MATRIX_CONTRACT);
        assert_eq!(
            report.accounting.accounting_contract,
            FAILURE_RESOURCE_ACCOUNTING_CONTRACT
        );
        assert_eq!(
            report.synthesis.synthesis_contract,
            FAMILY_STRATIFIED_SYNTHESIS_CONTRACT
        );
        let per_arm = REQUIRED_SYNTHESIS_FAMILIES.len() as u64
            * MAX_PREFLIGHT_SEED_BLOCKS
            * budget.cases_per_block;
        assert_eq!(report.matched_pairs, per_arm);
        assert_eq!(report.accounting.arms[0].scored_cases, per_arm);
        assert_eq!(report.accounting.arms[1].scored_cases, per_arm);
        // G6 is not a primary arm and is never silently substituted.
        assert_eq!(report.accounting.arms[2].scored_cases, 0);
        assert!(report.accounting.failures.is_empty());
        assert!(report.accounting.primary_attempts_matched);
        assert!(report.accounting.primary_failure_free);
        assert_eq!(report.synthesis.family_effects.len(), 4);
        assert_eq!(report.synthesis.seed_block_effects.len(), 8);
        assert!(!report.protected_or_final_access);
        assert!(!report.scientific_claim);
        assert!(report.experimental_non_final);
        validate_stage_c_preflight_report(&report).unwrap();
    }
}

#[test]
fn preflight_is_deterministic() {
    let budget = StageCPreflightBudget {
        seed_blocks: 1,
        cases_per_block: 4,
    };
    let left = run_stage_c_preflight(DataSplit::Development, budget).unwrap();
    let right = run_stage_c_preflight(DataSplit::Development, budget).unwrap();
    assert_eq!(left, right);
}

#[test]
fn over_budget_or_empty_preflights_are_rejected_before_any_case_runs() {
    for (seed_blocks, cases_per_block, reason) in [
        (0, 8, "seed_block_budget"),
        (MAX_PREFLIGHT_SEED_BLOCKS + 1, 8, "seed_block_budget"),
        (1, 0, "case_budget"),
        (1, 1, "case_budget"),
        (1, MAX_PREFLIGHT_CASES_PER_BLOCK + 1, "case_budget"),
    ] {
        assert_eq!(
            run_stage_c_preflight(
                DataSplit::Validation,
                StageCPreflightBudget {
                    seed_blocks,
                    cases_per_block,
                },
            ),
            Err(EvalError::StageCPreflightInvalid { reason })
        );
    }
}

#[test]
fn protected_or_final_labels_never_start_a_preflight() {
    for label in ["protected", "final", "holdout", "confirmatory", ""] {
        assert_eq!(
            run_stage_c_preflight_for_label(label, StageCPreflightBudget::smoke()),
            Err(EvalError::ProtectedOrFinalSplit)
        );
    }
    assert!(run_stage_c_preflight_for_label("development", StageCPreflightBudget::smoke()).is_ok());
}

#[test]
fn tampered_reports_fail_closed() {
    let report =
        run_stage_c_preflight(DataSplit::Development, StageCPreflightBudget::smoke()).unwrap();
    let reject = |mutate: &dyn Fn(&mut StageCPreflightReport), reason: &'static str| {
        let mut tampered = report.clone();
        mutate(&mut tampered);
        assert_eq!(
            validate_stage_c_preflight_report(&tampered),
            Err(EvalError::StageCPreflightInvalid { reason })
        );
    };
    reject(
        &|r| r.protected_or_final_access = true,
        "protected_or_final_access",
    );
    reject(&|r| r.scientific_claim = true, "scientific_claim");
    reject(
        &|r| r.experimental_non_final = false,
        "experimental_non_final",
    );
    reject(
        &|r| r.preflight_contract = "tdi25-stage-c-preflight-v0",
        "preflight_contract",
    );
    reject(
        &|r| r.matrix_contract = "tdi25-matched-evaluator-matrix-v1",
        "matrix_contract",
    );
    reject(&|r| r.matched_pairs += 1, "matched_pairs_mismatch");
    reject(&|r| r.split = DataSplit::Validation, "split_mismatch");
    reject(
        &|r| {
            r.synthesis.seed_block_effects.pop();
        },
        "synthesis_coverage",
    );
    reject(
        &|r| r.budget.seed_blocks = MAX_PREFLIGHT_SEED_BLOCKS + 1,
        "seed_block_budget",
    );
}
