//! Experimental event-driven torsor aggregation, reusing frozen TDI-22 algebra.
//!
//! The carrier is an *unnormalized*, signed weighted sum of torsors. It is not
//! softmax attention, a trained model, an energy measurement or a novelty claim.
//! Only a changed source updates the stored (R, C) totals; all arithmetic is
//! checked before committing the event. Scores use the frozen TDI-22 pairing.

use std::collections::BTreeMap;

use super::tdi22_torsor::{FactorizedTorsorKey, Torsor3, TorsorError, Twist3, Vec3};

/// A caller-identified contribution. Each identifier has at most one live value.
#[derive(Clone, Copy, Debug)]
pub enum TorsorEvent {
    Upsert {
        id: u64,
        torsor: Torsor3,
        weight: f64,
    },
    Remove {
        id: u64,
    },
}

/// Rejections never partially modify the accumulated state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventError {
    InvalidWeight,
    MissingSource(u64),
    Algebra(TorsorError),
}

impl From<TorsorError> for EventError {
    fn from(error: TorsorError) -> Self {
        Self::Algebra(error)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Contribution {
    resultant: Vec3,
    origin_moment: Vec3,
}

impl Contribution {
    fn new(torsor: Torsor3, weight: f64) -> Result<Self, EventError> {
        if !weight.is_finite() {
            return Err(EventError::InvalidWeight);
        }
        let key = torsor.factorized_key()?;
        Ok(Self {
            resultant: checked_scale(key.resultant, weight)?,
            origin_moment: checked_scale(key.origin_moment, weight)?,
        })
    }
}

fn checked_scale(value: Vec3, weight: f64) -> Result<Vec3, EventError> {
    Vec3::new(value.x * weight, value.y * weight, value.z * weight).map_err(Into::into)
}

fn checked_replace(total: Vec3, old: Vec3, new: Vec3) -> Result<Vec3, EventError> {
    Vec3::new(
        (total.x - old.x) + new.x,
        (total.y - old.y) + new.y,
        (total.z - old.z) + new.z,
    )
    .map_err(Into::into)
}

/// Observed binary64 disagreement between an incremental state and a
/// deterministic compensated full replay of its live contributions.
///
/// These are numerical diagnostics, NOT certified upper error bounds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RebaseAudit {
    pub before: FactorizedTorsorKey,
    pub recomputed: FactorizedTorsorKey,
    pub resultant_max_abs: f64,
    pub origin_moment_max_abs: f64,
}

/// Neumaier-compensated scalar summation, with explicit finite checks.
///
/// Reference-only O(K) full replay. This is not used on the normal event path.
#[derive(Clone, Copy, Debug, Default)]
struct Compensated {
    sum: f64,
    correction: f64,
}

impl Compensated {
    fn add(&mut self, value: f64) -> Result<(), EventError> {
        let next = self.sum + value;
        if !next.is_finite() {
            return Err(EventError::Algebra(TorsorError::NonFiniteScalar {
                field: "rebase_sum",
            }));
        }
        let lost = if self.sum.abs() >= value.abs() {
            (self.sum - next) + value
        } else {
            (value - next) + self.sum
        };
        let correction = self.correction + lost;
        if !lost.is_finite() || !correction.is_finite() {
            return Err(EventError::Algebra(TorsorError::NonFiniteScalar {
                field: "rebase_correction",
            }));
        }
        self.sum = next;
        self.correction = correction;
        Ok(())
    }

    fn finish(self) -> Result<f64, EventError> {
        let result = self.sum + self.correction;
        if !result.is_finite() {
            return Err(EventError::Algebra(TorsorError::NonFiniteScalar {
                field: "rebase_result",
            }));
        }
        Ok(result)
    }
}

fn max_component_disagreement(a: Vec3, b: Vec3) -> Result<f64, EventError> {
    let values = [(a.x, b.x), (a.y, b.y), (a.z, b.z)];
    let mut maximum = 0.0_f64;
    for (old, new) in values {
        let delta = (old - new).abs();
        if !delta.is_finite() {
            return Err(EventError::Algebra(TorsorError::NonFiniteScalar {
                field: "rebase_disagreement",
            }));
        }
        maximum = maximum.max(delta);
    }
    Ok(maximum)
}

/// An event-driven cache of the global resultant and origin-reduced moment.
///
/// Each accepted event performs constant-size vector arithmetic and one
/// BTreeMap lookup/mutation (O(log number_of_live_sources)). Query scores
/// depend on the six cached components, not on an O(N) scan.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EventTorsorAggregate {
    entries: BTreeMap<u64, Contribution>,
    resultant: Vec3,
    origin_moment: Vec3,
}

impl EventTorsorAggregate {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Atomic insert/replace/delete; a failing event leaves all fields intact.
    pub fn apply(&mut self, event: TorsorEvent) -> Result<(), EventError> {
        let (id, replacement) = match event {
            TorsorEvent::Upsert { id, torsor, weight } => {
                (id, Some(Contribution::new(torsor, weight)?))
            }
            TorsorEvent::Remove { id } => (id, None),
        };
        let old = self.entries.get(&id).copied();
        if old.is_none() && replacement.is_none() {
            return Err(EventError::MissingSource(id));
        }
        if old == replacement {
            return Ok(());
        }
        let old_r = old.map_or(Vec3::zero(), |item| item.resultant);
        let old_c = old.map_or(Vec3::zero(), |item| item.origin_moment);
        let new_r = replacement.map_or(Vec3::zero(), |item| item.resultant);
        let new_c = replacement.map_or(Vec3::zero(), |item| item.origin_moment);
        let next_r = checked_replace(self.resultant, old_r, new_r)?;
        let next_c = checked_replace(self.origin_moment, old_c, new_c)?;

        // Publish only after both channels have passed all numerical checks.
        if let Some(item) = replacement {
            self.entries.insert(id, item);
        } else {
            self.entries.remove(&id);
        }
        self.resultant = next_r;
        self.origin_moment = next_c;
        Ok(())
    }

    #[must_use]
    pub fn factorized_key(&self) -> FactorizedTorsorKey {
        FactorizedTorsorKey {
            resultant: self.resultant,
            origin_moment: self.origin_moment,
        }
    }

    /// Deterministic full replay in ascending source-id order, using six
    /// independent Neumaier accumulators. No state is changed.
    ///
    /// This O(K) operation can be compared against the O(log K) event path,
    /// but is not itself evidence of reduced physical cost or model utility.
    fn compensated_key(&self) -> Result<FactorizedTorsorKey, EventError> {
        let mut accumulators = [Compensated::default(); 6];
        for contribution in self.entries.values() {
            let channels = [
                contribution.resultant.x,
                contribution.resultant.y,
                contribution.resultant.z,
                contribution.origin_moment.x,
                contribution.origin_moment.y,
                contribution.origin_moment.z,
            ];
            for (accumulator, value) in accumulators.iter_mut().zip(channels) {
                accumulator.add(value)?;
            }
        }
        let mut channel = [0.0_f64; 6];
        for (value, accumulator) in channel.iter_mut().zip(accumulators) {
            *value = accumulator.finish()?;
        }
        Ok(FactorizedTorsorKey {
            resultant: Vec3::new(channel[0], channel[1], channel[2])?,
            origin_moment: Vec3::new(channel[3], channel[4], channel[5])?,
        })
    }

    /// Diagnose accumulated rounding drift without mutating stored totals.
    /// Disagreement is an observation, not a bound on true mathematical error.
    pub fn audit_rebase(&self) -> Result<RebaseAudit, EventError> {
        let before = self.factorized_key();
        let recomputed = self.compensated_key()?;
        Ok(RebaseAudit {
            resultant_max_abs: max_component_disagreement(before.resultant, recomputed.resultant)?,
            origin_moment_max_abs: max_component_disagreement(
                before.origin_moment,
                recomputed.origin_moment,
            )?,
            before,
            recomputed,
        })
    }

    /// Explicitly replace incremental totals by a compensated replay.
    /// All numerical checks finish before either cached channel changes.
    /// Source identities and contributions are preserved.
    pub fn rebase(&mut self) -> Result<RebaseAudit, EventError> {
        let report = self.audit_rebase()?;
        self.resultant = report.recomputed.resultant;
        self.origin_moment = report.recomputed.origin_moment;
        Ok(report)
    }

    /// Recover the aggregate moment at any reduction point Q: M(Q) = C-Q×R.
    pub fn reduced_at(&self, point: Vec3) -> Result<Torsor3, EventError> {
        let moment = self.origin_moment.minus(point.cross(self.resultant));
        Ok(Torsor3::new(self.resultant, moment, point)?)
    }

    /// Evaluate an interaction with a twist without rescanning contributions.
    pub fn score(&self, twist: Twist3, point: Vec3) -> Result<f64, EventError> {
        Ok(twist.factorized_at(point)?.pair(self.factorized_key())?)
    }
}

#[cfg(test)]
mod tests {
    use super::super::tdi22_torsor::direct_pairing;
    use super::*;

    fn v(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3::new(x, y, z).unwrap()
    }

    fn torsor(r: Vec3, m: Vec3, p: Vec3) -> Torsor3 {
        Torsor3::new(r, m, p).unwrap()
    }

    fn close(a: f64, b: f64) {
        let scale = 1.0_f64.max(a.abs()).max(b.abs());
        assert!((a - b).abs() <= 256.0 * f64::EPSILON * scale, "{a} != {b}");
    }

    fn close_vec(a: Vec3, b: Vec3) {
        close(a.x, b.x);
        close(a.y, b.y);
        close(a.z, b.z);
    }

    #[test]
    fn sparse_event_updates_match_independent_full_recomputation() {
        let a = torsor(v(2.0, 1.0, -3.0), v(4.0, -1.0, 2.0), v(1.0, 2.0, 3.0));
        let b = torsor(v(-3.0, 4.0, 1.0), v(2.0, 3.0, 5.0), v(0.0, -1.0, 2.0));
        let c = torsor(v(5.0, -2.0, 1.0), v(-3.0, 2.0, 6.0), v(2.0, 0.0, 1.0));
        let twist = Twist3::new(v(3.0, -2.0, 0.5), v(-1.0, 4.0, 2.0)).unwrap();
        let q = v(2.5, -3.0, 1.0);
        let mut state = EventTorsorAggregate::new();
        let mut reference: BTreeMap<u64, (Torsor3, f64)> = BTreeMap::new();
        let stream = [
            TorsorEvent::Upsert {
                id: 9,
                torsor: a,
                weight: 2.0,
            },
            TorsorEvent::Upsert {
                id: 3,
                torsor: b,
                weight: -0.5,
            },
            TorsorEvent::Upsert {
                id: 9,
                torsor: c,
                weight: 1.25,
            },
            TorsorEvent::Remove { id: 3 },
            TorsorEvent::Remove { id: 9 },
        ];
        for event in stream {
            state.apply(event).unwrap();
            match event {
                TorsorEvent::Upsert { id, torsor, weight } => {
                    reference.insert(id, (torsor, weight));
                }
                TorsorEvent::Remove { id } => {
                    reference.remove(&id);
                }
            }
            let expected: f64 = reference
                .values()
                .map(|(item, weight)| weight * direct_pairing(twist, *item, q).unwrap())
                .sum();
            close(state.score(twist, q).unwrap(), expected);
            assert_eq!(state.len(), reference.len());
        }
        assert!(state.is_empty());
        assert_eq!(state.factorized_key().resultant, Vec3::zero());
        assert_eq!(state.factorized_key().origin_moment, Vec3::zero());
    }

    #[test]
    fn result_transports_and_preserves_automoment() {
        let a = torsor(v(2.0, 3.0, 1.0), v(1.0, -2.0, 4.0), v(1.0, 0.0, 2.0));
        let b = torsor(v(-1.0, 2.0, 3.0), v(3.0, 2.0, -1.0), v(-2.0, 1.0, 0.0));
        let mut state = EventTorsorAggregate::new();
        state
            .apply(TorsorEvent::Upsert {
                id: 0,
                torsor: a,
                weight: 1.0,
            })
            .unwrap();
        state
            .apply(TorsorEvent::Upsert {
                id: 1,
                torsor: b,
                weight: 2.0,
            })
            .unwrap();
        let p = state.reduced_at(v(4.0, -3.0, 2.0)).unwrap();
        let q = state.reduced_at(v(-1.0, 3.0, -2.0)).unwrap();
        close_vec(p.resultant(), q.resultant());
        close_vec(p.transport(q.reference()).unwrap().moment(), q.moment());
        close(p.scalar_invariant().unwrap(), q.scalar_invariant().unwrap());
        close_vec(
            p.origin_moment().unwrap(),
            state.factorized_key().origin_moment,
        );
    }

    #[test]
    fn origin_translation_preserves_pairing() {
        let r = v(2.0, -3.0, 5.0);
        let m = v(1.0, 7.0, -2.0);
        let p = v(4.0, 2.0, -1.0);
        let d = v(15.0, -8.0, 11.0);
        let q = v(-5.0, 3.0, 2.0);
        let twist = Twist3::new(v(1.0, 2.0, 3.0), v(2.0, -1.0, 4.0)).unwrap();
        let mut original = EventTorsorAggregate::new();
        let mut translated = EventTorsorAggregate::new();
        original
            .apply(TorsorEvent::Upsert {
                id: 0,
                torsor: torsor(r, m, p),
                weight: 1.5,
            })
            .unwrap();
        translated
            .apply(TorsorEvent::Upsert {
                id: 0,
                torsor: torsor(r, m, p.plus(d)),
                weight: 1.5,
            })
            .unwrap();
        close(
            original.score(twist, q).unwrap(),
            translated.score(twist, q.plus(d)).unwrap(),
        );
    }

    #[test]
    fn rejected_inputs_and_overflow_leave_state_unchanged() {
        let t = torsor(v(1.0, 0.0, 0.0), v(0.0, 1.0, 0.0), Vec3::zero());
        let mut state = EventTorsorAggregate::new();
        state
            .apply(TorsorEvent::Upsert {
                id: 2,
                torsor: t,
                weight: 1.0,
            })
            .unwrap();
        let before = state.clone();
        assert_eq!(
            state.apply(TorsorEvent::Upsert {
                id: 2,
                torsor: t,
                weight: f64::NAN
            }),
            Err(EventError::InvalidWeight)
        );
        assert_eq!(state, before);
        assert_eq!(
            state.apply(TorsorEvent::Remove { id: 999 }),
            Err(EventError::MissingSource(999))
        );
        assert_eq!(state, before);
        let huge = torsor(v(f64::MAX, 0.0, 0.0), Vec3::zero(), Vec3::zero());
        assert!(
            state
                .apply(TorsorEvent::Upsert {
                    id: 3,
                    torsor: huge,
                    weight: 2.0
                })
                .is_err()
        );
        assert_eq!(state, before);
        assert!(
            state
                .score(
                    Twist3::new(v(1.0, 0.0, 0.0), v(0.0, f64::MAX, 0.0)).unwrap(),
                    v(f64::MAX, 0.0, 0.0)
                )
                .is_err()
        );
        assert_eq!(state, before);
    }

    #[test]
    fn identical_upsert_is_numerically_idempotent() {
        let t = torsor(v(1.0, 2.0, 3.0), v(2.0, 3.0, 4.0), v(2.0, -1.0, 0.0));
        let event = TorsorEvent::Upsert {
            id: 5,
            torsor: t,
            weight: 1.0,
        };
        let mut state = EventTorsorAggregate::new();
        state.apply(event).unwrap();
        let before = state.clone();
        state.apply(event).unwrap();
        assert_eq!(state, before);
    }

    #[test]
    fn compensated_rebase_recovers_small_residual_after_large_cancellation() {
        let mut aggregate = EventTorsorAggregate::new();
        for (id, value) in [(0, 1.0e16), (1, 1.0), (2, -1.0e16)] {
            aggregate
                .apply(TorsorEvent::Upsert {
                    id,
                    torsor: torsor(v(value, 0.0, 0.0), Vec3::zero(), Vec3::zero()),
                    weight: 1.0,
                })
                .unwrap();
        }
        assert_eq!(aggregate.factorized_key().resultant.x, 0.0);
        let frozen_state = aggregate.clone();
        let report = aggregate.audit_rebase().unwrap();
        assert_eq!(aggregate, frozen_state, "audit must not mutate state");
        assert_eq!(report.resultant_max_abs, 1.0);
        assert_eq!(report.recomputed.resultant.x, 1.0);
        assert_eq!(report.origin_moment_max_abs, 0.0);
        let executed = aggregate.rebase().unwrap();
        assert_eq!(executed, report);
        assert_eq!(aggregate.factorized_key().resultant.x, 1.0);
        let again = aggregate.rebase().unwrap();
        assert_eq!(again.resultant_max_abs, 0.0);
        assert_eq!(aggregate.len(), 3);
    }

    #[test]
    fn rebase_preserves_reduction_point_and_dual_pairing() {
        let mut aggregate = EventTorsorAggregate::new();
        let a = torsor(v(1.0e16, 2.0, -3.0), v(4.0, 5.0, -6.0), v(1.0, -2.0, 3.0));
        let b = torsor(v(1.0, -4.0, 2.0), v(-2.0, 3.0, 1.0), v(0.0, 2.0, -1.0));
        let c = torsor(v(-1.0e16, 5.0, 1.0), v(6.0, -2.0, 0.0), v(-1.0, 0.0, 2.0));
        for (id, torsor) in [(0, a), (1, b), (2, c)] {
            aggregate
                .apply(TorsorEvent::Upsert {
                    id,
                    torsor,
                    weight: 1.0,
                })
                .unwrap();
        }
        let q = v(0.5, 1.0, -2.0);
        let twist = Twist3::new(v(2.0, -1.0, 3.0), v(0.0, 1.0, 0.5)).unwrap();
        let audit = aggregate.rebase().unwrap();
        let reduced = aggregate.reduced_at(q).unwrap();
        close_vec(
            reduced.origin_moment().unwrap(),
            audit.recomputed.origin_moment,
        );
        close(
            aggregate.score(twist, q).unwrap(),
            direct_pairing(twist, reduced, q).unwrap(),
        );
        assert_eq!(aggregate.audit_rebase().unwrap().resultant_max_abs, 0.0);
    }

    #[test]
    fn empty_rebase_is_noop() {
        let mut aggregate = EventTorsorAggregate::new();
        let audit = aggregate.rebase().unwrap();
        assert_eq!(audit.resultant_max_abs, 0.0);
        assert_eq!(audit.origin_moment_max_abs, 0.0);
        assert!(aggregate.is_empty());
    }
}
