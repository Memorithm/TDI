//! Compact bounded Gaussian/MMI information decomposition for TDI-7.x.
//!
//! Generalizes the TDI-6.3 two-source Gaussian/MMI discipline to vector source
//! blocks. This module has no final-holdout execution or authorization surface.

pub const PID_TOLERANCE_BITS: f64 = 1.0e-9;
pub const PIVOT_FLOOR: f64 = 1.0e-12;
pub const RANK_RESIDUAL_SCALE: f64 = 1.0e-10;
pub const TDI74_BOOTSTRAP_REPLICATES: usize = 4_000;
pub const TDI74_BOOTSTRAP_SEED: u64 = 0x5444_4937_3400_4700;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InformationError {
    EmptyDataset,
    TooFewRecords,
    EmptyFeatureBlock,
    FeatureWidthMismatch,
    NonFiniteValue,
    ZeroSourceRank,
    DegenerateTarget,
    NonPositiveDefinite,
    SingularSystem,
    InvalidInformation,
    CrossMethodDisagreement,
    NegativePidComponent,
    PidIdentityFailure,
    InvalidBootstrap,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InformationRecord {
    generator_id: u64,
    static_block: Vec<f64>,
    recovery_block: Vec<f64>,
    target: f64,
}

impl InformationRecord {
    pub fn new(
        generator_id: u64,
        static_block: Vec<f64>,
        recovery_block: Vec<f64>,
        target: f64,
    ) -> Result<Self, InformationError> {
        if static_block.is_empty() || recovery_block.is_empty() {
            return Err(InformationError::EmptyFeatureBlock);
        }
        if !target.is_finite()
            || static_block.iter().chain(&recovery_block).any(|value| !value.is_finite())
        {
            return Err(InformationError::NonFiniteValue);
        }
        Ok(Self {
            generator_id,
            static_block,
            recovery_block,
            target,
        })
    }

    #[must_use]
    pub const fn generator_id(&self) -> u64 {
        self.generator_id
    }

    #[must_use]
    pub fn static_block(&self) -> &[f64] {
        &self.static_block
    }

    #[must_use]
    pub fn recovery_block(&self) -> &[f64] {
        &self.recovery_block
    }

    #[must_use]
    pub const fn target(&self) -> f64 {
        self.target
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MiEstimate {
    rank: usize,
    canonical_bits: f64,
    crosscheck_bits: f64,
    discrepancy_bits: f64,
}

impl MiEstimate {
    #[must_use]
    pub const fn rank(self) -> usize {
        self.rank
    }

    #[must_use]
    pub const fn canonical_bits(self) -> f64 {
        self.canonical_bits
    }

    #[must_use]
    pub const fn crosscheck_bits(self) -> f64 {
        self.crosscheck_bits
    }

    #[must_use]
    pub const fn discrepancy_bits(self) -> f64 {
        self.discrepancy_bits
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PidEstimate {
    static_mi: MiEstimate,
    recovery_mi: MiEstimate,
    joint_mi: MiEstimate,
    redundancy_bits: f64,
    unique_static_bits: f64,
    unique_recovery_bits: f64,
    synergy_bits: f64,
}

impl PidEstimate {
    #[must_use]
    pub const fn static_mi(self) -> MiEstimate {
        self.static_mi
    }

    #[must_use]
    pub const fn recovery_mi(self) -> MiEstimate {
        self.recovery_mi
    }

    #[must_use]
    pub const fn joint_mi(self) -> MiEstimate {
        self.joint_mi
    }

    #[must_use]
    pub const fn redundancy_bits(self) -> f64 {
        self.redundancy_bits
    }

    #[must_use]
    pub const fn unique_static_bits(self) -> f64 {
        self.unique_static_bits
    }

    #[must_use]
    pub const fn unique_recovery_bits(self) -> f64 {
        self.unique_recovery_bits
    }

    #[must_use]
    pub const fn synergy_bits(self) -> f64 {
        self.synergy_bits
    }

    #[must_use]
    pub fn canonical_report(self) -> String {
        format!(
            concat!(
                "static_rank={};recovery_rank={};joint_rank={};",
                "i_static_bits={:016x};i_recovery_bits={:016x};i_joint_bits={:016x};",
                "redundancy_bits={:016x};unique_static_bits={:016x};",
                "unique_recovery_bits={:016x};synergy_bits={:016x};",
                "max_crosscheck_discrepancy_bits={:016x}"
            ),
            self.static_mi.rank,
            self.recovery_mi.rank,
            self.joint_mi.rank,
            self.static_mi.canonical_bits.to_bits(),
            self.recovery_mi.canonical_bits.to_bits(),
            self.joint_mi.canonical_bits.to_bits(),
            self.redundancy_bits.to_bits(),
            self.unique_static_bits.to_bits(),
            self.unique_recovery_bits.to_bits(),
            self.synergy_bits.to_bits(),
            self.static_mi
                .discrepancy_bits
                .max(self.recovery_mi.discrepancy_bits)
                .max(self.joint_mi.discrepancy_bits)
                .to_bits(),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Interval {
    lower: f64,
    median: f64,
    upper: f64,
}

impl Interval {
    #[must_use]
    pub const fn lower(self) -> f64 {
        self.lower
    }

    #[must_use]
    pub const fn median(self) -> f64 {
        self.median
    }

    #[must_use]
    pub const fn upper(self) -> f64 {
        self.upper
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BootstrapSummary {
    point: PidEstimate,
    joint: Interval,
    redundancy: Interval,
    unique_static: Interval,
    unique_recovery: Interval,
    synergy: Interval,
    accepted: usize,
    rejected: usize,
}

impl BootstrapSummary {
    #[must_use]
    pub const fn point(self) -> PidEstimate {
        self.point
    }

    #[must_use]
    pub const fn joint(self) -> Interval {
        self.joint
    }

    #[must_use]
    pub const fn redundancy(self) -> Interval {
        self.redundancy
    }

    #[must_use]
    pub const fn unique_static(self) -> Interval {
        self.unique_static
    }

    #[must_use]
    pub const fn unique_recovery(self) -> Interval {
        self.unique_recovery
    }

    #[must_use]
    pub const fn synergy(self) -> Interval {
        self.synergy
    }

    #[must_use]
    pub const fn accepted_replicates(self) -> usize {
        self.accepted
    }

    #[must_use]
    pub const fn rejected_replicates(self) -> usize {
        self.rejected
    }

    #[must_use]
    pub fn canonical_report(self) -> String {
        format!(
            concat!(
                "{};accepted_replicates={};rejected_replicates={};",
                "joint_ci={:016x},{:016x},{:016x};",
                "redundancy_ci={:016x},{:016x},{:016x};",
                "unique_static_ci={:016x},{:016x},{:016x};",
                "unique_recovery_ci={:016x},{:016x},{:016x};",
                "synergy_ci={:016x},{:016x},{:016x}"
            ),
            self.point.canonical_report(),
            self.accepted,
            self.rejected,
            self.joint.lower.to_bits(),
            self.joint.median.to_bits(),
            self.joint.upper.to_bits(),
            self.redundancy.lower.to_bits(),
            self.redundancy.median.to_bits(),
            self.redundancy.upper.to_bits(),
            self.unique_static.lower.to_bits(),
            self.unique_static.median.to_bits(),
            self.unique_static.upper.to_bits(),
            self.unique_recovery.lower.to_bits(),
            self.unique_recovery.median.to_bits(),
            self.unique_recovery.upper.to_bits(),
            self.synergy.lower.to_bits(),
            self.synergy.median.to_bits(),
            self.synergy.upper.to_bits(),
        )
    }
}

#[derive(Clone, Debug)]
struct Reduced {
    rows: Vec<Vec<f64>>,
    rank: usize,
}

fn obs_width(rows: &[Vec<f64>]) -> Result<usize, InformationError> {
    let first = rows.first().ok_or(InformationError::EmptyDataset)?;
    if rows.len() < 3 {
        return Err(InformationError::TooFewRecords);
    }
    if first.is_empty() {
        return Err(InformationError::EmptyFeatureBlock);
    }
    let width = first.len();
    for row in rows {
        if row.len() != width {
            return Err(InformationError::FeatureWidthMismatch);
        }
        if row.iter().any(|value| !value.is_finite()) {
            return Err(InformationError::NonFiniteValue);
        }
    }
    Ok(width)
}

fn reduce(rows: &[Vec<f64>]) -> Result<Reduced, InformationError> {
    let width = obs_width(rows)?;
    let n = rows.len();
    let threshold = RANK_RESIDUAL_SCALE * (n as f64).sqrt();
    let mut kept = Vec::<Vec<f64>>::new();
    let mut basis = Vec::<Vec<f64>>::new();
    for column in 0..width {
        let mean = rows.iter().map(|row| row[column]).sum::<f64>() / n as f64;
        let mut standardized = rows
            .iter()
            .map(|row| row[column] - mean)
            .collect::<Vec<_>>();
        let rms = (standardized.iter().map(|value| value * value).sum::<f64>() / n as f64).sqrt();
        if !rms.is_finite() {
            return Err(InformationError::NonFiniteValue);
        }
        if rms == 0.0 {
            continue;
        }
        for value in &mut standardized {
            *value /= rms;
        }
        let original = standardized.clone();
        for direction in &basis {
            let projection = standardized
                .iter()
                .zip(direction)
                .map(|(left, right)| left * right)
                .sum::<f64>();
            for (value, direction_value) in standardized.iter_mut().zip(direction) {
                *value -= projection * direction_value;
            }
        }
        let norm = standardized
            .iter()
            .map(|value| value * value)
            .sum::<f64>()
            .sqrt();
        if !norm.is_finite() {
            return Err(InformationError::NonFiniteValue);
        }
        if norm > threshold {
            for value in &mut standardized {
                *value /= norm;
            }
            kept.push(original);
            basis.push(standardized);
        }
    }
    if kept.is_empty() {
        return Err(InformationError::ZeroSourceRank);
    }
    let rank = kept.len();
    let mut reduced_rows = vec![vec![0.0; rank]; n];
    for (column, values) in kept.iter().enumerate() {
        for (row, value) in values.iter().enumerate() {
            reduced_rows[row][column] = *value;
        }
    }
    Ok(Reduced {
        rows: reduced_rows,
        rank,
    })
}

fn covariance(rows: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, InformationError> {
    let width = obs_width(rows)?;
    let n = rows.len();
    let means = (0..width)
        .map(|column| rows.iter().map(|row| row[column]).sum::<f64>() / n as f64)
        .collect::<Vec<_>>();
    let mut out = vec![vec![0.0; width]; width];
    for i in 0..width {
        for j in 0..=i {
            let value = rows
                .iter()
                .map(|row| (row[i] - means[i]) * (row[j] - means[j]))
                .sum::<f64>()
                / (n - 1) as f64;
            if !value.is_finite() {
                return Err(InformationError::NonFiniteValue);
            }
            out[i][j] = value;
            out[j][i] = value;
        }
    }
    Ok(out)
}

fn target_variance(target: &[f64]) -> Result<f64, InformationError> {
    if target.len() < 3 || target.iter().any(|value| !value.is_finite()) {
        return Err(InformationError::TooFewRecords);
    }
    let mean = target.iter().sum::<f64>() / target.len() as f64;
    let value = target
        .iter()
        .map(|item| (item - mean).powi(2))
        .sum::<f64>()
        / (target.len() - 1) as f64;
    if !value.is_finite() {
        return Err(InformationError::NonFiniteValue);
    }
    if value <= PIVOT_FLOOR {
        return Err(InformationError::DegenerateTarget);
    }
    Ok(value)
}

fn cross_covariance(target: &[f64], source: &[Vec<f64>]) -> Result<Vec<f64>, InformationError> {
    let width = obs_width(source)?;
    if target.len() != source.len() {
        return Err(InformationError::FeatureWidthMismatch);
    }
    let target_mean = target.iter().sum::<f64>() / target.len() as f64;
    let means = (0..width)
        .map(|column| source.iter().map(|row| row[column]).sum::<f64>() / source.len() as f64)
        .collect::<Vec<_>>();
    let mut out = Vec::with_capacity(width);
    for column in 0..width {
        let value = target
            .iter()
            .zip(source)
            .map(|(target_value, row)| (target_value - target_mean) * (row[column] - means[column]))
            .sum::<f64>()
            / (target.len() - 1) as f64;
        if !value.is_finite() {
            return Err(InformationError::NonFiniteValue);
        }
        out.push(value);
    }
    Ok(out)
}

fn cholesky_log2_det(matrix: &[Vec<f64>]) -> Result<f64, InformationError> {
    let n = matrix.len();
    if n == 0 || matrix.iter().any(|row| row.len() != n) {
        return Err(InformationError::FeatureWidthMismatch);
    }
    let mut lower = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..=i {
            let sum = (0..j).map(|k| lower[i][k] * lower[j][k]).sum::<f64>();
            if i == j {
                let pivot = matrix[i][i] - sum;
                if !pivot.is_finite() || pivot <= PIVOT_FLOOR {
                    return Err(InformationError::NonPositiveDefinite);
                }
                lower[i][j] = pivot.sqrt();
            } else {
                lower[i][j] = (matrix[i][j] - sum) / lower[j][j];
                if !lower[i][j].is_finite() {
                    return Err(InformationError::NonPositiveDefinite);
                }
            }
        }
    }
    let value = 2.0 * (0..n).map(|i| lower[i][i].log2()).sum::<f64>();
    if value.is_finite() {
        Ok(value)
    } else {
        Err(InformationError::NonPositiveDefinite)
    }
}

fn solve(mut matrix: Vec<Vec<f64>>, mut rhs: Vec<f64>) -> Result<Vec<f64>, InformationError> {
    let n = rhs.len();
    if n == 0 || matrix.len() != n || matrix.iter().any(|row| row.len() != n) {
        return Err(InformationError::FeatureWidthMismatch);
    }
    for pivot in 0..n {
        let mut best = pivot;
        for row in (pivot + 1)..n {
            if matrix[row][pivot].abs() > matrix[best][pivot].abs() {
                best = row;
            }
        }
        if matrix[best][pivot].abs() <= PIVOT_FLOOR || !matrix[best][pivot].is_finite() {
            return Err(InformationError::SingularSystem);
        }
        matrix.swap(pivot, best);
        rhs.swap(pivot, best);
        let scale = matrix[pivot][pivot];
        for value in &mut matrix[pivot][pivot..] {
            *value /= scale;
        }
        rhs[pivot] /= scale;
        let pivot_row = matrix[pivot].clone();
        let pivot_rhs = rhs[pivot];
        for row in 0..n {
            if row == pivot {
                continue;
            }
            let factor = matrix[row][pivot];
            for (value, pivot_value) in matrix[row][pivot..].iter_mut().zip(&pivot_row[pivot..]) {
                *value -= factor * pivot_value;
            }
            rhs[row] -= factor * pivot_rhs;
        }
    }
    Ok(rhs)
}

fn normalize_mi(value: f64) -> Result<f64, InformationError> {
    if !value.is_finite() || value < -PID_TOLERANCE_BITS {
        return Err(InformationError::InvalidInformation);
    }
    Ok(value.max(0.0))
}

pub fn gaussian_mi(target: &[f64], source: &[Vec<f64>]) -> Result<MiEstimate, InformationError> {
    if target.len() != source.len() {
        return Err(InformationError::FeatureWidthMismatch);
    }
    let reduced = reduce(source)?;
    let var_t = target_variance(target)?;
    let cov_x = covariance(&reduced.rows)?;
    let source_logdet = cholesky_log2_det(&cov_x)?;
    let mut joint_rows = Vec::with_capacity(target.len());
    for (target_value, row) in target.iter().zip(&reduced.rows) {
        let mut joint = Vec::with_capacity(reduced.rank + 1);
        joint.push(*target_value);
        joint.extend_from_slice(row);
        joint_rows.push(joint);
    }
    let canonical = normalize_mi(
        0.5 * (var_t.log2() + source_logdet - cholesky_log2_det(&covariance(&joint_rows)?)?),
    )?;
    let c = cross_covariance(target, &reduced.rows)?;
    let beta = solve(cov_x, c.clone())?;
    let mut r2 = c.iter().zip(&beta).map(|(left, right)| left * right).sum::<f64>() / var_t;
    if r2 < 0.0 && r2 >= -PID_TOLERANCE_BITS {
        r2 = 0.0;
    }
    if !r2.is_finite() || r2 < 0.0 || 1.0 - r2 <= PIVOT_FLOOR {
        return Err(InformationError::InvalidInformation);
    }
    let crosscheck = normalize_mi(-0.5 * (1.0 - r2).log2())?;
    let discrepancy = (canonical - crosscheck).abs();
    if discrepancy > PID_TOLERANCE_BITS {
        return Err(InformationError::CrossMethodDisagreement);
    }
    Ok(MiEstimate {
        rank: reduced.rank,
        canonical_bits: canonical,
        crosscheck_bits: crosscheck,
        discrepancy_bits: discrepancy,
    })
}

fn nonnegative(value: f64) -> Result<f64, InformationError> {
    if value < -PID_TOLERANCE_BITS || !value.is_finite() {
        return Err(InformationError::NegativePidComponent);
    }
    Ok(value.max(0.0))
}

pub fn evaluate_pid(records: &[InformationRecord]) -> Result<PidEstimate, InformationError> {
    let first = records.first().ok_or(InformationError::EmptyDataset)?;
    if records.len() < 3 {
        return Err(InformationError::TooFewRecords);
    }
    for record in records {
        if record.static_block.len() != first.static_block.len()
            || record.recovery_block.len() != first.recovery_block.len()
        {
            return Err(InformationError::FeatureWidthMismatch);
        }
    }
    let target = records.iter().map(|record| record.target).collect::<Vec<_>>();
    let static_rows = records.iter().map(|record| record.static_block.clone()).collect::<Vec<_>>();
    let recovery_rows = records
        .iter()
        .map(|record| record.recovery_block.clone())
        .collect::<Vec<_>>();
    let joint_rows = records
        .iter()
        .map(|record| {
            let mut row = record.static_block.clone();
            row.extend_from_slice(&record.recovery_block);
            row
        })
        .collect::<Vec<_>>();
    let static_mi = gaussian_mi(&target, &static_rows)?;
    let recovery_mi = gaussian_mi(&target, &recovery_rows)?;
    let joint_mi = gaussian_mi(&target, &joint_rows)?;
    let redundancy = static_mi.canonical_bits.min(recovery_mi.canonical_bits);
    let unique_static = nonnegative(static_mi.canonical_bits - redundancy)?;
    let unique_recovery = nonnegative(recovery_mi.canonical_bits - redundancy)?;
    let synergy = nonnegative(
        joint_mi.canonical_bits - static_mi.canonical_bits - recovery_mi.canonical_bits + redundancy,
    )?;
    if (redundancy + unique_static + unique_recovery + synergy - joint_mi.canonical_bits).abs()
        > PID_TOLERANCE_BITS
    {
        return Err(InformationError::PidIdentityFailure);
    }
    Ok(PidEstimate {
        static_mi,
        recovery_mi,
        joint_mi,
        redundancy_bits: redundancy,
        unique_static_bits: unique_static,
        unique_recovery_bits: unique_recovery,
        synergy_bits: synergy,
    })
}

#[derive(Clone, Copy, Debug)]
struct SplitMix64(u64);

impl SplitMix64 {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
}

fn percentile(sorted: &[f64], p: f64) -> Result<f64, InformationError> {
    if sorted.is_empty() || !(0.0..=1.0).contains(&p) {
        return Err(InformationError::InvalidBootstrap);
    }
    let position = p * (sorted.len() - 1) as f64;
    let lo = position.floor() as usize;
    let hi = position.ceil() as usize;
    Ok(if lo == hi {
        sorted[lo]
    } else {
        let weight = position - lo as f64;
        sorted[lo] * (1.0 - weight) + sorted[hi] * weight
    })
}

fn interval(mut values: Vec<f64>) -> Result<Interval, InformationError> {
    values.sort_by(f64::total_cmp);
    Ok(Interval {
        lower: percentile(&values, 0.025)?,
        median: percentile(&values, 0.5)?,
        upper: percentile(&values, 0.975)?,
    })
}

fn expected_bootstrap_degeneracy(error: InformationError) -> bool {
    matches!(
        error,
        InformationError::ZeroSourceRank
            | InformationError::DegenerateTarget
            | InformationError::NonPositiveDefinite
            | InformationError::SingularSystem
            | InformationError::InvalidInformation
    )
}

pub fn bootstrap_pid(
    records: &[InformationRecord],
    replicates: usize,
    seed: u64,
) -> Result<BootstrapSummary, InformationError> {
    let point = evaluate_pid(records)?;
    if replicates < 2 {
        return Err(InformationError::InvalidBootstrap);
    }
    let mut groups = Vec::<(u64, Vec<usize>)>::new();
    for (index, record) in records.iter().enumerate() {
        if let Some((_, indices)) = groups.iter_mut().find(|(id, _)| *id == record.generator_id) {
            indices.push(index);
        } else {
            groups.push((record.generator_id, vec![index]));
        }
    }
    if groups.is_empty() {
        return Err(InformationError::InvalidBootstrap);
    }
    let mut rng = SplitMix64(seed);
    let mut samples = [Vec::with_capacity(replicates), Vec::with_capacity(replicates), Vec::with_capacity(replicates), Vec::with_capacity(replicates), Vec::with_capacity(replicates)];
    let mut rejected = 0usize;
    for _ in 0..replicates {
        let mut resampled = Vec::with_capacity(records.len());
        for _ in 0..groups.len() {
            let group = &groups[(rng.next() % groups.len() as u64) as usize].1;
            for &index in group {
                resampled.push(records[index].clone());
            }
        }
        match evaluate_pid(&resampled) {
            Ok(estimate) => {
                samples[0].push(estimate.joint_mi.canonical_bits);
                samples[1].push(estimate.redundancy_bits);
                samples[2].push(estimate.unique_static_bits);
                samples[3].push(estimate.unique_recovery_bits);
                samples[4].push(estimate.synergy_bits);
            }
            Err(error) if expected_bootstrap_degeneracy(error) => rejected += 1,
            Err(error) => return Err(error),
        }
    }
    let accepted = samples[0].len();
    if accepted < 2 || (accepted as f64) / (replicates as f64) < 0.95 {
        return Err(InformationError::InvalidBootstrap);
    }
    Ok(BootstrapSummary {
        point,
        joint: interval(std::mem::take(&mut samples[0]))?,
        redundancy: interval(std::mem::take(&mut samples[1]))?,
        unique_static: interval(std::mem::take(&mut samples[2]))?,
        unique_recovery: interval(std::mem::take(&mut samples[3]))?,
        synergy: interval(std::mem::take(&mut samples[4]))?,
        accepted,
        rejected,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(left: f64, right: f64, tolerance: f64) {
        assert!((left - right).abs() <= tolerance, "{left} != {right}");
    }

    fn records(generators: usize) -> Vec<InformationRecord> {
        let mut out = Vec::with_capacity(generators * 2);
        for generator in 0..generators {
            let x = generator as f64 - generators as f64 / 2.0;
            let z = ((generator * 7 + 3) % 17) as f64 - 8.0;
            let w = ((generator * 11 + 5) % 23) as f64 - 11.0;
            for site in 0..2 {
                out.push(
                    InformationRecord::new(
                        generator as u64,
                        vec![x, z, x + z, 2.0 * x, w, z - w],
                        vec![0.2 * x + 0.03 * w, 0.15 * z - 0.02 * w, (x / 9.0).sin(), (z / 7.0).cos()],
                        0.31 * x - 0.17 * z + 0.08 * w + if site == 0 { -0.35 } else { 0.55 },
                    )
                    .unwrap(),
                );
            }
        }
        out
    }

    #[test]
    fn one_dimensional_cholesky_is_supported() {
        close(cholesky_log2_det(&[vec![4.0]]).unwrap(), 2.0, 1.0e-12);
    }

    #[test]
    fn independent_scalar_source_has_zero_information() {
        let target = vec![-1.0, 1.0, -1.0, 1.0, -2.0, 2.0, -2.0, 2.0];
        let source = vec![vec![-1.0], vec![-1.0], vec![1.0], vec![1.0], vec![-2.0], vec![-2.0], vec![2.0], vec![2.0]];
        let estimate = gaussian_mi(&target, &source).unwrap();
        assert_eq!(estimate.rank(), 1);
        close(estimate.canonical_bits(), 0.0, 1.0e-12);
        close(estimate.crosscheck_bits(), 0.0, 1.0e-12);
    }

    #[test]
    fn duplicate_columns_preserve_information_after_rank_reduction() {
        let target = vec![-2.0, -1.0, 0.5, 1.0, 2.5, 4.0, 5.0, 7.0];
        let x = vec![-3.0, -1.5, -0.5, 0.5, 1.5, 2.0, 4.0, 5.0];
        let one = x.iter().map(|value| vec![*value]).collect::<Vec<_>>();
        let duplicates = x.iter().map(|value| vec![*value, *value, 2.0 * *value]).collect::<Vec<_>>();
        let left = gaussian_mi(&target, &one).unwrap();
        let right = gaussian_mi(&target, &duplicates).unwrap();
        assert_eq!(right.rank(), 1);
        close(left.canonical_bits(), right.canonical_bits(), 1.0e-12);
    }

    #[test]
    fn vector_pid_identity_and_two_methods_hold() {
        let estimate = evaluate_pid(&records(48)).unwrap();
        close(
            estimate.redundancy_bits()
                + estimate.unique_static_bits()
                + estimate.unique_recovery_bits()
                + estimate.synergy_bits(),
            estimate.joint_mi().canonical_bits(),
            PID_TOLERANCE_BITS,
        );
        for mi in [estimate.static_mi(), estimate.recovery_mi(), estimate.joint_mi()] {
            assert!(mi.discrepancy_bits() <= PID_TOLERANCE_BITS);
        }
    }

    #[test]
    fn grouped_bootstrap_is_deterministic() {
        let data = records(40);
        let left = bootstrap_pid(&data, 128, 0x1234_5678_9abc_def0).unwrap();
        let right = bootstrap_pid(&data, 128, 0x1234_5678_9abc_def0).unwrap();
        assert_eq!(left, right);
        assert_eq!(left.accepted_replicates() + left.rejected_replicates(), 128);
        assert!(left.accepted_replicates() >= 122);
    }

    #[test]
    fn constant_source_fails_closed() {
        assert_eq!(
            gaussian_mi(&[0.0, 1.0, 2.0, 3.0], &vec![vec![1.0, 2.0]; 4]),
            Err(InformationError::ZeroSourceRank)
        );
    }

    #[test]
    fn source_has_no_final_holdout_authorization_secret() {
        let source = include_str!("gaussian_mmi_v7_impl.rs");
        let confirmation = ["I_ACCEPT_THE_TDI7_", "HOLDOUT_FREEZE"].concat();
        let environment = ["TDI7_CONFIRM_FINAL_", "HOLDOUT"].concat();
        assert!(!source.contains(&confirmation));
        assert!(!source.contains(&environment));
    }
}
