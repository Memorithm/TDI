//! External operation accounting for TDI-2.1 reference experiments.
//!
//! These counters describe what an inference run did. They are measurements and
//! are never inputs to template applicability, ranking, transfer, or abstention.

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

#[cfg(test)]
mod tests {
    use super::OperationAccounting;

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
}
