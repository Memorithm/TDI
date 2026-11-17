//! Non-final, task-defined Boolean event campaigns for TDI-22 interactions.
//!
//! Task labels are Boolean relations over an independently generated symbol
//! history, NEVER a torsor score, a geometry-dependent oracle, or a learned
//! target. All four frozen-reference event observers receive exactly the same
//! input sequence. This diagnostic does not claim learned intelligence,
//! scientific confirmation, physical energy savings, or statistical power.

use super::tdi22_event_aggregate::TorsorEvent;
use super::tdi22_event_protocol::{
    ArmMetrics, ComparisonArm, PairedEventProtocol, PairedStep, ProtocolError,
};
use super::tdi22_torsor::{Torsor3, TorsorError, Twist3, Vec3};

const STREAM_DOMAIN_A: u64 = 0x5444_4932_3245_5641;
const STREAM_DOMAIN_B: u64 = 0x5444_4932_3245_5642;
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
pub const MIN_DIAGNOSTIC_EVENTS: u16 = 32;
pub const MAX_DIAGNOSTIC_EVENTS: u16 = 256;

/// Both partitions are public diagnostic data, never protected final splits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticPartition {
    Development,
    Stress,
}

impl DiagnosticPartition {
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Stress => "stress",
        }
    }
}

/// Label-generating rules are independent of the torsor projection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BooleanTask {
    /// True if current symbol equals the symbol three events earlier.
    LagThreeMatch,
    /// True on a 0-to-1 transition in the external symbol stream.
    RisingBit,
    /// True if the most recent four symbols have odd XOR parity.
    FourBitParity,
    /// Negative control: rotate LagThreeMatch labels, leave input unchanged.
    PermutedLagThree,
    /// Negative control: hide all symbols from the torsor input, keep labels.
    NoSignalLagThree,
}

impl BooleanTask {
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::LagThreeMatch => "lag3-match",
            Self::RisingBit => "rising-bit",
            Self::FourBitParity => "parity4",
            Self::PermutedLagThree => "permuted-lag3",
            Self::NoSignalLagThree => "no-signal-lag3",
        }
    }

    #[must_use]
    const fn index(self) -> u64 {
        match self {
            Self::LagThreeMatch => 1,
            Self::RisingBit => 2,
            Self::FourBitParity => 3,
            Self::PermutedLagThree => 4,
            Self::NoSignalLagThree => 5,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaskConfig {
    pub partition: DiagnosticPartition,
    pub task: BooleanTask,
    pub seed: u64,
    pub events: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CampaignError {
    InvalidLength { observed: u16 },
    Algebra(TorsorError),
    Protocol(ProtocolError),
}

impl From<TorsorError> for CampaignError {
    fn from(error: TorsorError) -> Self {
        Self::Algebra(error)
    }
}

impl From<ProtocolError> for CampaignError {
    fn from(error: ProtocolError) -> Self {
        Self::Protocol(error)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TaskFrame {
    pub sequence: u64,
    pub symbol: bool,
    pub independent_target: bool,
    /// The source encodes the symbol, but never the target or its history.
    pub source: Torsor3,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CampaignRow {
    pub frame: TaskFrame,
    pub predictions: PairedStep,
    /// T4 reference re-run at half the thresholds, not a learned calibration.
    pub scale_matched_predictions: PairedStep,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CampaignReport {
    pub config: TaskConfig,
    pub stream_fingerprint: u64,
    pub rows: Vec<CampaignRow>,
    pub metrics: [ArmMetrics; 4],
    /// T4 at analytically halved thresholds, for the same source stream.
    /// Index 0 is instantaneous, index 1 is impulse memory.
    pub scale_matched_t4: [ArmMetrics; 2],
    pub always_negative: ArmMetrics,
    /// This is the source Boolean oracle count, not a deployable baseline.
    pub source_oracle_correct: u64,
}

impl CampaignReport {
    #[must_use]
    pub fn metrics_for(&self, arm: ComparisonArm) -> ArmMetrics {
        self.metrics[match arm {
            ComparisonArm::InstantT3 => 0,
            ComparisonArm::InstantT4 => 1,
            ComparisonArm::ImpulseT3 => 2,
            ComparisonArm::ImpulseT4 => 3,
        }]
    }
}

/// Fixed draw order: one symbol bit and three independent nuisance words
/// per event. No task label participates in this pseudo-random stream.
#[derive(Clone, Copy)]
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64, partition: DiagnosticPartition) -> Self {
        Self {
            state: seed
                ^ match partition {
                    DiagnosticPartition::Development => STREAM_DOMAIN_A,
                    DiagnosticPartition::Stress => STREAM_DOMAIN_B,
                },
        }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
}

fn label_for(task: BooleanTask, history: &[bool], index: usize) -> bool {
    match task {
        BooleanTask::LagThreeMatch
        | BooleanTask::PermutedLagThree
        | BooleanTask::NoSignalLagThree => index >= 3 && history[index] == history[index - 3],
        BooleanTask::RisingBit => index > 0 && history[index] && !history[index - 1],
        BooleanTask::FourBitParity => {
            index >= 3
                && history[index - 3..=index]
                    .iter()
                    .copied()
                    .fold(false, |parity, bit| parity ^ bit)
        }
    }
}

fn nuisance_coordinate(word: u64) -> f64 {
    // All four values are exact binary fractions: -0.75, -0.25, 0.25, 0.75.
    ((word & 3) as f64 - 1.5) * 0.5
}

fn fingerprint(config: TaskConfig, frames: &[TaskFrame]) -> u64 {
    let mut digest = FNV_OFFSET;
    let header = [
        config.seed,
        u64::from(config.events),
        config.task.index(),
        match config.partition {
            DiagnosticPartition::Development => STREAM_DOMAIN_A,
            DiagnosticPartition::Stress => STREAM_DOMAIN_B,
        },
    ];
    for word in header.into_iter().chain(frames.iter().flat_map(|frame| {
        [
            frame.sequence,
            u64::from(frame.symbol),
            u64::from(frame.independent_target),
            frame.source.resultant().x.to_bits(),
            frame.source.reference().x.to_bits(),
            frame.source.reference().y.to_bits(),
            frame.source.reference().z.to_bits(),
        ]
    })) {
        digest = (digest ^ word).wrapping_mul(FNV_PRIME);
    }
    digest
}

/// Generate a source stream and task labels *before* running any T3/T4 arm.
///
/// Both the coordinate-nuisance words and the raw symbol history are drawn
/// without access to the scoring law. All source torsors satisfy C=0 by
/// construction (M(P) = -P x R), so random coordinate nuisance cannot
/// define or leak a target. This also exposes the T3/T4 threshold-scale
/// confound: T3 scores twice the T4 score for the fixed query below.
pub fn generate_task(config: TaskConfig) -> Result<Vec<TaskFrame>, CampaignError> {
    if !(MIN_DIAGNOSTIC_EVENTS..=MAX_DIAGNOSTIC_EVENTS).contains(&config.events) {
        return Err(CampaignError::InvalidLength {
            observed: config.events,
        });
    }
    let mut rng = SplitMix64::new(config.seed, config.partition);
    let mut symbols = Vec::with_capacity(config.events as usize);
    let mut frames = Vec::with_capacity(config.events as usize);

    for index in 0..usize::from(config.events) {
        let symbol = (rng.next_u64() & 1) != 0;
        let point = Vec3::new(
            nuisance_coordinate(rng.next_u64()),
            nuisance_coordinate(rng.next_u64()),
            nuisance_coordinate(rng.next_u64()),
        )?;
        let amplitude = if config.task == BooleanTask::NoSignalLagThree {
            0.0
        } else if symbol {
            1.0
        } else {
            -1.0
        };
        let resultant = Vec3::new(amplitude, 0.0, 0.0)?;
        let moment = Vec3::zero().minus(point.cross(resultant));
        let source = Torsor3::new(resultant, moment, point)?;
        symbols.push(symbol);
        frames.push(TaskFrame {
            sequence: index as u64,
            symbol,
            independent_target: false,
            source,
        });
    }

    let labels: Vec<bool> = (0..symbols.len())
        .map(|index| label_for(config.task, &symbols, index))
        .collect();
    for (index, frame) in frames.iter_mut().enumerate() {
        frame.independent_target = if config.task == BooleanTask::PermutedLagThree {
            labels[(index + 11) % labels.len()]
        } else {
            labels[index]
        };
    }
    Ok(frames)
}

/// Run all four arms without any fitting, tuning or label-conditioned
/// parameter choice. This function actually executes the diagnostic trace.
pub fn run_campaign(config: TaskConfig) -> Result<CampaignReport, CampaignError> {
    let frames = generate_task(config)?;
    let stream_fingerprint = fingerprint(config, &frames);
    let twist = Twist3::new(Vec3::new(1.0, 0.0, 0.0)?, Vec3::new(0.0, 0.0, 1.0)?)?;
    let mut protocol = PairedEventProtocol::new(twist, Vec3::new(0.0, 1.0, 0.0)?, 0.25, 1.5, 0.5)?;
    // On this C=0 test carrier, T3=2*T4 exactly for the declared twist.
    // Halving T4 thresholds is a precomputed scale-control, not a
    // data-driven or label-fitted model adjustment.
    let mut scale_control =
        PairedEventProtocol::new(twist, Vec3::new(0.0, 1.0, 0.0)?, 0.125, 0.75, 0.5)?;
    let mut rows = Vec::with_capacity(frames.len());
    let mut positives = 0_u64;
    for frame in frames {
        let predictions = protocol.push(
            frame.sequence,
            TorsorEvent::Upsert {
                id: 7,
                torsor: frame.source,
                weight: 1.0,
            },
            frame.independent_target,
        )?;
        let scale_matched_predictions = scale_control.push(
            frame.sequence,
            TorsorEvent::Upsert {
                id: 7,
                torsor: frame.source,
                weight: 1.0,
            },
            frame.independent_target,
        )?;
        if frame.independent_target {
            positives += 1;
        }
        rows.push(CampaignRow {
            frame,
            predictions,
            scale_matched_predictions,
        });
    }
    let total = config.events as u64;
    Ok(CampaignReport {
        config,
        stream_fingerprint,
        rows,
        metrics: [
            protocol.metrics(ComparisonArm::InstantT3),
            protocol.metrics(ComparisonArm::InstantT4),
            protocol.metrics(ComparisonArm::ImpulseT3),
            protocol.metrics(ComparisonArm::ImpulseT4),
        ],
        scale_matched_t4: [
            scale_control.metrics(ComparisonArm::InstantT4),
            scale_control.metrics(ComparisonArm::ImpulseT4),
        ],
        always_negative: ArmMetrics {
            true_negative: total - positives,
            false_negative: positives,
            ..ArmMetrics::default()
        },
        source_oracle_correct: total,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(task: BooleanTask) -> TaskConfig {
        TaskConfig {
            partition: DiagnosticPartition::Development,
            task,
            seed: 17,
            events: 64,
        }
    }

    #[test]
    fn oracle_rules_are_boolean_history_not_torsor_scores() {
        let bits = [false, true, true, false, true, false];
        let lag: Vec<bool> = (0..bits.len())
            .map(|i| label_for(BooleanTask::LagThreeMatch, &bits, i))
            .collect();
        assert_eq!(lag, vec![false, false, false, true, true, false]);
        let rise: Vec<bool> = (0..bits.len())
            .map(|i| label_for(BooleanTask::RisingBit, &bits, i))
            .collect();
        assert_eq!(rise, vec![false, true, false, false, true, false]);
        assert!(!label_for(BooleanTask::FourBitParity, &bits, 3));
        assert!(label_for(BooleanTask::FourBitParity, &bits, 4));
    }

    #[test]
    fn deterministic_replay_and_partition_isolation() {
        let c = config(BooleanTask::LagThreeMatch);
        let first = generate_task(c).unwrap();
        assert_eq!(first, generate_task(c).unwrap());
        let changed_partition = generate_task(TaskConfig {
            partition: DiagnosticPartition::Stress,
            ..c
        })
        .unwrap();
        assert_ne!(first, changed_partition);
        let report_a = run_campaign(c).unwrap();
        assert_eq!(report_a, run_campaign(c).unwrap());
        assert_ne!(
            report_a.stream_fingerprint,
            run_campaign(TaskConfig {
                partition: DiagnosticPartition::Stress,
                ..c
            })
            .unwrap()
            .stream_fingerprint
        );
    }

    #[test]
    fn targets_are_independent_of_coordinate_nuisance() {
        let frames = generate_task(config(BooleanTask::FourBitParity)).unwrap();
        for frame in frames {
            assert_eq!(frame.source.origin_moment().unwrap(), Vec3::zero());
            assert!(frame.source.resultant().x.abs() == 1.0);
        }
    }

    #[test]
    fn all_task_families_execute_and_keep_four_arm_counts_matched() {
        for kind in [
            BooleanTask::LagThreeMatch,
            BooleanTask::RisingBit,
            BooleanTask::FourBitParity,
            BooleanTask::PermutedLagThree,
            BooleanTask::NoSignalLagThree,
        ] {
            let result = run_campaign(config(kind)).unwrap();
            assert_eq!(result.rows.len(), 64);
            for metrics in result.metrics {
                assert_eq!(metrics.total(), 64);
            }
            assert_eq!(result.always_negative.total(), 64);
            for metrics in result.scale_matched_t4 {
                assert_eq!(metrics.total(), 64);
            }
            assert_eq!(result.source_oracle_correct, 64);
        }
    }

    #[test]
    fn permuted_targets_change_only_metrics_not_predictions() {
        let natural = run_campaign(config(BooleanTask::LagThreeMatch)).unwrap();
        let permuted = run_campaign(config(BooleanTask::PermutedLagThree)).unwrap();
        let before: Vec<_> = natural.rows.iter().map(|r| r.predictions).collect();
        let after: Vec<_> = permuted.rows.iter().map(|r| r.predictions).collect();
        assert_eq!(before, after);
        assert!(
            natural
                .rows
                .iter()
                .zip(&permuted.rows)
                .any(|(a, b)| { a.frame.independent_target != b.frame.independent_target })
        );
    }

    #[test]
    fn zero_signal_negative_control_cannot_learn_hidden_labels() {
        let normal = generate_task(config(BooleanTask::LagThreeMatch)).unwrap();
        let null = run_campaign(config(BooleanTask::NoSignalLagThree)).unwrap();
        assert!(null.rows.iter().zip(&normal).all(|(row, ordinary)| {
            row.frame.independent_target == ordinary.independent_target
        }));
        assert!(null.rows.iter().any(|row| row.frame.independent_target));
        for arm in null.metrics {
            assert_eq!(arm, null.always_negative);
        }
        assert!(null.rows.iter().all(|row| {
            !row.predictions.instant_t3.active
                && !row.predictions.instant_t4.active
                && !row.predictions.impulse_t3.active
                && !row.predictions.impulse_t4.active
        }));
    }

    #[test]
    fn analytically_scaled_t4_reproduces_t3_for_content_only_input() {
        // This is an explicit falsification of interpreting a T3-vs-T4
        // difference on the present carrier as a torsor-specific advantage.
        for task in [
            BooleanTask::LagThreeMatch,
            BooleanTask::RisingBit,
            BooleanTask::FourBitParity,
            BooleanTask::PermutedLagThree,
            BooleanTask::NoSignalLagThree,
        ] {
            let result = run_campaign(config(task)).unwrap();
            assert_eq!(result.metrics[0], result.scale_matched_t4[0]);
            assert_eq!(result.metrics[2], result.scale_matched_t4[1]);
            for row in result.rows {
                assert_eq!(
                    row.predictions.instant_t3.active,
                    row.scale_matched_predictions.instant_t4.active
                );
                assert_eq!(
                    row.predictions.instant_t3.edge,
                    row.scale_matched_predictions.instant_t4.edge
                );
                assert_eq!(
                    row.predictions.impulse_t3.active,
                    row.scale_matched_predictions.impulse_t4.active
                );
                assert_eq!(
                    row.predictions.impulse_t3.edge,
                    row.scale_matched_predictions.impulse_t4.edge
                );
            }
        }
    }

    #[test]
    fn invalid_stream_lengths_fail_closed() {
        for events in [0, 31, 257, u16::MAX] {
            let c = TaskConfig {
                events,
                ..config(BooleanTask::RisingBit)
            };
            assert_eq!(
                run_campaign(c),
                Err(CampaignError::InvalidLength { observed: events })
            );
        }
    }
}
