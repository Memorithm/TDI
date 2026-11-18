//! TDI-22 experimental association-to-routing integration (development only).
//!
//! Pairs co-observed in a bounded sequence window are learned as live links.
//! A newly active link launches a bounded pulse wave over the *current*
//! dynamically selected graph; eviction immediately blocks that path while
//! preserving historical node delivery stamps. No background clock or timer.
//!
//! The very same learned topology, thresholds, hysteresis results, hop budget,
//! node stamps and routing policy are given to the ordinary six-channel
//! bilinear control. Logical parity is expected, not scientific novelty.
//!
//! Full-state staging in both the learner and routers is reference-only,
//! not a sparse incremental fast path or an energy/latency measurement.

use super::tdi22_bilinear_control::BinaryMetrics;
use super::tdi22_event_aggregate::TorsorEvent;
use super::tdi22_event_routing_memory::{
    EventRoutingMemory, RewiredRoutingStep, RoutingError,
};
use super::tdi22_online_associative_links::{
    LearningConfig, LearningError, LearningStep, MatchedAssociativeLearner,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetworkError {
    Learning(LearningError),
    Routing(RoutingError),
    StateMismatch,
    CounterOverflow,
}

impl From<LearningError> for NetworkError {
    fn from(error: LearningError) -> Self {
        Self::Learning(error)
    }
}

impl From<RoutingError> for NetworkError {
    fn from(error: RoutingError) -> Self {
        Self::Routing(error)
    }
}

fn record(metric: &mut BinaryMetrics, prediction: bool, target: bool) -> Result<(), NetworkError> {
    let counter = match (prediction, target) {
        (true, true) => &mut metric.true_positive,
        (true, false) => &mut metric.false_positive,
        (false, false) => &mut metric.true_negative,
        (false, true) => &mut metric.false_negative,
    };
    *counter = counter.checked_add(1).ok_or(NetworkError::CounterOverflow)?;
    Ok(())
}

#[derive(Clone, Debug, PartialEq)]
pub struct LearnedPulseStep {
    pub learning: LearningStep,
    pub candidate: RewiredRoutingStep,
    pub ordinary_bilinear: RewiredRoutingStep,
    pub target_node: u64,
    pub predicted_reached: bool,
}

/// Full paired research reference, requiring identical learned wiring and
/// delivery on both representations. External query labels never enter
/// co-occurrence counters, edge selection, score gates or pulse traversal.
#[derive(Clone, Debug, PartialEq)]
pub struct MatchedLearnedPulseNetwork {
    learner: MatchedAssociativeLearner,
    candidate: EventRoutingMemory,
    ordinary_bilinear: EventRoutingMemory,
    candidate_metrics: BinaryMetrics,
    ordinary_metrics: BinaryMetrics,
}

impl MatchedLearnedPulseNetwork {
    pub fn new(
        learning: LearningConfig,
        lower: f64,
        upper: f64,
        max_hops: u8,
    ) -> Result<Self, NetworkError> {
        Ok(Self {
            learner: MatchedAssociativeLearner::new(learning, lower, upper)?,
            candidate: EventRoutingMemory::for_learned_wiring(max_hops)?,
            ordinary_bilinear: EventRoutingMemory::for_learned_wiring(max_hops)?,
            candidate_metrics: BinaryMetrics::default(),
            ordinary_metrics: BinaryMetrics::default(),
        })
    }

    #[must_use]
    pub fn learned_links(&self) -> Vec<(u64, u64)> {
        self.learner.learned_links()
    }

    #[must_use]
    pub fn last_reached_at(&self, node: u64) -> Option<u64> {
        self.candidate.last_reached_at(node)
    }

    #[must_use]
    pub const fn metrics(&self) -> (BinaryMetrics, BinaryMetrics) {
        (self.candidate_metrics, self.ordinary_metrics)
    }

    /// Atomic source learning, topology change and pulse delivery.
    ///
    /// The learner's existing classification interface receives a fixed
    /// impossible sentinel link and a constant false label. These do NOT
    /// affect its link selection or activation, and its internal diagnostic
    /// counters are unused. The actual independent target node and task
    /// label are evaluated only after BOTH pulse deliveries finish.
    pub fn push(
        &mut self,
        sequence: u64,
        event: TorsorEvent,
        target_node: u64,
        independent_target: bool,
    ) -> Result<LearnedPulseStep, NetworkError> {
        let mut staged = self.clone();
        let learning = staged
            .learner
            .push(sequence, event, (u64::MAX, u64::MAX), false)?;
        let candidate = staged.candidate.observe_rewired(
            sequence,
            &learning.selected_links,
            &learning.active_links,
        )?;
        let ordinary_bilinear = staged.ordinary_bilinear.observe_rewired(
            sequence,
            &learning.selected_links,
            &learning.active_links,
        )?;

        if candidate != ordinary_bilinear {
            return Err(NetworkError::StateMismatch);
        }
        // Compare independent change accounting from the learning layer
        // against the dynamic router, including link evictions and falls.
        let mut expected_falls = learning.falling.clone();
        expected_falls.sort_unstable();
        if candidate.admitted != learning.admitted
            || candidate.evicted != learning.evicted
            || candidate.rising != learning.rising
            || candidate.falling != expected_falls
            || staged.candidate.declared_links() != learning.selected_links
            || staged.ordinary_bilinear.declared_links() != learning.selected_links
        {
            return Err(NetworkError::StateMismatch);
        }

        let predicted_reached = candidate.routing.reached_node(target_node);
        let control_reached = ordinary_bilinear.routing.reached_node(target_node);
        record(
            &mut staged.candidate_metrics,
            predicted_reached,
            independent_target,
        )?;
        record(
            &mut staged.ordinary_metrics,
            control_reached,
            independent_target,
        )?;
        if staged.candidate_metrics != staged.ordinary_metrics {
            return Err(NetworkError::StateMismatch);
        }
        *self = staged;
        Ok(LearnedPulseStep {
            learning,
            candidate,
            ordinary_bilinear,
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

    fn event(id: u64, moment: f64) -> TorsorEvent {
        TorsorEvent::Upsert {
            id,
            torsor: Torsor3::new(v(1.0), v(moment), Vec3::zero()).unwrap(),
            weight: 1.0,
        }
    }

    fn cfg(max_edges: usize) -> LearningConfig {
        LearningConfig {
            recency_window: 2,
            min_support: 2,
            max_edges,
            max_degree: 2,
        }
    }

    fn network(max_edges: usize, hops: u8) -> MatchedLearnedPulseNetwork {
        MatchedLearnedPulseNetwork::new(cfg(max_edges), 0.25, 0.75, hops).unwrap()
    }

    #[test]
    fn learned_edge_triggers_both_pulse_arms_without_fixed_wiring() {
        let mut n = network(2, 2);
        for (seq, id) in [(0, 1), (1, 2), (2, 1)] {
            let step = n.push(seq, event(id, 0.5), 2, true).unwrap();
            assert_eq!(step.candidate, step.ordinary_bilinear);
            if seq < 2 {
                assert!(step.learning.selected_links.is_empty());
                assert!(!step.predicted_reached);
            } else {
                assert_eq!(step.learning.admitted, vec![(1, 2)]);
                assert_eq!(step.candidate.rising, vec![(1, 2)]);
                assert_eq!(step.candidate.routing.rising_links, 1);
                assert!(step.predicted_reached);
                assert_eq!(n.last_reached_at(2), Some(2));
            }
        }
        assert_eq!(n.learned_links(), vec![(1, 2)]);
        assert_eq!(n.metrics().0, n.metrics().1);
    }

    #[test]
    fn no_fresh_rise_and_eviction_keep_old_delivery_stamps() {
        let mut n = network(1, 2);
        for (seq, id) in [(0, 1), (1, 2), (2, 1)] {
            n.push(seq, event(id, 0.5), 1, false).unwrap();
        }
        assert_eq!(n.last_reached_at(1), Some(2));
        let same = n.push(3, event(2, 0.5), 1, true).unwrap();
        assert!(same.candidate.rising.is_empty());
        assert!(same.candidate.routing.reached.is_empty());
        assert_eq!(n.last_reached_at(1), Some(2));
        let removed = n.push(4, TorsorEvent::Remove { id: 2 }, 1, false).unwrap();
        assert_eq!(removed.learning.evicted, vec![(1, 2)]);
        assert_eq!(removed.candidate.falling, vec![(1, 2)]);
        assert!(removed.candidate.routing.reached.is_empty());
        assert!(n.learned_links().is_empty());
        assert_eq!(n.last_reached_at(1), Some(2));
    }

    #[test]
    fn stronger_association_displaces_live_route_with_equal_control() {
        let mut n = network(1, 2);
        for (seq, id) in [(0, 1), (1, 2), (2, 1), (3, 2)] {
            n.push(seq, event(id, 0.5), 3, false).unwrap();
        }
        assert_eq!(n.learned_links(), vec![(1, 2)]);
        let mut replacement = false;
        for (seq, id) in [(4, 3), (5, 4), (6, 3), (7, 4), (8, 3)] {
            let step = n.push(seq, event(id, 0.5), 4, true).unwrap();
            assert_eq!(step.candidate, step.ordinary_bilinear);
            if step.learning.evicted.contains(&(1, 2)) {
                assert_eq!(step.learning.admitted, vec![(3, 4)]);
                assert_eq!(step.candidate.falling, vec![(1, 2)]);
                assert_eq!(step.candidate.rising, vec![(3, 4)]);
                replacement = true;
            }
        }
        assert!(replacement);
        assert_eq!(n.learned_links(), vec![(3, 4)]);
        assert_eq!(n.metrics().0, n.metrics().1);
    }

    #[test]
    fn dynamic_two_hop_wave_reaches_node_via_retained_links() {
        let mut routed = EventRoutingMemory::for_learned_wiring(2).unwrap();
        let first = routed
            .observe_rewired(0, &[(1, 2), (2, 3)], &[(1, 2), (2, 3)])
            .unwrap();
        assert_eq!(first.rising, vec![(1, 2), (2, 3)]);
        let second = routed
            .observe_rewired(
                1,
                &[(1, 2), (2, 3), (3, 4)],
                &[(1, 2), (2, 3), (3, 4)],
            )
            .unwrap();
        assert_eq!(second.admitted, vec![(3, 4)]);
        assert_eq!(second.rising, vec![(3, 4)]);
        assert_eq!(second.routing.distance(1), Some(2));
        assert_eq!(routed.last_reached_at(1), Some(1));
        assert_eq!(second.routing.distance(4), Some(0));
    }

    #[test]
    fn equal_topology_zero_hop_ablation_does_not_reach_distant_node() {
        let mut local = EventRoutingMemory::for_learned_wiring(0).unwrap();
        let mut wave = EventRoutingMemory::for_learned_wiring(2).unwrap();
        for router in [&mut local, &mut wave] {
            router
                .observe_rewired(0, &[(1, 2), (2, 3)], &[(1, 2), (2, 3)])
                .unwrap();
        }
        let selected = [(1, 2), (2, 3), (3, 4)];
        let immediate = local.observe_rewired(1, &selected, &selected).unwrap();
        let propagated = wave.observe_rewired(1, &selected, &selected).unwrap();
        assert_eq!(immediate.routing.distance(1), None);
        assert_eq!(propagated.routing.distance(1), Some(2));
        assert_eq!(immediate.rising, propagated.rising);
    }

    #[test]
    fn task_queries_and_labels_are_not_learning_features() {
        let mut yes = network(2, 2);
        let mut no = network(2, 2);
        for (seq, id) in [(0, 1), (1, 2), (2, 1), (3, 2), (4, 3)] {
            let a = yes.push(seq, event(id, 0.5), 1, true).unwrap();
            let b = no.push(seq, event(id, 0.5), 99, false).unwrap();
            assert_eq!(a.learning.selected_links, b.learning.selected_links);
            assert_eq!(a.learning.active_links, b.learning.active_links);
            assert_eq!(a.candidate, b.candidate);
        }
        assert_eq!(yes.learned_links(), no.learned_links());
        assert_eq!(yes.candidate, no.candidate);
        assert_eq!(yes.ordinary_bilinear, no.ordinary_bilinear);
        assert_ne!(yes.metrics(), no.metrics());
    }

    #[test]
    fn zero_signal_can_learn_wiring_but_cannot_deliver_pulse() {
        let mut n = network(2, 2);
        for (seq, id) in [(0, 1), (1, 2), (2, 1)] {
            let step = n.push(seq, event(id, 0.0), 2, true).unwrap();
            assert!(step.candidate.routing.reached.is_empty());
            assert_eq!(step.candidate, step.ordinary_bilinear);
        }
        assert_eq!(n.learned_links(), vec![(1, 2)]);
        assert_eq!(n.last_reached_at(2), None);
        assert_eq!(n.metrics().0.true_positive, 0);
    }

    #[test]
    fn untrusted_rewire_and_stale_sequence_fail_transactionally() {
        let mut r = EventRoutingMemory::for_learned_wiring(2).unwrap();
        r.observe_rewired(0, &[(1, 2)], &[(1, 2)]).unwrap();
        let before = r.clone();
        assert_eq!(
            r.observe_rewired(1, &[(1, 2)], &[(2, 3)]),
            Err(RoutingError::UnknownLink(2, 3))
        );
        assert_eq!(r, before);
        assert_eq!(
            r.observe_rewired(1, &[(2, 1)], &[(1, 2)]),
            Err(RoutingError::UnknownLink(2, 1))
        );
        assert_eq!(r, before);
        assert_eq!(
            r.observe_rewired(1, &[(1, 2)], &[(1, 2), (1, 2)]),
            Err(RoutingError::DuplicateLink(1, 2))
        );
        assert_eq!(r, before);
        assert_eq!(
            r.observe_rewired(0, &[(1, 2)], &[(1, 2)]),
            Err(RoutingError::StaleSequence { previous: 0, received: 0 })
        );
        assert_eq!(r, before);
    }

    #[test]
    fn invalid_source_event_rolls_back_learner_and_both_memories() {
        let mut n = network(1, 2);
        n.push(0, event(1, 0.5), 2, false).unwrap();
        let before = n.clone();
        assert_eq!(
            n.push(1, TorsorEvent::Remove { id: 42 }, 2, false),
            Err(NetworkError::Learning(LearningError::MissingSource(42)))
        );
        assert_eq!(n, before);
        assert_eq!(
            n.push(0, event(2, 0.5), 2, true),
            Err(NetworkError::Learning(LearningError::StaleSequence {
                previous: 0, received: 0
            }))
        );
        assert_eq!(n, before);
    }
}
