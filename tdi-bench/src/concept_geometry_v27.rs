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
pub struct InnovationBootstrapReplicates {
    point: ResidualDirection,
    innovation_energy_ratios: Vec<f64>,
    undefined_contrast_replicates: usize,
    requested_replicates: usize,
    seed: u64,
}

impl InnovationBootstrapReplicates {
    #[must_use]
    pub fn point(&self) -> &ResidualDirection {
        &self.point
    }

    #[must_use]
    pub fn innovation_energy_ratios(&self) -> &[f64] {
        &self.innovation_energy_ratios
    }

    #[must_use]
    pub const fn undefined_contrast_replicates(&self) -> usize {
        self.undefined_contrast_replicates
    }

    #[must_use]
    pub fn defined_replicates(&self) -> usize {
        self.innovation_energy_ratios.len()
    }

    #[must_use]
    pub const fn requested_replicates(&self) -> usize {
        self.requested_replicates
    }

    #[must_use]
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    pub fn validate_complete_accounting(&self) -> Result<(), ConceptGeometryError> {
        let accounted = self
            .defined_replicates()
            .checked_add(self.undefined_contrast_replicates)
            .ok_or(ConceptGeometryError::ReplicateAccountingOverflow)?;
        if accounted != self.requested_replicates {
            return Err(ConceptGeometryError::ReplicateAccountingMismatch);
        }
        Ok(())
    }
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
    ReplicateAccountingMismatch,
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

fn mean_from_indices(
    rows: &[Vec<f64>],
    indices: &[usize],
) -> Result<Vec<f64>, ConceptGeometryError> {
    let width = validate_rows(rows)?;
    if indices.is_empty() {
        return Err(ConceptGeometryError::EmptyGroup);
    }

    let mut result = vec![0.0; width];
    for &index in indices {
        let row = rows
            .get(index)
            .ok_or(ConceptGeometryError::SampleCountTooLarge)?;
        for (target, value) in result.iter_mut().zip(row) {
            *target += value;
        }
    }
    let scale = 1.0 / indices.len() as f64;
    for value in &mut result {
        *value *= scale;
    }
    Ok(result)
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

/// Bootstrap the TDI-27 innovation-energy ratio using separate P/C resampling.
///
/// A replicate whose resampled mean contrast has norm at or below `tolerance`
/// is reported explicitly as undefined because the innovation-energy ratio has
/// a zero denominator. Such replicates are never silently dropped from
/// accounting or coerced to a finite value.
pub fn bootstrap_innovation_energy_replicates(
    positive: &[Vec<f64>],
    control: &[Vec<f64>],
    basis_directions: &[Vec<f64>],
    plan: DevelopmentResamplingPlan,
    tolerance: f64,
) -> Result<InnovationBootstrapReplicates, ConceptGeometryError> {
    valid_tolerance(tolerance)?;
    let positive_width = validate_rows(positive)?;
    let control_width = validate_rows(control)?;
    if positive_width != control_width {
        return Err(ConceptGeometryError::DimensionMismatch);
    }

    let point_contrast = mean_difference(positive, control)?;
    let point = residualize(&point_contrast, basis_directions, tolerance)?;
    let index_replicates =
        independent_group_bootstrap_indices(positive.len(), control.len(), plan)?;

    let mut innovation_energy_ratios = Vec::new();
    innovation_energy_ratios
        .try_reserve_exact(plan.replicates())
        .map_err(|_| ConceptGeometryError::ReplicateAccountingOverflow)?;
    let mut undefined_contrast_replicates = 0usize;

    for indices in &index_replicates {
        let positive_mean = mean_from_indices(positive, indices.positive())?;
        let control_mean = mean_from_indices(control, indices.control())?;
        let contrast = positive_mean
            .iter()
            .zip(control_mean)
            .map(|(left, right)| left - right)
            .collect::<Vec<_>>();
        let contrast_norm = norm(&contrast);
        if !contrast_norm.is_finite() {
            return Err(ConceptGeometryError::NonFiniteValue);
        }
        if contrast_norm <= tolerance {
            undefined_contrast_replicates = undefined_contrast_replicates
                .checked_add(1)
                .ok_or(ConceptGeometryError::ReplicateAccountingOverflow)?;
            continue;
        }

        let report = residualize(&contrast, basis_directions, tolerance)?;
        innovation_energy_ratios.push(report.innovation_energy_ratio());
    }

    let output = InnovationBootstrapReplicates {
        point,
        innovation_energy_ratios,
        undefined_contrast_replicates,
        requested_replicates: plan.replicates(),
        seed: plan.seed(),
    };
    output.validate_complete_accounting()?;
    Ok(output)
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
    fn innovation_bootstrap_recovers_constant_shift_ratio() {
        let control = vec![vec![0.0, 0.0, 0.0]; 6];
        let positive = vec![vec![3.0, 4.0, 0.0]; 6];
        let basis = vec![vec![1.0, 0.0, 0.0]];
        let plan = DevelopmentResamplingPlan::new(12, 0x27_03).unwrap();
        let output = bootstrap_innovation_energy_replicates(
            &positive,
            &control,
            &basis,
            plan,
            DEFAULT_TOLERANCE,
        )
        .unwrap();

        close(output.point().innovation_energy_ratio(), 16.0 / 25.0);
        assert_eq!(output.requested_replicates(), 12);
        assert_eq!(output.defined_replicates(), 12);
        assert_eq!(output.undefined_contrast_replicates(), 0);
        assert_eq!(output.seed(), 0x27_03);
        for value in output.innovation_energy_ratios() {
            close(*value, 16.0 / 25.0);
        }
        output.validate_complete_accounting().unwrap();
    }

    #[test]
    fn innovation_bootstrap_accounts_for_undefined_zero_contrasts() {
        let positive = vec![vec![1.0, 0.0], vec![0.0, 0.0]];
        let control = vec![vec![0.0, 0.0], vec![0.0, 0.0]];
        let plan = DevelopmentResamplingPlan::new(8, 0x27_03).unwrap();
        let output = bootstrap_innovation_energy_replicates(
            &positive,
            &control,
            &[],
            plan,
            DEFAULT_TOLERANCE,
        )
        .unwrap();

        assert!(output.defined_replicates() > 0);
        assert!(output.undefined_contrast_replicates() > 0);
        assert_eq!(
            output.defined_replicates() + output.undefined_contrast_replicates(),
            output.requested_replicates()
        );
        assert!(
            output
                .innovation_energy_ratios()
                .iter()
                .all(|value| (*value - 1.0).abs() < 1.0e-12)
        );
        output.validate_complete_accounting().unwrap();
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
