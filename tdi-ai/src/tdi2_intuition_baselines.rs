//! Matched development/validation baselines for TDI-2.1 intuition experiments.

use super::tdi2_intuition::{BooleanState, TemplateId};
use super::tdi2_intuition_matching::match_template;
use super::tdi2_intuition_store::ExperienceStore;
use super::tdi2_intuition_tasks::SyntheticCase;

/// B0: a system with no usable experience must abstain.
#[must_use]
pub const fn no_experience_baseline() -> Option<TemplateId> {
    None
}

/// B1: deterministic invalid-experience control.
///
/// The mapping is deliberately wrong but reproducible. It never aliases the
/// expected template because a non-zero high bit is toggled.
#[must_use]
pub fn invalid_experience_target(case: &SyntheticCase) -> TemplateId {
    TemplateId::new(case.expected_template().raw() ^ (1_u64 << 63))
}

/// B2 result: nearest Boolean-clause template, ignoring empirical reliability
/// and relational transfer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NearestBooleanResult {
    template_id: TemplateId,
    score: f64,
}

impl NearestBooleanResult {
    /// Selected template identifier.
    #[must_use]
    pub const fn template_id(self) -> TemplateId {
        self.template_id
    }

    /// Fraction of crisp Boolean clauses satisfied.
    #[must_use]
    pub const fn score(self) -> f64 {
        self.score
    }
}

/// B2: choose the template with greatest crisp-clause overlap.
///
/// Reliability, support and relation mappings are intentionally ignored.
#[must_use]
pub fn nearest_boolean_baseline(
    store: &ExperienceStore,
    state: &BooleanState,
) -> Option<NearestBooleanResult> {
    store
        .entries()
        .iter()
        .map(|entry| {
            let matched = match_template(entry.template().base(), state);
            NearestBooleanResult {
                template_id: entry.template().base().id(),
                score: matched.score(),
            }
        })
        .max_by(|left, right| {
            left.score
                .total_cmp(&right.score)
                .then_with(|| right.template_id.cmp(&left.template_id))
        })
}

#[cfg(test)]
mod tests {
    use super::{invalid_experience_target, nearest_boolean_baseline, no_experience_baseline};
    use crate::experimental::tdi2_intuition::{BooleanState, PredicateId, Template, TemplateId};
    use crate::experimental::tdi2_intuition_relations::RelationalTemplate;
    use crate::experimental::tdi2_intuition_reliability::ReliabilityEvidence;
    use crate::experimental::tdi2_intuition_store::{ExperienceEntry, ExperienceStore};
    use crate::experimental::tdi2_intuition_tasks::motif_retrieval_case;

    fn entry(id: u64, required: Vec<u32>) -> ExperienceEntry {
        let base = Template::new(
            TemplateId::new(id),
            required.into_iter().map(PredicateId::new).collect(),
            Vec::new(),
            Vec::new(),
        )
        .expect("valid template");
        ExperienceEntry::new(
            RelationalTemplate::new(base, Vec::new()).expect("valid relational template"),
            ReliabilityEvidence::new(100, 0),
        )
    }

    #[test]
    fn b0_always_abstains() {
        assert_eq!(no_experience_baseline(), None);
    }

    #[test]
    fn b1_never_returns_the_frozen_expected_target() {
        let case = motif_retrieval_case(4);
        assert_ne!(invalid_experience_target(&case), case.expected_template());
    }

    #[test]
    fn b2_ignores_reliability_and_uses_clause_overlap() {
        let mut store = ExperienceStore::new(2).expect("valid store");
        store.insert(entry(1, vec![1, 2])).expect("insert");
        store.insert(entry(2, vec![1, 3])).expect("insert");
        let state = BooleanState::new(vec![PredicateId::new(1), PredicateId::new(2)]);
        let result = nearest_boolean_baseline(&store, &state).expect("candidate exists");
        assert_eq!(result.template_id(), TemplateId::new(1));
        assert_eq!(result.score(), 1.0);
    }
}
