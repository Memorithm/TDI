//! Bounded TDI-7.4 Gaussian/MMI qualification evaluator.
//!
//! This executable evaluates development and validation populations only. It
//! reuses the corrected TDI-7 task/intervention/recovery mechanics, extracts
//! exactly the six preregistered static controls and four pre-target recovery
//! depths, and applies the vector Gaussian/MMI software-qualification core.
//! No final-holdout execution or confirmatory H-AI-3 verdict exists here.

use tdi_bench::attention_v7::TaskKind;
use tdi_bench::gaussian_mmi_v7::{
    BootstrapSummary, InformationError, InformationRecord, TDI74_BOOTSTRAP_REPLICATES,
    TDI74_BOOTSTRAP_SEED, bootstrap_pid,
};
use tdi_bench::predictive_v7::{
    PredictiveError, PredictiveRecord, TDI71_INTERVENTION_AMPLITUDE, TDI71_TARGET_DEPTH,
    attention_population,
};

const DEVELOPMENT_START: u64 = 7_100_010_000;
const VALIDATION_START: u64 = 7_100_020_000;
const GENERATORS_PER_TASK: usize = 64;
const RECOVERY_DEPTHS: usize = 4;
const TDI74_FINAL_START: u64 = 7_100_050_000;
const TDI74_FINAL_END: u64 = 7_100_059_999;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BoundedSplit {
    Development,
    Validation,
}

impl BoundedSplit {
    const fn start(self) -> u64 {
        match self {
            Self::Development => DEVELOPMENT_START,
            Self::Validation => VALIDATION_START,
        }
    }

    const fn id(self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Validation => "validation",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EvaluationError {
    SeedOverflow,
    FinalRangeOverlap,
    PredictiveFailure(PredictiveError),
    InvalidRecordLayout,
    InformationFailure(InformationError),
}

fn bounded_end(start: u64) -> Result<u64, EvaluationError> {
    let count = u64::try_from(GENERATORS_PER_TASK).map_err(|_| EvaluationError::SeedOverflow)?;
    start
        .checked_add(count.saturating_sub(1))
        .ok_or(EvaluationError::SeedOverflow)
}

fn ranges_overlap(left_start: u64, left_end: u64, right_start: u64, right_end: u64) -> bool {
    left_start <= right_end && right_start <= left_end
}

fn validate_split(split: BoundedSplit) -> Result<(), EvaluationError> {
    let end = bounded_end(split.start())?;
    if ranges_overlap(split.start(), end, TDI74_FINAL_START, TDI74_FINAL_END) {
        return Err(EvaluationError::FinalRangeOverlap);
    }
    Ok(())
}

fn information_record(record: &PredictiveRecord) -> Result<InformationRecord, EvaluationError> {
    if record.baseline().len() != 10 || record.evidence().len() != RECOVERY_DEPTHS {
        return Err(EvaluationError::InvalidRecordLayout);
    }
    // The TDI-7.1 baseline layout is:
    // [length, distractors, retrieval_distance, six static controls, site].
    // TDI-7.4's information source S is exactly the six controls at 3..9.
    InformationRecord::new(
        record.generator_id(),
        record.baseline()[3..9].to_vec(),
        record.evidence().to_vec(),
        record.target(),
    )
    .map_err(EvaluationError::InformationFailure)
}

fn bounded_records(
    split: BoundedSplit,
    task: TaskKind,
) -> Result<Vec<InformationRecord>, EvaluationError> {
    validate_split(split)?;
    let predictive = attention_population(
        split.start(),
        GENERATORS_PER_TASK,
        task,
        RECOVERY_DEPTHS,
        TDI71_TARGET_DEPTH,
        TDI71_INTERVENTION_AMPLITUDE,
    )
    .map_err(EvaluationError::PredictiveFailure)?;
    predictive.iter().map(information_record).collect()
}

fn evaluate_cell(split: BoundedSplit, task: TaskKind) -> Result<BootstrapSummary, EvaluationError> {
    let records = bounded_records(split, task)?;
    bootstrap_pid(&records, TDI74_BOOTSTRAP_REPLICATES, TDI74_BOOTSTRAP_SEED)
        .map_err(EvaluationError::InformationFailure)
}

fn canonical_cell(split: BoundedSplit, task: TaskKind, summary: BootstrapSummary) -> String {
    format!(
        "tdi74-gaussian-mmi-v1;split={};task={};{}",
        split.id(),
        task.id(),
        summary.canonical_report()
    )
}

fn evaluate_all() -> Result<Vec<String>, EvaluationError> {
    let mut output = Vec::with_capacity(4);
    for split in [BoundedSplit::Development, BoundedSplit::Validation] {
        for task in [TaskKind::AssociativeRecall, TaskKind::Copy] {
            output.push(canonical_cell(split, task, evaluate_cell(split, task)?));
        }
    }
    Ok(output)
}

fn main() {
    match evaluate_all() {
        Ok(lines) => {
            println!("TDI-7.4 bounded Gaussian/MMI qualification evaluator");
            println!("scope=development+validation");
            println!("source_static=exactly_six_TDI7.1_static_controls");
            println!("source_recovery=depths_1_2_3_4");
            println!("target=bounded_retrieval_deficit_depth_5");
            println!("estimator=gaussian_mmi_vector_v1");
            println!("rank_reduction=declared_order_reorthogonalized_mgs_v2");
            println!("bootstrap=generator_grouped_percentile_95");
            println!("final_holdout=NOT_ACCESSED");
            println!("confirmatory_H_AI_3_verdict=NOT_COMPUTED");
            for line in lines {
                println!("{line}");
            }
        }
        Err(error) => {
            eprintln!("TDI-7.4 bounded evaluator failed: {error:?}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_ranges_are_disjoint_from_tdi74_final_population() {
        for split in [BoundedSplit::Development, BoundedSplit::Validation] {
            assert_eq!(validate_split(split), Ok(()));
            assert!(!ranges_overlap(
                split.start(),
                bounded_end(split.start()).unwrap(),
                TDI74_FINAL_START,
                TDI74_FINAL_END,
            ));
        }
    }

    #[test]
    fn adapter_selects_exactly_six_static_controls_and_four_recovery_depths() {
        let predictive = attention_population(
            DEVELOPMENT_START,
            1,
            TaskKind::AssociativeRecall,
            RECOVERY_DEPTHS,
            TDI71_TARGET_DEPTH,
            TDI71_INTERVENTION_AMPLITUDE,
        )
        .unwrap();
        assert_eq!(predictive.len(), 2);
        for source in predictive {
            let expected_static = source.baseline()[3..9].to_vec();
            let expected_recovery = source.evidence().to_vec();
            let information = information_record(&source).unwrap();
            assert_eq!(information.static_block(), expected_static);
            assert_eq!(information.recovery_block(), expected_recovery);
            assert_eq!(information.static_block().len(), 6);
            assert_eq!(information.recovery_block().len(), 4);
            assert_eq!(information.generator_id(), source.generator_id());
            assert_eq!(information.target(), source.target());
        }
    }

    #[test]
    fn both_sites_from_each_generator_share_generator_identity() {
        let records = bounded_records(BoundedSplit::Development, TaskKind::Copy).unwrap();
        assert_eq!(records.len(), GENERATORS_PER_TASK * 2);
        for pair in records.chunks_exact(2) {
            assert_eq!(pair[0].generator_id(), pair[1].generator_id());
        }
    }

    #[test]
    fn all_four_bounded_point_estimates_are_deterministic() {
        for split in [BoundedSplit::Development, BoundedSplit::Validation] {
            for task in [TaskKind::AssociativeRecall, TaskKind::Copy] {
                let records = bounded_records(split, task).unwrap();
                let left = tdi_bench::gaussian_mmi_v7::evaluate_pid(&records).unwrap();
                let right = tdi_bench::gaussian_mmi_v7::evaluate_pid(&records).unwrap();
                assert_eq!(left, right);
                assert!(left.static_mi().rank() >= 1);
                assert!(left.recovery_mi().rank() >= 1);
                assert!(left.joint_mi().rank() >= left.static_mi().rank());
            }
        }
    }

    #[test]
    fn source_contains_no_final_holdout_authorization_secret() {
        let source = include_str!("tdi-attention-v74.rs");
        let confirmation = ["I_ACCEPT_THE_TDI7_", "HOLDOUT_FREEZE"].concat();
        let environment = ["TDI7_CONFIRM_FINAL_", "HOLDOUT"].concat();
        assert!(!source.contains(&confirmation));
        assert!(!source.contains(&environment));
    }
}
