//! Bounded non-final comparison of instantaneous and impulse-driven interaction.
//!
//! A single externally labelled event feeds four matched observers:
//! T3 transported / T4 six-channel, each with instantaneous or score-delta
//! hysteresis. Labels are supplied by the caller only after predictions;
//! they are never read by either scorer. This is a software evaluation harness,
//! not a confirmation of task accuracy, a validated dataset or an energy result.
//!
//! Every push is transactional. For explicit reference semantics, the current
//! BTreeMap-backed aggregate is cloned before mutation (O(K) per event).
//! Do NOT interpret this research harness as a sparse production fast path.

use super::tdi22_event_aggregate::{EventError, EventTorsorAggregate, TorsorEvent};
use super::tdi22_event_gate::{
    Edge, GateError, HystereticInteraction, InteractionArm, Observation,
};
use super::tdi22_event_impulse::{EventImpulseMemory, ImpulseError, ImpulseObservation};
use super::tdi22_torsor::{Twist3, Vec3};

pub const MAX_REFERENCE_EVENTS: u64 = 4_096;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProtocolError {
    Gate(GateError),
    Impulse(ImpulseError),
    Source(EventError),
    StaleSequence { previous: u64, submitted: u64 },
    EventLimit,
    CounterOverflow,
}

impl From<GateError> for ProtocolError {
    fn from(error: GateError) -> Self {
        Self::Gate(error)
    }
}

impl From<ImpulseError> for ProtocolError {
    fn from(error: ImpulseError) -> Self {
        Self::Impulse(error)
    }
}

impl From<EventError> for ProtocolError {
    fn from(error: EventError) -> Self {
        Self::Source(error)
    }
}

/// The four software controls have exactly the same source-event snapshots.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComparisonArm {
    InstantT3,
    InstantT4,
    ImpulseT3,
    ImpulseT4,
}

/// Confusion counts are compared without merging accuracy with sensitivity.
/// These metrics describe the supplied trace only; they are not estimates
/// of real-world model performance or externally verified labels.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ArmMetrics {
    pub true_positive: u64,
    pub false_positive: u64,
    pub true_negative: u64,
    pub false_negative: u64,
    pub rising_edges: u64,
    pub falling_edges: u64,
}

impl ArmMetrics {
    fn record(
        &mut self,
        prediction: bool,
        independent_target: bool,
        edge: Edge,
    ) -> Result<(), ProtocolError> {
        let slot = match (prediction, independent_target) {
            (true, true) => &mut self.true_positive,
            (true, false) => &mut self.false_positive,
            (false, false) => &mut self.true_negative,
            (false, true) => &mut self.false_negative,
        };
        *slot = slot.checked_add(1).ok_or(ProtocolError::CounterOverflow)?;
        match edge {
            Edge::Rising => {
                self.rising_edges = self
                    .rising_edges
                    .checked_add(1)
                    .ok_or(ProtocolError::CounterOverflow)?;
            }
            Edge::Falling => {
                self.falling_edges = self
                    .falling_edges
                    .checked_add(1)
                    .ok_or(ProtocolError::CounterOverflow)?;
            }
            Edge::None => {}
        };
        Ok(())
    }

    #[must_use]
    pub const fn total(self) -> u64 {
        self.true_positive + self.false_positive + self.true_negative + self.false_negative
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PairedStep {
    pub sequence: u64,
    pub live_sources: usize,
    pub instant_t3: Observation,
    pub instant_t4: Observation,
    pub impulse_t3: ImpulseObservation,
    pub impulse_t4: ImpulseObservation,
}

/// O(K)-per-event transactional reference comparison, not an optimized engine.
/// All four observers use identical lower/upper thresholds and observation
/// intervals. Both impulse observers have the same retention and state width.
#[derive(Clone, Debug, PartialEq)]
pub struct PairedEventProtocol {
    aggregate: EventTorsorAggregate,
    instant_t3: HystereticInteraction,
    instant_t4: HystereticInteraction,
    impulse_t3: EventImpulseMemory,
    impulse_t4: EventImpulseMemory,
    metrics: [ArmMetrics; 4],
    last_sequence: Option<u64>,
    events: u64,
}

impl PairedEventProtocol {
    pub fn new(
        query: Twist3,
        point: Vec3,
        lower: f64,
        upper: f64,
        retention: f64,
    ) -> Result<Self, ProtocolError> {
        let aggregate = EventTorsorAggregate::new();
        Ok(Self {
            instant_t3: HystereticInteraction::new(
                InteractionArm::TransportedT3,
                query,
                point,
                lower,
                upper,
            )?,
            instant_t4: HystereticInteraction::new(
                InteractionArm::SixChannelT4,
                query,
                point,
                lower,
                upper,
            )?,
            impulse_t3: EventImpulseMemory::new(
                InteractionArm::TransportedT3,
                query,
                point,
                retention,
                lower,
                upper,
                &aggregate,
            )?,
            impulse_t4: EventImpulseMemory::new(
                InteractionArm::SixChannelT4,
                query,
                point,
                retention,
                lower,
                upper,
                &aggregate,
            )?,
            aggregate,
            metrics: [ArmMetrics::default(); 4],
            last_sequence: None,
            events: 0,
        })
    }

    #[must_use]
    pub const fn processed_events(&self) -> u64 {
        self.events
    }

    #[must_use]
    pub fn metrics(&self, arm: ComparisonArm) -> ArmMetrics {
        self.metrics[match arm {
            ComparisonArm::InstantT3 => 0,
            ComparisonArm::InstantT4 => 1,
            ComparisonArm::ImpulseT3 => 2,
            ComparisonArm::ImpulseT4 => 3,
        }]
    }

    #[must_use]
    pub fn aggregate(&self) -> &EventTorsorAggregate {
        &self.aggregate
    }

    /// Apply the source event, run all four arms, and only *then* score an
    /// externally supplied task target. No label enters the candidate law.
    ///
    /// If any stage fails, even due to a T3/T4 score overflow, no source,
    /// observer, metric or sequence state is published.
    pub fn push(
        &mut self,
        sequence: u64,
        source_event: TorsorEvent,
        independent_target: bool,
    ) -> Result<PairedStep, ProtocolError> {
        if self.events >= MAX_REFERENCE_EVENTS {
            return Err(ProtocolError::EventLimit);
        }
        if let Some(previous) = self.last_sequence {
            if sequence <= previous {
                return Err(ProtocolError::StaleSequence {
                    previous,
                    submitted: sequence,
                });
            }
        }

        // Reference-only O(K) transactional staging, deliberately not an
        // O(log K) claim about the full coupled four-arm protocol.
        let mut staged = self.clone();
        staged.aggregate.apply(source_event)?;

        let instant_t3 = staged.instant_t3.observe(&staged.aggregate)?;
        let instant_t4 = staged.instant_t4.observe(&staged.aggregate)?;
        let impulse_t3 = staged
            .impulse_t3
            .observe_event(sequence, &staged.aggregate)?;
        let impulse_t4 = staged
            .impulse_t4
            .observe_event(sequence, &staged.aggregate)?;

        for (metric, (prediction, edge)) in staged.metrics.iter_mut().zip([
            (instant_t3.active, instant_t3.edge),
            (instant_t4.active, instant_t4.edge),
            (impulse_t3.active, impulse_t3.edge),
            (impulse_t4.active, impulse_t4.edge),
        ]) {
            metric.record(prediction, independent_target, edge)?;
        }
        staged.events = staged
            .events
            .checked_add(1)
            .ok_or(ProtocolError::CounterOverflow)?;
        staged.last_sequence = Some(sequence);

        let result = PairedStep {
            sequence,
            live_sources: staged.aggregate.len(),
            instant_t3,
            instant_t4,
            impulse_t3,
            impulse_t4,
        };
        *self = staged;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::super::tdi22_torsor::Torsor3;
    use super::*;

    fn v(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3::new(x, y, z).unwrap()
    }

    fn torsor(value: f64, point: Vec3) -> Torsor3 {
        Torsor3::new(v(value, 0.0, 0.0), Vec3::zero(), point).unwrap()
    }

    fn query() -> Twist3 {
        Twist3::new(v(1.0, 0.0, 0.0), v(0.0, 0.0, 1.0)).unwrap()
    }

    fn close(a: f64, b: f64) {
        let scale = a.abs().max(b.abs()).max(1.0);
        assert!((a - b).abs() <= 256.0 * f64::EPSILON * scale, "{a} != {b}");
    }

    fn run(labels: [bool; 4]) -> (PairedEventProtocol, [PairedStep; 4]) {
        let mut protocol =
            PairedEventProtocol::new(query(), v(0.0, 1.0, 0.0), 0.25, 1.5, 0.5).unwrap();
        let mut outcomes = Vec::new();
        for (index, (amplitude, target)) in [
            (Some(1.0), labels[0]),
            (Some(1.25), labels[1]),
            (Some(1.0), labels[2]),
            (None, labels[3]),
        ]
        .into_iter()
        .enumerate()
        {
            let event = match amplitude {
                Some(value) => TorsorEvent::Upsert {
                    id: 7,
                    torsor: torsor(value, Vec3::zero()),
                    weight: 1.0,
                },
                None => TorsorEvent::Remove { id: 7 },
            };
            outcomes.push(protocol.push(index as u64, event, target).unwrap());
        }
        (protocol, outcomes.try_into().unwrap())
    }

    #[test]
    fn independent_labels_are_counted_only_after_four_matched_predictions() {
        // Labels are deliberately external and NOT the target implied by
        // transported geometry; no mechanism is declared a task winner.
        let (protocol, trace) = run([false, true, false, true]);
        assert_eq!(protocol.processed_events(), 4);
        assert_eq!(trace[0].instant_t3.edge, Edge::Rising);
        assert_eq!(trace[0].instant_t4.edge, Edge::None);
        assert_eq!(trace[2].impulse_t3.edge, Edge::Falling);
        let instant = protocol.metrics(ComparisonArm::InstantT3);
        assert_eq!((instant.true_positive, instant.false_positive), (1, 2));
        assert_eq!((instant.true_negative, instant.false_negative), (0, 1));
        let impulse = protocol.metrics(ComparisonArm::ImpulseT3);
        assert_eq!((impulse.true_positive, impulse.false_positive), (1, 1));
        assert_eq!((impulse.true_negative, impulse.false_negative), (1, 1));
        let control = protocol.metrics(ComparisonArm::ImpulseT4);
        assert_eq!((control.true_positive, control.false_positive), (0, 0));
        assert_eq!((control.true_negative, control.false_negative), (2, 2));
        for arm in [
            ComparisonArm::InstantT3,
            ComparisonArm::InstantT4,
            ComparisonArm::ImpulseT3,
            ComparisonArm::ImpulseT4,
        ] {
            assert_eq!(protocol.metrics(arm).total(), 4);
        }
    }

    #[test]
    fn changing_only_external_targets_cannot_change_predictions_or_state() {
        let (a, outputs_a) = run([false, true, false, true]);
        let (b, outputs_b) = run([true, false, true, false]);
        assert_eq!(outputs_a, outputs_b);
        assert_eq!(a.aggregate(), b.aggregate());
        assert_ne!(
            a.metrics(ComparisonArm::InstantT3),
            b.metrics(ComparisonArm::InstantT3)
        );
    }

    #[test]
    fn no_signal_control_has_no_edges_for_any_arm() {
        let mut protocol =
            PairedEventProtocol::new(query(), v(0.0, 1.0, 0.0), 0.25, 1.5, 0.5).unwrap();
        for sequence in 0..16 {
            let step = protocol
                .push(
                    sequence,
                    TorsorEvent::Upsert {
                        id: 0,
                        torsor: torsor(0.0, Vec3::zero()),
                        weight: 1.0,
                    },
                    sequence % 2 == 0,
                )
                .unwrap();
            assert!(!step.instant_t3.active);
            assert!(!step.instant_t4.active);
            assert!(!step.impulse_t3.active);
            assert!(!step.impulse_t4.active);
        }
        for arm in [
            ComparisonArm::InstantT3,
            ComparisonArm::InstantT4,
            ComparisonArm::ImpulseT3,
            ComparisonArm::ImpulseT4,
        ] {
            let m = protocol.metrics(arm);
            assert_eq!(m.true_positive + m.false_positive, 0);
            assert_eq!((m.true_negative, m.false_negative), (8, 8));
        }
    }

    #[test]
    fn identical_source_and_observer_states_rollback_on_bad_events() {
        let mut protocol =
            PairedEventProtocol::new(query(), v(0.0, 1.0, 0.0), 0.25, 1.5, 0.5).unwrap();
        protocol
            .push(
                7,
                TorsorEvent::Upsert {
                    id: 5,
                    torsor: torsor(1.0, Vec3::zero()),
                    weight: 1.0,
                },
                true,
            )
            .unwrap();
        let snapshot = protocol.clone();
        assert_eq!(
            protocol.push(7, TorsorEvent::Remove { id: 5 }, false),
            Err(ProtocolError::StaleSequence {
                previous: 7,
                submitted: 7
            })
        );
        assert_eq!(protocol, snapshot);
        assert_eq!(
            protocol.push(8, TorsorEvent::Remove { id: 123 }, false),
            Err(ProtocolError::Source(EventError::MissingSource(123)))
        );
        assert_eq!(protocol, snapshot);
        assert_eq!(
            protocol.push(
                8,
                TorsorEvent::Upsert {
                    id: 5,
                    torsor: torsor(2.0, Vec3::zero()),
                    weight: f64::NAN,
                },
                false,
            ),
            Err(ProtocolError::Source(EventError::InvalidWeight))
        );
        assert_eq!(protocol, snapshot);
        // A mathematically valid source mutation can produce a non-finite
        // derived score. The entire *coupled protocol* must still roll back.
        assert!(
            protocol
                .push(
                    8,
                    TorsorEvent::Upsert {
                        id: 5,
                        torsor: torsor(f64::MAX, Vec3::zero()),
                        weight: 1.0,
                    },
                    false,
                )
                .is_err()
        );
        assert_eq!(protocol, snapshot);
    }

    #[test]
    fn translated_t3_interaction_traces_match_independent_target_labels() {
        let shift = v(3.0, -2.0, 5.0);
        let point = v(0.0, 1.0, 0.0);
        let mut a = PairedEventProtocol::new(query(), point, 0.25, 1.5, 0.5).unwrap();
        let mut b = PairedEventProtocol::new(query(), point.plus(shift), 0.25, 1.5, 0.5).unwrap();
        for (sequence, (value, target)) in [(1.0, false), (1.25, true), (1.0, false)]
            .into_iter()
            .enumerate()
        {
            let left = a
                .push(
                    sequence as u64,
                    TorsorEvent::Upsert {
                        id: 1,
                        torsor: torsor(value, Vec3::zero()),
                        weight: 1.0,
                    },
                    target,
                )
                .unwrap();
            let right = b
                .push(
                    sequence as u64,
                    TorsorEvent::Upsert {
                        id: 1,
                        torsor: torsor(value, shift),
                        weight: 1.0,
                    },
                    target,
                )
                .unwrap();
            close(left.instant_t3.score, right.instant_t3.score);
            close(left.impulse_t3.memory, right.impulse_t3.memory);
            assert_eq!(left.instant_t3.edge, right.instant_t3.edge);
            assert_eq!(left.impulse_t3.edge, right.impulse_t3.edge);
        }
        assert_eq!(
            a.metrics(ComparisonArm::ImpulseT3),
            b.metrics(ComparisonArm::ImpulseT3)
        );
    }
}
