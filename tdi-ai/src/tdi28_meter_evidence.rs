//! TDI-28 Stage-0 measured-energy evidence intake and paired qualification.
//!
//! This module DOES NOT read an energy sensor. It validates independently
//! supplied cumulative microjoule counter windows with explicit provenance,
//! bracketed idle controls and matched C0/C3 observations. The provenance is
//! an unverified caller declaration, not cryptographic attestation. Only
//! RemoteOps-authorized runs may eventually supply physical observations.
//! All tests use fabricated counter readings and establish software behavior
//! only; no empirical energy saving or calibrated coefficient is claimed.

use std::collections::BTreeSet;

use super::tdi28_energy_algebra::Demand;
use super::tdi28_four_arm::FourArmReport;

/// Limits ingestion before any result aggregation.
pub const MAX_ENERGY_PAIRS: usize = 128;
pub const MIN_BALANCED_PAIRS: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvidenceSplit {
    Development,
    Validation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterSource {
    CpuEnergyCounter,
    ExternalPowerMeter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeterDomain {
    CpuPackage,
    WholeSystem,
}

/// A reported measurement context; digest fields bind the declared
/// workload/binary/configuration and isolation manifest externally.
/// This struct does NOT verify the authenticity of those digests.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvidenceContext {
    pub protocol_revision: u64,
    pub source: MeterSource,
    pub domain: MeterDomain,
    pub sensor_id: String,
    pub sensor_resolution_microjoules: u64,
    pub hardware_sha256: [u8; 32],
    pub binary_sha256: [u8; 32],
    pub workload_sha256: [u8; 32],
    pub isolation_manifest_sha256: [u8; 32],
}

/// A cumulative, non-wrapping physical meter sample in microjoules.
/// Counter resets/wraparound are rejected; no modulus is assumed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MeterReading {
    pub monotonic_ns: u64,
    pub cumulative_microjoules: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MeterWindow {
    pub begin: MeterReading,
    pub end: MeterReading,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObservedQuality {
    pub required_expected: u64,
    pub required_correct: u64,
    pub optional_expected: u64,
    pub optional_delivered: u64,
    pub deadline_misses: u64,
    pub outstanding: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnergyTrial {
    pub run_id: u64,
    pub context: EvidenceContext,
    pub quality: ObservedQuality,
    /// An idle observation ending before the active experiment begins.
    pub idle_before: MeterWindow,
    pub active: MeterWindow,
    /// An idle observation beginning after active work completes.
    pub idle_after: MeterWindow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunOrder {
    BaselineFirst,
    CandidateFirst,
}

/// C0 vs C3, matched on pair_id, reference trace and environment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnergyPair {
    pub pair_id: u64,
    pub split: EvidenceSplit,
    pub order: RunOrder,
    pub baseline_c0: EnergyTrial,
    pub candidate_c3: EnergyTrial,
}

/// A *descriptive*, idle-correction sensitivity interval. Bounds include
/// the two independently observed idle rates, not a statistical CI.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MicrojouleInterval {
    pub lower: i128,
    pub upper: i128,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvidenceError {
    MissingStrictReferenceParity,
    PairCountOutsideBounds,
    InvalidContext,
    InconsistentContext,
    MixedSplits,
    DuplicatePairId,
    DuplicateRunId,
    UnbalancedOrder,
    ZeroOrReversedInterval,
    MeterCounterResetOrWrap,
    NonBracketingIdleWindows,
    InvalidRunChronology,
    IncorrectObservedQuality,
    ArithmeticOverflow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PairedDifference {
    pub pair_id: u64,
    pub order: RunOrder,
    pub baseline_net_microjoules: MicrojouleInterval,
    pub candidate_net_microjoules: MicrojouleInterval,
    /// Positive: baseline consumes more net energy than candidate.
    /// An interval straddling zero remains inconclusive.
    pub baseline_minus_candidate: MicrojouleInterval,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvidenceReport {
    pub split: EvidenceSplit,
    pub context: EvidenceContext,
    pub pairs: Vec<PairedDifference>,
    /// Descriptive sorted central bounds, not a confidence interval.
    pub central_pair_bounds: MicrojouleInterval,
    pub strictly_positive_pairs: usize,
    pub strictly_negative_pairs: usize,
    pub sign_ambiguous_pairs: usize,
}

fn check_context(ctx: &EvidenceContext) -> Result<(), EvidenceError> {
    if ctx.protocol_revision == 0
        || ctx.sensor_resolution_microjoules == 0
        || ctx.sensor_id.trim().is_empty()
        || [
            ctx.hardware_sha256,
            ctx.binary_sha256,
            ctx.workload_sha256,
            ctx.isolation_manifest_sha256,
        ]
        .iter()
        .any(|digest| digest.iter().all(|byte| *byte == 0))
    {
        return Err(EvidenceError::InvalidContext);
    }
    Ok(())
}

fn delta(window: MeterWindow) -> Result<(u64, u64), EvidenceError> {
    let time = window
        .end
        .monotonic_ns
        .checked_sub(window.begin.monotonic_ns)
        .filter(|elapsed| *elapsed > 0)
        .ok_or(EvidenceError::ZeroOrReversedInterval)?;
    let energy = window
        .end
        .cumulative_microjoules
        .checked_sub(window.begin.cumulative_microjoules)
        .ok_or(EvidenceError::MeterCounterResetOrWrap)?;
    Ok((time, energy))
}

/// Compute a bracket-sensitive idle-corrected energy interval. Negative
/// endpoints are retained: an over-correction is not silently clamped to 0.
/// Assumes a locally constant idle-power approximation at each bracket,
/// which must be assessed independently on real hardware. Integer
/// truncation of the per-bracket idle prediction is <= 1 microjoule.
pub fn idle_corrected_energy(trial: &EnergyTrial) -> Result<MicrojouleInterval, EvidenceError> {
    let (before_time, before_energy) = delta(trial.idle_before)?;
    let (active_time, active_energy) = delta(trial.active)?;
    let (after_time, after_energy) = delta(trial.idle_after)?;

    let times = (
        trial.idle_before.end.monotonic_ns,
        trial.active.begin.monotonic_ns,
        trial.active.end.monotonic_ns,
        trial.idle_after.begin.monotonic_ns,
    );
    if times.0 > times.1 || times.2 > times.3 {
        return Err(EvidenceError::NonBracketingIdleWindows);
    }
    // All three windows must belong to one uninterrupted cumulative
    // counter epoch. A counter reset between windows is not corrected.
    if trial.idle_before.end.cumulative_microjoules > trial.active.begin.cumulative_microjoules
        || trial.active.end.cumulative_microjoules > trial.idle_after.begin.cumulative_microjoules
    {
        return Err(EvidenceError::MeterCounterResetOrWrap);
    }

    let expected_before =
        (u128::from(before_energy) * u128::from(active_time)) / u128::from(before_time);
    let expected_after =
        (u128::from(after_energy) * u128::from(active_time)) / u128::from(after_time);
    let active = i128::from(active_energy);
    let lowest_idle = i128::try_from(expected_before.min(expected_after))
        .map_err(|_| EvidenceError::ArithmeticOverflow)?;
    let highest_idle = i128::try_from(expected_before.max(expected_after))
        .map_err(|_| EvidenceError::ArithmeticOverflow)?;
    Ok(MicrojouleInterval {
        lower: active
            .checked_sub(highest_idle)
            .ok_or(EvidenceError::ArithmeticOverflow)?,
        upper: active
            .checked_sub(lowest_idle)
            .ok_or(EvidenceError::ArithmeticOverflow)?,
    })
}

fn reference_quality(report: &FourArmReport) -> Result<ObservedQuality, EvidenceError> {
    let required = report
        .events
        .iter()
        .filter(|event| event.demand == Demand::Required)
        .count();
    let optional = report.events.len() - required;
    let required = u64::try_from(required).map_err(|_| EvidenceError::ArithmeticOverflow)?;
    let optional = u64::try_from(optional).map_err(|_| EvidenceError::ArithmeticOverflow)?;
    Ok(ObservedQuality {
        required_expected: required,
        required_correct: required,
        optional_expected: optional,
        optional_delivered: optional,
        deadline_misses: 0,
        outstanding: 0,
    })
}

fn validate_trial(
    trial: &EnergyTrial,
    expected_quality: ObservedQuality,
) -> Result<MicrojouleInterval, EvidenceError> {
    check_context(&trial.context)?;
    if trial.run_id == 0 || trial.quality != expected_quality {
        return Err(EvidenceError::IncorrectObservedQuality);
    }
    idle_corrected_energy(trial)
}

fn chronological_trials(pair: &EnergyPair) -> (&EnergyTrial, &EnergyTrial) {
    match pair.order {
        RunOrder::BaselineFirst => (&pair.baseline_c0, &pair.candidate_c3),
        RunOrder::CandidateFirst => (&pair.candidate_c3, &pair.baseline_c0),
    }
}

fn check_pair_order(pair: &EnergyPair) -> Result<(), EvidenceError> {
    let (first, second) = chronological_trials(pair);
    // Same declared meter/source: an ordered pair must use disjoint
    // monotonic time windows and a non-reset cumulative counter epoch.
    if first.idle_after.end.monotonic_ns > second.idle_before.begin.monotonic_ns
        || first.idle_after.end.cumulative_microjoules
            > second.idle_before.begin.cumulative_microjoules
    {
        return Err(EvidenceError::InvalidRunChronology);
    }
    Ok(())
}

fn check_consecutive_pairs(previous: &EnergyPair, next: &EnergyPair) -> Result<(), EvidenceError> {
    let (_, last_trial) = chronological_trials(previous);
    let (first_trial, _) = chronological_trials(next);
    // Input pairs are in observation chronology. Pair IDs are identifiers,
    // NOT permission to reuse an earlier meter epoch or time interval.
    // This protects the same shared meter between separate C0/C3 pairs;
    // equal boundaries are permitted when windows just touch.
    if last_trial.idle_after.end.monotonic_ns > first_trial.idle_before.begin.monotonic_ns
        || last_trial.idle_after.end.cumulative_microjoules
            > first_trial.idle_before.begin.cumulative_microjoules
    {
        return Err(EvidenceError::InvalidRunChronology);
    }
    Ok(())
}

/// Analyze matched C0/C3 trial pairs from **one** declared Development or
/// Validation split; no split pooling, coefficient fitting, or validation
/// feedback is permitted by this API. Requires an independent Stage-0
/// strict-output reference and trial-level observed quality assertions.
///
/// At least four pairs, alternating run order with balance <= 1, identical
/// sensor/hardware/configuration/workload context and unique run IDs are
/// mandatory. Input pairs must follow uninterrupted chronological meter
/// order, including at the boundaries between pairs. Pair IDs need not be
/// monotonic; they are only uniqueness identifiers.
/// Observations and context are supplied by the caller; neither
/// runtime nor sensor authenticity is verified. This report is descriptive
/// and does not establish a causal energy benefit or calibrated costs.
pub fn qualify_paired_observations(
    reference: &FourArmReport,
    pairs: &[EnergyPair],
) -> Result<EvidenceReport, EvidenceError> {
    if !reference.strict_quality_parity() {
        return Err(EvidenceError::MissingStrictReferenceParity);
    }
    if pairs.len() < MIN_BALANCED_PAIRS || pairs.len() > MAX_ENERGY_PAIRS {
        return Err(EvidenceError::PairCountOutsideBounds);
    }

    let quality = reference_quality(reference)?;
    let first = &pairs[0];
    check_context(&first.baseline_c0.context)?;
    let context = &first.baseline_c0.context;
    let split = first.split;
    let mut seen_pairs = BTreeSet::new();
    let mut seen_runs = BTreeSet::new();
    let mut baseline_first = 0usize;
    let mut candidate_first = 0usize;
    let mut comparisons = Vec::with_capacity(pairs.len());
    let mut strictly_positive_pairs = 0;
    let mut strictly_negative_pairs = 0;
    let mut sign_ambiguous_pairs = 0;
    let mut previous_pair: Option<&EnergyPair> = None;

    for pair in pairs {
        if pair.split != split {
            return Err(EvidenceError::MixedSplits);
        }
        if pair.baseline_c0.context != *context || pair.candidate_c3.context != *context {
            return Err(EvidenceError::InconsistentContext);
        }
        if pair.pair_id == 0 || !seen_pairs.insert(pair.pair_id) {
            return Err(EvidenceError::DuplicatePairId);
        }
        if !seen_runs.insert(pair.baseline_c0.run_id) || !seen_runs.insert(pair.candidate_c3.run_id)
        {
            return Err(EvidenceError::DuplicateRunId);
        }
        match pair.order {
            RunOrder::BaselineFirst => baseline_first += 1,
            RunOrder::CandidateFirst => candidate_first += 1,
        }

        let baseline = validate_trial(&pair.baseline_c0, quality)?;
        let candidate = validate_trial(&pair.candidate_c3, quality)?;
        check_pair_order(pair)?;
        if let Some(previous) = previous_pair {
            check_consecutive_pairs(previous, pair)?;
        }
        previous_pair = Some(pair);
        let delta = MicrojouleInterval {
            lower: baseline
                .lower
                .checked_sub(candidate.upper)
                .ok_or(EvidenceError::ArithmeticOverflow)?,
            upper: baseline
                .upper
                .checked_sub(candidate.lower)
                .ok_or(EvidenceError::ArithmeticOverflow)?,
        };
        if delta.lower > 0 {
            strictly_positive_pairs += 1;
        } else if delta.upper < 0 {
            strictly_negative_pairs += 1;
        } else {
            sign_ambiguous_pairs += 1;
        }
        comparisons.push(PairedDifference {
            pair_id: pair.pair_id,
            order: pair.order,
            baseline_net_microjoules: baseline,
            candidate_net_microjoules: candidate,
            baseline_minus_candidate: delta,
        });
    }
    if baseline_first < 2 || candidate_first < 2 || baseline_first.abs_diff(candidate_first) > 1 {
        return Err(EvidenceError::UnbalancedOrder);
    }

    let mut lower: Vec<_> = comparisons
        .iter()
        .map(|pair| pair.baseline_minus_candidate.lower)
        .collect();
    let mut upper: Vec<_> = comparisons
        .iter()
        .map(|pair| pair.baseline_minus_candidate.upper)
        .collect();
    lower.sort_unstable();
    upper.sort_unstable();
    // For even N use the two central ranks, preserving the uncertainty
    // interval rather than fabricating a rounded point median.
    let central = MicrojouleInterval {
        lower: lower[(pairs.len() - 1) / 2],
        upper: upper[pairs.len() / 2],
    };
    Ok(EvidenceReport {
        split,
        context: context.clone(),
        pairs: comparisons,
        central_pair_bounds: central,
        strictly_positive_pairs,
        strictly_negative_pairs,
        sign_ambiguous_pairs,
    })
}

#[cfg(test)]
mod tests {
    use super::super::tdi28_energy_algebra::{Gate, InputRevisions, Work};
    use super::super::tdi28_four_arm::{MatchedCostModel, MatchedEvent, replay_four};
    use super::super::tdi28_trace::TraceEvent;
    use super::*;

    fn reference() -> FourArmReport {
        let events = [
            MatchedEvent {
                event: TraceEvent {
                    tick: 0,
                    gate: Gate::Xor,
                    revisions: InputRevisions { left: 1, right: 1 },
                    left: false,
                    right: true,
                    demand: Demand::Required,
                },
                optional_deadline: None,
                event_ceiling: 10_000,
            },
            MatchedEvent {
                event: TraceEvent {
                    tick: 2,
                    gate: Gate::Xor,
                    revisions: InputRevisions { left: 1, right: 1 },
                    left: false,
                    right: true,
                    demand: Demand::Required,
                },
                optional_deadline: None,
                event_ceiling: 10_000,
            },
        ];
        replay_four(
            &events,
            3,
            MatchedCostModel {
                model_revision: 1,
                weights: Work {
                    gate_evaluations: 10,
                    input_value_reads: 1,
                    ..Work::default()
                },
            },
        )
        .unwrap()
    }

    fn context() -> EvidenceContext {
        EvidenceContext {
            protocol_revision: 1,
            source: MeterSource::ExternalPowerMeter,
            domain: MeterDomain::WholeSystem,
            sensor_id: "synthetic-fixture-only".to_owned(),
            sensor_resolution_microjoules: 1,
            hardware_sha256: [1; 32],
            binary_sha256: [2; 32],
            workload_sha256: [3; 32],
            isolation_manifest_sha256: [4; 32],
        }
    }

    fn window(start: u64, end: u64, base: u64, delta: u64) -> MeterWindow {
        MeterWindow {
            begin: MeterReading {
                monotonic_ns: start,
                cumulative_microjoules: base,
            },
            end: MeterReading {
                monotonic_ns: end,
                cumulative_microjoules: base + delta,
            },
        }
    }

    fn trial(run_id: u64, active_delta: u64, timebase: u64, energybase: u64) -> EnergyTrial {
        EnergyTrial {
            run_id,
            context: context(),
            quality: ObservedQuality {
                required_expected: 2,
                required_correct: 2,
                optional_expected: 0,
                optional_delivered: 0,
                deadline_misses: 0,
                outstanding: 0,
            },
            idle_before: window(timebase, timebase + 10, energybase, 10),
            active: window(timebase + 10, timebase + 30, energybase + 10, active_delta),
            idle_after: window(
                timebase + 30,
                timebase + 40,
                energybase + 10 + active_delta,
                10,
            ),
        }
    }

    fn pairs() -> Vec<EnergyPair> {
        (0..4)
            .map(|idx| {
                let order = if idx % 2 == 0 {
                    RunOrder::BaselineFirst
                } else {
                    RunOrder::CandidateFirst
                };
                let timebase = 200 * idx;
                let energybase = 10_000 * idx;
                let baseline_first = order == RunOrder::BaselineFirst;
                EnergyPair {
                    pair_id: idx + 1,
                    split: EvidenceSplit::Development,
                    order,
                    baseline_c0: trial(
                        10 + 2 * idx,
                        120,
                        timebase + if baseline_first { 0 } else { 60 },
                        energybase + if baseline_first { 0 } else { 1_000 },
                    ),
                    candidate_c3: trial(
                        11 + 2 * idx,
                        90,
                        timebase + if baseline_first { 60 } else { 0 },
                        energybase + if baseline_first { 1_000 } else { 0 },
                    ),
                }
            })
            .collect()
    }

    #[test]
    fn balanced_exact_quality_pairs_report_only_descriptive_deltas() {
        let result = qualify_paired_observations(&reference(), &pairs()).unwrap();
        assert_eq!(result.pairs.len(), 4);
        assert_eq!(result.strictly_positive_pairs, 4);
        assert_eq!(result.strictly_negative_pairs, 0);
        assert_eq!(result.sign_ambiguous_pairs, 0);
        assert_eq!(
            result.central_pair_bounds,
            MicrojouleInterval {
                lower: 30,
                upper: 30
            }
        );
    }

    #[test]
    fn harmful_and_null_energy_outcomes_are_preserved() {
        let mut observations = pairs();
        // C3 requires *more* submitted energy in the first matched pair.
        observations[0].candidate_c3 = trial(11, 170, 60, 1_000);
        // One matched pair is exactly tied after idle correction.
        observations[1].candidate_c3 = trial(13, 120, 200, 10_000);
        let report = qualify_paired_observations(&reference(), &observations).unwrap();
        assert_eq!(report.strictly_positive_pairs, 2);
        assert_eq!(report.strictly_negative_pairs, 1);
        assert_eq!(report.sign_ambiguous_pairs, 1);
        assert!(report.pairs[0].baseline_minus_candidate.upper < 0);
        assert_eq!(
            report.pairs[1].baseline_minus_candidate,
            MicrojouleInterval { lower: 0, upper: 0 }
        );
    }

    #[test]
    fn bracketed_idle_drift_remains_an_uncertainty_interval() {
        let mut candidate = trial(8, 70, 0, 0);
        candidate.idle_after = window(30, 40, 80, 30);
        // Non-wrapping cumulative reading can be later/higher across
        // the hand-off; this test changes the idle rate deliberately.
        let interval = idle_corrected_energy(&candidate).unwrap();
        assert_eq!(
            interval,
            MicrojouleInterval {
                lower: 10,
                upper: 50
            }
        );
    }

    #[test]
    fn negative_idle_corrected_values_are_not_clamped() {
        let mut sample = trial(9, 10, 0, 0);
        sample.idle_after = window(30, 40, 20, 20);
        let interval = idle_corrected_energy(&sample).unwrap();
        assert_eq!(
            interval,
            MicrojouleInterval {
                lower: -30,
                upper: -10
            }
        );
    }

    #[test]
    fn wrap_or_counter_reset_is_rejected() {
        let mut sample = trial(9, 100, 0, 0);
        sample.active.end.cumulative_microjoules = 9;
        assert_eq!(
            idle_corrected_energy(&sample),
            Err(EvidenceError::MeterCounterResetOrWrap)
        );
        let mut between_windows = trial(9, 100, 0, 0);
        between_windows.idle_after.begin.cumulative_microjoules = 100;
        assert_eq!(
            idle_corrected_energy(&between_windows),
            Err(EvidenceError::MeterCounterResetOrWrap)
        );
    }

    #[test]
    fn invalid_quality_and_unmatched_instrument_provenance_fail_closed() {
        let mut observations = pairs();
        observations[0].candidate_c3.quality.required_correct = 1;
        assert_eq!(
            qualify_paired_observations(&reference(), &observations),
            Err(EvidenceError::IncorrectObservedQuality)
        );
        let mut observations = pairs();
        observations[0].candidate_c3.context.sensor_id = "a-different-meter".to_owned();
        assert_eq!(
            qualify_paired_observations(&reference(), &observations),
            Err(EvidenceError::InconsistentContext)
        );
    }

    #[test]
    fn split_leakage_duplicates_and_unbalanced_order_fail_closed() {
        let mut observations = pairs();
        observations[0].split = EvidenceSplit::Validation;
        assert_eq!(
            qualify_paired_observations(&reference(), &observations),
            Err(EvidenceError::MixedSplits)
        );
        let mut observations = pairs();
        observations[1].pair_id = observations[0].pair_id;
        assert_eq!(
            qualify_paired_observations(&reference(), &observations),
            Err(EvidenceError::DuplicatePairId)
        );
        let mut observations = pairs();
        observations[1].candidate_c3.run_id = observations[0].candidate_c3.run_id;
        assert_eq!(
            qualify_paired_observations(&reference(), &observations),
            Err(EvidenceError::DuplicateRunId)
        );
        let mut observations = pairs();
        for (idx, obs) in observations.iter_mut().enumerate() {
            // Make all physical runs genuinely C0-first. The test then
            // isolates AB/BA allocation imbalance from chronology errors.
            let i = idx as u64;
            obs.order = RunOrder::BaselineFirst;
            obs.baseline_c0 = trial(10 + 2 * i, 120, 200 * i, 10_000 * i);
            obs.candidate_c3 = trial(11 + 2 * i, 90, 200 * i + 60, 10_000 * i + 1_000);
        }
        assert_eq!(
            qualify_paired_observations(&reference(), &observations),
            Err(EvidenceError::UnbalancedOrder)
        );
    }

    #[test]
    fn cross_pair_overlaps_and_counter_epochs_are_rejected() {
        let mut observations = pairs();
        // Each of these trials is individually well formed and correctly
        // ordered inside pair #2, but occupies pair #1's meter interval.
        observations[1].candidate_c3 = trial(13, 90, 0, 0);
        observations[1].baseline_c0 = trial(12, 120, 60, 1_000);
        assert_eq!(
            qualify_paired_observations(&reference(), &observations),
            Err(EvidenceError::InvalidRunChronology)
        );

        let mut observations = pairs();
        // Timestamps are disjoint and increasing, but the cumulative
        // meter starts a new epoch between consecutive pairs.
        observations[1].candidate_c3 = trial(13, 90, 200, 0);
        observations[1].baseline_c0 = trial(12, 120, 260, 1_000);
        assert_eq!(
            qualify_paired_observations(&reference(), &observations),
            Err(EvidenceError::InvalidRunChronology)
        );

        let mut observations = pairs();
        observations.swap(0, 1);
        // Pair identity and balancing remain valid; source row order is
        // not a license to reuse earlier physical measurement windows.
        assert_eq!(
            qualify_paired_observations(&reference(), &observations),
            Err(EvidenceError::InvalidRunChronology)
        );
    }

    #[test]
    fn reported_arm_order_must_match_actual_meter_timestamps() {
        let mut observations = pairs();
        observations[0].order = RunOrder::CandidateFirst;
        assert_eq!(
            qualify_paired_observations(&reference(), &observations),
            Err(EvidenceError::InvalidRunChronology)
        );
        let mut observations = pairs();
        observations[0]
            .candidate_c3
            .idle_before
            .begin
            .cumulative_microjoules = 0;
        assert_eq!(
            qualify_paired_observations(&reference(), &observations),
            Err(EvidenceError::InvalidRunChronology)
        );
    }

    #[test]
    fn zero_digest_and_missing_stable_protocol_are_rejected() {
        let mut observations = pairs();
        observations[0].baseline_c0.context.hardware_sha256 = [0; 32];
        assert_eq!(
            qualify_paired_observations(&reference(), &observations),
            Err(EvidenceError::InvalidContext)
        );
        let mut observations = pairs();
        observations[0].baseline_c0.context.protocol_revision = 0;
        assert_eq!(
            qualify_paired_observations(&reference(), &observations),
            Err(EvidenceError::InvalidContext)
        );
    }

    #[test]
    fn unqualified_reference_or_too_few_pairs_are_rejected() {
        assert_eq!(
            qualify_paired_observations(&reference(), &pairs()[..2]),
            Err(EvidenceError::PairCountOutsideBounds)
        );
        let mut observations = pairs();
        observations[0].candidate_c3.quality.outstanding = 1;
        assert_eq!(
            qualify_paired_observations(&reference(), &observations),
            Err(EvidenceError::IncorrectObservedQuality)
        );
    }
}
