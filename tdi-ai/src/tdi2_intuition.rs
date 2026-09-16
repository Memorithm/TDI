//! Experimental TDI-2.1 primitives for experience-conditioned intuition.
//!
//! This module implements research-only structures from the frozen TDI-2.1
//! programme. It does not modify historical TDI-2 results and carries no
//! confirmatory, performance, or production claim.

/// Stable identifier for a Boolean predicate extracted from a situation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PredicateId(u32);

impl PredicateId {
    /// Construct an identifier from its canonical integer representation.
    #[must_use]
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    /// Return the canonical integer representation.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// Stable identifier for one role appearing in a transferable template.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RoleId(u16);

impl RoleId {
    /// Construct a role identifier.
    #[must_use]
    pub const fn new(raw: u16) -> Self {
        Self(raw)
    }

    /// Return the canonical integer representation.
    #[must_use]
    pub const fn raw(self) -> u16 {
        self.0
    }
}

/// Stable identifier for one consolidated experiential template.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TemplateId(u64);

impl TemplateId {
    /// Construct a template identifier.
    #[must_use]
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// Return the canonical integer representation.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// Canonical set of predicates observed as true in one situation.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BooleanState {
    predicates: Vec<PredicateId>,
}

impl BooleanState {
    /// Build a canonical state by sorting and deduplicating predicates.
    #[must_use]
    pub fn new(mut predicates: Vec<PredicateId>) -> Self {
        predicates.sort_unstable();
        predicates.dedup();
        Self { predicates }
    }

    /// Return the canonical sorted predicate slice.
    #[must_use]
    pub fn predicates(&self) -> &[PredicateId] {
        &self.predicates
    }

    /// Test whether a predicate is active.
    #[must_use]
    pub fn contains(&self, predicate: PredicateId) -> bool {
        self.predicates.binary_search(&predicate).is_ok()
    }

    /// Number of active predicates.
    #[must_use]
    pub fn len(&self) -> usize {
        self.predicates.len()
    }

    /// Whether no predicate is active.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.predicates.is_empty()
    }
}

/// Validated continuous representation retained alongside Boolean structure.
#[derive(Clone, Debug, PartialEq)]
pub struct NumericState {
    values: Vec<f64>,
}

/// Validation failures in the TDI-2.1 intuition reference path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IntuitionError {
    /// A continuous feature was NaN or infinite.
    NonFiniteNumeric { index: usize },
}

impl core::fmt::Display for IntuitionError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NonFiniteNumeric { index } => {
                write!(formatter, "numeric intuition feature at index {index} is not finite")
            }
        }
    }
}

impl std::error::Error for IntuitionError {}

impl NumericState {
    /// Validate and construct the continuous state.
    pub fn new(values: Vec<f64>) -> Result<Self, IntuitionError> {
        if let Some(index) = values.iter().position(|value| !value.is_finite()) {
            return Err(IntuitionError::NonFiniteNumeric { index });
        }
        Ok(Self { values })
    }

    /// Continuous features in frozen order.
    #[must_use]
    pub fn values(&self) -> &[f64] {
        &self.values
    }

    /// Number of continuous features.
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Whether no continuous feature is present.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::{BooleanState, IntuitionError, NumericState, PredicateId};

    #[test]
    fn boolean_state_is_canonical() {
        let state = BooleanState::new(vec![
            PredicateId::new(7),
            PredicateId::new(2),
            PredicateId::new(7),
        ]);
        assert_eq!(
            state.predicates(),
            &[PredicateId::new(2), PredicateId::new(7)]
        );
        assert!(state.contains(PredicateId::new(7)));
        assert!(!state.contains(PredicateId::new(3)));
    }

    #[test]
    fn numeric_state_rejects_non_finite_values() {
        let error = NumericState::new(vec![0.0, f64::NAN]).expect_err("NaN must be rejected");
        assert_eq!(error, IntuitionError::NonFiniteNumeric { index: 1 });
    }
}
