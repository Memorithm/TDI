use std::f64::consts::TAU;

use tdi_operator::{FrozenToeplitzCavity, FrozenToeplitzError};

fn assert_close(left: f64, right: f64, tolerance: f64) {
    let scale = 1.0_f64.max(left.abs()).max(right.abs());
    assert!(
        (left - right).abs() <= tolerance * scale,
        "left={left:.17e}, right={right:.17e}, |delta|={:.3e}",
        (left - right).abs()
    );
}

/// Independent numerical oracle for the two-sided constant Jacobi Green bands.
///
/// This midpoint Fourier quadrature does not use the closed-form cavity or
/// Green formulas implemented by `FrozenToeplitzCavity`.
fn fourier_green_oracle(diagonal: f64, edge: f64) -> (f64, f64) {
    const SAMPLES: usize = 65_536;
    let (diagonal_sum, off_diagonal_sum) =
        (0..SAMPLES).fold((0.0_f64, 0.0_f64), |(d_sum, o_sum), sample| {
            let theta = TAU * (sample as f64 + 0.5) / SAMPLES as f64;
            let cosine = theta.cos();
            let symbol = diagonal + 2.0 * edge * cosine;
            (d_sum + 1.0 / symbol, o_sum + cosine / symbol)
        });
    (
        diagonal_sum / SAMPLES as f64,
        off_diagonal_sum / SAMPLES as f64,
    )
}

#[test]
fn frozen_cavity_satisfies_exact_defining_identities_numerically() {
    for (diagonal, edge) in [(3.0, 1.0), (4.5, -1.25), (2.01, 1.0), (7.0, 0.0)] {
        let frozen = FrozenToeplitzCavity::new(diagonal, edge).unwrap();
        let q = frozen.cavity();
        let kappa = frozen.contraction();

        assert!(q > 0.0);
        assert!(frozen.discriminant_sqrt() > 0.0);
        assert!((0.0..1.0).contains(&kappa));

        assert_close(q, diagonal - edge * edge / q, 2.0e-14);
        assert_close(kappa, (edge / q) * (edge / q), 2.0e-15);
        assert_close(
            diagonal * frozen.green_diagonal() + 2.0 * edge * frozen.green_off_diagonal(),
            1.0,
            3.0e-14,
        );
        assert_close(
            frozen.green_off_diagonal(),
            -edge * frozen.green_diagonal() / q,
            2.0e-15,
        );
    }
}

#[test]
fn closed_form_green_bands_match_independent_fourier_quadrature() {
    for (diagonal, edge) in [(3.0, 1.0), (4.5, -1.25), (2.05, 1.0), (7.0, 0.0)] {
        let frozen = FrozenToeplitzCavity::new(diagonal, edge).unwrap();
        let (oracle_diagonal, oracle_off_diagonal) = fourier_green_oracle(diagonal, edge);
        assert_close(frozen.green_diagonal(), oracle_diagonal, 2.0e-11);
        assert_close(frozen.green_off_diagonal(), oracle_off_diagonal, 2.0e-11);
    }
}

#[test]
fn strict_symbol_positivity_is_stronger_than_positive_discriminant() {
    // a=-3, b=1 has a^2-4b^2=5>0, but the constant Jacobi symbol is
    // strictly negative rather than positive. It must not enter the positive
    // frozen model merely because the quadratic discriminant is positive.
    assert_eq!(
        FrozenToeplitzCavity::new(-3.0, 1.0),
        Err(FrozenToeplitzError::NonPositiveSymbol {
            diagonal: -3.0,
            edge: 1.0,
        })
    );
    assert!(matches!(
        FrozenToeplitzCavity::new(2.0, 1.0),
        Err(FrozenToeplitzError::NonPositiveSymbol { .. })
    ));
    assert!(matches!(
        FrozenToeplitzCavity::new(1.0, 0.6),
        Err(FrozenToeplitzError::NonPositiveSymbol { .. })
    ));
}

#[test]
fn edge_sign_changes_only_the_first_off_diagonal_green_sign() {
    let positive = FrozenToeplitzCavity::new(5.0, 1.25).unwrap();
    let negative = FrozenToeplitzCavity::new(5.0, -1.25).unwrap();

    assert_eq!(positive.discriminant_sqrt(), negative.discriminant_sqrt());
    assert_eq!(positive.cavity(), negative.cavity());
    assert_eq!(positive.green_diagonal(), negative.green_diagonal());
    assert_eq!(positive.contraction(), negative.contraction());
    assert_eq!(
        positive.green_off_diagonal(),
        -negative.green_off_diagonal()
    );
}

#[test]
fn scaled_discriminant_avoids_unnecessary_large_coefficient_overflow() {
    let frozen = FrozenToeplitzCavity::new(1.0e308, 1.0e307).unwrap();
    assert!(frozen.discriminant_sqrt().is_finite());
    assert!(frozen.cavity().is_finite());
    assert!(frozen.green_diagonal().is_finite());
    assert!(frozen.green_off_diagonal().is_finite());
    assert!(frozen.contraction().is_finite());
    assert!((0.0..1.0).contains(&frozen.contraction()));
}

#[test]
fn non_finite_inputs_and_unrepresentable_green_data_fail_closed() {
    assert!(matches!(
        FrozenToeplitzCavity::new(f64::INFINITY, 1.0),
        Err(FrozenToeplitzError::NonFiniteDiagonal { .. })
    ));
    assert!(matches!(
        FrozenToeplitzCavity::new(3.0, f64::NAN),
        Err(FrozenToeplitzError::NonFiniteEdge { .. })
    ));

    // The exact symbol remains positive, but the f64 Green value overflows as
    // the representable gap collapses to the smallest positive subnormal range.
    let tiny = f64::from_bits(1);
    assert!(matches!(
        FrozenToeplitzCavity::new(tiny, 0.0),
        Err(FrozenToeplitzError::NonFiniteDerivedQuantity)
    ));
}
