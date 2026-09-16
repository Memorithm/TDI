//! Boolean structural matching for the experimental TDI-2.1 intuition path.

use super::tdi2_intuition::{BooleanState, Template};

/// Exact applicability plus an interpretable partial-match diagnostic.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TemplateMatch {
    exact: bool,
    satisfied_clauses: usize,
    total_clauses: usize,
}

impl TemplateMatch {
    /// Whether every required and forbidden Boolean clause is satisfied.
    #[must_use]
    pub const fn is_exact(self) -> bool {
        self.exact
    }

    /// Number of individually satisfied Boolean clauses.
    #[must_use]
    pub const fn satisfied_clauses(self) -> usize {
        self.satisfied_clauses
    }

    /// Total number of Boolean clauses in the template.
    #[must_use]
    pub const fn total_clauses(self) -> usize {
        self.total_clauses
    }

    /// Fraction of Boolean clauses satisfied.
    ///
    /// This is a diagnostic over crisp Boolean clauses, not fuzzy logic.
    #[must_use]
    pub fn score(self) -> f64 {
        self.satisfied_clauses as f64 / self.total_clauses as f64
    }
}

/// Match one canonical Boolean state against one validated template.
#[must_use]
pub fn match_template(template: &Template, state: &BooleanState) -> TemplateMatch {
    let required_ok = template
        .required()
        .iter()
        .filter(|predicate| state.contains(**predicate))
        .count();
    let forbidden_ok = template
        .forbidden()
        .iter()
        .filter(|predicate| !state.contains(**predicate))
        .count();
    let satisfied_clauses = required_ok + forbidden_ok;
    let total_clauses = template.required().len() + template.forbidden().len();

    TemplateMatch {
        exact: satisfied_clauses == total_clauses,
        satisfied_clauses,
        total_clauses,
    }
}

#[cfg(test)]
mod tests {
    use super::match_template;
    use crate::experimental::tdi2_intuition::{
        BooleanState, PredicateId, Template, TemplateId,
    };

    fn template() -> Template {
        Template::new(
            TemplateId::new(11),
            vec![PredicateId::new(1), PredicateId::new(2)],
            vec![PredicateId::new(9)],
            Vec::new(),
        )
        .expect("valid template")
    }

    #[test]
    fn exact_match_requires_all_crisp_clauses() {
        let state = BooleanState::new(vec![PredicateId::new(1), PredicateId::new(2)]);
        let result = match_template(&template(), &state);
        assert!(result.is_exact());
        assert_eq!(result.satisfied_clauses(), 3);
        assert_eq!(result.score(), 1.0);
    }

    #[test]
    fn partial_match_reports_boolean_clause_fraction() {
        let state = BooleanState::new(vec![PredicateId::new(1), PredicateId::new(9)]);
        let result = match_template(&template(), &state);
        assert!(!result.is_exact());
        assert_eq!(result.satisfied_clauses(), 1);
        assert_eq!(result.total_clauses(), 3);
        assert!((result.score() - (1.0 / 3.0)).abs() < f64::EPSILON);
    }
}
