//! Non-final online association of source IDs from temporal co-occurrence.
//!
//! This is an *actual topology-changing* event learner, not only an
//! evaluator of a prewired graph. At each source event, association support
//! increments for sources observed within a fixed event-sequence window.
//! Eligible pairs compete deterministically for a limited edge budget,
//! with bounded node degree. Obsolete edges are deleted as newer eligible
//! associations displace them; removing a source erases its evidence.
//!
//! The selection rule is an ordinary temporal association heuristic, not a
//! novel learning law. Pair activation is evaluated independently with (a)
//! the classical reciprocal torsor pairing and (b) a fixed six-channel
//! bilinear matrix-J control. Both arms receive the SAME learned topology,
//! source features, edge budget, thresholds, timestamps and labels.
//! Labels are accessed only after predictions. No scientific or physical
//! speed/energy advantage is inferred.
//!
//! This bounded reference clones its entire state on each event for atomic
//! rollback. Complexity is not a sparse production-fast-path claim.

use std::collections::{BTreeMap, BTreeSet};

use super::tdi22_bilinear_control::BinaryMetrics;
use super::tdi22_event_aggregate::TorsorEvent;
use super::tdi22_event_gate::Edge;
use super::tdi22_sparse_reciprocal_graph::GraphError;
use super::tdi22_torsor::{FactorizedTorsorKey, Torsor3};

pub const MAX_ASSOCIATIVE_EVENTS: u64 = 4_096;
pub const MAX_ASSOCIATIVE_SOURCES: usize = 64;
pub const MAX_ASSOCIATIVE_EDGES: usize = 32;
pub const MAX_ASSOCIATIVE_DEGREE: usize = 8;

type Pair = (u64, u64);

fn ordered(a: u64, b: u64) -> Pair {
    (a.min(b), a.max(b))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LearningError {
    InvalidConfiguration,
    SourceBudget,
    StaleSequence { previous: u64, received: u64 },
    EventLimit,
    MissingSource(u64),
    Graph(GraphError),
    NonFinitePair,
    ControlMismatch,
    CounterOverflow,
}

impl From<GraphError> for LearningError {
    fn from(value: GraphError) -> Self {
        Self::Graph(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LearningConfig {
    /// Events may be sparse in sequence; co-observation is within this
    /// difference in sequence numbers, not elapsed wall-clock time.
    pub recency_window: u64,
    /// Pair co-observations needed before the link can be selected.
    pub min_support: u16,
    pub max_edges: usize,
    pub max_degree: usize,
}

impl LearningConfig {
    pub fn validate(self) -> Result<(), LearningError> {
        if self.recency_window == 0
            || self.recency_window > MAX_ASSOCIATIVE_EVENTS
            || self.min_support == 0
            || self.max_edges > MAX_ASSOCIATIVE_EDGES
            || self.max_degree == 0
            || self.max_degree > MAX_ASSOCIATIVE_DEGREE
        {
            return Err(LearningError::InvalidConfiguration);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PairEvidence {
    count: u16,
    last_observed: u64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Source {
    torsor: FactorizedTorsorKey,
    ordinary: [f64; 6],
}

impl Source {
    fn from_event(torsor: Torsor3, weight: f64) -> Result<Self, LearningError> {
        if !weight.is_finite() {
            return Err(LearningError::Graph(GraphError::InvalidWeight));
        }
        let key = torsor
            .factorized_key()
            .map_err(|error| LearningError::Graph(GraphError::Algebra(error)))?;
        let scale = |v: super::tdi22_torsor::Vec3| {
            super::tdi22_torsor::Vec3::new(v.x * weight, v.y * weight, v.z * weight)
                .map_err(|error| LearningError::Graph(GraphError::Algebra(error)))
        };
        let torsor = FactorizedTorsorKey {
            resultant: scale(key.resultant)?,
            origin_moment: scale(key.origin_moment)?,
        };
        let ordinary = [
            torsor.resultant.x,
            torsor.resultant.y,
            torsor.resultant.z,
            torsor.origin_moment.x,
            torsor.origin_moment.y,
            torsor.origin_moment.z,
        ];
        Ok(Self { torsor, ordinary })
    }
}

fn torsor_pair(a: Source, b: Source) -> Result<f64, LearningError> {
    let value = a.torsor.resultant.dot(b.torsor.origin_moment)
        + b.torsor.resultant.dot(a.torsor.origin_moment);
    if value.is_finite() {
        Ok(value)
    } else {
        Err(LearningError::NonFinitePair)
    }
}

/// Ordinary x^T J y six-channel bilinear control, independent of the
/// candidate's Vec3-based scoring function.
fn ordinary_pair(a: Source, b: Source) -> Result<f64, LearningError> {
    let x = a.ordinary;
    let y = b.ordinary;
    let forward = (x[0] * y[3] + x[1] * y[4]) + x[2] * y[5];
    let backward = (y[0] * x[3] + y[1] * x[4]) + y[2] * x[5];
    let result = forward + backward;
    if result.is_finite() {
        Ok(result)
    } else {
        Err(LearningError::NonFinitePair)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LearnedEdge {
    pub score: f64,
    pub active: bool,
}

fn observe_gate(previous: LearnedEdge, score: f64, lower: f64, upper: f64) -> (LearnedEdge, Edge) {
    let transition = if !previous.active && score >= upper {
        Edge::Rising
    } else if previous.active && score <= lower {
        Edge::Falling
    } else {
        Edge::None
    };
    let active = match transition {
        Edge::Rising => true,
        Edge::Falling => false,
        Edge::None => previous.active,
    };
    (LearnedEdge { score, active }, transition)
}

fn record(metric: &mut BinaryMetrics, predicted: bool, target: bool) -> Result<(), LearningError> {
    let counter = match (predicted, target) {
        (true, true) => &mut metric.true_positive,
        (true, false) => &mut metric.false_positive,
        (false, false) => &mut metric.true_negative,
        (false, true) => &mut metric.false_negative,
    };
    *counter = counter
        .checked_add(1)
        .ok_or(LearningError::CounterOverflow)?;
    Ok(())
}

#[derive(Clone, Debug, PartialEq)]
pub struct LearningStep {
    pub sequence: u64,
    pub admitted: Vec<Pair>,
    pub evicted: Vec<Pair>,
    pub rising: Vec<Pair>,
    pub falling: Vec<Pair>,
    pub selected_links: Vec<Pair>,
    pub active_links: Vec<Pair>,
    pub target_link_active: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MatchedAssociativeLearner {
    config: LearningConfig,
    lower: f64,
    upper: f64,
    sources: BTreeMap<u64, Source>,
    last_seen: BTreeMap<u64, u64>,
    evidence: BTreeMap<Pair, PairEvidence>,
    candidate: BTreeMap<Pair, LearnedEdge>,
    ordinary: BTreeMap<Pair, LearnedEdge>,
    candidate_metrics: BinaryMetrics,
    ordinary_metrics: BinaryMetrics,
    last_sequence: Option<u64>,
    events: u64,
}

impl MatchedAssociativeLearner {
    pub fn new(config: LearningConfig, lower: f64, upper: f64) -> Result<Self, LearningError> {
        config.validate()?;
        if !lower.is_finite() || !upper.is_finite() || lower < 0.0 || lower >= upper {
            return Err(LearningError::InvalidConfiguration);
        }
        Ok(Self {
            config,
            lower,
            upper,
            sources: BTreeMap::new(),
            last_seen: BTreeMap::new(),
            evidence: BTreeMap::new(),
            candidate: BTreeMap::new(),
            ordinary: BTreeMap::new(),
            candidate_metrics: BinaryMetrics::default(),
            ordinary_metrics: BinaryMetrics::default(),
            last_sequence: None,
            events: 0,
        })
    }

    #[must_use]
    pub const fn processed_events(&self) -> u64 {
        self.events
    }

    #[must_use]
    pub fn learned_links(&self) -> Vec<Pair> {
        self.candidate.keys().copied().collect()
    }

    #[must_use]
    pub fn edge(&self, a: u64, b: u64) -> Option<LearnedEdge> {
        self.candidate.get(&ordered(a, b)).copied()
    }

    #[must_use]
    pub fn support(&self, a: u64, b: u64) -> u16 {
        self.evidence
            .get(&ordered(a, b))
            .map_or(0, |evidence| evidence.count)
    }

    #[must_use]
    pub const fn metrics(&self) -> (BinaryMetrics, BinaryMetrics) {
        (self.candidate_metrics, self.ordinary_metrics)
    }

    /// One event is a complete learning-and-activation transaction. External
    /// labels and target edge are consulted only after evidence selection,
    /// score evaluation and hysteresis have all finished successfully.
    pub fn push(
        &mut self,
        sequence: u64,
        event: TorsorEvent,
        target_edge: Pair,
        independent_target: bool,
    ) -> Result<LearningStep, LearningError> {
        if self.events >= MAX_ASSOCIATIVE_EVENTS {
            return Err(LearningError::EventLimit);
        }
        if let Some(previous) = self.last_sequence {
            if sequence <= previous {
                return Err(LearningError::StaleSequence {
                    previous,
                    received: sequence,
                });
            }
        }
        let mut staged = self.clone();
        match event {
            TorsorEvent::Upsert { id, torsor, weight } => {
                let source = Source::from_event(torsor, weight)?;
                if !staged.sources.contains_key(&id)
                    && staged.sources.len() >= MAX_ASSOCIATIVE_SOURCES
                {
                    return Err(LearningError::SourceBudget);
                }
                // Re-observations of a source are allowed, but do not
                // autocorrelate with itself. Each other recently touched
                // source yields precisely one support increment per event.
                for (&other, &prior) in &staged.last_seen {
                    if other != id && sequence - prior <= staged.config.recency_window {
                        let evidence =
                            staged
                                .evidence
                                .entry(ordered(id, other))
                                .or_insert(PairEvidence {
                                    count: 0,
                                    last_observed: sequence,
                                });
                        evidence.count = evidence.count.saturating_add(1);
                        evidence.last_observed = sequence;
                    }
                }
                staged.sources.insert(id, source);
                staged.last_seen.insert(id, sequence);
            }
            TorsorEvent::Remove { id } => {
                if staged.sources.remove(&id).is_none() {
                    return Err(LearningError::MissingSource(id));
                }
                staged.last_seen.remove(&id);
                staged.evidence.retain(|&(a, b), _| a != id && b != id);
            }
        }

        // Highest evidence support wins, then newest co-observation,
        // then ascending (source_a, source_b). Degree/budget caps are
        // applied in that exact stable order, independent of task labels.
        let mut priorities: Vec<_> = staged
            .evidence
            .iter()
            .filter(|entry| {
                let (pair, e) = *entry;
                e.count >= staged.config.min_support
                    && staged.sources.contains_key(&pair.0)
                    && staged.sources.contains_key(&pair.1)
            })
            .map(|(&pair, &e)| (pair, e))
            .collect();
        priorities.sort_by(|(pair_a, a), (pair_b, b)| {
            b.count
                .cmp(&a.count)
                .then(b.last_observed.cmp(&a.last_observed))
                .then(pair_a.cmp(pair_b))
        });
        let mut degree: BTreeMap<u64, usize> = BTreeMap::new();
        let mut selected = BTreeSet::new();
        for (pair, _) in priorities {
            if selected.len() == staged.config.max_edges {
                break;
            }
            if degree.get(&pair.0).copied().unwrap_or(0) >= staged.config.max_degree
                || degree.get(&pair.1).copied().unwrap_or(0) >= staged.config.max_degree
            {
                continue;
            }
            selected.insert(pair);
            *degree.entry(pair.0).or_default() += 1;
            *degree.entry(pair.1).or_default() += 1;
        }

        let previous: BTreeSet<_> = staged.candidate.keys().copied().collect();
        let admitted: Vec<_> = selected.difference(&previous).copied().collect();
        let evicted: Vec<_> = previous.difference(&selected).copied().collect();
        let mut rising = Vec::new();
        let mut falling = Vec::new();
        for pair in &evicted {
            if staged
                .candidate
                .remove(pair)
                .is_some_and(|edge| edge.active)
            {
                falling.push(*pair);
            }
            staged.ordinary.remove(pair);
        }
        for &pair in &selected {
            let a = staged.sources[&pair.0];
            let b = staged.sources[&pair.1];
            let score = torsor_pair(a, b)?;
            let plain_score = ordinary_pair(a, b)?;
            let candidate_prior = staged.candidate.get(&pair).copied().unwrap_or_default();
            let ordinary_prior = staged.ordinary.get(&pair).copied().unwrap_or_default();
            let (next, event) = observe_gate(candidate_prior, score, staged.lower, staged.upper);
            let (ordinary, plain_event) =
                observe_gate(ordinary_prior, plain_score, staged.lower, staged.upper);
            // Six-channel control must reproduce all candidate edge states;
            // discrepancy invalidates this entire learning transaction.
            if next != ordinary || event != plain_event {
                return Err(LearningError::ControlMismatch);
            }
            staged.candidate.insert(pair, next);
            staged.ordinary.insert(pair, ordinary);
            match event {
                Edge::Rising => rising.push(pair),
                Edge::Falling => falling.push(pair),
                Edge::None => {}
            }
        }

        let selected_links: Vec<_> = staged.candidate.keys().copied().collect();
        let active_links: Vec<_> = staged
            .candidate
            .iter()
            .filter_map(|(&pair, edge)| edge.active.then_some(pair))
            .collect();
        let target_link_active = staged
            .candidate
            .get(&ordered(target_edge.0, target_edge.1))
            .is_some_and(|edge| edge.active);
        let control_target = staged
            .ordinary
            .get(&ordered(target_edge.0, target_edge.1))
            .is_some_and(|edge| edge.active);
        if target_link_active != control_target {
            return Err(LearningError::ControlMismatch);
        }
        record(
            &mut staged.candidate_metrics,
            target_link_active,
            independent_target,
        )?;
        record(
            &mut staged.ordinary_metrics,
            control_target,
            independent_target,
        )?;
        staged.events = staged
            .events
            .checked_add(1)
            .ok_or(LearningError::CounterOverflow)?;
        staged.last_sequence = Some(sequence);

        let report = LearningStep {
            sequence,
            admitted,
            evicted,
            rising,
            falling,
            selected_links,
            active_links,
            target_link_active,
        };
        *self = staged;
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::super::tdi22_torsor::{Torsor3, Vec3};
    use super::*;

    fn v(value: f64) -> Vec3 {
        Vec3::new(value, 0.0, 0.0).unwrap()
    }

    fn source(id: u64, moment: f64) -> TorsorEvent {
        TorsorEvent::Upsert {
            id,
            torsor: Torsor3::new(v(1.0), v(moment), Vec3::zero()).unwrap(),
            weight: 1.0,
        }
    }

    fn config(max_edges: usize) -> LearningConfig {
        LearningConfig {
            recency_window: 2,
            min_support: 2,
            max_edges,
            max_degree: 2,
        }
    }

    fn learner(max_edges: usize) -> MatchedAssociativeLearner {
        MatchedAssociativeLearner::new(config(max_edges), 0.25, 0.75).unwrap()
    }

    fn repeat_pair(
        learner: &mut MatchedAssociativeLearner,
        sequence: &mut u64,
        a: u64,
        b: u64,
        count: usize,
    ) {
        for i in 0..count {
            learner
                .push(
                    *sequence,
                    source(if i % 2 == 0 { a } else { b }, 0.5),
                    (a, b),
                    true,
                )
                .unwrap();
            *sequence += 1;
        }
    }

    #[test]
    fn repeated_temporal_proximity_creates_a_real_active_connection() {
        let mut learner = learner(2);
        for (seq, id) in [(0, 1), (1, 2), (2, 1)] {
            let step = learner.push(seq, source(id, 0.5), (1, 2), true).unwrap();
            if seq < 2 {
                assert!(step.selected_links.is_empty());
            } else {
                assert_eq!(step.admitted, vec![(1, 2)]);
                assert_eq!(step.rising, vec![(1, 2)]);
                assert!(step.target_link_active);
            }
        }
        assert_eq!(learner.support(1, 2), 2);
        assert_eq!(learner.learned_links(), vec![(1, 2)]);
        assert_eq!(learner.metrics().0, learner.metrics().1);
        assert_eq!(learner.processed_events(), 3);
    }

    #[test]
    fn stronger_newer_association_replaces_old_link_at_fixed_budget() {
        let mut learner = learner(1);
        let mut seq = 0;
        repeat_pair(&mut learner, &mut seq, 1, 2, 4);
        assert_eq!(learner.learned_links(), vec![(1, 2)]);
        let mut saw_eviction = false;
        for _ in 0..5 {
            let id = if seq % 2 == 0 { 3 } else { 4 };
            let step = learner.push(seq, source(id, 0.5), (3, 4), false).unwrap();
            if !step.evicted.is_empty() {
                assert_eq!(step.evicted, vec![(1, 2)]);
                assert_eq!(step.admitted, vec![(3, 4)]);
                assert_eq!(step.falling, vec![(1, 2)]);
                saw_eviction = true;
            }
            seq += 1;
        }
        assert!(saw_eviction);
        assert_eq!(learner.learned_links(), vec![(3, 4)]);
        assert_eq!(learner.metrics().0, learner.metrics().1);
    }

    #[test]
    fn deletion_discards_support_and_active_edge_without_revival() {
        let mut learner = learner(2);
        let mut seq = 0;
        repeat_pair(&mut learner, &mut seq, 1, 2, 3);
        let step = learner
            .push(seq, TorsorEvent::Remove { id: 2 }, (1, 2), false)
            .unwrap();
        assert_eq!(step.evicted, vec![(1, 2)]);
        assert_eq!(step.falling, vec![(1, 2)]);
        assert_eq!(learner.support(1, 2), 0);
        assert!(learner.learned_links().is_empty());
        assert!(!step.target_link_active);
    }

    #[test]
    fn labels_and_target_queries_never_influence_learning() {
        let mut a = learner(2);
        let mut b = learner(2);
        for (sequence, id) in [1, 2, 1, 2, 3, 2, 3].into_iter().enumerate() {
            let left = a
                .push(sequence as u64, source(id, 0.5), (1, 2), true)
                .unwrap();
            let right = b
                .push(sequence as u64, source(id, 0.5), (8, 9), false)
                .unwrap();
            assert_eq!(left.admitted, right.admitted);
            assert_eq!(left.evicted, right.evicted);
            assert_eq!(left.selected_links, right.selected_links);
            assert_eq!(left.active_links, right.active_links);
        }
        assert_eq!(a.candidate, b.candidate);
        assert_eq!(a.ordinary, b.ordinary);
        assert_eq!(a.evidence, b.evidence);
        assert_ne!(a.metrics(), b.metrics());
    }

    #[test]
    fn zero_signal_still_learns_topology_but_cannot_activate_a_link() {
        let mut learner = learner(2);
        for (seq, id) in [1, 2, 1].into_iter().enumerate() {
            learner
                .push(seq as u64, source(id, 0.0), (1, 2), true)
                .unwrap();
        }
        assert_eq!(learner.learned_links(), vec![(1, 2)]);
        assert_eq!(learner.edge(1, 2).unwrap().score, 0.0);
        assert!(!learner.edge(1, 2).unwrap().active);
        assert_eq!(learner.metrics().0.true_positive, 0);
    }

    #[test]
    fn budget_and_degree_limits_do_not_depend_on_input_order_ties() {
        let mut learner = MatchedAssociativeLearner::new(
            LearningConfig {
                recency_window: 4,
                min_support: 1,
                max_edges: 2,
                max_degree: 1,
            },
            0.0,
            0.1,
        )
        .unwrap();
        for (seq, id) in [1, 2, 3, 4].into_iter().enumerate() {
            learner
                .push(seq as u64, source(id, 0.5), (1, 2), false)
                .unwrap();
        }
        assert!(learner.learned_links().len() <= 2);
        let mut degrees: BTreeMap<u64, usize> = BTreeMap::new();
        for (a, b) in learner.learned_links() {
            *degrees.entry(a).or_default() += 1;
            *degrees.entry(b).or_default() += 1;
        }
        assert!(degrees.values().all(|&degree| degree <= 1));
    }

    #[test]
    fn stale_sequence_missing_source_and_nonfinite_pair_roll_back() {
        let mut learner = learner(2);
        learner.push(0, source(1, 0.5), (1, 2), false).unwrap();
        let baseline = learner.clone();
        assert_eq!(
            learner.push(0, source(2, 0.5), (1, 2), true),
            Err(LearningError::StaleSequence {
                previous: 0,
                received: 0
            })
        );
        assert_eq!(learner, baseline);
        assert_eq!(
            learner.push(1, TorsorEvent::Remove { id: 99 }, (1, 2), false),
            Err(LearningError::MissingSource(99))
        );
        assert_eq!(learner, baseline);

        // Force pair overflow on link admission and ensure none of the
        // candidate-support or source mutations leak.
        let mut overflow = MatchedAssociativeLearner::new(
            LearningConfig {
                min_support: 1,
                ..config(2)
            },
            0.0,
            1.0,
        )
        .unwrap();
        overflow
            .push(0, source(1, f64::MAX), (1, 2), false)
            .unwrap();
        let snapshot = overflow.clone();
        assert_eq!(
            overflow.push(
                1,
                TorsorEvent::Upsert {
                    id: 2,
                    torsor: Torsor3::new(v(f64::MAX), v(f64::MAX), Vec3::zero()).unwrap(),
                    weight: 1.0,
                },
                (1, 2),
                true
            ),
            Err(LearningError::NonFinitePair)
        );
        assert_eq!(overflow, snapshot);
    }

    #[test]
    fn no_edges_is_a_valid_predeclared_negative_control() {
        let mut learner = learner(0);
        for (seq, id) in [1, 2, 1, 2].into_iter().enumerate() {
            let step = learner
                .push(seq as u64, source(id, 0.5), (1, 2), true)
                .unwrap();
            assert!(step.selected_links.is_empty());
            assert!(!step.target_link_active);
        }
        assert_eq!(learner.metrics().0, learner.metrics().1);
        assert_eq!(learner.metrics().0.false_negative, 4);
    }
}
