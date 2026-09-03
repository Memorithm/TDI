//! Bounded TDI-7.4 source-identifiability audit.
//!
//! This executable checks whether the static diagnostic block S and the early
//! recovery block R span distinct centered linear source spaces on non-holdout
//! populations. It does not compute a confirmatory H-AI-3 verdict.

use tdi_bench::attention_v7::TaskKind;
use tdi_bench::predictive_v7::{
    PredictiveError, TDI71_INTERVENTION_AMPLITUDE, TDI71_TARGET_DEPTH, attention_population,
};
use tdi_bench::subspace_v7::{SubspaceError, TwoBlockSubspaceAudit, audit_two_blocks};

const TRAIN_START: u64 = 7_100_000_000;
const TRAIN_COUNT: usize = 96;
const DEVELOPMENT_START: u64 = 7_100_010_000;
const DEVELOPMENT_COUNT: usize = 64;
const VALIDATION_START: u64 = 7_100_020_000;
const VALIDATION_COUNT: usize = 64;
const RECOVERY_DEPTHS: usize = 4;
const TDI74_FINAL_START: u64 = 7_100_050_000;
const TDI74_FINAL_END: u64 = 7_100_059_999;

type FeatureRows = Vec<Vec<f64>>;
type SourceBlocks = (FeatureRows, FeatureRows);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BoundedSplit {
    Training,
    Development,
    Validation,
}

impl BoundedSplit {
    const fn start(self) -> u64 {
        match self {
            Self::Training => TRAIN_START,
            Self::Development => DEVELOPMENT_START,
            Self::Validation => VALIDATION_START,
        }
    }

    const fn count(self) -> usize {
        match self {
            Self::Training => TRAIN_COUNT,
            Self::Development => DEVELOPMENT_COUNT,
            Self::Validation => VALIDATION_COUNT,
        }
    }

    const fn id(self) -> &'static str {
        match self {
            Self::Training => "training",
            Self::Development => "development",
            Self::Validation => "validation",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AuditError {
    SeedOverflow,
    FinalRangeOverlap,
    PredictiveFailure(PredictiveError),
    InvalidRecordLayout,
    SubspaceFailure(SubspaceError),
}

fn bounded_end(start: u64, count: usize) -> Result<u64, AuditError> {
    let count = u64::try_from(count).map_err(|_| AuditError::SeedOverflow)?;
    start
        .checked_add(count.saturating_sub(1))
        .ok_or(AuditError::SeedOverflow)
}

fn ranges_overlap(left_start: u64, left_end: u64, right_start: u64, right_end: u64) -> bool {
    left_start <= right_end && right_start <= left_end
}

fn validate_split(split: BoundedSplit) -> Result<(), AuditError> {
    let end = bounded_end(split.start(), split.count())?;
    if ranges_overlap(split.start(), end, TDI74_FINAL_START, TDI74_FINAL_END) {
        return Err(AuditError::FinalRangeOverlap);
    }
    Ok(())
}

fn source_blocks(split: BoundedSplit, task: TaskKind) -> Result<SourceBlocks, AuditError> {
    validate_split(split)?;
    let records = attention_population(
        split.start(),
        split.count(),
        task,
        RECOVERY_DEPTHS,
        TDI71_TARGET_DEPTH,
        TDI71_INTERVENTION_AMPLITUDE,
    )
    .map_err(AuditError::PredictiveFailure)?;
    let mut static_rows = Vec::with_capacity(records.len());
    let mut recovery_rows = Vec::with_capacity(records.len());
    for record in records {
        if record.baseline().len() != 10 || record.evidence().len() != RECOVERY_DEPTHS {
            return Err(AuditError::InvalidRecordLayout);
        }
        static_rows.push(record.baseline()[3..9].to_vec());
        recovery_rows.push(record.evidence().to_vec());
    }
    Ok((static_rows, recovery_rows))
}

fn pooled_source_blocks(task: TaskKind) -> Result<SourceBlocks, AuditError> {
    let total_records = (TRAIN_COUNT + DEVELOPMENT_COUNT + VALIDATION_COUNT) * 2;
    let mut static_rows = Vec::with_capacity(total_records);
    let mut recovery_rows = Vec::with_capacity(total_records);
    for split in [
        BoundedSplit::Training,
        BoundedSplit::Development,
        BoundedSplit::Validation,
    ] {
        let (mut split_static, mut split_recovery) = source_blocks(split, task)?;
        static_rows.append(&mut split_static);
        recovery_rows.append(&mut split_recovery);
    }
    Ok((static_rows, recovery_rows))
}

fn audit_cell(split: BoundedSplit, task: TaskKind) -> Result<TwoBlockSubspaceAudit, AuditError> {
    let (static_rows, recovery_rows) = source_blocks(split, task)?;
    audit_two_blocks(&static_rows, &recovery_rows).map_err(AuditError::SubspaceFailure)
}

fn audit_pooled(task: TaskKind) -> Result<TwoBlockSubspaceAudit, AuditError> {
    let (static_rows, recovery_rows) = pooled_source_blocks(task)?;
    audit_two_blocks(&static_rows, &recovery_rows).map_err(AuditError::SubspaceFailure)
}

fn canonical_report(split_id: &str, task: TaskKind, audit: TwoBlockSubspaceAudit) -> String {
    format!(
        concat!(
            "tdi74-identifiability-v1;split={};task={};left=static;right=recovery;{};",
            "affine_linear_predictor_spaces_equivalent={}"
        ),
        split_id,
        task.id(),
        audit.canonical_report(),
        audit.equivalent_within_tolerance(),
    )
}

fn evaluate_all() -> Result<Vec<String>, AuditError> {
    let mut output = Vec::with_capacity(8);
    for split in [
        BoundedSplit::Training,
        BoundedSplit::Development,
        BoundedSplit::Validation,
    ] {
        for task in [TaskKind::AssociativeRecall, TaskKind::Copy] {
            output.push(canonical_report(split.id(), task, audit_cell(split, task)?));
        }
    }
    for task in [TaskKind::AssociativeRecall, TaskKind::Copy] {
        output.push(canonical_report(
            "pooled_non_holdout",
            task,
            audit_pooled(task)?,
        ));
    }
    Ok(output)
}

fn main() {
    match evaluate_all() {
        Ok(lines) => {
            println!("TDI-7.4 bounded source-identifiability audit");
            println!("scope=training+development+validation");
            println!("source_static=exactly_six_TDI7.1_static_controls");
            println!("source_recovery=depths_1_2_3_4");
            println!("geometry=centered_unit_rms_reorthogonalized_mgs_v1");
            println!("interpretation=source_geometry_only_not_H_AI_3_verdict");
            println!("final_holdout=NOT_ACCESSED");
            println!("confirmatory_H_AI_3_verdict=NOT_COMPUTED");
            for line in lines {
                println!("{line}");
            }
        }
        Err(error) => {
            eprintln!("TDI-7.4 identifiability audit failed: {error:?}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_non_holdout_ranges_are_disjoint_from_tdi74_final_population() {
        for split in [
            BoundedSplit::Training,
            BoundedSplit::Development,
            BoundedSplit::Validation,
        ] {
            assert_eq!(validate_split(split), Ok(()));
        }
    }

    #[test]
    fn all_six_bounded_cells_are_deterministic() {
        for split in [
            BoundedSplit::Training,
            BoundedSplit::Development,
            BoundedSplit::Validation,
        ] {
            for task in [TaskKind::AssociativeRecall, TaskKind::Copy] {
                let left = audit_cell(split, task).unwrap();
                let right = audit_cell(split, task).unwrap();
                assert_eq!(left, right);
                assert!(left.left_rank() >= 1);
                assert!(left.right_rank() >= 1);
                assert!(left.joint_rank() >= left.left_rank());
                assert!(left.joint_rank() >= left.right_rank());
            }
        }
    }

    #[test]
    fn pooled_non_holdout_geometry_is_deterministic() {
        for task in [TaskKind::AssociativeRecall, TaskKind::Copy] {
            let left = audit_pooled(task).unwrap();
            let right = audit_pooled(task).unwrap();
            assert_eq!(left, right);
            assert!(left.left_rank() >= 1);
            assert!(left.right_rank() >= 1);
            assert!(left.joint_rank() >= left.left_rank());
            assert!(left.joint_rank() >= left.right_rank());
        }
    }

    #[test]
    fn adapter_uses_exact_tdi74_source_blocks() {
        let (static_rows, recovery_rows) =
            source_blocks(BoundedSplit::Development, TaskKind::AssociativeRecall).unwrap();
        assert_eq!(static_rows.len(), DEVELOPMENT_COUNT * 2);
        assert_eq!(recovery_rows.len(), DEVELOPMENT_COUNT * 2);
        assert!(static_rows.iter().all(|row| row.len() == 6));
        assert!(recovery_rows.iter().all(|row| row.len() == RECOVERY_DEPTHS));
    }

    #[test]
    fn pooled_adapter_contains_all_non_holdout_records() {
        let expected = (TRAIN_COUNT + DEVELOPMENT_COUNT + VALIDATION_COUNT) * 2;
        for task in [TaskKind::AssociativeRecall, TaskKind::Copy] {
            let (static_rows, recovery_rows) = pooled_source_blocks(task).unwrap();
            assert_eq!(static_rows.len(), expected);
            assert_eq!(recovery_rows.len(), expected);
        }
    }

    #[test]
    fn source_contains_no_final_holdout_authorization_secret() {
        let source = include_str!("tdi-attention-v74-identifiability.rs");
        let confirmation = ["I_ACCEPT_THE_TDI7_", "HOLDOUT_FREEZE"].concat();
        let environment = ["TDI7_CONFIRM_FINAL_", "HOLDOUT"].concat();
        assert!(!source.contains(&confirmation));
        assert!(!source.contains(&environment));
    }
}
