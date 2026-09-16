//! Deterministic synthetic task fixtures for TDI-2.1 development/validation.

use super::tdi2_intuition::{BooleanState, PredicateId, TemplateId};
use super::tdi2_intuition_temporal::BooleanSequence;

/// Frozen synthetic family identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskFamily {
    /// Retrieve a matching experiential motif in the presence of a distractor.
    MotifRetrieval,
    /// The same base motif maps differently under two explicit contexts.
    ContextReversal,
    /// Ordered Boolean frames distinguish temporal direction.
    TemporalTrend,
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

/// One labelled temporal development/validation case.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemporalCase {
    query: BooleanSequence,
    expected_template: TemplateId,
}

impl TemporalCase {
    /// Ordered Boolean temporal query.
    #[must_use]
    pub const fn query(&self) -> &BooleanSequence {
        &self.query
    }

    /// Expected temporal template identifier.
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

/// Motif retrieval with a controlled number of irrelevant novel predicates.
#[must_use]
pub fn motif_retrieval_stress_case(case_id: u32, distractor_count: u16) -> SyntheticCase {
    let motif = PredicateId::new(100 + case_id % 17);
    let block = 1_000_000 + (case_id % 10_000) * 1024;
    let mut predicates = Vec::with_capacity(usize::from(distractor_count) + 1);
    predicates.push(motif);
    predicates
        .extend((0..distractor_count).map(|offset| PredicateId::new(block + u32::from(offset))));
    SyntheticCase {
        family: TaskFamily::MotifRetrieval,
        query: BooleanState::new(predicates),
        expected_template: TemplateId::new(u64::from(case_id % 17) + 1),
    }
}

/// State deliberately containing no known motif predicate.
#[must_use]
pub fn motif_ood_state(case_id: u32, predicate_count: u16) -> BooleanState {
    let block = 20_000_000 + (case_id % 10_000) * 1024;
    BooleanState::new(
        (0..predicate_count)
            .map(|offset| PredicateId::new(block + u32::from(offset)))
            .collect(),
    )
}

/// State containing two known motifs simultaneously, creating exact ambiguity.
#[must_use]
pub fn motif_ambiguous_state(first: u8, second: u8) -> Option<BooleanState> {
    if first >= 17 || second >= 17 || first == second {
        return None;
    }
    Some(BooleanState::new(vec![
        PredicateId::new(100 + u32::from(first)),
        PredicateId::new(100 + u32::from(second)),
    ]))
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

/// Deterministic ordered temporal trend case.
///
/// Reversing the frames changes the structure while preserving the same predicate vocabulary.
#[must_use]
pub fn temporal_trend_case(case_id: u32) -> TemporalCase {
    let base = 30_000 + 3 * case_id;
    let frames = vec![
        BooleanState::new(vec![PredicateId::new(base)]),
        BooleanState::new(vec![PredicateId::new(base + 1)]),
        BooleanState::new(vec![PredicateId::new(base + 2)]),
    ];
    TemporalCase {
        query: BooleanSequence::new(frames),
        expected_template: TemplateId::new(80_000 + u64::from(case_id)),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        TaskFamily, context_reversal_pair, motif_ambiguous_state, motif_ood_state,
        motif_retrieval_case, motif_retrieval_stress_case, temporal_trend_case,
    };

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

    #[test]
    fn temporal_fixture_has_frozen_order() {
        let case = temporal_trend_case(4);
        assert_eq!(case.query().len(), 3);
        assert_ne!(case.query().frames()[0], case.query().frames()[2]);
    }

    #[test]
    fn stress_fixture_preserves_motif_while_adding_irrelevant_predicates() {
        let case = motif_retrieval_stress_case(5, 64);
        assert_eq!(
            case.expected_template(),
            motif_retrieval_case(5).expected_template()
        );
        assert_eq!(case.query().len(), 65);
    }

    #[test]
    fn ood_and_ambiguous_states_have_explicit_shapes() {
        assert_eq!(motif_ood_state(2, 8).len(), 8);
        assert_eq!(motif_ambiguous_state(0, 1).expect("valid pair").len(), 2);
        assert!(motif_ambiguous_state(1, 1).is_none());
    }
}
