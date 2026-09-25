//! TDI-27 Development-only latent concept-geometry primitives.
//!
//! These routines implement deterministic mean contrasts, orthogonal
//! residualisation, sequential innovation, and small causal-summary helpers.
//! They are research diagnostics, not a novelty classifier and not a
//! confirmatory evaluation surface.

pub const DEFAULT_TOLERANCE: f64 = 1.0e-12;

/// Caller-supplied deterministic resampling plan for TDI-27 Development work.
///
/// No default seed or replicate count is provided so a later protocol can
/// declare them explicitly without silently changing statistical semantics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DevelopmentResamplingPlan {
    replicates: usize,
    seed: u64,
}

impl DevelopmentResamplingPlan {
    pub fn new(replicates: usize, seed: u64) -> Result<Self, ConceptGeometryError> {
        if replicates < 2 {
            return Err(ConceptGeometryError::InvalidReplicateCount);
        }
        Ok(Self { replicates, seed })
    }

    #[must_use]
    pub const fn replicates(self) -> usize {
        self.replicates
    }

    #[must_use]
    pub const fn seed(self) -> u64 {
        self.seed
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
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }

    fn bounded(&mut self, upper: usize) -> Result<usize, ConceptGeometryError> {
        let upper = u64::try_from(upper).map_err(|_| ConceptGeometryError::SampleCountTooLarge)?;
        if upper == 0 {
            return Err(ConceptGeometryError::EmptyGroup);
        }

        let threshold = upper.wrapping_neg() % upper;
        loop {
            let value = self.next_u64();
            if value >= threshold {
                return usize::try_from(value % upper)
                    .map_err(|_| ConceptGeometryError::SampleCountTooLarge);
            }
        }
    }
}

/// Deterministically draw bootstrap indices with replacement.
///
/// The output contains exactly `plan.replicates()` rows and each row contains
/// exactly `sample_count` indices in `0..sample_count`.
pub fn bootstrap_index_replicates(
    sample_count: usize,
    plan: DevelopmentResamplingPlan,
) -> Result<Vec<Vec<usize>>, ConceptGeometryError> {
    if sample_count == 0 {
        return Err(ConceptGeometryError::EmptyGroup);
    }
    let total = plan
        .replicates()
        .checked_mul(sample_count)
        .ok_or(ConceptGeometryError::ReplicateAccountingOverflow)?;
    let mut output = Vec::new();
    output
        .try_reserve_exact(plan.replicates())
        .map_err(|_| ConceptGeometryError::ReplicateAccountingOverflow)?;
    let mut rng = SplitMix64::new(plan.seed());

    let mut drawn = 0usize;
    for _ in 0..plan.replicates() {
        let mut replicate = Vec::new();
        replicate
            .try_reserve_exact(sample_count)
            .map_err(|_| ConceptGeometryError::ReplicateAccountingOverflow)?;
        for _ in 0..sample_count {
            replicate.push(rng.bounded(sample_count)?);
            drawn = drawn
                .checked_add(1)
                .ok_or(ConceptGeometryError::ReplicateAccountingOverflow)?;
        }
        output.push(replicate);
    }
    if drawn != total {
        return Err(ConceptGeometryError::ReplicateAccountingOverflow);
    }
    Ok(output)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndependentGroupBootstrapIndices {
    positive: Vec<usize>,
    control: Vec<usize>,
}

impl IndependentGroupBootstrapIndices {
    #[must_use]
    pub fn positive(&self) -> &[usize] {
        &self.positive
    }

    #[must_use]
    pub fn control(&self) -> &[usize] {
        &self.control
    }
}

const POSITIVE_BOOTSTRAP_DOMAIN: u64 = 0x5444_4932_3750_4f53;
const CONTROL_BOOTSTRAP_DOMAIN: u64 = 0x5444_4932_3743_5452;

fn domain_separated_seed(seed: u64, domain: u64) -> u64 {
    let mut rng = SplitMix64::new(seed ^ domain);
    rng.next_u64()
}

/// Draw independent bootstrap indices for the positive and control groups.
///
/// The two groups use domain-separated deterministic RNG streams. Changing the
/// size of one group therefore does not perturb the draw sequence of the other
/// group. Each replicate preserves the original cardinality of each group and
/// samples with replacement inside that group only.
pub fn independent_group_bootstrap_indices(
    positive_count: usize,
    control_count: usize,
    plan: DevelopmentResamplingPlan,
) -> Result<Vec<IndependentGroupBootstrapIndices>, ConceptGeometryError> {
    if positive_count == 0 || control_count == 0 {
        return Err(ConceptGeometryError::EmptyGroup);
    }

    let positive_draws = plan
        .replicates()
        .checked_mul(positive_count)
        .ok_or(ConceptGeometryError::ReplicateAccountingOverflow)?;
    let control_draws = plan
        .replicates()
        .checked_mul(control_count)
        .ok_or(ConceptGeometryError::ReplicateAccountingOverflow)?;

    let mut output = Vec::new();
    output
        .try_reserve_exact(plan.replicates())
        .map_err(|_| ConceptGeometryError::ReplicateAccountingOverflow)?;

    let mut positive_rng = SplitMix64::new(domain_separated_seed(
        plan.seed(),
        POSITIVE_BOOTSTRAP_DOMAIN,
    ));
    let mut control_rng =
        SplitMix64::new(domain_separated_seed(plan.seed(), CONTROL_BOOTSTRAP_DOMAIN));
    let mut positive_drawn = 0usize;
    let mut control_drawn = 0usize;

    for _ in 0..plan.replicates() {
        let mut positive = Vec::new();
        positive
            .try_reserve_exact(positive_count)
            .map_err(|_| ConceptGeometryError::ReplicateAccountingOverflow)?;
        for _ in 0..positive_count {
            positive.push(positive_rng.bounded(positive_count)?);
            positive_drawn = positive_drawn
                .checked_add(1)
                .ok_or(ConceptGeometryError::ReplicateAccountingOverflow)?;
        }

        let mut control = Vec::new();
        control
            .try_reserve_exact(control_count)
            .map_err(|_| ConceptGeometryError::ReplicateAccountingOverflow)?;
        for _ in 0..control_count {
            control.push(control_rng.bounded(control_count)?);
            control_drawn = control_drawn
                .checked_add(1)
                .ok_or(ConceptGeometryError::ReplicateAccountingOverflow)?;
        }

        output.push(IndependentGroupBootstrapIndices { positive, control });
    }

    if positive_drawn != positive_draws || control_drawn != control_draws {
        return Err(ConceptGeometryError::ReplicateAccountingOverflow);
    }

    Ok(output)
}

#[derive(Clone, Debug, PartialEq)]
pub struct BootstrapMeanContrast {
    positive_mean: Vec<f64>,
    control_mean: Vec<f64>,
    contrast: Vec<f64>,
}

impl BootstrapMeanContrast {
    #[must_use]
    pub fn positive_mean(&self) -> &[f64] {
        &self.positive_mean
    }

    #[must_use]
    pub fn control_mean(&self) -> &[f64] {
        &self.control_mean
    }

    #[must_use]
    pub fn contrast(&self) -> &[f64] {
        &self.contrast
    }
}

fn indexed_mean(rows: &[Vec<f64>], indices: &[usize]) -> Result<Vec<f64>, ConceptGeometryError> {
    let width = validate_rows(rows)?;
    if indices.is_empty() {
        return Err(ConceptGeometryError::EmptyGroup);
    }

    let mut result = vec![0.0; width];
    for &index in indices {
        let row = rows
            .get(index)
            .ok_or(ConceptGeometryError::InvalidBootstrapIndex)?;
        for (target, value) in result.iter_mut().zip(row) {
            *target += value;
            if !target.is_finite() {
                return Err(ConceptGeometryError::NonFiniteValue);
            }
        }
    }

    let scale = 1.0 / indices.len() as f64;
    for value in &mut result {
        *value *= scale;
        if !value.is_finite() {
            return Err(ConceptGeometryError::NonFiniteValue);
        }
    }
    Ok(result)
}

/// Compute deterministic bootstrap mean contrasts for the positive/control groups.
///
/// Every replicate reuses the domain-separated group index streams, preserves
/// each original group cardinality, and returns the signed contrast
/// `mean(positive) - mean(control)`. This Development primitive intentionally
/// does not define confidence intervals, p-values, thresholds, or verdicts.
pub fn bootstrap_mean_contrasts(
    positive: &[Vec<f64>],
    control: &[Vec<f64>],
    plan: DevelopmentResamplingPlan,
) -> Result<Vec<BootstrapMeanContrast>, ConceptGeometryError> {
    let positive_width = validate_rows(positive)?;
    let control_width = validate_rows(control)?;
    if positive_width != control_width {
        return Err(ConceptGeometryError::DimensionMismatch);
    }

    let replicates = independent_group_bootstrap_indices(positive.len(), control.len(), plan)?;
    let mut output = Vec::new();
    output
        .try_reserve_exact(plan.replicates())
        .map_err(|_| ConceptGeometryError::ReplicateAccountingOverflow)?;

    for replicate in replicates {
        let positive_mean = indexed_mean(positive, replicate.positive())?;
        let control_mean = indexed_mean(control, replicate.control())?;
        let contrast = positive_mean
            .iter()
            .zip(&control_mean)
            .map(|(left, right)| left - right)
            .collect::<Vec<_>>();
        if contrast.iter().any(|value| !value.is_finite()) {
            return Err(ConceptGeometryError::NonFiniteValue);
        }

        output.push(BootstrapMeanContrast {
            positive_mean,
            control_mean,
            contrast,
        });
    }

    Ok(output)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LabelShuffleIndices {
    positive: Vec<usize>,
    control: Vec<usize>,
}

impl LabelShuffleIndices {
    #[must_use]
    pub fn positive(&self) -> &[usize] {
        &self.positive
    }

    #[must_use]
    pub fn control(&self) -> &[usize] {
        &self.control
    }
}

const LABEL_SHUFFLE_DOMAIN: u64 = 0x5444_4932_374c_424c;

/// Deterministically permute group labels while preserving P/C cardinalities.
///
/// Every replicate is a complete partition of the pooled sample indices.
/// This Development primitive defines only the permutation mechanism. It does
/// not define a null rejection threshold, tail rule, p-value, or verdict.
pub fn label_shuffle_indices(
    positive_count: usize,
    control_count: usize,
    plan: DevelopmentResamplingPlan,
) -> Result<Vec<LabelShuffleIndices>, ConceptGeometryError> {
    if positive_count == 0 || control_count == 0 {
        return Err(ConceptGeometryError::EmptyGroup);
    }
    let total = positive_count
        .checked_add(control_count)
        .ok_or(ConceptGeometryError::SampleCountOverflow)?;

    let mut rng = SplitMix64::new(domain_separated_seed(plan.seed(), LABEL_SHUFFLE_DOMAIN));
    let mut output = Vec::new();
    output
        .try_reserve_exact(plan.replicates())
        .map_err(|_| ConceptGeometryError::ReplicateAccountingOverflow)?;

    for _ in 0..plan.replicates() {
        let mut permutation = (0..total).collect::<Vec<_>>();
        for index in (1..total).rev() {
            let swap_with = rng.bounded(index + 1)?;
            permutation.swap(index, swap_with);
        }

        let positive = permutation[..positive_count].to_vec();
        let control = permutation[positive_count..].to_vec();
        output.push(LabelShuffleIndices { positive, control });
    }

    Ok(output)
}

#[derive(Clone, Debug, PartialEq)]
pub struct LabelShuffleMeanContrast {
    positive_indices: Vec<usize>,
    control_indices: Vec<usize>,
    contrast: Vec<f64>,
}

impl LabelShuffleMeanContrast {
    #[must_use]
    pub fn positive_indices(&self) -> &[usize] {
        &self.positive_indices
    }

    #[must_use]
    pub fn control_indices(&self) -> &[usize] {
        &self.control_indices
    }

    #[must_use]
    pub fn contrast(&self) -> &[f64] {
        &self.contrast
    }
}

/// Convert cardinality-preserving shuffled labels into signed null contrasts.
///
/// The pooled observations are unchanged; only group membership is permuted.
/// Every report retains its exact pooled indices for provenance and computes
/// `mean(shuffled_positive) - mean(shuffled_control)`.
pub fn label_shuffle_mean_contrasts(
    positive: &[Vec<f64>],
    control: &[Vec<f64>],
    plan: DevelopmentResamplingPlan,
) -> Result<Vec<LabelShuffleMeanContrast>, ConceptGeometryError> {
    let positive_width = validate_rows(positive)?;
    let control_width = validate_rows(control)?;
    if positive_width != control_width {
        return Err(ConceptGeometryError::DimensionMismatch);
    }

    let mut pooled = Vec::new();
    pooled
        .try_reserve_exact(
            positive
                .len()
                .checked_add(control.len())
                .ok_or(ConceptGeometryError::SampleCountOverflow)?,
        )
        .map_err(|_| ConceptGeometryError::ReplicateAccountingOverflow)?;
    pooled.extend_from_slice(positive);
    pooled.extend_from_slice(control);

    let shuffles = label_shuffle_indices(positive.len(), control.len(), plan)?;
    let mut output = Vec::new();
    output
        .try_reserve_exact(shuffles.len())
        .map_err(|_| ConceptGeometryError::ReplicateAccountingOverflow)?;

    for shuffle in shuffles {
        let positive_mean = indexed_mean(&pooled, shuffle.positive())?;
        let control_mean = indexed_mean(&pooled, shuffle.control())?;
        let contrast = positive_mean
            .iter()
            .zip(&control_mean)
            .map(|(left, right)| left - right)
            .collect::<Vec<_>>();
        if contrast.iter().any(|value| !value.is_finite()) {
            return Err(ConceptGeometryError::NonFiniteValue);
        }

        output.push(LabelShuffleMeanContrast {
            positive_indices: shuffle.positive().to_vec(),
            control_indices: shuffle.control().to_vec(),
            contrast,
        });
    }

    Ok(output)
}

#[derive(Clone, Debug, PartialEq)]
pub struct LabelShuffleResidualGeometry {
    positive_indices: Vec<usize>,
    control_indices: Vec<usize>,
    contrast: Vec<f64>,
    residual: Option<ResidualDirection>,
}

impl LabelShuffleResidualGeometry {
    #[must_use]
    pub fn positive_indices(&self) -> &[usize] {
        &self.positive_indices
    }

    #[must_use]
    pub fn control_indices(&self) -> &[usize] {
        &self.control_indices
    }

    #[must_use]
    pub fn contrast(&self) -> &[f64] {
        &self.contrast
    }

    #[must_use]
    pub fn residual(&self) -> Option<&ResidualDirection> {
        self.residual.as_ref()
    }
}

/// Residualize shuffled-label null contrasts against one declared basis.
///
/// An exactly/near-zero shuffled raw contrast is a valid null outcome and is
/// retained as `residual = None`. It is never coerced to zero innovation
/// energy and never removed from replicate accounting.
pub fn label_shuffle_residual_geometries(
    positive: &[Vec<f64>],
    control: &[Vec<f64>],
    basis_directions: &[Vec<f64>],
    tolerance: f64,
    plan: DevelopmentResamplingPlan,
) -> Result<Vec<LabelShuffleResidualGeometry>, ConceptGeometryError> {
    valid_tolerance(tolerance)?;
    let contrasts = label_shuffle_mean_contrasts(positive, control, plan)?;
    let mut output = Vec::new();
    output
        .try_reserve_exact(contrasts.len())
        .map_err(|_| ConceptGeometryError::ReplicateAccountingOverflow)?;

    for report in contrasts {
        let raw_norm = norm(report.contrast());
        if !raw_norm.is_finite() {
            return Err(ConceptGeometryError::NonFiniteValue);
        }
        let residual = if raw_norm <= tolerance {
            None
        } else {
            Some(residualize(report.contrast(), basis_directions, tolerance)?)
        };

        output.push(LabelShuffleResidualGeometry {
            positive_indices: report.positive_indices().to_vec(),
            control_indices: report.control_indices().to_vec(),
            contrast: report.contrast().to_vec(),
            residual,
        });
    }

    Ok(output)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndependentGroupSubsampleIndices {
    positive: Vec<usize>,
    control: Vec<usize>,
}

impl IndependentGroupSubsampleIndices {
    #[must_use]
    pub fn positive(&self) -> &[usize] {
        &self.positive
    }

    #[must_use]
    pub fn control(&self) -> &[usize] {
        &self.control
    }
}

const POSITIVE_SUBSAMPLE_DOMAIN: u64 = 0x5444_4932_3753_5053;
const CONTROL_SUBSAMPLE_DOMAIN: u64 = 0x5444_4932_3753_4354;

fn subsample_without_replacement(
    population_count: usize,
    sample_count: usize,
    rng: &mut SplitMix64,
) -> Result<Vec<usize>, ConceptGeometryError> {
    if sample_count == 0 || sample_count > population_count {
        return Err(ConceptGeometryError::InvalidSubsampleSize);
    }

    let mut indices = (0..population_count).collect::<Vec<_>>();
    for offset in 0..sample_count {
        let remaining = population_count - offset;
        let swap_with = offset
            .checked_add(rng.bounded(remaining)?)
            .ok_or(ConceptGeometryError::SampleCountOverflow)?;
        indices.swap(offset, swap_with);
    }
    indices.truncate(sample_count);
    Ok(indices)
}

/// Draw deterministic P/C subsamples without replacement.
///
/// Positive and control streams are domain-separated so changing one group's
/// requested sample size does not perturb the other group's RNG stream.
pub fn independent_group_subsample_indices(
    positive_count: usize,
    control_count: usize,
    positive_sample_count: usize,
    control_sample_count: usize,
    plan: DevelopmentResamplingPlan,
) -> Result<Vec<IndependentGroupSubsampleIndices>, ConceptGeometryError> {
    if positive_count == 0 || control_count == 0 {
        return Err(ConceptGeometryError::EmptyGroup);
    }
    if positive_sample_count == 0
        || positive_sample_count > positive_count
        || control_sample_count == 0
        || control_sample_count > control_count
    {
        return Err(ConceptGeometryError::InvalidSubsampleSize);
    }

    let mut positive_rng = SplitMix64::new(domain_separated_seed(
        plan.seed(),
        POSITIVE_SUBSAMPLE_DOMAIN,
    ));
    let mut control_rng =
        SplitMix64::new(domain_separated_seed(plan.seed(), CONTROL_SUBSAMPLE_DOMAIN));
    let mut output = Vec::new();
    output
        .try_reserve_exact(plan.replicates())
        .map_err(|_| ConceptGeometryError::ReplicateAccountingOverflow)?;

    for _ in 0..plan.replicates() {
        output.push(IndependentGroupSubsampleIndices {
            positive: subsample_without_replacement(
                positive_count,
                positive_sample_count,
                &mut positive_rng,
            )?,
            control: subsample_without_replacement(
                control_count,
                control_sample_count,
                &mut control_rng,
            )?,
        });
    }

    Ok(output)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConceptGeometryError {
    EmptyGroup,
    EmptyVector,
    DimensionMismatch,
    NonFiniteValue,
    InvalidTolerance,
    ZeroNorm,
    EmptyControls,
    InvalidReplicateCount,
    SampleCountTooLarge,
    ReplicateAccountingOverflow,
    InvalidBootstrapIndex,
    UndefinedReferenceDirection,
    EmptyScalarSample,
    InvalidOrderStatisticRank,
    InvalidOrderStatisticRanks,
    SampleCountOverflow,
    InvalidSubsampleSize,
    EmptySampleSizeGrid,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResidualDirection {
    raw: Vec<f64>,
    residual: Vec<f64>,
    unit_direction: Option<Vec<f64>>,
    raw_norm: f64,
    residual_norm: f64,
    innovation_energy_ratio: f64,
    basis_rank: usize,
}

impl ResidualDirection {
    #[must_use]
    pub fn raw(&self) -> &[f64] {
        &self.raw
    }

    #[must_use]
    pub fn residual(&self) -> &[f64] {
        &self.residual
    }

    #[must_use]
    pub fn unit_direction(&self) -> Option<&[f64]> {
        self.unit_direction.as_deref()
    }

    #[must_use]
    pub const fn raw_norm(&self) -> f64 {
        self.raw_norm
    }

    #[must_use]
    pub const fn residual_norm(&self) -> f64 {
        self.residual_norm
    }

    #[must_use]
    pub const fn innovation_energy_ratio(&self) -> f64 {
        self.innovation_energy_ratio
    }

    #[must_use]
    pub const fn basis_rank(&self) -> usize {
        self.basis_rank
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SequentialInnovationStep {
    index: usize,
    raw_norm: f64,
    residual_norm: f64,
    innovation_energy_ratio: f64,
    accepted: bool,
    unit_direction: Option<Vec<f64>>,
}

impl SequentialInnovationStep {
    #[must_use]
    pub const fn index(&self) -> usize {
        self.index
    }

    #[must_use]
    pub const fn raw_norm(&self) -> f64 {
        self.raw_norm
    }

    #[must_use]
    pub const fn residual_norm(&self) -> f64 {
        self.residual_norm
    }

    #[must_use]
    pub const fn innovation_energy_ratio(&self) -> f64 {
        self.innovation_energy_ratio
    }

    #[must_use]
    pub const fn accepted(&self) -> bool {
        self.accepted
    }

    #[must_use]
    pub fn unit_direction(&self) -> Option<&[f64]> {
        self.unit_direction.as_deref()
    }
}

fn valid_tolerance(tolerance: f64) -> Result<(), ConceptGeometryError> {
    if !tolerance.is_finite() || tolerance <= 0.0 {
        return Err(ConceptGeometryError::InvalidTolerance);
    }
    Ok(())
}

fn dot(left: &[f64], right: &[f64]) -> f64 {
    left.iter().zip(right).map(|(a, b)| a * b).sum()
}

fn norm(vector: &[f64]) -> f64 {
    dot(vector, vector).sqrt()
}

fn validate_vector(vector: &[f64]) -> Result<(), ConceptGeometryError> {
    if vector.is_empty() {
        return Err(ConceptGeometryError::EmptyVector);
    }
    if vector.iter().any(|value| !value.is_finite()) {
        return Err(ConceptGeometryError::NonFiniteValue);
    }
    Ok(())
}

fn validate_rows(rows: &[Vec<f64>]) -> Result<usize, ConceptGeometryError> {
    let first = rows.first().ok_or(ConceptGeometryError::EmptyGroup)?;
    validate_vector(first)?;
    let width = first.len();
    for row in rows {
        validate_vector(row)?;
        if row.len() != width {
            return Err(ConceptGeometryError::DimensionMismatch);
        }
    }
    Ok(width)
}

fn mean(rows: &[Vec<f64>]) -> Result<Vec<f64>, ConceptGeometryError> {
    let width = validate_rows(rows)?;
    let mut result = vec![0.0; width];
    for row in rows {
        for (target, value) in result.iter_mut().zip(row) {
            *target += value;
        }
    }
    let scale = 1.0 / rows.len() as f64;
    for value in &mut result {
        *value *= scale;
    }
    Ok(result)
}

/// Compute the signed mean contrast P - C.
pub fn mean_difference(
    positive: &[Vec<f64>],
    control: &[Vec<f64>],
) -> Result<Vec<f64>, ConceptGeometryError> {
    let positive_mean = mean(positive)?;
    let control_mean = mean(control)?;
    if positive_mean.len() != control_mean.len() {
        return Err(ConceptGeometryError::DimensionMismatch);
    }
    Ok(positive_mean
        .iter()
        .zip(control_mean)
        .map(|(left, right)| left - right)
        .collect())
}

fn remove_basis_components(vector: &mut [f64], basis: &[Vec<f64>]) {
    // A second pass reduces loss of orthogonality for nearly dependent inputs.
    for _ in 0..2 {
        for direction in basis {
            let coefficient = dot(vector, direction);
            for (value, basis_value) in vector.iter_mut().zip(direction) {
                *value -= coefficient * basis_value;
            }
        }
    }
}

/// Build an orthonormal basis with two-pass modified Gram-Schmidt.
///
/// Degenerate directions are skipped. An empty input basis is valid.
pub fn orthonormalize(
    directions: &[Vec<f64>],
    width: usize,
    tolerance: f64,
) -> Result<Vec<Vec<f64>>, ConceptGeometryError> {
    valid_tolerance(tolerance)?;
    if width == 0 {
        return Err(ConceptGeometryError::EmptyVector);
    }
    let mut basis = Vec::<Vec<f64>>::new();
    for direction in directions {
        validate_vector(direction)?;
        if direction.len() != width {
            return Err(ConceptGeometryError::DimensionMismatch);
        }
        let mut candidate = direction.clone();
        remove_basis_components(&mut candidate, &basis);
        let candidate_norm = norm(&candidate);
        if !candidate_norm.is_finite() {
            return Err(ConceptGeometryError::NonFiniteValue);
        }
        if candidate_norm > tolerance {
            for value in &mut candidate {
                *value /= candidate_norm;
            }
            basis.push(candidate);
        }
    }
    Ok(basis)
}

/// Remove the span of `basis_directions` from `vector`.
///
/// If the residual norm is at or below `tolerance`, `unit_direction` is
/// `None`. This explicitly represents the case in which normalisation would
/// divide by zero or amplify numerical noise.
pub fn residualize(
    vector: &[f64],
    basis_directions: &[Vec<f64>],
    tolerance: f64,
) -> Result<ResidualDirection, ConceptGeometryError> {
    valid_tolerance(tolerance)?;
    validate_vector(vector)?;
    let raw_norm = norm(vector);
    if !raw_norm.is_finite() {
        return Err(ConceptGeometryError::NonFiniteValue);
    }
    if raw_norm <= tolerance {
        return Err(ConceptGeometryError::ZeroNorm);
    }

    let basis = orthonormalize(basis_directions, vector.len(), tolerance)?;
    let mut residual = vector.to_vec();
    remove_basis_components(&mut residual, &basis);
    let residual_norm = norm(&residual);
    if !residual_norm.is_finite() {
        return Err(ConceptGeometryError::NonFiniteValue);
    }
    let innovation_energy_ratio = (residual_norm * residual_norm) / (raw_norm * raw_norm);
    let unit_direction = if residual_norm > tolerance {
        Some(residual.iter().map(|value| value / residual_norm).collect())
    } else {
        None
    };

    Ok(ResidualDirection {
        raw: vector.to_vec(),
        residual,
        unit_direction,
        raw_norm,
        residual_norm,
        innovation_energy_ratio,
        basis_rank: basis.len(),
    })
}

#[derive(Clone, Debug, PartialEq)]
pub struct BootstrapResidualGeometry {
    mean_contrast: BootstrapMeanContrast,
    residual: ResidualDirection,
}

impl BootstrapResidualGeometry {
    #[must_use]
    pub fn mean_contrast(&self) -> &BootstrapMeanContrast {
        &self.mean_contrast
    }

    #[must_use]
    pub fn residual(&self) -> &ResidualDirection {
        &self.residual
    }
}

/// Apply one declared nuisance/known subspace to every bootstrap mean contrast.
///
/// The same basis and tolerance are used for every replicate. A replicate with
/// a zero raw contrast is rejected by `residualize` rather than being dropped
/// from the bootstrap distribution. A fully explained non-zero contrast remains
/// representable with zero residual energy and no unit residual direction.
pub fn bootstrap_residual_geometries(
    positive: &[Vec<f64>],
    control: &[Vec<f64>],
    basis_directions: &[Vec<f64>],
    tolerance: f64,
    plan: DevelopmentResamplingPlan,
) -> Result<Vec<BootstrapResidualGeometry>, ConceptGeometryError> {
    valid_tolerance(tolerance)?;
    let contrasts = bootstrap_mean_contrasts(positive, control, plan)?;
    let mut output = Vec::new();
    output
        .try_reserve_exact(contrasts.len())
        .map_err(|_| ConceptGeometryError::ReplicateAccountingOverflow)?;

    for mean_contrast in contrasts {
        let residual = residualize(mean_contrast.contrast(), basis_directions, tolerance)?;
        output.push(BootstrapResidualGeometry {
            mean_contrast,
            residual,
        });
    }
    Ok(output)
}

#[derive(Clone, Debug, PartialEq)]
pub struct BootstrapDirectionStability {
    reference_unit_direction: Vec<f64>,
    replicate_cosines: Vec<Option<f64>>,
}

impl BootstrapDirectionStability {
    #[must_use]
    pub fn reference_unit_direction(&self) -> &[f64] {
        &self.reference_unit_direction
    }

    #[must_use]
    pub fn replicate_cosines(&self) -> &[Option<f64>] {
        &self.replicate_cosines
    }
}

/// Compare every bootstrap residual direction with the signed full-sample residual.
///
/// The positive-minus-control sign convention is preserved. Replicates whose
/// non-zero raw contrast is fully explained by the declared basis remain in the
/// result as `None` rather than being dropped. A full-sample residual with no
/// defined unit direction is a typed blocker for directional-stability analysis.
pub fn bootstrap_direction_stability(
    positive: &[Vec<f64>],
    control: &[Vec<f64>],
    basis_directions: &[Vec<f64>],
    tolerance: f64,
    plan: DevelopmentResamplingPlan,
) -> Result<BootstrapDirectionStability, ConceptGeometryError> {
    valid_tolerance(tolerance)?;
    let reference_contrast = mean_difference(positive, control)?;
    let reference_residual = residualize(&reference_contrast, basis_directions, tolerance)?;
    let reference_unit_direction = reference_residual
        .unit_direction()
        .ok_or(ConceptGeometryError::UndefinedReferenceDirection)?
        .to_vec();

    let bootstrap =
        bootstrap_residual_geometries(positive, control, basis_directions, tolerance, plan)?;
    let mut replicate_cosines = Vec::new();
    replicate_cosines
        .try_reserve_exact(bootstrap.len())
        .map_err(|_| ConceptGeometryError::ReplicateAccountingOverflow)?;

    for replicate in bootstrap {
        let cosine = match replicate.residual().unit_direction() {
            Some(unit) => Some(cosine_similarity(
                &reference_unit_direction,
                unit,
                tolerance,
            )?),
            None => None,
        };
        replicate_cosines.push(cosine);
    }

    Ok(BootstrapDirectionStability {
        reference_unit_direction,
        replicate_cosines,
    })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DevelopmentOrderInterval {
    lower_rank: usize,
    upper_rank: usize,
    lower_value: f64,
    upper_value: f64,
}

impl DevelopmentOrderInterval {
    #[must_use]
    pub const fn lower_rank(self) -> usize {
        self.lower_rank
    }

    #[must_use]
    pub const fn upper_rank(self) -> usize {
        self.upper_rank
    }

    #[must_use]
    pub const fn lower_value(self) -> f64 {
        self.lower_value
    }

    #[must_use]
    pub const fn upper_value(self) -> f64 {
        self.upper_value
    }
}

fn sorted_finite_scalars(values: &[f64]) -> Result<Vec<f64>, ConceptGeometryError> {
    if values.is_empty() {
        return Err(ConceptGeometryError::EmptyScalarSample);
    }
    if values.iter().any(|value| !value.is_finite()) {
        return Err(ConceptGeometryError::NonFiniteValue);
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    Ok(sorted)
}

/// Select one zero-based order statistic from finite Development values.
///
/// The caller supplies the rank directly. This primitive deliberately does not
/// choose a probability-to-rank convention or attach confidence semantics.
pub fn development_order_statistic(
    values: &[f64],
    rank: usize,
) -> Result<f64, ConceptGeometryError> {
    let sorted = sorted_finite_scalars(values)?;
    sorted
        .get(rank)
        .copied()
        .ok_or(ConceptGeometryError::InvalidOrderStatisticRank)
}

/// Select a closed interval between two caller-supplied zero-based ranks.
///
/// This is an order-statistic container only. It does not define a confidence
/// level, bootstrap coverage rule, interpolation method, or scientific verdict.
pub fn development_order_interval(
    values: &[f64],
    lower_rank: usize,
    upper_rank: usize,
) -> Result<DevelopmentOrderInterval, ConceptGeometryError> {
    if lower_rank > upper_rank {
        return Err(ConceptGeometryError::InvalidOrderStatisticRanks);
    }

    let sorted = sorted_finite_scalars(values)?;
    let lower_value = sorted
        .get(lower_rank)
        .copied()
        .ok_or(ConceptGeometryError::InvalidOrderStatisticRank)?;
    let upper_value = sorted
        .get(upper_rank)
        .copied()
        .ok_or(ConceptGeometryError::InvalidOrderStatisticRank)?;

    Ok(DevelopmentOrderInterval {
        lower_rank,
        upper_rank,
        lower_value,
        upper_value,
    })
}

#[derive(Clone, Debug, PartialEq)]
pub struct BootstrapInnovationEnergySummary {
    full_sample_value: f64,
    replicate_values: Vec<f64>,
    order_interval: DevelopmentOrderInterval,
}

impl BootstrapInnovationEnergySummary {
    #[must_use]
    pub const fn full_sample_value(&self) -> f64 {
        self.full_sample_value
    }

    #[must_use]
    pub fn replicate_values(&self) -> &[f64] {
        &self.replicate_values
    }

    #[must_use]
    pub const fn order_interval(&self) -> DevelopmentOrderInterval {
        self.order_interval
    }
}

/// Summarize bootstrap innovation-energy values with caller-supplied ranks.
///
/// The returned interval is only an interval of ordered bootstrap values. This
/// function does not assign confidence coverage, convert probabilities to
/// ranks, choose an interpolation rule, or classify the scientific result.
pub fn bootstrap_innovation_energy_summary(
    positive: &[Vec<f64>],
    control: &[Vec<f64>],
    basis_directions: &[Vec<f64>],
    tolerance: f64,
    plan: DevelopmentResamplingPlan,
    lower_rank: usize,
    upper_rank: usize,
) -> Result<BootstrapInnovationEnergySummary, ConceptGeometryError> {
    valid_tolerance(tolerance)?;

    let full_sample_contrast = mean_difference(positive, control)?;
    let full_sample_residual = residualize(&full_sample_contrast, basis_directions, tolerance)?;
    let full_sample_value = full_sample_residual.innovation_energy_ratio();

    let bootstrap =
        bootstrap_residual_geometries(positive, control, basis_directions, tolerance, plan)?;
    let replicate_values = bootstrap
        .iter()
        .map(|report| report.residual().innovation_energy_ratio())
        .collect::<Vec<_>>();
    let order_interval = development_order_interval(&replicate_values, lower_rank, upper_rank)?;

    Ok(BootstrapInnovationEnergySummary {
        full_sample_value,
        replicate_values,
        order_interval,
    })
}

#[derive(Clone, Debug, PartialEq)]
pub struct NullInnovationEnergyComparison {
    observed_value: f64,
    defined_null_values: Vec<f64>,
    undefined_replicates: usize,
    greater_or_equal_count: usize,
    total_replicates: usize,
}

impl NullInnovationEnergyComparison {
    #[must_use]
    pub const fn observed_value(&self) -> f64 {
        self.observed_value
    }

    #[must_use]
    pub fn defined_null_values(&self) -> &[f64] {
        &self.defined_null_values
    }

    #[must_use]
    pub const fn undefined_replicates(&self) -> usize {
        self.undefined_replicates
    }

    #[must_use]
    pub const fn greater_or_equal_count(&self) -> usize {
        self.greater_or_equal_count
    }

    #[must_use]
    pub const fn total_replicates(&self) -> usize {
        self.total_replicates
    }
}

/// Descriptively compare one observed scalar with an optional null sample.
///
/// No normalization is performed: the output is counts plus the exact defined
/// null values. In particular, this is not a p-value and has no rejection rule.
pub fn summarize_null_innovation_energy(
    observed_value: f64,
    null_values: &[Option<f64>],
) -> Result<NullInnovationEnergyComparison, ConceptGeometryError> {
    if !observed_value.is_finite() {
        return Err(ConceptGeometryError::NonFiniteValue);
    }
    if null_values.is_empty() {
        return Err(ConceptGeometryError::EmptyScalarSample);
    }

    let mut defined_null_values = Vec::new();
    defined_null_values
        .try_reserve_exact(null_values.len())
        .map_err(|_| ConceptGeometryError::ReplicateAccountingOverflow)?;
    let mut undefined_replicates = 0usize;
    let mut greater_or_equal_count = 0usize;

    for value in null_values {
        match value {
            Some(value) => {
                if !value.is_finite() {
                    return Err(ConceptGeometryError::NonFiniteValue);
                }
                if *value >= observed_value {
                    greater_or_equal_count = greater_or_equal_count
                        .checked_add(1)
                        .ok_or(ConceptGeometryError::ReplicateAccountingOverflow)?;
                }
                defined_null_values.push(*value);
            }
            None => {
                undefined_replicates = undefined_replicates
                    .checked_add(1)
                    .ok_or(ConceptGeometryError::ReplicateAccountingOverflow)?;
            }
        }
    }

    Ok(NullInnovationEnergyComparison {
        observed_value,
        defined_null_values,
        undefined_replicates,
        greater_or_equal_count,
        total_replicates: null_values.len(),
    })
}

/// Build a descriptive shuffled-label null comparison for innovation energy.
pub fn compare_innovation_energy_to_shuffled_null(
    positive: &[Vec<f64>],
    control: &[Vec<f64>],
    basis_directions: &[Vec<f64>],
    tolerance: f64,
    plan: DevelopmentResamplingPlan,
) -> Result<NullInnovationEnergyComparison, ConceptGeometryError> {
    valid_tolerance(tolerance)?;
    let observed_contrast = mean_difference(positive, control)?;
    let observed_residual = residualize(&observed_contrast, basis_directions, tolerance)?;
    let observed_value = observed_residual.innovation_energy_ratio();

    let null_geometries =
        label_shuffle_residual_geometries(positive, control, basis_directions, tolerance, plan)?;
    let null_values = null_geometries
        .iter()
        .map(|report| {
            report
                .residual()
                .map(ResidualDirection::innovation_energy_ratio)
        })
        .collect::<Vec<_>>();

    summarize_null_innovation_energy(observed_value, &null_values)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DevelopmentSampleSizePoint {
    positive_sample_count: usize,
    control_sample_count: usize,
}

impl DevelopmentSampleSizePoint {
    pub fn new(
        positive_sample_count: usize,
        control_sample_count: usize,
    ) -> Result<Self, ConceptGeometryError> {
        if positive_sample_count == 0 || control_sample_count == 0 {
            return Err(ConceptGeometryError::InvalidSubsampleSize);
        }
        Ok(Self {
            positive_sample_count,
            control_sample_count,
        })
    }

    #[must_use]
    pub const fn positive_sample_count(self) -> usize {
        self.positive_sample_count
    }

    #[must_use]
    pub const fn control_sample_count(self) -> usize {
        self.control_sample_count
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SampleSizeSensitivityCell {
    sample_size: DevelopmentSampleSizePoint,
    replicate_innovation_values: Vec<Option<f64>>,
}

impl SampleSizeSensitivityCell {
    #[must_use]
    pub const fn sample_size(&self) -> DevelopmentSampleSizePoint {
        self.sample_size
    }

    #[must_use]
    pub fn replicate_innovation_values(&self) -> &[Option<f64>] {
        &self.replicate_innovation_values
    }
}

/// Evaluate caller-declared sample-size points under deterministic subsampling.
///
/// The grid order is preserved exactly. No sample-size point is generated,
/// selected, optimized, or classified by this function.
pub fn sample_size_sensitivity_grid(
    positive: &[Vec<f64>],
    control: &[Vec<f64>],
    basis_directions: &[Vec<f64>],
    tolerance: f64,
    plan: DevelopmentResamplingPlan,
    sample_sizes: &[DevelopmentSampleSizePoint],
) -> Result<Vec<SampleSizeSensitivityCell>, ConceptGeometryError> {
    valid_tolerance(tolerance)?;
    let positive_width = validate_rows(positive)?;
    let control_width = validate_rows(control)?;
    if positive_width != control_width {
        return Err(ConceptGeometryError::DimensionMismatch);
    }
    if sample_sizes.is_empty() {
        return Err(ConceptGeometryError::EmptySampleSizeGrid);
    }

    let mut cells = Vec::new();
    cells
        .try_reserve_exact(sample_sizes.len())
        .map_err(|_| ConceptGeometryError::ReplicateAccountingOverflow)?;

    for &sample_size in sample_sizes {
        let subsets = independent_group_subsample_indices(
            positive.len(),
            control.len(),
            sample_size.positive_sample_count(),
            sample_size.control_sample_count(),
            plan,
        )?;
        let mut replicate_innovation_values = Vec::new();
        replicate_innovation_values
            .try_reserve_exact(subsets.len())
            .map_err(|_| ConceptGeometryError::ReplicateAccountingOverflow)?;

        for subset in subsets {
            let positive_rows = subset
                .positive()
                .iter()
                .map(|&index| positive[index].clone())
                .collect::<Vec<_>>();
            let control_rows = subset
                .control()
                .iter()
                .map(|&index| control[index].clone())
                .collect::<Vec<_>>();
            let contrast = mean_difference(&positive_rows, &control_rows)?;
            let raw_norm = norm(&contrast);
            if !raw_norm.is_finite() {
                return Err(ConceptGeometryError::NonFiniteValue);
            }
            let value = if raw_norm <= tolerance {
                None
            } else {
                Some(
                    residualize(&contrast, basis_directions, tolerance)?
                        .innovation_energy_ratio(),
                )
            };
            replicate_innovation_values.push(value);
        }

        cells.push(SampleSizeSensitivityCell {
            sample_size,
            replicate_innovation_values,
        });
    }

    Ok(cells)
}

pub fn cosine_similarity(
    left: &[f64],
    right: &[f64],
    tolerance: f64,
) -> Result<f64, ConceptGeometryError> {
    valid_tolerance(tolerance)?;
    validate_vector(left)?;
    validate_vector(right)?;
    if left.len() != right.len() {
        return Err(ConceptGeometryError::DimensionMismatch);
    }
    let left_norm = norm(left);
    let right_norm = norm(right);
    if left_norm <= tolerance || right_norm <= tolerance {
        return Err(ConceptGeometryError::ZeroNorm);
    }
    Ok(dot(left, right) / (left_norm * right_norm))
}

/// Target intervention effect minus the mean matched-control effect.
pub fn causal_novelty_gap(
    target_effect: f64,
    matched_control_effects: &[f64],
) -> Result<f64, ConceptGeometryError> {
    if !target_effect.is_finite()
        || matched_control_effects
            .iter()
            .any(|effect| !effect.is_finite())
    {
        return Err(ConceptGeometryError::NonFiniteValue);
    }
    if matched_control_effects.is_empty() {
        return Err(ConceptGeometryError::EmptyControls);
    }
    let control_mean =
        matched_control_effects.iter().sum::<f64>() / matched_control_effects.len() as f64;
    Ok(target_effect - control_mean)
}

/// Non-additive interaction when all effects are measured from one baseline.
pub fn interaction_residual(
    effect_a: f64,
    effect_b: f64,
    effect_ab: f64,
) -> Result<f64, ConceptGeometryError> {
    if !effect_a.is_finite() || !effect_b.is_finite() || !effect_ab.is_finite() {
        return Err(ConceptGeometryError::NonFiniteValue);
    }
    Ok(effect_ab - effect_a - effect_b)
}

/// Sequentially remove all previously accepted innovation directions.
pub fn sequential_innovations(
    directions: &[Vec<f64>],
    tolerance: f64,
) -> Result<Vec<SequentialInnovationStep>, ConceptGeometryError> {
    valid_tolerance(tolerance)?;
    let first = directions.first().ok_or(ConceptGeometryError::EmptyGroup)?;
    validate_vector(first)?;
    let width = first.len();
    let mut basis = Vec::<Vec<f64>>::new();
    let mut steps = Vec::with_capacity(directions.len());

    for (index, direction) in directions.iter().enumerate() {
        validate_vector(direction)?;
        if direction.len() != width {
            return Err(ConceptGeometryError::DimensionMismatch);
        }
        let report = residualize(direction, &basis, tolerance)?;
        let unit_direction = report.unit_direction.clone();
        let accepted = unit_direction.is_some();
        if let Some(unit) = &unit_direction {
            basis.push(unit.clone());
        }
        steps.push(SequentialInnovationStep {
            index,
            raw_norm: report.raw_norm,
            residual_norm: report.residual_norm,
            innovation_energy_ratio: report.innovation_energy_ratio,
            accepted,
            unit_direction,
        });
    }
    Ok(steps)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(left: f64, right: f64) {
        assert!(
            (left - right).abs() < 1.0e-12,
            "left={left:?} right={right:?}"
        );
    }

    #[test]
    fn development_resampling_plan_requires_two_replicates() {
        assert_eq!(
            DevelopmentResamplingPlan::new(1, 7),
            Err(ConceptGeometryError::InvalidReplicateCount)
        );
        let plan = DevelopmentResamplingPlan::new(4, 7).unwrap();
        assert_eq!(plan.replicates(), 4);
        assert_eq!(plan.seed(), 7);
    }

    #[test]
    fn bootstrap_indices_are_deterministic_bounded_and_complete() {
        let plan = DevelopmentResamplingPlan::new(5, 0x27_01).unwrap();
        let left = bootstrap_index_replicates(7, plan).unwrap();
        let right = bootstrap_index_replicates(7, plan).unwrap();
        assert_eq!(left, right);
        assert_eq!(left.len(), 5);
        assert!(left.iter().all(|replicate| replicate.len() == 7));
        assert!(left.iter().flatten().all(|index| *index < 7));
    }

    #[test]
    fn changing_resampling_seed_changes_draws() {
        let left =
            bootstrap_index_replicates(8, DevelopmentResamplingPlan::new(4, 1).unwrap()).unwrap();
        let right =
            bootstrap_index_replicates(8, DevelopmentResamplingPlan::new(4, 2).unwrap()).unwrap();
        assert_ne!(left, right);
    }

    #[test]
    fn independent_group_bootstrap_preserves_group_sizes_and_bounds() {
        let plan = DevelopmentResamplingPlan::new(6, 0x27_02).unwrap();
        let replicates = independent_group_bootstrap_indices(4, 7, plan).unwrap();
        assert_eq!(replicates.len(), 6);
        assert!(
            replicates
                .iter()
                .all(|replicate| replicate.positive().len() == 4)
        );
        assert!(
            replicates
                .iter()
                .all(|replicate| replicate.control().len() == 7)
        );
        assert!(
            replicates
                .iter()
                .flat_map(IndependentGroupBootstrapIndices::positive)
                .all(|index| *index < 4)
        );
        assert!(
            replicates
                .iter()
                .flat_map(IndependentGroupBootstrapIndices::control)
                .all(|index| *index < 7)
        );
    }

    #[test]
    fn independent_group_bootstrap_is_deterministic() {
        let plan = DevelopmentResamplingPlan::new(5, 0xfeed_2702).unwrap();
        let left = independent_group_bootstrap_indices(5, 9, plan).unwrap();
        let right = independent_group_bootstrap_indices(5, 9, plan).unwrap();
        assert_eq!(left, right);
    }

    #[test]
    fn group_streams_are_cardinality_isolated() {
        let plan = DevelopmentResamplingPlan::new(4, 0xabc0_2702).unwrap();
        let baseline = independent_group_bootstrap_indices(5, 7, plan).unwrap();
        let wider_control = independent_group_bootstrap_indices(5, 11, plan).unwrap();
        let wider_positive = independent_group_bootstrap_indices(9, 7, plan).unwrap();

        for (left, right) in baseline.iter().zip(&wider_control) {
            assert_eq!(left.positive(), right.positive());
        }
        for (left, right) in baseline.iter().zip(&wider_positive) {
            assert_eq!(left.control(), right.control());
        }
    }

    #[test]
    fn independent_group_bootstrap_rejects_empty_groups() {
        let plan = DevelopmentResamplingPlan::new(2, 17).unwrap();
        assert_eq!(
            independent_group_bootstrap_indices(0, 4, plan),
            Err(ConceptGeometryError::EmptyGroup)
        );
        assert_eq!(
            independent_group_bootstrap_indices(4, 0, plan),
            Err(ConceptGeometryError::EmptyGroup)
        );
    }

    #[test]
    fn bootstrap_mean_contrasts_preserve_sign_and_group_means() {
        let positive = vec![
            vec![3.0, 4.0, 1.0],
            vec![3.0, 4.0, 1.0],
            vec![3.0, 4.0, 1.0],
        ];
        let control = vec![
            vec![1.0, 1.0, 1.0],
            vec![1.0, 1.0, 1.0],
            vec![1.0, 1.0, 1.0],
            vec![1.0, 1.0, 1.0],
        ];
        let plan = DevelopmentResamplingPlan::new(5, 0x2701_0003).unwrap();

        let left = bootstrap_mean_contrasts(&positive, &control, plan).unwrap();
        let right = bootstrap_mean_contrasts(&positive, &control, plan).unwrap();
        assert_eq!(left, right);
        assert_eq!(left.len(), 5);

        for replicate in left {
            assert_eq!(replicate.positive_mean(), &[3.0, 4.0, 1.0]);
            assert_eq!(replicate.control_mean(), &[1.0, 1.0, 1.0]);
            assert_eq!(replicate.contrast(), &[2.0, 3.0, 0.0]);
        }
    }

    #[test]
    fn bootstrap_mean_contrasts_match_manual_index_reconstruction() {
        let positive = vec![vec![1.0, 2.0], vec![3.0, 5.0], vec![8.0, 13.0]];
        let control = vec![
            vec![0.0, 1.0],
            vec![2.0, 3.0],
            vec![5.0, 8.0],
            vec![13.0, 21.0],
        ];
        let plan = DevelopmentResamplingPlan::new(4, 0x2701_0303).unwrap();

        let indices =
            independent_group_bootstrap_indices(positive.len(), control.len(), plan).unwrap();
        let reports = bootstrap_mean_contrasts(&positive, &control, plan).unwrap();

        for (replicate, report) in indices.iter().zip(&reports) {
            let positive_rows = replicate
                .positive()
                .iter()
                .map(|&index| positive[index].clone())
                .collect::<Vec<_>>();
            let control_rows = replicate
                .control()
                .iter()
                .map(|&index| control[index].clone())
                .collect::<Vec<_>>();
            let expected = mean_difference(&positive_rows, &control_rows).unwrap();
            assert_eq!(report.contrast(), expected);
        }
    }

    #[test]
    fn bootstrap_mean_contrasts_reject_dimension_mismatch() {
        let positive = vec![vec![1.0, 2.0]];
        let control = vec![vec![1.0, 2.0, 3.0]];
        let plan = DevelopmentResamplingPlan::new(2, 0x2701_0304).unwrap();

        assert_eq!(
            bootstrap_mean_contrasts(&positive, &control, plan),
            Err(ConceptGeometryError::DimensionMismatch)
        );
    }

    #[test]
    fn bootstrap_mean_contrasts_fail_closed_on_derived_overflow() {
        let positive = vec![vec![f64::MAX], vec![f64::MAX]];
        let control = vec![vec![0.0], vec![0.0]];
        let plan = DevelopmentResamplingPlan::new(2, 0x2701_0305).unwrap();

        assert_eq!(
            bootstrap_mean_contrasts(&positive, &control, plan),
            Err(ConceptGeometryError::NonFiniteValue)
        );
    }

    #[test]
    fn label_shuffle_is_deterministic_and_preserves_a_complete_partition() {
        let plan = DevelopmentResamplingPlan::new(5, 0x2701_0801).unwrap();
        let left = label_shuffle_indices(3, 4, plan).unwrap();
        let right = label_shuffle_indices(3, 4, plan).unwrap();
        assert_eq!(left, right);
        assert_eq!(left.len(), 5);

        for replicate in left {
            assert_eq!(replicate.positive().len(), 3);
            assert_eq!(replicate.control().len(), 4);

            let mut pooled = replicate
                .positive()
                .iter()
                .chain(replicate.control())
                .copied()
                .collect::<Vec<_>>();
            pooled.sort_unstable();
            assert_eq!(pooled, (0..7).collect::<Vec<_>>());
        }
    }

    #[test]
    fn label_shuffle_changes_with_seed_and_rejects_empty_groups() {
        let left =
            label_shuffle_indices(4, 4, DevelopmentResamplingPlan::new(3, 1).unwrap()).unwrap();
        let right =
            label_shuffle_indices(4, 4, DevelopmentResamplingPlan::new(3, 2).unwrap()).unwrap();
        assert_ne!(left, right);

        let plan = DevelopmentResamplingPlan::new(2, 9).unwrap();
        assert_eq!(
            label_shuffle_indices(0, 4, plan),
            Err(ConceptGeometryError::EmptyGroup)
        );
        assert_eq!(
            label_shuffle_indices(4, 0, plan),
            Err(ConceptGeometryError::EmptyGroup)
        );
    }

    #[test]
    fn label_shuffle_contrasts_match_manual_pooled_reconstruction() {
        let positive = vec![vec![4.0, 8.0], vec![6.0, 10.0]];
        let control = vec![vec![0.0, 2.0], vec![2.0, 4.0], vec![8.0, 12.0]];
        let plan = DevelopmentResamplingPlan::new(4, 0x2701_0901).unwrap();
        let reports = label_shuffle_mean_contrasts(&positive, &control, plan).unwrap();

        let pooled = positive.iter().chain(&control).cloned().collect::<Vec<_>>();

        assert_eq!(reports.len(), 4);
        for report in reports {
            let shuffled_positive = report
                .positive_indices()
                .iter()
                .map(|&index| pooled[index].clone())
                .collect::<Vec<_>>();
            let shuffled_control = report
                .control_indices()
                .iter()
                .map(|&index| pooled[index].clone())
                .collect::<Vec<_>>();
            assert_eq!(
                report.contrast(),
                mean_difference(&shuffled_positive, &shuffled_control).unwrap()
            );
        }
    }

    #[test]
    fn label_shuffle_contrasts_are_deterministic_and_dimension_checked() {
        let positive = vec![vec![1.0, 3.0], vec![2.0, 4.0]];
        let control = vec![vec![5.0, 7.0], vec![6.0, 8.0]];
        let plan = DevelopmentResamplingPlan::new(3, 0x2701_0902).unwrap();
        assert_eq!(
            label_shuffle_mean_contrasts(&positive, &control, plan).unwrap(),
            label_shuffle_mean_contrasts(&positive, &control, plan).unwrap()
        );

        assert_eq!(
            label_shuffle_mean_contrasts(&[vec![1.0, 2.0]], &[vec![1.0, 2.0, 3.0]], plan,),
            Err(ConceptGeometryError::DimensionMismatch)
        );
    }

    #[test]
    fn shuffled_null_geometry_retains_zero_contrasts_as_undefined() {
        let positive = vec![vec![1.0, 1.0], vec![1.0, 1.0]];
        let control = vec![vec![1.0, 1.0], vec![1.0, 1.0]];
        let plan = DevelopmentResamplingPlan::new(4, 0x2701_1001).unwrap();

        let reports =
            label_shuffle_residual_geometries(&positive, &control, &[], DEFAULT_TOLERANCE, plan)
                .unwrap();

        assert_eq!(reports.len(), 4);
        for report in reports {
            assert_eq!(report.contrast(), &[0.0, 0.0]);
            assert!(report.residual().is_none());
            assert_eq!(report.positive_indices().len(), 2);
            assert_eq!(report.control_indices().len(), 2);
        }
    }

    #[test]
    fn shuffled_null_geometry_matches_direct_residualization_when_defined() {
        let positive = vec![vec![0.0, 0.0], vec![2.0, 1.0], vec![4.0, 3.0]];
        let control = vec![vec![1.0, 4.0], vec![3.0, 2.0], vec![5.0, 6.0]];
        let basis = vec![vec![1.0, 0.0]];
        let plan = DevelopmentResamplingPlan::new(5, 0x2701_1002).unwrap();

        let reports =
            label_shuffle_residual_geometries(&positive, &control, &basis, DEFAULT_TOLERANCE, plan)
                .unwrap();

        for report in reports {
            if norm(report.contrast()) > DEFAULT_TOLERANCE {
                let expected = residualize(report.contrast(), &basis, DEFAULT_TOLERANCE).unwrap();
                assert_eq!(report.residual(), Some(&expected));
            } else {
                assert!(report.residual().is_none());
            }
        }
    }

    #[test]
    fn independent_subsamples_are_deterministic_unique_and_bounded() {
        let plan = DevelopmentResamplingPlan::new(5, 0x2701_1201).unwrap();
        let left = independent_group_subsample_indices(7, 9, 4, 5, plan).unwrap();
        let right = independent_group_subsample_indices(7, 9, 4, 5, plan).unwrap();
        assert_eq!(left, right);
        assert_eq!(left.len(), 5);

        for replicate in left {
            assert_eq!(replicate.positive().len(), 4);
            assert_eq!(replicate.control().len(), 5);
            assert!(replicate.positive().iter().all(|index| *index < 7));
            assert!(replicate.control().iter().all(|index| *index < 9));

            let mut positive = replicate.positive().to_vec();
            positive.sort_unstable();
            positive.dedup();
            assert_eq!(positive.len(), 4);

            let mut control = replicate.control().to_vec();
            control.sort_unstable();
            control.dedup();
            assert_eq!(control.len(), 5);
        }
    }

    #[test]
    fn subsample_group_streams_are_cardinality_isolated() {
        let plan = DevelopmentResamplingPlan::new(4, 0x2701_1202).unwrap();
        let baseline = independent_group_subsample_indices(8, 10, 4, 5, plan).unwrap();
        let different_control = independent_group_subsample_indices(8, 10, 4, 7, plan).unwrap();
        let different_positive = independent_group_subsample_indices(8, 10, 6, 5, plan).unwrap();

        for (left, right) in baseline.iter().zip(&different_control) {
            assert_eq!(left.positive(), right.positive());
        }
        for (left, right) in baseline.iter().zip(&different_positive) {
            assert_eq!(left.control(), right.control());
        }
    }

    #[test]
    fn independent_subsamples_reject_invalid_sizes() {
        let plan = DevelopmentResamplingPlan::new(2, 11).unwrap();
        assert_eq!(
            independent_group_subsample_indices(4, 5, 0, 3, plan),
            Err(ConceptGeometryError::InvalidSubsampleSize)
        );
        assert_eq!(
            independent_group_subsample_indices(4, 5, 5, 3, plan),
            Err(ConceptGeometryError::InvalidSubsampleSize)
        );
        assert_eq!(
            independent_group_subsample_indices(4, 5, 3, 6, plan),
            Err(ConceptGeometryError::InvalidSubsampleSize)
        );
    }

    #[test]
    fn mean_contrast_preserves_declared_sign() {
        let control = vec![vec![0.0, 1.0], vec![2.0, 3.0]];
        let positive = vec![vec![3.0, 5.0], vec![5.0, 7.0]];
        let contrast = mean_difference(&positive, &control).unwrap();
        assert_eq!(contrast, vec![3.0, 4.0]);
    }

    #[test]
    fn residual_energy_matches_analytic_case() {
        let report =
            residualize(&[3.0, 4.0, 0.0], &[vec![1.0, 0.0, 0.0]], DEFAULT_TOLERANCE).unwrap();
        close(report.raw_norm(), 5.0);
        close(report.residual_norm(), 4.0);
        close(report.innovation_energy_ratio(), 16.0 / 25.0);
        assert_eq!(report.basis_rank(), 1);
        assert_eq!(report.unit_direction().unwrap(), &[0.0, 1.0, 0.0]);
    }

    #[test]
    fn fully_explained_direction_has_no_unit_residual() {
        let report = residualize(&[2.0, 0.0], &[vec![1.0, 0.0]], DEFAULT_TOLERANCE).unwrap();
        close(report.residual_norm(), 0.0);
        close(report.innovation_energy_ratio(), 0.0);
        assert!(report.unit_direction().is_none());
    }

    #[test]
    fn non_orthogonal_inputs_do_not_duplicate_basis_rank() {
        let basis = orthonormalize(
            &[
                vec![1.0, 0.0, 0.0],
                vec![2.0, 0.0, 0.0],
                vec![1.0, 1.0, 0.0],
            ],
            3,
            DEFAULT_TOLERANCE,
        )
        .unwrap();
        assert_eq!(basis.len(), 2);
        close(dot(&basis[0], &basis[1]), 0.0);
        close(norm(&basis[0]), 1.0);
        close(norm(&basis[1]), 1.0);
    }

    #[test]
    fn sequential_innovation_extracts_three_independent_axes() {
        let steps = sequential_innovations(
            &[
                vec![1.0, 0.0, 0.0],
                vec![1.0, 1.0, 0.0],
                vec![2.0, 2.0, 1.0],
            ],
            DEFAULT_TOLERANCE,
        )
        .unwrap();
        assert_eq!(steps.len(), 3);
        assert!(steps.iter().all(SequentialInnovationStep::accepted));
        close(steps[0].innovation_energy_ratio(), 1.0);
        close(steps[1].innovation_energy_ratio(), 0.5);
        close(steps[2].innovation_energy_ratio(), 1.0 / 9.0);
    }

    #[test]
    fn bootstrap_residual_geometry_matches_analytic_constant_groups() {
        let positive = vec![
            vec![3.0, 4.0, 1.0],
            vec![3.0, 4.0, 1.0],
            vec![3.0, 4.0, 1.0],
        ];
        let control = vec![
            vec![1.0, 1.0, 1.0],
            vec![1.0, 1.0, 1.0],
            vec![1.0, 1.0, 1.0],
        ];
        let basis = vec![vec![1.0, 0.0, 0.0]];
        let plan = DevelopmentResamplingPlan::new(4, 0x2701_0401).unwrap();

        let reports =
            bootstrap_residual_geometries(&positive, &control, &basis, DEFAULT_TOLERANCE, plan)
                .unwrap();

        assert_eq!(reports.len(), 4);
        for report in reports {
            assert_eq!(report.mean_contrast().contrast(), &[2.0, 3.0, 0.0]);
            close(report.residual().raw_norm(), 13.0_f64.sqrt());
            close(report.residual().residual_norm(), 3.0);
            close(report.residual().innovation_energy_ratio(), 9.0 / 13.0);
            assert_eq!(
                report.residual().unit_direction().unwrap(),
                &[0.0, 1.0, 0.0]
            );
        }
    }

    #[test]
    fn bootstrap_residual_geometry_preserves_fully_explained_replicates() {
        let positive = vec![vec![2.0, 0.0], vec![2.0, 0.0]];
        let control = vec![vec![0.0, 0.0], vec![0.0, 0.0]];
        let basis = vec![vec![1.0, 0.0]];
        let plan = DevelopmentResamplingPlan::new(3, 0x2701_0402).unwrap();

        let reports =
            bootstrap_residual_geometries(&positive, &control, &basis, DEFAULT_TOLERANCE, plan)
                .unwrap();

        for report in reports {
            close(report.residual().innovation_energy_ratio(), 0.0);
            assert!(report.residual().unit_direction().is_none());
        }
    }

    #[test]
    fn bootstrap_residual_geometry_fails_closed_on_zero_raw_contrast() {
        let positive = vec![vec![1.0, 1.0], vec![1.0, 1.0]];
        let control = vec![vec![1.0, 1.0], vec![1.0, 1.0]];
        let plan = DevelopmentResamplingPlan::new(2, 0x2701_0403).unwrap();

        assert_eq!(
            bootstrap_residual_geometries(&positive, &control, &[], DEFAULT_TOLERANCE, plan,),
            Err(ConceptGeometryError::ZeroNorm)
        );
    }

    #[test]
    fn bootstrap_direction_stability_is_signed_and_complete() {
        let positive = vec![
            vec![3.0, 4.0, 1.0],
            vec![3.0, 4.0, 1.0],
            vec![3.0, 4.0, 1.0],
        ];
        let control = vec![
            vec![1.0, 1.0, 1.0],
            vec![1.0, 1.0, 1.0],
            vec![1.0, 1.0, 1.0],
        ];
        let basis = vec![vec![1.0, 0.0, 0.0]];
        let plan = DevelopmentResamplingPlan::new(5, 0x2701_0501).unwrap();

        let report =
            bootstrap_direction_stability(&positive, &control, &basis, DEFAULT_TOLERANCE, plan)
                .unwrap();

        assert_eq!(report.reference_unit_direction(), &[0.0, 1.0, 0.0]);
        assert_eq!(report.replicate_cosines().len(), 5);
        for cosine in report.replicate_cosines() {
            close(cosine.unwrap(), 1.0);
        }
    }

    #[test]
    fn bootstrap_direction_stability_blocks_undefined_reference_direction() {
        let positive = vec![vec![2.0, 0.0], vec![2.0, 0.0]];
        let control = vec![vec![0.0, 0.0], vec![0.0, 0.0]];
        let basis = vec![vec![1.0, 0.0]];
        let plan = DevelopmentResamplingPlan::new(2, 0x2701_0502).unwrap();

        assert_eq!(
            bootstrap_direction_stability(&positive, &control, &basis, DEFAULT_TOLERANCE, plan,),
            Err(ConceptGeometryError::UndefinedReferenceDirection)
        );
    }

    #[test]
    fn development_order_statistics_use_exact_caller_supplied_ranks() {
        let values = [4.0, 1.0, 3.0, 3.0, 9.0];

        close(development_order_statistic(&values, 0).unwrap(), 1.0);
        close(development_order_statistic(&values, 2).unwrap(), 3.0);
        close(development_order_statistic(&values, 4).unwrap(), 9.0);

        let interval = development_order_interval(&values, 1, 3).unwrap();
        assert_eq!(interval.lower_rank(), 1);
        assert_eq!(interval.upper_rank(), 3);
        close(interval.lower_value(), 3.0);
        close(interval.upper_value(), 4.0);
    }

    #[test]
    fn development_order_statistics_fail_closed_on_invalid_inputs() {
        assert_eq!(
            development_order_statistic(&[], 0),
            Err(ConceptGeometryError::EmptyScalarSample)
        );
        assert_eq!(
            development_order_statistic(&[1.0, f64::NAN], 0),
            Err(ConceptGeometryError::NonFiniteValue)
        );
        assert_eq!(
            development_order_statistic(&[1.0, 2.0], 2),
            Err(ConceptGeometryError::InvalidOrderStatisticRank)
        );
        assert_eq!(
            development_order_interval(&[1.0, 2.0], 1, 0),
            Err(ConceptGeometryError::InvalidOrderStatisticRanks)
        );
        assert_eq!(
            development_order_interval(&[1.0, 2.0], 0, 2),
            Err(ConceptGeometryError::InvalidOrderStatisticRank)
        );
    }

    #[test]
    fn bootstrap_innovation_energy_summary_retains_full_distribution() {
        let positive = vec![
            vec![3.0, 4.0, 1.0],
            vec![3.0, 4.0, 1.0],
            vec![3.0, 4.0, 1.0],
        ];
        let control = vec![
            vec![1.0, 1.0, 1.0],
            vec![1.0, 1.0, 1.0],
            vec![1.0, 1.0, 1.0],
        ];
        let basis = vec![vec![1.0, 0.0, 0.0]];
        let plan = DevelopmentResamplingPlan::new(5, 0x2701_0701).unwrap();

        let summary = bootstrap_innovation_energy_summary(
            &positive,
            &control,
            &basis,
            DEFAULT_TOLERANCE,
            plan,
            1,
            3,
        )
        .unwrap();

        close(summary.full_sample_value(), 9.0 / 13.0);
        assert_eq!(summary.replicate_values().len(), 5);
        for value in summary.replicate_values() {
            close(*value, 9.0 / 13.0);
        }
        close(summary.order_interval().lower_value(), 9.0 / 13.0);
        close(summary.order_interval().upper_value(), 9.0 / 13.0);
        assert_eq!(summary.order_interval().lower_rank(), 1);
        assert_eq!(summary.order_interval().upper_rank(), 3);
    }

    #[test]
    fn bootstrap_innovation_energy_summary_propagates_rank_errors() {
        let positive = vec![vec![2.0, 1.0], vec![2.0, 1.0]];
        let control = vec![vec![0.0, 0.0], vec![0.0, 0.0]];
        let plan = DevelopmentResamplingPlan::new(2, 0x2701_0702).unwrap();

        assert_eq!(
            bootstrap_innovation_energy_summary(
                &positive,
                &control,
                &[],
                DEFAULT_TOLERANCE,
                plan,
                0,
                2,
            ),
            Err(ConceptGeometryError::InvalidOrderStatisticRank)
        );
    }

    #[test]
    fn null_innovation_summary_preserves_defined_and_undefined_accounting() {
        let summary =
            summarize_null_innovation_energy(0.5, &[Some(0.1), None, Some(0.5), Some(0.9)])
                .unwrap();

        close(summary.observed_value(), 0.5);
        assert_eq!(summary.defined_null_values(), &[0.1, 0.5, 0.9]);
        assert_eq!(summary.undefined_replicates(), 1);
        assert_eq!(summary.greater_or_equal_count(), 2);
        assert_eq!(summary.total_replicates(), 4);
    }

    #[test]
    fn shuffled_null_innovation_comparison_is_deterministic_and_complete() {
        let positive = vec![vec![2.0, 1.0], vec![3.0, 2.0], vec![4.0, 4.0]];
        let control = vec![vec![0.0, 0.0], vec![1.0, 2.0], vec![2.0, 3.0]];
        let basis = vec![vec![1.0, 0.0]];
        let plan = DevelopmentResamplingPlan::new(8, 0x2701_1101).unwrap();

        let left = compare_innovation_energy_to_shuffled_null(
            &positive,
            &control,
            &basis,
            DEFAULT_TOLERANCE,
            plan,
        )
        .unwrap();
        let right = compare_innovation_energy_to_shuffled_null(
            &positive,
            &control,
            &basis,
            DEFAULT_TOLERANCE,
            plan,
        )
        .unwrap();

        assert_eq!(left, right);
        assert_eq!(left.total_replicates(), 8);
        assert_eq!(
            left.defined_null_values().len() + left.undefined_replicates(),
            8
        );
        assert!(left.greater_or_equal_count() <= left.defined_null_values().len());
    }

    #[test]
    fn sample_size_sensitivity_preserves_caller_grid_and_replicates() {
        let positive = vec![
            vec![3.0, 4.0, 1.0],
            vec![3.0, 4.0, 1.0],
            vec![3.0, 4.0, 1.0],
            vec![3.0, 4.0, 1.0],
        ];
        let control = vec![
            vec![1.0, 1.0, 1.0],
            vec![1.0, 1.0, 1.0],
            vec![1.0, 1.0, 1.0],
            vec![1.0, 1.0, 1.0],
            vec![1.0, 1.0, 1.0],
        ];
        let basis = vec![vec![1.0, 0.0, 0.0]];
        let plan = DevelopmentResamplingPlan::new(4, 0x2701_1301).unwrap();
        let grid = [
            DevelopmentSampleSizePoint::new(2, 2).unwrap(),
            DevelopmentSampleSizePoint::new(3, 4).unwrap(),
        ];

        let cells = sample_size_sensitivity_grid(
            &positive,
            &control,
            &basis,
            DEFAULT_TOLERANCE,
            plan,
            &grid,
        )
        .unwrap();

        assert_eq!(cells.len(), 2);
        assert_eq!(cells[0].sample_size(), grid[0]);
        assert_eq!(cells[1].sample_size(), grid[1]);
        for cell in cells {
            assert_eq!(cell.replicate_innovation_values().len(), 4);
            for value in cell.replicate_innovation_values() {
                close(value.unwrap(), 9.0 / 13.0);
            }
        }
    }

    #[test]
    fn sample_size_sensitivity_retains_zero_contrast_subsets_as_undefined() {
        let positive = vec![vec![1.0, 1.0], vec![1.0, 1.0]];
        let control = vec![vec![1.0, 1.0], vec![1.0, 1.0]];
        let plan = DevelopmentResamplingPlan::new(3, 0x2701_1302).unwrap();
        let grid = [DevelopmentSampleSizePoint::new(1, 1).unwrap()];

        let cells =
            sample_size_sensitivity_grid(&positive, &control, &[], DEFAULT_TOLERANCE, plan, &grid)
                .unwrap();

        assert_eq!(cells[0].replicate_innovation_values(), &[None, None, None]);
    }

    #[test]
    fn sample_size_sensitivity_rejects_empty_grid() {
        let plan = DevelopmentResamplingPlan::new(2, 0x2701_1303).unwrap();
        assert_eq!(
            sample_size_sensitivity_grid(
                &[vec![1.0]],
                &[vec![0.0]],
                &[],
                DEFAULT_TOLERANCE,
                plan,
                &[],
            ),
            Err(ConceptGeometryError::EmptySampleSizeGrid)
        );
    }

    #[test]
    fn causal_and_interaction_helpers_keep_semantics_separate() {
        close(causal_novelty_gap(0.75, &[0.0, 0.25]).unwrap(), 0.625);
        close(interaction_residual(1.0, 1.0, 2.5).unwrap(), 0.5);
    }

    #[test]
    fn cosine_rejects_zero_norm() {
        assert_eq!(
            cosine_similarity(&[0.0, 0.0], &[1.0, 0.0], DEFAULT_TOLERANCE),
            Err(ConceptGeometryError::ZeroNorm)
        );
    }
}
