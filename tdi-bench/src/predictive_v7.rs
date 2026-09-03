//! Reusable non-holdout predictive evaluation machinery for TDI-7.x.
//!
//! This module factors the frozen TDI-7.1 nested ridge-model and generator-level
//! paired-bootstrap discipline into one audited implementation. Follow-up stages
//! can vary their evidence block while retaining identical model selection,
//! validation, uncertainty, and provenance semantics.

use crate::attention_v7::{
    DeterministicLocalMixer, InterventionSite, MechanisticState, SingleSiteIntervention, TaskKind,
    generate_task, late_retrieval_deficit, recovery_trajectory,
};

pub const TDI71_LAMBDA_GRID: [f64; 4] = [0.0, 1.0e-6, 1.0e-3, 1.0e-1];
pub const TDI71_BOOTSTRAP_REPLICATES: usize = 2_000;
pub const TDI71_BOOTSTRAP_SEED: u64 = 0x5444_4937_4532_4501;
pub const TDI71_EARLY_DEPTHS: usize = 2;
pub const TDI71_TARGET_DEPTH: usize = 5;
pub const TDI71_INTERVENTION_AMPLITUDE: f64 = 0.25;

const PIVOT_TOLERANCE: f64 = 1.0e-12;
const ROW_SUM_TOLERANCE: f64 = 1.0e-12;

#[derive(Clone, Debug, PartialEq)]
pub struct PredictiveRecord {
    generator_id: u64,
    baseline: Vec<f64>,
    evidence: Vec<f64>,
    target: f64,
}

impl PredictiveRecord {
    pub fn new(
        generator_id: u64,
        baseline: Vec<f64>,
        evidence: Vec<f64>,
        target: f64,
    ) -> Result<Self, PredictiveError> {
        if baseline.is_empty() || evidence.is_empty() {
            return Err(PredictiveError::EmptyFeatureBlock);
        }
        if !target.is_finite()
            || baseline.iter().any(|value| !value.is_finite())
            || evidence.iter().any(|value| !value.is_finite())
        {
            return Err(PredictiveError::NonFiniteValue);
        }
        Ok(Self {
            generator_id,
            baseline,
            evidence,
            target,
        })
    }

    #[must_use]
    pub const fn generator_id(&self) -> u64 {
        self.generator_id
    }

    #[must_use]
    pub fn baseline(&self) -> &[f64] {
        &self.baseline
    }

    #[must_use]
    pub fn evidence(&self) -> &[f64] {
        &self.evidence
    }

    #[must_use]
    pub const fn target(&self) -> f64 {
        self.target
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PredictiveError {
    EmptyDataset,
    EmptyFeatureBlock,
    FeatureWidthMismatch,
    NonFiniteValue,
    SingularSystem,
    InvalidBootstrap,
    DegenerateBaseline,
    InvalidHorizon,
    InterventionFailure,
    DynamicsFailure,
    StaticDiagnosticFailure,
    SeedOverflow,
}

#[derive(Clone, Debug, PartialEq)]
struct RidgeModel {
    weights: Vec<f64>,
    lambda: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NestedSummary {
    b0_lambda: f64,
    b1_lambda: f64,
    b0_mse: f64,
    b1_mse: f64,
    relative_reduction: f64,
    lower_95: f64,
    upper_95: f64,
    validation_records: usize,
    validation_generators: usize,
}

impl NestedSummary {
    #[must_use]
    pub const fn b0_lambda(self) -> f64 {
        self.b0_lambda
    }

    #[must_use]
    pub const fn b1_lambda(self) -> f64 {
        self.b1_lambda
    }

    #[must_use]
    pub const fn b0_mse(self) -> f64 {
        self.b0_mse
    }

    #[must_use]
    pub const fn b1_mse(self) -> f64 {
        self.b1_mse
    }

    #[must_use]
    pub const fn relative_reduction(self) -> f64 {
        self.relative_reduction
    }

    #[must_use]
    pub const fn lower_95(self) -> f64 {
        self.lower_95
    }

    #[must_use]
    pub const fn upper_95(self) -> f64 {
        self.upper_95
    }

    #[must_use]
    pub const fn validation_records(self) -> usize {
        self.validation_records
    }

    #[must_use]
    pub const fn validation_generators(self) -> usize {
        self.validation_generators
    }

    #[must_use]
    pub fn canonical_report(self) -> String {
        format!(
            concat!(
                "b0_lambda_bits={:016x};b1_lambda_bits={:016x};",
                "b0_mse_bits={:016x};b1_mse_bits={:016x};",
                "relative_reduction_bits={:016x};lower_95_bits={:016x};",
                "upper_95_bits={:016x};validation_records={};validation_generators={}"
            ),
            self.b0_lambda.to_bits(),
            self.b1_lambda.to_bits(),
            self.b0_mse.to_bits(),
            self.b1_mse.to_bits(),
            self.relative_reduction.to_bits(),
            self.lower_95.to_bits(),
            self.upper_95.to_bits(),
            self.validation_records,
            self.validation_generators,
        )
    }
}

#[derive(Clone, Copy, Debug)]
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn bounded(&mut self, upper: usize) -> Result<usize, PredictiveError> {
        if upper == 0 {
            return Err(PredictiveError::InvalidBootstrap);
        }
        Ok((self.next_u64() % upper as u64) as usize)
    }
}

/// Static controls frozen by TDI-7.1, evaluated locally so the historical
/// `tdi-bench` dependency graph remains unchanged. The six returned fields are,
/// in order: mean entropy, normalized entropy, max weight, L2 concentration,
/// entropy-derived support, and Frobenius norm.
fn static_controls(weights: &[Vec<f64>]) -> Result<[f64; 6], PredictiveError> {
    let first = weights
        .first()
        .ok_or(PredictiveError::StaticDiagnosticFailure)?;
    if first.is_empty() {
        return Err(PredictiveError::StaticDiagnosticFailure);
    }
    let columns = first.len();
    let entropy_normalizer = (columns > 1).then(|| (columns as f64).ln());
    let mut entropy_sum = 0.0;
    let mut normalized_entropy_sum = 0.0;
    let mut max_weight_sum = 0.0;
    let mut l2_sum = 0.0;
    let mut support_sum = 0.0;
    let mut squared_weight_sum = 0.0;

    for row in weights {
        if row.len() != columns || row.is_empty() {
            return Err(PredictiveError::StaticDiagnosticFailure);
        }
        let mut row_sum = 0.0;
        let mut entropy = 0.0;
        let mut row_max = 0.0_f64;
        let mut row_l2 = 0.0;
        for &weight in row {
            if !weight.is_finite() || weight < 0.0 {
                return Err(PredictiveError::StaticDiagnosticFailure);
            }
            row_sum += weight;
            row_max = row_max.max(weight);
            let square = weight * weight;
            row_l2 += square;
            squared_weight_sum += square;
            if weight > 0.0 {
                entropy -= weight * weight.ln();
            }
        }
        if !row_sum.is_finite() || (row_sum - 1.0).abs() > ROW_SUM_TOLERANCE {
            return Err(PredictiveError::StaticDiagnosticFailure);
        }
        entropy_sum += entropy;
        normalized_entropy_sum += entropy_normalizer.map_or(0.0, |value| entropy / value);
        max_weight_sum += row_max;
        l2_sum += row_l2;
        support_sum += entropy.exp();
    }

    let count = weights.len() as f64;
    let controls = [
        entropy_sum / count,
        normalized_entropy_sum / count,
        max_weight_sum / count,
        l2_sum / count,
        support_sum / count,
        squared_weight_sum.sqrt(),
    ];
    if controls.iter().any(|value| !value.is_finite()) {
        return Err(PredictiveError::StaticDiagnosticFailure);
    }
    Ok(controls)
}

fn validate_dataset(records: &[PredictiveRecord]) -> Result<(usize, usize), PredictiveError> {
    let first = records.first().ok_or(PredictiveError::EmptyDataset)?;
    let baseline_width = first.baseline.len();
    let evidence_width = first.evidence.len();
    if baseline_width == 0 || evidence_width == 0 {
        return Err(PredictiveError::EmptyFeatureBlock);
    }
    for record in records {
        if record.baseline.len() != baseline_width || record.evidence.len() != evidence_width {
            return Err(PredictiveError::FeatureWidthMismatch);
        }
        if !record.target.is_finite()
            || record.baseline.iter().any(|value| !value.is_finite())
            || record.evidence.iter().any(|value| !value.is_finite())
        {
            return Err(PredictiveError::NonFiniteValue);
        }
    }
    Ok((baseline_width, evidence_width))
}

fn validate_compatible(
    training: &[PredictiveRecord],
    development: &[PredictiveRecord],
    validation: &[PredictiveRecord],
) -> Result<(), PredictiveError> {
    let train_widths = validate_dataset(training)?;
    let dev_widths = validate_dataset(development)?;
    let validation_widths = validate_dataset(validation)?;
    if train_widths != dev_widths || train_widths != validation_widths {
        return Err(PredictiveError::FeatureWidthMismatch);
    }
    Ok(())
}

fn design(record: &PredictiveRecord, augmented: bool) -> Vec<f64> {
    let mut row = Vec::with_capacity(1 + record.baseline.len() + record.evidence.len());
    row.push(1.0);
    row.extend_from_slice(&record.baseline);
    if augmented {
        row.extend_from_slice(&record.evidence);
    }
    row
}

fn solve_linear(mut a: Vec<Vec<f64>>, mut b: Vec<f64>) -> Result<Vec<f64>, PredictiveError> {
    let n = b.len();
    if n == 0 || a.len() != n || a.iter().any(|row| row.len() != n) {
        return Err(PredictiveError::FeatureWidthMismatch);
    }
    for pivot in 0..n {
        let mut best = pivot;
        for row in (pivot + 1)..n {
            if a[row][pivot].abs() > a[best][pivot].abs() {
                best = row;
            }
        }
        if !a[best][pivot].is_finite() || a[best][pivot].abs() <= PIVOT_TOLERANCE {
            return Err(PredictiveError::SingularSystem);
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
    if b.iter().any(|value| !value.is_finite()) {
        return Err(PredictiveError::NonFiniteValue);
    }
    Ok(b)
}

fn fit(
    records: &[PredictiveRecord],
    augmented: bool,
    lambda: f64,
) -> Result<RidgeModel, PredictiveError> {
    validate_dataset(records)?;
    if !lambda.is_finite() || lambda < 0.0 {
        return Err(PredictiveError::NonFiniteValue);
    }
    let width = design(&records[0], augmented).len();
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
    Ok(RidgeModel {
        weights: solve_linear(gram, rhs)?,
        lambda,
    })
}

fn predict(model: &RidgeModel, record: &PredictiveRecord, augmented: bool) -> f64 {
    design(record, augmented)
        .iter()
        .zip(&model.weights)
        .map(|(feature, weight)| feature * weight)
        .sum()
}

fn squared_errors(
    model: &RidgeModel,
    records: &[PredictiveRecord],
    augmented: bool,
) -> Result<Vec<f64>, PredictiveError> {
    validate_dataset(records)?;
    let mut errors = Vec::with_capacity(records.len());
    for record in records {
        let error = predict(model, record, augmented) - record.target;
        let squared = error * error;
        if !squared.is_finite() {
            return Err(PredictiveError::NonFiniteValue);
        }
        errors.push(squared);
    }
    Ok(errors)
}

fn mse(
    model: &RidgeModel,
    records: &[PredictiveRecord],
    augmented: bool,
) -> Result<f64, PredictiveError> {
    let errors = squared_errors(model, records, augmented)?;
    Ok(errors.iter().sum::<f64>() / errors.len() as f64)
}

fn select_model(
    training: &[PredictiveRecord],
    development: &[PredictiveRecord],
    augmented: bool,
) -> Result<RidgeModel, PredictiveError> {
    let mut best: Option<(f64, RidgeModel)> = None;
    for lambda in TDI71_LAMBDA_GRID {
        let Ok(model) = fit(training, augmented, lambda) else {
            continue;
        };
        let loss = mse(&model, development, augmented)?;
        if !loss.is_finite() {
            return Err(PredictiveError::NonFiniteValue);
        }
        if best
            .as_ref()
            .is_none_or(|(best_loss, _)| loss.total_cmp(best_loss).is_lt())
        {
            best = Some((loss, model));
        }
    }
    best.map(|(_, model)| model)
        .ok_or(PredictiveError::SingularSystem)
}

fn generator_groups(records: &[PredictiveRecord]) -> Result<Vec<Vec<usize>>, PredictiveError> {
    validate_dataset(records)?;
    let mut groups = Vec::<(u64, Vec<usize>)>::new();
    for (index, record) in records.iter().enumerate() {
        if let Some((_, indices)) = groups
            .iter_mut()
            .find(|(generator_id, _)| *generator_id == record.generator_id)
        {
            indices.push(index);
        } else {
            groups.push((record.generator_id, vec![index]));
        }
    }
    if groups.is_empty() || groups.iter().any(|(_, indices)| indices.is_empty()) {
        return Err(PredictiveError::InvalidBootstrap);
    }
    Ok(groups.into_iter().map(|(_, indices)| indices).collect())
}

fn percentile(sorted: &[f64], probability: f64) -> Result<f64, PredictiveError> {
    if sorted.is_empty() || !(0.0..=1.0).contains(&probability) {
        return Err(PredictiveError::InvalidBootstrap);
    }
    let position = probability * (sorted.len() - 1) as f64;
    let lo = position.floor() as usize;
    let hi = position.ceil() as usize;
    let value = if lo == hi {
        sorted[lo]
    } else {
        let weight = position - lo as f64;
        sorted[lo] * (1.0 - weight) + sorted[hi] * weight
    };
    if value.is_finite() {
        Ok(value)
    } else {
        Err(PredictiveError::NonFiniteValue)
    }
}

pub fn evaluate_nested_ridge(
    training: &[PredictiveRecord],
    development: &[PredictiveRecord],
    validation: &[PredictiveRecord],
    bootstrap_replicates: usize,
    bootstrap_seed: u64,
) -> Result<NestedSummary, PredictiveError> {
    validate_compatible(training, development, validation)?;
    if bootstrap_replicates < 2 {
        return Err(PredictiveError::InvalidBootstrap);
    }

    let b0 = select_model(training, development, false)?;
    let b1 = select_model(training, development, true)?;
    let b0_errors = squared_errors(&b0, validation, false)?;
    let b1_errors = squared_errors(&b1, validation, true)?;
    let b0_mse = b0_errors.iter().sum::<f64>() / b0_errors.len() as f64;
    let b1_mse = b1_errors.iter().sum::<f64>() / b1_errors.len() as f64;
    if !b0_mse.is_finite() || !b1_mse.is_finite() {
        return Err(PredictiveError::NonFiniteValue);
    }
    if b0_mse <= 0.0 {
        return Err(PredictiveError::DegenerateBaseline);
    }
    let relative_reduction = (b0_mse - b1_mse) / b0_mse;

    let groups = generator_groups(validation)?;
    let mut rng = SplitMix64::new(bootstrap_seed);
    let mut reductions = Vec::with_capacity(bootstrap_replicates);
    for _ in 0..bootstrap_replicates {
        let mut sum0 = 0.0;
        let mut sum1 = 0.0;
        let mut count = 0usize;
        for _ in 0..groups.len() {
            let group = &groups[rng.bounded(groups.len())?];
            for &index in group {
                sum0 += b0_errors[index];
                sum1 += b1_errors[index];
                count = count
                    .checked_add(1)
                    .ok_or(PredictiveError::InvalidBootstrap)?;
            }
        }
        if count == 0 {
            return Err(PredictiveError::InvalidBootstrap);
        }
        let mean0 = sum0 / count as f64;
        let mean1 = sum1 / count as f64;
        if !mean0.is_finite() || !mean1.is_finite() || mean0 <= 0.0 {
            return Err(PredictiveError::DegenerateBaseline);
        }
        reductions.push((mean0 - mean1) / mean0);
    }
    reductions.sort_by(f64::total_cmp);

    Ok(NestedSummary {
        b0_lambda: b0.lambda,
        b1_lambda: b1.lambda,
        b0_mse,
        b1_mse,
        relative_reduction,
        lower_95: percentile(&reductions, 0.025)?,
        upper_95: percentile(&reductions, 0.975)?,
        validation_records: validation.len(),
        validation_generators: groups.len(),
    })
}

pub fn attention_record(
    seed: u64,
    kind: TaskKind,
    site: InterventionSite,
    early_depths: usize,
    target_depth: usize,
    amplitude: f64,
) -> Result<PredictiveRecord, PredictiveError> {
    if early_depths == 0 || target_depth <= early_depths {
        return Err(PredictiveError::InvalidHorizon);
    }
    let task = generate_task(kind, seed);
    let mixer = DeterministicLocalMixer::from_task(&task);
    let static_diag = static_controls(mixer.matrix())?;
    let reference = MechanisticState::from_task(&task);
    let perturbed = SingleSiteIntervention::new(site, amplitude)
        .apply(&reference)
        .map_err(|_| PredictiveError::InterventionFailure)?;
    let recovery = recovery_trajectory(&mixer, &reference, &perturbed, early_depths)
        .map_err(|_| PredictiveError::DynamicsFailure)?;
    let target = late_retrieval_deficit(&task, site, target_depth, amplitude)
        .map_err(|_| PredictiveError::DynamicsFailure)?;

    let baseline = vec![
        task.input().len() as f64,
        task.distractor_count() as f64,
        task.retrieval_distance() as f64,
        static_diag[0],
        static_diag[1],
        static_diag[2],
        static_diag[3],
        static_diag[4],
        static_diag[5],
        match site {
            InterventionSite::EarlyToken => 0.0,
            InterventionSite::LateToken => 1.0,
        },
    ];

    PredictiveRecord::new(seed, baseline, recovery, target)
}

pub fn attention_population(
    start: u64,
    generator_count: usize,
    kind: TaskKind,
    early_depths: usize,
    target_depth: usize,
    amplitude: f64,
) -> Result<Vec<PredictiveRecord>, PredictiveError> {
    if generator_count == 0 {
        return Err(PredictiveError::EmptyDataset);
    }
    let mut records = Vec::with_capacity(generator_count.saturating_mul(2));
    for offset in 0..generator_count {
        let offset = u64::try_from(offset).map_err(|_| PredictiveError::SeedOverflow)?;
        let seed = start
            .checked_add(offset)
            .ok_or(PredictiveError::SeedOverflow)?;
        records.push(attention_record(
            seed,
            kind,
            InterventionSite::EarlyToken,
            early_depths,
            target_depth,
            amplitude,
        )?);
        records.push(attention_record(
            seed,
            kind,
            InterventionSite::LateToken,
            early_depths,
            target_depth,
            amplitude,
        )?);
    }
    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TRAIN_START: u64 = 7_100_000_000;
    const DEV_START: u64 = 7_100_010_000;
    const VALIDATION_START: u64 = 7_100_020_000;

    fn assert_close(actual: f64, expected: f64) {
        assert!((actual - expected).abs() <= 1.0e-12, "{actual} != {expected}");
    }

    #[test]
    fn local_static_controls_match_known_row_stochastic_oracles() {
        let identity = static_controls(&[vec![1.0, 0.0], vec![0.0, 1.0]]).unwrap();
        assert_close(identity[0], 0.0);
        assert_close(identity[1], 0.0);
        assert_close(identity[2], 1.0);
        assert_close(identity[3], 1.0);
        assert_close(identity[4], 1.0);
        assert_close(identity[5], 2.0_f64.sqrt());

        let uniform = static_controls(&[vec![0.5, 0.5], vec![0.5, 0.5]]).unwrap();
        assert_close(uniform[0], 2.0_f64.ln());
        assert_close(uniform[1], 1.0);
        assert_close(uniform[2], 0.5);
        assert_close(uniform[3], 0.5);
        assert_close(uniform[4], 2.0);
        assert_close(uniform[5], 1.0);
    }

    #[test]
    fn malformed_static_controls_fail_closed() {
        assert_eq!(static_controls(&[]), Err(PredictiveError::StaticDiagnosticFailure));
        assert_eq!(
            static_controls(&[vec![0.4, 0.4]]),
            Err(PredictiveError::StaticDiagnosticFailure)
        );
        assert_eq!(
            static_controls(&[vec![1.25, -0.25]]),
            Err(PredictiveError::StaticDiagnosticFailure)
        );
    }

    #[test]
    fn protocol_record_has_frozen_static_and_recovery_widths() {
        let record = attention_record(
            TRAIN_START,
            TaskKind::AssociativeRecall,
            InterventionSite::EarlyToken,
            TDI71_EARLY_DEPTHS,
            TDI71_TARGET_DEPTH,
            TDI71_INTERVENTION_AMPLITUDE,
        )
        .unwrap();
        assert_eq!(record.baseline().len(), 10);
        assert_eq!(record.evidence().len(), 2);
        assert!((0.0..1.0).contains(&record.target()));
    }

    #[test]
    fn protocol_record_is_deterministic() {
        let left = attention_record(
            TRAIN_START,
            TaskKind::Copy,
            InterventionSite::LateToken,
            2,
            5,
            0.25,
        )
        .unwrap();
        let right = attention_record(
            TRAIN_START,
            TaskKind::Copy,
            InterventionSite::LateToken,
            2,
            5,
            0.25,
        )
        .unwrap();
        assert_eq!(left, right);
    }

    #[test]
    fn generator_grouping_keeps_both_sites_together() {
        let records = attention_population(TRAIN_START, 5, TaskKind::Copy, 2, 5, 0.25).unwrap();
        let groups = generator_groups(&records).unwrap();
        assert_eq!(groups.len(), 5);
        assert!(groups.iter().all(|group| group.len() == 2));
    }

    #[test]
    fn nested_evaluation_is_deterministic() {
        let training =
            attention_population(TRAIN_START, 24, TaskKind::AssociativeRecall, 2, 5, 0.25).unwrap();
        let development =
            attention_population(DEV_START, 16, TaskKind::AssociativeRecall, 2, 5, 0.25).unwrap();
        let validation = attention_population(
            VALIDATION_START,
            16,
            TaskKind::AssociativeRecall,
            2,
            5,
            0.25,
        )
        .unwrap();
        let left = evaluate_nested_ridge(&training, &development, &validation, 128, 1234).unwrap();
        let right = evaluate_nested_ridge(&training, &development, &validation, 128, 1234).unwrap();
        assert_eq!(left, right);
        assert_eq!(left.validation_records(), 32);
        assert_eq!(left.validation_generators(), 16);
        assert!(left.b0_mse().is_finite());
        assert!(left.b1_mse().is_finite());
        assert!(left.lower_95().is_finite());
        assert!(left.upper_95().is_finite());
    }

    #[test]
    fn malformed_feature_width_fails_closed() {
        let good = PredictiveRecord::new(1, vec![1.0], vec![2.0], 0.5).unwrap();
        let bad = PredictiveRecord::new(2, vec![1.0, 2.0], vec![2.0], 0.5).unwrap();
        assert_eq!(
            validate_dataset(&[good, bad]),
            Err(PredictiveError::FeatureWidthMismatch)
        );
    }

    #[test]
    fn invalid_horizon_fails_closed() {
        assert_eq!(
            attention_record(
                TRAIN_START,
                TaskKind::Copy,
                InterventionSite::EarlyToken,
                5,
                5,
                0.25,
            ),
            Err(PredictiveError::InvalidHorizon)
        );
    }

    #[test]
    fn source_has_no_final_holdout_authorization_secret() {
        let source = include_str!("predictive_v7.rs");
        let confirmation = ["I_ACCEPT_THE_TDI7_", "HOLDOUT_FREEZE"].concat();
        let environment = ["TDI7_CONFIRM_FINAL_", "HOLDOUT"].concat();
        assert!(!source.contains(&confirmation));
        assert!(!source.contains(&environment));
    }
}
