//! Frozen TDI-8 primary-cell and hypothesis-level decision rules.
//!
//! This module is a direct software transcription of the TDI-8.0
//! preregistration. It deliberately does not estimate uncertainty, choose
//! horizons, access a holdout, or change the frozen decision margin.

use core::fmt;

/// Exactly three frozen task families × three frozen horizon strata.
pub const TDI8_PRIMARY_CELL_COUNT: usize = 9;

/// Frozen equivalence/decision margin: two percentage points of relative
/// mean-deficit reduction.
pub const TDI8_EQUIVALENCE_MARGIN: f64 = 0.02;

/// Frozen four-way TDI-8 scientific verdict vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PrimaryVerdict {
    /// The candidate is beneficial under the frozen interval rule.
    Beneficial,
    /// The full interval lies inside the closed equivalence band.
    Equivalent,
    /// The candidate is harmful under the frozen interval rule.
    Harmful,
    /// Evidence satisfies none of the other three frozen classes.
    Inconclusive,
}

/// Valid two-sided interval for relative mean-deficit reduction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RelativeEffectInterval {
    lower: f64,
    upper: f64,
}

impl RelativeEffectInterval {
    /// Construct a finite interval satisfying `lower <= upper`.
    pub fn new(lower: f64, upper: f64) -> Result<Self, PrimaryDecisionError> {
        if !lower.is_finite() {
            return Err(PrimaryDecisionError::NonFiniteIntervalBound {
                bound: IntervalBound::Lower,
            });
        }
        if !upper.is_finite() {
            return Err(PrimaryDecisionError::NonFiniteIntervalBound {
                bound: IntervalBound::Upper,
            });
        }
        if lower > upper {
            return Err(PrimaryDecisionError::ReversedInterval { lower, upper });
        }
        Ok(Self { lower, upper })
    }

    /// Lower endpoint.
    #[must_use]
    pub const fn lower(self) -> f64 {
        self.lower
    }

    /// Upper endpoint.
    #[must_use]
    pub const fn upper(self) -> f64 {
        self.upper
    }
}

/// Interval endpoint named by a validation error.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IntervalBound {
    /// Lower endpoint.
    Lower,
    /// Upper endpoint.
    Upper,
}

/// Fail-closed validation errors for one primary-cell decision.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PrimaryDecisionError {
    /// Baseline deficit is NaN or infinite.
    NonFiniteBaselineDeficit,
    /// Candidate deficit is NaN or infinite.
    NonFiniteCandidateDeficit,
    /// Baseline deficit violates the non-negative deficit definition.
    NegativeBaselineDeficit,
    /// Candidate deficit violates the non-negative deficit definition.
    NegativeCandidateDeficit,
    /// Relative mean-deficit reduction became non-finite.
    NonFiniteRelativeEffect,
    /// A supplied interval endpoint is non-finite.
    NonFiniteIntervalBound {
        /// Invalid endpoint.
        bound: IntervalBound,
    },
    /// The supplied lower endpoint exceeds the upper endpoint.
    ReversedInterval {
        /// Supplied lower endpoint.
        lower: f64,
        /// Supplied upper endpoint.
        upper: f64,
    },
}

impl fmt::Display for PrimaryDecisionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for PrimaryDecisionError {}

/// One valid, non-rejected primary-cell decision record.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PrimaryCellDecision {
    verdict: PrimaryVerdict,
    relative_effect: Option<f64>,
    interval: Option<RelativeEffectInterval>,
    absolute_degradation: Option<f64>,
}

impl PrimaryCellDecision {
    /// Frozen four-way verdict.
    #[must_use]
    pub const fn verdict(self) -> PrimaryVerdict {
        self.verdict
    }

    /// Relative effect `R=(B-C)/B` when the baseline deficit is positive.
    #[must_use]
    pub const fn relative_effect(self) -> Option<f64> {
        self.relative_effect
    }

    /// Supplied uncertainty interval when the positive-baseline branch used it.
    #[must_use]
    pub const fn interval(self) -> Option<RelativeEffectInterval> {
        self.interval
    }

    /// Absolute degradation `C-B` for the frozen zero-baseline harmful branch.
    #[must_use]
    pub const fn absolute_degradation(self) -> Option<f64> {
        self.absolute_degradation
    }
}

/// Apply the frozen TDI-8 primary-cell classifier.
///
/// A missing interval for a valid positive-baseline cell is `Inconclusive` and
/// can never become a favorable result. Invalid deficits remain typed errors.
pub fn classify_primary_cell(
    baseline_deficit: f64,
    candidate_deficit: f64,
    interval: Option<RelativeEffectInterval>,
) -> Result<PrimaryCellDecision, PrimaryDecisionError> {
    validate_deficit(baseline_deficit, true)?;
    validate_deficit(candidate_deficit, false)?;

    if baseline_deficit == 0.0 {
        if candidate_deficit == 0.0 {
            return Ok(PrimaryCellDecision {
                verdict: PrimaryVerdict::Equivalent,
                relative_effect: None,
                interval: None,
                absolute_degradation: None,
            });
        }
        return Ok(PrimaryCellDecision {
            verdict: PrimaryVerdict::Harmful,
            relative_effect: None,
            interval: None,
            absolute_degradation: Some(candidate_deficit),
        });
    }

    let relative_effect = (baseline_deficit - candidate_deficit) / baseline_deficit;
    if !relative_effect.is_finite() {
        return Err(PrimaryDecisionError::NonFiniteRelativeEffect);
    }

    let Some(interval) = interval else {
        return Ok(PrimaryCellDecision {
            verdict: PrimaryVerdict::Inconclusive,
            relative_effect: Some(relative_effect),
            interval: None,
            absolute_degradation: None,
        });
    };

    let delta = TDI8_EQUIVALENCE_MARGIN;
    let verdict = if interval.lower() > delta {
        PrimaryVerdict::Beneficial
    } else if interval.upper() < -delta {
        PrimaryVerdict::Harmful
    } else if interval.lower() >= -delta && interval.upper() <= delta {
        PrimaryVerdict::Equivalent
    } else {
        PrimaryVerdict::Inconclusive
    };

    Ok(PrimaryCellDecision {
        verdict,
        relative_effect: Some(relative_effect),
        interval: Some(interval),
        absolute_degradation: None,
    })
}

fn validate_deficit(value: f64, baseline: bool) -> Result<(), PrimaryDecisionError> {
    if !value.is_finite() {
        return Err(if baseline {
            PrimaryDecisionError::NonFiniteBaselineDeficit
        } else {
            PrimaryDecisionError::NonFiniteCandidateDeficit
        });
    }
    if value < 0.0 {
        return Err(if baseline {
            PrimaryDecisionError::NegativeBaselineDeficit
        } else {
            PrimaryDecisionError::NegativeCandidateDeficit
        });
    }
    Ok(())
}

/// Hypothesis-level state of one of the exactly nine primary cells.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PrimaryCellDisposition {
    /// A valid classified primary-cell verdict.
    Classified(PrimaryVerdict),
    /// A missing or rejected primary cell.
    MissingOrRejected,
}

impl From<PrimaryCellDecision> for PrimaryCellDisposition {
    fn from(decision: PrimaryCellDecision) -> Self {
        Self::Classified(decision.verdict())
    }
}

/// Apply the frozen nine-cell hypothesis-level aggregation gate.
#[must_use]
pub fn aggregate_primary_hypothesis(
    cells: [PrimaryCellDisposition; TDI8_PRIMARY_CELL_COUNT],
) -> PrimaryVerdict {
    if cells
        .iter()
        .any(|cell| matches!(cell, PrimaryCellDisposition::MissingOrRejected))
    {
        return PrimaryVerdict::Inconclusive;
    }

    let verdicts = cells.map(|cell| match cell {
        PrimaryCellDisposition::Classified(verdict) => verdict,
        PrimaryCellDisposition::MissingOrRejected => unreachable!("handled above"),
    });

    let all_beneficial_or_equivalent = verdicts
        .iter()
        .all(|verdict| matches!(verdict, PrimaryVerdict::Beneficial | PrimaryVerdict::Equivalent));
    let any_beneficial = verdicts.contains(&PrimaryVerdict::Beneficial);
    if all_beneficial_or_equivalent && any_beneficial {
        return PrimaryVerdict::Beneficial;
    }

    let all_harmful_or_equivalent = verdicts
        .iter()
        .all(|verdict| matches!(verdict, PrimaryVerdict::Harmful | PrimaryVerdict::Equivalent));
    let any_harmful = verdicts.contains(&PrimaryVerdict::Harmful);
    if all_harmful_or_equivalent && any_harmful {
        return PrimaryVerdict::Harmful;
    }

    if verdicts
        .iter()
        .all(|verdict| *verdict == PrimaryVerdict::Equivalent)
    {
        return PrimaryVerdict::Equivalent;
    }

    PrimaryVerdict::Inconclusive
}

#[cfg(test)]
mod tests {
    use super::{
        PrimaryCellDisposition, PrimaryDecisionError, PrimaryVerdict, RelativeEffectInterval,
        TDI8_EQUIVALENCE_MARGIN, aggregate_primary_hypothesis, classify_primary_cell,
    };

    fn interval(lower: f64, upper: f64) -> RelativeEffectInterval {
        RelativeEffectInterval::new(lower, upper).expect("valid synthetic interval")
    }

    #[test]
    fn zero_baseline_branch_never_divides_and_never_becomes_beneficial() {
        let equivalent = classify_primary_cell(0.0, 0.0, None).expect("zero/zero");
        assert_eq!(equivalent.verdict(), PrimaryVerdict::Equivalent);
        assert_eq!(equivalent.relative_effect(), None);

        let harmful = classify_primary_cell(0.0, 0.25, None).expect("zero/positive");
        assert_eq!(harmful.verdict(), PrimaryVerdict::Harmful);
        assert_eq!(harmful.relative_effect(), None);
        assert_eq!(harmful.absolute_degradation(), Some(0.25));
    }

    #[test]
    fn nonzero_baseline_classifier_uses_exact_frozen_precedence_and_boundaries() {
        let delta = TDI8_EQUIVALENCE_MARGIN;
        assert_eq!(
            classify_primary_cell(1.0, 0.5, Some(interval(delta + 0.001, 0.5)))
                .expect("beneficial")
                .verdict(),
            PrimaryVerdict::Beneficial
        );
        assert_eq!(
            classify_primary_cell(1.0, 1.5, Some(interval(-0.5, -delta - 0.001)))
                .expect("harmful")
                .verdict(),
            PrimaryVerdict::Harmful
        );
        assert_eq!(
            classify_primary_cell(1.0, 1.0, Some(interval(-delta, delta)))
                .expect("closed equivalence band")
                .verdict(),
            PrimaryVerdict::Equivalent
        );
        assert_eq!(
            classify_primary_cell(1.0, 1.0, Some(interval(-0.03, 0.01)))
                .expect("outside equivalence band")
                .verdict(),
            PrimaryVerdict::Inconclusive
        );
    }

    #[test]
    fn missing_interval_is_inconclusive_not_favorable() {
        let decision = classify_primary_cell(2.0, 1.0, None).expect("valid deficits");
        assert_eq!(decision.relative_effect(), Some(0.5));
        assert_eq!(decision.verdict(), PrimaryVerdict::Inconclusive);
    }

    #[test]
    fn invalid_numeric_inputs_fail_closed() {
        assert_eq!(
            classify_primary_cell(f64::NAN, 0.0, None),
            Err(PrimaryDecisionError::NonFiniteBaselineDeficit)
        );
        assert_eq!(
            classify_primary_cell(1.0, -0.1, None),
            Err(PrimaryDecisionError::NegativeCandidateDeficit)
        );
        assert!(RelativeEffectInterval::new(0.2, -0.2).is_err());
        assert!(RelativeEffectInterval::new(f64::INFINITY, 0.2).is_err());
    }

    #[test]
    fn nine_cell_aggregation_matches_the_frozen_closed_rule() {
        let equivalent = PrimaryCellDisposition::Classified(PrimaryVerdict::Equivalent);
        let beneficial = PrimaryCellDisposition::Classified(PrimaryVerdict::Beneficial);
        let harmful = PrimaryCellDisposition::Classified(PrimaryVerdict::Harmful);
        let inconclusive = PrimaryCellDisposition::Classified(PrimaryVerdict::Inconclusive);

        let mut cells = [equivalent; 9];
        assert_eq!(aggregate_primary_hypothesis(cells), PrimaryVerdict::Equivalent);

        cells[3] = beneficial;
        assert_eq!(aggregate_primary_hypothesis(cells), PrimaryVerdict::Beneficial);

        let mut cells = [equivalent; 9];
        cells[4] = harmful;
        assert_eq!(aggregate_primary_hypothesis(cells), PrimaryVerdict::Harmful);

        let mut cells = [equivalent; 9];
        cells[0] = beneficial;
        cells[8] = harmful;
        assert_eq!(aggregate_primary_hypothesis(cells), PrimaryVerdict::Inconclusive);

        let mut cells = [equivalent; 9];
        cells[2] = inconclusive;
        assert_eq!(aggregate_primary_hypothesis(cells), PrimaryVerdict::Inconclusive);

        let mut cells = [equivalent; 9];
        cells[5] = PrimaryCellDisposition::MissingOrRejected;
        assert_eq!(aggregate_primary_hypothesis(cells), PrimaryVerdict::Inconclusive);
    }
}
