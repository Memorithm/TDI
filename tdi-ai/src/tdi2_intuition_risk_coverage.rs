//! Risk-coverage diagnostics for TDI-2.1 development/validation.

/// One selected prediction carrying its frozen experience-strength score.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScoredDecision {
    /// Experience-strength score used for selection.
    pub weight: f64,
    /// Whether the selected template matched the labelled target.
    pub correct: bool,
}

/// One point on a deterministic risk-coverage curve.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RiskCoveragePoint {
    /// Minimum accepted weight.
    pub threshold: f64,
    /// Fraction of all cases selected at this threshold.
    pub coverage: f64,
    /// Error rate among selected cases, or `None` when coverage is zero.
    pub selective_risk: Option<f64>,
    /// Number of selected cases.
    pub selected: usize,
}

/// Validation errors for risk-coverage computation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RiskCoverageError {
    /// A score or threshold was non-finite or negative.
    InvalidWeight,
    /// The declared total case count was smaller than the selected-decision list.
    InvalidTotal,
}

impl core::fmt::Display for RiskCoverageError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidWeight => {
                formatter.write_str("risk-coverage weights must be finite and non-negative")
            }
            Self::InvalidTotal => {
                formatter.write_str("total case count cannot be smaller than selected decisions")
            }
        }
    }
}

impl std::error::Error for RiskCoverageError {}

/// Evaluate fixed thresholds without learning from the evaluated cases.
pub fn risk_coverage_curve(
    decisions: &[ScoredDecision],
    thresholds: &[f64],
    total_cases: usize,
) -> Result<Vec<RiskCoveragePoint>, RiskCoverageError> {
    if total_cases < decisions.len() {
        return Err(RiskCoverageError::InvalidTotal);
    }
    if decisions
        .iter()
        .any(|decision| !decision.weight.is_finite() || decision.weight < 0.0)
        || thresholds
            .iter()
            .any(|threshold| !threshold.is_finite() || *threshold < 0.0)
    {
        return Err(RiskCoverageError::InvalidWeight);
    }

    let mut points = Vec::with_capacity(thresholds.len());
    for &threshold in thresholds {
        let selected = decisions
            .iter()
            .filter(|decision| decision.weight >= threshold)
            .count();
        let errors = decisions
            .iter()
            .filter(|decision| decision.weight >= threshold && !decision.correct)
            .count();
        let coverage = if total_cases == 0 {
            0.0
        } else {
            selected as f64 / total_cases as f64
        };
        let selective_risk = (selected > 0).then(|| errors as f64 / selected as f64);
        points.push(RiskCoveragePoint {
            threshold,
            coverage,
            selective_risk,
            selected,
        });
    }
    Ok(points)
}

#[cfg(test)]
mod tests {
    use super::{ScoredDecision, risk_coverage_curve};

    #[test]
    fn increasing_threshold_reduces_or_preserves_coverage() {
        let decisions = [
            ScoredDecision {
                weight: 0.2,
                correct: false,
            },
            ScoredDecision {
                weight: 0.8,
                correct: true,
            },
        ];
        let points = risk_coverage_curve(&decisions, &[0.0, 0.5, 0.9], 3).expect("valid curve");
        assert!(points[0].coverage >= points[1].coverage);
        assert!(points[1].coverage >= points[2].coverage);
        assert_eq!(points[1].selective_risk, Some(0.0));
        assert_eq!(points[2].selective_risk, None);
    }
}

/// Compact summary of an already computed risk-coverage curve.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RiskCoverageSummary {
    /// Highest observed coverage across supplied thresholds.
    pub max_coverage: f64,
    /// Lowest defined selective risk, if any threshold selected a case.
    pub min_defined_risk: Option<f64>,
    /// Highest coverage attained with exactly zero observed selective risk.
    pub max_zero_risk_coverage: f64,
}

/// Summarize a fixed risk-coverage curve without choosing a new threshold.
#[must_use]
pub fn summarize_risk_coverage(points: &[RiskCoveragePoint]) -> RiskCoverageSummary {
    let mut max_coverage = 0.0_f64;
    let mut min_defined_risk: Option<f64> = None;
    let mut max_zero_risk_coverage = 0.0_f64;
    for point in points {
        max_coverage = max_coverage.max(point.coverage);
        if let Some(risk) = point.selective_risk {
            min_defined_risk = Some(min_defined_risk.map_or(risk, |current| current.min(risk)));
            if risk == 0.0 {
                max_zero_risk_coverage = max_zero_risk_coverage.max(point.coverage);
            }
        }
    }
    RiskCoverageSummary {
        max_coverage,
        min_defined_risk,
        max_zero_risk_coverage,
    }
}

#[cfg(test)]
mod risk_coverage_summary_tests {
    use super::{ScoredDecision, risk_coverage_curve, summarize_risk_coverage};

    #[test]
    fn summary_reports_zero_risk_coverage_without_retuning() {
        let decisions = [
            ScoredDecision {
                weight: 0.2,
                correct: false,
            },
            ScoredDecision {
                weight: 0.8,
                correct: true,
            },
        ];
        let points = risk_coverage_curve(&decisions, &[0.0, 0.5, 0.9], 3).unwrap();
        let summary = summarize_risk_coverage(&points);
        assert_eq!(summary.max_coverage, 2.0 / 3.0);
        assert_eq!(summary.min_defined_risk, Some(0.0));
        assert_eq!(summary.max_zero_risk_coverage, 1.0 / 3.0);
    }
}
