//! TDI-22 matched six-channel bilinear negative control (development only).
//!
//! The sparse torsor edge law is exactly x_i^T J x_j for six-vectors
//! x=(R_x,R_y,R_z,C_x,C_y,C_z), J=[[0,I_3],[I_3,0]].
//! Thus an ordinary fixed bilinear edge layer has identical information.
//! The code compares *two independent implementations* of this identity,
//! with identical topology, thresholds, source events, and one bit per edge.
//!
//! An external binary task label enters only after predictions. Equal scores
//! and edge traces are an expected negative scientific result, NOT evidence
//! of a new law, learned task ability, runtime speedup, or energy benefit.
//!
//! Paired updates stage full cloned state to fail closed; O(N+E) copying per
//! event dominates this reference comparator. There is no fast-path claim.

use std::collections::BTreeMap;

use super::tdi22_event_aggregate::TorsorEvent;
use super::tdi22_event_gate::Edge;
use super::tdi22_sparse_reciprocal_graph::{
    GraphError, PairState, PairUpdate, SparseReciprocalEdgeGraph,
};
use super::tdi22_torsor::Torsor3;

const MAX_PAIRED_OBSERVATIONS: u64 = 4_096;

/// Control scores use only ordinary six-dimensional floating-point inputs.
type SixChannels = [f64; 6];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlError {
    Graph(GraphError),
    StaleSequence { previous: u64, received: u64 },
    EventLimit,
    CounterOverflow,
    TraceMismatch,
}

impl From<GraphError> for ControlError {
    fn from(error: GraphError) -> Self {
        Self::Graph(error)
    }
}

/// The control stores one six-channel source record per key and a score plus
/// active bit per declared edge, matching the candidate's logical state.
/// Its O(E) edge scan is deliberately not a sparsity performance control.
#[derive(Clone, Debug, PartialEq)]
struct OrdinaryBilinearGraph {
    sources: BTreeMap<u64, SixChannels>,
    edges: BTreeMap<(u64, u64), PairState>,
    lower: f64,
    upper: f64,
}

impl OrdinaryBilinearGraph {
    fn new(wiring: &[(u64, u64)], lower: f64, upper: f64) -> Self {
        Self {
            sources: BTreeMap::new(),
            edges: wiring
                .iter()
                .map(|&(a, b)| ((a.min(b), a.max(b)), PairState::default()))
                .collect(),
            lower,
            upper,
        }
    }

    fn channels(torsor: Torsor3, weight: f64) -> Result<SixChannels, GraphError> {
        if !weight.is_finite() {
            return Err(GraphError::InvalidWeight);
        }
        let key = torsor.factorized_key().map_err(GraphError::Algebra)?;
        let values = [
            key.resultant.x * weight,
            key.resultant.y * weight,
            key.resultant.z * weight,
            key.origin_moment.x * weight,
            key.origin_moment.y * weight,
            key.origin_moment.z * weight,
        ];
        if values.iter().all(|x| x.is_finite()) {
            Ok(values)
        } else {
            Err(GraphError::NonFinitePair)
        }
    }

    /// Explicit six-channel matrix-J control, independent of the torsor
    /// graph's pairwise scoring method. Float operation order is chosen to
    /// permit exact bit-for-bit software trace comparisons on finite inputs.
    fn bilinear(a: SixChannels, b: SixChannels) -> Result<f64, GraphError> {
        let forward = (a[0] * b[3] + a[1] * b[4]) + a[2] * b[5];
        let backward = (b[0] * a[3] + b[1] * a[4]) + b[2] * a[5];
        let result = forward + backward;
        if result.is_finite() {
            Ok(result)
        } else {
            Err(GraphError::NonFinitePair)
        }
    }

    fn apply(&mut self, event: TorsorEvent) -> Result<Vec<PairUpdate>, GraphError> {
        let (id, new) = match event {
            TorsorEvent::Upsert { id, torsor, weight } => {
                (id, Some(Self::channels(torsor, weight)?))
            }
            TorsorEvent::Remove { id } => (id, None),
        };
        if new.is_none() && !self.sources.contains_key(&id) {
            return Err(GraphError::MissingSource(id));
        }
        if self.sources.get(&id).copied() == new {
            return Ok(Vec::new());
        }
        let mut staged = Vec::new();
        for (&pair, &previous) in &self.edges {
            if pair.0 != id && pair.1 != id {
                continue;
            }
            let neighbor = if pair.0 == id { pair.1 } else { pair.0 };
            let score = match (new, self.sources.get(&neighbor).copied()) {
                (Some(updated), Some(other)) => Self::bilinear(updated, other)?,
                _ => 0.0,
            };
            let edge = if !previous.active && score >= self.upper {
                Edge::Rising
            } else if previous.active && score <= self.lower {
                Edge::Falling
            } else {
                Edge::None
            };
            let active = match edge {
                Edge::Rising => true,
                Edge::Falling => false,
                Edge::None => previous.active,
            };
            staged.push((
                pair,
                PairState { score, active },
                PairUpdate {
                    source_a: pair.0,
                    source_b: pair.1,
                    score,
                    active,
                    edge,
                },
            ));
        }
        // Transactional control update; neither source nor edges change
        // until every touched edge has a valid score.
        match new {
            Some(channels) => {
                self.sources.insert(id, channels);
            }
            None => {
                self.sources.remove(&id);
            }
        }
        for (pair, state, _) in &staged {
            *self.edges.get_mut(pair).expect("wired edge") = *state;
        }
        Ok(staged.into_iter().map(|(_, _, update)| update).collect())
    }

    fn active_edges(&self) -> Vec<(u64, u64)> {
        self.edges
            .iter()
            .filter_map(|(&pair, &state)| state.active.then_some(pair))
            .collect()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BinaryMetrics {
    pub true_positive: u64,
    pub false_positive: u64,
    pub true_negative: u64,
    pub false_negative: u64,
}

impl BinaryMetrics {
    fn record(&mut self, predicted: bool, target: bool) -> Result<(), ControlError> {
        let counter = match (predicted, target) {
            (true, true) => &mut self.true_positive,
            (true, false) => &mut self.false_positive,
            (false, false) => &mut self.true_negative,
            (false, true) => &mut self.false_negative,
        };
        *counter = counter.checked_add(1).ok_or(ControlError::CounterOverflow)?;
        Ok(())
    }

    #[must_use]
    pub const fn total(self) -> u64 {
        self.true_positive + self.false_positive + self.true_negative + self.false_negative
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MatchedEdgeStep {
    pub sequence: u64,
    pub candidate_updates: Vec<PairUpdate>,
    pub bilinear_updates: Vec<PairUpdate>,
    pub active_edges: Vec<(u64, u64)>,
    pub any_active: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MatchedBilinearProtocol {
    candidate: SparseReciprocalEdgeGraph,
    control: OrdinaryBilinearGraph,
    candidate_metrics: BinaryMetrics,
    control_metrics: BinaryMetrics,
    last_sequence: Option<u64>,
    observations: u64,
}

impl MatchedBilinearProtocol {
    pub fn new(wiring: &[(u64, u64)], lower: f64, upper: f64) -> Result<Self, GraphError> {
        let candidate = SparseReciprocalEdgeGraph::new(wiring, lower, upper)?;
        Ok(Self {
            candidate,
            control: OrdinaryBilinearGraph::new(wiring, lower, upper),
            candidate_metrics: BinaryMetrics::default(),
            control_metrics: BinaryMetrics::default(),
            last_sequence: None,
            observations: 0,
        })
    }

    #[must_use]
    pub const fn observation_count(&self) -> u64 {
        self.observations
    }

    #[must_use]
    pub const fn metrics(&self) -> (BinaryMetrics, BinaryMetrics) {
        (self.candidate_metrics, self.control_metrics)
    }

    /// Score an externally defined binary target *after* both arms have
    /// already produced their event transitions. The target is never
    /// consulted by either edge mechanism.
    pub fn push(
        &mut self,
        sequence: u64,
        event: TorsorEvent,
        independent_target: bool,
    ) -> Result<MatchedEdgeStep, ControlError> {
        if self.observations >= MAX_PAIRED_OBSERVATIONS {
            return Err(ControlError::EventLimit);
        }
        if let Some(previous) = self.last_sequence {
            if sequence <= previous {
                return Err(ControlError::StaleSequence {
                    previous,
                    received: sequence,
                });
            }
        }

        // Fail closed across both independently implemented graph arms.
        // Cloning here makes the complete paired reference O(N+E), not the
        // O(degree) candidate graph fast path being tested semantically.
        let mut staged = self.clone();
        let candidate_updates = staged.candidate.apply(event)?;
        let bilinear_updates = staged.control.apply(event)?;

        if candidate_updates != bilinear_updates {
            return Err(ControlError::TraceMismatch);
        }
        let candidates = staged.candidate.active_edges();
        let controls = staged.control.active_edges();
        if candidates != controls {
            return Err(ControlError::TraceMismatch);
        }

        let predicted = !candidates.is_empty();
        staged.candidate_metrics.record(predicted, independent_target)?;
        staged.control_metrics.record(!controls.is_empty(), independent_target)?;
        staged.observations = staged
            .observations
            .checked_add(1)
            .ok_or(ControlError::CounterOverflow)?;
        staged.last_sequence = Some(sequence);
        *self = staged;

        Ok(MatchedEdgeStep {
            sequence,
            candidate_updates,
            bilinear_updates,
            active_edges: candidates,
            any_active: predicted,
        })
    }

    /// O(E) independent replay check of both score caches.
    pub fn verify(&self) -> Result<bool, GraphError> {
        if !self.candidate.verify_all_edges()? {
            return Ok(false);
        }
        for (&(a, b), &stored) in &self.control.edges {
            let reference = match (self.control.sources.get(&a), self.control.sources.get(&b)) {
                (Some(lhs), Some(rhs)) => OrdinaryBilinearGraph::bilinear(*lhs, *rhs)?,
                _ => 0.0,
            };
            if reference != stored.score {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::tdi22_torsor::Vec3;

    fn v(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3::new(x, y, z).unwrap()
    }

    fn upsert(id: u64, r: Vec3, c: Vec3) -> TorsorEvent {
        TorsorEvent::Upsert {
            id,
            torsor: Torsor3::new(r, c, Vec3::zero()).unwrap(),
            weight: 1.0,
        }
    }

    fn fixture(labels: &[bool]) -> (MatchedBilinearProtocol, Vec<MatchedEdgeStep>) {
        let mut paired = MatchedBilinearProtocol::new(&[(1, 2), (1, 3), (3, 4)], 0.25, 1.0).unwrap();
        let events = [
            upsert(1, v(1.0, 0.0, 0.0), Vec3::zero()),
            upsert(2, Vec3::zero(), v(2.0, 0.0, 0.0)),
            upsert(3, Vec3::zero(), v(-2.0, 0.0, 0.0)),
            upsert(4, v(-1.0, 0.0, 0.0), Vec3::zero()),
            upsert(2, Vec3::zero(), v(0.1, 0.0, 0.0)),
            TorsorEvent::Remove { id: 1 },
        ];
        let results: Vec<_> = events
            .into_iter()
            .zip(labels)
            .enumerate()
            .map(|(i, (event, &target))| paired.push(i as u64, event, target).unwrap())
            .collect();
        assert!(paired.verify().unwrap());
        (paired, results)
    }

    #[test]
    fn fixed_six_channel_j_control_matches_local_edges_exactly() {
        let (paired, trace) = fixture(&[false, true, true, false, true, false]);
        for step in trace {
            assert_eq!(step.candidate_updates, step.bilinear_updates);
        }
        assert_eq!(paired.observation_count(), 6);
        let (candidate, ordinary) = paired.metrics();
        assert_eq!(candidate, ordinary);
        assert_eq!(candidate.total(), 6);
    }

    #[test]
    fn cancellation_is_not_a_torsor_specific_advantage() {
        let (_, trace) = fixture(&[false, true, false, true, false, false]);
        assert_eq!(trace[2].active_edges, vec![(1, 2)]);
        assert_eq!(trace[2].candidate_updates, trace[2].bilinear_updates);
    }

    #[test]
    fn changing_only_task_labels_never_changes_any_prediction() {
        let (a, trace_a) = fixture(&[false, false, false, false, false, false]);
        let (b, trace_b) = fixture(&[true, true, true, true, true, true]);
        assert_eq!(trace_a, trace_b);
        assert_ne!(a.metrics(), b.metrics());
        assert_eq!(a.candidate, b.candidate);
        assert_eq!(a.control, b.control);
    }

    #[test]
    fn same_connection_budget_preserves_unaffected_edge_state() {
        let (_, trace) = fixture(&[false, false, true, false, false, false]);
        assert_eq!(trace[4].candidate_updates.len(), 1);
        assert_eq!(trace[4].candidate_updates[0].source_a, 1);
        assert_eq!(trace[4].candidate_updates[0].source_b, 2);
        assert_eq!(trace[4].candidate_updates, trace[4].bilinear_updates);
    }

    #[test]
    fn rejected_pair_overflow_and_duplicate_sequence_rollback_both_arms() {
        let mut paired = MatchedBilinearProtocol::new(&[(1, 2)], 0.0, 1.0).unwrap();
        paired
            .push(0, upsert(1, v(f64::MAX, 0.0, 0.0), Vec3::zero()), false)
            .unwrap();
        let before = paired.clone();
        assert_eq!(
            paired.push(
                1,
                upsert(2, Vec3::zero(), v(f64::MAX, 0.0, 0.0)),
                true
            ),
            Err(ControlError::Graph(GraphError::NonFinitePair))
        );
        assert_eq!(paired, before);
        assert_eq!(
            paired.push(0, TorsorEvent::Remove { id: 1 }, false),
            Err(ControlError::StaleSequence {
                previous: 0,
                received: 0
            })
        );
        assert_eq!(paired, before);
        assert_eq!(
            paired.push(1, TorsorEvent::Remove { id: 999 }, false),
            Err(ControlError::Graph(GraphError::MissingSource(999)))
        );
        assert_eq!(paired, before);
    }

    #[test]
    fn zero_signal_alternating_labels_cannot_create_a_positive_prediction() {
        let mut paired = MatchedBilinearProtocol::new(&[(1, 2)], 0.0, 1.0).unwrap();
        for i in 0..32 {
            let event = upsert(
                if i % 2 == 0 { 1 } else { 2 },
                Vec3::zero(),
                Vec3::zero(),
            );
            let outcome = paired.push(i, event, i % 2 == 0).unwrap();
            assert!(!outcome.any_active);
            assert!(outcome.active_edges.is_empty());
        }
        let (a, b) = paired.metrics();
        assert_eq!(a, b);
        assert_eq!(a.true_positive, 0);
        assert_eq!(a.false_positive, 0);
        assert_eq!(a.false_negative, 16);
        assert_eq!(a.true_negative, 16);
    }

    #[test]
    fn known_bilinear_identity_is_not_a_candidate_only_result() {
        let a = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let b = [-2.0, 1.0, 3.0, 5.0, -1.0, 2.0];
        let expected =
            v(a[0], a[1], a[2]).dot(v(b[3], b[4], b[5]))
            + v(b[0], b[1], b[2]).dot(v(a[3], a[4], a[5]));
        assert_eq!(OrdinaryBilinearGraph::bilinear(a, b).unwrap(), expected);
    }
}
