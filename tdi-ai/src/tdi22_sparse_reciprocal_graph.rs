//! Development-only sparse, event-driven reciprocal torsor edge graph.
//!
//! This is an alternative *interaction topology*: only declared source pairs
//! can exchange an event. Updating one source evaluates only its neighboring
//! edges; unrelated edges retain their state without being rescored.
//!
//! Each edge uses the classical symmetric screw/Klein bilinear form
//! K(i,j) = R_i·C_j + R_j·C_i, where C is origin-reduced moment.
//! The formula itself is known mathematics. This module claims neither
//! novelty, useful cognition, task advantage, nor measured energy savings.
//!
//! All candidate transitions are staged before source/edge mutation.
//! Failed arithmetic leaves nodes, edges, and all activation bits intact.

use std::collections::{BTreeMap, BTreeSet};

use super::tdi22_event_aggregate::TorsorEvent;
use super::tdi22_event_gate::Edge;
use super::tdi22_torsor::{FactorizedTorsorKey, Torsor3, TorsorError, Vec3};

pub const MAX_DECLARED_EDGES: usize = 4_096;
pub const MAX_SOURCE_NEIGHBORS: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphError {
    InvalidThresholds,
    SelfConnection(u64),
    DuplicateConnection(u64, u64),
    TooManyConnections,
    TooManyNeighbors(u64),
    InvalidWeight,
    MissingSource(u64),
    Algebra(TorsorError),
    NonFinitePair,
}

impl From<TorsorError> for GraphError {
    fn from(value: TorsorError) -> Self {
        Self::Algebra(value)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PairState {
    pub score: f64,
    pub active: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PairUpdate {
    pub source_a: u64,
    pub source_b: u64,
    pub score: f64,
    pub active: bool,
    pub edge: Edge,
}

fn ordered_pair(a: u64, b: u64) -> (u64, u64) {
    (a.min(b), a.max(b))
}

fn scaled_key(torsor: Torsor3, weight: f64) -> Result<FactorizedTorsorKey, GraphError> {
    if !weight.is_finite() {
        return Err(GraphError::InvalidWeight);
    }
    let key = torsor.factorized_key()?;
    let scale = |v: Vec3| {
        Vec3::new(v.x * weight, v.y * weight, v.z * weight).map_err(GraphError::Algebra)
    };
    Ok(FactorizedTorsorKey {
        resultant: scale(key.resultant)?,
        origin_moment: scale(key.origin_moment)?,
    })
}

fn pair_score(a: FactorizedTorsorKey, b: FactorizedTorsorKey) -> Result<f64, GraphError> {
    let score = a.resultant.dot(b.origin_moment) + b.resultant.dot(a.origin_moment);
    if score.is_finite() {
        Ok(score)
    } else {
        Err(GraphError::NonFinitePair)
    }
}

/// A finite, caller-declared edge topology. No background ticks or implicit
/// dense all-to-all scan are performed. Input is identified TorsorEvent data.
#[derive(Clone, Debug, PartialEq)]
pub struct SparseReciprocalEdgeGraph {
    nodes: BTreeMap<u64, FactorizedTorsorKey>,
    neighbors: BTreeMap<u64, BTreeSet<u64>>,
    edges: BTreeMap<(u64, u64), PairState>,
    lower: f64,
    upper: f64,
}

impl SparseReciprocalEdgeGraph {
    /// All edges are declared before source events. Self-links, duplicate
    /// undirected links and excessive fan-out are rejected, not normalized.
    ///
    /// Nonnegative lower threshold guarantees a removed source deactivates
    /// an active incident edge (score becomes zero).
    pub fn new(
        wiring: &[(u64, u64)],
        lower: f64,
        upper: f64,
    ) -> Result<Self, GraphError> {
        if !lower.is_finite() || !upper.is_finite() || lower < 0.0 || lower >= upper {
            return Err(GraphError::InvalidThresholds);
        }
        if wiring.len() > MAX_DECLARED_EDGES {
            return Err(GraphError::TooManyConnections);
        }

        let mut edges = BTreeMap::new();
        let mut neighbors: BTreeMap<u64, BTreeSet<u64>> = BTreeMap::new();
        for &(x, y) in wiring {
            if x == y {
                return Err(GraphError::SelfConnection(x));
            }
            let (a, b) = ordered_pair(x, y);
            if edges.contains_key(&(a, b)) {
                return Err(GraphError::DuplicateConnection(a, b));
            }
            for (source, neighbor) in [(a, b), (b, a)] {
                let next = neighbors.entry(source).or_default();
                if next.len() >= MAX_SOURCE_NEIGHBORS {
                    return Err(GraphError::TooManyNeighbors(source));
                }
                next.insert(neighbor);
            }
            edges.insert((a, b), PairState::default());
        }
        Ok(Self {
            nodes: BTreeMap::new(),
            neighbors,
            edges,
            lower,
            upper,
        })
    }

    #[must_use]
    pub fn source_count(&self) -> usize {
        self.nodes.len()
    }

    #[must_use]
    pub fn declared_edge_count(&self) -> usize {
        self.edges.len()
    }

    #[must_use]
    pub fn edge_state(&self, a: u64, b: u64) -> Option<PairState> {
        self.edges.get(&ordered_pair(a, b)).copied()
    }

    /// Compute only the changed source's declared neighbors, O(degree
    /// * (log N + log E)) indexed work. The returned list includes
    /// "Edge::None" observations for touched but non-transitioning pairs.
    ///
    /// Event input and the selected graph wiring are untrusted. This is
    /// an experimental reference interface, not production admission.
    pub fn apply(&mut self, event: TorsorEvent) -> Result<Vec<PairUpdate>, GraphError> {
        let (id, replacement) = match event {
            TorsorEvent::Upsert { id, torsor, weight } => {
                (id, Some(scaled_key(torsor, weight)?))
            }
            TorsorEvent::Remove { id } => (id, None),
        };
        if replacement.is_none() && !self.nodes.contains_key(&id) {
            return Err(GraphError::MissingSource(id));
        }
        if self.nodes.get(&id).copied() == replacement {
            return Ok(Vec::new());
        }

        let mut staged = Vec::new();
        if let Some(connected) = self.neighbors.get(&id) {
            staged.reserve(connected.len());
            for &neighbor in connected {
                let pair = ordered_pair(id, neighbor);
                let prior = *self.edges.get(&pair).expect("declared topology is internally complete");
                let next_score = match (replacement, self.nodes.get(&neighbor).copied()) {
                    (Some(source), Some(other)) => pair_score(source, other)?,
                    _ => 0.0,
                };
                let transition = if !prior.active && next_score >= self.upper {
                    Edge::Rising
                } else if prior.active && next_score <= self.lower {
                    Edge::Falling
                } else {
                    Edge::None
                };
                let active = match transition {
                    Edge::Rising => true,
                    Edge::Falling => false,
                    Edge::None => prior.active,
                };
                staged.push((
                    pair,
                    PairState { score: next_score, active },
                    PairUpdate {
                        source_a: pair.0,
                        source_b: pair.1,
                        score: next_score,
                        active,
                        edge: transition,
                    },
                ));
            }
        }

        // No mutations above this line, including on derived overflow.
        match replacement {
            Some(source) => {
                self.nodes.insert(id, source);
            }
            None => {
                self.nodes.remove(&id);
            }
        }
        for (pair, state, _) in &staged {
            *self.edges.get_mut(pair).expect("declared edge") = *state;
        }
        Ok(staged.into_iter().map(|(_, _, update)| update).collect())
    }

    /// Independent O(E) per-edge oracle. Useful only as a reference check;
    /// unlike apply, this scans all declared connections.
    pub fn verify_all_edges(&self) -> Result<bool, GraphError> {
        for (&(a, b), &state) in &self.edges {
            let score = match (self.nodes.get(&a), self.nodes.get(&b)) {
                (Some(lhs), Some(rhs)) => pair_score(*lhs, *rhs)?,
                _ => 0.0,
            };
            if score != state.score {
                return Ok(false);
            }
        }
        Ok(true)
    }

    #[must_use]
    pub fn active_edges(&self) -> Vec<(u64, u64)> {
        self.edges
            .iter()
            .filter_map(|(&pair, state)| state.active.then_some(pair))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::tdi22_cross_source_reciprocity::CrossSourceReciprocity;

    fn v(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3::new(x, y, z).unwrap()
    }

    fn t(r: Vec3, m: Vec3, p: Vec3) -> Torsor3 {
        Torsor3::new(r, m, p).unwrap()
    }

    fn insert(id: u64, r: Vec3, c: Vec3) -> TorsorEvent {
        TorsorEvent::Upsert {
            id,
            torsor: t(r, c, Vec3::zero()),
            weight: 1.0,
        }
    }

    fn close(a: f64, b: f64) {
        let scale = 1.0_f64.max(a.abs()).max(b.abs());
        assert!((a - b).abs() <= 256.0 * f64::EPSILON * scale, "{a} != {b}");
    }

    #[test]
    fn local_edges_detect_opposition_hidden_by_zero_global_interaction() {
        let mut graph = SparseReciprocalEdgeGraph::new(&[(1, 2), (1, 3)], 0.25, 1.0).unwrap();
        let mut aggregate = CrossSourceReciprocity::new();
        let events = [
            insert(1, v(1.0, 0.0, 0.0), Vec3::zero()),
            insert(2, Vec3::zero(), v(2.0, 0.0, 0.0)),
            insert(3, Vec3::zero(), v(-2.0, 0.0, 0.0)),
        ];
        for event in events {
            graph.apply(event).unwrap();
            aggregate.apply(event).unwrap();
        }
        close(aggregate.interaction_score(), 0.0);
        close(aggregate.aggregate_only_score().unwrap(), 0.0);
        assert_eq!(graph.active_edges(), vec![(1, 2)]);
        close(graph.edge_state(1, 2).unwrap().score, 2.0);
        close(graph.edge_state(1, 3).unwrap().score, -2.0);
        assert!(graph.verify_all_edges().unwrap());
    }

    #[test]
    fn only_adjacent_edges_are_touched_by_source_mutation() {
        let mut graph = SparseReciprocalEdgeGraph::new(&[(1, 2), (2, 3)], 0.25, 1.0).unwrap();
        graph.apply(insert(1, v(1.0, 0.0, 0.0), Vec3::zero())).unwrap();
        let changes = graph.apply(insert(2, Vec3::zero(), v(2.0, 0.0, 0.0))).unwrap();
        assert_eq!(changes.len(), 2);
        assert_eq!(changes[0].edge, Edge::Rising);
        assert_eq!(changes[1].edge, Edge::None);
        graph.apply(insert(3, v(1.0, 0.0, 0.0), Vec3::zero())).unwrap();
        assert_eq!(graph.active_edges(), vec![(1, 2), (2, 3)]);
        let old_second_edge = graph.edge_state(2, 3).unwrap();
        let changed = graph.apply(insert(1, v(-1.0, 0.0, 0.0), Vec3::zero())).unwrap();
        assert_eq!(changed.len(), 1);
        assert_eq!(changed[0].edge, Edge::Falling);
        assert_eq!(graph.edge_state(2, 3).unwrap(), old_second_edge);
        assert!(graph.verify_all_edges().unwrap());
    }

    #[test]
    fn replacement_deletion_and_repeat_are_deterministic() {
        let mut graph = SparseReciprocalEdgeGraph::new(&[(1, 2)], 0.0, 0.5).unwrap();
        let first = insert(1, v(1.0, 0.0, 0.0), Vec3::zero());
        graph.apply(first).unwrap();
        assert!(graph.apply(first).unwrap().is_empty());
        let second = insert(2, Vec3::zero(), v(1.0, 0.0, 0.0));
        assert_eq!(graph.apply(second).unwrap()[0].edge, Edge::Rising);
        assert_eq!(
            graph.apply(insert(2, Vec3::zero(), v(-1.0, 0.0, 0.0))).unwrap()[0].edge,
            Edge::Falling
        );
        assert_eq!(graph.apply(second).unwrap()[0].edge, Edge::Rising);
        assert_eq!(graph.apply(TorsorEvent::Remove { id: 2 }).unwrap()[0].edge, Edge::Falling);
        assert!(graph.active_edges().is_empty());
        assert!(graph.verify_all_edges().unwrap());
    }

    #[test]
    fn common_spatial_translation_preserves_edge_scores_and_events() {
        let mut a = SparseReciprocalEdgeGraph::new(&[(1, 2)], 0.25, 1.0).unwrap();
        let mut b = SparseReciprocalEdgeGraph::new(&[(1, 2)], 0.25, 1.0).unwrap();
        let delta = v(3.0, -2.0, 5.0);
        for (id, r, m, p) in [
            (1, v(1.0, 0.0, 0.0), v(0.0, 1.0, 0.0), v(1.0, 2.0, 0.0)),
            (2, v(0.0, 1.0, 0.0), v(2.0, 0.0, 0.0), v(-1.0, 0.0, 2.0)),
        ] {
            let left = a.apply(TorsorEvent::Upsert { id, torsor: t(r, m, p), weight: 1.0 }).unwrap();
            let right = b.apply(TorsorEvent::Upsert {
                id, torsor: t(r, m, p.plus(delta)), weight: 1.0
            }).unwrap();
            assert_eq!(left.len(), right.len());
            for (x, y) in left.iter().zip(&right) {
                close(x.score, y.score);
                assert_eq!(x.edge, y.edge);
                assert_eq!(x.active, y.active);
            }
        }
        assert!(a.verify_all_edges().unwrap());
        assert!(b.verify_all_edges().unwrap());
    }

    #[test]
    fn rejected_nonfinite_pair_and_invalid_input_preserve_all_state() {
        let mut graph = SparseReciprocalEdgeGraph::new(&[(1, 2)], 0.1, 1.0).unwrap();
        graph.apply(insert(1, v(f64::MAX, 0.0, 0.0), Vec3::zero())).unwrap();
        let prior = graph.clone();
        assert_eq!(
            graph.apply(insert(2, Vec3::zero(), v(f64::MAX, 0.0, 0.0))),
            Err(GraphError::NonFinitePair)
        );
        assert_eq!(graph, prior);
        assert_eq!(graph.apply(TorsorEvent::Remove { id: 7 }), Err(GraphError::MissingSource(7)));
        assert_eq!(graph, prior);
        assert_eq!(
            graph.apply(TorsorEvent::Upsert {
                id: 2,
                torsor: t(Vec3::zero(), Vec3::zero(), Vec3::zero()),
                weight: f64::NAN,
            }),
            Err(GraphError::InvalidWeight)
        );
        assert_eq!(graph, prior);
    }

    #[test]
    fn invalid_topology_is_rejected_without_implicit_rewiring() {
        assert_eq!(
            SparseReciprocalEdgeGraph::new(&[(1, 1)], 0.1, 1.0),
            Err(GraphError::SelfConnection(1))
        );
        assert_eq!(
            SparseReciprocalEdgeGraph::new(&[(1, 2), (2, 1)], 0.1, 1.0),
            Err(GraphError::DuplicateConnection(1, 2))
        );
        assert_eq!(
            SparseReciprocalEdgeGraph::new(&[(1, 2)], -1.0, 1.0),
            Err(GraphError::InvalidThresholds)
        );
        let many: Vec<_> = (1..=MAX_SOURCE_NEIGHBORS as u64 + 1)
            .map(|neighbor| (0, neighbor))
            .collect();
        assert_eq!(
            SparseReciprocalEdgeGraph::new(&many, 0.0, 1.0),
            Err(GraphError::TooManyNeighbors(0))
        );
    }

    #[test]
    fn disconnected_node_does_not_trigger_dense_global_scoring() {
        let mut graph = SparseReciprocalEdgeGraph::new(&[(1, 2)], 0.0, 1.0).unwrap();
        assert!(graph.apply(insert(3, v(1.0, 0.0, 0.0), Vec3::zero())).unwrap().is_empty());
        assert_eq!(graph.source_count(), 1);
        assert_eq!(graph.declared_edge_count(), 1);
        assert!(graph.active_edges().is_empty());
        assert!(graph.verify_all_edges().unwrap());
    }
}
