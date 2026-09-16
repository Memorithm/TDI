//! Candidate-separation diagnostics for TDI-2.1 intuition inference.

use super::tdi2_intuition_selection::Candidate;

/// Separation between the top two ranked experiential candidates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CandidateMargin {
    /// Top candidate weight.
    pub top: f64,
    /// Runner-up weight.
    pub second: f64,
    /// Absolute separation `top - second`.
    pub absolute: f64,
    /// Separation relative to the top weight, or zero when both weights are zero.
    pub relative: f64,
}

/// Margin validation failures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarginError {
    /// Candidate weights were invalid or not ranked descending.
    InvalidRanking,
}

impl core::fmt::Display for MarginError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("candidate weights must be finite, non-negative, and ranked descending")
    }
}
impl std::error::Error for MarginError {}

/// Measure top-two separation without treating it as calibrated confidence.
pub fn top_two_margin(candidates: &[Candidate]) -> Result<Option<CandidateMargin>, MarginError> {
    if candidates.len() < 2 {
        return Ok(None);
    }
    let top = candidates[0].weight();
    let second = candidates[1].weight();
    if !top.is_finite() || !second.is_finite() || top < 0.0 || second < 0.0 || second > top {
        return Err(MarginError::InvalidRanking);
    }
    let absolute = top - second;
    let relative = if top == 0.0 { 0.0 } else { absolute / top };
    Ok(Some(CandidateMargin { top, second, absolute, relative }))
}

#[cfg(test)]
mod tests {
    use super::top_two_margin;
    use crate::experimental::tdi2_intuition::TemplateId;
    use crate::experimental::tdi2_intuition_selection::Candidate;

    #[test]
    fn margin_is_zero_for_equal_candidates() {
        let candidates = [
            Candidate::from_parts_for_reference(TemplateId::new(1), 2.0, 3),
            Candidate::from_parts_for_reference(TemplateId::new(2), 2.0, 3),
        ];
        let margin = top_two_margin(&candidates).expect("valid ranking").expect("two candidates");
        assert_eq!(margin.absolute, 0.0);
        assert_eq!(margin.relative, 0.0);
    }
}
