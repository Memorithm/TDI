//! TDI-24 Slice 07 deterministic masked-normalizer reference.

/// Versioned deterministic normalizer contract shared by V6 and C6.
pub const NORMALIZER_CONTRACT: &str = "tdi24-masked-softmax-reference-v1";

/// Errors produced by the bounded reference normalizer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NormalizerError {
    /// Logit and mask lengths differ.
    LengthMismatch,
    /// Empty input is not a valid attention row.
    EmptyInput,
    /// At least one logit is NaN or infinite.
    NonFiniteLogit,
    /// Every position is masked.
    AllMasked,
    /// A derived exponential, sum or probability became invalid.
    NonFiniteDerived,
}

/// Stable f64 masked softmax reference.
///
/// The same function is intended for both V6 and C6 arms. Masked positions
/// receive exactly zero probability. All logits must be finite even when
/// masked, so malformed inputs cannot be hidden by masking.
pub fn masked_softmax(logits: &[f64], mask: &[bool]) -> Result<Vec<f64>, NormalizerError> {
    if logits.len() != mask.len() {
        return Err(NormalizerError::LengthMismatch);
    }
    if logits.is_empty() {
        return Err(NormalizerError::EmptyInput);
    }
    if !logits.iter().all(|value| value.is_finite()) {
        return Err(NormalizerError::NonFiniteLogit);
    }

    let mut maximum = f64::NEG_INFINITY;
    let mut any = false;
    for (&logit, &keep) in logits.iter().zip(mask) {
        if keep {
            maximum = maximum.max(logit);
            any = true;
        }
    }
    if !any {
        return Err(NormalizerError::AllMasked);
    }

    let mut exponentials = vec![0.0; logits.len()];
    let mut sum = 0.0;
    for (index, (&logit, &keep)) in logits.iter().zip(mask).enumerate() {
        if keep {
            let value = (logit - maximum).exp();
            if !value.is_finite() {
                return Err(NormalizerError::NonFiniteDerived);
            }
            exponentials[index] = value;
            sum += value;
            if !sum.is_finite() {
                return Err(NormalizerError::NonFiniteDerived);
            }
        }
    }
    if !(sum > 0.0 && sum.is_finite()) {
        return Err(NormalizerError::NonFiniteDerived);
    }

    for value in &mut exponentials {
        *value /= sum;
        if !value.is_finite() {
            return Err(NormalizerError::NonFiniteDerived);
        }
    }
    Ok(exponentials)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(lhs: f64, rhs: f64) {
        assert!((lhs - rhs).abs() <= 128.0 * f64::EPSILON);
    }

    #[test]
    fn masked_softmax_is_deterministic_normalized_and_zero_on_masked_slots() {
        let logits = [1.0, 2.0, -3.0, 4.0];
        let mask = [true, false, true, true];
        let first = masked_softmax(&logits, &mask).unwrap();
        let second = masked_softmax(&logits, &mask).unwrap();
        assert_eq!(first, second);
        assert_eq!(first[1], 0.0);
        close(first.iter().sum(), 1.0);
    }

    #[test]
    fn masked_softmax_is_shift_invariant_on_active_logits() {
        let mask = [true, true, false];
        let base = masked_softmax(&[1.0, 2.0, 3.0], &mask).unwrap();
        let shifted = masked_softmax(&[1001.0, 1002.0, 1003.0], &mask).unwrap();
        for (lhs, rhs) in base.iter().zip(shifted.iter()) {
            close(*lhs, *rhs);
        }
    }

    #[test]
    fn invalid_rows_fail_closed() {
        assert_eq!(masked_softmax(&[], &[]), Err(NormalizerError::EmptyInput));
        assert_eq!(
            masked_softmax(&[1.0], &[]),
            Err(NormalizerError::LengthMismatch)
        );
        assert_eq!(
            masked_softmax(&[1.0, 2.0], &[false, false]),
            Err(NormalizerError::AllMasked)
        );
        assert_eq!(
            masked_softmax(&[f64::NAN], &[true]),
            Err(NormalizerError::NonFiniteLogit)
        );
    }
}
