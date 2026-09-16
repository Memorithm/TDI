//! Bounded experiential-template storage for the experimental TDI-2.1 path.
//!
//! The store is deliberately read-only during inference. Consolidation is an
//! explicit later operation so evaluation cannot silently mutate its evidence.

use super::tdi2_intuition::TemplateId;
use super::tdi2_intuition_relations::RelationalTemplate;
use super::tdi2_intuition_reliability::ReliabilityEvidence;

/// One consolidated experiential template and its empirical validation record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExperienceEntry {
    template: RelationalTemplate,
    evidence: ReliabilityEvidence,
}

impl ExperienceEntry {
    /// Construct one stored experience entry.
    #[must_use]
    pub const fn new(template: RelationalTemplate, evidence: ReliabilityEvidence) -> Self {
        Self { template, evidence }
    }

    /// Structural template carried by this entry.
    #[must_use]
    pub const fn template(&self) -> &RelationalTemplate {
        &self.template
    }

    /// Historical success/failure evidence.
    #[must_use]
    pub const fn evidence(&self) -> ReliabilityEvidence {
        self.evidence
    }
}

/// Deterministically ordered bounded memory of experiential templates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExperienceStore {
    capacity: usize,
    entries: Vec<ExperienceEntry>,
}

/// Fail-closed storage errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoreError {
    /// Capacity must be strictly positive.
    ZeroCapacity,
    /// One template identifier already exists in the store.
    DuplicateTemplate { template: TemplateId },
    /// The declared store capacity has been reached.
    Full { capacity: usize },
}

impl core::fmt::Display for StoreError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ZeroCapacity => formatter.write_str("experience store capacity must be positive"),
            Self::DuplicateTemplate { template } => {
                write!(formatter, "template {} already exists", template.raw())
            }
            Self::Full { capacity } => {
                write!(formatter, "experience store capacity {capacity} reached")
            }
        }
    }
}

impl std::error::Error for StoreError {}

impl ExperienceStore {
    /// Create an empty bounded store.
    pub fn new(capacity: usize) -> Result<Self, StoreError> {
        if capacity == 0 {
            return Err(StoreError::ZeroCapacity);
        }
        Ok(Self {
            capacity,
            entries: Vec::with_capacity(capacity),
        })
    }

    /// Insert an entry while preserving ascending template-id order.
    pub fn insert(&mut self, entry: ExperienceEntry) -> Result<(), StoreError> {
        let id = entry.template().base().id();
        match self
            .entries
            .binary_search_by_key(&id, |candidate| candidate.template().base().id())
        {
            Ok(_) => return Err(StoreError::DuplicateTemplate { template: id }),
            Err(_) if self.entries.len() == self.capacity => {
                return Err(StoreError::Full {
                    capacity: self.capacity,
                });
            }
            Err(index) => self.entries.insert(index, entry),
        }
        Ok(())
    }

    /// Retrieve one entry without mutating memory.
    #[must_use]
    pub fn get(&self, id: TemplateId) -> Option<&ExperienceEntry> {
        self.entries
            .binary_search_by_key(&id, |candidate| candidate.template().base().id())
            .ok()
            .map(|index| &self.entries[index])
    }

    /// Canonically ordered entries.
    #[must_use]
    pub fn entries(&self) -> &[ExperienceEntry] {
        &self.entries
    }

    /// Declared maximum number of entries.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    /// Current number of entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether no experience is currently stored.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::{ExperienceEntry, ExperienceStore, StoreError};
    use crate::experimental::tdi2_intuition::{PredicateId, Template, TemplateId};
    use crate::experimental::tdi2_intuition_relations::RelationalTemplate;
    use crate::experimental::tdi2_intuition_reliability::ReliabilityEvidence;

    fn entry(id: u64) -> ExperienceEntry {
        let base = Template::new(
            TemplateId::new(id),
            vec![PredicateId::new(id as u32 + 1)],
            Vec::new(),
            Vec::new(),
        )
        .expect("valid template");
        ExperienceEntry::new(
            RelationalTemplate::new(base, Vec::new()).expect("valid relational template"),
            ReliabilityEvidence::new(1, 0),
        )
    }

    #[test]
    fn store_orders_entries_and_supports_read_only_lookup() {
        let mut store = ExperienceStore::new(3).expect("positive capacity");
        store.insert(entry(9)).expect("insert");
        store.insert(entry(2)).expect("insert");
        assert_eq!(store.entries()[0].template().base().id(), TemplateId::new(2));
        assert!(store.get(TemplateId::new(9)).is_some());
        assert_eq!(store.len(), 2);
    }

    #[test]
    fn store_rejects_duplicate_and_over_capacity_entries() {
        let mut store = ExperienceStore::new(1).expect("positive capacity");
        store.insert(entry(1)).expect("insert");
        assert_eq!(
            store.insert(entry(1)),
            Err(StoreError::DuplicateTemplate {
                template: TemplateId::new(1)
            })
        );
        assert_eq!(store.insert(entry(2)), Err(StoreError::Full { capacity: 1 }));
    }
}
