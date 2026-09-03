//! Protocol-faithful bounded predictive layer for TDI-7.3 H-AI-2.
//!
//! B0/B1 retains the exact TDI-7.1 model ladder and uncertainty discipline.
//! Heterogeneity and coupling remain separate mechanistic diagnostics in
//! `tdi-attention-v73`; this executable does not reinterpret predictive gain as
//! an H-AI-2 coupling verdict. Training/development/validation are non-holdout.

use tdi_bench::attention_v7::TaskKind;
use tdi_bench::predictive_v7::{
    NestedSummary, TDI71_BOOTSTRAP_REPLICATES, TDI71_BOOTSTRAP_SEED, TDI71_EARLY_DEPTHS,
    TDI71_INTERVENTION_AMPLITUDE, TDI71_TARGET_DEPTH, attention_population, evaluate_nested_ridge,
};

const TRAIN_START: u64 = 7_100_000_000;
const TRAIN_COUNT: usize = 96;
const DEVELOPMENT_START: u64 = 7_100_010_000;
const DEVELOPMENT_COUNT: usize = 48;
const VALIDATION_START: u64 = 7_100_020_000;
const VALIDATION_COUNT: usize = 48;

// Public TDI-7.3 final range is used only for explicit non-overlap proof.
const TDI73_FINAL_START: u64 = 7_100_040_000;
const TDI73_FINAL_END: u64 = 7_100_049_999;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EvaluationError {
    FinalRangeOverlap,
    PredictiveFailure,
    SeedOverflow,
}

fn bounded_end(start: u64, count: usize) -> Result<u64, EvaluationError> {
    let count = u64::try_from(count).map_err(|_| EvaluationError::SeedOverflow)?;
    start
        .checked_add(count.saturating_sub(1))
        .ok_or(EvaluationError::SeedOverflow)
}

fn ranges_overlap(left_start: u64, left_end: u64, right_start: u64, right_end: u64) -> bool {
    left_start <= right_end && right_start <= left_end
}

fn validate_non_holdout_ranges() -> Result<(), EvaluationError> {
    for (start, count) in [
        (TRAIN_START, TRAIN_COUNT),
        (DEVELOPMENT_START, DEVELOPMENT_COUNT),
        (VALIDATION_START, VALIDATION_COUNT),
    ] {
        let end = bounded_end(start, count)?;
        if ranges_overlap(start, end, TDI73_FINAL_START, TDI73_FINAL_END) {
            return Err(EvaluationError::FinalRangeOverlap);
        }
    }
    Ok(())
}

fn run_task(kind: TaskKind) -> Result<NestedSummary, EvaluationError> {
    validate_non_holdout_ranges()?;
    let training = attention_population(
        TRAIN_START,
        TRAIN_COUNT,
        kind,
        TDI71_EARLY_DEPTHS,
        TDI71_TARGET_DEPTH,
        TDI71_INTERVENTION_AMPLITUDE,
    )
    .map_err(|_| EvaluationError::PredictiveFailure)?;
    let development = attention_population(
        DEVELOPMENT_START,
        DEVELOPMENT_COUNT,
        kind,
        TDI71_EARLY_DEPTHS,
        TDI71_TARGET_DEPTH,
        TDI71_INTERVENTION_AMPLITUDE,
    )
    .map_err(|_| EvaluationError::PredictiveFailure)?;
    let validation = attention_population(
        VALIDATION_START,
        VALIDATION_COUNT,
        kind,
        TDI71_EARLY_DEPTHS,
        TDI71_TARGET_DEPTH,
        TDI71_INTERVENTION_AMPLITUDE,
    )
    .map_err(|_| EvaluationError::PredictiveFailure)?;
    evaluate_nested_ridge(
        &training,
        &development,
        &validation,
        TDI71_BOOTSTRAP_REPLICATES,
        TDI71_BOOTSTRAP_SEED,
    )
    .map_err(|_| EvaluationError::PredictiveFailure)
}

fn canonical_task_summary(kind: TaskKind, summary: NestedSummary) -> String {
    format!(
        "tdi73-predictive-v1;task={};{}",
        kind.id(),
        summary.canonical_report()
    )
}

fn main() {
    let result = (|| {
        let associative = run_task(TaskKind::AssociativeRecall)?;
        let copy = run_task(TaskKind::Copy)?;
        Ok::<_, EvaluationError>((associative, copy))
    })();

    match result {
        Ok((associative, copy)) => {
            println!("TDI-7.3 bounded predictive evaluator");
            println!("scope=training+development+validation");
            println!("semantic=deterministic_local_row_stochastic_v1");
            println!("intervention=balanced_add_subtract_v1");
            println!("model_ladder=TDI-7.1_B0_B1_ridge");
            println!("bootstrap=generator_paired_percentile_95");
            println!("final_holdout=NOT_ACCESSED");
            println!("confirmatory_H_AI_2_verdict=NOT_COMPUTED");
            println!(
                "{}",
                canonical_task_summary(TaskKind::AssociativeRecall, associative)
            );
            println!("{}", canonical_task_summary(TaskKind::Copy, copy));
        }
        Err(error) => {
            eprintln!("TDI-7.3 predictive evaluator failed: {error:?}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_non_holdout_ranges_are_disjoint_from_tdi73_final_population() {
        assert_eq!(validate_non_holdout_ranges(), Ok(()));
    }

    #[test]
    fn both_task_families_produce_deterministic_predictive_summaries() {
        for kind in [TaskKind::AssociativeRecall, TaskKind::Copy] {
            let left = run_task(kind).unwrap();
            let right = run_task(kind).unwrap();
            assert_eq!(left, right);
            assert_eq!(left.validation_records(), VALIDATION_COUNT * 2);
            assert_eq!(left.validation_generators(), VALIDATION_COUNT);
            assert!(left.b0_mse().is_finite());
            assert!(left.b1_mse().is_finite());
            assert!(left.relative_reduction().is_finite());
            assert!(left.lower_95().is_finite());
            assert!(left.upper_95().is_finite());
        }
    }

    #[test]
    fn predictive_reporting_cannot_be_mislabelled_as_hai2_verdict() {
        let source = include_str!("tdi-attention-v73-predictive.rs");
        assert!(source.contains("confirmatory_H_AI_2_verdict=NOT_COMPUTED"));
    }

    #[test]
    fn source_contains_no_final_holdout_authorization_secret() {
        let source = include_str!("tdi-attention-v73-predictive.rs");
        let confirmation = ["I_ACCEPT_THE_TDI7_", "HOLDOUT_FREEZE"].concat();
        let environment = ["TDI7_CONFIRM_FINAL_", "HOLDOUT"].concat();
        assert!(!source.contains(&confirmation));
        assert!(!source.contains(&environment));
    }
}
