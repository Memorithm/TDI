//! Bounded end-to-end TDI-7.1 evaluator on non-holdout seeds only.
//!
//! This executable integrates deterministic task generation, a declared
//! attention-like reference semantic, one-shot task-label-preserving
//! interventions, static controls, early TDI recovery descriptors, a late
//! bounded retrieval-deficit target, nested B0/B1 ridge models, and paired
//! bootstrap uncertainty.
//!
//! It deliberately has no TDI-7.2 final-holdout authorization path.

use tdi_ai::{
    BalancedTokenShift, FixedAttentionMixer, FullStateObservable, ReciprocalLInfRecovery,
    ToyAttentionState, analyze_intervention_recovery, analyze_static_attention,
};

const TRAIN_START: u64 = 7_100_000_000;
const TRAIN_COUNT: usize = 96;
const DEV_START: u64 = 7_100_010_000;
const DEV_COUNT: usize = 48;
const VALIDATION_START: u64 = 7_100_020_000;
const VALIDATION_COUNT: usize = 48;
const EARLY_DEPTHS: usize = 2;
const TARGET_DEPTH: usize = 5;
const INTERVENTION_AMPLITUDE: f64 = 0.25;
const LAMBDA_GRID: [f64; 4] = [0.0, 1.0e-6, 1.0e-3, 1.0e-1];
const BOOTSTRAP_REPLICATES: usize = 2_000;
const BOOTSTRAP_SEED: u64 = 0x5444_4937_4532_4501;
const PIVOT_TOLERANCE: f64 = 1.0e-12;

const _: () = assert!(TARGET_DEPTH > EARLY_DEPTHS);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TaskKind {
    AssociativeRecall,
    Copy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Site {
    Early,
    Late,
}

#[derive(Clone, Copy, Debug)]
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn bounded(&mut self, upper: usize) -> usize {
        assert!(upper > 0);
        (self.next_u64() % upper as u64) as usize
    }
}

#[derive(Clone, Debug)]
struct Task {
    tokens: Vec<u16>,
    retrieval_index: usize,
    distractors: usize,
    retrieval_distance: usize,
}

fn unique_tokens(rng: &mut SplitMix64, count: usize, base: u16, width: u16) -> Vec<u16> {
    let mut out = Vec::with_capacity(count);
    while out.len() < count {
        let value = base + (rng.next_u64() % u64::from(width)) as u16;
        if !out.contains(&value) {
            out.push(value);
        }
    }
    out
}

fn task(seed: u64, kind: TaskKind) -> Task {
    match kind {
        TaskKind::AssociativeRecall => {
            let mut rng = SplitMix64::new(seed ^ 0x5444_4937_4152_0001);
            let keys = unique_tokens(&mut rng, 4, 1, 64);
            let values = unique_tokens(&mut rng, 4, 128, 64);
            let query = rng.bounded(4);
            let mut tokens = Vec::with_capacity(10);
            for index in 0..4 {
                tokens.push(keys[index]);
                tokens.push(values[index]);
            }
            tokens.push(250);
            tokens.push(keys[query]);
            let retrieval_index = query * 2 + 1;
            Task {
                retrieval_index,
                retrieval_distance: 9 - retrieval_index,
                distractors: 3,
                tokens,
            }
        }
        TaskKind::Copy => {
            let mut rng = SplitMix64::new(seed ^ 0x5444_4937_434F_0001);
            let source = unique_tokens(&mut rng, 4, 16, 96);
            let distractors = rng.bounded(4) + 1;
            let noise = unique_tokens(&mut rng, distractors, 160, 64);
            let mut tokens = source;
            tokens.extend(noise);
            tokens.push(251);
            Task {
                tokens,
                retrieval_index: 0,
                distractors,
                retrieval_distance: distractors + 1,
            }
        }
    }
}

/// Deterministic row-stochastic local mixer. The task controls alter the
/// self/neighbor balance before any intervention or target is observed.
fn mixer(task: &Task) -> Vec<Vec<f64>> {
    let n = task.tokens.len();
    let spread = (task.retrieval_distance as f64 / (n as f64 + 1.0)).clamp(0.0, 1.0);
    let side = 0.15 + 0.10 * spread;
    let center = 1.0 - 2.0 * side;
    let mut matrix = vec![vec![0.0; n]; n];
    for row in 0..n {
        if row == 0 {
            matrix[row][row] = center + side;
            matrix[row][row + 1] = side;
        } else if row + 1 == n {
            matrix[row][row - 1] = side;
            matrix[row][row] = center + side;
        } else {
            matrix[row][row - 1] = side;
            matrix[row][row] = center;
            matrix[row][row + 1] = side;
        }
    }
    matrix
}

fn initial_state(task: &Task) -> ToyAttentionState {
    ToyAttentionState::new(
        task.tokens
            .iter()
            .map(|token| f64::from(*token) / 256.0)
            .collect(),
    )
    .expect("generated task tokens are finite")
}

fn intervention_indices(task: &Task, site: Site) -> (usize, usize) {
    let len = task.tokens.len();
    match site {
        Site::Early => (1, 0),
        Site::Late => (len - 2, len - 1),
    }
}

fn bounded_deficit(distance: f64) -> f64 {
    assert!(distance.is_finite() && distance >= 0.0);
    distance / (1.0 + distance)
}

fn late_retrieval_deficit(task: &Task, site: Site) -> f64 {
    let weights = mixer(task);
    let dynamics = FixedAttentionMixer::new(weights).expect("generated mixer is row stochastic");
    let initial = initial_state(task);
    let (add_to, subtract_from) = intervention_indices(task, site);
    let intervention = BalancedTokenShift::new(add_to, subtract_from, INTERVENTION_AMPLITUDE)
        .expect("generated intervention is valid");
    let mut reference = initial.clone();
    let mut perturbed = tdi_ai::Intervention::apply(&intervention, &initial)
        .expect("generated intervention applies");
    for _ in 0..TARGET_DEPTH {
        reference = tdi_ai::ReferenceDynamics::advance(&dynamics, &reference)
            .expect("reference dynamics succeeds");
        perturbed = tdi_ai::ReferenceDynamics::advance(&dynamics, &perturbed)
            .expect("perturbed dynamics succeeds");
    }
    let index = task.retrieval_index.min(reference.len() - 1);
    bounded_deficit((reference.values()[index] - perturbed.values()[index]).abs())
}

#[derive(Clone, Debug)]
struct Record {
    generator_id: u64,
    baseline: Vec<f64>,
    recovery: Vec<f64>,
    target: f64,
}

fn record(seed: u64, kind: TaskKind, site: Site) -> Record {
    let task = task(seed, kind);
    let matrix = mixer(&task);
    let static_diag = analyze_static_attention(&matrix).expect("generated matrix is valid");
    let dynamics = FixedAttentionMixer::new(matrix).expect("generated matrix is valid");
    let initial = initial_state(&task);
    let (add_to, subtract_from) = intervention_indices(&task, site);
    let intervention = BalancedTokenShift::new(add_to, subtract_from, INTERVENTION_AMPLITUDE)
        .expect("generated intervention is valid");
    let recovery = analyze_intervention_recovery(
        &dynamics,
        &intervention,
        &FullStateObservable,
        &ReciprocalLInfRecovery,
        &initial,
        EARLY_DEPTHS,
    )
    .expect("recovery extraction succeeds")
    .points()
    .iter()
    .map(|point| *point.overlap())
    .collect::<Vec<_>>();

    let baseline = vec![
        task.tokens.len() as f64,
        task.distractors as f64,
        task.retrieval_distance as f64,
        static_diag.mean_row_entropy_nats(),
        static_diag.mean_normalized_row_entropy(),
        static_diag.mean_row_max_weight(),
        static_diag.mean_row_l2_concentration(),
        static_diag.mean_row_effective_support(),
        static_diag.frobenius_norm(),
        match site {
            Site::Early => 0.0,
            Site::Late => 1.0,
        },
    ];

    Record {
        generator_id: seed,
        baseline,
        recovery,
        target: late_retrieval_deficit(&task, site),
    }
}

fn population(start: u64, count: usize, kind: TaskKind) -> Vec<Record> {
    let mut records = Vec::with_capacity(count * 2);
    for offset in 0..count {
        let seed = start + offset as u64;
        records.push(record(seed, kind, Site::Early));
        records.push(record(seed, kind, Site::Late));
    }
    records
}

#[derive(Clone, Debug)]
struct RidgeModel {
    weights: Vec<f64>,
    lambda: f64,
}

fn design(record: &Record, augmented: bool) -> Vec<f64> {
    let mut row = Vec::with_capacity(1 + record.baseline.len() + record.recovery.len());
    row.push(1.0);
    row.extend_from_slice(&record.baseline);
    if augmented {
        row.extend_from_slice(&record.recovery);
    }
    row
}

fn solve(mut a: Vec<Vec<f64>>, mut b: Vec<f64>) -> Option<Vec<f64>> {
    let n = b.len();
    for pivot in 0..n {
        let mut best = pivot;
        for row in (pivot + 1)..n {
            if a[row][pivot].abs() > a[best][pivot].abs() {
                best = row;
            }
        }
        if a[best][pivot].abs() <= PIVOT_TOLERANCE {
            return None;
        }
        a.swap(pivot, best);
        b.swap(pivot, best);
        let scale = a[pivot][pivot];
        for value in &mut a[pivot][pivot..] {
            *value /= scale;
        }
        b[pivot] /= scale;
        let pivot_row = a[pivot].clone();
        let pivot_rhs = b[pivot];
        for (row_index, row_values) in a.iter_mut().enumerate() {
            if row_index == pivot {
                continue;
            }
            let factor = row_values[pivot];
            for (value, pivot_value) in row_values[pivot..].iter_mut().zip(&pivot_row[pivot..]) {
                *value -= factor * pivot_value;
            }
            b[row_index] -= factor * pivot_rhs;
        }
    }
    Some(b)
}

fn fit(records: &[Record], augmented: bool, lambda: f64) -> Option<RidgeModel> {
    let width = design(records.first()?, augmented).len();
    let mut gram = vec![vec![0.0; width]; width];
    let mut rhs = vec![0.0; width];
    for record in records {
        let row = design(record, augmented);
        for i in 0..width {
            rhs[i] += row[i] * record.target;
            for j in 0..width {
                gram[i][j] += row[i] * row[j];
            }
        }
    }
    for (index, diagonal) in gram.iter_mut().enumerate().skip(1) {
        diagonal[index] += lambda;
    }
    Some(RidgeModel {
        weights: solve(gram, rhs)?,
        lambda,
    })
}

fn predict(model: &RidgeModel, record: &Record, augmented: bool) -> f64 {
    design(record, augmented)
        .iter()
        .zip(&model.weights)
        .map(|(x, w)| x * w)
        .sum()
}

fn squared_errors(model: &RidgeModel, records: &[Record], augmented: bool) -> Vec<f64> {
    records
        .iter()
        .map(|record| {
            let error = predict(model, record, augmented) - record.target;
            error * error
        })
        .collect()
}

fn mse(model: &RidgeModel, records: &[Record], augmented: bool) -> f64 {
    let errors = squared_errors(model, records, augmented);
    errors.iter().sum::<f64>() / errors.len() as f64
}

fn select(training: &[Record], development: &[Record], augmented: bool) -> RidgeModel {
    LAMBDA_GRID
        .into_iter()
        .filter_map(|lambda| {
            fit(training, augmented, lambda)
                .map(|model| (mse(&model, development, augmented), model))
        })
        .min_by(|left, right| left.0.total_cmp(&right.0))
        .expect("at least one ridge system must be solvable")
        .1
}

#[derive(Clone, Copy, Debug)]
struct Summary {
    b0_lambda: f64,
    b1_lambda: f64,
    b0_mse: f64,
    b1_mse: f64,
    relative_reduction: f64,
    lower_95: f64,
    upper_95: f64,
}

impl Summary {
    fn report(&self) -> String {
        format!(
            "b0_lambda={} b1_lambda={} b0_mse={} b1_mse={} relative_reduction={} lower_95={} upper_95={}",
            self.b0_lambda,
            self.b1_lambda,
            self.b0_mse,
            self.b1_mse,
            self.relative_reduction,
            self.lower_95,
            self.upper_95
        )
    }
}

fn percentile(sorted: &[f64], probability: f64) -> f64 {
    let position = probability * (sorted.len() - 1) as f64;
    let lo = position.floor() as usize;
    let hi = position.ceil() as usize;
    if lo == hi {
        sorted[lo]
    } else {
        let weight = position - lo as f64;
        sorted[lo] * (1.0 - weight) + sorted[hi] * weight
    }
}

fn evaluate(training: &[Record], development: &[Record], validation: &[Record]) -> Summary {
    let b0 = select(training, development, false);
    let b1 = select(training, development, true);
    let b0_errors = squared_errors(&b0, validation, false);
    let b1_errors = squared_errors(&b1, validation, true);
    let b0_mse = b0_errors.iter().sum::<f64>() / b0_errors.len() as f64;
    let b1_mse = b1_errors.iter().sum::<f64>() / b1_errors.len() as f64;
    assert!(b0_mse.is_finite() && b0_mse > 0.0 && b1_mse.is_finite());
    let relative_reduction = (b0_mse - b1_mse) / b0_mse;

    // Pair by generator id: both intervention-site records for a seed are
    // resampled together, preserving the generator-example unit.
    let mut groups = Vec::<(u64, Vec<usize>)>::new();
    for (index, record) in validation.iter().enumerate() {
        if let Some((_, indices)) = groups.iter_mut().find(|(id, _)| *id == record.generator_id) {
            indices.push(index);
        } else {
            groups.push((record.generator_id, vec![index]));
        }
    }
    let mut rng = SplitMix64::new(BOOTSTRAP_SEED);
    let mut reductions = Vec::with_capacity(BOOTSTRAP_REPLICATES);
    for _ in 0..BOOTSTRAP_REPLICATES {
        let mut sum0 = 0.0;
        let mut sum1 = 0.0;
        let mut count = 0usize;
        for _ in 0..groups.len() {
            let (_, indices) = &groups[rng.bounded(groups.len())];
            for &index in indices {
                sum0 += b0_errors[index];
                sum1 += b1_errors[index];
                count += 1;
            }
        }
        let mean0 = sum0 / count as f64;
        let mean1 = sum1 / count as f64;
        reductions.push((mean0 - mean1) / mean0);
    }
    reductions.sort_by(f64::total_cmp);
    Summary {
        b0_lambda: b0.lambda,
        b1_lambda: b1.lambda,
        b0_mse,
        b1_mse,
        relative_reduction,
        lower_95: percentile(&reductions, 0.025),
        upper_95: percentile(&reductions, 0.975),
    }
}

fn run_task(kind: TaskKind) -> Summary {
    let training = population(TRAIN_START, TRAIN_COUNT, kind);
    let development = population(DEV_START, DEV_COUNT, kind);
    let validation = population(VALIDATION_START, VALIDATION_COUNT, kind);
    evaluate(&training, &development, &validation)
}

fn main() {
    let associative = run_task(TaskKind::AssociativeRecall);
    let copy = run_task(TaskKind::Copy);
    println!("TDI-7.1 bounded end-to-end evaluator: PASS");
    println!("scope=train/development/validation only");
    println!("semantic=deterministic_local_row_stochastic_v1");
    println!("early_depths=1..={EARLY_DEPTHS} target_depth={TARGET_DEPTH}");
    println!("target=bounded_retrieval_deficit:d/(1+d)");
    println!("associative_validation={}", associative.report());
    println!("copy_validation={}", copy.report());
    println!("TDI-7.2 final holdout: NOT ACCESSED");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deficit_is_bounded_and_has_frozen_orientation() {
        assert_eq!(bounded_deficit(0.0), 0.0);
        assert!(bounded_deficit(0.5) > 0.0);
        assert!(bounded_deficit(1.0) < 1.0);
    }

    #[test]
    fn intervention_locations_preserve_generated_task() {
        for kind in [TaskKind::AssociativeRecall, TaskKind::Copy] {
            let generated = task(TRAIN_START, kind);
            for site in [Site::Early, Site::Late] {
                let original_tokens = generated.tokens.clone();
                let _ = late_retrieval_deficit(&generated, site);
                assert_eq!(generated.tokens, original_tokens);
            }
        }
    }

    #[test]
    fn static_and_recovery_blocks_are_nested_only_in_b1() {
        let sample = record(TRAIN_START, TaskKind::AssociativeRecall, Site::Early);
        assert_eq!(design(&sample, false).len(), 1 + sample.baseline.len());
        assert_eq!(
            design(&sample, true).len(),
            1 + sample.baseline.len() + sample.recovery.len()
        );
    }

    #[test]
    fn generated_records_are_deterministic_and_finite() {
        let left = record(TRAIN_START, TaskKind::Copy, Site::Late);
        let right = record(TRAIN_START, TaskKind::Copy, Site::Late);
        assert_eq!(left.generator_id, right.generator_id);
        assert_eq!(left.baseline, right.baseline);
        assert_eq!(left.recovery, right.recovery);
        assert_eq!(left.target, right.target);
        assert!(left.target.is_finite() && left.target >= 0.0 && left.target < 1.0);
    }

    #[test]
    fn validation_run_is_deterministic() {
        let first = run_task(TaskKind::AssociativeRecall);
        let second = run_task(TaskKind::AssociativeRecall);
        assert_eq!(first.b0_lambda, second.b0_lambda);
        assert_eq!(first.b1_lambda, second.b1_lambda);
        assert_eq!(first.b0_mse, second.b0_mse);
        assert_eq!(first.b1_mse, second.b1_mse);
        assert_eq!(first.relative_reduction, second.relative_reduction);
        assert_eq!(first.lower_95, second.lower_95);
        assert_eq!(first.upper_95, second.upper_95);
    }

    #[test]
    fn source_has_no_final_holdout_authorization_surface() {
        let source = include_str!("tdi7_end_to_end.rs");
        let confirmation = ["I_ACCEPT_THE_TDI7_", "HOLDOUT_FREEZE"].concat();
        let environment = ["TDI7_CONFIRM_FINAL_", "HOLDOUT"].concat();
        let final_seed = ["7_100_03", "0_000"].concat();
        assert!(!source.contains(&confirmation));
        assert!(!source.contains(&environment));
        assert!(!source.contains(&final_seed));
    }
}
