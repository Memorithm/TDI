//! Experimental event-driven interaction gates over the frozen TDI-22 algebra.
//!
//! One-bit hysteresis is compared for transported T3 and matched-width T4.
//! No scientific superiority, saved computation, or energy claim is made.
//! Observe an EventTorsorAggregate after updating it; this observer does not
//! modify the source and refuses non-finite results without changing state.

use super::tdi22_event_aggregate::{EventError, EventTorsorAggregate};
use super::tdi22_torsor::{TorsorError, Twist3, Vec3};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InteractionArm {
    /// Score: (v + Q cross omega) dot R + omega dot C.
    TransportedT3,
    /// Matched six-channel control: v dot R + omega dot C.
    SixChannelT4,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    None,
    Rising,
    Falling,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Observation {
    pub score: f64,
    pub active: bool,
    pub edge: Edge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GateError {
    InvalidThresholds,
    InvalidPoint(TorsorError),
    Torsor(EventError),
    NonFiniteScore,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HystereticInteraction {
    arm: InteractionArm,
    twist: Twist3,
    point: Vec3,
    lower: f64,
    upper: f64,
    active: bool,
}

impl HystereticInteraction {
    pub fn new(
        arm: InteractionArm,
        twist: Twist3,
        point: Vec3,
        lower: f64,
        upper: f64,
    ) -> Result<Self, GateError> {
        if !lower.is_finite() || !upper.is_finite() || lower >= upper {
            return Err(GateError::InvalidThresholds);
        }
        // The frozen factorization validates a query position without
        // altering the TDI-22 torsor contract.
        Twist3::new(Vec3::zero(), Vec3::zero())
            .expect("finite zero vectors")
            .factorized_at(point)
            .map_err(GateError::InvalidPoint)?;
        Ok(Self {
            arm,
            twist,
            point,
            lower,
            upper,
            active: false,
        })
    }

    #[must_use]
    pub const fn active(&self) -> bool {
        self.active
    }

    pub fn score(&self, aggregate: &EventTorsorAggregate) -> Result<f64, GateError> {
        let score = match self.arm {
            InteractionArm::TransportedT3 => aggregate
                .score(self.twist, self.point)
                .map_err(GateError::Torsor)?,
            InteractionArm::SixChannelT4 => {
                let key = aggregate.factorized_key();
                self.twist.linear().dot(key.resultant)
                    + self.twist.angular().dot(key.origin_moment)
            }
        };
        if !score.is_finite() {
            return Err(GateError::NonFiniteScore);
        }
        Ok(score)
    }

    /// Scores are recomputed on every explicit observation. The gate
    /// stores only one bit and cannot claim an execution-cost saving.
    pub fn observe(
        &mut self,
        aggregate: &EventTorsorAggregate,
    ) -> Result<Observation, GateError> {
        let score = self.score(aggregate)?;
        let edge = if !self.active && score >= self.upper {
            Edge::Rising
        } else if self.active && score <= self.lower {
            Edge::Falling
        } else {
            Edge::None
        };
        match edge {
            Edge::Rising => self.active = true,
            Edge::Falling => self.active = false,
            Edge::None => {}
        }
        Ok(Observation {
            score,
            active: self.active,
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
        let scale = a.abs().max(b.abs()).max(1.0);
        assert!((a - b).abs() <= 256.0 * f64::EPSILON * scale, "{a} != {b}");
    }

    #[test]
    fn event_transitions_separate_t3_from_t4_without_new_memory_budget() {
        let twist = Twist3::new(v(1.0, 0.0, 0.0), v(0.0, 0.0, 1.0)).unwrap();
        let q = v(0.0, 1.0, 0.0);
        let mut t3 =
            HystereticInteraction::new(InteractionArm::TransportedT3, twist, q, 0.5, 1.5).unwrap();
        let mut t4 =
            HystereticInteraction::new(InteractionArm::SixChannelT4, twist, q, 0.5, 1.5).unwrap();
        let mut state = EventTorsorAggregate::new();
        let events = [
            (Some(1.0), Edge::Rising),
            (Some(0.3), Edge::None),
            (Some(0.2), Edge::Falling),
            (Some(1.0), Edge::Rising),
            (None, Edge::Falling),
        ];
        for (value, expected_edge) in events {
            let event = match value {
                Some(amplitude) => TorsorEvent::Upsert {
                    id: 7,
                    torsor: source(v(amplitude, 0.0, 0.0), Vec3::zero(), Vec3::zero()),
                    weight: 1.0,
                },
                None => TorsorEvent::Remove { id: 7 },
            };
            state.apply(event).unwrap();
            let t3_result = t3.observe(&state).unwrap();
            let t4_result = t4.observe(&state).unwrap();
            close(t3_result.score, 2.0 * value.unwrap_or(0.0));
            close(t4_result.score, value.unwrap_or(0.0));
            assert_eq!(t3_result.edge, expected_edge);
            assert_eq!(t4_result.edge, Edge::None);
        }
        assert!(!t3.active());
        assert!(!t4.active());
    }

    #[test]
    fn global_translation_preserves_t3_events_but_not_t4() {
        let twist = Twist3::new(Vec3::zero(), v(0.0, 0.0, 1.0)).unwrap();
        let r = v(0.0, 1.0, 0.0);
        let p = v(1.0, 0.0, 0.0);
        let d = v(1.0, 0.0, 0.0);
        let q = Vec3::zero();
        let mut original = EventTorsorAggregate::new();
        let mut translated = EventTorsorAggregate::new();
        let t = source(r, Vec3::zero(), p);
        original.apply(TorsorEvent::Upsert { id: 1, torsor: t, weight: 1.0 }).unwrap();
        translated.apply(TorsorEvent::Upsert {
            id: 1,
            torsor: source(r, Vec3::zero(), p.plus(d)),
            weight: 1.0,
        }).unwrap();
        let mut t3a =
            HystereticInteraction::new(InteractionArm::TransportedT3, twist, q, 0.5, 1.5).unwrap();
        let mut t3b = HystereticInteraction::new(
            InteractionArm::TransportedT3, twist, q.plus(d), 0.5, 1.5,
        ).unwrap();
        let left = t3a.observe(&original).unwrap();
        let right = t3b.observe(&translated).unwrap();
        close(left.score, right.score);
        close(left.score, direct_pairing(twist, t, q).unwrap());
        assert_eq!(left.edge, right.edge);
        assert_eq!(left.edge, Edge::None);
        let mut t4a =
            HystereticInteraction::new(InteractionArm::SixChannelT4, twist, q, 0.5, 1.5).unwrap();
        let mut t4b = HystereticInteraction::new(
            InteractionArm::SixChannelT4, twist, q.plus(d), 0.5, 1.5,
        ).unwrap();
        assert_eq!(t4a.observe(&original).unwrap().edge, Edge::None);
        assert_eq!(t4b.observe(&translated).unwrap().edge, Edge::Rising);
    }

    #[test]
    fn reduction_point_transport_preserves_both_arm_scores() {
        let t = source(v(1.0, 2.0, -3.0), v(-2.0, 5.0, 7.0), v(1.0, -4.0, 3.0));
        let moved = t.transport(v(6.0, 0.0, -1.0)).unwrap();
        let twist = Twist3::new(v(0.5, 1.0, 2.0), v(-1.0, 3.0, 0.25)).unwrap();
        let q = v(2.0, 3.0, -5.0);
        let mut a = EventTorsorAggregate::new();
        let mut b = EventTorsorAggregate::new();
        a.apply(TorsorEvent::Upsert { id: 0, torsor: t, weight: 1.0 }).unwrap();
        b.apply(TorsorEvent::Upsert { id: 0, torsor: moved, weight: 1.0 }).unwrap();
        for arm in [InteractionArm::TransportedT3, InteractionArm::SixChannelT4] {
            let gate = HystereticInteraction::new(arm, twist, q, -1.0, 1.0).unwrap();
            close(gate.score(&a).unwrap(), gate.score(&b).unwrap());
        }
    }

    #[test]
    fn errors_leave_state_unchanged() {
        let twist = Twist3::new(v(2.0, 0.0, 0.0), Vec3::zero()).unwrap();
        let invalid = HystereticInteraction::new(
            InteractionArm::SixChannelT4, twist, Vec3::zero(), 1.0, 1.0,
        );
        assert_eq!(invalid, Err(GateError::InvalidThresholds));
        assert!(HystereticInteraction::new(
            InteractionArm::SixChannelT4, twist, Vec3 { x: f64::NAN, y: 0.0, z: 0.0 }, 0.0, 1.0
        ).is_err());

        let mut gate =
            HystereticInteraction::new(InteractionArm::SixChannelT4, twist, Vec3::zero(), 0.25, 0.75)
                .unwrap();
        let mut state = EventTorsorAggregate::new();
        state.apply(TorsorEvent::Upsert {
            id: 0,
            torsor: source(v(1.0, 0.0, 0.0), Vec3::zero(), Vec3::zero()),
            weight: 1.0,
        }).unwrap();
        assert_eq!(gate.observe(&state).unwrap().edge, Edge::Rising);
        state.apply(TorsorEvent::Upsert {
            id: 0,
            torsor: source(v(f64::MAX, 0.0, 0.0), Vec3::zero(), Vec3::zero()),
            weight: 1.0,
        }).unwrap();
        assert_eq!(gate.observe(&state), Err(GateError::NonFiniteScore));
        assert!(gate.active());
    }
}
