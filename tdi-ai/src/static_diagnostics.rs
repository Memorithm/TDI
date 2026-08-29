//! Static attention/operator controls for TDI-AI experiments.
//!
//! These descriptors are deliberately independent from intervention/recovery
//! trajectories. They exist so a future H-AI-1 experiment can test whether
//! dynamic recovery features add predictive information beyond competent,
//! cheaper static summaries.

const ROW_SUM_TOLERANCE: f64 = 1.0e-12;

/// Failure while validating or summarizing a row-stochastic attention matrix.
#[derive(Clone, Debug, PartialEq)]
pub enum StaticAttentionError {
    /// No attention rows were supplied.
    EmptyMatrix,
    /// A row contains no weights.
    EmptyRow {
        /// Zero-based row index.
        row: usize,
    },
    /// Rows do not share one common width.
    RaggedRow {
        /// Zero-based row index.
        row: usize,
        /// Width established by the first row.
        expected: usize,
        /// Width of the offending row.
        actual: usize,
    },
    /// A matrix entry is NaN or infinite.
    NonFiniteWeight {
        /// Zero-based row index.
        row: usize,
        /// Zero-based column index.
        column: usize,
    },
    /// A row-stochastic attention weight is negative.
    NegativeWeight {
        /// Zero-based row index.
        row: usize,
        /// Zero-based column index.
        column: usize,
        /// Rejected value.
        value: f64,
    },
    /// Summing a row overflowed to a non-finite value.
    NonFiniteRowSum {
        /// Zero-based row index.
        row: usize,
    },
    /// A row does not sum to one within the frozen construction tolerance.
    RowNotNormalized {
        /// Zero-based row index.
        row: usize,
        /// Observed row sum.
        sum: f64,
    },
}

/// Aggregate static descriptors of a validated row-stochastic attention matrix.
///
/// All entropy values use natural logarithms (nats). `mean_row_effective_support`
/// is the row-wise quantity `exp(H(row))` averaged across rows; it is not matrix
/// rank or effective rank. No field is a TDI recovery measurement.
#[derive(Clone, Debug, PartialEq)]
pub struct StaticAttentionDiagnostics {
    rows: usize,
    columns: usize,
    mean_row_entropy_nats: f64,
    mean_normalized_row_entropy: f64,
    mean_row_max_weight: f64,
    mean_row_l2_concentration: f64,
    mean_row_effective_support: f64,
    frobenius_norm: f64,
}

impl StaticAttentionDiagnostics {
    /// Number of query/attention rows summarized.
    #[must_use]
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// Number of key positions in each row.
    #[must_use]
    pub fn columns(&self) -> usize {
        self.columns
    }

    /// Mean Shannon entropy of rows in nats.
    #[must_use]
    pub fn mean_row_entropy_nats(&self) -> f64 {
        self.mean_row_entropy_nats
    }

    /// Mean row entropy divided by `ln(columns)`.
    ///
    /// A one-column matrix is defined to have normalized entropy zero.
    #[must_use]
    pub fn mean_normalized_row_entropy(&self) -> f64 {
        self.mean_normalized_row_entropy
    }

    /// Mean, over rows, of the largest attention weight in each row.
    #[must_use]
    pub fn mean_row_max_weight(&self) -> f64 {
        self.mean_row_max_weight
    }

    /// Mean row sum of squared weights, `mean(sum_j p_ij^2)`.
    #[must_use]
    pub fn mean_row_l2_concentration(&self) -> f64 {
        self.mean_row_l2_concentration
    }

    /// Mean row effective support `mean(exp(H(row)))`.
    ///
    /// This is an entropy-derived support size and must not be interpreted as
    /// matrix rank or spectral effective rank.
    #[must_use]
    pub fn mean_row_effective_support(&self) -> f64 {
        self.mean_row_effective_support
    }

    /// Frobenius norm of the complete attention matrix.
    #[must_use]
    pub fn frobenius_norm(&self) -> f64 {
        self.frobenius_norm
    }
}

/// Validate and summarize a row-stochastic attention matrix.
///
/// The matrix may be rectangular, which keeps this baseline usable for future
/// self-attention and cross-attention experiments. Every row must be non-empty,
/// finite, non-negative and sum to one within `1e-12` absolute tolerance.
///
/// The returned descriptors are static controls only. In particular, this
/// function does not compute intervention recovery, matrix rank, singular
/// values, eigenvalues, or any information-theoretic quantity beyond ordinary
/// row-wise Shannon entropy.
pub fn analyze_static_attention(
    weights: &[Vec<f64>],
) -> Result<StaticAttentionDiagnostics, StaticAttentionError> {
    let Some(first_row) = weights.first() else {
        return Err(StaticAttentionError::EmptyMatrix);
    };
    if first_row.is_empty() {
        return Err(StaticAttentionError::EmptyRow { row: 0 });
    }

    let columns = first_row.len();
    let mut entropy_sum = 0.0;
    let mut normalized_entropy_sum = 0.0;
    let mut max_weight_sum = 0.0;
    let mut l2_concentration_sum = 0.0;
    let mut effective_support_sum = 0.0;
    let mut squared_weight_sum = 0.0;
    let entropy_normalizer = if columns > 1 {
        Some((columns as f64).ln())
    } else {
        None
    };

    for (row_index, row) in weights.iter().enumerate() {
        if row.is_empty() {
            return Err(StaticAttentionError::EmptyRow { row: row_index });
        }
        if row.len() != columns {
            return Err(StaticAttentionError::RaggedRow {
                row: row_index,
                expected: columns,
                actual: row.len(),
            });
        }

        let mut row_sum = 0.0;
        let mut row_entropy = 0.0;
        let mut row_max = 0.0_f64;
        let mut row_l2 = 0.0;

        for (column_index, &weight) in row.iter().enumerate() {
            if !weight.is_finite() {
                return Err(StaticAttentionError::NonFiniteWeight {
                    row: row_index,
                    column: column_index,
                });
            }
            if weight < 0.0 {
                return Err(StaticAttentionError::NegativeWeight {
                    row: row_index,
                    column: column_index,
                    value: weight,
                });
            }

            row_sum += weight;
            row_max = row_max.max(weight);
            let square = weight * weight;
            row_l2 += square;
            squared_weight_sum += square;
            if weight > 0.0 {
                row_entropy -= weight * weight.ln();
            }
        }

        if !row_sum.is_finite() {
            return Err(StaticAttentionError::NonFiniteRowSum { row: row_index });
        }
        if (row_sum - 1.0).abs() > ROW_SUM_TOLERANCE {
            return Err(StaticAttentionError::RowNotNormalized {
                row: row_index,
                sum: row_sum,
            });
        }

        entropy_sum += row_entropy;
        normalized_entropy_sum += entropy_normalizer.map_or(0.0, |normalizer| {
            row_entropy / normalizer
        });
        max_weight_sum += row_max;
        l2_concentration_sum += row_l2;
        effective_support_sum += row_entropy.exp();
    }

    let rows = weights.len();
    let row_count = rows as f64;
    Ok(StaticAttentionDiagnostics {
        rows,
        columns,
        mean_row_entropy_nats: entropy_sum / row_count,
        mean_normalized_row_entropy: normalized_entropy_sum / row_count,
        mean_row_max_weight: max_weight_sum / row_count,
        mean_row_l2_concentration: l2_concentration_sum / row_count,
        mean_row_effective_support: effective_support_sum / row_count,
        frobenius_norm: squared_weight_sum.sqrt(),
    })
}

#[cfg(test)]
mod tests {
    use super::{StaticAttentionError, analyze_static_attention};

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() <= 1.0e-12,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn identity_attention_has_zero_entropy_and_unit_support() {
        let matrix = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let diagnostics = analyze_static_attention(&matrix).expect("valid matrix");

        assert_eq!(diagnostics.rows(), 2);
        assert_eq!(diagnostics.columns(), 2);
        assert_close(diagnostics.mean_row_entropy_nats(), 0.0);
        assert_close(diagnostics.mean_normalized_row_entropy(), 0.0);
        assert_close(diagnostics.mean_row_max_weight(), 1.0);
        assert_close(diagnostics.mean_row_l2_concentration(), 1.0);
        assert_close(diagnostics.mean_row_effective_support(), 1.0);
        assert_close(diagnostics.frobenius_norm(), 2.0_f64.sqrt());
    }

    #[test]
    fn uniform_attention_has_expected_static_controls() {
        let matrix = vec![vec![0.5, 0.5], vec![0.5, 0.5]];
        let diagnostics = analyze_static_attention(&matrix).expect("valid matrix");

        assert_close(diagnostics.mean_row_entropy_nats(), 2.0_f64.ln());
        assert_close(diagnostics.mean_normalized_row_entropy(), 1.0);
        assert_close(diagnostics.mean_row_max_weight(), 0.5);
        assert_close(diagnostics.mean_row_l2_concentration(), 0.5);
        assert_close(diagnostics.mean_row_effective_support(), 2.0);
        assert_close(diagnostics.frobenius_norm(), 1.0);
    }

    #[test]
    fn rectangular_attention_is_supported() {
        let matrix = vec![vec![0.25, 0.25, 0.5], vec![0.5, 0.25, 0.25]];
        let diagnostics = analyze_static_attention(&matrix).expect("valid matrix");

        assert_eq!(diagnostics.rows(), 2);
        assert_eq!(diagnostics.columns(), 3);
        assert_close(diagnostics.mean_row_max_weight(), 0.5);
        assert_close(diagnostics.mean_row_l2_concentration(), 0.375);
        assert_close(diagnostics.frobenius_norm(), 0.75_f64.sqrt());
    }

    #[test]
    fn malformed_attention_is_rejected() {
        assert_eq!(
            analyze_static_attention(&[]),
            Err(StaticAttentionError::EmptyMatrix)
        );
        assert_eq!(
            analyze_static_attention(&[Vec::new()]),
            Err(StaticAttentionError::EmptyRow { row: 0 })
        );
        assert_eq!(
            analyze_static_attention(&[vec![0.5, 0.5], vec![1.0]]),
            Err(StaticAttentionError::RaggedRow {
                row: 1,
                expected: 2,
                actual: 1,
            })
        );
        assert_eq!(
            analyze_static_attention(&[vec![1.25, -0.25]]),
            Err(StaticAttentionError::NegativeWeight {
                row: 0,
                column: 1,
                value: -0.25,
            })
        );
        assert_eq!(
            analyze_static_attention(&[vec![0.4, 0.4]]),
            Err(StaticAttentionError::RowNotNormalized { row: 0, sum: 0.8 })
        );
        assert_eq!(
            analyze_static_attention(&[vec![f64::NAN, 1.0]]),
            Err(StaticAttentionError::NonFiniteWeight { row: 0, column: 0 })
        );
    }
}
