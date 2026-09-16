//! Deterministic candidate selection for the experimental TDI-2.1 intuition path.

use super::tdi2_intuition::{BooleanState, TemplateId};
use super::tdi2_intuition_matching::match_template;
use super::tdi2_intuition_reliability::ReliabilityError;
use super::tdi2_intuition_store::ExperienceStore;
use super::tdi2_intuition_weight::ExperienceWeightPolicy;

/// One structurally applicable experiential candidate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Candidate {
    template_id: TemplateId,
    weight: f64,
    support: u64,
}

impl Candidate {
    /// Construct an already-evaluated candidate value.
    ///
    /// Normal engine operation obtains candidates through [`select_candidates`].
    /// This constructor exists for composition and deterministic reference tests.
    #[must_use]
    pub const fn new(template_id: TemplateId, weight: f64, support: u64) -> Self {
        Self {
            template_id,
            weight,
            support,
        }
    }

    /// Stable template identifier.
    #[must_use]
    pub const fn template_id(self) -> TemplateId {
        self.template_id
    }

    /// Numeric experience strength.
    #[must_use]
    pub const fn weight(self) -> f64 {
        self.weight
    }

    /// Number of empirically validated applications.
    #[must_use]
    pub const fn support(self) -> u64 {
        self.support
    }
}

/// Collect exact Boolean matches and rank them by empirical experience strength.
///
/// Boolean applicability is evaluated first. Numeric evidence cannot rescue a
/// structurally inapplicable template. Ties are resolved by ascending template id.
pub fn select_candidates(
    store: &ExperienceStore,
    state: &BooleanState,
    policy: ExperienceWeightPolicy,
) -> Result<Vec<Candidate>, ReliabilityError> {
    let mut candidates = Vec::new();

    for entry in store.entries() {
        if !match_template(entry.template().base(), state).is_exact() {
            continue;
        }
        candidates.push(Candidate::new(
            entry.template().base().id(),
            policy.weight(entry.evidence())?,
            entry.evidence().support(),
        ));
    }

    candidates.sort_by(|left, right| {
        right
            .weight
            .total_cmp(&left.weight)
            .then_with(|| left.template_id.cmp(&right.template_id))
    });
    Ok(candidates)
}

#[cfg(test)]
mod tests {
    use super::select_candidates;
    use crate::experimental::tdi2_intuition::{BooleanState, PredicateId, Template, TemplateId};
    use crate::experimental::tdi2_intuition_relations::RelationalTemplate;
    use crate::experimental::tdi2_intuition_reliability::ReliabilityEvidence;
    use crate::experimental::tdi2_intuition_store::{ExperienceEntry, ExperienceStore};
    use crate::experimental::tdi2_intuition_weight::ExperienceWeightPolicy;

    fn entry(id: u64, predicate: u32, successes: u64, failures: u64) -> ExperienceEntry {
        let base = Template::new(
            TemplateId::new(id),
            vec![PredicateId::new(predicate)],
            Vec::new(),
            Vec::new(),
        )
        .expect("valid template");
        ExperienceEntry::new(
            RelationalTemplate::new(base, Vec::new()).expect("valid relational template"),
            ReliabilityEvidence::new(successes, failures),
        )
    }

    #[test]
    fn inapplicable_templates_never_enter_ranking() {
        let mut store = ExperienceStore::new(3).expect("valid store");
        store.insert(entry(1, 1, 2, 0)).expect("insert");
        store.insert(entry(2, 2, 100, 0)).expect("insert");
        let state = BooleanState::new(vec![PredicateId::new(1)]);
        let candidates = select_candidates(&store, &state, ExperienceWeightPolicy::default())
            .expect("valid evidence");
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].template_id(), TemplateId::new(1));
    }

    #[test]
    fn stronger_experience_ranks_first() {
        let mut store = ExperienceStore::new(3).expect("valid store");
        store.insert(entry(1, 1, 2, 0)).expect("insert");
        store.insert(entry(2, 1, 20, 0)).expect("insert");
        let state = BooleanState::new(vec![PredicateId::new(1)]);
        let candidates = select_candidates(&store, &state, ExperienceWeightPolicy::default())
            .expect("valid evidence");
        assert_eq!(candidates[0].template_id(), TemplateId::new(2));
        assert!(candidates[0].weight() > candidates[1].weight());
    }
}
