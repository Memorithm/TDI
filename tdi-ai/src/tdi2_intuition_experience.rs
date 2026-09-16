//! Deterministic experience fixtures for non-final TDI-2.1 campaigns.
//!
//! These builders encode the task generator's public structural rules only.
//! They never inspect an evaluation case's expected label during inference.

use super::tdi2_intuition::{PredicateId, Template, TemplateId};
use super::tdi2_intuition_relations::RelationalTemplate;
use super::tdi2_intuition_reliability::ReliabilityEvidence;
use super::tdi2_intuition_store::{ExperienceEntry, ExperienceStore};

/// Number of reusable motif classes in the frozen motif-retrieval generator.
pub const MOTIF_CLASS_COUNT: usize = 17;

/// Build one reusable template per motif class.
///
/// Evaluation cases may carry arbitrary novel distractor predicates; only the
/// invariant motif predicate is stored. `successes_per_motif` is explicit
/// accumulated empirical support and is never inferred from evaluation labels.
#[must_use]
pub fn motif_experience_store(successes_per_motif: u64) -> ExperienceStore {
    let mut store = ExperienceStore::new(MOTIF_CLASS_COUNT).expect("fixed positive capacity");
    for index in 0..MOTIF_CLASS_COUNT {
        let template = Template::new(
            TemplateId::new(index as u64 + 1),
            vec![PredicateId::new(100 + index as u32)],
            Vec::new(),
            Vec::new(),
        )
        .expect("fixed motif template is valid");
        let relational = RelationalTemplate::new(template, Vec::new())
            .expect("fixed motif template has no invalid relations");
        store
            .insert(ExperienceEntry::new(
                relational,
                ReliabilityEvidence::new(successes_per_motif, 0),
            ))
            .expect("fixed motif ids are unique and fit capacity");
    }
    store
}

#[cfg(test)]
mod tests {
    use super::{MOTIF_CLASS_COUNT, motif_experience_store};
    use crate::experimental::tdi2_intuition::TemplateId;

    #[test]
    fn motif_store_contains_exactly_one_template_per_class() {
        let store = motif_experience_store(3);
        assert_eq!(store.len(), MOTIF_CLASS_COUNT);
        assert_eq!(store.get(TemplateId::new(1)).expect("template").evidence().successes(), 3);
        assert_eq!(store.get(TemplateId::new(17)).expect("template").evidence().support(), 3);
    }
}
