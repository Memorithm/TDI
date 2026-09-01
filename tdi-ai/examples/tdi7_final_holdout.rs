//! TDI-7.2 confirmatory final-holdout runner (armed 2026-09-01).
//!
//! Frozen inputs: the TDI-7.0 preregistration (seed range, task families,
//! decision rule, multi-task gate), the TDI-7.1 evaluator specification, and
//! the three reviewed TDI-7.2 decision records (population, seed selection,
//!
//! Frozen inputs: the TDI-7.0 preregistration (seed range, task families,
//! decision rule, multi-task gate), the TDI-7.1 evaluator specification, and
//! the three reviewed TDI-7.2 decision records (population, seed selection,
//! rejection policy), frozen on 2026-09-01.
//!
//! The protocol pipeline below is a faithful copy of the frozen bounded
//! evaluator (`tdi7_end_to_end.rs` at the merged pre-arm head): identical task
//! generators, mixer, interventions, features, B0/B1 ridge models, lambda
//! grid, and paired bootstrap. Only the evaluated population differs.
//!
//! Authorization: execution requires the human-supplied environment value
//! `TDI7_CONFIRM_FINAL_HOLDOUT=I_ACCEPT_THE_TDI7_HOLDOUT_FREEZE` at run time.
//! Without it the runner fails closed before touching any data. CI, tests and
//! workflows never supply that value and never execute this example.

use std::env;
use std::fs;

use tdi_ai::{
    BalancedTokenShift, FixedAttentionMixer, FullStateObservable, ReciprocalLInfRecovery,
    ToyAttentionState, analyze_intervention_recovery, analyze_static_attention,
};

const CONFIRM_VAR: &str = "TDI7_CONFIRM_FINAL_HOLDOUT";
const CONFIRM_VALUE: &str = "I_ACCEPT_THE_TDI7_HOLDOUT_FREEZE";

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
const RELEVANCE_MARGIN: f64 = 0.02;

// Frozen TDI-7.2 decision constants (must match the decision records).
const FINAL_SEED_START: u64 = 7_100_030_000;
const FINAL_SEED_END: u64 = 7_100_039_999;
const FINAL_SEED_COUNT: u64 = FINAL_SEED_END - FINAL_SEED_START + 1;
const FROZEN_SELECTION_RULE: &str = "contiguous_ascending_v1";
const FROZEN_REJECTION_POLICY: &str = "frozen_tdi71_typed_errors_v1";
const FROZEN_REJECTION_REASONS: &str =
    "invalid_mixer,invalid_intervention,recovery_extraction_failed,non_finite_target";
const DECISION_REFERENCE: &str = "TDI-7.2-HUMAN-DECISION-2026-09-01-CHECKUPAUTO";

const _: () = assert!(TARGET_DEPTH > EARLY_DEPTHS);
// Frozen split disjointness: train/dev/validation/holdout ranges never overlap.
const _: () = assert!(TRAIN_START + TRAIN_COUNT as u64 <= DEV_START);
const _: () = assert!(DEV_START + DEV_COUNT as u64 <= VALIDATION_START);
const _: () = assert!(VALIDATION_START + VALIDATION_COUNT as u64 <= FINAL_SEED_START);
const _: () = assert!(FINAL_SEED_START <= FINAL_SEED_END);
const _: () = assert!(FINAL_SEED_END - FINAL_SEED_START + 1 == FINAL_SEED_COUNT);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AuthorizationError {
    Missing,
    Mismatch,
}

/// Pure authorization gate: the exact human-supplied value is required.
fn authorization(supplied: Option<&str>) -> Result<(), AuthorizationError> {
    match supplied {
        None => Err(AuthorizationError::Missing),
        Some(value) if value == CONFIRM_VALUE => Ok(()),
        Some(_) => Err(AuthorizationError::Mismatch),
    }
}

/// Minimal line-based decision-record reader (same style as the validators).
fn decision_field(path: &str, key: &str) -> Result<String, String> {
    let input = fs::read_to_string(path).map_err(|_| format!("{path}: unreadable"))?;
    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((name, value)) = line.split_once('=') {
            if name.trim() == key {
                return Ok(value.trim().trim_matches('"').to_string());
            }
        }
    }
    Err(format!("{path}: missing key {key}"))
}

/// Re-validate the three frozen decision records; fail closed on any drift.
fn validate_frozen_decisions() -> Result<(), String> {
    let checks: [(&str, &str, &str); 12] = [
        (
            "docs/TDI-7.2-FINAL-HOLDOUT-DECISION.toml",
            "decision_status",
            "FROZEN",
        ),
        (
            "docs/TDI-7.2-FINAL-HOLDOUT-DECISION.toml",
            "final_holdout_generator_count",
            "10000",
        ),
        (
            "docs/TDI-7.2-FINAL-HOLDOUT-DECISION.toml",
            "final_seed_start",
            "7100030000",
        ),
        (
            "docs/TDI-7.2-FINAL-HOLDOUT-DECISION.toml",
            "final_seed_end",
            "7100039999",
        ),
        (
            "docs/TDI-7.2-FINAL-HOLDOUT-DECISION.toml",
            "decision_reference",
            DECISION_REFERENCE,
        ),
        (
            "docs/TDI-7.2-FINAL-HOLDOUT-SELECTION.toml",
            "selection_status",
            "FROZEN",
        ),
        (
            "docs/TDI-7.2-FINAL-HOLDOUT-SELECTION.toml",
            "selection_rule",
            FROZEN_SELECTION_RULE,
        ),
        (
            "docs/TDI-7.2-FINAL-HOLDOUT-SELECTION.toml",
            "selection_start",
            "7100030000",
        ),
        (
            "docs/TDI-7.2-FINAL-HOLDOUT-SELECTION.toml",
            "selection_count",
            "10000",
        ),
        (
            "docs/TDI-7.2-FINAL-HOLDOUT-REJECTION-POLICY.toml",
            "policy_status",
            "FROZEN",
        ),
        (
            "docs/TDI-7.2-FINAL-HOLDOUT-REJECTION-POLICY.toml",
            "rejection_policy",
            FROZEN_REJECTION_POLICY,
        ),
        (
            "docs/TDI-7.2-FINAL-HOLDOUT-REJECTION-POLICY.toml",
            "rejection_reasons",
            FROZEN_REJECTION_REASONS,
        ),
    ];
    for (path, key, expected) in checks {
        let actual = decision_field(path, key)?;
        if actual != expected {
            return Err(format!("{path}: {key} drifted: {actual} != {expected}"));
        }
    }
    for path in [
        "docs/TDI-7.2-FINAL-HOLDOUT-DECISION.toml",
        "docs/TDI-7.2-FINAL-HOLDOUT-SELECTION.toml",
        "docs/TDI-7.2-FINAL-HOLDOUT-REJECTION-POLICY.toml",
    ] {
        if decision_field(path, "authorization_state")? != "NOT_AUTHORIZED" {
            return Err(format!(
                "{path}: authorization_state must stay NOT_AUTHORIZED in the record"
            ));
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TaskKind {
    AssociativeRecall,
    Copy,
}

impl TaskKind {
    const fn label(self) -> &'static str {
        match self {
            Self::AssociativeRecall => "associative_recall",
            Self::Copy => "copy",
        }
    }
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

/// Deterministic row-stochastic local mixer (frozen TDI-7.1 semantics).
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

/// Frozen rejection taxonomy (frozen_tdi71_typed_errors_v1). Every rejection
/// is reported with its reason; result-driven exclusions are impossible by
/// construction because rejections can only arise from deterministic
/// construction failures, never from observed values' magnitude or outcome.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RejectionReason {
    InvalidMixer,
    InvalidIntervention,
    RecoveryExtractionFailed,
    NonFiniteTarget,
}

impl RejectionReason {
    const fn label(self) -> &'static str {
        match self {
            Self::InvalidMixer => "invalid_mixer",
            Self::InvalidIntervention => "invalid_intervention",
            Self::RecoveryExtractionFailed => "recovery_extraction_failed",
            Self::NonFiniteTarget => "non_finite_target",
        }
    }
}

fn bounded_deficit(distance: f64) -> f64 {
    assert!(distance.is_finite() && distance >= 0.0);
    distance / (1.0 + distance)
}

/// Late retrieval deficit at the frozen target depth; every construction
/// failure maps onto the frozen rejection taxonomy.
fn late_retrieval_deficit(task: &Task, site: Site) -> Result<f64, RejectionReason> {
    let dynamics =
        FixedAttentionMixer::new(mixer(task)).map_err(|_| RejectionReason::InvalidMixer)?;
    let initial = initial_state(task);
    let (add_to, subtract_from) = intervention_indices(task, site);
    let intervention = BalancedTokenShift::new(add_to, subtract_from, INTERVENTION_AMPLITUDE)
        .map_err(|_| RejectionReason::InvalidIntervention)?;
    let mut reference = initial.clone();
    let mut perturbed = tdi_ai::Intervention::apply(&intervention, &initial)
        .map_err(|_| RejectionReason::InvalidIntervention)?;
    for _ in 0..TARGET_DEPTH {
        reference = tdi_ai::ReferenceDynamics::advance(&dynamics, &reference)
            .map_err(|_| RejectionReason::RecoveryExtractionFailed)?;
        perturbed = tdi_ai::ReferenceDynamics::advance(&dynamics, &perturbed)
            .map_err(|_| RejectionReason::RecoveryExtractionFailed)?;
    }
    let index = task.retrieval_index.min(reference.len() - 1);
    Ok(bounded_deficit(
        (reference.values()[index] - perturbed.values()[index]).abs(),
    ))
}

#[derive(Clone, Debug)]
struct Record {
    generator_id: u64,
    baseline: Vec<f64>,
    recovery: Vec<f64>,
    target: f64,
}

/// Fallible record construction; each failure is a frozen rejection reason.
fn try_record(seed: u64, kind: TaskKind, site: Site) -> Result<Record, RejectionReason> {
    let generated = task(seed, kind);
    let weights = mixer(&generated);
    let dynamics =
        FixedAttentionMixer::new(weights.clone()).map_err(|_| RejectionReason::InvalidMixer)?;
    let static_diag =
        analyze_static_attention(&weights).map_err(|_| RejectionReason::InvalidMixer)?;
    let initial = initial_state(&generated);
    let (add_to, subtract_from) = intervention_indices(&generated, site);
    let intervention = BalancedTokenShift::new(add_to, subtract_from, INTERVENTION_AMPLITUDE)
        .map_err(|_| RejectionReason::InvalidIntervention)?;
    let recovery = analyze_intervention_recovery(
        &dynamics,
        &intervention,
        &FullStateObservable,
        &ReciprocalLInfRecovery,
        &initial,
        EARLY_DEPTHS,
    )
    .map_err(|_| RejectionReason::RecoveryExtractionFailed)?;
    let recovery = recovery
        .points()
        .iter()
        .map(|point| *point.overlap())
        .collect::<Vec<_>>();
    let target = late_retrieval_deficit(&generated, site)?;
    if !target.is_finite() {
        return Err(RejectionReason::NonFiniteTarget);
    }
    let baseline = vec![
        generated.tokens.len() as f64,
        generated.distractors as f64,
        generated.retrieval_distance as f64,
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
    Ok(Record {
        generator_id: seed,
        baseline,
        recovery,
        target,
    })
}

/// Contiguous ascending enumeration of the entire frozen final seed range
/// (frozen selection rule contiguous_ascending_v1).
fn final_seeds() -> impl Iterator<Item = u64> {
    FINAL_SEED_START..=FINAL_SEED_END
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

#[derive(Clone, Copy, Debug)]
struct Summary {
    accepted: usize,
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
            "accepted={} b0_lambda={} b1_lambda={} b0_mse={} b1_mse={} relative_reduction={} lower_95={} upper_95={}",
            self.accepted,
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

/// Paired generator-level bootstrap: both intervention-site records of a seed
/// are resampled together, preserving the generator-example unit. Groups are
/// indexed in an ordered map so the 10000-generator holdout stays tractable.
fn evaluate(training: &[Record], development: &[Record], holdout: &[Record]) -> Summary {
    let b0 = select(training, development, false);
    let b1 = select(training, development, true);
    let b0_errors = squared_errors(&b0, holdout, false);
    let b1_errors = squared_errors(&b1, holdout, true);
    let b0_mse = b0_errors.iter().sum::<f64>() / b0_errors.len() as f64;
    let b1_mse = b1_errors.iter().sum::<f64>() / b1_errors.len() as f64;
    assert!(b1_mse.is_finite());
    let relative_reduction = (b0_mse - b1_mse) / b0_mse;

    let mut groups = std::collections::BTreeMap::<u64, Vec<usize>>::new();
    for (index, record) in holdout.iter().enumerate() {
        groups.entry(record.generator_id).or_default().push(index);
    }
    let group_list = groups.values().cloned().collect::<Vec<_>>();
    let mut rng = SplitMix64::new(BOOTSTRAP_SEED);
    let mut reductions = Vec::with_capacity(BOOTSTRAP_REPLICATES);
    for _ in 0..BOOTSTRAP_REPLICATES {
        let mut sum0 = 0.0;
        let mut sum1 = 0.0;
        let mut count = 0usize;
        for _ in 0..group_list.len() {
            let indices = &group_list[rng.bounded(group_list.len())];
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
        accepted: holdout.len(),
        b0_lambda: b0.lambda,
        b1_lambda: b1.lambda,
        b0_mse,
        b1_mse,
        relative_reduction,
        lower_95: percentile(&reductions, 0.025),
        upper_95: percentile(&reductions, 0.975),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Verdict {
    Beneficial,
    Harmful,
    Equivalent,
    Inconclusive,
}

impl Verdict {
    const fn label(self) -> &'static str {
        match self {
            Self::Beneficial => "Beneficial",
            Self::Harmful => "Harmful",
            Self::Equivalent => "Equivalent",
            Self::Inconclusive => "Inconclusive",
        }
    }
}

/// Frozen TDI-7.0 decision rule (preregistration section 12).
fn classify(summary: &Summary) -> Verdict {
    let r = summary.relative_reduction;
    let lower = summary.lower_95;
    let upper = summary.upper_95;
    if !r.is_finite() || !lower.is_finite() || !upper.is_finite() || lower > upper {
        return Verdict::Inconclusive;
    }
    if r >= RELEVANCE_MARGIN && lower > 0.0 {
        return Verdict::Beneficial;
    }
    if r <= -RELEVANCE_MARGIN && upper < 0.0 {
        return Verdict::Harmful;
    }
    if lower >= -RELEVANCE_MARGIN && upper <= RELEVANCE_MARGIN {
        return Verdict::Equivalent;
    }
    Verdict::Inconclusive
}

/// Frozen TDI-7.0 multi-task gate (preregistration section 13).
fn multi_task_gate(associative: Verdict, copy: Verdict) -> Verdict {
    if associative == Verdict::Harmful || copy == Verdict::Harmful {
        return Verdict::Harmful;
    }
    if associative == Verdict::Beneficial || copy == Verdict::Beneficial {
        return Verdict::Beneficial;
    }
    if associative == Verdict::Equivalent && copy == Verdict::Equivalent {
        return Verdict::Equivalent;
    }
    Verdict::Inconclusive
}

/// Build the training/development populations from the frozen bounded ranges
/// (same range constants as the frozen TDI-7.1 bounded evaluator). Any
/// deterministic construction failure here is a protocol-level defect and must
/// abort; these ranges were already exercised by TDI-7.1 with zero rejections.
fn bounded_population(
    start: u64,
    count: usize,
    kind: TaskKind,
) -> Result<Vec<Record>, RejectionReason> {
    let mut records = Vec::with_capacity(count * 2);
    for offset in 0..count {
        let seed = start + offset as u64;
        records.push(try_record(seed, kind, Site::Early)?);
        records.push(try_record(seed, kind, Site::Late)?);
    }
    Ok(records)
}

struct RunOutcome {
    summary: Summary,
    verdict: Verdict,
    rejected: Vec<(u64, RejectionReason)>,
}

fn run_holdout(kind: TaskKind) -> Result<RunOutcome, String> {
    let training = bounded_population(TRAIN_START, TRAIN_COUNT, kind).map_err(|reason| {
        format!(
            "training population rejected with reason {}",
            reason.label()
        )
    })?;
    let development = bounded_population(DEV_START, DEV_COUNT, kind).map_err(|reason| {
        format!(
            "development population rejected with reason {}",
            reason.label()
        )
    })?;

    let mut holdout_records = Vec::new();
    let mut rejected = Vec::new();
    for seed in final_seeds() {
        for site in [Site::Early, Site::Late] {
            match try_record(seed, kind, site) {
                Ok(record) => holdout_records.push(record),
                Err(reason) => rejected.push((seed, reason)),
            }
        }
    }
    let summary = evaluate(&training, &development, &holdout_records);
    let verdict = classify(&summary);
    Ok(RunOutcome {
        summary,
        verdict,
        rejected,
    })
}

fn provenance() -> String {
    let commit = env::var_os("TDI_REPORT_COMMIT")
        .and_then(|value| value.into_string().ok())
        .unwrap_or_else(|| "local-unreported".to_string());
    format!(
        "provenance_commit={commit}\n\
         semantic=deterministic_local_row_stochastic_v1\n\
         selection_rule={FROZEN_SELECTION_RULE}\n\
         final_seed_range={FINAL_SEED_START}-{FINAL_SEED_END}\n\
         rejection_policy={FROZEN_REJECTION_POLICY}\n\
         rejection_reasons={FROZEN_REJECTION_REASONS}\n\
         decision_reference={DECISION_REFERENCE}\n\
         final_holdout_status=ACCESSED_ONCE"
    )
}

fn main() {
    match authorization(env::var(CONFIRM_VAR).ok().as_deref()) {
        Ok(()) => {}
        Err(AuthorizationError::Missing) => {
            eprintln!(
                "BLOCKED: final holdout requires the human-supplied confirmation variable {CONFIRM_VAR} at execution time"
            );
            std::process::exit(3);
        }
        Err(AuthorizationError::Mismatch) => {
            eprintln!(
                "BLOCKED: supplied {CONFIRM_VAR} does not match the frozen confirmation value"
            );
            std::process::exit(3);
        }
    }
    if let Err(error) = validate_frozen_decisions() {
        eprintln!("BLOCKED: frozen decision validation failed: {error}");
        std::process::exit(3);
    }
    println!("TDI-7.2 final-holdout runner: AUTHORIZED (human confirmation)");
    println!("{}", provenance());

    let mut outcomes = Vec::new();
    for kind in [TaskKind::AssociativeRecall, TaskKind::Copy] {
        let outcome = match run_holdout(kind) {
            Ok(value) => value,
            Err(error) => {
                eprintln!("ERROR: {error}");
                std::process::exit(1);
            }
        };
        println!("task={}", kind.label());
        println!("{}", outcome.summary.report());
        println!("task_verdict={}", outcome.verdict.label());
        println!("rejected_count={}", outcome.rejected.len());
        for (seed, reason) in &outcome.rejected {
            println!("rejected seed={seed} reason={}", reason.label());
        }
        outcomes.push(outcome.verdict);
    }

    println!(
        "aggregate_verdict={}",
        multi_task_gate(outcomes[0], outcomes[1]).label()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authorization_requires_the_exact_human_value() {
        assert_eq!(authorization(None), Err(AuthorizationError::Missing));
        assert_eq!(
            authorization(Some("wrong-value")),
            Err(AuthorizationError::Mismatch)
        );
        assert_eq!(authorization(Some(CONFIRM_VALUE)), Ok(()));
        assert_eq!(CONFIRM_VAR, "TDI7_CONFIRM_FINAL_HOLDOUT");
    }

    #[test]
    fn final_seeds_cover_the_entire_frozen_range_contiguously() {
        let seeds = final_seeds().collect::<Vec<_>>();
        assert_eq!(seeds.len(), 10_000);
        assert_eq!(FINAL_SEED_COUNT, 10_000);
        assert_eq!(seeds.first(), Some(&FINAL_SEED_START));
        assert_eq!(seeds.last(), Some(&FINAL_SEED_END));
        for pair in seeds.windows(2) {
            assert_eq!(pair[1], pair[0] + 1);
        }
        assert!(TRAIN_START + TRAIN_COUNT as u64 <= DEV_START);
        assert!(DEV_START + DEV_COUNT as u64 <= VALIDATION_START);
        assert!(VALIDATION_START + VALIDATION_COUNT as u64 <= FINAL_SEED_START);
    }

    #[test]
    fn frozen_decision_constants_match_the_records() {
        assert_eq!(FROZEN_SELECTION_RULE, "contiguous_ascending_v1");
        assert_eq!(FROZEN_REJECTION_POLICY, "frozen_tdi71_typed_errors_v1");
        assert_eq!(
            FROZEN_REJECTION_REASONS,
            "invalid_mixer,invalid_intervention,recovery_extraction_failed,non_finite_target"
        );
        assert_eq!(
            DECISION_REFERENCE,
            "TDI-7.2-HUMAN-DECISION-2026-09-01-CHECKUPAUTO"
        );
        assert_eq!(FINAL_SEED_START, 7_100_030_000);
        assert_eq!(FINAL_SEED_END, 7_100_039_999);
    }

    #[test]
    fn rejected_reasons_exactly_match_the_frozen_taxonomy() {
        let labels = [
            RejectionReason::InvalidMixer.label(),
            RejectionReason::InvalidIntervention.label(),
            RejectionReason::RecoveryExtractionFailed.label(),
            RejectionReason::NonFiniteTarget.label(),
        ]
        .join(",");
        assert_eq!(labels, FROZEN_REJECTION_REASONS);
    }

    #[test]
    fn records_are_deterministic_finite_and_on_the_holdout_range() {
        for kind in [TaskKind::AssociativeRecall, TaskKind::Copy] {
            for site in [Site::Early, Site::Late] {
                let left = try_record(FINAL_SEED_START, kind, site).expect("constructible");
                let right = try_record(FINAL_SEED_START, kind, site).expect("constructible");
                assert_eq!(left.generator_id, right.generator_id);
                assert_eq!(left.baseline, right.baseline);
                assert_eq!(left.recovery, right.recovery);
                assert_eq!(left.target, right.target);
                assert!(left.target.is_finite() && left.target >= 0.0 && left.target < 1.0);
                assert_eq!(left.recovery.len(), EARLY_DEPTHS);
            }
        }
    }

    #[test]
    fn classify_follows_the_frozen_decision_rule() {
        let mut summary = Summary {
            accepted: 100,
            b0_lambda: 0.0,
            b1_lambda: 0.0,
            b0_mse: 1.0,
            b1_mse: 0.9,
            relative_reduction: 0.10,
            lower_95: 0.01,
            upper_95: 0.19,
        };
        assert_eq!(classify(&summary), Verdict::Beneficial);

        summary.relative_reduction = -0.10;
        summary.lower_95 = -0.19;
        summary.upper_95 = -0.01;
        assert_eq!(classify(&summary), Verdict::Harmful);

        summary.relative_reduction = 0.0;
        summary.lower_95 = -0.01;
        summary.upper_95 = 0.01;
        assert_eq!(classify(&summary), Verdict::Equivalent);

        summary.relative_reduction = 0.05;
        summary.lower_95 = -0.01;
        summary.upper_95 = 0.11;
        assert_eq!(classify(&summary), Verdict::Inconclusive);

        summary.lower_95 = f64::NAN;
        assert_eq!(classify(&summary), Verdict::Inconclusive);
    }

    #[test]
    fn multi_task_gate_matches_the_preregistration() {
        assert_eq!(
            multi_task_gate(Verdict::Beneficial, Verdict::Harmful),
            Verdict::Harmful
        );
        assert_eq!(
            multi_task_gate(Verdict::Equivalent, Verdict::Equivalent),
            Verdict::Equivalent
        );
        assert_eq!(
            multi_task_gate(Verdict::Beneficial, Verdict::Equivalent),
            Verdict::Beneficial
        );
        assert_eq!(
            multi_task_gate(Verdict::Inconclusive, Verdict::Equivalent),
            Verdict::Inconclusive
        );
        assert_eq!(
            multi_task_gate(Verdict::Inconclusive, Verdict::Inconclusive),
            Verdict::Inconclusive
        );
    }

    #[test]
    fn source_requires_environmental_human_authorization() {
        let source = include_str!("tdi7_final_holdout.rs");
        assert!(source.contains(CONFIRM_VAR));
        assert!(source.contains(CONFIRM_VALUE));
        assert!(source.contains("env::var(CONFIRM_VAR)"));
        assert!(source.contains("std::process::exit(3)"));
    }
}
