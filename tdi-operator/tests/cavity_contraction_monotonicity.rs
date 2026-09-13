//! TDI-10.16 — EXACT κ(a,b) domain / monotonicity + frozen factorization Type-U link.
//!
//! Domain calculus (evenness, homogeneity, ratio reduction), radial
//! monotonicity, matched TDI-10.3 transport factor = κ = Type-U ρ;
//! REFUTED that κ → 0 at the positivity boundary; REFUTED that larger
//! diagonal alone forces smaller κ without fixing |b|.

use tdi_operator::{CavityDriftFactorization, CavityTransportStep, FrozenToeplitzCavity};

fn assert_close(left: f64, right: f64, tolerance: f64) {
    let scale = 1.0_f64.max(left.abs()).max(right.abs());
    assert!(
        (left - right).abs() <= tolerance * scale,
        "left={left:.17e}, right={right:.17e}, |delta|={:.3e}",
        (left - right).abs()
    );
}

fn kappa_star(r: f64) -> f64 {
    assert!((0.0..1.0).contains(&r));
    let s = (1.0 - r * r).sqrt();
    (1.0 - s) / (1.0 + s)
}

#[test]
fn evenness_homogeneity_and_ratio_reduction() {
    let samples = [
        (3.0, 1.0),
        (5.0, 1.5),
        (2.5, -0.75),
        (8.0, 0.0),
        (4.0, -1.2),
    ];
    for (diagonal, edge) in samples {
        let frozen = FrozenToeplitzCavity::new(diagonal, edge)
            .expect("admissible frozen symbol must construct");
        let kappa = frozen.contraction();
        assert!((0.0..1.0).contains(&kappa));

        // Evenness in b.
        let flipped = FrozenToeplitzCavity::new(diagonal, -edge).unwrap();
        assert_close(flipped.contraction(), kappa, 1.0e-15);

        // Positive homogeneity.
        for lambda in [0.5_f64, 2.0, 10.0] {
            let scaled = FrozenToeplitzCavity::new(lambda * diagonal, lambda * edge).unwrap();
            assert_close(scaled.contraction(), kappa, 2.0e-14);
        }

        // Ratio reduction: κ = (1-s)/(1+s) with r = 2|b|/a.
        let r = 2.0 * edge.abs() / diagonal;
        assert!((0.0..1.0).contains(&r));
        assert_close(kappa, kappa_star(r), 2.0e-14);
    }
}

#[test]
fn radial_map_is_strictly_increasing_and_cone_monotone() {
    let radii = [0.0_f64, 0.1, 0.25, 0.5, 0.75, 0.9, 0.99];
    for window in radii.windows(2) {
        let (r0, r1) = (window[0], window[1]);
        assert!(kappa_star(r0) < kappa_star(r1));
    }

    // Fixed nonzero edge: larger a → smaller κ.
    let edge = 1.0_f64;
    let diagonals = [2.5_f64, 3.0, 4.0, 6.0, 12.0];
    let mut previous = f64::INFINITY;
    for &diagonal in &diagonals {
        assert!(diagonal > 2.0 * edge);
        let kappa = FrozenToeplitzCavity::new(diagonal, edge)
            .unwrap()
            .contraction();
        assert!(kappa < previous);
        previous = kappa;
    }

    // Fixed diagonal: larger |b| → larger κ.
    let diagonal = 5.0_f64;
    let edges = [0.0_f64, 0.5, 1.0, 1.5, 2.0];
    previous = -1.0;
    for &edge in &edges {
        assert!(diagonal > 2.0 * edge.abs());
        let kappa = FrozenToeplitzCavity::new(diagonal, edge)
            .unwrap()
            .contraction();
        assert!(kappa > previous);
        previous = kappa;
    }
}

#[test]
fn matched_frozen_factorization_transport_equals_type_u_kappa() {
    let symbols = [(3.0, 1.0), (4.5, -1.25), (6.0, 2.0)];
    for (diagonal, edge) in symbols {
        let frozen = FrozenToeplitzCavity::new(diagonal, edge).unwrap();
        let kappa = frozen.contraction();
        assert!(kappa > 0.0 && kappa < 1.0);
        let q = frozen.cavity();

        let step = CavityTransportStep::left(diagonal, edge, q, q, q)
            .expect("matched frozen step must be admissible");
        let factors = CavityDriftFactorization::new(step, edge)
            .expect("matched factorization must be admissible");

        assert_close(step.drift(), 0.0, 4.0e-15);
        assert_close(factors.reference_defect(), 0.0, 4.0e-15);
        assert_close(factors.edge_drift(), 0.0, 4.0e-15);
        assert_close(factors.reference_drift(), 0.0, 4.0e-15);
        assert_close(factors.cavity_correction(), 1.0, 2.0e-15);
        assert_close(factors.normalized_edge_square(), kappa, 3.0e-15);
        assert_close(factors.reconstructed_transport_factor(), kappa, 3.0e-15);
        assert_close(step.transport_factor(), kappa, 3.0e-15);
    }
}

#[test]
fn refuted_kappa_vanishes_at_positivity_boundary() {
    // REFUTED: κ → 0 as a ↓ 2|b|+. Near-boundary symbol has κ close to 1.
    let near = FrozenToeplitzCavity::new(2.0001, 1.0).unwrap();
    let far = FrozenToeplitzCavity::new(10.0, 1.0).unwrap();
    let kappa_near = near.contraction();
    let kappa_far = far.contraction();

    assert!(kappa_near > 0.9);
    assert!(kappa_near < 1.0);
    assert!(kappa_far < 0.05);
    assert!(kappa_near > kappa_far);

    // Reduced-map endpoint: r close to 1 ⇒ κ_* close to 1.
    let r_near = 2.0 / 2.0001;
    assert!(r_near > 0.9999);
    assert_close(kappa_near, kappa_star(r_near), 2.0e-12);
    assert!(kappa_star(r_near) > 0.9);
}

#[test]
fn refuted_larger_diagonal_alone_forces_smaller_kappa() {
    // REFUTED: larger a alone forces smaller κ without fixing |b|.
    let small_a = FrozenToeplitzCavity::new(3.0, 1.0).unwrap().contraction();
    let large_a = FrozenToeplitzCavity::new(10.0, 4.0).unwrap().contraction();

    let diagonal_small = 3.0_f64;
    let diagonal_large = 10.0_f64;
    assert!(diagonal_large > diagonal_small);
    assert!(large_a > small_a);
    assert_close(large_a, 0.25, 1.0e-14);

    // Controlling quantity is r = 2|b|/a, not a alone.
    let r_small_a = 2.0 * 1.0 / diagonal_small;
    let r_large_a = 2.0 * 4.0 / diagonal_large;
    assert!(r_large_a > r_small_a);
}
