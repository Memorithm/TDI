//! TDI-24 Slice 07 deterministic masked-normalizer reference.

/// Versioned deterministic normalizer contract shared by V6 and C6.
pub const NORMALIZER_CONTRACT: &str = "tdi24-masked-softmax-reference-v1";

/// Versioned causal/non-causal mask contract.
pub const MASKING_CONTRACT: &str = "tdi24-attention-mask-reference-v1";

/// Reference masking policy shared by V6 and C6.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaskPolicy {
    /// Every key in the row is visible.
    Full,
    /// Key index k is visible iff k <= query_index.
    Causal,
}

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

/// Build the deterministic mask for one attention row.
pub fn attention_mask(
    policy: MaskPolicy,
    query_index: usize,
    key_count: usize,
) -> Result<Vec<bool>, NormalizerError> {
    if key_count == 0 {
        return Err(NormalizerError::EmptyInput);
    }
    Ok((0..key_count)
        .map(|key_index| match policy {
            MaskPolicy::Full => true,
            MaskPolicy::Causal => key_index <= query_index,
        })
        .collect())
}

/// Apply the shared mask contract and deterministic normalizer in one call.
pub fn normalize_with_policy(
    logits: &[f64],
    policy: MaskPolicy,
    query_index: usize,
) -> Result<Vec<f64>, NormalizerError> {
    let mask = attention_mask(policy, query_index, logits.len())?;
    masked_softmax(logits, &mask)
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
            let shifted = logit - maximum;
            if !shifted.is_finite() {
                return Err(NormalizerError::NonFiniteDerived);
            }
            let value = shifted.exp();
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
        assert_eq!(
            masked_softmax(&[-f64::MAX, f64::MAX], &[true, true]),
            Err(NormalizerError::NonFiniteDerived)
        );
    }
    #[test]
    fn full_and_causal_masks_are_deterministic_and_shared() {
        assert_eq!(
            attention_mask(MaskPolicy::Full, 1, 4).unwrap(),
            vec![true; 4]
        );
        assert_eq!(
            attention_mask(MaskPolicy::Causal, 1, 4).unwrap(),
            vec![true, true, false, false]
        );
        assert_eq!(
            attention_mask(MaskPolicy::Causal, 99, 4).unwrap(),
            vec![true, true, true, true]
        );
    }

    #[test]
    fn causal_normalization_never_assigns_mass_to_future_keys() {
        let probabilities =
            normalize_with_policy(&[1.0, 2.0, 30.0, 40.0], MaskPolicy::Causal, 1).unwrap();
        assert_eq!(probabilities[2], 0.0);
        assert_eq!(probabilities[3], 0.0);
        close(probabilities.iter().sum(), 1.0);
    }

    #[test]
    fn empty_mask_construction_fails_closed() {
        assert_eq!(
            attention_mask(MaskPolicy::Full, 0, 0),
            Err(NormalizerError::EmptyInput)
        );
    }
}
