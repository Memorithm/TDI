//! Reusable ordered Boolean templates for TDI-2.1 temporal transfer.
use super::tdi2_intuition::{PredicateId, TemplateId};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemporalClause {
    required: Vec<PredicateId>,
    forbidden: Vec<PredicateId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemporalPattern {
    id: TemplateId,
    frames: Vec<TemporalClause>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemporalTransferError {
    EmptyClause,
    ContradictoryPredicate { predicate: PredicateId },
    EmptyPattern,
}

impl core::fmt::Display for TemporalTransferError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyClause => formatter.write_str("temporal transfer clause must not be empty"),
            Self::ContradictoryPredicate { predicate } => write!(
                formatter,
                "temporal predicate {} is required and forbidden",
                predicate.raw()
            ),
            Self::EmptyPattern => {
                formatter.write_str("temporal transfer pattern must contain frames")
            }
        }
    }
}
impl std::error::Error for TemporalTransferError {}

impl TemporalClause {
    pub fn new(
        mut required: Vec<PredicateId>,
        mut forbidden: Vec<PredicateId>,
    ) -> Result<Self, TemporalTransferError> {
        required.sort_unstable();
        required.dedup();
        forbidden.sort_unstable();
        forbidden.dedup();
        if required.is_empty() && forbidden.is_empty() {
            return Err(TemporalTransferError::EmptyClause);
        }
        if let Some(predicate) = required
            .iter()
            .copied()
            .find(|p| forbidden.binary_search(p).is_ok())
        {
            return Err(TemporalTransferError::ContradictoryPredicate { predicate });
        }
        Ok(Self {
            required,
            forbidden,
        })
    }
    #[must_use]
    pub fn required(&self) -> &[PredicateId] {
        &self.required
    }
    #[must_use]
    pub fn forbidden(&self) -> &[PredicateId] {
        &self.forbidden
    }
}

impl TemporalPattern {
    pub fn new(id: TemplateId, frames: Vec<TemporalClause>) -> Result<Self, TemporalTransferError> {
        if frames.is_empty() {
            return Err(TemporalTransferError::EmptyPattern);
        }
        Ok(Self { id, frames })
    }
    #[must_use]
    pub const fn id(&self) -> TemplateId {
        self.id
    }
    #[must_use]
    pub fn frames(&self) -> &[TemporalClause] {
        &self.frames
    }
}

#[cfg(test)]
mod tests {
    use super::{TemporalClause, TemporalPattern, TemporalTransferError};
    use crate::experimental::tdi2_intuition::{PredicateId, TemplateId};
    #[test]
    fn temporal_pattern_rejects_empty_or_contradictory_structure() {
        assert_eq!(
            TemporalClause::new(Vec::new(), Vec::new()),
            Err(TemporalTransferError::EmptyClause)
        );
        assert!(matches!(
            TemporalClause::new(vec![PredicateId::new(1)], vec![PredicateId::new(1)]),
            Err(TemporalTransferError::ContradictoryPredicate { .. })
        ));
        assert_eq!(
            TemporalPattern::new(TemplateId::new(1), Vec::new()),
            Err(TemporalTransferError::EmptyPattern)
        );
    }
}

use super::tdi2_intuition_temporal::BooleanSequence;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TemporalPatternMatch {
    pub exact: bool,
    pub satisfied_clauses: usize,
    pub total_clauses: usize,
}

#[must_use]
pub fn match_temporal_pattern(
    pattern: &TemporalPattern,
    query: &BooleanSequence,
) -> TemporalPatternMatch {
    let mut satisfied = 0usize;
    let mut total = 0usize;
    if pattern.frames().len() != query.len() {
        return TemporalPatternMatch {
            exact: false,
            satisfied_clauses: 0,
            total_clauses: pattern
                .frames()
                .iter()
                .map(|f| f.required().len() + f.forbidden().len())
                .sum(),
        };
    }
    for (clause, state) in pattern.frames().iter().zip(query.frames()) {
        satisfied += clause
            .required()
            .iter()
            .filter(|p| state.contains(**p))
            .count();
        satisfied += clause
            .forbidden()
            .iter()
            .filter(|p| !state.contains(**p))
            .count();
        total += clause.required().len() + clause.forbidden().len();
    }
    TemporalPatternMatch {
        exact: satisfied == total,
        satisfied_clauses: satisfied,
        total_clauses: total,
    }
}

#[cfg(test)]
mod matching_tests {
    use super::{TemporalClause, TemporalPattern, match_temporal_pattern};
    use crate::experimental::tdi2_intuition::{BooleanState, PredicateId, TemplateId};
    use crate::experimental::tdi2_intuition_temporal::BooleanSequence;
    #[test]
    fn temporal_pattern_ignores_distractors_but_preserves_order() {
        let pattern = TemporalPattern::new(
            TemplateId::new(1),
            vec![
                TemporalClause::new(vec![PredicateId::new(1)], Vec::new()).unwrap(),
                TemporalClause::new(vec![PredicateId::new(2)], Vec::new()).unwrap(),
            ],
        )
        .unwrap();
        let query = BooleanSequence::new(vec![
            BooleanState::new(vec![PredicateId::new(1), PredicateId::new(99)]),
            BooleanState::new(vec![PredicateId::new(2), PredicateId::new(98)]),
        ]);
        assert!(match_temporal_pattern(&pattern, &query).exact);
        let reversed = BooleanSequence::new(query.frames().iter().cloned().rev().collect());
        assert!(!match_temporal_pattern(&pattern, &reversed).exact);
    }
}
