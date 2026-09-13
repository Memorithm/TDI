//! TDI-12.0 Stage-0 exact ordinal ranking primitives.
//!
//! Scientific status: **EXACT** finite combinatorics / elementary algebra for
//! average-rank assignment, Spearman ρ, and Kendall τ-b on finite real
//! sequences. Candidate Green-band response observables are exposed only as
//! non-frozen Stage-0 calibration helpers that consume generic TDI-10
//! primitives. Coefficient-only control keys (Frobenius norm, Gershgorin
//! dominance margin) and a deterministic shuffle helper support Stage-0
//! control-battery scaffolding without freezing the control_battery field.
//! No confirmatory TDI-12 population, split, or execution is authorized by
//! this module.

use core::fmt;

use crate::green::GreenBands;
use crate::jacobi::JacobiMatrix;
use crate::resolvent::ResolventError;

/// Fail-closed errors for ordinal ranking and Stage-0 response extraction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OrdinalError {
    EmptySample,
    LengthMismatch { left: usize, right: usize },
    NonFiniteValue { index: usize },
    DegenerateRanks,
    EmptyOperator,
    Resolvent(ResolventError),
}

impl fmt::Display for OrdinalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::EmptySample => write!(f, "ordinal sample is empty"),
            Self::LengthMismatch { left, right } => {
                write!(
                    f,
                    "ordinal sample lengths differ: left={left}, right={right}"
                )
            }
            Self::NonFiniteValue { index } => {
                write!(f, "ordinal sample value at index {index} is not finite")
            }
            Self::DegenerateRanks => {
                write!(f, "rank correlation is undefined on a fully tied sample")
            }
            Self::EmptyOperator => {
                write!(f, "response observable requires a non-empty Jacobi matrix")
            }
            Self::Resolvent(err) => write!(f, "resolvent/Green failure: {err}"),
        }
    }
}

impl std::error::Error for OrdinalError {}

impl From<ResolventError> for OrdinalError {
    fn from(value: ResolventError) -> Self {
        Self::Resolvent(value)
    }
}

/// Exact average ranks for ties (1-based midranks).
///
/// For each tied block occupying sorted positions `p..=q` (1-based), every
/// member receives the midrank `(p + q) / 2`. The assignment is unique given
/// the multiset of finite values and does not invent a secondary tie-break.
pub fn average_ranks(values: &[f64]) -> Result<Vec<f64>, OrdinalError> {
    if values.is_empty() {
        return Err(OrdinalError::EmptySample);
    }
    for (index, value) in values.iter().enumerate() {
        if !value.is_finite() {
            return Err(OrdinalError::NonFiniteValue { index });
        }
    }

    let mut order: Vec<usize> = (0..values.len()).collect();
    order.sort_by(|&left, &right| {
        values[left]
            .partial_cmp(&values[right])
            .expect("finite values admit a total order")
    });

    let mut ranks = vec![0.0; values.len()];
    let mut start = 0;
    while start < order.len() {
        let mut end = start + 1;
        while end < order.len() && values[order[end]] == values[order[start]] {
            end += 1;
        }
        // 1-based inclusive positions occupied by this tied block.
        let first = (start + 1) as f64;
        let last = end as f64;
        let midrank = (first + last) / 2.0;
        for &index in &order[start..end] {
            ranks[index] = midrank;
        }
        start = end;
    }
    Ok(ranks)
}

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / (values.len() as f64)
}

fn sample_covariance(left: &[f64], right: &[f64]) -> f64 {
    let left_mean = mean(left);
    let right_mean = mean(right);
    left.iter()
        .zip(right.iter())
        .map(|(l, r)| (l - left_mean) * (r - right_mean))
        .sum::<f64>()
        / ((left.len() - 1) as f64)
}

/// Exact Spearman ρ via Pearson correlation of average ranks.
///
/// Returns [`OrdinalError::DegenerateRanks`] when either rank vector is
/// constant (all ties), matching the classical undefined case.
pub fn spearman_rho(left: &[f64], right: &[f64]) -> Result<f64, OrdinalError> {
    if left.len() != right.len() {
        return Err(OrdinalError::LengthMismatch {
            left: left.len(),
            right: right.len(),
        });
    }
    if left.len() < 2 {
        return Err(OrdinalError::EmptySample);
    }

    let left_ranks = average_ranks(left)?;
    let right_ranks = average_ranks(right)?;
    let left_var = sample_covariance(&left_ranks, &left_ranks);
    let right_var = sample_covariance(&right_ranks, &right_ranks);
    if left_var == 0.0 || right_var == 0.0 {
        return Err(OrdinalError::DegenerateRanks);
    }
    Ok(sample_covariance(&left_ranks, &right_ranks) / (left_var.sqrt() * right_var.sqrt()))
}

/// Exact Kendall τ-b with pair-tie adjustment.
///
/// Counts concordant and discordant pairs among the `n(n-1)/2` unordered
/// pairs and applies the classical τ-b denominators
/// `sqrt((P+Q+T)*(P+Q+U))` where `T`/`U` are left/right tied pairs.
pub fn kendall_tau_b(left: &[f64], right: &[f64]) -> Result<f64, OrdinalError> {
    if left.len() != right.len() {
        return Err(OrdinalError::LengthMismatch {
            left: left.len(),
            right: right.len(),
        });
    }
    if left.len() < 2 {
        return Err(OrdinalError::EmptySample);
    }
    for (index, value) in left.iter().chain(right.iter()).enumerate() {
        if !value.is_finite() {
            return Err(OrdinalError::NonFiniteValue {
                index: index % left.len(),
            });
        }
    }

    let n = left.len();
    let mut concordant = 0.0;
    let mut discordant = 0.0;
    let mut left_ties = 0.0;
    let mut right_ties = 0.0;

    for i in 0..n {
        for j in (i + 1)..n {
            let dx = left[i] - left[j];
            let dy = right[i] - right[j];
            if dx == 0.0 && dy == 0.0 {
                // Joint ties contribute to neither numerator nor the
                // classical τ-b pair-tie denominators used here.
                continue;
            }
            if dx == 0.0 {
                left_ties += 1.0;
                continue;
            }
            if dy == 0.0 {
                right_ties += 1.0;
                continue;
            }
            let product = dx * dy;
            if product > 0.0 {
                concordant += 1.0;
            } else {
                discordant += 1.0;
            }
        }
    }

    let numerator = concordant - discordant;
    let left_denom: f64 = concordant + discordant + left_ties;
    let right_denom: f64 = concordant + discordant + right_ties;
    if left_denom == 0.0 || right_denom == 0.0 {
        return Err(OrdinalError::DegenerateRanks);
    }
    Ok(numerator / (left_denom.sqrt() * right_denom.sqrt()))
}

/// Apply a strictly increasing affine map `scale * x + shift` with `scale > 0`.
///
/// Used only to state the exact monotone-invariance lemma for Stage-0 tests.
pub fn strictly_increasing_affine(
    values: &[f64],
    scale: f64,
    shift: f64,
) -> Result<Vec<f64>, OrdinalError> {
    if !(scale.is_finite() && scale > 0.0 && shift.is_finite()) {
        return Err(OrdinalError::NonFiniteValue { index: 0 });
    }
    let mut out = Vec::with_capacity(values.len());
    for (index, value) in values.iter().enumerate() {
        if !value.is_finite() {
            return Err(OrdinalError::NonFiniteValue { index });
        }
        let mapped = scale * value + shift;
        if !mapped.is_finite() {
            return Err(OrdinalError::NonFiniteValue { index });
        }
        out.push(mapped);
    }
    Ok(out)
}

/// Non-frozen Stage-0 candidate response observables extracted from TDI-10
/// Green bands. These identifiers are **not** a TDI-12.0 freeze pin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CandidateResponseObservable {
    /// Mid-index diagonal Green entry `G_{⌊n/2⌋,⌊n/2⌋}`.
    MidDiagonalGreen,
    /// Trace of the Green diagonal band.
    GreenTrace,
    /// Mean absolute first off-diagonal Green entry (0 on 1×1 operators).
    MeanAbsOffDiagonalGreen,
}

impl CandidateResponseObservable {
    /// Stable identifier matching the Stage-0 freeze-template
    /// `non_authorizing_candidates` list. Not a freeze pin.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MidDiagonalGreen => "MidDiagonalGreen",
            Self::GreenTrace => "GreenTrace",
            Self::MeanAbsOffDiagonalGreen => "MeanAbsOffDiagonalGreen",
        }
    }

    /// Evaluate from already-computed public TDI-10 [`GreenBands`].
    ///
    /// EXACT wiring identity: [`Self::evaluate`] equals
    /// `from_green_bands(GreenBands::compute(...))` on the same operator/shift.
    pub fn from_green_bands(self, bands: &GreenBands) -> Result<f64, OrdinalError> {
        let diagonal = bands.diagonal();
        if diagonal.is_empty() {
            return Err(OrdinalError::EmptyOperator);
        }
        match self {
            Self::MidDiagonalGreen => Ok(diagonal[diagonal.len() / 2]),
            Self::GreenTrace => Ok(diagonal.iter().sum()),
            Self::MeanAbsOffDiagonalGreen => {
                let off = bands.off_diagonal();
                if off.is_empty() {
                    Ok(0.0)
                } else {
                    Ok(off.iter().map(|value| value.abs()).sum::<f64>() / (off.len() as f64))
                }
            }
        }
    }

    pub fn evaluate(self, matrix: &JacobiMatrix, shift: f64) -> Result<f64, OrdinalError> {
        if matrix.is_empty() {
            return Err(OrdinalError::EmptyOperator);
        }
        let bands = GreenBands::compute(matrix, shift)?;
        self.from_green_bands(&bands)
    }
}

/// Dimension-only control ranking key: the operator length itself.
///
/// Required Stage-0 control from `docs/TDI-12-PROGRAMME.md`. Ranking by
/// dimension alone must not consult response observables.
#[inline]
pub fn dimension_only_key(matrix: &JacobiMatrix) -> f64 {
    matrix.len() as f64
}

/// Identity ordering key: the response value itself (control that ordering
/// equals the observable ranking).
#[inline]
pub fn identity_ordering_key(response: f64) -> f64 {
    response
}

/// Coefficient Frobenius-norm control key (Stage-0 `norm_baseline` scaffolding).
///
/// Computes `sqrt(sum_i a_i^2 + sum_j b_j^2)` from the Jacobi diagonal and
/// off-diagonal only. Does **not** consult Green / resolvent values and does
/// **not** freeze the `control_battery` field.
pub fn coefficient_frobenius_norm_key(matrix: &JacobiMatrix) -> Result<f64, OrdinalError> {
    if matrix.is_empty() {
        return Err(OrdinalError::EmptyOperator);
    }
    let mut sum_sq = 0.0;
    for (index, value) in matrix.diagonal().iter().enumerate() {
        if !value.is_finite() {
            return Err(OrdinalError::NonFiniteValue { index });
        }
        sum_sq += value * value;
    }
    for (index, value) in matrix.off_diagonal().iter().enumerate() {
        if !value.is_finite() {
            return Err(OrdinalError::NonFiniteValue { index });
        }
        sum_sq += value * value;
    }
    let norm = sum_sq.sqrt();
    if !norm.is_finite() {
        return Err(OrdinalError::NonFiniteValue { index: 0 });
    }
    Ok(norm)
}

/// Gershgorin diagonal-dominance margin (Stage-0 `spectral_gap_baseline` proxy).
///
/// For each row `i`, the margin is `a_i - |b_{i-1}| - |b_i|` (missing edges 0).
/// The key is the minimum margin across rows. Exact coefficient algebra; does
/// not consult Green values and does not freeze `control_battery`.
pub fn gershgorin_dominance_margin_key(matrix: &JacobiMatrix) -> Result<f64, OrdinalError> {
    if matrix.is_empty() {
        return Err(OrdinalError::EmptyOperator);
    }
    let diagonal = matrix.diagonal();
    let off = matrix.off_diagonal();
    let mut min_margin = f64::INFINITY;
    for (index, &a) in diagonal.iter().enumerate() {
        if !a.is_finite() {
            return Err(OrdinalError::NonFiniteValue { index });
        }
        let left = if index > 0 { off[index - 1].abs() } else { 0.0 };
        let right = if index < off.len() {
            off[index].abs()
        } else {
            0.0
        };
        if !left.is_finite() || !right.is_finite() {
            return Err(OrdinalError::NonFiniteValue { index });
        }
        let margin = a - left - right;
        if !margin.is_finite() {
            return Err(OrdinalError::NonFiniteValue { index });
        }
        if margin < min_margin {
            min_margin = margin;
        }
    }
    Ok(min_margin)
}

/// Deterministic in-place Knuth shuffle with a fixed LCG seed stream.
///
/// Stage-0 `shuffled_family` scaffolding: applying this permutation to one side
/// of a perfectly ordered pair destroys ρ = 1 for nondegenerate length ≥ 3
/// samples. Not a freeze of split/population discipline.
pub fn deterministic_shuffle(values: &mut [f64], seed: u64) {
    let mut state = seed | 1;
    for i in (1..values.len()).rev() {
        // Numerical Recipes LCG; deterministic across platforms for Stage-0.
        state = state.wrapping_mul(1664525).wrapping_add(1013904223);
        let j = (state as usize) % (i + 1);
        values.swap(i, j);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        average_ranks, deterministic_shuffle, kendall_tau_b, spearman_rho,
        strictly_increasing_affine,
    };

    #[test]
    fn average_ranks_assign_midranks_for_ties() {
        let ranks = average_ranks(&[3.0, 1.0, 2.0, 2.0, 5.0]).unwrap();
        assert_eq!(ranks, vec![4.0, 1.0, 2.5, 2.5, 5.0]);
    }

    #[test]
    fn spearman_is_one_on_identity_nondegenerate() {
        let values = [1.0, 2.0, 3.0, 4.0];
        let rho = spearman_rho(&values, &values).unwrap();
        assert!((rho - 1.0).abs() < 1.0e-15);
    }

    #[test]
    fn spearman_rejects_full_ties() {
        let values = [2.0, 2.0, 2.0];
        assert!(matches!(
            spearman_rho(&values, &values),
            Err(super::OrdinalError::DegenerateRanks)
        ));
    }

    #[test]
    fn monotone_affine_preserves_spearman_and_kendall() {
        let left = [0.2, -1.0, 3.5, 3.5, 8.0];
        let right = [4.0, 1.0, 0.0, 2.0, 7.0];
        let mapped = strictly_increasing_affine(&left, 2.5, -11.0).unwrap();
        let rho = spearman_rho(&left, &right).unwrap();
        let tau = kendall_tau_b(&left, &right).unwrap();
        assert!((spearman_rho(&mapped, &right).unwrap() - rho).abs() < 1.0e-14);
        assert!((kendall_tau_b(&mapped, &right).unwrap() - tau).abs() < 1.0e-14);
    }

    #[test]
    fn reverse_ordering_yields_exact_negative_unit_correlations() {
        let ascending = [1.0, 2.0, 3.0, 4.0, 5.0];
        let descending = [5.0, 4.0, 3.0, 2.0, 1.0];
        assert!((spearman_rho(&ascending, &descending).unwrap() + 1.0).abs() < 1.0e-15);
        assert!((kendall_tau_b(&ascending, &descending).unwrap() + 1.0).abs() < 1.0e-15);
    }

    #[test]
    fn deterministic_shuffle_destroys_perfect_spearman_for_length_ge_3() {
        let original = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let mut shuffled = original;
        deterministic_shuffle(&mut shuffled, 0x07d1_1200_u64);
        assert_ne!(shuffled.to_vec(), original.to_vec());
        let rho = spearman_rho(&original, &shuffled).unwrap();
        assert!(rho.is_finite());
        assert!((rho - 1.0).abs() > 1.0e-12);
    }
}
