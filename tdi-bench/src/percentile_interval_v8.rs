//! Conservative deterministic percentile-interval candidate for bounded TDI-8.1.
//!
//! TDI-8.0 freezes paired generator-level uncertainty, exactly nine primary cells,
//! and Bonferroni family-wise allocation, but leaves the concrete interval method,
//! bootstrap replicate count, and resampling seed to bounded TDI-8.1 work.
//!
//! This module qualifies one deliberately simple candidate without selecting any
//! replicate count or seed: sort the already-generated paired bootstrap relative
//! effects, use non-interpolated order statistics, and retain at least the frozen
//! Bonferroni two-sided empirical mass. Any zero-baseline bootstrap replicate causes
//! interval construction to fail closed rather than silently dropping a replicate.
//!
//! This is software-preflight infrastructure, not a frozen experimental choice and
//! not H8-A/H8-B evidence.

use core::fmt;

use crate::decision_v8::RelativeEffectInterval;
use crate::paired_resampling_v8::{PairedResamplingError, PairedResamplingReplicates};

/// Exact rational denominator of the frozen two-sided Bonferroni tail probability.
///
/// TDI-8.0 freezes family alpha `0.05 = 1/20`, nine primary cells and two tails,
/// therefore each tail receives `1 / (20 * 9 * 2) = 1/360`.
pub const TDI8_PERCENTILE_TAIL_DENOMINATOR: usize = 360;

/// Fully auditable output of the bounded percentile candidate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PercentileIntervalCandidate {
    interval: RelativeEffectInterval,
    dropped_per_tail: usize,
    requested_replicates: usize,
    seed: u64,
}

impl PercentileIntervalCandidate {
    /// Finite non-interpolated relative-effect interval.
    #[must_use]
    pub const fn interval(self) -> RelativeEffectInterval {
        self.interval
    }

    /// Number of sorted bootstrap effects excluded below and above the interval.
    #[must_use]
    pub const fn dropped_per_tail(self) -> usize {
        self.dropped_per_tail
    }

    /// Exact caller-selected bootstrap replicate count carried into provenance.
    #[must_use]
    pub const fn requested_replicates(self) -> usize {
        self.requested_replicates
    }

    /// Exact caller-selected bootstrap seed carried into provenance.
    #[must_use]
    pub const fn seed(self) -> u64 {
        self.seed
    }
}

/// Fail-closed errors for the bounded interval candidate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PercentileIntervalError {
    /// The paired-resampling output fails its exact replicate-accounting invariant.
    Resampling(PairedResamplingError),
    /// The complete-sample point baseline is zero, so the frozen zero-baseline
    /// classifier branch applies directly and a finite relative interval is invalid.
    PointBaselineZero,
    /// At least one bootstrap resample has a zero baseline. Such replicates cannot
    /// be represented as finite relative effects and are never silently removed.
    ZeroBaselineReplicates {
        zero_zero: usize,
        zero_positive: usize,
    },
    /// No finite relative-effect replicate is available.
    EmptyDefinedReplicates,
    /// Defensive validation found a non-finite relative-effect replicate.
    NonFiniteEffect { index: usize },
    /// Host allocation for deterministic sorting failed.
    EffectAllocationFailed { replicates: usize },
    /// Order-statistic index arithmetic overflowed.
    IndexArithmeticOverflow,
    /// The internal order-statistic endpoints could not form a valid finite interval.
    InvalidConstructedInterval,
}

impl fmt::Display for PercentileIntervalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for PercentileIntervalError {}

impl From<PairedResamplingError> for PercentileIntervalError {
    fn from(error: PairedResamplingError) -> Self {
        Self::Resampling(error)
    }
}

/// Construct the bounded deterministic percentile candidate.
///
/// The function does not choose a replicate count or seed: both must already be
/// present in `replicates`, where they were supplied explicitly by the caller.
///
/// For `n` defined relative-effect replicates, `floor(n / 360)` observations are
/// excluded from each sorted tail and no interpolation is performed. Consequently
/// the retained empirical mass is never smaller than `1 - 2/360`, modulo the
/// discreteness of the finite bootstrap sample. For fewer than 360 replicates the
/// candidate conservatively returns the complete observed min/max range.
///
/// Any zero-baseline bootstrap replicate rejects interval construction. This avoids
/// selection bias from conditioning the interval on only resamples for which the
/// relative statistic happened to be defined.
pub fn conservative_percentile_interval_candidate(
    replicates: &PairedResamplingReplicates,
) -> Result<PercentileIntervalCandidate, PercentileIntervalError> {
    replicates.validate_complete_accounting()?;

    if replicates.point().baseline_mean() == 0.0 {
        return Err(PercentileIntervalError::PointBaselineZero);
    }

    let zero_zero = replicates.zero_zero_replicates();
    let zero_positive = replicates.zero_positive_replicates();
    if zero_zero != 0 || zero_positive != 0 {
        return Err(PercentileIntervalError::ZeroBaselineReplicates {
            zero_zero,
            zero_positive,
        });
    }

    let effects = replicates.relative_effects();
    if effects.is_empty() {
        return Err(PercentileIntervalError::EmptyDefinedReplicates);
    }
    if let Some(index) = effects.iter().position(|effect| !effect.is_finite()) {
        return Err(PercentileIntervalError::NonFiniteEffect { index });
    }

    let mut sorted = Vec::new();
    sorted
        .try_reserve_exact(effects.len())
        .map_err(|_| PercentileIntervalError::EffectAllocationFailed {
            replicates: effects.len(),
        })?;
    sorted.extend_from_slice(effects);

    let (interval, dropped_per_tail) = conservative_order_statistic_interval(&mut sorted)?;
    Ok(PercentileIntervalCandidate {
        interval,
        dropped_per_tail,
        requested_replicates: replicates.requested_replicates(),
        seed: replicates.seed(),
    })
}

fn conservative_order_statistic_interval(
    effects: &mut [f64],
) -> Result<(RelativeEffectInterval, usize), PercentileIntervalError> {
    if effects.is_empty() {
        return Err(PercentileIntervalError::EmptyDefinedReplicates);
    }
    if let Some(index) = effects.iter().position(|effect| !effect.is_finite()) {
        return Err(PercentileIntervalError::NonFiniteEffect { index });
    }

    effects.sort_by(f64::total_cmp);
    let dropped_per_tail = effects.len() / TDI8_PERCENTILE_TAIL_DENOMINATOR;
    let upper_index = effects
        .len()
        .checked_sub(1)
        .and_then(|last| last.checked_sub(dropped_per_tail))
        .ok_or(PercentileIntervalError::IndexArithmeticOverflow)?;
    let lower = effects[dropped_per_tail];
    let upper = effects[upper_index];
    let interval = RelativeEffectInterval::new(lower, upper)
        .map_err(|_| PercentileIntervalError::InvalidConstructedInterval)?;
    Ok((interval, dropped_per_tail))
}

#[cfg(test)]
mod tests {
    use super::{
        PercentileIntervalError, TDI8_PERCENTILE_TAIL_DENOMINATOR,
        conservative_order_statistic_interval, conservative_percentile_interval_candidate,
    };
    use crate::paired_resampling_v8::{
        PairedDeficitObservation, PairedResamplingPlan, bonferroni_tail_probability,
        paired_bootstrap_replicates,
    };

    fn pair(baseline: f64, candidate: f64) -> PairedDeficitObservation {
        PairedDeficitObservation::new(baseline, candidate).expect("valid synthetic pair")
    }

    #[test]
    fn rational_tail_matches_frozen_bonferroni_allocation() {
        assert_eq!(TDI8_PERCENTILE_TAIL_DENOMINATOR, 20 * 9 * 2);
        let rational_tail = 1.0 / TDI8_PERCENTILE_TAIL_DENOMINATOR as f64;
        assert_eq!(rational_tail, bonferroni_tail_probability());
    }

    #[test]
    fn order_statistics_are_deterministic_and_non_interpolated() {
        let mut effects: Vec<f64> = (0..720).rev().map(|value| value as f64).collect();
        let (interval, dropped) =
            conservative_order_statistic_interval(&mut effects).expect("interval");
        assert_eq!(dropped, 2);
        assert_eq!(interval.lower(), 2.0);
        assert_eq!(interval.upper(), 717.0);
    }

    #[test]
    fn fewer_than_one_tail_denominator_uses_full_observed_range() {
        let mut effects = [0.5, -0.25, 0.125, 0.75];
        let (interval, dropped) =
            conservative_order_statistic_interval(&mut effects).expect("interval");
        assert_eq!(dropped, 0);
        assert_eq!(interval.lower(), -0.25);
        assert_eq!(interval.upper(), 0.75);
    }

    #[test]
    fn constant_relative_effect_produces_exact_point_interval() {
        let observations = [pair(1.0, 2.0), pair(2.0, 4.0), pair(8.0, 16.0)];
        let plan = PairedResamplingPlan::new(720, 91).expect("plan");
        let replicates = paired_bootstrap_replicates(&observations, plan).expect("bootstrap");
        let candidate =
            conservative_percentile_interval_candidate(&replicates).expect("interval candidate");
        assert_eq!(candidate.interval().lower(), -1.0);
        assert_eq!(candidate.interval().upper(), -1.0);
        assert_eq!(candidate.dropped_per_tail(), 2);
        assert_eq!(candidate.requested_replicates(), 720);
        assert_eq!(candidate.seed(), 91);
    }

    #[test]
    fn zero_baseline_bootstrap_replicates_fail_closed() {
        let observations = [pair(0.0, 1.0), pair(1.0, 0.0)];
        let plan = PairedResamplingPlan::new(256, 17).expect("plan");
        let replicates = paired_bootstrap_replicates(&observations, plan).expect("bootstrap");
        let error = conservative_percentile_interval_candidate(&replicates)
            .expect_err("degenerate relative replicates must reject");
        assert!(matches!(
            error,
            PercentileIntervalError::ZeroBaselineReplicates { zero_positive, .. }
                if zero_positive > 0
        ));
    }

    #[test]
    fn complete_sample_zero_baseline_uses_frozen_non_relative_branch() {
        let observations = [pair(0.0, 0.0), pair(0.0, 0.0)];
        let plan = PairedResamplingPlan::new(32, 7).expect("plan");
        let replicates = paired_bootstrap_replicates(&observations, plan).expect("bootstrap");
        assert_eq!(
            conservative_percentile_interval_candidate(&replicates),
            Err(PercentileIntervalError::PointBaselineZero)
        );
    }

    #[test]
    fn helper_rejects_non_finite_effects_defensively() {
        let mut effects = [0.0, f64::NAN, 1.0];
        assert_eq!(
            conservative_order_statistic_interval(&mut effects),
            Err(PercentileIntervalError::NonFiniteEffect { index: 1 })
        );
    }
}
