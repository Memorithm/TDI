//! Development-only event-driven relational routing with a matched bilinear control.
//!
//! An external torsor source update may cause reciprocal edges to rise.
//! Rising edges seed a bounded breadth-first wave over *currently active*
//! wired links. Nodes reached by that wave retain their last event sequence,
//! even after the corresponding edge falls. No timer or background tick exists.
//!
//! Candidate: existing sparse torsor source graph. Matched negative control:
//! the independently implemented fixed six-channel bilinear graph from
//! tdi22_bilinear_control. Both run the IDENTICAL routing law, topology,
//! thresholds, hop budget and source events. Identical reachability is the
//! expected result: neither the classical pairing nor its routing control
//! establishes novel representational power or any hardware advantage.
//!
//! The paired reference clones bounded source and edge state per event. This
//! is not an optimized O(degree) routing implementation or an energy result.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use super::tdi22_bilinear_control::{
    BinaryMetrics, ControlError, MatchedBilinearProtocol, MatchedEdgeStep,
};
use super::tdi22_event_aggregate::TorsorEvent;
use super::tdi22_event_gate::Edge;
use super::tdi22_sparse_reciprocal_graph::{GraphError, PairUpdate};

pub const MAX_ROUTING_EVENTS: u64 = 4_096;
pub const MAX_ROUTING_HOPS: u8 = 16;

type Link = (u64, u64);

fn link(a: u64, b: u64) -> Link {
    (a.min(b), a.max(b))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoutingError {
    Control(ControlError),
    Graph(GraphError),
    InvalidHopBudget(u8),
    StaleSequence { previous: u64, received: u64 },
    EventLimit,
    UnknownLink(u64, u64),
    DuplicateLink(u64, u64),
    InvalidUpdate(u64, u64),
    CounterOverflow,
    ControlMismatch,
}

impl From<ControlError> for RoutingError {
    fn from(error: ControlError) -> Self {
        Self::Control(error)
    }
}

impl From<GraphError> for RoutingError {
    fn from(error: GraphError) -> Self {
        Self::Graph(error)
    }
}

/// Minimum hop count from any rising edge endpoint in one event wave.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReachedNode {
    pub node: u64,
    pub hops: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoutingStep {
    pub sequence: u64,
    pub rising_links: usize,
    pub falling_links: usize,
    pub active_links: usize,
    pub reached: Vec<ReachedNode>,
}

impl RoutingStep {
    #[must_use]
    pub fn reached_node(&self, node: u64) -> bool {
        self.reached.iter().any(|item| item.node == node)
    }

    #[must_use]
    pub fn distance(&self, node: u64) -> Option<u8> {
        self.reached
            .iter()
            .find(|item| item.node == node)
            .map(|item| item.hops)
    }
}

/// Pure event-driven relational state. The active edge map, adjacency and
/// last-reached event stamp are explicit. Memory is sticky by design: a
/// falling edge does not erase evidence of previously delivered pulses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventRoutingMemory {
    adjacency: BTreeMap<u64, BTreeSet<u64>>,
    wired: BTreeSet<Link>,
    active: BTreeSet<Link>,
    last_reached: BTreeMap<u64, u64>,
    max_hops: u8,
    last_sequence: Option<u64>,
    observations: u64,
}

impl EventRoutingMemory {
    /// Caller is responsible for prior wiring validation by
    /// MatchedBilinearProtocol::new. No duplicate/self-link is admitted.
    fn from_validated_wiring(wiring: &[(u64, u64)], max_hops: u8) -> Self {
        let mut adjacency: BTreeMap<u64, BTreeSet<u64>> = BTreeMap::new();
        let mut wired = BTreeSet::new();
        for &(a, b) in wiring {
            adjacency.entry(a).or_default().insert(b);
            adjacency.entry(b).or_default().insert(a);
            wired.insert(link(a, b));
        }
        Self {
            adjacency,
            wired,
            active: BTreeSet::new(),
            last_reached: BTreeMap::new(),
            max_hops,
            last_sequence: None,
            observations: 0,
        }
    }

    #[must_use]
    pub fn last_reached_at(&self, node: u64) -> Option<u64> {
        self.last_reached.get(&node).copied()
    }

    #[must_use]
    pub fn active_link_count(&self) -> usize {
        self.active.len()
    }

    #[must_use]
    pub const fn event_count(&self) -> u64 {
        self.observations
    }

    /// Apply only edges updated by one accepted source event. A wave
    /// starts at BOTH endpoints of each newly rising edge, traverses
    /// only active declared links, and stops after max_hops steps.
    ///
    /// The inner execution touches the updated edges and reachable active
    /// neighborhood, not the entire unrelated graph. The outer clone
    /// intentionally makes this reference entrypoint O(N+E) in state size.
    pub fn observe(
        &mut self,
        sequence: u64,
        updates: &[PairUpdate],
    ) -> Result<RoutingStep, RoutingError> {
        if self.observations >= MAX_ROUTING_EVENTS {
            return Err(RoutingError::EventLimit);
        }
        if let Some(previous) = self.last_sequence {
            if sequence <= previous {
                return Err(RoutingError::StaleSequence {
                    previous,
                    received: sequence,
                });
            }
        }

        let mut staged = self.clone();
        let mut seen = BTreeSet::new();
        let mut seeds = BTreeSet::new();
        let mut rising_links = 0_usize;
        let mut falling_links = 0_usize;
        for update in updates {
            let pair = link(update.source_a, update.source_b);
            if update.source_a >= update.source_b
                || !staged.wired.contains(&pair)
            {
                return Err(RoutingError::UnknownLink(update.source_a, update.source_b));
            }
            if !seen.insert(pair) {
                return Err(RoutingError::DuplicateLink(pair.0, pair.1));
            }
            if !update.score.is_finite() {
                return Err(RoutingError::InvalidUpdate(pair.0, pair.1));
            }
            let prior_active = staged.active.contains(&pair);
            let expected_edge = match (prior_active, update.active) {
                (false, true) => Edge::Rising,
                (true, false) => Edge::Falling,
                _ => Edge::None,
            };
            if update.edge != expected_edge {
                return Err(RoutingError::InvalidUpdate(pair.0, pair.1));
            }
            if update.active {
                staged.active.insert(pair);
            } else {
                staged.active.remove(&pair);
            }
            match update.edge {
                Edge::Rising => {
                    rising_links += 1;
                    seeds.insert(pair.0);
                    seeds.insert(pair.1);
                }
                Edge::Falling => falling_links += 1,
                Edge::None => {}
            }
        }

        let mut distance = BTreeMap::new();
        let mut queue = VecDeque::new();
        for source in seeds {
            distance.insert(source, 0_u8);
            queue.push_back(source);
        }
        while let Some(current) = queue.pop_front() {
            let hops = distance[&current];
            if hops >= staged.max_hops {
                continue;
            }
            if let Some(neighbors) = staged.adjacency.get(&current) {
                for &neighbor in neighbors {
                    if staged.active.contains(&link(current, neighbor))
                        && !distance.contains_key(&neighbor)
                    {
                        distance.insert(neighbor, hops + 1);
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        let reached: Vec<_> = distance
            .iter()
            .map(|(&node, &hops)| ReachedNode { node, hops })
            .collect();
        for entry in &reached {
            staged.last_reached.insert(entry.node, sequence);
        }
        staged.observations = staged
            .observations
            .checked_add(1)
            .ok_or(RoutingError::CounterOverflow)?;
        staged.last_sequence = Some(sequence);
        let result = RoutingStep {
            sequence,
            rising_links,
            falling_links,
            active_links: staged.active.len(),
            reached,
        };
        *self = staged;
        Ok(result)
    }
}

fn record(
    metric: &mut BinaryMetrics,
    prediction: bool,
    target: bool,
) -> Result<(), RoutingError> {
    let slot = match (prediction, target) {
        (true, true) => &mut metric.true_positive,
        (true, false) => &mut metric.false_positive,
        (false, false) => &mut metric.true_negative,
        (false, true) => &mut metric.false_negative,
    };
    *slot = slot.checked_add(1).ok_or(RoutingError::CounterOverflow)?;
    Ok(())
}

#[derive(Clone, Debug, PartialEq)]
pub struct MatchedRoutingStep {
    pub source_edges: MatchedEdgeStep,
    pub candidate_route: RoutingStep,
    pub ordinary_bilinear_route: RoutingStep,
    pub target_node: u64,
    pub predicted_reached: bool,
}

/// Scores a task-supplied target node only *after* both routing waves are
/// computed. Neither routing algorithm sees the node query or its label.
#[derive(Clone, Debug, PartialEq)]
pub struct MatchedEventRoutingProtocol {
    paired_edges: MatchedBilinearProtocol,
    candidate: EventRoutingMemory,
    ordinary_bilinear: EventRoutingMemory,
    candidate_metrics: BinaryMetrics,
    ordinary_metrics: BinaryMetrics,
}

impl MatchedEventRoutingProtocol {
    pub fn new(
        wiring: &[(u64, u64)],
        lower: f64,
        upper: f64,
        max_hops: u8,
    ) -> Result<Self, RoutingError> {
        if max_hops > MAX_ROUTING_HOPS {
            return Err(RoutingError::InvalidHopBudget(max_hops));
        }
        let paired_edges = MatchedBilinearProtocol::new(wiring, lower, upper)?;
        Ok(Self {
            paired_edges,
            candidate: EventRoutingMemory::from_validated_wiring(wiring, max_hops),
            ordinary_bilinear: EventRoutingMemory::from_validated_wiring(wiring, max_hops),
            candidate_metrics: BinaryMetrics::default(),
            ordinary_metrics: BinaryMetrics::default(),
        })
    }

    #[must_use]
    pub const fn metrics(&self) -> (BinaryMetrics, BinaryMetrics) {
        (self.candidate_metrics, self.ordinary_metrics)
    }

    #[must_use]
    pub fn last_reached_at(&self, node: u64) -> Option<u64> {
        self.candidate.last_reached_at(node)
    }

    #[must_use]
    pub fn active_link_count(&self) -> usize {
        self.candidate.active_link_count()
    }

    pub fn push(
        &mut self,
        sequence: u64,
        event: TorsorEvent,
        target_node: u64,
        independent_target: bool,
    ) -> Result<MatchedRoutingStep, RoutingError> {
        // Full protocol cloning preserves atomicity across the two edge
        // scorers, two routing memories and all task-confusion counters.
        let mut staged = self.clone();
        let source_edges = staged.paired_edges.push(sequence, event, independent_target)?;
        let candidate_route =
            staged.candidate.observe(sequence, &source_edges.candidate_updates)?;
        let ordinary_bilinear_route =
            staged.ordinary_bilinear.observe(sequence, &source_edges.bilinear_updates)?;
        if candidate_route != ordinary_bilinear_route {
            return Err(RoutingError::ControlMismatch);
        }
        let predicted_reached = candidate_route.reached_node(target_node);
        record(
            &mut staged.candidate_metrics,
            predicted_reached,
            independent_target,
        )?;
        record(
            &mut staged.ordinary_metrics,
            ordinary_bilinear_route.reached_node(target_node),
            independent_target,
        )?;
        *self = staged;
        Ok(MatchedRoutingStep {
            source_edges,
            candidate_route,
            ordinary_bilinear_route,
            target_node,
            predicted_reached,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::tdi22_torsor::{Torsor3, Vec3};

    fn v(x: f64) -> Vec3 {
        Vec3::new(x, 0.0, 0.0).unwrap()
    }

    fn source(id: u64, amplitude: f64) -> TorsorEvent {
        TorsorEvent::Upsert {
            id,
            torsor: Torsor3::new(v(amplitude), v(0.5), Vec3::zero()).unwrap(),
            weight: 1.0,
        }
    }

    fn chain(hops: u8) -> MatchedEventRoutingProtocol {
        MatchedEventRoutingProtocol::new(
            &[(1, 2), (2, 3), (3, 4)],
            0.25,
            0.75,
            hops,
        )
        .unwrap()
    }

    #[test]
    fn wave_reaches_two_hops_without_a_source_update_at_destination() {
        let mut graph = chain(2);
        for id in 1..4 {
            graph.push(id - 1, source(id, 1.0), 1, false).unwrap();
        }
        let wave = graph.push(3, source(4, 1.0), 1, true).unwrap();
        assert_eq!(wave.candidate_route.distance(1), Some(2));
        assert_eq!(wave.candidate_route.distance(2), Some(1));
        assert_eq!(wave.candidate_route.distance(3), Some(0));
        assert_eq!(wave.candidate_route.distance(4), Some(0));
        assert!(wave.predicted_reached);
        assert_eq!(wave.candidate_route, wave.ordinary_bilinear_route);
        assert_eq!(graph.metrics().0, graph.metrics().1);
        assert_eq!(graph.last_reached_at(1), Some(3));
    }

    #[test]
    fn no_propagation_control_cannot_reach_nonincident_node() {
        let mut direct = chain(0);
        let mut propagated = chain(2);
        for id in 1..4 {
            direct.push(id - 1, source(id, 1.0), 1, false).unwrap();
            propagated.push(id - 1, source(id, 1.0), 1, false).unwrap();
        }
        let d = direct.push(3, source(4, 1.0), 1, true).unwrap();
        let p = propagated.push(3, source(4, 1.0), 1, true).unwrap();
        assert!(!d.predicted_reached);
        assert!(p.predicted_reached);
        assert_eq!(d.candidate_route.distance(1), None);
        assert_eq!(p.candidate_route.distance(1), Some(2));
    }

    #[test]
    fn no_new_rising_edge_produces_no_new_pulse_and_retains_memory() {
        let mut graph = chain(2);
        for id in 1..=4 {
            graph.push(id - 1, source(id, 1.0), 1, true).unwrap();
        }
        let before = graph.last_reached_at(1);
        let same = graph.push(4, source(4, 1.0), 1, false).unwrap();
        assert_eq!(same.candidate_route.rising_links, 0);
        assert!(same.candidate_route.reached.is_empty());
        assert_eq!(graph.last_reached_at(1), before);
        let fallen = graph.push(5, TorsorEvent::Remove { id: 4 }, 1, false).unwrap();
        assert_eq!(fallen.candidate_route.falling_links, 1);
        assert!(fallen.candidate_route.reached.is_empty());
        assert_eq!(graph.active_link_count(), 2);
        assert_eq!(graph.last_reached_at(1), before);
    }

    #[test]
    fn target_labels_do_not_change_reachability() {
        let mut a = chain(2);
        let mut b = chain(2);
        for id in 1..=4 {
            let left = a.push(id - 1, source(id, 1.0), 1, true).unwrap();
            let right = b.push(id - 1, source(id, 1.0), 1, false).unwrap();
            assert_eq!(left.candidate_route, right.candidate_route);
            assert_eq!(left.ordinary_bilinear_route, right.ordinary_bilinear_route);
        }
        assert_ne!(a.metrics(), b.metrics());
        assert_eq!(a.candidate, b.candidate);
        assert_eq!(a.ordinary_bilinear, b.ordinary_bilinear);
    }

    #[test]
    fn disconnected_node_remains_unreached() {
        let mut graph = MatchedEventRoutingProtocol::new(
            &[(1, 2), (3, 4)],
            0.25,
            0.75,
            4,
        )
        .unwrap();
        graph.push(0, source(1, 1.0), 3, false).unwrap();
        let step = graph.push(1, source(2, 1.0), 3, false).unwrap();
        assert!(!step.predicted_reached);
        assert_eq!(step.candidate_route.distance(3), None);
        assert_eq!(step.candidate_route, step.ordinary_bilinear_route);
    }

    #[test]
    fn invalid_event_and_stale_sequence_rollback_complete_protocol() {
        let mut graph = chain(2);
        graph.push(0, source(1, 1.0), 1, false).unwrap();
        let before = graph.clone();
        let error = graph.push(0, source(2, 1.0), 1, false);
        assert_eq!(
            error,
            Err(RoutingError::Control(ControlError::StaleSequence {
                previous: 0,
                received: 0
            }))
        );
        assert_eq!(graph, before);
        assert_eq!(
            graph.push(1, TorsorEvent::Remove { id: 999 }, 1, false),
            Err(RoutingError::Control(ControlError::Graph(
                GraphError::MissingSource(999)
            )))
        );
        assert_eq!(graph, before);
    }

    #[test]
    fn contradictory_edge_updates_are_rejected_transactionally() {
        let mut state = EventRoutingMemory::from_validated_wiring(&[(1, 2)], 2);
        let malformed = PairUpdate {
            source_a: 1,
            source_b: 2,
            score: 1.0,
            active: false,
            edge: Edge::Rising,
        };
        let before = state.clone();
        assert_eq!(
            state.observe(0, &[malformed]),
            Err(RoutingError::InvalidUpdate(1, 2))
        );
        assert_eq!(state, before);
        let unknown = PairUpdate {
            source_a: 3,
            source_b: 4,
            ..malformed
        };
        assert_eq!(
            state.observe(0, &[unknown]),
            Err(RoutingError::UnknownLink(3, 4))
        );
        assert_eq!(state, before);
    }

    #[test]
    fn no_signal_events_cannot_emit_routing_pulses() {
        let mut graph = chain(3);
        for id in 1..=4 {
            let step = graph.push(id - 1, source(id, 0.0), 1, id % 2 == 0).unwrap();
            assert!(step.candidate_route.reached.is_empty());
            assert_eq!(step.candidate_route, step.ordinary_bilinear_route);
        }
        let (torsor, bilinear) = graph.metrics();
        assert_eq!(torsor, bilinear);
        assert_eq!((torsor.true_positive, torsor.false_positive), (0, 0));
        assert_eq!((torsor.true_negative, torsor.false_negative), (2, 2));
    }
}
