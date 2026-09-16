//! Controlled ablations for TDI-2.1 mechanism attribution.

use super::tdi2_intuition::{BooleanState, PredicateId};

/// Remove a frozen set of context predicates from one Boolean query.
///
/// The source state is not mutated. This is an evaluation control, not an
/// alternative inference algorithm.
#[must_use]
pub fn remove_context(
    state: &BooleanState,
    context_predicates: &[PredicateId],
) -> BooleanState {
    let mut masked = context_predicates.to_vec();
    masked.sort_unstable();
    masked.dedup();
    BooleanState::new(
        state
            .predicates()
            .iter()
            .copied()
            .filter(|predicate| masked.binary_search(predicate).is_err())
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::remove_context;
    use crate::experimental::tdi2_intuition::{BooleanState, PredicateId};

    #[test]
    fn context_ablation_preserves_non_context_structure() {
        let state = BooleanState::new(vec![
            PredicateId::new(1),
            PredicateId::new(2),
            PredicateId::new(9),
        ]);
        let ablated = remove_context(&state, &[PredicateId::new(9)]);
        assert_eq!(
            ablated.predicates(),
            &[PredicateId::new(1), PredicateId::new(2)]
        );
        assert_eq!(state.len(), 3);
    }
}
