//! Numeric experience-strength policies for TDI-2.1 templates.

use super::tdi2_intuition_reliability::{ReliabilityError, ReliabilityEvidence};

/// Configuration for converting historical reliability/support into a numeric weight.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExperienceWeightPolicy {
    alpha: f64,
    beta: f64,
}

impl ExperienceWeightPolicy {
    /// Construct a policy from a positive finite Beta prior.
    pub fn new(alpha: f64, beta: f64) -> Result<Self, ReliabilityError> {
        if !alpha.is_finite() || !beta.is_finite() || alpha <= 0.0 || beta <= 0.0 {
            return Err(ReliabilityError::InvalidPrior);
        }
        Ok(Self { alpha, beta })
    }

    /// Reliability-weighted logarithmic experience mass.
    ///
    /// This quantity is numeric evidence. It does not alter Boolean applicability.
    pub fn weight(self, evidence: ReliabilityEvidence) -> Result<f64, ReliabilityError> {
        let reliability = evidence.posterior_mean(self.alpha, self.beta)?;
        let mass = (evidence.support() as f64).ln_1p();
        Ok(reliability * mass)
    }
}

impl Default for ExperienceWeightPolicy {
    fn default() -> Self {
        Self {
            alpha: 1.0,
            beta: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ExperienceWeightPolicy;
    use crate::experimental::tdi2_intuition_reliability::ReliabilityEvidence;

    #[test]
    fn unsupported_template_has_zero_experience_mass() {
        let weight = ExperienceWeightPolicy::default()
            .weight(ReliabilityEvidence::default())
            .expect("default prior is valid");
        assert_eq!(weight, 0.0);
    }

    #[test]
    fn additional_successful_support_increases_weight() {
        let policy = ExperienceWeightPolicy::default();
        let low = policy
            .weight(ReliabilityEvidence::new(2, 0))
            .expect("valid evidence");
        let high = policy
            .weight(ReliabilityEvidence::new(20, 0))
            .expect("valid evidence");
        assert!(high > low);
    }

    #[test]
    fn failures_reduce_weight_at_equal_support() {
        let policy = ExperienceWeightPolicy::default();
        let reliable = policy
            .weight(ReliabilityEvidence::new(8, 2))
            .expect("valid evidence");
        let unreliable = policy
            .weight(ReliabilityEvidence::new(2, 8))
            .expect("valid evidence");
        assert!(reliable > unreliable);
    }
}
