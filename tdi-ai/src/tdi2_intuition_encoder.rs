//! Frozen predicate-catalogue encoder for TDI-2.1.

use super::tdi2_intuition::{BooleanState, NumericState, PredicateId};
use super::tdi2_intuition_predicates::{Comparison, PredicateError, ScalarPredicate};

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

/// Canonical text record binding ids, feature indices, directions and exact threshold bits.
#[must_use]
pub fn predicate_catalogue_record(predicates: &[ScalarPredicate]) -> String {
    let body = predicates
        .iter()
        .map(|predicate| {
            let comparison = match predicate.comparison {
                Comparison::GreaterOrEqual => "ge",
                Comparison::LessOrEqual => "le",
            };
            format!(
                "{}:{}:{}:{:016x}",
                predicate.id.raw(),
                predicate.feature,
                comparison,
                predicate.threshold.to_bits()
            )
        })
        .collect::<Vec<_>>()
        .join("|");
    format!("tdi2-intuition-predicate-catalogue-v1;{body}")
}

#[cfg(test)]
mod tests {
    use super::{encode_boolean, predicate_catalogue_record};
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

    #[test]
    fn catalogue_record_changes_when_threshold_bits_change() {
        let first = [
            ScalarPredicate::new(PredicateId::new(1), 0, Comparison::GreaterOrEqual, 1.0)
                .expect("predicate"),
        ];
        let second =
            [
                ScalarPredicate::new(PredicateId::new(1), 0, Comparison::GreaterOrEqual, 1.5)
                    .expect("predicate"),
            ];
        assert_ne!(
            predicate_catalogue_record(&first),
            predicate_catalogue_record(&second)
        );
    }
}
