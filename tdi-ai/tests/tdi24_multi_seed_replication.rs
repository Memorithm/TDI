#![cfg(feature = "experimental")]

//! TDI-24 slice 41: multi-seed replication (Phase E).
//!
//! The Stage-C preflight on four frozen, disjoint seed blocks with per-block
//! paired summaries, pooled counts and descriptive sign tallies. No training,
//! no protected/final access, no scientific claim.

use tdi_ai::experimental::tdi24_eval::{
    EvalError, MAX_PREFLIGHT_PAIRS_PER_FAMILY, MULTI_SEED_BLOCK_STRIDE, MULTI_SEED_BLOCKS,
    MULTI_SEED_REPLICATION_CONTRACT, MultiSeedReplicationReport, StageCPreflightBudget,
    run_multi_seed_replication, run_multi_seed_replication_for_label, run_stage_c_preflight,
    validate_multi_seed_replication_report,
};
use tdi_ai::experimental::tdi24_tasks::DataSplit;

fn replicate(split: DataSplit) -> MultiSeedReplicationReport {
    run_multi_seed_replication(split, StageCPreflightBudget::bounded(4, 0)).unwrap()
}

fn rejection(report: &MultiSeedReplicationReport) -> &'static str {
    match validate_multi_seed_replication_report(report) {
        Err(EvalError::MultiSeedReplicationInvalid { reason }) => reason,
        other => panic!("expected a replication rejection, got {other:?}"),
    }
}

#[test]
fn seed_blocks_are_frozen_and_disjoint() {
    assert_eq!(
        MULTI_SEED_REPLICATION_CONTRACT,
        "tdi24-multi-seed-replication-v1"
    );
    assert_eq!(MULTI_SEED_BLOCKS, [0, 1, 2, 3]);
    assert_eq!(MULTI_SEED_BLOCK_STRIDE, MAX_PREFLIGHT_PAIRS_PER_FAMILY);
    let report = replicate(DataSplit::Development);
    for (entry, block) in report.blocks.iter().zip(MULTI_SEED_BLOCKS) {
        assert_eq!(entry.first_pair_id, block * MULTI_SEED_BLOCK_STRIDE);
    }
}

#[test]
fn first_block_reproduces_the_stage_c_preflight() {
    let budget = StageCPreflightBudget::bounded(4, 0);
    let report = replicate(DataSplit::Validation);
    let preflight = run_stage_c_preflight(DataSplit::Validation, budget).unwrap();
    assert_eq!(report.blocks[0].cases_per_arm, preflight.cases_per_arm);
    assert_eq!(report.blocks[0].paired_summary, preflight.paired_summary);
}

#[test]
fn pooled_counts_and_tallies_are_consistent() {
    for split in [DataSplit::Development, DataSplit::Validation] {
        let report = replicate(split);
        assert_eq!(report.blocks.len(), 4);
        assert_eq!(
            report.pooled_cases_per_arm,
            report.blocks.iter().map(|b| b.cases_per_arm).sum::<u64>()
        );
        assert_eq!(
            report.blocks_c6_ahead + report.blocks_v6_ahead + report.blocks_tied,
            4
        );
        assert!(!report.protected_or_final_access);
        assert!(!report.training_executed);
        assert!(!report.scientific_claim);
        assert!(report.experimental_non_final);
    }
}

#[test]
fn tampered_reports_fail_closed() {
    let report = replicate(DataSplit::Development);
    let tamper = |mutate: &dyn Fn(&mut MultiSeedReplicationReport), reason: &str| {
        let mut tampered = report.clone();
        mutate(&mut tampered);
        assert_eq!(rejection(&tampered), reason);
    };
    tamper(&|r| r.replication_contract = "legacy", "contract_drift");
    tamper(
        &|r| r.preflight_contract = "legacy",
        "preflight_contract_drift",
    );
    tamper(
        &|r| r.budget = StageCPreflightBudget::bounded(4, 1),
        "frozen_offset_drift",
    );
    tamper(
        &|r| {
            r.blocks.pop();
        },
        "block_set_drift",
    );
    tamper(&|r| r.blocks.swap(0, 1), "block_set_drift");
    tamper(&|r| r.blocks[1].first_pair_id += 1, "block_set_drift");
    tamper(&|r| r.blocks[0].cases_per_arm += 1, "block_count_drift");
    tamper(&|r| r.pooled_v6_matches += 1, "pooled_sum_drift");
    tamper(&|r| r.blocks_tied += 1, "sign_tally_drift");
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
    // A block summary bound to another split is caught before regeneration.
    tamper(&|r| r.split = DataSplit::Validation, "block_count_drift");
    tamper(
        &|r| {
            r.blocks[2].v6_matches -= 1;
            r.pooled_v6_matches -= 1;
        },
        "case_evidence_drift",
    );
}

#[test]
fn protected_or_final_labels_offsets_and_bad_budgets_never_score() {
    for label in ["protected", "final", "holdout", ""] {
        assert_eq!(
            run_multi_seed_replication_for_label(label, StageCPreflightBudget::bounded(1, 0)),
            Err(EvalError::ProtectedOrFinalSplit)
        );
    }
    assert_eq!(
        run_multi_seed_replication(DataSplit::Development, StageCPreflightBudget::bounded(1, 5)),
        Err(EvalError::MultiSeedReplicationInvalid {
            reason: "frozen_offset_drift"
        })
    );
    assert!(
        run_multi_seed_replication(DataSplit::Development, StageCPreflightBudget::bounded(0, 0))
            .is_err()
    );
}

#[test]
fn replication_is_deterministic() {
    assert_eq!(
        replicate(DataSplit::Validation),
        replicate(DataSplit::Validation)
    );
}
