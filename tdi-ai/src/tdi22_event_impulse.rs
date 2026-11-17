//! Experimental torsor-driven impulse memory for event-only updates.
//!
//! The hypothesis is a path-dependent scalar state driven by differences of
//! successive T3 or matched-width T4 scores. It is NOT an experimentally
//! discovered law, an attention improvement, or measured energy efficiency.
//! Source events are applied externally to the event torsor aggregate. The
//! caller supplies a strictly increasing sequence for each observation.

use super::tdi22_event_aggregate::EventTorsorAggregate;
use super::tdi22_event_gate::{Edge, GateError, HystereticInteraction, InteractionArm};
use super::tdi22_torsor::{Twist3, Vec3};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImpulseError {
    InvalidRetention,
    Gate(GateError),
    StaleSequence { last: u64, received: u64 },
    NonFiniteDelta,
    NonFiniteMemory,
}

impl From<GateError> for ImpulseError {
    fn from(error: GateError) -> Self {
        Self::Gate(error)
    }
}

/// A transition produced only when the caller delivers an event.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImpulseObservation {
    pub sequence: u64,
    pub score: f64,
    pub score_delta: f64,
    pub memory: f64,
    pub active: bool,
    pub edge: Edge,
}

/// One scalar memory, one previous score, one active bit, one sequence stamp.
///
/// Given a freshly observed score s[n] and previous s[n-1]:
///
///   delta[n] = s[n] - s[n-1]
///   memory[n] = retention * memory[n-1] + delta[n]
///
/// A rising edge occurs if inactive and memory >= upper; a falling edge
/// occurs if active and memory <= lower. Between events no tick is applied.
/// This is a research candidate recurrence, not a calibrated physical impulse.
///
/// Both T3 and T4 arms use the same recurrence and retained state width.
/// Observation is transactional: invalid arithmetic or stale sequence must
/// leave every field unchanged. The sequence is caller-supplied provenance;
/// this module does not authenticate or enforce upstream event delivery.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EventImpulseMemory {
    scorer: HystereticInteraction,
    retention: f64,
    lower: f64,
    upper: f64,
    previous_score: f64,
    memory: f64,
    active: bool,
    last_sequence: Option<u64>,
}

impl EventImpulseMemory {
    /// Bind the initial score to the *current* aggregate. No synthetic
    /// impulse is introduced if an observer starts after sources are loaded.
    pub fn new(
        arm: InteractionArm,
        twist: Twist3,
        point: Vec3,
        retention: f64,
        lower: f64,
        upper: f64,
        aggregate: &EventTorsorAggregate,
    ) -> Result<Self, ImpulseError> {
        if !retention.is_finite() || !(0.0..=1.0).contains(&retention) {
            return Err(ImpulseError::InvalidRetention);
        }
        let scorer = HystereticInteraction::new(arm, twist, point, lower, upper)?;
        let previous_score = scorer.score(aggregate)?;
        Ok(Self {
            scorer,
            retention,
            lower,
            upper,
            previous_score,
            memory: 0.0,
            active: false,
            last_sequence: None,
        })
    }

    #[must_use]
    pub const fn memory(&self) -> f64 {
        self.memory
    }

    #[must_use]
    pub const fn active(&self) -> bool {
        self.active
    }

    #[must_use]
    pub const fn last_sequence(&self) -> Option<u64> {
        self.last_sequence
    }

    /// Apply one declared new event boundary. The aggregate was already
    /// changed by the caller; this update does not mutate that aggregate.
    pub fn observe_event(
        &mut self,
        sequence: u64,
        aggregate: &EventTorsorAggregate,
    ) -> Result<ImpulseObservation, ImpulseError> {
        if let Some(last) = self.last_sequence {
            if sequence <= last {
                return Err(ImpulseError::StaleSequence {
                    last,
                    received: sequence,
                });
            }
        }
        let score = self.scorer.score(aggregate)?;
        let delta = score - self.previous_score;
        if !delta.is_finite() {
            return Err(ImpulseError::NonFiniteDelta);
        }
        let memory = self.retention * self.memory + delta;
        if !memory.is_finite() {
            return Err(ImpulseError::NonFiniteMemory);
        }
        let edge = if !self.active && memory >= self.upper {
            Edge::Rising
        } else if self.active && memory <= self.lower {
            Edge::Falling
        } else {
            Edge::None
        };
        let active = match edge {
            Edge::Rising => true,
            Edge::Falling => false,
            Edge::None => self.active,
        };

        // All potentially failing arithmetic has completed.
        self.previous_score = score;
        self.memory = memory;
        self.active = active;
        self.last_sequence = Some(sequence);
        Ok(ImpulseObservation {
            sequence,
            score,
            score_delta: delta,
            memory,
            active,
            edge,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::tdi22_event_aggregate::TorsorEvent;
    use super::super::tdi22_torsor::{Torsor3, direct_pairing};

    fn v(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3::new(x, y, z).unwrap()
    }

    fn source(r: Vec3, m: Vec3, p: Vec3) -> Torsor3 {
        Torsor3::new(r, m, p).unwrap()
    }

    fn close(a: f64, b: f64) {
        let scale = 1.0_f64.max(a.abs()).max(b.abs());
        assert!((a - b).abs() <= 256.0 * f64::EPSILON * scale, "{a:?} != {b:?}");
    }

    #[test]
    fn signed_event_impulses_produce_distinct_t3_t4_edges_with_matched_memory() {
        let mut aggregate = EventTorsorAggregate::new();
        let twist = Twist3::new(v(1.0, 0.0, 0.0), v(0.0, 0.0, 1.0)).unwrap();
        let point = v(0.0, 1.0, 0.0);
        let mut t3 = EventImpulseMemory::new(
            InteractionArm::TransportedT3, twist, point, 0.5, 0.25, 1.5, &aggregate,
        ).unwrap();
        let mut t4 = EventImpulseMemory::new(
            InteractionArm::SixChannelT4, twist, point, 0.5, 0.25, 1.5, &aggregate,
        ).unwrap();

        let stream = [
            (Some(1.0), Edge::Rising),
            (Some(1.25), Edge::None),
            (Some(1.0), Edge::Falling),
            (None, Edge::None),
        ];
        for (index, (amplitude, expected_t3_edge)) in stream.into_iter().enumerate() {
            let event = match amplitude {
                Some(value) => TorsorEvent::Upsert {
                    id: 9,
                    torsor: source(v(value, 0.0, 0.0), Vec3::zero(), Vec3::zero()),
                    weight: 1.0,
                },
                None => TorsorEvent::Remove { id: 9 },
            };
            aggregate.apply(event).unwrap();
            let a = t3.observe_event(index as u64, &aggregate).unwrap();
            let b = t4.observe_event(index as u64, &aggregate).unwrap();
            close(a.score, 2.0 * amplitude.unwrap_or(0.0));
            close(b.score, amplitude.unwrap_or(0.0));
            assert_eq!(a.edge, expected_t3_edge);
            assert_eq!(b.edge, Edge::None);
            assert_eq!(a.sequence, b.sequence);
        }
        assert!(!t3.active());
        assert!(!t4.active());
        close(t3.memory(), -1.875);
        close(t4.memory(), -0.9375);
    }

    #[test]
    fn memory_is_event_order_dependent_even_with_identical_last_source_state() {
        let twist = Twist3::new(v(1.0, 0.0, 0.0), Vec3::zero()).unwrap();
        let mut first = EventTorsorAggregate::new();
        let mut second = EventTorsorAggregate::new();
        let mut first_mem = EventImpulseMemory::new(
            InteractionArm::SixChannelT4, twist, Vec3::zero(), 0.5, -10.0, 10.0, &first,
        ).unwrap();
        let mut second_mem = EventImpulseMemory::new(
            InteractionArm::SixChannelT4, twist, Vec3::zero(), 0.5, -10.0, 10.0, &second,
        ).unwrap();
        for (index, value) in [1.0, 2.0, 1.0].into_iter().enumerate() {
            first.apply(TorsorEvent::Upsert {
                id: 1,
                torsor: source(v(value, 0.0, 0.0), Vec3::zero(), Vec3::zero()),
                weight: 1.0,
            }).unwrap();
            first_mem.observe_event(index as u64, &first).unwrap();
        }
        for (index, value) in [2.0, 1.0].into_iter().enumerate() {
            second.apply(TorsorEvent::Upsert {
                id: 1,
                torsor: source(v(value, 0.0, 0.0), Vec3::zero(), Vec3::zero()),
                weight: 1.0,
            }).unwrap();
            second_mem.observe_event(index as u64, &second).unwrap();
        }
        assert_eq!(first.factorized_key(), second.factorized_key());
        close(first_mem.memory(), -0.25);
        close(second_mem.memory(), 0.0);
    }

    #[test]
    fn global_translation_preserves_t3_impulse_trace() {
        let twist = Twist3::new(v(1.0, -2.0, 3.0), v(0.0, 0.0, 1.0)).unwrap();
        let p = v(1.0, 0.0, 0.0);
        let q = v(2.0, 1.0, 0.0);
        let shift = v(3.0, -4.0, 5.0);
        let mut original = EventTorsorAggregate::new();
        let mut translated = EventTorsorAggregate::new();
        let mut a = EventImpulseMemory::new(
            InteractionArm::TransportedT3, twist, q, 0.75, -0.5, 0.5, &original,
        ).unwrap();
        let mut b = EventImpulseMemory::new(
            InteractionArm::TransportedT3, twist, q.plus(shift), 0.75, -0.5, 0.5, &translated,
        ).unwrap();
        for (id, r) in [
            (10, v(1.0, 2.0, 0.0)),
            (11, v(-0.5, 1.0, 0.0)),
            (12, v(0.75, -2.0, 0.0)),
        ] {
            let t = source(r, v(0.0, 0.0, 2.0), p);
            original.apply(TorsorEvent::Upsert { id, torsor: t, weight: 1.0 }).unwrap();
            translated.apply(TorsorEvent::Upsert {
                id, torsor: source(r, t.moment(), p.plus(shift)), weight: 1.0,
            }).unwrap();
            let x = a.observe_event(id, &original).unwrap();
            let y = b.observe_event(id, &translated).unwrap();
            close(x.score, y.score);
            close(x.score_delta, y.score_delta);
            close(x.memory, y.memory);
            assert_eq!(x.edge, y.edge);
            assert_eq!(x.active, y.active);
        }
        let key = source(v(1.0, 2.0, 0.0), v(0.0, 0.0, 2.0), p);
        close(direct_pairing(twist, key, q).unwrap(), -2.0);
    }

    #[test]
    fn baseline_binds_to_preloaded_aggregate_without_initial_artificial_pulse() {
        let mut aggregate = EventTorsorAggregate::new();
        let twist = Twist3::new(v(1.0, 0.0, 0.0), Vec3::zero()).unwrap();
        aggregate.apply(TorsorEvent::Upsert {
            id: 1, torsor: source(v(3.0, 0.0, 0.0), Vec3::zero(), Vec3::zero()), weight: 1.0,
        }).unwrap();
        let mut memory = EventImpulseMemory::new(
            InteractionArm::SixChannelT4, twist, Vec3::zero(), 0.0, -0.5, 0.5, &aggregate,
        ).unwrap();
        let observed = memory.observe_event(1, &aggregate).unwrap();
        close(observed.score, 3.0);
        close(observed.score_delta, 0.0);
        close(observed.memory, 0.0);
        assert_eq!(observed.edge, Edge::None);
    }

    #[test]
    fn stale_event_sequences_and_invalid_parameters_fail_closed() {
        let aggregate = EventTorsorAggregate::new();
        let twist = Twist3::new(v(1.0, 0.0, 0.0), Vec3::zero()).unwrap();
        for retention in [f64::NAN, f64::INFINITY, -0.1, 1.1] {
            assert_eq!(
                EventImpulseMemory::new(
                    InteractionArm::SixChannelT4, twist, Vec3::zero(),
                    retention, -1.0, 1.0, &aggregate,
                ),
                Err(ImpulseError::InvalidRetention)
            );
        }
        let mut memory = EventImpulseMemory::new(
            InteractionArm::SixChannelT4, twist, Vec3::zero(),
            0.5, -1.0, 1.0, &aggregate,
        ).unwrap();
        memory.observe_event(100, &aggregate).unwrap();
        let before = memory;
        assert_eq!(
            memory.observe_event(100, &aggregate),
            Err(ImpulseError::StaleSequence { last: 100, received: 100 })
        );
        assert_eq!(
            memory.observe_event(99, &aggregate),
            Err(ImpulseError::StaleSequence { last: 100, received: 99 })
        );
        assert_eq!(memory, before);
    }

    #[test]
    fn overflowing_score_difference_does_not_commit_partial_memory() {
        let mut aggregate = EventTorsorAggregate::new();
        let twist = Twist3::new(v(1.0, 0.0, 0.0), Vec3::zero()).unwrap();
        let mut memory = EventImpulseMemory::new(
            InteractionArm::SixChannelT4, twist, Vec3::zero(), 1.0, -1.0, 1.0, &aggregate,
        ).unwrap();
        aggregate.apply(TorsorEvent::Upsert {
            id: 1,
            torsor: source(v(f64::MAX, 0.0, 0.0), Vec3::zero(), Vec3::zero()),
            weight: 1.0,
        }).unwrap();
        memory.observe_event(0, &aggregate).unwrap();
        let before = memory;
        // Two source mutations without an intervening observer event:
        // each aggregate mutation succeeds, but the net score jump is -INF.
        aggregate.apply(TorsorEvent::Remove { id: 1 }).unwrap();
        aggregate.apply(TorsorEvent::Upsert {
            id: 1,
            torsor: source(v(-f64::MAX, 0.0, 0.0), Vec3::zero(), Vec3::zero()),
            weight: 1.0,
        }).unwrap();
        assert_eq!(memory.observe_event(1, &aggregate), Err(ImpulseError::NonFiniteDelta));
        assert_eq!(memory, before);
    }
}
