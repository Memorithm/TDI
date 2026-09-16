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
    motif_experience_store_with_evidence(successes_per_motif, 0)
}

/// Build the motif memory with explicit success/failure evidence per class.
#[must_use]
pub fn motif_experience_store_with_evidence(
    successes_per_motif: u64,
    failures_per_motif: u64,
) -> ExperienceStore {
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
                ReliabilityEvidence::new(successes_per_motif, failures_per_motif),
            ))
            .expect("fixed motif ids are unique and fit capacity");
    }
    store
}

/// Build 26 reusable context templates: 13 base classes times two contexts.
#[must_use]
pub fn context_experience_store(successes_per_template: u64) -> ExperienceStore {
    let mut store = ExperienceStore::new(26).expect("fixed positive capacity");
    for class in 0..13_u32 {
        for side in 0..2_u32 {
            let template = Template::new(
                TemplateId::new(200_000 + u64::from(2 * class + side)),
                vec![
                    PredicateId::new(2_000 + class),
                    PredicateId::new(30_000 + side),
                ],
                Vec::new(),
                Vec::new(),
            )
            .expect("fixed context template is valid");
            store
                .insert(ExperienceEntry::new(
                    RelationalTemplate::new(template, Vec::new()).expect("no invalid relations"),
                    ReliabilityEvidence::new(successes_per_template, 0),
                ))
                .expect("fixed context ids are unique and fit capacity");
        }
    }
    store
}

#[cfg(test)]
mod tests {
    use super::{
        MOTIF_CLASS_COUNT, context_experience_store, motif_experience_store,
        motif_experience_store_with_evidence,
    };
    use crate::experimental::tdi2_intuition::TemplateId;

    #[test]
    fn motif_store_contains_exactly_one_template_per_class() {
        let store = motif_experience_store(3);
        assert_eq!(store.len(), MOTIF_CLASS_COUNT);
        assert_eq!(
            store
                .get(TemplateId::new(1))
                .expect("template")
                .evidence()
                .successes(),
            3
        );
        assert_eq!(
            store
                .get(TemplateId::new(17))
                .expect("template")
                .evidence()
                .support(),
            3
        );
    }

    #[test]
    fn motif_store_preserves_failure_evidence() {
        let store = motif_experience_store_with_evidence(7, 3);
        let evidence = store.get(TemplateId::new(1)).expect("template").evidence();
        assert_eq!(evidence.successes(), 7);
        assert_eq!(evidence.failures(), 3);
        assert_eq!(evidence.support(), 10);
    }

    #[test]
    fn context_store_contains_all_base_context_combinations() {
        let store = context_experience_store(5);
        assert_eq!(store.len(), 26);
        assert_eq!(
            store
                .get(TemplateId::new(200_000))
                .expect("template")
                .evidence()
                .support(),
            5
        );
        assert!(store.get(TemplateId::new(200_025)).is_some());
    }
}
