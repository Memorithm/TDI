//! Final candidate acceptance and abstention for TDI-2.1 reference inference.

use super::tdi2_intuition::TemplateId;
use super::tdi2_intuition_selection::Candidate;

/// Frozen acceptance policy for one inference run.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InferencePolicy {
    min_weight: f64,
}

/// Policy validation errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InferenceError {
    /// The configured minimum weight is NaN, infinite or negative.
    InvalidMinimumWeight,
}

impl core::fmt::Display for InferenceError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidMinimumWeight => {
                formatter.write_str("minimum intuition weight must be finite and non-negative")
            }
        }
    }
}

impl std::error::Error for InferenceError {}

impl InferencePolicy {
    /// Construct a validated policy.
    pub fn new(min_weight: f64) -> Result<Self, InferenceError> {
        if !min_weight.is_finite() || min_weight < 0.0 {
            return Err(InferenceError::InvalidMinimumWeight);
        }
        Ok(Self { min_weight })
    }

    /// Frozen minimum experience weight.
    #[must_use]
    pub const fn min_weight(self) -> f64 {
        self.min_weight
    }
}

impl Default for InferencePolicy {
    fn default() -> Self {
        Self { min_weight: 0.0 }
    }
}

/// Reference inference outcome. Abstention is an ordinary, non-error result.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IntuitionOutcome {
    /// One structurally applicable, empirically supported template was selected.
    Selected {
        /// Template identifier.
        template_id: TemplateId,
        /// Numeric experience strength used for ranking.
        weight: f64,
        /// Historical validated support.
        support: u64,
    },
    /// No candidate had both empirical support and sufficient weight.
    InsufficientExperience,
}

/// Select the highest-ranked admissible candidate or abstain.
///
/// Runtime latency is deliberately absent from this decision rule.
#[must_use]
pub fn infer(candidates: &[Candidate], policy: InferencePolicy) -> IntuitionOutcome {
    let Some(candidate) = candidates.first().copied() else {
        return IntuitionOutcome::InsufficientExperience;
    };

    if candidate.support() == 0 || candidate.weight() < policy.min_weight() {
        return IntuitionOutcome::InsufficientExperience;
    }

    IntuitionOutcome::Selected {
        template_id: candidate.template_id(),
        weight: candidate.weight(),
        support: candidate.support(),
    }
}

#[cfg(test)]
mod tests {
    use super::{InferenceError, InferencePolicy, IntuitionOutcome, infer};
    use crate::experimental::tdi2_intuition::TemplateId;
    use crate::experimental::tdi2_intuition_selection::Candidate;

    #[test]
    fn empty_candidate_set_abstains() {
        assert_eq!(
            infer(&[], InferencePolicy::default()),
            IntuitionOutcome::InsufficientExperience
        );
    }

    #[test]
    fn unsupported_candidate_abstains_even_at_zero_threshold() {
        let candidate = Candidate::new(TemplateId::new(1), 0.0, 0);
        assert_eq!(
            infer(&[candidate], InferencePolicy::default()),
            IntuitionOutcome::InsufficientExperience
        );
    }

    #[test]
    fn invalid_policy_fails_closed() {
        assert_eq!(
            InferencePolicy::new(f64::NAN),
            Err(InferenceError::InvalidMinimumWeight)
        );
    }
}
