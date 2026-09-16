//! External operation and resource accounting for TDI-2.1 reference experiments.
//!
//! These measurements are never inputs to template applicability, ranking,
//! transfer, consolidation, or abstention.

use std::time::{Duration, Instant};

use super::tdi2_intuition_store::ExperienceStore;

/// Logical operation counts for one inference run.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OperationAccounting {
    /// Templates inspected for Boolean applicability.
    pub templates_considered: u64,
    /// Individual required/forbidden clauses evaluated.
    pub boolean_clauses_checked: u64,
    /// Applicable templates whose experience strength was evaluated.
    pub weights_computed: u64,
    /// Candidates retained after structural filtering.
    pub candidates_selected: u64,
    /// Role relations materialized during structural transfer.
    pub relations_transferred: u64,
}

impl OperationAccounting {
    /// Canonical record suitable for experiment artifacts.
    #[must_use]
    pub fn canonical_record(self) -> String {
        format!(
            "tdi2.1-operation-accounting-v1;templates={};clauses={};weights={};candidates={};relations={}",
            self.templates_considered,
            self.boolean_clauses_checked,
            self.weights_computed,
            self.candidates_selected,
            self.relations_transferred
        )
    }

    /// Total logical operations represented by the declared categories.
    #[must_use]
    pub const fn declared_total(self) -> u64 {
        self.templates_considered
            .saturating_add(self.boolean_clauses_checked)
            .saturating_add(self.weights_computed)
            .saturating_add(self.candidates_selected)
            .saturating_add(self.relations_transferred)
    }
}

/// Logical memory footprint, deliberately not reported as physical bytes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LogicalMemoryAccounting {
    /// Stored experiential templates.
    pub templates: u64,
    /// Required plus forbidden Boolean clauses.
    pub boolean_clauses: u64,
    /// Declared abstract roles.
    pub roles: u64,
    /// Directed typed role relations.
    pub relations: u64,
    /// Explicit success/failure counters.
    pub reliability_counters: u64,
}

/// Count logical memory units without inferring allocator, cache, or resident-byte usage.
#[must_use]
pub fn logical_memory_accounting(store: &ExperienceStore) -> LogicalMemoryAccounting {
    let mut accounting = LogicalMemoryAccounting::default();
    for entry in store.entries() {
        accounting.templates += 1;
        accounting.boolean_clauses += (entry.template().base().required().len()
            + entry.template().base().forbidden().len()) as u64;
        accounting.roles += entry.template().base().roles().len() as u64;
        accounting.relations += entry.template().relations().len() as u64;
        accounting.reliability_counters += 2;
    }
    accounting
}

/// Result plus externally observed wall-clock duration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExternalTiming<T> {
    /// Unmodified function result.
    pub value: T,
    /// Wall-clock duration measured outside the algorithm.
    pub elapsed: Duration,
}

/// Measure one call externally.
///
/// The elapsed duration is returned alongside the result and is never provided
/// to the measured closure. Latency therefore remains an observed consequence,
/// not a decision variable of the intuition algorithm.
pub fn measure_external<T, F>(operation: F) -> ExternalTiming<T>
where
    F: FnOnce() -> T,
{
    let start = Instant::now();
    let value = operation();
    ExternalTiming {
        value,
        elapsed: start.elapsed(),
    }
}

#[cfg(test)]
mod tests {
    use super::{OperationAccounting, logical_memory_accounting, measure_external};
    use crate::experimental::tdi2_intuition::{PredicateId, RoleId, Template, TemplateId};
    use crate::experimental::tdi2_intuition_relations::{RelationId, RelationalTemplate, RoleRelation};
    use crate::experimental::tdi2_intuition_reliability::ReliabilityEvidence;
    use crate::experimental::tdi2_intuition_store::{ExperienceEntry, ExperienceStore};

    #[test]
    fn operation_record_is_stable_and_additive() {
        let accounting = OperationAccounting {
            templates_considered: 3,
            boolean_clauses_checked: 7,
            weights_computed: 2,
            candidates_selected: 2,
            relations_transferred: 1,
        };
        assert_eq!(accounting.declared_total(), 15);
        assert!(accounting.canonical_record().contains("templates=3"));
    }

    #[test]
    fn logical_memory_accounting_does_not_claim_bytes() {
        let base = Template::new(
            TemplateId::new(1),
            vec![PredicateId::new(1), PredicateId::new(2)],
            Vec::new(),
            vec![RoleId::new(1), RoleId::new(2)],
        )
        .expect("template");
        let relational = RelationalTemplate::new(
            base,
            vec![RoleRelation::new(RoleId::new(1), RelationId::new(7), RoleId::new(2))],
        )
        .expect("relations");
        let mut store = ExperienceStore::new(2).expect("store");
        store
            .insert(ExperienceEntry::new(relational, ReliabilityEvidence::new(3, 1)))
            .expect("insert");
        let accounting = logical_memory_accounting(&store);
        assert_eq!(accounting.templates, 1);
        assert_eq!(accounting.boolean_clauses, 2);
        assert_eq!(accounting.roles, 2);
        assert_eq!(accounting.relations, 1);
        assert_eq!(accounting.reliability_counters, 2);
    }

    #[test]
    fn timing_observer_does_not_change_return_value() {
        let measured = measure_external(|| 42_u64);
        assert_eq!(measured.value, 42);
    }
}
