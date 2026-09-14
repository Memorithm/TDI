//! Deterministic Stage-0 task fixtures for TDI-13.

use crate::tdi13::{BooleanState, Clause, DirectAddressMemory, Literal, MemoryRead, ResourceCounters};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskOutcome {
    pub correct: bool,
    pub counters: ResourceCounters,
}

pub fn delayed_bit_recall<const SLOTS: usize>(key: u64, value: bool) -> TaskOutcome {
    let mut memory = DirectAddressMemory::<SLOTS>::default();
    let mut counters = ResourceCounters::default();
    let encoded = BooleanState::from_bits(u64::from(value));

    memory.write(key, encoded, &mut counters);
    let correct = matches!(
        memory.read(key, &mut counters),
        MemoryRead::Hit(state) if state.bits() == u64::from(value)
    );

    TaskOutcome { correct, counters }
}

pub fn distractor_rejection(target: BooleanState, distractor: BooleanState) -> TaskOutcome {
    let clause = Clause {
        literals: [Literal::Bit(0), Literal::NotBit(1), Literal::Bit(3)],
    };
    let mut counters = ResourceCounters::default();
    let target_selected = crate::tdi13::activate_route(target, &clause, &mut counters);
    let distractor_selected = crate::tdi13::activate_route(distractor, &clause, &mut counters);

    TaskOutcome {
        correct: target_selected && !distractor_selected,
        counters,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delayed_bit_recall_fixture_passes() {
        let outcome = delayed_bit_recall::<8>(3, true);
        assert!(outcome.correct);
        assert_eq!(outcome.counters.memory_writes, 1);
        assert_eq!(outcome.counters.memory_reads, 1);
    }

    #[test]
    fn distractor_fixture_passes() {
        let target = BooleanState::from_bits(0b1001);
        let distractor = BooleanState::from_bits(0b1011);
        let outcome = distractor_rejection(target, distractor);
        assert!(outcome.correct);
        assert_eq!(outcome.counters.route_activations, 1);
    }
}
