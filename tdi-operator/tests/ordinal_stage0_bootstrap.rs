//! TDI-12.0 Stage-0 — EXACT ordinal ranking + candidate Green observables.
//!
//! Claims:
//! - EXACT average-rank / Spearman / Kendall τ-b on finite sequences;
//! - EXACT strictly-increasing affine invariance of both rank correlations;
//! - EXACT identity ordering on nondegenerate samples;
//! - EXACT fail-closed full-tie rejection;
//! - EXACT reverse-order Spearman/Kendall = −1 on distinct samples;
//! - EXACT wiring of candidate observables to public TDI-10 `GreenBands`;
//! - EXACT coefficient-only Frobenius-norm and Gershgorin-margin controls;
//! - EXACT deterministic shuffle destroys ρ = 1 on a nondegenerate length≥3 sample;
//! - EXACT closed-form constant-Toeplitz Frobenius / Gershgorin controls;
//! - EXACT Frobenius↔dimension concordance on a positive Toeplitz width ladder;
//! - EXACT Gershgorin width-invariance (n≥3) REFUTES covert dimension keying;
//! - EXACT rank-normalize / negate-response Stage-0 normalization scaffolding;
//! - EXACT tie-heavy adversarial midranks with non-degenerate Spearman;
//! - EXACT diagonal-only Frobenius / Gershgorin closed forms (edge-zero case);
//! - EXACT diagonal-only Green closed forms; MidDiagonalGreen width-invariant REFUTE;
//! - EXACT GreenTrace↔dimension concordance on DiagonalOnlyWidthLadder;
//! - EXACT MeanAbsOffDiagonalGreen ≡ 0 REFUTES dimension keying on diagonal-only;
//! - candidate Green response observables from TDI-10 primitives only;
//! - dimension-only control ranking does not consult Green values;
//! - no confirmatory TDI-12 execution is authorized.

use tdi_operator::{
    CandidateNormalization, CandidateOperatorPopulation, CandidateResponseObservable, GreenBands,
    JacobiMatrix, OrdinalError, average_ranks, coefficient_frobenius_norm_key,
    constant_diagonal_only_frobenius_norm, constant_diagonal_only_gershgorin_margin,
    constant_diagonal_only_green_trace, constant_diagonal_only_mid_diagonal_green,
    constant_toeplitz_frobenius_norm, constant_toeplitz_gershgorin_margin, deterministic_shuffle,
    dimension_only_key, evaluate_observable_ladder, gershgorin_dominance_margin_key,
    identity_ordering_key, kendall_tau_b, negate_values, rank_normalize, spearman_rho,
    strictly_increasing_affine, tie_heavy_adversarial_sample,
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

fn diagonal_only(n: usize, diagonal: f64) -> JacobiMatrix {
    toeplitz(n, diagonal, 0.0)
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
fn exact_reverse_order_unit_anticorrelation() {
    let ascending = [0.0, 1.0, 2.0, 3.0, 4.0];
    let descending = [4.0, 3.0, 2.0, 1.0, 0.0];
    assert_close(
        spearman_rho(&ascending, &descending).unwrap(),
        -1.0,
        1.0e-15,
    );
    assert_close(
        kendall_tau_b(&ascending, &descending).unwrap(),
        -1.0,
        1.0e-15,
    );
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
fn candidate_observables_wire_exactly_to_public_green_bands() {
    // EXACT wiring: CandidateResponseObservable equals the public TDI-10
    // GreenBands extractors already allowed by Stage-0. No new operator
    // semantics and no freeze of response_observable_registry.
    let matrix = toeplitz(6, 4.0, 1.25);
    let shift = 0.75;
    let bands = GreenBands::compute(&matrix, shift).expect("positive Toeplitz admits GreenBands");

    for observable in [
        CandidateResponseObservable::MidDiagonalGreen,
        CandidateResponseObservable::GreenTrace,
        CandidateResponseObservable::MeanAbsOffDiagonalGreen,
    ] {
        let via_evaluate = observable.evaluate(&matrix, shift).unwrap();
        let via_bands = observable.from_green_bands(&bands).unwrap();
        assert_close(via_evaluate, via_bands, 0.0);

        let expected = match observable {
            CandidateResponseObservable::MidDiagonalGreen => bands.diagonal()[matrix.len() / 2],
            CandidateResponseObservable::GreenTrace => bands.diagonal().iter().sum::<f64>(),
            CandidateResponseObservable::MeanAbsOffDiagonalGreen => {
                let off = bands.off_diagonal();
                off.iter().map(|v| v.abs()).sum::<f64>() / (off.len() as f64)
            }
        };
        assert_close(via_evaluate, expected, 0.0);
        assert_eq!(
            observable.as_str(),
            match observable {
                CandidateResponseObservable::MidDiagonalGreen => "MidDiagonalGreen",
                CandidateResponseObservable::GreenTrace => "GreenTrace",
                CandidateResponseObservable::MeanAbsOffDiagonalGreen => {
                    "MeanAbsOffDiagonalGreen"
                }
            }
        );
    }

    // 1×1 operator: off-diagonal observable is exactly 0.
    let singleton = toeplitz(1, 5.0, 0.0);
    let off = CandidateResponseObservable::MeanAbsOffDiagonalGreen
        .evaluate(&singleton, 1.0)
        .unwrap();
    assert_eq!(off, 0.0);

    // Empty operator fails closed.
    let empty = JacobiMatrix::new(Vec::new(), Vec::new()).unwrap();
    assert!(matches!(
        CandidateResponseObservable::GreenTrace.evaluate(&empty, 1.0),
        Err(OrdinalError::EmptyOperator)
    ));
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
fn coefficient_norm_and_gershgorin_controls_ignore_green() {
    let weak = toeplitz(4, 3.0, 1.0); // margin = 3 - 1 - 1 = 1
    let strong = toeplitz(4, 6.0, 1.0); // margin = 6 - 1 - 1 = 4
    let shift = 0.5;

    let weak_norm = coefficient_frobenius_norm_key(&weak).unwrap();
    let strong_norm = coefficient_frobenius_norm_key(&strong).unwrap();
    assert!(strong_norm > weak_norm);

    let weak_margin = gershgorin_dominance_margin_key(&weak).unwrap();
    let strong_margin = gershgorin_dominance_margin_key(&strong).unwrap();
    assert_close(weak_margin, 1.0, 0.0);
    assert_close(strong_margin, 4.0, 0.0);

    // Distinct Green traces must not enter the coefficient-only keys.
    let weak_trace = CandidateResponseObservable::GreenTrace
        .evaluate(&weak, shift)
        .unwrap();
    let strong_trace = CandidateResponseObservable::GreenTrace
        .evaluate(&strong, shift)
        .unwrap();
    assert_ne!(weak_trace, strong_trace);

    // Same coefficients ⇒ same keys even if we never call Green.
    assert_eq!(coefficient_frobenius_norm_key(&weak).unwrap(), weak_norm);
    assert_eq!(
        gershgorin_dominance_margin_key(&strong).unwrap(),
        strong_margin
    );
}

#[test]
fn shuffled_family_control_destroys_perfect_correlation() {
    let original = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
    assert_close(spearman_rho(&original, &original).unwrap(), 1.0, 1.0e-15);

    let mut shuffled = original;
    deterministic_shuffle(&mut shuffled, 0x07d1_1200);
    assert_ne!(shuffled.as_slice(), original.as_slice());
    let rho = spearman_rho(&original, &shuffled).unwrap();
    let tau = kendall_tau_b(&original, &shuffled).unwrap();
    assert!(rho.is_finite() && tau.is_finite());
    assert!((rho - 1.0).abs() > 1.0e-12);
    assert!((tau - 1.0).abs() > 1.0e-12);

    // Re-running with the same seed is deterministic.
    let mut again = original;
    deterministic_shuffle(&mut again, 0x07d1_1200);
    assert_eq!(again.as_slice(), shuffled.as_slice());
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

#[test]
fn exact_constant_toeplitz_frobenius_closed_form_matches_key() {
    let a = 4.0;
    let b = 1.0;
    for n in 1..=8 {
        let matrix = toeplitz(n, a, b);
        let key = coefficient_frobenius_norm_key(&matrix).unwrap();
        let closed = constant_toeplitz_frobenius_norm(n, a, b).unwrap();
        assert_close(key, closed, 1.0e-15);
    }
}

#[test]
fn exact_frobenius_width_ladder_concordant_with_dimension() {
    let a = 4.0;
    let b = 1.0;
    let widths = [1usize, 2, 3, 4, 5, 6];
    let mut frobenius = Vec::new();
    let mut dimensions = Vec::new();
    for &n in &widths {
        let matrix = toeplitz(n, a, b);
        frobenius.push(coefficient_frobenius_norm_key(&matrix).unwrap());
        dimensions.push(dimension_only_key(&matrix));
        if n >= 2 {
            assert!(frobenius[frobenius.len() - 1] > frobenius[frobenius.len() - 2]);
        }
    }
    assert_close(spearman_rho(&frobenius, &dimensions).unwrap(), 1.0, 1.0e-15);
    assert_close(
        kendall_tau_b(&frobenius, &dimensions).unwrap(),
        1.0,
        1.0e-15,
    );
}

#[test]
fn exact_gershgorin_constant_toeplitz_width_invariant_refutes_dimension_key() {
    let a: f64 = 5.0;
    let b: f64 = 1.0;
    // Require a > 2|b| so the symbol is strictly positive in the TDI-10 sense.
    assert!(a > 2.0 * b.abs());
    let widths = [3usize, 4, 5, 6, 7];
    let mut margins = Vec::new();
    let mut dimensions = Vec::new();
    for &n in &widths {
        let matrix = toeplitz(n, a, b);
        let key = gershgorin_dominance_margin_key(&matrix).unwrap();
        let closed = constant_toeplitz_gershgorin_margin(n, a, b).unwrap();
        assert_close(key, closed, 1.0e-15);
        assert_close(closed, a - 2.0 * b.abs(), 1.0e-15);
        margins.push(key);
        dimensions.push(dimension_only_key(&matrix));
    }
    // All margins equal ⇒ rank correlation fail-closed (not a covert width key).
    assert!(matches!(
        spearman_rho(&margins, &dimensions),
        Err(OrdinalError::DegenerateRanks)
    ));
    assert!(matches!(
        kendall_tau_b(&margins, &dimensions),
        Err(OrdinalError::DegenerateRanks)
    ));
}

#[test]
fn exact_rank_normalize_and_negate_response_scaffolding() {
    let values = [-2.0, 0.5, 0.5, 3.0, 9.0];
    let ranks = rank_normalize(&values).unwrap();
    assert_close(spearman_rho(&values, &ranks).unwrap(), 1.0, 1.0e-15);
    assert_close(kendall_tau_b(&values, &ranks).unwrap(), 1.0, 1.0e-15);

    let distinct = [1.0, 2.0, 3.0, 4.0, 5.0];
    let negated = negate_values(&distinct).unwrap();
    assert_close(spearman_rho(&distinct, &negated).unwrap(), -1.0, 1.0e-15);
    assert_close(kendall_tau_b(&distinct, &negated).unwrap(), -1.0, 1.0e-15);

    assert_eq!(
        CandidateNormalization::IdentityResponse.as_str(),
        "identity_response"
    );
    assert_eq!(
        CandidateNormalization::RankNormalizeToAverageRanks.as_str(),
        "rank_normalize_to_average_ranks"
    );
    assert_eq!(
        CandidateNormalization::NegateResponse.as_str(),
        "negate_response"
    );
}

#[test]
fn exact_tie_heavy_adversarial_control_nondegenerate() {
    let sample = tie_heavy_adversarial_sample(6).unwrap();
    assert_eq!(sample, vec![0.0, 1.0, 1.0, 1.0, 1.0, 2.0]);
    // Midranks: 1, then block positions 2..=5 → midrank 3.5, then 6.
    assert_eq!(
        average_ranks(&sample).unwrap(),
        vec![1.0, 3.5, 3.5, 3.5, 3.5, 6.0]
    );
    assert_close(spearman_rho(&sample, &sample).unwrap(), 1.0, 1.0e-15);
    assert_close(kendall_tau_b(&sample, &sample).unwrap(), 1.0, 1.0e-15);
}

#[test]
fn exact_observable_ladder_and_population_candidate_ids() {
    let shift = 1.0;
    let matrices = vec![
        toeplitz(2, 4.0, 1.0),
        toeplitz(3, 4.0, 1.0),
        toeplitz(4, 4.0, 1.0),
    ];
    let ladder = evaluate_observable_ladder(
        &matrices,
        shift,
        CandidateResponseObservable::MidDiagonalGreen,
    )
    .unwrap();
    assert_eq!(ladder.len(), 3);
    for (matrix, value) in matrices.iter().zip(ladder.iter()) {
        assert_close(
            *value,
            CandidateResponseObservable::MidDiagonalGreen
                .evaluate(matrix, shift)
                .unwrap(),
            1.0e-15,
        );
    }
    assert!(matches!(
        evaluate_observable_ladder(&[], shift, CandidateResponseObservable::GreenTrace),
        Err(OrdinalError::EmptySample)
    ));

    assert_eq!(
        CandidateOperatorPopulation::PositiveConstantToeplitzWidthLadder.as_str(),
        "PositiveConstantToeplitzWidthLadder"
    );
    assert_eq!(
        CandidateOperatorPopulation::DiagonalOnlyWidthLadder.as_str(),
        "DiagonalOnlyWidthLadder"
    );
}

#[test]
fn exact_constant_diagonal_only_frobenius_closed_form_matches_key() {
    let a = 3.0;
    for n in 1..=8 {
        let matrix = diagonal_only(n, a);
        let key = coefficient_frobenius_norm_key(&matrix).unwrap();
        let closed = constant_diagonal_only_frobenius_norm(n, a).unwrap();
        assert_close(key, closed, 1.0e-15);
        assert_close(closed, a * (n as f64).sqrt(), 1.0e-15);
        // Edge-zero specialization of the Toeplitz closed form.
        assert_close(
            closed,
            constant_toeplitz_frobenius_norm(n, a, 0.0).unwrap(),
            1.0e-15,
        );
    }
}

#[test]
fn exact_diagonal_only_frobenius_width_ladder_concordant_with_dimension() {
    let a = 3.0;
    let widths = [1usize, 2, 3, 4, 5, 6];
    let mut frobenius = Vec::new();
    let mut dimensions = Vec::new();
    for &n in &widths {
        let matrix = diagonal_only(n, a);
        frobenius.push(coefficient_frobenius_norm_key(&matrix).unwrap());
        dimensions.push(dimension_only_key(&matrix));
        if n >= 2 {
            assert!(frobenius[frobenius.len() - 1] > frobenius[frobenius.len() - 2]);
        }
    }
    assert_close(spearman_rho(&frobenius, &dimensions).unwrap(), 1.0, 1.0e-15);
    assert_close(
        kendall_tau_b(&frobenius, &dimensions).unwrap(),
        1.0,
        1.0e-15,
    );
}

#[test]
fn exact_diagonal_only_gershgorin_width_invariant_refutes_dimension_key() {
    let a: f64 = 5.0;
    // Stronger than Toeplitz: invariant for ALL n ≥ 1, not merely n ≥ 3.
    let widths = [1usize, 2, 3, 4, 5, 6, 7];
    let mut margins = Vec::new();
    let mut dimensions = Vec::new();
    for &n in &widths {
        let matrix = diagonal_only(n, a);
        let key = gershgorin_dominance_margin_key(&matrix).unwrap();
        let closed = constant_diagonal_only_gershgorin_margin(n, a).unwrap();
        assert_close(key, closed, 1.0e-15);
        assert_close(closed, a, 1.0e-15);
        margins.push(key);
        dimensions.push(dimension_only_key(&matrix));
    }
    assert!(matches!(
        spearman_rho(&margins, &dimensions),
        Err(OrdinalError::DegenerateRanks)
    ));
    assert!(matches!(
        kendall_tau_b(&margins, &dimensions),
        Err(OrdinalError::DegenerateRanks)
    ));
}

#[test]
fn exact_diagonal_only_green_closed_forms_and_width_invariance() {
    let a = 4.0;
    let shift = 1.0;
    let closed_mid = constant_diagonal_only_mid_diagonal_green(a, shift).unwrap();
    assert_close(closed_mid, 1.0 / (a + shift), 1.0e-15);

    let widths = [1usize, 2, 3, 4, 5, 6];
    let mut mids = Vec::new();
    let mut traces = Vec::new();
    let mut offs = Vec::new();
    let mut dimensions = Vec::new();
    for &n in &widths {
        let matrix = diagonal_only(n, a);
        let bands = GreenBands::compute(&matrix, shift).unwrap();
        // Every diagonal Green entry equals 1/(a+shift); off-diagonals vanish.
        for entry in bands.diagonal() {
            assert_close(*entry, closed_mid, 1.0e-15);
        }
        for entry in bands.off_diagonal() {
            assert_close(*entry, 0.0, 1.0e-15);
        }

        let mid = CandidateResponseObservable::MidDiagonalGreen
            .evaluate(&matrix, shift)
            .unwrap();
        let trace = CandidateResponseObservable::GreenTrace
            .evaluate(&matrix, shift)
            .unwrap();
        let off = CandidateResponseObservable::MeanAbsOffDiagonalGreen
            .evaluate(&matrix, shift)
            .unwrap();
        assert_close(mid, closed_mid, 1.0e-15);
        assert_close(
            trace,
            constant_diagonal_only_green_trace(n, a, shift).unwrap(),
            1.0e-15,
        );
        assert_close(trace, (n as f64) * closed_mid, 1.0e-15);
        assert_close(off, 0.0, 1.0e-15);

        mids.push(mid);
        traces.push(trace);
        offs.push(off);
        dimensions.push(dimension_only_key(&matrix));
    }

    // MidDiagonalGreen is width-invariant ⇒ REFUTES covert dimension keying.
    assert!(matches!(
        spearman_rho(&mids, &dimensions),
        Err(OrdinalError::DegenerateRanks)
    ));
    assert!(matches!(
        kendall_tau_b(&mids, &dimensions),
        Err(OrdinalError::DegenerateRanks)
    ));

    // GreenTrace is strictly monotone in width ⇒ ρ = τ = 1 vs dimension.
    assert_close(spearman_rho(&traces, &dimensions).unwrap(), 1.0, 1.0e-15);
    assert_close(kendall_tau_b(&traces, &dimensions).unwrap(), 1.0, 1.0e-15);

    // MeanAbsOffDiagonalGreen ≡ 0 ⇒ REFUTES covert dimension keying.
    assert!(matches!(
        spearman_rho(&offs, &dimensions),
        Err(OrdinalError::DegenerateRanks)
    ));
    assert!(matches!(
        kendall_tau_b(&offs, &dimensions),
        Err(OrdinalError::DegenerateRanks)
    ));

    assert_eq!(
        CandidateOperatorPopulation::DiagonalOnlyWidthLadder.as_str(),
        "DiagonalOnlyWidthLadder"
    );
}
