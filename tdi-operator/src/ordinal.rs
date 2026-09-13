//! TDI-12.0 Stage-0 exact ordinal ranking primitives.
//!
//! Scientific status: **EXACT** finite combinatorics / elementary algebra for
//! average-rank assignment, Spearman ρ, and Kendall τ-b on finite real
//! sequences. Candidate Green-band response observables are exposed only as
//! non-frozen Stage-0 calibration helpers that consume generic TDI-10
//! primitives. Coefficient-only control keys (Frobenius norm, Gershgorin
//! dominance margin), closed-form constant-Toeplitz control identities,
//! rank-normalization / negate-response scaffolding, observable ladders, and a
//! deterministic shuffle helper support Stage-0 control-battery and candidate
//! freeze-field declarations without freezing those fields. No confirmatory
//! TDI-12 population, split, or execution is authorized by this module.

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

/// Non-frozen Stage-0 candidate operator-population identifiers.
///
/// These match `operator_population_families.non_authorizing_candidates` in the
/// Stage-0 freeze template and do **not** pin that field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CandidateOperatorPopulation {
    /// Constant positive Toeplitz Jacobi matrices across a declared width ladder.
    PositiveConstantToeplitzWidthLadder,
    /// Diagonal-only Jacobi matrices across a declared width ladder.
    DiagonalOnlyWidthLadder,
}

impl CandidateOperatorPopulation {
    /// Stable identifier matching the Stage-0 freeze-template candidate list.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PositiveConstantToeplitzWidthLadder => "PositiveConstantToeplitzWidthLadder",
            Self::DiagonalOnlyWidthLadder => "DiagonalOnlyWidthLadder",
        }
    }
}

/// Non-frozen Stage-0 candidate normalization identifiers.
///
/// These match `normalization_contract.non_authorizing_candidates` and do
/// **not** pin that field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CandidateNormalization {
    /// Leave response values unchanged.
    IdentityResponse,
    /// Replace values by their average ranks (midranks).
    RankNormalizeToAverageRanks,
    /// Multiply every finite response by −1.
    NegateResponse,
}

impl CandidateNormalization {
    /// Stable identifier matching the Stage-0 freeze-template candidate list.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::IdentityResponse => "identity_response",
            Self::RankNormalizeToAverageRanks => "rank_normalize_to_average_ranks",
            Self::NegateResponse => "negate_response",
        }
    }
}

/// Replace each finite value by its average rank (Stage-0 rank-normalization).
///
/// EXACT: for a nondegenerate sample, Spearman / Kendall of the rank-normalized
/// vector against the original equal 1, because average ranks are a strictly
/// increasing function of the distinct order statistics and midranks preserve
/// the ordinal multiset used by both correlations.
pub fn rank_normalize(values: &[f64]) -> Result<Vec<f64>, OrdinalError> {
    average_ranks(values)
}

/// Pointwise negation of a finite sample (`negate_response` scaffolding).
///
/// EXACT: on a sample of pairwise-distinct finite values, Spearman / Kendall
/// against the negated sample equal −1.
pub fn negate_values(values: &[f64]) -> Result<Vec<f64>, OrdinalError> {
    if values.is_empty() {
        return Err(OrdinalError::EmptySample);
    }
    let mut out = Vec::with_capacity(values.len());
    for (index, value) in values.iter().enumerate() {
        if !value.is_finite() {
            return Err(OrdinalError::NonFiniteValue { index });
        }
        let negated = -*value;
        if !negated.is_finite() {
            return Err(OrdinalError::NonFiniteValue { index });
        }
        out.push(negated);
    }
    Ok(out)
}

/// Closed-form coefficient Frobenius norm for a constant Toeplitz Jacobi symbol.
///
/// For width `n ≥ 1` with diagonal `a` and off-diagonal `b`:
/// `sqrt(n a² + (n − 1) b²)` (the `n = 1` case has no off-diagonal term).
///
/// EXACT elementary algebra; matches [`coefficient_frobenius_norm_key`] on the
/// corresponding constant Toeplitz matrix. Does not freeze populations.
pub fn constant_toeplitz_frobenius_norm(
    width: usize,
    diagonal: f64,
    edge: f64,
) -> Result<f64, OrdinalError> {
    if width == 0 {
        return Err(OrdinalError::EmptyOperator);
    }
    if !diagonal.is_finite() {
        return Err(OrdinalError::NonFiniteValue { index: 0 });
    }
    if !edge.is_finite() {
        return Err(OrdinalError::NonFiniteValue { index: 1 });
    }
    let n = width as f64;
    let sum_sq = if width == 1 {
        n * diagonal * diagonal
    } else {
        n * diagonal * diagonal + (n - 1.0) * edge * edge
    };
    if !sum_sq.is_finite() || sum_sq < 0.0 {
        return Err(OrdinalError::NonFiniteValue { index: 0 });
    }
    let norm = sum_sq.sqrt();
    if !norm.is_finite() {
        return Err(OrdinalError::NonFiniteValue { index: 0 });
    }
    Ok(norm)
}

/// Closed-form Gershgorin dominance margin for a constant Toeplitz Jacobi symbol.
///
/// EXACT row-margin algebra:
/// - width 1: `a`
/// - width 2: `a − |b|`
/// - width ≥ 3: `a − 2|b|` (interior rows dominate the minimum)
///
/// Consequently the margin is **width-invariant for all n ≥ 3**. Correlating it
/// against dimension on a ladder contained in `{n : n ≥ 3}` fails closed with
/// [`OrdinalError::DegenerateRanks`]. This REFUTES treating the Stage-0
/// Gershgorin control as a covert dimension key on constant Toeplitz families.
pub fn constant_toeplitz_gershgorin_margin(
    width: usize,
    diagonal: f64,
    edge: f64,
) -> Result<f64, OrdinalError> {
    if width == 0 {
        return Err(OrdinalError::EmptyOperator);
    }
    if !diagonal.is_finite() {
        return Err(OrdinalError::NonFiniteValue { index: 0 });
    }
    if !edge.is_finite() {
        return Err(OrdinalError::NonFiniteValue { index: 1 });
    }
    let abs_edge = edge.abs();
    if !abs_edge.is_finite() {
        return Err(OrdinalError::NonFiniteValue { index: 1 });
    }
    let margin = match width {
        1 => diagonal,
        2 => diagonal - abs_edge,
        _ => diagonal - 2.0 * abs_edge,
    };
    if !margin.is_finite() {
        return Err(OrdinalError::NonFiniteValue { index: 0 });
    }
    Ok(margin)
}

/// Closed-form coefficient Frobenius norm for a constant diagonal-only Jacobi symbol.
///
/// For width `n ≥ 1` with diagonal `a` and **zero** off-diagonals:
/// `sqrt(n a²) = |a| √n`. This is the edge=`0` specialization of
/// [`constant_toeplitz_frobenius_norm`]. When `a ≠ 0` the key is strictly
/// increasing in `n`, so Spearman / Kendall against dimension equal 1 on any
/// strictly increasing width ladder. Does **not** freeze
/// `operator_population_families` (scaffolds `DiagonalOnlyWidthLadder` only).
pub fn constant_diagonal_only_frobenius_norm(
    width: usize,
    diagonal: f64,
) -> Result<f64, OrdinalError> {
    constant_toeplitz_frobenius_norm(width, diagonal, 0.0)
}

/// Closed-form Gershgorin dominance margin for a constant diagonal-only symbol.
///
/// With zero edges every row margin equals `a`, so the key is **width-invariant
/// for all n ≥ 1** (stronger than the constant-Toeplitz n≥3 case). Correlating
/// it against dimension on any ladder of length ≥ 2 fails closed with
/// [`OrdinalError::DegenerateRanks`]. This REFUTES treating the Stage-0
/// Gershgorin control as a covert dimension key on `DiagonalOnlyWidthLadder`.
pub fn constant_diagonal_only_gershgorin_margin(
    width: usize,
    diagonal: f64,
) -> Result<f64, OrdinalError> {
    constant_toeplitz_gershgorin_margin(width, diagonal, 0.0)
}

/// Closed-form mid-diagonal Green entry for a constant diagonal-only symbol.
///
/// Under the TDI-10 positive-pivot regime `a + shift > 0`, every diagonal Green
/// entry of `(K + t I)^{-1}` equals `1/(a + t)`, so the mid-diagonal observable
/// is **width-invariant**. Spearman / Kendall against dimension therefore fail
/// closed on any multi-width ladder — REFUTING MidDiagonalGreen as a covert
/// dimension key on `DiagonalOnlyWidthLadder`. Does **not** freeze
/// `response_observable_registry`.
pub fn constant_diagonal_only_mid_diagonal_green(
    diagonal: f64,
    shift: f64,
) -> Result<f64, OrdinalError> {
    if !diagonal.is_finite() {
        return Err(OrdinalError::NonFiniteValue { index: 0 });
    }
    if !shift.is_finite() {
        return Err(OrdinalError::NonFiniteValue { index: 1 });
    }
    let denom = diagonal + shift;
    if !(denom.is_finite() && denom > 0.0) {
        return Err(OrdinalError::NonFiniteValue { index: 0 });
    }
    let value = 1.0 / denom;
    if !value.is_finite() {
        return Err(OrdinalError::NonFiniteValue { index: 0 });
    }
    Ok(value)
}

/// Closed-form Green-trace observable for a constant diagonal-only symbol.
///
/// Under `a + shift > 0`, `GreenTrace = n / (a + shift)`. On any strictly
/// increasing width ladder this is strictly monotone in `n`, so Spearman /
/// Kendall against dimension equal 1. Matches
/// [`CandidateResponseObservable::GreenTrace`] on the corresponding matrix.
pub fn constant_diagonal_only_green_trace(
    width: usize,
    diagonal: f64,
    shift: f64,
) -> Result<f64, OrdinalError> {
    if width == 0 {
        return Err(OrdinalError::EmptyOperator);
    }
    let unit = constant_diagonal_only_mid_diagonal_green(diagonal, shift)?;
    let trace = (width as f64) * unit;
    if !trace.is_finite() {
        return Err(OrdinalError::NonFiniteValue { index: 0 });
    }
    Ok(trace)
}

/// Evaluate one candidate response observable on each matrix of a finite ladder.
///
/// Fail-closed: empty ladder → [`OrdinalError::EmptySample`]; any per-matrix
/// Green / resolvent failure propagates. Does **not** freeze
/// `operator_population_families` or `response_observable_registry`.
pub fn evaluate_observable_ladder(
    matrices: &[JacobiMatrix],
    shift: f64,
    observable: CandidateResponseObservable,
) -> Result<Vec<f64>, OrdinalError> {
    if matrices.is_empty() {
        return Err(OrdinalError::EmptySample);
    }
    let mut out = Vec::with_capacity(matrices.len());
    for matrix in matrices {
        out.push(observable.evaluate(matrix, shift)?);
    }
    Ok(out)
}

/// Stage-0 `tie_heavy_adversarial` scaffolding sample.
///
/// Returns a length-`n` (`n ≥ 3`) sequence with a single large interior tied
/// block and two distinct endpoints so ranks are non-constant (Spearman /
/// Kendall remain defined). EXACT midranks: endpoints occupy positions 1 and
/// `n`; the interior block of size `n − 2` occupies positions `2..=(n − 1)`
/// and therefore receives midrank `(n + 1) / 2`.
pub fn tie_heavy_adversarial_sample(n: usize) -> Result<Vec<f64>, OrdinalError> {
    if n < 3 {
        return Err(OrdinalError::EmptySample);
    }
    let mut values = Vec::with_capacity(n);
    values.push(0.0);
    values.extend(std::iter::repeat_n(1.0, n - 2));
    values.push(2.0);
    Ok(values)
}

#[cfg(test)]
mod tests {
    use super::{
        average_ranks, constant_diagonal_only_frobenius_norm,
        constant_diagonal_only_gershgorin_margin, constant_diagonal_only_green_trace,
        constant_diagonal_only_mid_diagonal_green, constant_toeplitz_frobenius_norm,
        constant_toeplitz_gershgorin_margin, deterministic_shuffle, kendall_tau_b, negate_values,
        rank_normalize, spearman_rho, strictly_increasing_affine, tie_heavy_adversarial_sample,
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

    #[test]
    fn constant_toeplitz_frobenius_closed_form_and_width_monotone() {
        let a = 4.0;
        let b = 1.0;
        let n3 = constant_toeplitz_frobenius_norm(3, a, b).unwrap();
        let n4 = constant_toeplitz_frobenius_norm(4, a, b).unwrap();
        assert!((n3 - (3.0 * a * a + 2.0 * b * b).sqrt()).abs() < 1.0e-15);
        assert!(n4 > n3);
    }

    #[test]
    fn constant_toeplitz_gershgorin_width_invariant_for_n_ge_3() {
        let a = 5.0;
        let b = 1.5;
        let m3 = constant_toeplitz_gershgorin_margin(3, a, b).unwrap();
        let m7 = constant_toeplitz_gershgorin_margin(7, a, b).unwrap();
        assert!((m3 - (a - 2.0 * b)).abs() < 1.0e-15);
        assert_eq!(m3, m7);
    }

    #[test]
    fn rank_normalize_preserves_unit_spearman_on_nondegenerate() {
        let values = [3.0, -1.0, 2.0, 2.0, 8.0];
        let ranks = rank_normalize(&values).unwrap();
        assert!((spearman_rho(&values, &ranks).unwrap() - 1.0).abs() < 1.0e-15);
        assert!((kendall_tau_b(&values, &ranks).unwrap() - 1.0).abs() < 1.0e-15);
    }

    #[test]
    fn negate_values_yields_exact_negative_unit_correlations() {
        let values = [1.0, 2.0, 3.0, 4.0];
        let negated = negate_values(&values).unwrap();
        assert!((spearman_rho(&values, &negated).unwrap() + 1.0).abs() < 1.0e-15);
        assert!((kendall_tau_b(&values, &negated).unwrap() + 1.0).abs() < 1.0e-15);
    }

    #[test]
    fn tie_heavy_adversarial_midranks_are_exact() {
        let sample = tie_heavy_adversarial_sample(5).unwrap();
        assert_eq!(sample, vec![0.0, 1.0, 1.0, 1.0, 2.0]);
        assert_eq!(
            average_ranks(&sample).unwrap(),
            vec![1.0, 3.0, 3.0, 3.0, 5.0]
        );
        assert!((spearman_rho(&sample, &sample).unwrap() - 1.0).abs() < 1.0e-15);
    }

    #[test]
    fn constant_diagonal_only_frobenius_closed_form_and_width_monotone() {
        let a = 3.0;
        let n3 = constant_diagonal_only_frobenius_norm(3, a).unwrap();
        let n5 = constant_diagonal_only_frobenius_norm(5, a).unwrap();
        assert!((n3 - a * (3.0_f64).sqrt()).abs() < 1.0e-15);
        assert!(n5 > n3);
        // Edge-zero specialization of the Toeplitz closed form.
        assert_eq!(n3, constant_toeplitz_frobenius_norm(3, a, 0.0).unwrap());
    }

    #[test]
    fn constant_diagonal_only_gershgorin_width_invariant_for_all_n() {
        let a = 4.5;
        let m1 = constant_diagonal_only_gershgorin_margin(1, a).unwrap();
        let m9 = constant_diagonal_only_gershgorin_margin(9, a).unwrap();
        assert_eq!(m1, a);
        assert_eq!(m9, a);
        assert_eq!(m1, constant_toeplitz_gershgorin_margin(1, a, 0.0).unwrap());
    }

    #[test]
    fn constant_diagonal_only_green_closed_forms() {
        let a = 4.0;
        let shift = 1.0;
        let mid = constant_diagonal_only_mid_diagonal_green(a, shift).unwrap();
        assert!((mid - 1.0 / (a + shift)).abs() < 1.0e-15);
        let trace = constant_diagonal_only_green_trace(7, a, shift).unwrap();
        assert!((trace - 7.0 / (a + shift)).abs() < 1.0e-15);
        // Non-positive pivot regime fail-closes.
        assert!(matches!(
            constant_diagonal_only_mid_diagonal_green(1.0, -1.0),
            Err(super::OrdinalError::NonFiniteValue { .. })
        ));
    }
}
