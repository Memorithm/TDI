use tdi_operator::{
    GreenBands, JacobiError, JacobiMatrix, ResolventError, SchurCavities, ShiftedLdl,
};

fn shifted_dense(matrix: &JacobiMatrix, shift: f64) -> Vec<Vec<f64>> {
    let n = matrix.len();
    let mut dense = vec![vec![0.0; n]; n];
    let mut i = 0;
    while i < n {
        dense[i][i] = matrix.diagonal()[i] + shift;
        if i + 1 < n {
            let edge = matrix.off_diagonal()[i];
            dense[i][i + 1] = edge;
            dense[i + 1][i] = edge;
        }
        i += 1;
    }
    dense
}

/// Independent small-matrix oracle: Gauss-Jordan elimination with partial
/// pivoting on a dense augmented matrix. It does not call any TDI-10 cavity or
/// LDL implementation.
fn dense_inverse(matrix: Vec<Vec<f64>>) -> Vec<Vec<f64>> {
    let n = matrix.len();
    let mut augmented = vec![vec![0.0; 2 * n]; n];
    let mut row = 0;
    while row < n {
        let mut column = 0;
        while column < n {
            augmented[row][column] = matrix[row][column];
            column += 1;
        }
        augmented[row][n + row] = 1.0;
        row += 1;
    }

    let mut pivot_column = 0;
    while pivot_column < n {
        let mut pivot_row = pivot_column;
        let mut pivot_abs = augmented[pivot_row][pivot_column].abs();
        let mut candidate = pivot_column + 1;
        while candidate < n {
            let candidate_abs = augmented[candidate][pivot_column].abs();
            if candidate_abs > pivot_abs {
                pivot_row = candidate;
                pivot_abs = candidate_abs;
            }
            candidate += 1;
        }
        assert!(
            pivot_abs > 1.0e-14,
            "dense oracle encountered a singular pivot"
        );
        augmented.swap(pivot_column, pivot_row);

        let pivot = augmented[pivot_column][pivot_column];
        let mut column = 0;
        while column < 2 * n {
            augmented[pivot_column][column] /= pivot;
            column += 1;
        }

        let mut eliminate_row = 0;
        while eliminate_row < n {
            if eliminate_row != pivot_column {
                let factor = augmented[eliminate_row][pivot_column];
                let mut eliminate_column = 0;
                while eliminate_column < 2 * n {
                    augmented[eliminate_row][eliminate_column] -=
                        factor * augmented[pivot_column][eliminate_column];
                    eliminate_column += 1;
                }
            }
            eliminate_row += 1;
        }
        pivot_column += 1;
    }

    let mut inverse = vec![vec![0.0; n]; n];
    let mut i = 0;
    while i < n {
        let mut j = 0;
        while j < n {
            inverse[i][j] = augmented[i][n + j];
            j += 1;
        }
        i += 1;
    }
    inverse
}

fn assert_close(left: f64, right: f64, tolerance: f64) {
    let scale = 1.0_f64.max(left.abs()).max(right.abs());
    assert!(
        (left - right).abs() <= tolerance * scale,
        "left={left:.17e}, right={right:.17e}, |delta|={:.3e}",
        (left - right).abs()
    );
}

#[test]
fn jacobi_constructor_enforces_dimensions_and_finite_coefficients() {
    let mismatch = JacobiMatrix::new(vec![1.0, 2.0], vec![]).unwrap_err();
    assert_eq!(
        mismatch,
        JacobiError::DimensionMismatch {
            diagonal: 2,
            off_diagonal: 0,
        }
    );

    assert!(matches!(
        JacobiMatrix::new(vec![1.0, f64::NAN], vec![0.5]),
        Err(JacobiError::NonFiniteDiagonal { index: 1, .. })
    ));
    assert!(matches!(
        JacobiMatrix::new(vec![1.0, 2.0], vec![f64::INFINITY]),
        Err(JacobiError::NonFiniteOffDiagonal { index: 0, .. })
    ));
}

#[test]
fn empty_operator_has_empty_exact_resolvent_data() {
    let matrix = JacobiMatrix::new(Vec::new(), Vec::new()).unwrap();
    let ldl = ShiftedLdl::factor(&matrix, 0.0).unwrap();
    let cavities = SchurCavities::compute(&matrix, 0.0).unwrap();
    let green = GreenBands::compute(&matrix, 0.0).unwrap();

    assert!(ldl.pivots().is_empty());
    assert!(ldl.selected_inverse_bands().0.is_empty());
    assert!(cavities.left().is_empty());
    assert!(cavities.right().is_empty());
    assert!(green.diagonal().is_empty());
    assert!(green.off_diagonal().is_empty());
}

#[test]
fn green_and_ldl_bands_match_independent_dense_inverse() {
    for n in 1_usize..=8 {
        let diagonal: Vec<f64> = (0..n)
            .map(|i| 3.5 + 0.2 * i as f64 + 0.03 * (i * i) as f64)
            .collect();
        let off_diagonal: Vec<f64> = (0..n.saturating_sub(1))
            .map(|i| {
                let sign = if i % 2 == 0 { 1.0 } else { -1.0 };
                sign * (0.25 + 0.02 * i as f64)
            })
            .collect();
        let matrix = JacobiMatrix::new(diagonal, off_diagonal).unwrap();

        for shift in [-0.5_f64, 0.0, 1.0e-8, 0.25, 3.0] {
            let dense = dense_inverse(shifted_dense(&matrix, shift));
            let green = GreenBands::compute(&matrix, shift).unwrap();
            let ldl = ShiftedLdl::factor(&matrix, shift).unwrap();
            let (ldl_diagonal, ldl_off_diagonal) = ldl.selected_inverse_bands();

            let mut i = 0;
            while i < n {
                assert_close(green.diagonal()[i], dense[i][i], 2.0e-12);
                assert_close(ldl_diagonal[i], dense[i][i], 2.0e-12);
                if i + 1 < n {
                    assert_close(green.off_diagonal()[i], dense[i][i + 1], 2.0e-12);
                    assert_close(ldl_off_diagonal[i], dense[i][i + 1], 2.0e-12);
                }
                i += 1;
            }
        }
    }
}

#[test]
fn left_cavities_equal_positive_ldl_pivots() {
    let matrix = JacobiMatrix::new(vec![4.0, 5.0, 6.0, 7.0], vec![-1.0, 0.5, 1.25]).unwrap();
    let shift = 0.125;
    let cavities = SchurCavities::compute(&matrix, shift).unwrap();
    let ldl = ShiftedLdl::factor(&matrix, shift).unwrap();
    assert_eq!(cavities.left(), ldl.pivots());

    let mut right_manual = vec![0.0; matrix.len()];
    let last = matrix.len() - 1;
    right_manual[last] = matrix.diagonal()[last] + shift;
    let mut i = last;
    while i > 0 {
        let row = i - 1;
        let edge = matrix.off_diagonal()[row];
        right_manual[row] = matrix.diagonal()[row] + shift - edge * edge / right_manual[row + 1];
        i -= 1;
    }
    assert_eq!(cavities.right(), right_manual);
}

#[test]
fn non_positive_shifted_pivot_is_rejected_fail_closed() {
    let matrix = JacobiMatrix::new(vec![1.0, 1.0], vec![2.0]).unwrap();
    assert!(matches!(
        ShiftedLdl::factor(&matrix, 0.0),
        Err(ResolventError::NonPositivePivot { index: 1, .. })
    ));
    assert!(matches!(
        SchurCavities::compute(&matrix, 0.0),
        Err(ResolventError::NonPositivePivot { index: 1, .. })
    ));
    assert!(matches!(
        GreenBands::compute(&matrix, 0.0),
        Err(ResolventError::NonPositivePivot { .. })
    ));
}

#[test]
fn finite_negative_shift_is_allowed_only_when_positive_pivots_survive() {
    let matrix = JacobiMatrix::new(vec![4.0, 4.0], vec![1.0]).unwrap();
    let green = GreenBands::compute(&matrix, -1.0).unwrap();
    let dense = dense_inverse(shifted_dense(&matrix, -1.0));
    assert_close(green.diagonal()[0], dense[0][0], 1.0e-13);
    assert_close(green.off_diagonal()[0], dense[0][1], 1.0e-13);

    assert!(matches!(
        ShiftedLdl::factor(&matrix, f64::NAN),
        Err(ResolventError::InvalidShift { .. })
    ));
}
