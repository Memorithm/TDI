//! Bounded development/validation evaluator for TDI-7.3 H-AI-2.
//!
//! This executable reuses the exact deterministic TDI-7.1 task/intervention
//! mechanics from `tdi_bench::attention_v7`. It intentionally evaluates only
//! inherited non-holdout development/validation seeds. It does not fit the
//! confirmatory B0/B1 models yet and therefore emits no confirmatory verdict.

use tdi_bench::attention_v7::{
    InterventionSite, MechanisticState, SingleSiteIntervention, TaskKind, apply_joint,
    generate_task, recovery_trajectory,
};

const DEVELOPMENT_START: u64 = 7_100_010_000;
const VALIDATION_START: u64 = 7_100_020_000;
const BOUNDED_GENERATORS_PER_TASK: usize = 64;
const RECOVERY_HORIZON: usize = 4;
const INTERVENTION_AMPLITUDE: f64 = 0.25;

// Frozen TDI-7.3 final population decision. The bounded evaluator may know the
// public range solely to prove non-overlap; it never iterates over this range.
const TDI73_FINAL_START: u64 = 7_100_040_000;
const TDI73_FINAL_END: u64 = 7_100_049_999;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Clone, Debug, PartialEq)]
struct GeneratorDiagnostic {
    split: BoundedSplit,
    task: TaskKind,
    seed: u64,
    early_recovery: Vec<f64>,
    late_recovery: Vec<f64>,
    joint_recovery: Vec<f64>,
    absolute_site_difference: Vec<f64>,
    relative_site_difference: Vec<f64>,
    excess_coupling: Vec<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct StabilitySummary {
    generator_count: usize,
    trajectory_points: usize,
    mean_absolute_site_difference: f64,
    max_absolute_site_difference: f64,
    mean_relative_site_difference: f64,
    max_relative_site_difference: f64,
    mean_excess_coupling: f64,
    max_absolute_excess_coupling: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EvaluationError {
    InvalidBoundedPopulation,
    FinalRangeOverlap,
    InterventionFailure,
    InvalidTrajectory,
    EmptyPopulation,
}

fn bounded_seed_end(start: u64) -> Result<u64, EvaluationError> {
    let count = u64::try_from(BOUNDED_GENERATORS_PER_TASK)
        .map_err(|_| EvaluationError::InvalidBoundedPopulation)?;
    start
        .checked_add(count.saturating_sub(1))
        .ok_or(EvaluationError::InvalidBoundedPopulation)
}

fn ranges_overlap(left_start: u64, left_end: u64, right_start: u64, right_end: u64) -> bool {
    left_start <= right_end && right_start <= left_end
}

fn validate_bounded_split(split: BoundedSplit) -> Result<(), EvaluationError> {
    let end = bounded_seed_end(split.start())?;
    if ranges_overlap(split.start(), end, TDI73_FINAL_START, TDI73_FINAL_END) {
        return Err(EvaluationError::FinalRangeOverlap);
    }
    Ok(())
}

fn checked_recovery(value: f64) -> Result<f64, EvaluationError> {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Ok(value)
    } else {
        Err(EvaluationError::InvalidTrajectory)
    }
}

fn symmetric_relative_difference(left: f64, right: f64) -> Result<f64, EvaluationError> {
    let left = checked_recovery(left)?;
    let right = checked_recovery(right)?;
    let denominator = left.abs().max(right.abs()).max(f64::EPSILON);
    Ok((left - right).abs() / denominator)
}

fn generator_diagnostic(
    split: BoundedSplit,
    task: TaskKind,
    seed: u64,
) -> Result<GeneratorDiagnostic, EvaluationError> {
    validate_bounded_split(split)?;
    let task_example = generate_task(task, seed);
    let reference = MechanisticState::from_task(&task_example);
    let early = SingleSiteIntervention::new(InterventionSite::EarlyToken, INTERVENTION_AMPLITUDE);
    let late = SingleSiteIntervention::new(InterventionSite::LateToken, INTERVENTION_AMPLITUDE);
    let early_state = early
        .apply(&reference)
        .map_err(|_| EvaluationError::InterventionFailure)?;
    let late_state = late
        .apply(&reference)
        .map_err(|_| EvaluationError::InterventionFailure)?;
    let joint_state =
        apply_joint(&reference, early, late).map_err(|_| EvaluationError::InterventionFailure)?;

    let early_recovery = recovery_trajectory(&reference, &early_state, RECOVERY_HORIZON);
    let late_recovery = recovery_trajectory(&reference, &late_state, RECOVERY_HORIZON);
    let joint_recovery = recovery_trajectory(&reference, &joint_state, RECOVERY_HORIZON);
    if early_recovery.len() != RECOVERY_HORIZON
        || late_recovery.len() != RECOVERY_HORIZON
        || joint_recovery.len() != RECOVERY_HORIZON
    {
        return Err(EvaluationError::InvalidTrajectory);
    }

    let mut absolute_site_difference = Vec::with_capacity(RECOVERY_HORIZON);
    let mut relative_site_difference = Vec::with_capacity(RECOVERY_HORIZON);
    let mut excess_coupling = Vec::with_capacity(RECOVERY_HORIZON);
    for depth in 0..RECOVERY_HORIZON {
        let early_overlap = checked_recovery(early_recovery[depth])?;
        let late_overlap = checked_recovery(late_recovery[depth])?;
        let joint_overlap = checked_recovery(joint_recovery[depth])?;
        absolute_site_difference.push((early_overlap - late_overlap).abs());
        relative_site_difference.push(symmetric_relative_difference(early_overlap, late_overlap)?);
        let early_deficit = 1.0 - early_overlap;
        let late_deficit = 1.0 - late_overlap;
        let joint_deficit = 1.0 - joint_overlap;
        excess_coupling.push(joint_deficit - early_deficit - late_deficit);
    }

    Ok(GeneratorDiagnostic {
        split,
        task,
        seed,
        early_recovery,
        late_recovery,
        joint_recovery,
        absolute_site_difference,
        relative_site_difference,
        excess_coupling,
    })
}

fn bounded_population(
    split: BoundedSplit,
    task: TaskKind,
) -> Result<Vec<GeneratorDiagnostic>, EvaluationError> {
    validate_bounded_split(split)?;
    (0..BOUNDED_GENERATORS_PER_TASK)
        .map(|offset| {
            let offset =
                u64::try_from(offset).map_err(|_| EvaluationError::InvalidBoundedPopulation)?;
            let seed = split
                .start()
                .checked_add(offset)
                .ok_or(EvaluationError::InvalidBoundedPopulation)?;
            generator_diagnostic(split, task, seed)
        })
        .collect()
}

fn summarize(records: &[GeneratorDiagnostic]) -> Result<StabilitySummary, EvaluationError> {
    if records.is_empty() {
        return Err(EvaluationError::EmptyPopulation);
    }
    let mut absolute_sum = 0.0;
    let mut absolute_max = 0.0_f64;
    let mut relative_sum = 0.0;
    let mut relative_max = 0.0_f64;
    let mut coupling_sum = 0.0;
    let mut coupling_max_abs = 0.0_f64;
    let mut points = 0usize;

    for record in records {
        if record.absolute_site_difference.len() != RECOVERY_HORIZON
            || record.relative_site_difference.len() != RECOVERY_HORIZON
            || record.excess_coupling.len() != RECOVERY_HORIZON
        {
            return Err(EvaluationError::InvalidTrajectory);
        }
        for depth in 0..RECOVERY_HORIZON {
            let absolute = record.absolute_site_difference[depth];
            let relative = record.relative_site_difference[depth];
            let coupling = record.excess_coupling[depth];
            if !absolute.is_finite() || !relative.is_finite() || !coupling.is_finite() {
                return Err(EvaluationError::InvalidTrajectory);
            }
            absolute_sum += absolute;
            absolute_max = absolute_max.max(absolute);
            relative_sum += relative;
            relative_max = relative_max.max(relative);
            coupling_sum += coupling;
            coupling_max_abs = coupling_max_abs.max(coupling.abs());
            points = points
                .checked_add(1)
                .ok_or(EvaluationError::InvalidBoundedPopulation)?;
        }
    }

    if points == 0 {
        return Err(EvaluationError::EmptyPopulation);
    }
    let denominator = points as f64;
    Ok(StabilitySummary {
        generator_count: records.len(),
        trajectory_points: points,
        mean_absolute_site_difference: absolute_sum / denominator,
        max_absolute_site_difference: absolute_max,
        mean_relative_site_difference: relative_sum / denominator,
        max_relative_site_difference: relative_max,
        mean_excess_coupling: coupling_sum / denominator,
        max_absolute_excess_coupling: coupling_max_abs,
    })
}

fn canonical_summary(split: BoundedSplit, task: TaskKind, summary: StabilitySummary) -> String {
    format!(
        concat!(
            "tdi73-bounded-v1;split={};task={};generators={};points={};",
            "mean_abs_bits={:016x};max_abs_bits={:016x};",
            "mean_rel_bits={:016x};max_rel_bits={:016x};",
            "mean_coupling_bits={:016x};max_abs_coupling_bits={:016x}"
        ),
        split.id(),
        task.id(),
        summary.generator_count,
        summary.trajectory_points,
        summary.mean_absolute_site_difference.to_bits(),
        summary.max_absolute_site_difference.to_bits(),
        summary.mean_relative_site_difference.to_bits(),
        summary.max_relative_site_difference.to_bits(),
        summary.mean_excess_coupling.to_bits(),
        summary.max_absolute_excess_coupling.to_bits(),
    )
}

fn evaluate_all_bounded() -> Result<Vec<String>, EvaluationError> {
    let mut lines = Vec::new();
    for split in [BoundedSplit::Development, BoundedSplit::Validation] {
        for task in [TaskKind::AssociativeRecall, TaskKind::Copy] {
            let population = bounded_population(split, task)?;
            let summary = summarize(&population)?;
            lines.push(canonical_summary(split, task, summary));
        }
    }
    Ok(lines)
}

fn main() {
    match evaluate_all_bounded() {
        Ok(lines) => {
            println!("TDI-7.3 bounded heterogeneity/coupling evaluator");
            println!("scope=development+validation");
            println!("final_holdout=NOT_ACCESSED");
            println!("confirmatory_verdict=NOT_COMPUTED");
            println!("predictive_B0_B1_layer=NOT_YET_IMPLEMENTED");
            for line in lines {
                println!("{line}");
            }
        }
        Err(error) => {
            eprintln!("TDI-7.3 bounded evaluator failed: {error:?}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_splits_cannot_overlap_frozen_tdi73_final_range() {
        for split in [BoundedSplit::Development, BoundedSplit::Validation] {
            assert_eq!(validate_bounded_split(split), Ok(()));
            let end = bounded_seed_end(split.start()).unwrap();
            assert!(!ranges_overlap(
                split.start(),
                end,
                TDI73_FINAL_START,
                TDI73_FINAL_END
            ));
        }
    }

    #[test]
    fn one_generator_contains_all_three_measured_trajectories() {
        let record = generator_diagnostic(
            BoundedSplit::Development,
            TaskKind::AssociativeRecall,
            DEVELOPMENT_START,
        )
        .unwrap();
        assert_eq!(record.early_recovery.len(), RECOVERY_HORIZON);
        assert_eq!(record.late_recovery.len(), RECOVERY_HORIZON);
        assert_eq!(record.joint_recovery.len(), RECOVERY_HORIZON);
        assert_eq!(record.absolute_site_difference.len(), RECOVERY_HORIZON);
        assert_eq!(record.relative_site_difference.len(), RECOVERY_HORIZON);
        assert_eq!(record.excess_coupling.len(), RECOVERY_HORIZON);
    }

    #[test]
    fn relative_site_difference_is_explicit_and_bounded() {
        assert_eq!(symmetric_relative_difference(0.5, 0.5), Ok(0.0));
        let difference = symmetric_relative_difference(0.5, 0.75).unwrap();
        assert!((difference - (1.0 / 3.0)).abs() <= 1.0e-12);
        assert_eq!(
            symmetric_relative_difference(f64::NAN, 0.5),
            Err(EvaluationError::InvalidTrajectory)
        );
    }

    #[test]
    fn bounded_population_is_exactly_reproducible() {
        let left = bounded_population(BoundedSplit::Validation, TaskKind::Copy).unwrap();
        let right = bounded_population(BoundedSplit::Validation, TaskKind::Copy).unwrap();
        assert_eq!(left, right);
    }

    #[test]
    fn all_four_bounded_task_split_summaries_are_deterministic() {
        let left = evaluate_all_bounded().unwrap();
        let right = evaluate_all_bounded().unwrap();
        assert_eq!(left, right);
        assert_eq!(left.len(), 4);
        assert!(
            left.iter()
                .all(|line| line.starts_with("tdi73-bounded-v1;"))
        );
    }

    #[test]
    fn task_labels_and_seed_provenance_are_preserved() {
        let record = generator_diagnostic(
            BoundedSplit::Validation,
            TaskKind::Copy,
            VALIDATION_START + 7,
        )
        .unwrap();
        assert_eq!(record.split, BoundedSplit::Validation);
        assert_eq!(record.task, TaskKind::Copy);
        assert_eq!(record.seed, VALIDATION_START + 7);
    }

    #[test]
    fn source_contains_no_final_holdout_authorization_secret() {
        let source = include_str!("tdi-attention-v73.rs");
        let confirmation = ["I_ACCEPT_THE_TDI7_", "HOLDOUT_FREEZE"].concat();
        let environment = ["TDI7_CONFIRM_FINAL_", "HOLDOUT"].concat();
        assert!(!source.contains(&confirmation));
        assert!(!source.contains(&environment));
    }
}
