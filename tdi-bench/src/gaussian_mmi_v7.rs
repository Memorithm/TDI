//! Deterministic Gaussian/MMI information-decomposition machinery for TDI-7.x.
//!
//! This module generalizes the numerical discipline exercised by TDI-6.3 from
//! two scalar sources to finite vector source blocks. It is a bounded research
//! utility only: it contains no final-holdout authorization path and emits no
//! confirmatory verdict.

pub const PID_CROSS_METHOD_TOLERANCE_BITS: f64 = 1.0e-9;
pub const PID_DEGENERACY_PIVOT_FLOOR: f64 = 1.0e-12;
pub const PID_RANK_RESIDUAL_SCALE: f64 = 1.0e-10;
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
            || static_block.iter().any(|value| !value.is_finite())
            || recovery_block.iter().any(|value| !value.is_finite())
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
pub struct MutualInformationEstimate {
    rank: usize,
    canonical_bits: f64,
    crosscheck_bits: f64,
    discrepancy_bits: f64,
}

impl MutualInformationEstimate {
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
    static_mi: MutualInformationEstimate,
    recovery_mi: MutualInformationEstimate,
    joint_mi: MutualInformationEstimate,
    redundancy_bits: f64,
    unique_static_bits: f64,
    unique_recovery_bits: f64,
    synergy_bits: f64,
}

impl PidEstimate {
    #[must_use]
    pub const fn static_mi(self) -> MutualInformationEstimate {
        self.static_mi
    }

    #[must_use]
    pub const fn recovery_mi(self) -> MutualInformationEstimate {
        self.recovery_mi
    }

    #[must_use]
    pub const fn joint_mi(self) -> MutualInformationEstimate {
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
pub struct BootstrapInterval {
    lower: f64,
    median: f64,
    upper: f64,
}

impl BootstrapInterval {
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
pub struct PidBootstrapSummary {
    point: PidEstimate,
    joint: BootstrapInterval,
    redundancy: BootstrapInterval,
    unique_static: BootstrapInterval,
    unique_recovery: BootstrapInterval,
    synergy: BootstrapInterval,
    accepted_replicates: usize,
    rejected_replicates: usize,
}

impl PidBootstrapSummary {
    #[must_use]
    pub const fn point(self) -> PidEstimate {
        self.point
    }

    #[must_use]
    pub const fn joint(self) -> BootstrapInterval {
        self.joint
    }

    #[must_use]
    pub const fn redundancy(self) -> BootstrapInterval {
        self.redundancy
    }

    #[must_use]
    pub const fn unique_static(self) -> BootstrapInterval {
        self.unique_static
    }

    #[must_use]
    pub const fn unique_recovery(self) -> BootstrapInterval {
        self.unique_recovery
    }

    #[must_use]
    pub const fn synergy(self) -> BootstrapInterval {
        self.synergy
    }

    #[must_use]
    pub const fn accepted_replicates(self) -> usize {
        self.accepted_replicates
    }

    #[must_use]
    pub const fn rejected_replicates(self) -> usize {
        self.rejected_replicates
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
            self.accepted_replicates,
            self.rejected_replicates,
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
struct ReducedBlock {
    rows: Vec<Vec<f64>>,
    retained_columns: Vec<usize>,
}

impl ReducedBlock {
    fn rank(&self) -> usize {
        self.retained_columns.len()
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

    fn bounded(&mut self, upper: usize) -> Result<usize, InformationError> {
        if upper == 0 {
            return Err(InformationError::InvalidBootstrap);
        }
        Ok((self.next_u64() % upper as u64) as usize)
    }
}

fn validate_matrix(rows: &[Vec<f64>]) -> Result<usize, InformationError> {
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

fn validate_records(records: &[InformationRecord]) -> Result<(usize, usize), InformationError> {
    let first = records.first().ok_or(InformationError::EmptyDataset)?;
    if records.len() < 3 {
        return Err(InformationError::TooFewRecords);
    }
    let static_width = first.static_block.len();
    let recovery_width = first.recovery_block.len();
    if static_width == 0 || recovery_width == 0 {
        return Err(InformationError::EmptyFeatureBlock);
    }
    for record in records {
        if record.static_block.len() != static_width
            || record.recovery_block.len() != recovery_width
        {
            return Err(InformationError::FeatureWidthMismatch);
        }
        if !record.target.is_finite()
            || record.static_block.iter().any(|value| !value.is_finite())
            || record.recovery_block.iter().any(|value| !value.is_finite())
        {
            return Err(InformationError::NonFiniteValue);
        }
    }
    Ok((static_width, recovery_width))
}

fn reduce_rank(rows: &[Vec<f64>]) -> Result<ReducedBlock, InformationError> {
    let width = validate_matrix(rows)?;
    let n = rows.len();
    let n_f64 = n as f64;
    let threshold = PID_RANK_RESIDUAL_SCALE * n_f64.sqrt();
    let mut retained_columns = Vec::new();
    let mut standardized_columns: Vec<Vec<f64>> = Vec::new();
    let mut orthonormal_basis: Vec<Vec<f64>> = Vec::new();

    for column in 0..width {
        let mean = rows.iter().map(|row| row[column]).sum::<f64>() / n_f64;
        let mut centered = rows
            .iter()
            .map(|row| row[column] - mean)
            .collect::<Vec<_>>();
        let rms = (centered.iter().map(|value| value * value).sum::<f64>() / n_f64).sqrt();
        if !rms.is_finite() {
            return Err(InformationError::NonFiniteValue);
        }
        if rms == 0.0 {
            continue;
        }
        for value in &mut centered {
            *value /= rms;
        }
        if centered.iter().any(|value| !value.is_finite()) {
            return Err(InformationError::NonFiniteValue);
        }

        let standardized = centered.clone();
        let mut residual = centered;
        for basis in &orthonormal_basis {
            let projection = residual
                .iter()
                .zip(basis)
                .map(|(left, right)| left * right)
                .sum::<f64>();
            for (value, basis_value) in residual.iter_mut().zip(basis) {
                *value -= projection * basis_value;
            }
        }
        let residual_norm = residual.iter().map(|value| value * value).sum::<f64>().sqrt();
        if !residual_norm.is_finite() {
            return Err(InformationError::NonFiniteValue);
        }
        if residual_norm > threshold {
            for value in &mut residual {
                *value /= residual_norm;
            }
            retained_columns.push(column);
            standardized_columns.push(standardized);
            orthonormal_basis.push(residual);
        }
    }

    if retained_columns.is_empty() {
        return Err(InformationError::ZeroSourceRank);
    }

    let mut reduced_rows = vec![vec![0.0; retained_columns.len()]; n];
    for (reduced_column, values) in standardized_columns.iter().enumerate() {
        for (row, value) in values.iter().enumerate() {
            reduced_rows[row][reduced_column] = *value;
        }
    }
    Ok(ReducedBlock {
        rows: reduced_rows,
        retained_columns,
    })
}

fn covariance_matrix(rows: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, InformationError> {
    let width = validate_matrix(rows)?;
    let n = rows.len();
    let denominator = (n - 1) as f64;
    let means = (0..width)
        .map(|column| rows.iter().map(|row| row[column]).sum::<f64>() / n as f64)
        .collect::<Vec<_>>();
    let mut covariance = vec![vec![0.0; width]; width];
    for i in 0..width {
        for j in 0..=i {
            let mut sum = 0.0;
            for row in rows {
                sum += (row[i] - means[i]) * (row[j] - means[j]);
            }
            let value = sum / denominator;
            if !value.is_finite() {
                return Err(InformationError::NonFiniteValue);
            }
            covariance[i][j] = value;
            covariance[j][i] = value;
        }
    }
    Ok(covariance)
}

fn target_variance(target: &[f64]) -> Result<f64, InformationError> {
    if target.len() < 3 {
        return Err(InformationError::TooFewRecords);
    }
    if target.iter().any(|value| !value.is_finite()) {
        return Err(InformationError::NonFiniteValue);
    }
    let mean = target.iter().sum::<f64>() / target.len() as f64;
    let variance = target
        .iter()
        .map(|value| {
            let centered = value - mean;
            centered * centered
        })
        .sum::<f64>()
        / (target.len() - 1) as f64;
    if !variance.is_finite() {
        return Err(InformationError::NonFiniteValue);
    }
    if variance <= PID_DEGENERACY_PIVOT_FLOOR {
        return Err(InformationError::DegenerateTarget);
    }
    Ok(variance)
}

fn target_source_covariance(
    target: &[f64],
    source: &[Vec<f64>],
) -> Result<Vec<f64>, InformationError> {
    let width = validate_matrix(source)?;
    if target.len() != source.len() {
        return Err(InformationError::FeatureWidthMismatch);
    }
    if target.iter().any(|value| !value.is_finite()) {
        return Err(InformationError::NonFiniteValue);
    }
    let target_mean = target.iter().sum::<f64>() / target.len() as f64;
    let source_means = (0..width)
        .map(|column| source.iter().map(|row| row[column]).sum::<f64>() / source.len() as f64)
        .collect::<Vec<_>>();
    let denominator = (target.len() - 1) as f64;
    let mut result = vec![0.0; width];
    for column in 0..width {
        let mut sum = 0.0;
        for (target_value, row) in target.iter().zip(source) {
            sum += (target_value - target_mean) * (row[column] - source_means[column]);
        }
        result[column] = sum / denominator;
        if !result[column].is_finite() {
            return Err(InformationError::NonFiniteValue);
        }
    }
    Ok(result)
}

fn cholesky_log2_determinant(matrix: &[Vec<f64>]) -> Result<f64, InformationError> {
    let n = validate_matrix(matrix)?;
    if matrix.len() != n {
        return Err(InformationError::FeatureWidthMismatch);
    }
    let mut lower = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..=i {
            let mut product_sum = 0.0;
            for k in 0..j {
                product_sum += lower[i][k] * lower[j][k];
            }
            if i == j {
                let pivot = matrix[i][i] - product_sum;
                if !pivot.is_finite() || pivot <= PID_DEGENERACY_PIVOT_FLOOR {
                    return Err(InformationError::NonPositiveDefinite);
                }
                lower[i][j] = pivot.sqrt();
            } else {
                let denominator = lower[j][j];
                if !denominator.is_finite() || denominator <= PID_DEGENERACY_PIVOT_FLOOR {
                    return Err(InformationError::NonPositiveDefinite);
                }
                let value = (matrix[i][j] - product_sum) / denominator;
                if !value.is_finite() {
                    return Err(InformationError::NonPositiveDefinite);
                }
                lower[i][j] = value;
            }
        }
    }
    let log2_determinant = 2.0 * (0..n).map(|index| lower[index][index].log2()).sum::<f64>();
    if log2_determinant.is_finite() {
        Ok(log2_determinant)
    } else {
        Err(InformationError::NonPositiveDefinite)
    }
}

fn solve_linear(
    mut matrix: Vec<Vec<f64>>,
    mut rhs: Vec<f64>,
) -> Result<Vec<f64>, InformationError> {
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
        if !matrix[best][pivot].is_finite()
            || matrix[best][pivot].abs() <= PID_DEGENERACY_PIVOT_FLOOR
        {
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
    if rhs.iter().any(|value| !value.is_finite()) {
        return Err(InformationError::SingularSystem);
    }
    Ok(rhs)
}

fn canonical_mutual_information(
    target: &[f64],
    source: &ReducedBlock,
) -> Result<f64, InformationError> {
    let var_t = target_variance(target)?;
    let source_covariance = covariance_matrix(&source.rows)?;
    let log2_source = cholesky_log2_determinant(&source_covariance)?;

    let mut joint_rows = Vec::with_capacity(source.rows.len());
    for (target_value, source_row) in target.iter().zip(&source.rows) {
        let mut row = Vec::with_capacity(source.rank() + 1);
        row.push(*target_value);
        row.extend_from_slice(source_row);
        joint_rows.push(row);
    }
    let joint_covariance = covariance_matrix(&joint_rows)?;
    let log2_joint = cholesky_log2_determinant(&joint_covariance)?;
    let information = 0.5 * (var_t.log2() + log2_source - log2_joint);
    normalize_information(information)
}

fn multiple_correlation_mutual_information(
    target: &[f64],
    source: &ReducedBlock,
) -> Result<f64, InformationError> {
    let var_t = target_variance(target)?;
    let source_covariance = covariance_matrix(&source.rows)?;
    let cross_covariance = target_source_covariance(target, &source.rows)?;
    let coefficients = solve_linear(source_covariance, cross_covariance.clone())?;
    let mut r_squared = cross_covariance
        .iter()
        .zip(&coefficients)
        .map(|(covariance, coefficient)| covariance * coefficient)
        .sum::<f64>()
        / var_t;
    if !r_squared.is_finite() {
        return Err(InformationError::InvalidInformation);
    }
    if r_squared < 0.0 && r_squared >= -PID_CROSS_METHOD_TOLERANCE_BITS {
        r_squared = 0.0;
    }
    if r_squared < 0.0 || 1.0 - r_squared <= PID_DEGENERACY_PIVOT_FLOOR {
        return Err(InformationError::InvalidInformation);
    }
    normalize_information(-0.5 * (1.0 - r_squared).log2())
}

fn normalize_information(value: f64) -> Result<f64, InformationError> {
    if !value.is_finite() {
        return Err(InformationError::InvalidInformation);
    }
    if value < -PID_CROSS_METHOD_TOLERANCE_BITS {
        return Err(InformationError::InvalidInformation);
    }
    Ok(if value < 0.0 { 0.0 } else { value })
}

pub fn gaussian_mutual_information(
    target: &[f64],
    raw_source: &[Vec<f64>],
) -> Result<MutualInformationEstimate, InformationError> {
    if target.len() != raw_source.len() {
        return Err(InformationError::FeatureWidthMismatch);
    }
    let reduced = reduce_rank(raw_source)?;
    let canonical_bits = canonical_mutual_information(target, &reduced)?;
    let crosscheck_bits = multiple_correlation_mutual_information(target, &reduced)?;
    let discrepancy_bits = (canonical_bits - crosscheck_bits).abs();
    if !discrepancy_bits.is_finite()
        || discrepancy_bits > PID_CROSS_METHOD_TOLERANCE_BITS
    {
        return Err(InformationError::CrossMethodDisagreement);
    }
    Ok(MutualInformationEstimate {
        rank: reduced.rank(),
        canonical_bits,
        crosscheck_bits,
        discrepancy_bits,
    })
}

fn clamp_pid_component(value: f64) -> Result<f64, InformationError> {
    if !value.is_finite() {
        return Err(InformationError::InvalidInformation);
    }
    if value < -PID_CROSS_METHOD_TOLERANCE_BITS {
        return Err(InformationError::NegativePidComponent);
    }
    Ok(if value < 0.0 { 0.0 } else { value })
}

pub fn evaluate_pid(records: &[InformationRecord]) -> Result<PidEstimate, InformationError> {
    validate_records(records)?;
    let target = records.iter().map(|record| record.target).collect::<Vec<_>>();
    let static_rows = records
        .iter()
        .map(|record| record.static_block.clone())
        .collect::<Vec<_>>();
    let recovery_rows = records
        .iter()
        .map(|record| record.recovery_block.clone())
        .collect::<Vec<_>>();
    let joint_rows = records
        .iter()
        .map(|record| {
            let mut row = Vec::with_capacity(record.static_block.len() + record.recovery_block.len());
            row.extend_from_slice(&record.static_block);
            row.extend_from_slice(&record.recovery_block);
            row
        })
        .collect::<Vec<_>>();

    let static_mi = gaussian_mutual_information(&target, &static_rows)?;
    let recovery_mi = gaussian_mutual_information(&target, &recovery_rows)?;
    let joint_mi = gaussian_mutual_information(&target, &joint_rows)?;

    let redundancy_bits = static_mi.canonical_bits.min(recovery_mi.canonical_bits);
    let unique_static_bits = clamp_pid_component(static_mi.canonical_bits - redundancy_bits)?;
    let unique_recovery_bits = clamp_pid_component(recovery_mi.canonical_bits - redundancy_bits)?;
    let synergy_bits = clamp_pid_component(
        joint_mi.canonical_bits - static_mi.canonical_bits - recovery_mi.canonical_bits
            + redundancy_bits,
    )?;
    let reconstructed =
        redundancy_bits + unique_static_bits + unique_recovery_bits + synergy_bits;
    if (reconstructed - joint_mi.canonical_bits).abs() > PID_CROSS_METHOD_TOLERANCE_BITS {
        return Err(InformationError::PidIdentityFailure);
    }

    Ok(PidEstimate {
        static_mi,
        recovery_mi,
        joint_mi,
        redundancy_bits,
        unique_static_bits,
        unique_recovery_bits,
        synergy_bits,
    })
}

fn generator_groups(records: &[InformationRecord]) -> Result<Vec<Vec<usize>>, InformationError> {
    validate_records(records)?;
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
    if groups.is_empty() {
        return Err(InformationError::InvalidBootstrap);
    }
    Ok(groups.into_iter().map(|(_, indices)| indices).collect())
}

fn percentile(sorted: &[f64], probability: f64) -> Result<f64, InformationError> {
    if sorted.is_empty() || !(0.0..=1.0).contains(&probability) {
        return Err(InformationError::InvalidBootstrap);
    }
    let position = probability * (sorted.len() - 1) as f64;
    let lower = position.floor() as usize;
    let upper = position.ceil() as usize;
    let value = if lower == upper {
        sorted[lower]
    } else {
        let weight = position - lower as f64;
        sorted[lower] * (1.0 - weight) + sorted[upper] * weight
    };
    if value.is_finite() {
        Ok(value)
    } else {
        Err(InformationError::InvalidBootstrap)
    }
}

fn interval(mut values: Vec<f64>) -> Result<BootstrapInterval, InformationError> {
    values.sort_by(f64::total_cmp);
    Ok(BootstrapInterval {
        lower: percentile(&values, 0.025)?,
        median: percentile(&values, 0.5)?,
        upper: percentile(&values, 0.975)?,
    })
}

fn is_bootstrap_degeneracy(error: InformationError) -> bool {
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
) -> Result<PidBootstrapSummary, InformationError> {
    let point = evaluate_pid(records)?;
    if replicates < 2 {
        return Err(InformationError::InvalidBootstrap);
    }
    let groups = generator_groups(records)?;
    let mut rng = SplitMix64::new(seed);
    let mut joint_samples = Vec::with_capacity(replicates);
    let mut redundancy_samples = Vec::with_capacity(replicates);
    let mut unique_static_samples = Vec::with_capacity(replicates);
    let mut unique_recovery_samples = Vec::with_capacity(replicates);
    let mut synergy_samples = Vec::with_capacity(replicates);
    let mut rejected_replicates = 0usize;

    for _ in 0..replicates {
        let mut resampled = Vec::with_capacity(records.len());
        for _ in 0..groups.len() {
            let group = &groups[rng.bounded(groups.len())?];
            for &index in group {
                resampled.push(records[index].clone());
            }
        }
        match evaluate_pid(&resampled) {
            Ok(estimate) => {
                joint_samples.push(estimate.joint_mi.canonical_bits);
                redundancy_samples.push(estimate.redundancy_bits);
                unique_static_samples.push(estimate.unique_static_bits);
                unique_recovery_samples.push(estimate.unique_recovery_bits);
                synergy_samples.push(estimate.synergy_bits);
            }
            Err(error) if is_bootstrap_degeneracy(error) => {
                rejected_replicates = rejected_replicates
                    .checked_add(1)
                    .ok_or(InformationError::InvalidBootstrap)?;
            }
            Err(error) => return Err(error),
        }
    }

    let accepted_replicates = joint_samples.len();
    if accepted_replicates < 2
        || (accepted_replicates as f64 / replicates as f64) < 0.95
    {
        return Err(InformationError::InvalidBootstrap);
    }

    Ok(PidBootstrapSummary {
        point,
        joint: interval(joint_samples)?,
        redundancy: interval(redundancy_samples)?,
        unique_static: interval(unique_static_samples)?,
        unique_recovery: interval(unique_recovery_samples)?,
        synergy: interval(synergy_samples)?,
        accepted_replicates,
        rejected_replicates,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(actual: f64, expected: f64, tolerance: f64) {
        assert!(
            (actual - expected).abs() <= tolerance,
            "expected {expected}, got {actual}"
        );
    }

    fn synthetic_records(generator_count: usize) -> Vec<InformationRecord> {
        let mut records = Vec::with_capacity(generator_count * 2);
        for generator in 0..generator_count {
            let x = generator as f64 - generator_count as f64 / 2.0;
            let z = ((generator * 7 + 3) % 17) as f64 - 8.0;
            let w = ((generator * 11 + 5) % 23) as f64 - 11.0;
            for site in 0..2 {
                let site_residual = if site == 0 { -0.35 } else { 0.55 };
                let static_block = vec![x, z, x + z, 2.0 * x, w, z - w];
                let recovery_block = vec![
                    0.2 * x + 0.03 * w,
                    0.15 * z - 0.02 * w,
                    (x / 9.0).sin() + 0.01 * z,
                    (z / 7.0).cos() + 0.02 * w,
                ];
                let target = 0.31 * x - 0.17 * z + 0.08 * w + site_residual;
                records.push(
                    InformationRecord::new(
                        generator as u64,
                        static_block,
                        recovery_block,
                        target,
                    )
                    .unwrap(),
                );
            }
        }
        records
    }

    #[test]
    fn independent_scalar_source_has_zero_gaussian_information() {
        let target = vec![-1.0, 1.0, -1.0, 1.0, -2.0, 2.0, -2.0, 2.0];
        let source = vec![
            vec![-1.0],
            vec![-1.0],
            vec![1.0],
            vec![1.0],
            vec![-2.0],
            vec![-2.0],
            vec![2.0],
            vec![2.0],
        ];
        let estimate = gaussian_mutual_information(&target, &source).unwrap();
        assert_eq!(estimate.rank(), 1);
        assert_close(estimate.canonical_bits(), 0.0, 1.0e-12);
        assert_close(estimate.crosscheck_bits(), 0.0, 1.0e-12);
    }

    #[test]
    fn scalar_source_matches_closed_form_correlation_identity() {
        let target = vec![-3.0, -1.0, 0.0, 2.0, 4.0, 5.0];
        let source_values = vec![-2.0, -0.5, 1.0, 1.5, 3.5, 4.0];
        let source = source_values
            .iter()
            .map(|value| vec![*value])
            .collect::<Vec<_>>();
        let target_mean = target.iter().sum::<f64>() / target.len() as f64;
        let source_mean = source_values.iter().sum::<f64>() / source_values.len() as f64;
        let numerator = target
            .iter()
            .zip(&source_values)
            .map(|(left, right)| (left - target_mean) * (right - source_mean))
            .sum::<f64>();
        let left_norm = target
            .iter()
            .map(|value| (value - target_mean).powi(2))
            .sum::<f64>();
        let right_norm = source_values
            .iter()
            .map(|value| (value - source_mean).powi(2))
            .sum::<f64>();
        let rho_squared = numerator * numerator / (left_norm * right_norm);
        let expected = -0.5 * (1.0 - rho_squared).log2();
        let estimate = gaussian_mutual_information(&target, &source).unwrap();
        assert_close(estimate.canonical_bits(), expected, 1.0e-9);
        assert_close(estimate.crosscheck_bits(), expected, 1.0e-9);
    }

    #[test]
    fn exact_duplicate_columns_are_removed_without_changing_information() {
        let target = vec![-2.0, -1.0, 0.5, 1.0, 2.5, 4.0, 5.0, 7.0];
        let single = vec![-3.0, -1.5, -0.5, 0.5, 1.5, 2.0, 4.0, 5.0];
        let one = single.iter().map(|value| vec![*value]).collect::<Vec<_>>();
        let duplicate = single
            .iter()
            .map(|value| vec![*value, *value, 2.0 * *value])
            .collect::<Vec<_>>();
        let left = gaussian_mutual_information(&target, &one).unwrap();
        let right = gaussian_mutual_information(&target, &duplicate).unwrap();
        assert_eq!(left.rank(), 1);
        assert_eq!(right.rank(), 1);
        assert_close(left.canonical_bits(), right.canonical_bits(), 1.0e-12);
    }

    #[test]
    fn constant_only_source_fails_closed() {
        let target = vec![0.0, 1.0, 2.0, 3.0];
        let source = vec![vec![1.0, 2.0]; 4];
        assert_eq!(
            gaussian_mutual_information(&target, &source),
            Err(InformationError::ZeroSourceRank)
        );
    }

    #[test]
    fn pid_identity_and_cross_method_agreement_hold_on_vector_blocks() {
        let records = synthetic_records(48);
        let estimate = evaluate_pid(&records).unwrap();
        let reconstructed = estimate.redundancy_bits()
            + estimate.unique_static_bits()
            + estimate.unique_recovery_bits()
            + estimate.synergy_bits();
        assert_close(
            reconstructed,
            estimate.joint_mi().canonical_bits(),
            PID_CROSS_METHOD_TOLERANCE_BITS,
        );
        assert!(estimate.static_mi().discrepancy_bits() <= PID_CROSS_METHOD_TOLERANCE_BITS);
        assert!(estimate.recovery_mi().discrepancy_bits() <= PID_CROSS_METHOD_TOLERANCE_BITS);
        assert!(estimate.joint_mi().discrepancy_bits() <= PID_CROSS_METHOD_TOLERANCE_BITS);
        assert!(estimate.redundancy_bits() >= 0.0);
        assert!(estimate.unique_static_bits() >= 0.0);
        assert!(estimate.unique_recovery_bits() >= 0.0);
        assert!(estimate.synergy_bits() >= 0.0);
    }

    #[test]
    fn generator_bootstrap_is_deterministic_and_keeps_replication_accounting() {
        let records = synthetic_records(40);
        let left = bootstrap_pid(&records, 128, 0x1234_5678_9abc_def0).unwrap();
        let right = bootstrap_pid(&records, 128, 0x1234_5678_9abc_def0).unwrap();
        assert_eq!(left, right);
        assert_eq!(
            left.accepted_replicates() + left.rejected_replicates(),
            128
        );
        assert!(left.accepted_replicates() >= 122);
        assert!(left.joint().lower().is_finite());
        assert!(left.joint().upper().is_finite());
        assert!(left.synergy().lower().is_finite());
        assert!(left.synergy().upper().is_finite());
    }

    #[test]
    fn malformed_record_widths_fail_closed() {
        let good = InformationRecord::new(1, vec![1.0], vec![2.0], 0.5).unwrap();
        let bad = InformationRecord::new(2, vec![1.0, 2.0], vec![2.0], 0.5).unwrap();
        assert_eq!(
            evaluate_pid(&[good.clone(), bad, good]),
            Err(InformationError::FeatureWidthMismatch)
        );
    }

    #[test]
    fn source_contains_no_final_holdout_authorization_secret() {
        let source = include_str!("gaussian_mmi_v7.rs");
        let confirmation = ["I_ACCEPT_THE_TDI7_", "HOLDOUT_FREEZE"].concat();
        let environment = ["TDI7_CONFIRM_FINAL_", "HOLDOUT"].concat();
        assert!(!source.contains(&confirmation));
        assert!(!source.contains(&environment));
    }
}
