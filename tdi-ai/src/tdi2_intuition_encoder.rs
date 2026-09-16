//! Frozen predicate-catalogue encoder for TDI-2.1.

use super::tdi2_intuition::{BooleanState, NumericState, PredicateId};
use super::tdi2_intuition_predicates::{PredicateError, ScalarPredicate};

/// Apply a frozen scalar predicate catalogue to one numeric state.
pub fn encode_boolean(
    state: &NumericState,
    predicates: &[ScalarPredicate],
) -> Result<BooleanState, PredicateError> {
    let mut active = Vec::new();
    for predicate in predicates {
        if predicate.evaluate(state)? {
            active.push(predicate.id);
        }
    }
    Ok(BooleanState::new(active))
}

/// Return predicate ids in catalogue order for provenance binding.
#[must_use]
pub fn predicate_catalogue_ids(predicates: &[ScalarPredicate]) -> Vec<PredicateId> {
    predicates.iter().map(|predicate| predicate.id).collect()
}

#[cfg(test)]
mod tests {
    use super::encode_boolean;
    use crate::experimental::tdi2_intuition::{NumericState, PredicateId};
    use crate::experimental::tdi2_intuition_predicates::{Comparison, ScalarPredicate};

    #[test]
    fn encoder_emits_only_true_predicates() {
        let state = NumericState::new(vec![2.0, -1.0]).expect("state");
        let predicates = [
            ScalarPredicate::new(PredicateId::new(1), 0, Comparison::GreaterOrEqual, 1.0)
                .expect("predicate"),
            ScalarPredicate::new(PredicateId::new(2), 1, Comparison::GreaterOrEqual, 0.0)
                .expect("predicate"),
        ];
        let encoded = encode_boolean(&state, &predicates).expect("encode");
        assert_eq!(encoded.predicates(), &[PredicateId::new(1)]);
    }
}
