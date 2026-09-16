//! Deterministic synthetic task fixtures for TDI-2.1 development/validation.

use super::tdi2_intuition::{BooleanState, PredicateId, TemplateId};

/// Frozen synthetic family identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskFamily {
    /// Retrieve a matching experiential motif in the presence of a distractor.
    MotifRetrieval,
    /// The same base motif maps differently under two explicit contexts.
    ContextReversal,
}

/// One labelled development/validation case.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyntheticCase {
    family: TaskFamily,
    query: BooleanState,
    expected_template: TemplateId,
}

impl SyntheticCase {
    /// Task family.
    #[must_use]
    pub const fn family(&self) -> TaskFamily {
        self.family
    }

    /// Boolean query state.
    #[must_use]
    pub const fn query(&self) -> &BooleanState {
        &self.query
    }

    /// Template expected by the frozen synthetic construction.
    #[must_use]
    pub const fn expected_template(&self) -> TemplateId {
        self.expected_template
    }
}

/// Deterministic motif-retrieval case. No RNG state is consumed.
#[must_use]
pub fn motif_retrieval_case(case_id: u32) -> SyntheticCase {
    let motif = PredicateId::new(100 + case_id % 17);
    let distractor = PredicateId::new(10_000 + case_id);
    SyntheticCase {
        family: TaskFamily::MotifRetrieval,
        query: BooleanState::new(vec![motif, distractor]),
        expected_template: TemplateId::new(u64::from(case_id % 17) + 1),
    }
}

/// Deterministic pair for context reversal.
///
/// Both cases share a base motif but differ by one context predicate and must
/// therefore resolve to distinct template identifiers.
#[must_use]
pub fn context_reversal_pair(case_id: u32) -> [SyntheticCase; 2] {
    let base = PredicateId::new(500 + case_id % 13);
    let context_a = PredicateId::new(20_000 + 2 * case_id);
    let context_b = PredicateId::new(20_001 + 2 * case_id);
    let first_id = 50_000 + 2 * u64::from(case_id);
    [
        SyntheticCase {
            family: TaskFamily::ContextReversal,
            query: BooleanState::new(vec![base, context_a]),
            expected_template: TemplateId::new(first_id),
        },
        SyntheticCase {
            family: TaskFamily::ContextReversal,
            query: BooleanState::new(vec![base, context_b]),
            expected_template: TemplateId::new(first_id + 1),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::{TaskFamily, context_reversal_pair, motif_retrieval_case};

    #[test]
    fn motif_fixture_is_deterministic() {
        assert_eq!(motif_retrieval_case(9), motif_retrieval_case(9));
        assert_eq!(motif_retrieval_case(9).family(), TaskFamily::MotifRetrieval);
    }

    #[test]
    fn context_pair_preserves_base_structure_but_changes_target() {
        let [left, right] = context_reversal_pair(3);
        assert_eq!(left.family(), TaskFamily::ContextReversal);
        assert_ne!(left.expected_template(), right.expected_template());
        assert_eq!(left.query().len(), 2);
        assert_eq!(right.query().len(), 2);
        assert_eq!(left.query().predicates()[0], right.query().predicates()[0]);
        assert_ne!(left.query().predicates()[1], right.query().predicates()[1]);
    }
}
