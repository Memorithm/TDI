//! TDI-12.0 Stage-0 — EXACT ordinal ranking + candidate Green observables.
//!
//! Claims:
//! - EXACT average-rank / Spearman / Kendall τ-b on finite sequences;
//! - EXACT strictly-increasing affine invariance of both rank correlations;
//! - EXACT identity ordering on nondegenerate samples;
//! - EXACT fail-closed full-tie rejection;
//! - candidate Green response observables from TDI-10 primitives only;
//! - dimension-only control ranking does not consult Green values;
//! - no confirmatory TDI-12 execution is authorized.

use tdi_operator::{
    CandidateResponseObservable, JacobiMatrix, OrdinalError, average_ranks, dimension_only_key,
    identity_ordering_key, kendall_tau_b, spearman_rho, strictly_increasing_affine,
};

fn assert_close(left: f64, right: f64, tol: f64) {
    let scale = 1.0_f64.max(left.abs()).max(right.abs());
    assert!(
        (left - right).abs() <= tol * scale,
        "left={left:.17e} right={right:.17e} |Δ|={:.3e}",
        (left - right).abs()
    );
}

fn toeplitz(n: usize, diagonal: f64, edge: f64) -> JacobiMatrix {
    assert!(n >= 1);
    let diag = vec![diagonal; n];
    let off = if n == 1 {
        Vec::new()
    } else {
        vec![edge; n - 1]
    };
    JacobiMatrix::new(diag, off).expect("synthetic Toeplitz must be admissible")
}

#[test]
fn exact_average_ranks_and_spearman_identity() {
    let values = [4.0, 1.0, 3.0, 2.0, 2.0];
    assert_eq!(
        average_ranks(&values).unwrap(),
        vec![5.0, 1.0, 4.0, 2.5, 2.5]
    );
    assert_close(spearman_rho(&values, &values).unwrap(), 1.0, 1.0e-15);
    assert_close(kendall_tau_b(&values, &values).unwrap(), 1.0, 1.0e-15);
}

#[test]
fn exact_monotone_affine_invariance() {
    let left = [-2.0, 0.5, 0.5, 3.0, 9.0];
    let right = [1.0, 4.0, 2.0, 8.0, 0.0];
    let mapped = strictly_increasing_affine(&left, 4.0, 7.0).unwrap();
    assert_close(
        spearman_rho(&left, &right).unwrap(),
        spearman_rho(&mapped, &right).unwrap(),
        1.0e-14,
    );
    assert_close(
        kendall_tau_b(&left, &right).unwrap(),
        kendall_tau_b(&mapped, &right).unwrap(),
        1.0e-14,
    );
}

#[test]
fn exact_full_tie_fail_closed() {
    let tied = [1.5, 1.5, 1.5, 1.5];
    assert!(matches!(
        spearman_rho(&tied, &tied),
        Err(OrdinalError::DegenerateRanks)
    ));
    assert!(matches!(
        kendall_tau_b(&tied, &tied),
        Err(OrdinalError::DegenerateRanks)
    ));
}

#[test]
fn candidate_green_observables_are_finite_on_positive_toeplitz() {
    let matrix = toeplitz(5, 3.0, 1.0);
    let shift = 1.0;
    for observable in [
        CandidateResponseObservable::MidDiagonalGreen,
        CandidateResponseObservable::GreenTrace,
        CandidateResponseObservable::MeanAbsOffDiagonalGreen,
    ] {
        let value = observable.evaluate(&matrix, shift).unwrap();
        assert!(
            value.is_finite(),
            "{observable:?} produced non-finite {value}"
        );
    }
}

#[test]
fn dimension_only_control_ignores_green_values() {
    let small = toeplitz(3, 4.0, 1.0);
    let large = toeplitz(6, 4.0, 1.0);
    let shift = 0.5;

    let small_key = dimension_only_key(&small);
    let large_key = dimension_only_key(&large);
    assert!(small_key < large_key);

    // Distinct Green responses must not affect the dimension-only key.
    let small_response = CandidateResponseObservable::GreenTrace
        .evaluate(&small, shift)
        .unwrap();
    let large_response = CandidateResponseObservable::GreenTrace
        .evaluate(&large, shift)
        .unwrap();
    assert_ne!(small_response, large_response);
    assert_eq!(dimension_only_key(&small), 3.0);
    assert_eq!(dimension_only_key(&large), 6.0);
}

#[test]
fn identity_ordering_matches_response_ranking() {
    let matrices = [
        toeplitz(2, 3.0, 0.5),
        toeplitz(3, 3.0, 0.5),
        toeplitz(4, 3.0, 0.5),
        toeplitz(5, 3.0, 0.5),
    ];
    let shift = 1.0;
    let responses: Vec<f64> = matrices
        .iter()
        .map(|matrix| {
            CandidateResponseObservable::MidDiagonalGreen
                .evaluate(matrix, shift)
                .unwrap()
        })
        .collect();
    let identity_keys: Vec<f64> = responses
        .iter()
        .copied()
        .map(identity_ordering_key)
        .collect();
    assert_close(
        spearman_rho(&responses, &identity_keys).unwrap(),
        1.0,
        1.0e-15,
    );
}

#[test]
fn synthetic_population_rank_correlation_is_deterministic() {
    // Non-final synthetic Stage-0 battery: fixed Toeplitz family across widths.
    // This is EXACT ranking bookkeeping on TDI-10 Green values, not a
    // confirmatory ordinal-transport claim.
    let widths = [2_usize, 3, 4, 5, 6, 7, 8];
    let shift = 1.0;
    let dimensions: Vec<f64> = widths.iter().map(|&n| n as f64).collect();
    let traces: Vec<f64> = widths
        .iter()
        .map(|&n| {
            CandidateResponseObservable::GreenTrace
                .evaluate(&toeplitz(n, 3.0, 1.0), shift)
                .unwrap()
        })
        .collect();
    let mids: Vec<f64> = widths
        .iter()
        .map(|&n| {
            CandidateResponseObservable::MidDiagonalGreen
                .evaluate(&toeplitz(n, 3.0, 1.0), shift)
                .unwrap()
        })
        .collect();

    // Trace grows with dimension for this positive family, so dimension-only
    // and GreenTrace ranking agree exactly on this synthetic battery.
    assert_close(spearman_rho(&dimensions, &traces).unwrap(), 1.0, 1.0e-12);
    // Mid-diagonal Green need not be monotone in width; the Stage-0 claim is
    // only that the rank correlation is a deterministic finite number.
    let rho_mid = spearman_rho(&dimensions, &mids).unwrap();
    assert!(rho_mid.is_finite());
    let tau_mid = kendall_tau_b(&dimensions, &mids).unwrap();
    assert!(tau_mid.is_finite());
}
