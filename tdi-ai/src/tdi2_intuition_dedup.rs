//! Structural duplicate diagnostics for TDI-2.1 experiential memory.

use super::tdi2_intuition::TemplateId;
use super::tdi2_intuition_store::ExperienceStore;
use super::tdi2_intuition_trace::template_structure_record;

/// Pair of distinct ids carrying the same structural template.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DuplicatePair {
    /// Lower template id.
    pub left: TemplateId,
    /// Higher template id.
    pub right: TemplateId,
}

/// Enumerate structural duplicates without mutating memory.
#[must_use]
pub fn structural_duplicates(store: &ExperienceStore) -> Vec<DuplicatePair> {
    let entries = store.entries();
    let records = entries
        .iter()
        .map(|entry| template_structure_record(entry.template()))
        .collect::<Vec<_>>();
    let mut duplicates = Vec::new();
    for left in 0..entries.len() {
        for right in (left + 1)..entries.len() {
            if records[left] == records[right] {
                let mut ids = [
                    entries[left].template().base().id(),
                    entries[right].template().base().id(),
                ];
                ids.sort_unstable();
                duplicates.push(DuplicatePair {
                    left: ids[0],
                    right: ids[1],
                });
            }
        }
    }
    duplicates.sort_unstable_by_key(|pair| (pair.left, pair.right));
    duplicates
}

#[cfg(test)]
mod tests {
    use super::{DuplicatePair, structural_duplicates};
    use crate::experimental::tdi2_intuition::{PredicateId, Template, TemplateId};
    use crate::experimental::tdi2_intuition_relations::RelationalTemplate;
    use crate::experimental::tdi2_intuition_reliability::ReliabilityEvidence;
    use crate::experimental::tdi2_intuition_store::{ExperienceEntry, ExperienceStore};

    fn entry(id: u64) -> ExperienceEntry {
        let base = Template::new(
            TemplateId::new(id),
            vec![PredicateId::new(1)],
            Vec::new(),
            Vec::new(),
        )
        .expect("template");
        ExperienceEntry::new(
            RelationalTemplate::new(base, Vec::new()).expect("relational"),
            ReliabilityEvidence::new(1, 0),
        )
    }

    #[test]
    fn duplicate_structure_is_reported_without_merging() {
        let mut store = ExperienceStore::new(3).expect("store");
        store.insert(entry(9)).expect("insert");
        store.insert(entry(2)).expect("insert");
        assert_eq!(
            structural_duplicates(&store),
            vec![DuplicatePair {
                left: TemplateId::new(2),
                right: TemplateId::new(9)
            }]
        );
    }
}
