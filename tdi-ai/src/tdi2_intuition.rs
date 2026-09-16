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

/// Validated structural template distilled from accumulated experience.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Template {
    id: TemplateId,
    required: Vec<PredicateId>,
    forbidden: Vec<PredicateId>,
    roles: Vec<RoleId>,
}

/// Validation failures in the TDI-2.1 intuition reference path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IntuitionError {
    /// A continuous feature was NaN or infinite.
    NonFiniteNumeric { index: usize },
    /// A template carried no Boolean structural condition.
    EmptyTemplate,
    /// One predicate was simultaneously required and forbidden.
    ContradictoryPredicate { predicate: PredicateId },
    /// A role occurred more than once in a template role declaration.
    DuplicateRole { role: RoleId },
}

impl core::fmt::Display for IntuitionError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NonFiniteNumeric { index } => {
                write!(
                    formatter,
                    "numeric intuition feature at index {index} is not finite"
                )
            }
            Self::EmptyTemplate => {
                formatter.write_str("intuition template has no Boolean condition")
            }
            Self::ContradictoryPredicate { predicate } => write!(
                formatter,
                "predicate {} is both required and forbidden",
                predicate.raw()
            ),
            Self::DuplicateRole { role } => {
                write!(formatter, "template role {} is duplicated", role.raw())
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

impl Template {
    /// Validate and construct one structural template.
    pub fn new(
        id: TemplateId,
        mut required: Vec<PredicateId>,
        mut forbidden: Vec<PredicateId>,
        mut roles: Vec<RoleId>,
    ) -> Result<Self, IntuitionError> {
        if required.is_empty() && forbidden.is_empty() {
            return Err(IntuitionError::EmptyTemplate);
        }

        required.sort_unstable();
        required.dedup();
        forbidden.sort_unstable();
        forbidden.dedup();

        if let Some(predicate) = required
            .iter()
            .copied()
            .find(|predicate| forbidden.binary_search(predicate).is_ok())
        {
            return Err(IntuitionError::ContradictoryPredicate { predicate });
        }

        roles.sort_unstable();
        if let Some(role) = roles
            .windows(2)
            .find_map(|pair| (pair[0] == pair[1]).then_some(pair[0]))
        {
            return Err(IntuitionError::DuplicateRole { role });
        }

        Ok(Self {
            id,
            required,
            forbidden,
            roles,
        })
    }

    /// Stable template identifier.
    #[must_use]
    pub const fn id(&self) -> TemplateId {
        self.id
    }

    /// Predicates that must be present for exact applicability.
    #[must_use]
    pub fn required(&self) -> &[PredicateId] {
        &self.required
    }

    /// Predicates that must be absent for exact applicability.
    #[must_use]
    pub fn forbidden(&self) -> &[PredicateId] {
        &self.forbidden
    }

    /// Abstract roles used by later transfer operations.
    #[must_use]
    pub fn roles(&self) -> &[RoleId] {
        &self.roles
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BooleanState, IntuitionError, NumericState, PredicateId, RoleId, Template, TemplateId,
    };

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

    #[test]
    fn template_rejects_boolean_contradictions() {
        let error = Template::new(
            TemplateId::new(1),
            vec![PredicateId::new(4)],
            vec![PredicateId::new(4)],
            vec![RoleId::new(0)],
        )
        .expect_err("contradictory template must fail closed");
        assert_eq!(
            error,
            IntuitionError::ContradictoryPredicate {
                predicate: PredicateId::new(4)
            }
        );
    }

    #[test]
    fn template_canonicalizes_conditions() {
        let template = Template::new(
            TemplateId::new(9),
            vec![
                PredicateId::new(3),
                PredicateId::new(1),
                PredicateId::new(3),
            ],
            vec![PredicateId::new(8)],
            vec![RoleId::new(2), RoleId::new(1)],
        )
        .expect("valid template");
        assert_eq!(
            template.required(),
            &[PredicateId::new(1), PredicateId::new(3)]
        );
        assert_eq!(template.roles(), &[RoleId::new(1), RoleId::new(2)]);
    }
}
