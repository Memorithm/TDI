//! Novelty and experiential-coverage diagnostics for TDI-2.1.

use super::tdi2_intuition::BooleanState;
use super::tdi2_intuition_matching::match_template;
use super::tdi2_intuition_store::ExperienceStore;

/// Coverage summary for one Boolean situation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NoveltyDiagnostic {
    /// Number of templates satisfying every required/forbidden clause.
    pub exact_applicable: usize,
    /// Highest partial Boolean clause coverage across stored templates.
    pub best_partial_match: Option<f64>,
}

impl NoveltyDiagnostic {
    /// Whether no stored template is exactly applicable.
    #[must_use]
    pub const fn is_uncovered(self) -> bool {
        self.exact_applicable == 0
    }
}

/// Diagnose coverage without using empirical reliability or candidate ranking.
#[must_use]
pub fn diagnose_novelty(store: &ExperienceStore, state: &BooleanState) -> NoveltyDiagnostic {
    let mut exact_applicable = 0usize;
    let mut best_partial_match: Option<f64> = None;
    for entry in store.entries() {
        let matched = match_template(entry.template().base(), state);
        if matched.is_exact() {
            exact_applicable += 1;
        }
        let fraction = matched.score();
        best_partial_match = Some(best_partial_match.map_or(fraction, |current| current.max(fraction)));
    }
    NoveltyDiagnostic { exact_applicable, best_partial_match }
}

#[cfg(test)]
mod tests {
    use super::diagnose_novelty;
    use crate::experimental::tdi2_intuition::{BooleanState, PredicateId, Template, TemplateId};
    use crate::experimental::tdi2_intuition_relations::RelationalTemplate;
    use crate::experimental::tdi2_intuition_reliability::ReliabilityEvidence;
    use crate::experimental::tdi2_intuition_store::{ExperienceEntry, ExperienceStore};

    #[test]
    fn unseen_state_is_reported_as_uncovered() {
        let base = Template::new(
            TemplateId::new(1),
            vec![PredicateId::new(1)],
            Vec::new(),
            Vec::new(),
        )
        .expect("template");
        let mut store = ExperienceStore::new(2).expect("store");
        store
            .insert(ExperienceEntry::new(
                RelationalTemplate::new(base, Vec::new()).expect("relational"),
                ReliabilityEvidence::new(3, 0),
            ))
            .expect("insert");
        let diagnostic = diagnose_novelty(
            &store,
            &BooleanState::new(vec![PredicateId::new(99)]),
        );
        assert!(diagnostic.is_uncovered());
        assert_eq!(diagnostic.best_partial_match, Some(0.0));
    }
}
