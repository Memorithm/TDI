//! TDI-25 Slice 08 shared masking/normalization bridge.
//!
//! This module reuses the TDI-24 reference masking and normalizer contracts
//! verbatim for T6, C6 and G6. It contains no copied softmax implementation and
//! no arm-specific masking rule.

use core::fmt;

use super::tdi24_attention::{
    MASKING_CONTRACT, MaskPolicy, NORMALIZER_CONTRACT, NormalizerError, normalize_with_policy,
};

/// Versioned TDI-25 bridge over the shared TDI-24 attention-row reference.
pub const SHARED_ATTENTION_BRIDGE_CONTRACT: &str = "tdi25-shared-attention-reference-v1";

/// Exact upstream contracts consumed by this TDI-25 bridge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SharedAttentionContracts {
    /// TDI-24 deterministic normalizer identity.
    pub normalizer: &'static str,
    /// TDI-24 full/causal mask identity.
    pub masking: &'static str,
    /// TDI-25 bridge identity.
    pub bridge: &'static str,
}

/// Return the exact shared attention-row contracts used by this build.
#[must_use]
pub const fn shared_attention_contracts() -> SharedAttentionContracts {
    SharedAttentionContracts {
        normalizer: NORMALIZER_CONTRACT,
        masking: MASKING_CONTRACT,
        bridge: SHARED_ATTENTION_BRIDGE_CONTRACT,
    }
}

/// One normalized row for each TDI-25 arm under the same mask policy.
#[derive(Clone, Debug, PartialEq)]
pub struct NormalizedArmRows {
    /// T6 normalized row.
    pub torsor: Vec<f64>,
    /// C6 normalized row.
    pub chiral: Vec<f64>,
    /// G6 normalized row.
    pub generic: Vec<f64>,
    /// Shared upstream/bridge contract provenance.
    pub contracts: SharedAttentionContracts,
}

/// Normalize T6/C6/G6 logits with one identical TDI-24 mask/normalizer path.
///
/// Row lengths must match before any arm is normalized. This prevents an arm
/// from silently receiving a different number of visible keys.
pub fn normalize_matched_rows(
    torsor_logits: &[f64],
    chiral_logits: &[f64],
    generic_logits: &[f64],
    policy: MaskPolicy,
    query_index: usize,
) -> Result<NormalizedArmRows, SharedAttentionError> {
    if torsor_logits.len() != chiral_logits.len() || torsor_logits.len() != generic_logits.len() {
        return Err(SharedAttentionError::RowLengthMismatch);
    }

    Ok(NormalizedArmRows {
        torsor: normalize_with_policy(torsor_logits, policy, query_index)
            .map_err(SharedAttentionError::Normalizer)?,
        chiral: normalize_with_policy(chiral_logits, policy, query_index)
            .map_err(SharedAttentionError::Normalizer)?,
        generic: normalize_with_policy(generic_logits, policy, query_index)
            .map_err(SharedAttentionError::Normalizer)?,
        contracts: shared_attention_contracts(),
    })
}

/// Fail-closed shared-attention bridge errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SharedAttentionError {
    /// T6/C6/G6 rows do not expose the same number of key positions.
    RowLengthMismatch,
    /// The upstream TDI-24 reference rejected a row.
    Normalizer(NormalizerError),
}

impl fmt::Display for SharedAttentionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RowLengthMismatch => {
                formatter.write_str("TDI-25 attention row lengths must match")
            }
            Self::Normalizer(error) => {
                write!(formatter, "shared normalizer rejected row: {error:?}")
            }
        }
    }
}

impl std::error::Error for SharedAttentionError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(lhs: f64, rhs: f64) {
        let scale = 1.0_f64.max(lhs.abs()).max(rhs.abs());
        assert!((lhs - rhs).abs() <= 128.0 * f64::EPSILON * scale);
    }

    #[test]
    fn bridge_pins_the_exact_tdi24_mask_and_normalizer_contracts() {
        assert_eq!(
            shared_attention_contracts(),
            SharedAttentionContracts {
                normalizer: "tdi24-masked-softmax-reference-v1",
                masking: "tdi24-attention-mask-reference-v1",
                bridge: "tdi25-shared-attention-reference-v1",
            }
        );
    }

    #[test]
    fn identical_arm_logits_receive_identical_normalization() {
        let logits = [1.0, 2.0, -3.0, 4.0];
        let rows = normalize_matched_rows(&logits, &logits, &logits, MaskPolicy::Full, 0).unwrap();
        assert_eq!(rows.torsor, rows.chiral);
        assert_eq!(rows.torsor, rows.generic);
        close(rows.torsor.iter().sum(), 1.0);
    }

    #[test]
    fn causal_policy_blocks_future_keys_for_every_arm() {
        let torsor = [1.0, 2.0, 30.0, 40.0];
        let chiral = [2.0, 1.0, 50.0, 60.0];
        let generic = [-1.0, 3.0, 70.0, 80.0];
        let rows =
            normalize_matched_rows(&torsor, &chiral, &generic, MaskPolicy::Causal, 1).unwrap();
        for row in [&rows.torsor, &rows.chiral, &rows.generic] {
            assert_eq!(row[2], 0.0);
            assert_eq!(row[3], 0.0);
            close(row.iter().sum(), 1.0);
        }
    }

    #[test]
    fn mismatched_row_lengths_fail_before_normalization() {
        assert_eq!(
            normalize_matched_rows(&[1.0, 2.0], &[1.0], &[1.0, 2.0], MaskPolicy::Full, 0),
            Err(SharedAttentionError::RowLengthMismatch)
        );
    }

    #[test]
    fn upstream_nonfinite_rejection_is_preserved() {
        assert_eq!(
            normalize_matched_rows(&[f64::NAN], &[1.0], &[1.0], MaskPolicy::Full, 0),
            Err(SharedAttentionError::Normalizer(
                NormalizerError::NonFiniteLogit
            ))
        );
    }
}
