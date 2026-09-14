//! Deterministic Stage-0 development tasks for TDI-21.
//!
//! These fixtures are development scaffolding only. They are not a frozen
//! benchmark and must not be interpreted as confirmatory evidence.

use super::tdi21::{
    BooleanState, Clause, DirectAddressMemory, Literal, MemoryRead, ResourceCounters,
    activate_route,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskKind {
    KeyedBooleanFactRecall,
    CopyAfterMarker,
    TwoFactConjunction,
    DistractorRejection,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaskOutcome {
    pub kind: TaskKind,
    pub correct: bool,
    pub resources: ResourceCounters,
}

/// Store and retrieve one Boolean fact by an already-computed route identifier.
/// No stored token is scanned or ranked.
#[must_use]
pub fn keyed_boolean_fact_recall<const SLOTS: usize>(route: u64, fact: bool) -> TaskOutcome {
    let mut memory = DirectAddressMemory::<SLOTS>::default();
    let mut resources = ResourceCounters::default();
    let encoded = BooleanState::from_bits(if fact { 1 } else { 0 });

    memory.write(route, encoded, &mut resources);
    let correct = matches!(
        memory.read(route, &mut resources),
        MemoryRead::Hit(state) if state == encoded
    );

    TaskOutcome {
        kind: TaskKind::KeyedBooleanFactRecall,
        correct,
        resources,
    }
}

/// A marker clause gates whether a payload is copied into bounded memory.
/// With no marker the correct answer is an explicit miss, not a fabricated
/// payload. This is a smoke fixture rather than a delayed sequence benchmark.
#[must_use]
pub fn copy_after_marker<const SLOTS: usize>(
    marker_state: BooleanState,
    payload: BooleanState,
) -> TaskOutcome {
    const COPY_ROUTE: u64 = 0x434f_5059;
    let marker = Clause {
        literals: [Literal::Bit(0)],
    };
    let mut memory = DirectAddressMemory::<SLOTS>::default();
    let mut resources = ResourceCounters::default();

    if activate_route(marker_state, &marker, &mut resources) {
        memory.write(COPY_ROUTE, payload, &mut resources);
    }

    // The task oracle is defined directly from the input, not from the route
    // activation outcome. A rejected genuine marker must still fail.
    let expected = if marker_state.bits() & 1 == 1 {
        MemoryRead::Hit(payload)
    } else {
        MemoryRead::Miss
    };
    let correct = memory.read(COPY_ROUTE, &mut resources) == expected;

    TaskOutcome {
        kind: TaskKind::CopyAfterMarker,
        correct,
        resources,
    }
}

/// Retrieve two independently addressed Boolean facts and evaluate their exact
/// conjunction. This is a relation over retrieved propositions, not a token
/// similarity score.
#[must_use]
pub fn two_fact_conjunction<const SLOTS: usize>(left: bool, right: bool) -> TaskOutcome {
    const LEFT_ROUTE: u64 = 1;
    const RIGHT_ROUTE: u64 = 2;
    let mut memory = DirectAddressMemory::<SLOTS>::default();
    let mut resources = ResourceCounters::default();
    let left_state = BooleanState::from_bits(if left { 1 } else { 0 });
    let right_state = BooleanState::from_bits(if right { 1 } else { 0 });

    memory.write(LEFT_ROUTE, left_state, &mut resources);
    memory.write(RIGHT_ROUTE, right_state, &mut resources);

    let observed = match (
        memory.read(LEFT_ROUTE, &mut resources),
        memory.read(RIGHT_ROUTE, &mut resources),
    ) {
        (MemoryRead::Hit(a), MemoryRead::Hit(b)) => {
            resources.charge_word_boolean_evals(1);
            Some(a.and_state(b).test(0))
        }
        _ => None,
    };

    TaskOutcome {
        kind: TaskKind::TwoFactConjunction,
        correct: observed == Some(left && right),
        resources,
    }
}

/// Select a target from Boolean predicates while rejecting a distractor. The
/// fixture deliberately performs no distance, dot product, or pairwise ranking.
#[must_use]
pub fn distractor_rejection(target: BooleanState, distractor: BooleanState) -> TaskOutcome {
    let selector = Clause {
        literals: [Literal::Bit(0), Literal::NotBit(1), Literal::Bit(3)],
    };
    let mut resources = ResourceCounters::default();
    let target_selected = activate_route(target, &selector, &mut resources);
    let distractor_selected = activate_route(distractor, &selector, &mut resources);

    TaskOutcome {
        kind: TaskKind::DistractorRejection,
        correct: target_selected && !distractor_selected,
        resources,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyed_fact_recall_is_exact_and_pairwise_free() {
        let outcome = keyed_boolean_fact_recall::<8>(3, true);
        assert!(outcome.correct);
        assert_eq!(outcome.resources.memory_writes, 1);
        assert_eq!(outcome.resources.memory_reads, 1);
        assert_eq!(outcome.resources.pairwise_comparisons, 0);
    }

    #[test]
    fn copy_after_true_marker_recovers_payload() {
        let marker = BooleanState::from_bits(0b0001);
        let payload = BooleanState::from_bits(0b1010_0110);
        let outcome = copy_after_marker::<8>(marker, payload);
        assert!(outcome.correct);
        assert_eq!(outcome.resources.route_activations, 1);
        assert_eq!(outcome.resources.pairwise_comparisons, 0);
    }

    #[test]
    fn copy_after_false_marker_correctly_reports_explicit_miss() {
        let marker = BooleanState::from_bits(0);
        let payload = BooleanState::from_bits(0b1010_0110);
        let outcome = copy_after_marker::<8>(marker, payload);
        assert!(outcome.correct);
        assert_eq!(outcome.resources.memory_misses, 1);
        assert_eq!(outcome.resources.pairwise_comparisons, 0);
    }

    #[test]
    fn missing_capacity_is_not_success_for_a_genuine_marker() {
        let outcome = copy_after_marker::<0>(BooleanState::from_bits(1), BooleanState::from_bits(0));
        assert!(!outcome.correct);
    }

    #[test]
    fn two_fact_conjunction_matches_truth_table() {
        for left in [false, true] {
            for right in [false, true] {
                let outcome = two_fact_conjunction::<8>(left, right);
                assert!(outcome.correct, "left={left}, right={right}");
                assert_eq!(outcome.resources.word_boolean_evals, 1);
                assert_eq!(outcome.resources.pairwise_comparisons, 0);
            }
        }
    }

    #[test]
    fn distractor_is_rejected_by_clause() {
        let target = BooleanState::from_bits(0b1001);
        let distractor = BooleanState::from_bits(0b1011);
        let outcome = distractor_rejection(target, distractor);
        assert!(outcome.correct);
        assert_eq!(outcome.resources.route_activations, 1);
        assert_eq!(outcome.resources.pairwise_comparisons, 0);
    }
}
