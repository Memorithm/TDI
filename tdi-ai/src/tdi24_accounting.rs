//! TDI-24 Slice 09 source-level operation and logical-storage accounting.
//!
//! Counts in this module describe the bounded Rust reference algorithms. They
//! are not CPU instructions, GPU kernels, latency measurements, allocator peak
//! RSS, or hardware-performance claims.

use core::fmt;

use super::tdi24_chiral::CHIRAL_WIDTH;
use super::tdi24_vector::VECTOR6_WIDTH;

/// Versioned accounting surface for TDI-24 reference implementations.
pub const REFERENCE_ACCOUNTING_CONTRACT: &str = "tdi24-reference-accounting-v1";

const _MATCHED_WIDTHS: [(); VECTOR6_WIDTH] = [(); CHIRAL_WIDTH];

/// Reference score arm whose source-level work is being declared.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScoreArm {
    /// Matched six-dimensional vector dot-product control.
    V6,
    /// Full Stage-A chiral scalar score `alpha*s + beta*m + gamma*chi`.
    C6,
}

/// Source-level scalar work for one query-key score.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PairScoreAccounting {
    /// Arm described by this record.
    pub arm: ScoreArm,
    /// Scalar multiplications in the bounded reference path.
    pub multiplications: usize,
    /// Scalar additions/subtractions in the bounded reference path.
    pub additions: usize,
    /// Query scalar components.
    pub query_scalars: usize,
    /// Key scalar components.
    pub key_scalars: usize,
    /// Logical bytes for query + key carriers at f64 precision.
    pub carrier_bytes: usize,
    /// Versioned accounting contract.
    pub accounting_contract: &'static str,
}

/// Return the exact source-level pair-score accounting for the current reference.
#[must_use]
pub const fn pair_score_accounting(arm: ScoreArm) -> PairScoreAccounting {
    let (multiplications, additions) = match arm {
        // Six products and six checked accumulator additions.
        ScoreArm::V6 => (6, 6),
        // s: 6M+6A, m: 6M+6A, chi: 6M+6A,
        // three channel weights: 3M, final channel combination: 2A.
        ScoreArm::C6 => (21, 20),
    };
    PairScoreAccounting {
        arm,
        multiplications,
        additions,
        query_scalars: 6,
        key_scalars: 6,
        carrier_bytes: 12 * core::mem::size_of::<f64>(),
        accounting_contract: REFERENCE_ACCOUNTING_CONTRACT,
    }
}

/// Source-level row accounting for masking and the deterministic normalizer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RowAccounting {
    /// Number of keys represented in the row.
    pub key_count: usize,
    /// Number of active/unmasked keys.
    pub active_count: usize,
    /// Boolean mask tests performed while normalizing.
    pub mask_tests: usize,
    /// Max-reduction comparisons on active logits.
    pub max_comparisons: usize,
    /// Active-logit shifts by the row maximum.
    pub subtractions: usize,
    /// Exponential evaluations.
    pub exponentials: usize,
    /// Sum-accumulator additions.
    pub additions: usize,
    /// Final divisions performed by the current reference implementation.
    pub divisions: usize,
    /// Logical bytes of the explicit boolean mask.
    pub mask_bytes: usize,
    /// Logical bytes of the normalizer's f64 exponential/probability buffer.
    pub normalizer_scratch_bytes: usize,
    /// Versioned accounting contract.
    pub accounting_contract: &'static str,
}

/// Account one already-materialized mask/normalizer row.
///
/// The current reference divides every output slot, including exact-zero masked
/// slots, so `divisions == key_count`. This documents the implementation as it
/// exists rather than silently substituting a more optimized kernel.
pub fn row_accounting(
    key_count: usize,
    active_count: usize,
) -> Result<RowAccounting, AccountingError> {
    if key_count == 0 || active_count == 0 || active_count > key_count {
        return Err(AccountingError::InvalidRowShape);
    }
    let mask_bytes = key_count
        .checked_mul(core::mem::size_of::<bool>())
        .ok_or(AccountingError::SizeOverflow)?;
    let normalizer_scratch_bytes = key_count
        .checked_mul(core::mem::size_of::<f64>())
        .ok_or(AccountingError::SizeOverflow)?;

    Ok(RowAccounting {
        key_count,
        active_count,
        mask_tests: key_count,
        max_comparisons: active_count,
        subtractions: active_count,
        exponentials: active_count,
        additions: active_count,
        divisions: key_count,
        mask_bytes,
        normalizer_scratch_bytes,
        accounting_contract: REFERENCE_ACCOUNTING_CONTRACT,
    })
}

/// Fail-closed accounting errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccountingError {
    /// Empty/all-masked rows or active counts larger than the row are invalid.
    InvalidRowShape,
    /// Logical byte-size arithmetic overflowed `usize`.
    SizeOverflow,
}

impl fmt::Display for AccountingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRowShape => formatter.write_str("invalid attention row accounting shape"),
            Self::SizeOverflow => formatter.write_str("attention accounting size overflow"),
        }
    }
}

impl std::error::Error for AccountingError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v6_and_c6_have_identical_carrier_storage() {
        let v6 = pair_score_accounting(ScoreArm::V6);
        let c6 = pair_score_accounting(ScoreArm::C6);
        assert_eq!(v6.query_scalars, c6.query_scalars);
        assert_eq!(v6.key_scalars, c6.key_scalars);
        assert_eq!(v6.carrier_bytes, c6.carrier_bytes);
        assert_eq!(v6.carrier_bytes, 12 * core::mem::size_of::<f64>());
    }

    #[test]
    fn pair_operation_counts_match_current_reference_code_paths() {
        let v6 = pair_score_accounting(ScoreArm::V6);
        let c6 = pair_score_accounting(ScoreArm::C6);
        assert_eq!((v6.multiplications, v6.additions), (6, 6));
        assert_eq!((c6.multiplications, c6.additions), (21, 20));
        assert_eq!(v6.accounting_contract, REFERENCE_ACCOUNTING_CONTRACT);
        assert_eq!(c6.accounting_contract, REFERENCE_ACCOUNTING_CONTRACT);
    }

    #[test]
    fn row_accounting_matches_masked_reference_semantics() {
        let row = row_accounting(8, 5).unwrap();
        assert_eq!(row.mask_tests, 8);
        assert_eq!(row.max_comparisons, 5);
        assert_eq!(row.subtractions, 5);
        assert_eq!(row.exponentials, 5);
        assert_eq!(row.additions, 5);
        assert_eq!(row.divisions, 8);
        assert_eq!(row.mask_bytes, 8 * core::mem::size_of::<bool>());
        assert_eq!(
            row.normalizer_scratch_bytes,
            8 * core::mem::size_of::<f64>()
        );
    }

    #[test]
    fn invalid_row_accounting_fails_closed() {
        assert_eq!(row_accounting(0, 0), Err(AccountingError::InvalidRowShape));
        assert_eq!(row_accounting(4, 0), Err(AccountingError::InvalidRowShape));
        assert_eq!(row_accounting(4, 5), Err(AccountingError::InvalidRowShape));
    }
}
