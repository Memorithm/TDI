//! TDI-25 slice 41: multi-seed replication (Phase E).
//!
//! T6 and C6 on eight frozen paired seed blocks of the bounded matched
//! Development/Validation population, with paired discordance, pooled counts
//! and descriptive block tallies. No training, no protected/final access, no
//! scientific claim.
#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi25_eval::matched_reference::{
    MATCHED_POPULATION_CONTRACT, MatchedPrimaryRun,
};
use tdi_ai::experimental::tdi25_eval::{
    EvalError, MULTI_SEED_REPLICATION_CONTRACT, MultiSeedReplicationReport,
    REPLICATION_SEED_BLOCKS, REQUIRED_SYNTHESIS_FAMILIES, StageCPreflightBudget,
    run_multi_seed_replication, run_multi_seed_replication_for_label,
    validate_multi_seed_replication_report,
};
use tdi_ai::experimental::tdi25_tasks::DataSplit;

fn replicate(split: DataSplit) -> MultiSeedReplicationReport {
    run_multi_seed_replication(split, StageCPreflightBudget::smoke()).unwrap()
}

fn rejection(report: &MultiSeedReplicationReport) -> &'static str {
    match validate_multi_seed_replication_report(report) {
        Err(EvalError::MultiSeedReplicationInvalid { reason }) => reason,
        other => panic!("expected a replication rejection, got {other:?}"),
    }
}

#[test]
fn seed_blocks_are_frozen() {
    assert_eq!(
        MULTI_SEED_REPLICATION_CONTRACT,
        "tdi25-multi-seed-replication-v1"
    );
    assert_eq!(REPLICATION_SEED_BLOCKS, [0, 1, 2, 3, 4, 5, 6, 7]);
}

#[test]
fn cells_are_paired_and_match_the_matched_primary_runs() {
    let report = replicate(DataSplit::Development);
    assert_eq!(report.population_contract, MATCHED_POPULATION_CONTRACT);
    assert_eq!(
        report.cells.len(),
        REPLICATION_SEED_BLOCKS.len() * REQUIRED_SYNTHESIS_FAMILIES.len()
    );
    let cell = report.cells[REQUIRED_SYNTHESIS_FAMILIES.len()];
    let run = MatchedPrimaryRun::evaluate(DataSplit::Development, cell.family, 1, 8).unwrap();
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
    assert_eq!(
        (cell.seed_block, cell.t6_matches, cell.c6_matches),
        (1, t6, c6)
    );
    for cell in &report.cells {
        assert_eq!(
            cell.t6_matches - cell.t6_only,
            cell.c6_matches - cell.c6_only
        );
    }
}

#[test]
fn pooled_counts_and_tallies_are_consistent() {
    for split in [DataSplit::Development, DataSplit::Validation] {
        let report = replicate(split);
        assert_eq!(
            report.pooled_t6_matches,
            report.cells.iter().map(|c| c.t6_matches).sum::<u64>()
        );
        assert_eq!(
            report.blocks_c6_ahead + report.blocks_t6_ahead + report.blocks_tied,
            8
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
    tamper(&|r| r.population_contract = "legacy", "population_drift");
    tamper(
        &|r| r.seed_blocks = vec![0, 1, 2, 3, 4, 5, 6, 9],
        "block_set_drift",
    );
    tamper(
        &|r| {
            r.cells.pop();
        },
        "cell_count",
    );
    tamper(&|r| r.cells.swap(0, 1), "cell_order");
    tamper(&|r| r.cells[0].n_cases += 1, "paired_count_drift");
    tamper(
        &|r| {
            r.cells[0].t6_only += 1;
            r.cells[0].t6_matches += 1;
            r.cells[0].c6_matches += 1;
        },
        "paired_count_drift",
    );
    tamper(&|r| r.pooled_c6_only += 1, "pooled_sum_drift");
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
    tamper(&|r| r.split = DataSplit::Validation, "case_evidence_drift");
}

#[test]
fn protected_or_final_labels_and_bad_budgets_never_score() {
    for label in ["protected", "final", "holdout", ""] {
        assert_eq!(
            run_multi_seed_replication_for_label(label, StageCPreflightBudget::smoke()),
            Err(EvalError::ProtectedOrFinalSplit)
        );
    }
    assert!(matches!(
        run_multi_seed_replication(
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
fn replication_is_deterministic() {
    assert_eq!(
        replicate(DataSplit::Validation),
        replicate(DataSplit::Validation)
    );
}
