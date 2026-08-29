//! Bounded software guards for the frozen TDI-7.0 attention protocol.
//!
//! This module deliberately does not execute the final holdout. It provides
//! deterministic split, target, classifier, and confirmation-token primitives
//! that the TDI-7.1 evaluator can build on without changing TDI-7.0 choices.

/// Frozen relevance margin for TDI-7.0 relative MSE reduction.
pub const TDI7_RELEVANCE_MARGIN: f64 = 0.02;

/// Environment variable reserved for a deliberate TDI-7.2 final run.
pub const TDI7_FINAL_RUN_CONFIRMATION_VAR: &str = "TDI7_CONFIRM_FINAL_HOLDOUT";

/// Value required by the future final-holdout entry point.
pub const TDI7_FINAL_RUN_CONFIRMATION_VALUE: &str = "I_ACCEPT_THE_TDI7_HOLDOUT_FREEZE";

/// Disjoint inclusive integer seed interval.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SeedRange {
    start: u64,
    end: u64,
}

impl SeedRange {
    /// Construct a non-empty inclusive seed interval.
    pub fn new(start: u64, end: u64) -> Result<Self, ProtocolError> {
        if start > end {
            return Err(ProtocolError::EmptySeedRange);
        }
        Ok(Self { start, end })
    }

    #[must_use]
    pub fn start(self) -> u64 { self.start }

    #[must_use]
    pub fn end(self) -> u64 { self.end }

    #[must_use]
    pub fn overlaps(self, other: Self) -> bool {
        self.start <= other.end && other.start <= self.end
    }
}

/// Frozen four-way split required by TDI-7.0.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SplitPlan {
    pub training: SeedRange,
    pub development: SeedRange,
    pub validation: SeedRange,
    pub final_holdout: SeedRange,
}

impl SplitPlan {
    /// Reject any seed reuse across protocol splits.
    pub fn validate(self) -> Result<Self, ProtocolError> {
        let ranges = [self.training, self.development, self.validation, self.final_holdout];
        for left in 0..ranges.len() {
            for right in (left + 1)..ranges.len() {
                if ranges[left].overlaps(ranges[right]) {
                    return Err(ProtocolError::OverlappingSeedRanges);
                }
            }
        }
        Ok(self)
    }
}

/// TDI-7.0 four-way result classification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tdi7Verdict {
    Beneficial,
    Equivalent,
    Harmful,
    Inconclusive,
}

/// Classify a relative-MSE-reduction estimate and paired 95% interval.
#[must_use]
pub fn classify_relative_mse_reduction(r: f64, lower: f64, upper: f64) -> Tdi7Verdict {
    if !r.is_finite() || !lower.is_finite() || !upper.is_finite() || lower > upper {
        return Tdi7Verdict::Inconclusive;
    }
    if r >= TDI7_RELEVANCE_MARGIN && lower > 0.0 {
        return Tdi7Verdict::Beneficial;
    }
    if r <= -TDI7_RELEVANCE_MARGIN && upper < 0.0 {
        return Tdi7Verdict::Harmful;
    }
    if lower >= -TDI7_RELEVANCE_MARGIN && upper <= TDI7_RELEVANCE_MARGIN {
        return Tdi7Verdict::Equivalent;
    }
    Tdi7Verdict::Inconclusive
}

/// Combine the two confirmatory task verdicts exactly as frozen in TDI-7.0.
#[must_use]
pub fn combine_task_verdicts(left: Tdi7Verdict, right: Tdi7Verdict) -> Tdi7Verdict {
    use Tdi7Verdict::{Beneficial, Equivalent, Harmful, Inconclusive};
    if left == Harmful || right == Harmful {
        Harmful
    } else if left == Equivalent && right == Equivalent {
        Equivalent
    } else if left == Beneficial || right == Beneficial {
        Beneficial
    } else {
        Inconclusive
    }
}

/// Validate the frozen retrieval-deficit orientation.
pub fn validate_retrieval_deficit(value: f64) -> Result<f64, ProtocolError> {
    if !value.is_finite() {
        return Err(ProtocolError::NonFiniteDeficit);
    }
    if value < 0.0 {
        return Err(ProtocolError::NegativeDeficit);
    }
    Ok(value)
}

/// Check a supplied final-run token without reading process environment.
///
/// Keeping this function pure lets CI prove rejection behavior without ever
/// supplying the real token through repository configuration.
#[must_use]
pub fn final_holdout_is_authorized(value: Option<&str>) -> bool {
    value == Some(TDI7_FINAL_RUN_CONFIRMATION_VALUE)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProtocolError {
    EmptySeedRange,
    OverlappingSeedRanges,
    NonFiniteDeficit,
    NegativeDeficit,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_plan_rejects_any_overlap() {
        let plan = SplitPlan {
            training: SeedRange::new(0, 99).unwrap(),
            development: SeedRange::new(100, 199).unwrap(),
            validation: SeedRange::new(199, 299).unwrap(),
            final_holdout: SeedRange::new(300, 399).unwrap(),
        };
        assert_eq!(plan.validate(), Err(ProtocolError::OverlappingSeedRanges));
    }

    #[test]
    fn split_plan_accepts_disjoint_ranges() {
        let plan = SplitPlan {
            training: SeedRange::new(0, 99).unwrap(),
            development: SeedRange::new(100, 199).unwrap(),
            validation: SeedRange::new(200, 299).unwrap(),
            final_holdout: SeedRange::new(300, 399).unwrap(),
        };
        assert_eq!(plan.validate(), Ok(plan));
    }

    #[test]
    fn classifier_boundaries_match_preregistration() {
        assert_eq!(classify_relative_mse_reduction(0.02, 0.001, 0.04), Tdi7Verdict::Beneficial);
        assert_eq!(classify_relative_mse_reduction(-0.02, -0.04, -0.001), Tdi7Verdict::Harmful);
        assert_eq!(classify_relative_mse_reduction(0.0, -0.02, 0.02), Tdi7Verdict::Equivalent);
        assert_eq!(classify_relative_mse_reduction(0.03, -0.001, 0.05), Tdi7Verdict::Inconclusive);
    }

    #[test]
    fn multi_task_gate_matches_frozen_logic() {
        assert_eq!(combine_task_verdicts(Tdi7Verdict::Beneficial, Tdi7Verdict::Equivalent), Tdi7Verdict::Beneficial);
        assert_eq!(combine_task_verdicts(Tdi7Verdict::Beneficial, Tdi7Verdict::Harmful), Tdi7Verdict::Harmful);
        assert_eq!(combine_task_verdicts(Tdi7Verdict::Equivalent, Tdi7Verdict::Equivalent), Tdi7Verdict::Equivalent);
        assert_eq!(combine_task_verdicts(Tdi7Verdict::Equivalent, Tdi7Verdict::Inconclusive), Tdi7Verdict::Inconclusive);
    }

    #[test]
    fn deficit_orientation_rejects_negative_and_nonfinite_values() {
        assert_eq!(validate_retrieval_deficit(0.0), Ok(0.0));
        assert_eq!(validate_retrieval_deficit(1.25), Ok(1.25));
        assert_eq!(validate_retrieval_deficit(-0.01), Err(ProtocolError::NegativeDeficit));
        assert_eq!(validate_retrieval_deficit(f64::NAN), Err(ProtocolError::NonFiniteDeficit));
    }

    #[test]
    fn final_holdout_requires_exact_token() {
        assert!(!final_holdout_is_authorized(None));
        assert!(!final_holdout_is_authorized(Some("wrong")));
        assert!(final_holdout_is_authorized(Some(TDI7_FINAL_RUN_CONFIRMATION_VALUE)));
    }
}
