//! Conditional two-block Schur-coercivity inequality.
//!
//! Let a self-adjoint operator have form [A B; B* D] with A >= alpha I,
//! D >= delta I and bounded cross coupling ||B|| <= beta. If alpha > 0,
//! its Schur complement is >= (delta - beta^2/alpha) I. Strict positivity
//! of both alpha and this margin is sufficient for block positivity.
//!
//! The caller must independently establish the two operator lower bounds,
//! the coupling norm bound, and compatible operator domains. This binary64
//! diagnostic is not outward-rounded interval arithmetic and does not
//! certify its own inputs.

use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlockSchurVerdict {
    StrictlyPositiveConditional,
    Undetermined,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConditionalBlockSchurMargin {
    head_lower_bound: f64,
    tail_lower_bound: f64,
    coupling_norm_upper_bound: f64,
    schur_tail_lower_bound: Option<f64>,
    verdict: BlockSchurVerdict,
}

impl ConditionalBlockSchurMargin {
    pub const fn head_lower_bound(self) -> f64 {
        self.head_lower_bound
    }

    pub const fn tail_lower_bound(self) -> f64 {
        self.tail_lower_bound
    }

    pub const fn coupling_norm_upper_bound(self) -> f64 {
        self.coupling_norm_upper_bound
    }

    /// None if the available head lower bound cannot justify inversion.
    pub const fn schur_tail_lower_bound(self) -> Option<f64> {
        self.schur_tail_lower_bound
    }

    pub const fn verdict(self) -> BlockSchurVerdict {
        self.verdict
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BlockSchurError {
    NonFiniteInput { index: usize, value: f64 },
    NegativeCouplingNorm { value: f64 },
    UnderflowedCouplingSquare,
    NonFiniteDerivedMargin,
}

impl fmt::Display for BlockSchurError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for BlockSchurError {}

/// Evaluate a numerical margin for the conditional Schur lemma.
///
/// Under independently established alpha/delta/beta operator bounds,
/// alpha > 0 and delta - beta^2/alpha > 0 imply a strictly positive
/// block form. This is a sufficient implication, not a characterization.
pub fn conditional_block_schur_margin(
    head_lower_bound: f64,
    tail_lower_bound: f64,
    coupling_norm_upper_bound: f64,
) -> Result<ConditionalBlockSchurMargin, BlockSchurError> {
    for (index, value) in [
        head_lower_bound,
        tail_lower_bound,
        coupling_norm_upper_bound,
    ]
    .into_iter()
    .enumerate()
    {
        if !value.is_finite() {
            return Err(BlockSchurError::NonFiniteInput { index, value });
        }
    }
    if coupling_norm_upper_bound < 0.0 {
        return Err(BlockSchurError::NegativeCouplingNorm {
            value: coupling_norm_upper_bound,
        });
    }

    let schur_tail_lower_bound = if head_lower_bound > 0.0 {
        let square = coupling_norm_upper_bound * coupling_norm_upper_bound;
        if !square.is_finite() {
            return Err(BlockSchurError::NonFiniteDerivedMargin);
        }
        // A nonzero beta can square to zero or a subnormal number. Dividing
        // that rounded value by tiny alpha can change the strict sign.
        // Refuse this range rather than silently certifying positivity.
        if coupling_norm_upper_bound != 0.0 && square < f64::MIN_POSITIVE {
            return Err(BlockSchurError::UnderflowedCouplingSquare);
        }
        let reduction = square / head_lower_bound;
        if !reduction.is_finite() {
            return Err(BlockSchurError::NonFiniteDerivedMargin);
        }
        let margin = tail_lower_bound - reduction;
        if !margin.is_finite() {
            return Err(BlockSchurError::NonFiniteDerivedMargin);
        }
        Some(margin)
    } else {
        None
    };
    let verdict = if schur_tail_lower_bound.is_some_and(|value| value > 0.0) {
        BlockSchurVerdict::StrictlyPositiveConditional
    } else {
        BlockSchurVerdict::Undetermined
    };

    Ok(ConditionalBlockSchurMargin {
        head_lower_bound,
        tail_lower_bound,
        coupling_norm_upper_bound,
        schur_tail_lower_bound,
        verdict,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strictly_positive_blocks_with_bounded_coupling() {
        let margin = conditional_block_schur_margin(2.0, 3.0, 1.0).unwrap();
        assert_eq!(margin.schur_tail_lower_bound(), Some(2.5));
        assert_eq!(
            margin.verdict(),
            BlockSchurVerdict::StrictlyPositiveConditional
        );
    }

    #[test]
    fn insufficient_lower_bound_is_undetermined_not_refuted() {
        let margin = conditional_block_schur_margin(1.0, 1.0, 2.0).unwrap();
        assert_eq!(margin.verdict(), BlockSchurVerdict::Undetermined);
        let margin = conditional_block_schur_margin(0.0, 2.0, 0.5).unwrap();
        assert_eq!(margin.verdict(), BlockSchurVerdict::Undetermined);
        assert_eq!(margin.schur_tail_lower_bound(), None);
    }

    #[test]
    fn zero_cross_coupling_keeps_tail_margin() {
        let margin = conditional_block_schur_margin(0.25, 0.5, 0.0).unwrap();
        assert_eq!(margin.schur_tail_lower_bound(), Some(0.5));
        assert_eq!(
            margin.verdict(),
            BlockSchurVerdict::StrictlyPositiveConditional
        );
    }

    #[test]
    fn underflowed_square_cannot_produce_false_schur_positivity() {
        // beta² = 1e-400 underflows f64, but beta²/alpha = 1e-100
        // exceeds delta = 1e-150: the exact margin is NEGATIVE.
        assert!(matches!(
            conditional_block_schur_margin(1e-300, 1e-150, 1e-200),
            Err(BlockSchurError::UnderflowedCouplingSquare)
        ));
        // A subnormal, nonzero square also has unacceptable relative error.
        assert!(matches!(
            conditional_block_schur_margin(1e-200, 1e-120, 1e-160),
            Err(BlockSchurError::UnderflowedCouplingSquare)
        ));
        // Exactly zero coupling remains legitimate.
        assert_eq!(
            conditional_block_schur_margin(1e-300, 1e-150, 0.0)
                .unwrap()
                .schur_tail_lower_bound(),
            Some(1e-150)
        );
    }

    #[test]
    fn invalid_inputs_fail_closed() {
        assert!(matches!(
            conditional_block_schur_margin(f64::NAN, 1.0, 0.1),
            Err(BlockSchurError::NonFiniteInput { .. })
        ));
        assert!(matches!(
            conditional_block_schur_margin(1.0, 1.0, -0.1),
            Err(BlockSchurError::NegativeCouplingNorm { .. })
        ));
        assert!(matches!(
            conditional_block_schur_margin(1.0, 1.0, f64::MAX),
            Err(BlockSchurError::NonFiniteDerivedMargin)
        ));
    }
}
