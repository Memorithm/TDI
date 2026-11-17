//! Experimental cross-source reciprocity rule over torsor contributions.
//!
//! Instead of pairing a query against a single six-channel aggregate, this
//! mechanism observes *interactions among identified sources*. For two
//! weighted sources i,j with R resultant and C origin-reduced moment:
//!
//!   K(i,j) = R_i · C_j + R_j · C_i
//!   S      = sum_{i<j} K(i,j)
//!
//! K is the classical symmetric screw/Klein bilinear form. The form itself
//! is NOT novel. Its use as an event-driven source-coupling state is only a
//! development hypothesis, not an attention law, learning result, or
//! demonstrated energetic improvement.
//!
//! Unlike R_total · C_total, S removes all i=j self contributions. Hence
//! distinct source decompositions with the same (R_total,C_total) can have
//! different interaction scores. Uniform translation of all source reference
//! points (with their local moments fixed) leaves S unchanged.
//!
//! The update below retains incremental totals and a source-keyed map.
//! All arithmetic is checked before mutation. Independent O(K²) replay and
//! drift diagnostics are available; no physical execution claims are made.

use std::collections::BTreeMap;

use super::tdi22_event_aggregate::TorsorEvent;
use super::tdi22_event_gate::Edge;
use super::tdi22_torsor::{FactorizedTorsorKey, Torsor3, TorsorError, Vec3};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReciprocityError {
    InvalidWeight,
    MissingSource(u64),
    InvalidThresholds,
    Algebra(TorsorError),
    NonFiniteScore(&'static str),
}

impl From<TorsorError> for ReciprocityError {
    fn from(error: TorsorError) -> Self {
        Self::Algebra(error)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Source {
    r: Vec3,
    c: Vec3,
}

impl Source {
    fn weighted(torsor: Torsor3, weight: f64) -> Result<Self, ReciprocityError> {
        if !weight.is_finite() {
            return Err(ReciprocityError::InvalidWeight);
        }
        let key = torsor.factorized_key()?;
        Ok(Self {
            r: scale(key.resultant, weight)?,
            c: scale(key.origin_moment, weight)?,
        })
    }
}

fn scale(vector: Vec3, factor: f64) -> Result<Vec3, ReciprocityError> {
    Ok(Vec3::new(
        vector.x * factor,
        vector.y * factor,
        vector.z * factor,
    )?)
}

fn add(a: Vec3, b: Vec3) -> Result<Vec3, ReciprocityError> {
    Ok(Vec3::new(a.x + b.x, a.y + b.y, a.z + b.z)?)
}

fn sub(a: Vec3, b: Vec3) -> Result<Vec3, ReciprocityError> {
    Ok(Vec3::new(a.x - b.x, a.y - b.y, a.z - b.z)?)
}

fn finite(value: f64, stage: &'static str) -> Result<f64, ReciprocityError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(ReciprocityError::NonFiniteScore(stage))
    }
}

/// K(a,b) is symmetric and invariant under common rigid translation.
/// This is NOT an invariant under arbitrary independent translations.
fn reciprocal(a: Source, b: Source) -> Result<f64, ReciprocityError> {
    finite(a.r.dot(b.c) + b.r.dot(a.c), "cross_source_pairing")
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReciprocityAudit {
    pub cached: f64,
    pub independently_replayed: f64,
    /// Observed binary64 discrepancy, not a certified numerical error bound.
    pub absolute_difference: f64,
}

/// A keyed set of signed contributions with a cross-source scalar interaction.
/// A non-finite intermediate calculation rejects the whole event.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CrossSourceReciprocity {
    entries: BTreeMap<u64, Source>,
    total_r: Vec3,
    total_c: Vec3,
    score: f64,
}

impl CrossSourceReciprocity {
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

    #[must_use]
    pub fn factorized_key(&self) -> FactorizedTorsorKey {
        FactorizedTorsorKey {
            resultant: self.total_r,
            origin_moment: self.total_c,
        }
    }

    #[must_use]
    pub const fn interaction_score(&self) -> f64 {
        self.score
    }

    /// Matched six-channel aggregate-only control, R_total · C_total.
    /// Unlike the cross-source score, it cannot see source identity.
    pub fn aggregate_only_score(&self) -> Result<f64, ReciprocityError> {
        finite(self.total_r.dot(self.total_c), "aggregate_only")
    }

    /// Insert, replace or delete one contribution, failing closed on any
    /// invalid operation. The score identity is
    ///
    /// S(new) = S(old) - K(old_i, rest) + K(new_i, rest)
    ///
    /// with the two totals of "rest" computed from the unchanged entries.
    /// This is arithmetic on the candidate law, not a learning update.
    pub fn apply(&mut self, event: TorsorEvent) -> Result<(), ReciprocityError> {
        let (id, replacement) = match event {
            TorsorEvent::Upsert { id, torsor, weight } => {
                (id, Some(Source::weighted(torsor, weight)?))
            }
            TorsorEvent::Remove { id } => (id, None),
        };
        let existing = self.entries.get(&id).copied();
        if existing.is_none() && replacement.is_none() {
            return Err(ReciprocityError::MissingSource(id));
        }
        if existing == replacement {
            return Ok(());
        }

        let old = existing.unwrap_or_default();
        let new = replacement.unwrap_or_default();
        let rest_r = sub(self.total_r, old.r)?;
        let rest_c = sub(self.total_c, old.c)?;
        let rest = Source {
            r: rest_r,
            c: rest_c,
        };
        let after_removal = finite(
            self.score - reciprocal(old, rest)?,
            "remove_source_interactions",
        )?;
        let next_score = finite(
            after_removal + reciprocal(new, rest)?,
            "insert_source_interactions",
        )?;
        let next_r = add(rest_r, new.r)?;
        let next_c = add(rest_c, new.c)?;

        // No state field changes until all calculations have succeeded.
        match replacement {
            Some(source) => {
                self.entries.insert(id, source);
            }
            None => {
                self.entries.remove(&id);
            }
        }
        self.total_r = next_r;
        self.total_c = next_c;
        self.score = next_score;
        Ok(())
    }

    /// Independent direct pair enumeration, deliberately O(K²) in live
    /// source count, without relying on the incremental score recurrence.
    pub fn direct_replay(&self) -> Result<f64, ReciprocityError> {
        let sources: Vec<_> = self.entries.values().copied().collect();
        let mut replayed = 0.0_f64;
        for (index, left) in sources.iter().enumerate() {
            for right in &sources[index + 1..] {
                replayed = finite(replayed + reciprocal(*left, *right)?, "direct_replay")?;
            }
        }
        Ok(replayed)
    }

    pub fn audit(&self) -> Result<ReciprocityAudit, ReciprocityError> {
        let independently_replayed = self.direct_replay()?;
        Ok(ReciprocityAudit {
            cached: self.score,
            independently_replayed,
            absolute_difference: finite(
                (self.score - independently_replayed).abs(),
                "audit_difference",
            )?,
        })
    }

    /// Explicit numerical rebase from the independent oracle. This does not
    /// change any source identities or their stored six-channel totals.
    pub fn rebase(&mut self) -> Result<ReciprocityAudit, ReciprocityError> {
        let audit = self.audit()?;
        self.score = audit.independently_replayed;
        Ok(audit)
    }
}

/// One-bit state transition on the cross-source rule. A single source
/// always has interaction score zero, so it cannot trigger a positive gate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReciprocalGate {
    lower: f64,
    upper: f64,
    active: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReciprocalObservation {
    pub score: f64,
    pub active: bool,
    pub edge: Edge,
}

impl ReciprocalGate {
    pub fn new(lower: f64, upper: f64) -> Result<Self, ReciprocityError> {
        if !lower.is_finite() || !upper.is_finite() || lower >= upper {
            return Err(ReciprocityError::InvalidThresholds);
        }
        Ok(Self {
            lower,
            upper,
            active: false,
        })
    }

    #[must_use]
    pub const fn active(&self) -> bool {
        self.active
    }

    pub fn observe(
        &mut self,
        source_state: &CrossSourceReciprocity,
    ) -> Result<ReciprocalObservation, ReciprocityError> {
        let score = finite(source_state.interaction_score(), "gate")?;
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
        Ok(ReciprocalObservation {
            score,
            active: self.active,
            edge,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3::new(x, y, z).unwrap()
    }

    fn at_origin(r: Vec3, c: Vec3) -> Torsor3 {
        Torsor3::new(r, c, Vec3::zero()).unwrap()
    }

    fn close(a: f64, b: f64) {
        let scale = a.abs().max(b.abs()).max(1.0);
        assert!(
            (a - b).abs() <= 512.0 * f64::EPSILON * scale,
            "{a:?} != {b:?}"
        );
    }

    #[test]
    fn same_global_six_channels_can_encode_opposite_pair_interactions() {
        let mut negative = CrossSourceReciprocity::new();
        let mut positive = CrossSourceReciprocity::new();
        for (index, (r, c_neg, c_pos)) in [
            (v(1.0, 0.0, 0.0), v(1.0, 0.0, 0.0), v(-1.0, 0.0, 0.0)),
            (v(-1.0, 0.0, 0.0), v(-1.0, 0.0, 0.0), v(1.0, 0.0, 0.0)),
        ]
        .into_iter()
        .enumerate()
        {
            negative
                .apply(TorsorEvent::Upsert {
                    id: index as u64,
                    torsor: at_origin(r, c_neg),
                    weight: 1.0,
                })
                .unwrap();
            positive
                .apply(TorsorEvent::Upsert {
                    id: index as u64,
                    torsor: at_origin(r, c_pos),
                    weight: 1.0,
                })
                .unwrap();
        }
        assert_eq!(negative.factorized_key(), positive.factorized_key());
        assert_eq!(negative.factorized_key().resultant, Vec3::zero());
        assert_eq!(negative.factorized_key().origin_moment, Vec3::zero());
        close(negative.aggregate_only_score().unwrap(), 0.0);
        close(positive.aggregate_only_score().unwrap(), 0.0);
        close(negative.interaction_score(), -2.0);
        close(positive.interaction_score(), 2.0);
        let mut a = ReciprocalGate::new(-1.0, 1.0).unwrap();
        let mut b = ReciprocalGate::new(-1.0, 1.0).unwrap();
        assert_eq!(a.observe(&negative).unwrap().edge, Edge::None);
        assert_eq!(b.observe(&positive).unwrap().edge, Edge::Rising);
    }

    #[test]
    fn whole_configuration_translation_preserves_pairwise_coupling() {
        let r1 = v(1.0, 2.0, 0.0);
        let r2 = v(-2.0, 1.0, 1.0);
        let m1 = v(0.0, 1.0, 2.0);
        let m2 = v(2.0, -1.0, 0.0);
        let p1 = v(1.0, 0.0, -1.0);
        let p2 = v(0.0, 2.0, 1.0);
        let shift = v(3.0, -2.0, 5.0);
        let mut original = CrossSourceReciprocity::new();
        let mut shifted = CrossSourceReciprocity::new();
        for (id, resultant, moment, point) in [(3, r1, m1, p1), (7, r2, m2, p2)] {
            original
                .apply(TorsorEvent::Upsert {
                    id,
                    torsor: Torsor3::new(resultant, moment, point).unwrap(),
                    weight: 1.0,
                })
                .unwrap();
            shifted
                .apply(TorsorEvent::Upsert {
                    id,
                    torsor: Torsor3::new(resultant, moment, point.plus(shift)).unwrap(),
                    weight: 1.0,
                })
                .unwrap();
        }
        close(original.interaction_score(), shifted.interaction_score());
        close(
            original.direct_replay().unwrap(),
            shifted.direct_replay().unwrap(),
        );
        close(
            original.interaction_score(),
            original.direct_replay().unwrap(),
        );
    }

    #[test]
    fn source_reduction_point_changes_leave_cross_source_score_unchanged() {
        let a = Torsor3::new(v(1.0, 2.0, 3.0), v(2.0, -1.0, 4.0), v(1.0, 0.0, 1.0)).unwrap();
        let b = at_origin(v(-1.0, 2.0, 0.0), v(1.0, 3.0, -1.0));
        let mut original = CrossSourceReciprocity::new();
        let mut reduced = CrossSourceReciprocity::new();
        for (id, first) in [(1, a), (2, b)] {
            original
                .apply(TorsorEvent::Upsert {
                    id,
                    torsor: first,
                    weight: 0.5,
                })
                .unwrap();
            reduced
                .apply(TorsorEvent::Upsert {
                    id,
                    torsor: first.transport(v(4.0, -2.0, 1.0)).unwrap(),
                    weight: 0.5,
                })
                .unwrap();
        }
        close(original.interaction_score(), reduced.interaction_score());
    }

    #[test]
    fn replacements_deletions_and_signed_weights_match_direct_replay() {
        let a = at_origin(v(2.0, 1.0, 0.0), v(1.0, 0.0, 1.0));
        let b = at_origin(v(-1.0, 0.0, 1.0), v(0.0, 2.0, 1.0));
        let c = at_origin(v(1.0, -2.0, 0.0), v(2.0, 1.0, 0.0));
        let mut state = CrossSourceReciprocity::new();
        for event in [
            TorsorEvent::Upsert {
                id: 3,
                torsor: a,
                weight: 1.0,
            },
            TorsorEvent::Upsert {
                id: 9,
                torsor: b,
                weight: -2.0,
            },
            TorsorEvent::Upsert {
                id: 4,
                torsor: c,
                weight: 0.5,
            },
            TorsorEvent::Upsert {
                id: 9,
                torsor: a,
                weight: 2.0,
            },
            TorsorEvent::Remove { id: 3 },
            TorsorEvent::Remove { id: 4 },
            TorsorEvent::Remove { id: 9 },
        ] {
            state.apply(event).unwrap();
            close(state.interaction_score(), state.direct_replay().unwrap());
            close(state.audit().unwrap().absolute_difference, 0.0);
        }
        assert!(state.is_empty());
        close(state.interaction_score(), 0.0);
    }

    #[test]
    fn invalid_inputs_and_overflows_are_transactional() {
        let mut state = CrossSourceReciprocity::new();
        state
            .apply(TorsorEvent::Upsert {
                id: 1,
                torsor: at_origin(v(f64::MAX, 0.0, 0.0), Vec3::zero()),
                weight: 1.0,
            })
            .unwrap();
        let before = state.clone();
        assert_eq!(
            state.apply(TorsorEvent::Upsert {
                id: 2,
                torsor: at_origin(Vec3::zero(), v(f64::MAX, 0.0, 0.0)),
                weight: 1.0,
            }),
            Err(ReciprocityError::NonFiniteScore("cross_source_pairing"))
        );
        assert_eq!(state, before);
        assert_eq!(
            state.apply(TorsorEvent::Remove { id: 999 }),
            Err(ReciprocityError::MissingSource(999))
        );
        assert_eq!(state, before);
        assert_eq!(
            state.apply(TorsorEvent::Upsert {
                id: 1,
                torsor: at_origin(v(1.0, 0.0, 0.0), Vec3::zero()),
                weight: f64::NAN,
            }),
            Err(ReciprocityError::InvalidWeight)
        );
        assert_eq!(state, before);
    }

    #[test]
    fn one_source_has_no_pair_activation_and_identical_updates_are_idempotent() {
        let mut state = CrossSourceReciprocity::new();
        let event = TorsorEvent::Upsert {
            id: 3,
            torsor: at_origin(v(1.0, 0.0, 0.0), v(3.0, 0.0, 0.0)),
            weight: 1.0,
        };
        state.apply(event).unwrap();
        close(state.interaction_score(), 0.0);
        let snapshot = state.clone();
        state.apply(event).unwrap();
        assert_eq!(state, snapshot);
        let mut gate = ReciprocalGate::new(-0.1, 0.1).unwrap();
        assert_eq!(gate.observe(&state).unwrap().edge, Edge::None);
    }
}
