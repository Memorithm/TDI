//! Slice 29 production-API regressions: typed failure/resource accounting by
//! arm. The library is compiled without cfg(test); no record mutation, oracle
//! access or synthetic outcome constructor is used. Development/Validation
//! only; no protected/final surface exists.
#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi25_eval::{
    ACCOUNTED_ARMS, C6EvaluatorRun, EvalError, EvaluatorConfig,
    FAILURE_RESOURCE_ACCOUNTING_CONTRACT, FailureClass, FailureResourceLedger,
    MAX_READOUT_SCALARS_PER_CASE, REQUIRED_SYNTHESIS_FAMILIES, T6EvaluatorRun,
    parse_non_final_split, require_complete_primary_accounting, validate_failure_resource_report,
};
use tdi_ai::experimental::tdi25_tasks::{
    DataSplit, mixed_geometry_pair_in_split, seal_mixed_geometry,
};
use tdi_ai::experimental::tdi25_torsor_chiral::{ComparisonArm, TaskFamily};

#[test]
fn legacy_mixed_runs_are_accounted_by_arm_on_both_splits() {
    for split in [DataSplit::Development, DataSplit::Validation] {
        let mut ledger = FailureResourceLedger::open(split).unwrap();
        assert_eq!(
            ledger.accounting_contract(),
            FAILURE_RESOURCE_ACCOUNTING_CONTRACT
        );
        let mut t6 = T6EvaluatorRun::open(EvaluatorConfig::t6(split)).unwrap();
        let mut c6 = C6EvaluatorRun::open(EvaluatorConfig::c6(split)).unwrap();
        for seed in 0..4 {
            let pair = mixed_geometry_pair_in_split(seed, split).unwrap();
            for case in [
                seal_mixed_geometry(pair.base, pair.base_oracle),
                seal_mixed_geometry(pair.transformed, pair.transformed_oracle),
            ] {
                assert!(ledger.account_t6_mixed(&mut t6, &case).unwrap().is_some());
                assert!(ledger.account_c6_mixed(&mut c6, &case).unwrap().is_some());
            }
        }
        let report = ledger.report().unwrap();
        let arms: Vec<_> = report.arms.iter().map(|account| account.arm).collect();
        assert_eq!(arms, ACCOUNTED_ARMS);
        for arm in [ComparisonArm::T6, ComparisonArm::C6] {
            let account = ledger.account(arm);
            assert_eq!(account.scored_cases, 8);
            assert_eq!(
                account.readout_scalars_used,
                8 * MAX_READOUT_SCALARS_PER_CASE
            );
            assert_eq!(account.retained_failures(), 0);
            assert_eq!(account.attempts_by_family, [0, 0, 8, 0]);
        }
        assert!(report.primary_attempts_matched);
        assert!(report.primary_failure_free);
        require_complete_primary_accounting(&report).unwrap();
    }
}

#[test]
fn cross_split_cases_are_retained_as_task_failures_not_dropped() {
    let mut ledger = FailureResourceLedger::open(DataSplit::Development).unwrap();
    let mut t6 = T6EvaluatorRun::open(EvaluatorConfig::t6(DataSplit::Development)).unwrap();
    let mut c6 = C6EvaluatorRun::open(EvaluatorConfig::c6(DataSplit::Development)).unwrap();
    let development = mixed_geometry_pair_in_split(1, DataSplit::Development).unwrap();
    let validation = mixed_geometry_pair_in_split(1, DataSplit::Validation).unwrap();
    let good = seal_mixed_geometry(development.base, development.base_oracle);
    let foreign = seal_mixed_geometry(validation.base, validation.base_oracle);

    assert!(ledger.account_t6_mixed(&mut t6, &good).unwrap().is_some());
    assert!(ledger.account_c6_mixed(&mut c6, &good).unwrap().is_some());
    // Only C6 sees the foreign-split case: the failure is retained on C6.
    assert!(
        ledger
            .account_c6_mixed(&mut c6, &foreign)
            .unwrap()
            .is_none()
    );

    let report = ledger.report().unwrap();
    assert_eq!(report.failures.len(), 1);
    let failure = &report.failures[0];
    assert_eq!(failure.arm, ComparisonArm::C6);
    assert_eq!(failure.class, FailureClass::Task);
    assert_eq!(failure.family, TaskFamily::Mixed);
    assert_eq!(failure.split, DataSplit::Development);
    assert_eq!(failure.message_code, "split_mismatch");
    assert_eq!(ledger.account(ComparisonArm::C6).task_failures, 1);
    assert_eq!(ledger.account(ComparisonArm::T6).retained_failures(), 0);
    assert!(!report.primary_attempts_matched);
    assert!(!report.primary_failure_free);
    validate_failure_resource_report(&report).unwrap();
    assert!(require_complete_primary_accounting(&report).is_err());
}

#[test]
fn matched_reference_blocks_are_accounted_for_every_required_family() {
    let mut ledger = FailureResourceLedger::open(DataSplit::Development).unwrap();
    for family in REQUIRED_SYNTHESIS_FAMILIES {
        for block in 0..2 {
            let run = ledger
                .account_matched_primary_block(*family, block, 64)
                .unwrap()
                .unwrap();
            assert_eq!(run.c6_outcomes().len(), 64);
        }
    }
    let report = ledger.report().unwrap();
    assert_eq!(report.arms[0].scored_cases, 512);
    assert_eq!(report.arms[1].scored_cases, 512);
    assert_eq!(report.arms[0].attempts_by_family, [128; 4]);
    require_complete_primary_accounting(&report).unwrap();

    // Oversized block: retained atomically on both primaries as a budget rejection.
    assert!(
        ledger
            .account_matched_primary_block(TaskFamily::Neutral, 0, 65)
            .unwrap()
            .is_none()
    );
    let report = ledger.report().unwrap();
    for arm in [ComparisonArm::T6, ComparisonArm::C6] {
        assert_eq!(ledger.account(arm).invalid_failures, 1);
    }
    assert!(report.primary_attempts_matched);
    assert_eq!(
        require_complete_primary_accounting(&report),
        Err(EvalError::FailureResourceAccountingInvalid {
            reason: "primary_failures_retained",
        })
    );
}

#[test]
fn protected_or_final_labels_never_open_a_ledger() {
    for label in ["protected", "final", "holdout", ""] {
        assert_eq!(
            parse_non_final_split(label),
            Err(EvalError::ProtectedOrFinalSplit)
        );
    }
    assert!(FailureResourceLedger::open(parse_non_final_split("validation").unwrap()).is_ok());
}
