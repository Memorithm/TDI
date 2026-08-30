//! TDI-7.1 bounded evaluator guards for the frozen attention-recovery protocol.
//!
//! This binary intentionally cannot execute the TDI-7.2 final holdout. It
//! establishes protocol primitives and software-oracle tests before task
//! generators and confirmatory evaluation are added.

const TDI7_RELEVANCE_MARGIN: f64 = 0.02;
const TDI7_FINAL_RUN_CONFIRMATION_VAR: &str = "TDI7_CONFIRM_FINAL_HOLDOUT";
const TDI7_FINAL_RUN_CONFIRMATION_VALUE: &str = "I_ACCEPT_THE_TDI7_HOLDOUT_FREEZE";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SeedRange {
    start: u64,
    end: u64,
}

impl SeedRange {
    fn new(start: u64, end: u64) -> Result<Self, ProtocolError> {
        if start > end {
            return Err(ProtocolError::EmptySeedRange);
        }
        Ok(Self { start, end })
    }

    fn overlaps(self, other: Self) -> bool {
        self.start <= other.end && other.start <= self.end
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SplitPlan {
    training: SeedRange,
    development: SeedRange,
    validation: SeedRange,
    final_holdout: SeedRange,
}

impl SplitPlan {
    fn validate(self) -> Result<Self, ProtocolError> {
        let ranges = [
            self.training,
            self.development,
            self.validation,
            self.final_holdout,
        ];

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tdi7Verdict {
    Beneficial,
    Equivalent,
    Harmful,
    Inconclusive,
}

fn classify_relative_mse_reduction(r: f64, lower: f64, upper: f64) -> Tdi7Verdict {
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

fn combine_task_verdicts(left: Tdi7Verdict, right: Tdi7Verdict) -> Tdi7Verdict {
    if left == Tdi7Verdict::Harmful || right == Tdi7Verdict::Harmful {
        Tdi7Verdict::Harmful
    } else if left == Tdi7Verdict::Equivalent && right == Tdi7Verdict::Equivalent {
        Tdi7Verdict::Equivalent
    } else if left == Tdi7Verdict::Beneficial || right == Tdi7Verdict::Beneficial {
        Tdi7Verdict::Beneficial
    } else {
        Tdi7Verdict::Inconclusive
    }
}

fn validate_retrieval_deficit(value: f64) -> Result<f64, ProtocolError> {
    if !value.is_finite() {
        return Err(ProtocolError::NonFiniteDeficit);
    }
    if value < 0.0 {
        return Err(ProtocolError::NegativeDeficit);
    }
    Ok(value)
}

fn final_holdout_is_authorized(value: Option<&str>) -> bool {
    value == Some(TDI7_FINAL_RUN_CONFIRMATION_VALUE)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProtocolError {
    EmptySeedRange,
    OverlappingSeedRanges,
    NonFiniteDeficit,
    NegativeDeficit,
}

fn preflight() -> Result<(), ProtocolError> {
    let plan = SplitPlan {
        training: SeedRange::new(7_100_000_000, 7_100_009_999)?,
        development: SeedRange::new(7_100_010_000, 7_100_019_999)?,
        validation: SeedRange::new(7_100_020_000, 7_100_029_999)?,
        final_holdout: SeedRange::new(7_100_030_000, 7_100_039_999)?,
    };
    plan.validate()?;

    if final_holdout_is_authorized(None) {
        unreachable!("final holdout must remain locked during preflight");
    }

    Ok(())
}

fn main() {
    if let Err(error) = preflight() {
        eprintln!("TDI-7.1 preflight failed: {error:?}");
        std::process::exit(1);
    }

    println!("TDI-7.1 preflight: PASS");
    println!("final holdout variable: {TDI7_FINAL_RUN_CONFIRMATION_VAR}");
    println!("TDI-7.2 final holdout: LOCKED");
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

        assert_eq!(
            plan.validate(),
            Err(ProtocolError::OverlappingSeedRanges)
        );
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
        assert_eq!(
            classify_relative_mse_reduction(0.02, 0.001, 0.04),
            Tdi7Verdict::Beneficial
        );
        assert_eq!(
            classify_relative_mse_reduction(-0.02, -0.04, -0.001),
            Tdi7Verdict::Harmful
        );
        assert_eq!(
            classify_relative_mse_reduction(0.0, -0.02, 0.02),
            Tdi7Verdict::Equivalent
        );
        assert_eq!(
            classify_relative_mse_reduction(0.03, -0.001, 0.05),
            Tdi7Verdict::Inconclusive
        );
    }

    #[test]
    fn multi_task_gate_matches_frozen_logic() {
        assert_eq!(
            combine_task_verdicts(Tdi7Verdict::Beneficial, Tdi7Verdict::Equivalent),
            Tdi7Verdict::Beneficial
        );
        assert_eq!(
            combine_task_verdicts(Tdi7Verdict::Beneficial, Tdi7Verdict::Harmful),
            Tdi7Verdict::Harmful
        );
        assert_eq!(
            combine_task_verdicts(Tdi7Verdict::Equivalent, Tdi7Verdict::Equivalent),
            Tdi7Verdict::Equivalent
        );
        assert_eq!(
            combine_task_verdicts(Tdi7Verdict::Equivalent, Tdi7Verdict::Inconclusive),
            Tdi7Verdict::Inconclusive
        );
    }

    #[test]
    fn deficit_orientation_rejects_negative_and_nonfinite_values() {
        assert_eq!(validate_retrieval_deficit(0.0), Ok(0.0));
        assert_eq!(validate_retrieval_deficit(1.25), Ok(1.25));
        assert_eq!(
            validate_retrieval_deficit(-0.01),
            Err(ProtocolError::NegativeDeficit)
        );
        assert_eq!(
            validate_retrieval_deficit(f64::NAN),
            Err(ProtocolError::NonFiniteDeficit)
        );
    }

    #[test]
    fn final_holdout_requires_exact_token() {
        assert!(!final_holdout_is_authorized(None));
        assert!(!final_holdout_is_authorized(Some("wrong")));
        assert!(final_holdout_is_authorized(Some(
            TDI7_FINAL_RUN_CONFIRMATION_VALUE
        )));
    }

    #[test]
    fn preflight_never_authorizes_final_holdout() {
        assert_eq!(preflight(), Ok(()));
        assert!(!final_holdout_is_authorized(None));
    }
}
