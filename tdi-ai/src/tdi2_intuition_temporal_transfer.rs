//! Reusable ordered Boolean templates for TDI-2.1 temporal transfer.
use super::tdi2_intuition::{BooleanState, PredicateId, TemplateId};

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

/// Frozen Development start for reusable temporal cases.
pub const TEMPORAL_TRANSFER_DEVELOPMENT_START: u32 = 5_000;
/// Frozen Validation start for reusable temporal cases.
pub const TEMPORAL_TRANSFER_VALIDATION_START: u32 = 6_000;
/// Number of cases per non-final temporal-transfer domain.
pub const TEMPORAL_TRANSFER_DOMAIN_CASES: usize = 33;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransferableTemporalCase {
    query: BooleanSequence,
    expected_template: TemplateId,
}

impl TransferableTemporalCase {
    #[must_use]
    pub const fn query(&self) -> &BooleanSequence {
        &self.query
    }
    #[must_use]
    pub const fn expected_template(&self) -> TemplateId {
        self.expected_template
    }
}

fn temporal_semantics(class: u32) -> [PredicateId; 3] {
    match class {
        0 => [
            PredicateId::new(60_000),
            PredicateId::new(60_001),
            PredicateId::new(60_002),
        ],
        1 => [
            PredicateId::new(60_002),
            PredicateId::new(60_001),
            PredicateId::new(60_000),
        ],
        _ => [
            PredicateId::new(60_000),
            PredicateId::new(60_003),
            PredicateId::new(60_002),
        ],
    }
}

/// Three reusable temporal patterns whose semantics are independent of case ids.
#[must_use]
pub fn reusable_temporal_patterns() -> Vec<TemporalPattern> {
    (0..3_u32)
        .map(|class| {
            let frames = temporal_semantics(class)
                .into_iter()
                .map(|predicate| {
                    TemporalClause::new(vec![predicate], Vec::new()).expect("fixed clause")
                })
                .collect();
            TemporalPattern::new(TemplateId::new(300_000 + u64::from(class)), frames)
                .expect("fixed temporal pattern")
        })
        .collect()
}

/// Novel-identity temporal case with one irrelevant predicate per frame.
#[must_use]
pub fn transferable_temporal_case(case_id: u32) -> TransferableTemporalCase {
    let class = case_id % 3;
    let block = 700_000 + (case_id % 10_000) * 4;
    let frames = temporal_semantics(class)
        .into_iter()
        .enumerate()
        .map(|(frame, semantic)| {
            BooleanState::new(vec![semantic, PredicateId::new(block + frame as u32)])
        })
        .collect();
    TransferableTemporalCase {
        query: BooleanSequence::new(frames),
        expected_template: TemplateId::new(300_000 + u64::from(class)),
    }
}

#[cfg(test)]
mod transferable_fixture_tests {
    use super::{match_temporal_pattern, reusable_temporal_patterns, transferable_temporal_case};
    #[test]
    fn reusable_temporal_case_matches_class_pattern_under_new_distractors() {
        let patterns = reusable_temporal_patterns();
        let first = transferable_temporal_case(1);
        let repeated = transferable_temporal_case(4);
        assert_eq!(first.expected_template(), repeated.expected_template());
        assert_ne!(first.query(), repeated.query());
        let pattern = patterns
            .iter()
            .find(|p| p.id() == first.expected_template())
            .unwrap();
        assert!(match_temporal_pattern(pattern, first.query()).exact);
        assert!(match_temporal_pattern(pattern, repeated.query()).exact);
    }
}

use super::tdi2_intuition_reliability::{ReliabilityError, ReliabilityEvidence};
use super::tdi2_intuition_weight::ExperienceWeightPolicy;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemporalExperience {
    pattern: TemporalPattern,
    evidence: ReliabilityEvidence,
}
impl TemporalExperience {
    #[must_use]
    pub const fn pattern(&self) -> &TemporalPattern {
        &self.pattern
    }
    #[must_use]
    pub const fn evidence(&self) -> ReliabilityEvidence {
        self.evidence
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TemporalSelection {
    pub template_id: TemplateId,
    pub weight: f64,
    pub support: u64,
}

#[must_use]
pub fn temporal_experience_library(successes: u64) -> Vec<TemporalExperience> {
    reusable_temporal_patterns()
        .into_iter()
        .map(|pattern| TemporalExperience {
            pattern,
            evidence: ReliabilityEvidence::new(successes, 0),
        })
        .collect()
}

pub fn select_temporal_experience(
    experiences: &[TemporalExperience],
    query: &BooleanSequence,
    policy: ExperienceWeightPolicy,
) -> Result<Option<TemporalSelection>, ReliabilityError> {
    let mut candidates = Vec::new();
    for experience in experiences {
        if !match_temporal_pattern(experience.pattern(), query).exact {
            continue;
        }
        let support = experience.evidence().support();
        if support == 0 {
            continue;
        }
        candidates.push(TemporalSelection {
            template_id: experience.pattern().id(),
            weight: policy.weight(experience.evidence())?,
            support,
        });
    }
    candidates.sort_by(|left, right| {
        right
            .weight
            .total_cmp(&left.weight)
            .then_with(|| left.template_id.cmp(&right.template_id))
    });
    Ok(candidates.into_iter().next())
}

#[cfg(test)]
mod temporal_selection_tests {
    use super::{
        select_temporal_experience, temporal_experience_library, transferable_temporal_case,
    };
    use crate::experimental::tdi2_intuition_weight::ExperienceWeightPolicy;
    #[test]
    fn temporal_selection_requires_empirical_support() {
        let case = transferable_temporal_case(7);
        assert!(
            select_temporal_experience(
                &temporal_experience_library(0),
                case.query(),
                ExperienceWeightPolicy::default()
            )
            .unwrap()
            .is_none()
        );
        let selected = select_temporal_experience(
            &temporal_experience_library(4),
            case.query(),
            ExperienceWeightPolicy::default(),
        )
        .unwrap()
        .unwrap();
        assert_eq!(selected.template_id, case.expected_template());
        assert_eq!(selected.support, 4);
    }
}
