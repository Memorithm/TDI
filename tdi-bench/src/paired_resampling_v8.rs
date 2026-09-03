//! Deterministic paired-resampling foundation for TDI-8.1.
//!
//! TDI-8.0 freezes paired generator-level contrasts and a Bonferroni allocation
//! of `alpha = 0.05 / 9`, but deliberately leaves the concrete interval
//! construction, replicate count and deterministic resampling seed to TDI-8.1.
//! This module therefore stops one layer before interval construction: it
//! validates paired non-negative deficits, computes the frozen point statistic,
//! produces deterministic paired bootstrap replicate effects, and reports every
//! zero-baseline replicate explicitly instead of silently dropping it.
//!
//! No default replicate count or resampling seed is provided here. Selecting and
//! freezing those values remains later non-final TDI-8.1 work.

use core::{fmt, mem};

use crate::decision_v8::TDI8_PRIMARY_CELL_COUNT;

/// Frozen family-wise alpha from TDI-8.0.
pub const TDI8_FAMILY_ALPHA: f64 = 0.05;

/// Per-cell Bonferroni alpha for each of the exactly nine primary cells.
#[must_use]
pub fn bonferroni_cell_alpha() -> f64 {
    TDI8_FAMILY_ALPHA / TDI8_PRIMARY_CELL_COUNT as f64
}

/// One tail probability for a two-sided Bonferroni primary-cell interval.
#[must_use]
pub fn bonferroni_tail_probability() -> f64 {
    bonferroni_cell_alpha() / 2.0
}

/// One generator-level paired deficit observation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PairedDeficitObservation {
    baseline: f64,
    candidate: f64,
}

impl PairedDeficitObservation {
    /// Validate one finite, non-negative baseline/candidate pair.
    pub fn new(baseline: f64, candidate: f64) -> Result<Self, PairedResamplingError> {
        validate_deficit(baseline, DeficitSide::Baseline)?;
        validate_deficit(candidate, DeficitSide::Candidate)?;
        Ok(Self {
            baseline,
            candidate,
        })
    }

    /// Baseline-arm deficit.
    #[must_use]
    pub const fn baseline(self) -> f64 {
        self.baseline
    }

    /// Candidate-arm deficit from the same generator-level observation.
    #[must_use]
    pub const fn candidate(self) -> f64 {
        self.candidate
    }
}

/// Side named by a deficit-validation error.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DeficitSide {
    /// Baseline arm of the frozen contrast.
    Baseline,
    /// Candidate arm of the frozen contrast.
    Candidate,
}

/// Caller-supplied deterministic paired-resampling plan.
///
/// TDI-8.1 must later freeze concrete values using non-final data. This type
/// deliberately contains no default replicate count or default seed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PairedResamplingPlan {
    replicates: usize,
    seed: u64,
}

impl PairedResamplingPlan {
    /// Construct a plan with at least two replicates and a caller-supplied seed.
    pub fn new(replicates: usize, seed: u64) -> Result<Self, PairedResamplingError> {
        if replicates < 2 {
            return Err(PairedResamplingError::InvalidReplicateCount { replicates });
        }
        validate_replicate_capacity(replicates)?;
        Ok(Self { replicates, seed })
    }

    /// Requested bootstrap replicate count.
    #[must_use]
    pub const fn replicates(self) -> usize {
        self.replicates
    }

    /// Deterministic caller-supplied resampling seed.
    #[must_use]
    pub const fn seed(self) -> u64 {
        self.seed
    }
}

/// Frozen relative mean-deficit point statistic with its zero-baseline branch.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PairedPointEstimate {
    baseline_mean: f64,
    candidate_mean: f64,
    relative_effect: Option<f64>,
    absolute_degradation: Option<f64>,
}

impl PairedPointEstimate {
    /// Mean baseline deficit `B`.
    #[must_use]
    pub const fn baseline_mean(self) -> f64 {
        self.baseline_mean
    }

    /// Mean candidate deficit `C`.
    #[must_use]
    pub const fn candidate_mean(self) -> f64 {
        self.candidate_mean
    }

    /// `R=(B-C)/B` when `B > 0`; undefined on the frozen zero-baseline branch.
    #[must_use]
    pub const fn relative_effect(self) -> Option<f64> {
        self.relative_effect
    }

    /// `C-B` only when the exact zero-baseline branch has `C > 0`.
    #[must_use]
    pub const fn absolute_degradation(self) -> Option<f64> {
        self.absolute_degradation
    }
}

/// Deterministic paired-bootstrap replicate output before interval construction.
#[derive(Clone, Debug, PartialEq)]
pub struct PairedResamplingReplicates {
    point: PairedPointEstimate,
    relative_effects: Vec<f64>,
    zero_zero_replicates: usize,
    zero_positive_replicates: usize,
    requested_replicates: usize,
    seed: u64,
}

impl PairedResamplingReplicates {
    /// Point estimate from the complete paired observation set.
    #[must_use]
    pub const fn point(&self) -> PairedPointEstimate {
        self.point
    }

    /// Relative-effect replicates whose resampled baseline mean was positive.
    ///
    /// These values are intentionally unsorted. A later reviewed interval
    /// implementation owns ordering/quantile semantics.
    #[must_use]
    pub fn relative_effects(&self) -> &[f64] {
        &self.relative_effects
    }

    /// Replicates whose resampled means satisfy exact `B == 0 && C == 0`.
    #[must_use]
    pub const fn zero_zero_replicates(&self) -> usize {
        self.zero_zero_replicates
    }

    /// Replicates whose resampled means satisfy exact `B == 0 && C > 0`.
    #[must_use]
    pub const fn zero_positive_replicates(&self) -> usize {
        self.zero_positive_replicates
    }

    /// Total number of requested replicates, including zero-baseline cases.
    #[must_use]
    pub const fn requested_replicates(&self) -> usize {
        self.requested_replicates
    }

    /// Deterministic caller-supplied resampling seed.
    #[must_use]
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    /// Number of positive-baseline relative-effect replicates.
    #[must_use]
    pub fn defined_replicates(&self) -> usize {
        self.relative_effects.len()
    }

    /// Verify that every requested replicate is accounted for exactly once.
    pub fn validate_complete_accounting(&self) -> Result<(), PairedResamplingError> {
        let accounted = self
            .defined_replicates()
            .checked_add(self.zero_zero_replicates)
            .and_then(|count| count.checked_add(self.zero_positive_replicates))
            .ok_or(PairedResamplingError::ReplicateAccountingOverflow)?;
        if accounted != self.requested_replicates {
            return Err(PairedResamplingError::ReplicateAccountingMismatch {
                requested: self.requested_replicates,
                accounted,
            });
        }
        Ok(())
    }
}

/// Fail-closed paired-resampling errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PairedResamplingError {
    /// At least one paired generator-level observation is required.
    EmptyObservations,
    /// One paired deficit is NaN or infinite.
    NonFiniteDeficit {
        /// Baseline or candidate side.
        side: DeficitSide,
    },
    /// One paired deficit violates the frozen non-negative deficit definition.
    NegativeDeficit {
        /// Baseline or candidate side.
        side: DeficitSide,
    },
    /// At least two resampling replicates are required by this software layer.
    InvalidReplicateCount {
        /// Requested count.
        replicates: usize,
    },
    /// Replicate storage would exceed the host vector-capacity bound.
    ReplicateCapacityTooLarge {
        /// Requested count.
        replicates: usize,
    },
    /// The host allocator rejected replicate storage.
    ReplicateAllocationFailed {
        /// Requested count.
        replicates: usize,
    },
    /// Observation count cannot be represented by the deterministic u64 sampler.
    ObservationCountTooLarge,
    /// Fixed-order accumulation produced a non-finite value.
    NonFiniteAccumulation,
    /// A positive-baseline relative effect became non-finite.
    NonFiniteRelativeEffect,
    /// Replicate-accounting arithmetic overflowed.
    ReplicateAccountingOverflow,
    /// Replicate output failed exact accounting.
    ReplicateAccountingMismatch {
        /// Requested count.
        requested: usize,
        /// Count reconstructed from all output categories.
        accounted: usize,
    },
}

impl fmt::Display for PairedResamplingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for PairedResamplingError {}

fn validate_deficit(value: f64, side: DeficitSide) -> Result<(), PairedResamplingError> {
    if !value.is_finite() {
        return Err(PairedResamplingError::NonFiniteDeficit { side });
    }
    if value < 0.0 {
        return Err(PairedResamplingError::NegativeDeficit { side });
    }
    Ok(())
}

fn validate_replicate_capacity(replicates: usize) -> Result<(), PairedResamplingError> {
    let bytes = replicates
        .checked_mul(mem::size_of::<f64>())
        .ok_or(PairedResamplingError::ReplicateCapacityTooLarge { replicates })?;
    if bytes > isize::MAX as usize {
        return Err(PairedResamplingError::ReplicateCapacityTooLarge { replicates });
    }
    Ok(())
}

fn allocate_effects(replicates: usize) -> Result<Vec<f64>, PairedResamplingError> {
    validate_replicate_capacity(replicates)?;
    let mut effects = Vec::new();
    effects
        .try_reserve_exact(replicates)
        .map_err(|_| PairedResamplingError::ReplicateAllocationFailed { replicates })?;
    Ok(effects)
}

fn checked_pair_sums(
    observations: &[PairedDeficitObservation],
) -> Result<(f64, f64), PairedResamplingError> {
    let mut baseline_sum = 0.0_f64;
    let mut candidate_sum = 0.0_f64;
    for observation in observations {
        baseline_sum += observation.baseline;
        candidate_sum += observation.candidate;
        if !baseline_sum.is_finite() || !candidate_sum.is_finite() {
            return Err(PairedResamplingError::NonFiniteAccumulation);
        }
    }
    Ok((baseline_sum, candidate_sum))
}

fn estimate_from_sums(
    baseline_sum: f64,
    candidate_sum: f64,
    count: usize,
) -> Result<PairedPointEstimate, PairedResamplingError> {
    if count == 0 {
        return Err(PairedResamplingError::EmptyObservations);
    }
    let denominator = count as f64;
    let baseline_mean = baseline_sum / denominator;
    let candidate_mean = candidate_sum / denominator;
    if !baseline_mean.is_finite() || !candidate_mean.is_finite() {
        return Err(PairedResamplingError::NonFiniteAccumulation);
    }

    if baseline_mean == 0.0 {
        return Ok(PairedPointEstimate {
            baseline_mean,
            candidate_mean,
            relative_effect: None,
            absolute_degradation: (candidate_mean > 0.0).then_some(candidate_mean),
        });
    }

    let relative_effect = (baseline_mean - candidate_mean) / baseline_mean;
    if !relative_effect.is_finite() {
        return Err(PairedResamplingError::NonFiniteRelativeEffect);
    }
    Ok(PairedPointEstimate {
        baseline_mean,
        candidate_mean,
        relative_effect: Some(relative_effect),
        absolute_degradation: None,
    })
}

/// Compute the frozen TDI-8 relative mean-deficit point statistic from paired
/// generator-level observations.
pub fn paired_point_estimate(
    observations: &[PairedDeficitObservation],
) -> Result<PairedPointEstimate, PairedResamplingError> {
    if observations.is_empty() {
        return Err(PairedResamplingError::EmptyObservations);
    }
    let (baseline_sum, candidate_sum) = checked_pair_sums(observations)?;
    estimate_from_sums(baseline_sum, candidate_sum, observations.len())
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

    fn bounded(&mut self, upper: usize) -> Result<usize, PairedResamplingError> {
        let upper = u64::try_from(upper)
            .map_err(|_| PairedResamplingError::ObservationCountTooLarge)?;
        if upper == 0 {
            return Err(PairedResamplingError::EmptyObservations);
        }

        // Rejection sampling removes the modulo bias that would otherwise be
        // introduced when `upper` does not divide the full u64 sample space.
        let threshold = upper.wrapping_neg() % upper;
        loop {
            let value = self.next_u64();
            if value >= threshold {
                return usize::try_from(value % upper)
                    .map_err(|_| PairedResamplingError::ObservationCountTooLarge);
            }
        }
    }
}

/// Generate deterministic paired-bootstrap relative-effect replicates.
///
/// Each draw selects one observation index and therefore resamples baseline and
/// candidate deficits together. Zero-baseline resamples are counted in explicit
/// categories and are never discarded or coerced to a finite relative effect.
/// This function does not sort replicates or construct an uncertainty interval.
pub fn paired_bootstrap_replicates(
    observations: &[PairedDeficitObservation],
    plan: PairedResamplingPlan,
) -> Result<PairedResamplingReplicates, PairedResamplingError> {
    if observations.is_empty() {
        return Err(PairedResamplingError::EmptyObservations);
    }
    let point = paired_point_estimate(observations)?;
    let mut relative_effects = allocate_effects(plan.replicates)?;
    let mut zero_zero_replicates = 0usize;
    let mut zero_positive_replicates = 0usize;
    let mut rng = SplitMix64::new(plan.seed);

    for _ in 0..plan.replicates {
        let mut baseline_sum = 0.0_f64;
        let mut candidate_sum = 0.0_f64;
        for _ in 0..observations.len() {
            let observation = observations[rng.bounded(observations.len())?];
            baseline_sum += observation.baseline;
            candidate_sum += observation.candidate;
            if !baseline_sum.is_finite() || !candidate_sum.is_finite() {
                return Err(PairedResamplingError::NonFiniteAccumulation);
            }
        }

        let replicate = estimate_from_sums(baseline_sum, candidate_sum, observations.len())?;
        match replicate.relative_effect() {
            Some(effect) => relative_effects.push(effect),
            None if replicate.candidate_mean() == 0.0 => {
                zero_zero_replicates = zero_zero_replicates
                    .checked_add(1)
                    .ok_or(PairedResamplingError::ReplicateAccountingOverflow)?;
            }
            None => {
                zero_positive_replicates = zero_positive_replicates
                    .checked_add(1)
                    .ok_or(PairedResamplingError::ReplicateAccountingOverflow)?;
            }
        }
    }

    let output = PairedResamplingReplicates {
        point,
        relative_effects,
        zero_zero_replicates,
        zero_positive_replicates,
        requested_replicates: plan.replicates,
        seed: plan.seed,
    };
    output.validate_complete_accounting()?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::{
        DeficitSide, PairedDeficitObservation, PairedResamplingError, PairedResamplingPlan,
        TDI8_FAMILY_ALPHA, bonferroni_cell_alpha, bonferroni_tail_probability,
        paired_bootstrap_replicates, paired_point_estimate,
    };

    fn pair(baseline: f64, candidate: f64) -> PairedDeficitObservation {
        PairedDeficitObservation::new(baseline, candidate).expect("valid synthetic pair")
    }

    #[test]
    fn frozen_bonferroni_allocation_is_exposed_without_interval_method_freeze() {
        assert_eq!(TDI8_FAMILY_ALPHA, 0.05);
        assert_eq!(bonferroni_cell_alpha(), 0.05 / 9.0);
        assert_eq!(bonferroni_tail_probability(), 0.05 / 18.0);
    }

    #[test]
    fn paired_inputs_and_plan_fail_closed() {
        assert_eq!(
            PairedDeficitObservation::new(f64::NAN, 0.0),
            Err(PairedResamplingError::NonFiniteDeficit {
                side: DeficitSide::Baseline,
            })
        );
        assert_eq!(
            PairedDeficitObservation::new(0.0, -1.0),
            Err(PairedResamplingError::NegativeDeficit {
                side: DeficitSide::Candidate,
            })
        );
        assert_eq!(
            PairedResamplingPlan::new(1, 7),
            Err(PairedResamplingError::InvalidReplicateCount { replicates: 1 })
        );
        assert_eq!(
            paired_point_estimate(&[]),
            Err(PairedResamplingError::EmptyObservations)
        );
    }

    #[test]
    fn point_estimate_matches_frozen_relative_and_zero_baseline_branches() {
        let ordinary = paired_point_estimate(&[pair(1.0, 0.5), pair(3.0, 1.5)])
            .expect("ordinary point estimate");
        assert_eq!(ordinary.baseline_mean(), 2.0);
        assert_eq!(ordinary.candidate_mean(), 1.0);
        assert_eq!(ordinary.relative_effect(), Some(0.5));
        assert_eq!(ordinary.absolute_degradation(), None);

        let zero_zero = paired_point_estimate(&[pair(0.0, 0.0), pair(0.0, 0.0)])
            .expect("zero/zero point estimate");
        assert_eq!(zero_zero.relative_effect(), None);
        assert_eq!(zero_zero.absolute_degradation(), None);

        let zero_positive = paired_point_estimate(&[pair(0.0, 1.0), pair(0.0, 3.0)])
            .expect("zero/positive point estimate");
        assert_eq!(zero_positive.relative_effect(), None);
        assert_eq!(zero_positive.absolute_degradation(), Some(2.0));
    }

    #[test]
    fn bootstrap_is_deterministic_for_identical_plan() {
        let observations = [pair(1.0, 0.5), pair(2.0, 1.5), pair(4.0, 2.0)];
        let plan = PairedResamplingPlan::new(64, 0x1234_5678_9abc_def0).expect("plan");
        let left = paired_bootstrap_replicates(&observations, plan).expect("left bootstrap");
        let right = paired_bootstrap_replicates(&observations, plan).expect("right bootstrap");
        assert_eq!(left, right);
        assert_eq!(left.requested_replicates(), 64);
        assert_eq!(left.seed(), 0x1234_5678_9abc_def0);
        left.validate_complete_accounting().expect("complete accounting");
    }

    #[test]
    fn bootstrap_resamples_baseline_and_candidate_as_indivisible_pairs() {
        let observations = [pair(1.0, 2.0), pair(2.0, 4.0), pair(8.0, 16.0)];
        let plan = PairedResamplingPlan::new(128, 91).expect("plan");
        let output = paired_bootstrap_replicates(&observations, plan).expect("paired bootstrap");
        assert_eq!(output.defined_replicates(), 128);
        assert_eq!(output.zero_zero_replicates(), 0);
        assert_eq!(output.zero_positive_replicates(), 0);
        assert!(
            output
                .relative_effects()
                .iter()
                .all(|effect| effect.to_bits() == (-1.0_f64).to_bits())
        );
    }

    #[test]
    fn zero_baseline_replicates_are_counted_not_silently_dropped() {
        let observations = [pair(0.0, 1.0), pair(1.0, 0.0)];
        let plan = PairedResamplingPlan::new(256, 17).expect("plan");
        let output = paired_bootstrap_replicates(&observations, plan).expect("bootstrap");
        assert!(output.zero_positive_replicates() > 0);
        assert_eq!(output.zero_zero_replicates(), 0);
        output.validate_complete_accounting().expect("complete accounting");
        assert_eq!(
            output.defined_replicates() + output.zero_positive_replicates(),
            output.requested_replicates()
        );
    }
}
