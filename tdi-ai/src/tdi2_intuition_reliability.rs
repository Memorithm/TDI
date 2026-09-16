//! Empirical reliability accounting for TDI-2.1 experiential templates.

/// Success/failure evidence attached to one template.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReliabilityEvidence {
    successes: u64,
    failures: u64,
}

/// Errors from bounded reliability accounting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReliabilityError {
    /// A success/failure counter would overflow.
    CounterOverflow,
    /// Beta-prior parameters must be finite and strictly positive.
    InvalidPrior,
}

impl core::fmt::Display for ReliabilityError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::CounterOverflow => formatter.write_str("reliability evidence counter overflow"),
            Self::InvalidPrior => formatter.write_str("reliability prior must be finite and positive"),
        }
    }
}

impl std::error::Error for ReliabilityError {}

impl ReliabilityEvidence {
    /// Construct evidence from explicit counters.
    #[must_use]
    pub const fn new(successes: u64, failures: u64) -> Self {
        Self {
            successes,
            failures,
        }
    }

    /// Number of successful historical applications.
    #[must_use]
    pub const fn successes(self) -> u64 {
        self.successes
    }

    /// Number of failed historical applications.
    #[must_use]
    pub const fn failures(self) -> u64 {
        self.failures
    }

    /// Total number of validated historical applications.
    #[must_use]
    pub const fn support(self) -> u64 {
        self.successes.saturating_add(self.failures)
    }

    /// Record one observed validation outcome.
    pub fn observe(&mut self, success: bool) -> Result<(), ReliabilityError> {
        if success {
            self.successes = self
                .successes
                .checked_add(1)
                .ok_or(ReliabilityError::CounterOverflow)?;
        } else {
            self.failures = self
                .failures
                .checked_add(1)
                .ok_or(ReliabilityError::CounterOverflow)?;
        }
        Ok(())
    }

    /// Posterior mean under a Beta(alpha, beta) prior.
    pub fn posterior_mean(self, alpha: f64, beta: f64) -> Result<f64, ReliabilityError> {
        if !alpha.is_finite() || !beta.is_finite() || alpha <= 0.0 || beta <= 0.0 {
            return Err(ReliabilityError::InvalidPrior);
        }
        let successes = self.successes as f64;
        let failures = self.failures as f64;
        Ok((successes + alpha) / (successes + failures + alpha + beta))
    }
}

#[cfg(test)]
mod tests {
    use super::{ReliabilityError, ReliabilityEvidence};

    #[test]
    fn posterior_mean_preserves_empirical_support() {
        let evidence = ReliabilityEvidence::new(8, 2);
        let mean = evidence.posterior_mean(1.0, 1.0).expect("valid prior");
        assert!((mean - 0.75).abs() < f64::EPSILON);
        assert_eq!(evidence.support(), 10);
    }

    #[test]
    fn invalid_prior_fails_closed() {
        let evidence = ReliabilityEvidence::default();
        assert_eq!(
            evidence.posterior_mean(0.0, 1.0),
            Err(ReliabilityError::InvalidPrior)
        );
    }

    #[test]
    fn observation_checks_overflow() {
        let mut evidence = ReliabilityEvidence::new(u64::MAX, 0);
        assert_eq!(
            evidence.observe(true),
            Err(ReliabilityError::CounterOverflow)
        );
    }
}
