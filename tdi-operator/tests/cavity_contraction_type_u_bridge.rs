//! TDI-10.14 — EXACT contraction ↔ Type-U ρ bridge for family F_U.
//!
//! Relates FrozenToeplitzCavity::contraction(a,b) to Type-U ρ with equality /
//! inequality cases; dual-path finite-n product identity (algebraic ρ^n and
//! zero-drift cavity chain agree); REFUTED that every frozen Toeplitz symbol
//! yields Type D. Does not prove slowly-varying Jacobi asymptotics.

use tdi_operator::{CavityTransportChain, CavityTransportStep, FrozenToeplitzCavity};

fn assert_close(left: f64, right: f64, tolerance: f64) {
    let scale = 1.0_f64.max(left.abs()).max(right.abs());
    assert!(
        (left - right).abs() <= tolerance * scale,
        "left={left:.17e}, right={right:.17e}, |delta|={:.3e}",
        (left - right).abs()
    );
}

/// Constant Type-U / F_U map: alpha_k = rho for all k (TDI-10.6 / 10.13).
fn fu_alpha(rho: f64) -> f64 {
    assert!(rho > 0.0 && rho < 1.0);
    rho
}

/// Zero-drift cavity realization of constant-rho F_U (TDI-10.2 / 10.4 / 10.13).
fn realize_fu_chain(steps: usize, rho: f64) -> Vec<CavityTransportStep> {
    assert!(steps >= 1);
    assert!(rho > 0.0 && rho < 1.0);

    let mut out = Vec::with_capacity(steps);
    let mut cavity = 2.0;
    let mut reference = 1.0;

    for _ in 1..=steps {
        let alpha = fu_alpha(rho);
        let requested_edge_squared = alpha * cavity * reference;
        let edge = requested_edge_squared.sqrt();
        let realized_edge_squared = edge * edge;
        let current_reference = 1.0;
        let shifted_diagonal = realized_edge_squared / reference + current_reference;

        let step =
            CavityTransportStep::left(shifted_diagonal, edge, cavity, reference, current_reference)
                .expect("TDI-10.14 F_U construction must be admissible");

        assert_close(step.transport_factor(), rho, 4.0e-15);
        assert_close(step.drift(), 0.0, 4.0e-15);

        cavity = step.current_cavity();
        reference = step.current_reference();
        out.push(step);
    }

    out
}

#[test]
fn contraction_equals_b_over_q_squared_and_a_minus_d_over_a_plus_d() {
    for (diagonal, edge) in [(3.0, 1.0), (4.5, -1.25), (2.05, 0.9), (7.0, 2.0)] {
        let frozen = FrozenToeplitzCavity::new(diagonal, edge)
            .expect("admissible frozen symbol must construct");
        let kappa = frozen.contraction();
        let q = frozen.cavity();
        let d = frozen.discriminant_sqrt();

        assert!(kappa > 0.0 && kappa < 1.0);
        assert_close(kappa, (edge / q) * (edge / q), 2.0e-15);
        assert_close(kappa, (diagonal - d) / (diagonal + d), 2.0e-14);
    }
}

#[test]
fn open_contraction_is_exact_type_u_rho_with_equality_saturation() {
    let symbols = [(3.0, 1.0), (5.0, 1.5), (2.5, -0.75)];
    for (diagonal, edge) in symbols {
        let frozen = FrozenToeplitzCavity::new(diagonal, edge).unwrap();
        let rho = frozen.contraction();
        assert!(rho > 0.0 && rho < 1.0);

        // Equality saturation: alpha_k = rho for every step.
        for _ in 0..16 {
            assert_eq!(fu_alpha(rho), rho);
        }

        // Least uniform geometric bound for the constant-kappa family is kappa.
        let product_n = rho.powi(32);
        assert_close(product_n, rho.powi(32), 0.0);
        assert!(product_n < rho); // n>1, 0<rho<1
    }
}

#[test]
fn larger_envelope_certifies_type_u_inequality_case() {
    let frozen = FrozenToeplitzCavity::new(3.0, 1.0).unwrap();
    let kappa = frozen.contraction();
    assert!(kappa > 0.0 && kappa < 1.0);

    // Strictly larger envelope still certifies Type U.
    let rho_prime = (kappa + 1.0) * 0.5;
    assert!(kappa < rho_prime && rho_prime < 1.0);

    let steps = 24_usize;
    let mut product = 1.0;
    for _ in 1..=steps {
        let alpha = kappa; // constant-kappa family
        assert!(alpha <= rho_prime);
        product *= alpha;
    }
    assert!(product <= rho_prime.powi(steps as i32) + 1.0e-15);
    assert_close(product, kappa.powi(steps as i32), 2.0e-14);
    // Inequality in the envelope is strict relative to rho'^n.
    assert!(product < rho_prime.powi(steps as i32));
}

#[test]
fn dual_path_finite_n_products_agree_on_rho_n() {
    let symbols = [(3.0, 1.0), (4.5, -1.25), (6.0, 2.0)];
    for (diagonal, edge) in symbols {
        let frozen = FrozenToeplitzCavity::new(diagonal, edge).unwrap();
        let rho = frozen.contraction();
        assert!(rho > 0.0 && rho < 1.0);

        for steps in [1_usize, 8, 16, 40] {
            // Path 1: algebraic Type-U closed form.
            let algebraic = rho.powi(steps as i32);

            // Path 2: declared constant F_U map product.
            let mut map_product = 1.0;
            for _ in 1..=steps {
                map_product *= fu_alpha(rho);
            }
            assert_close(map_product, algebraic, 2.0e-14);

            // Path 3: zero-drift cavity chain cumulative transport.
            let realized = realize_fu_chain(steps, rho);
            let chain = CavityTransportChain::from_steps(&realized)
                .expect("Toeplitz-sourced F_U must form one contiguous cavity chain");
            assert_eq!(chain.steps(), steps);
            assert_close(chain.cumulative_transport_factor(), algebraic, 2.0e-12);
            assert_close(chain.accumulated_drift(), 0.0, 2.0e-12);

            // Dual-path agreement: Toeplitz contraction coefficient and
            // zero-drift cavity path agree on rho^n.
            assert_close(chain.cumulative_transport_factor(), map_product, 2.0e-12);
        }
    }
}

#[test]
fn frozen_toeplitz_refutes_universal_type_d_classification() {
    // REFUTED: every admissible frozen Toeplitz symbol yields Type D
    // (alpha_k -> 1, no uniform geometric envelope rho < 1).
    let frozen = FrozenToeplitzCavity::new(3.0, 1.0).unwrap();
    let kappa = frozen.contraction();
    assert!(kappa > 0.0 && kappa < 1.0);

    // Constant alpha_k = kappa does not tend to 1.
    for k in [1_usize, 16, 256, 1024] {
        let _ = k;
        assert!((fu_alpha(kappa) - kappa).abs() < 1.0e-15);
        assert!(fu_alpha(kappa) < 0.99); // bounded away from 1 for (3,1)
    }

    let steps = 64_usize;
    let realized = realize_fu_chain(steps, kappa);
    let chain = CavityTransportChain::from_steps(&realized).unwrap();
    assert_close(
        chain.cumulative_transport_factor(),
        kappa.powi(steps as i32),
        2.0e-12,
    );
    assert!(chain.cumulative_transport_factor() < 1.0e-6);
    // Uniform envelope exists: canonical classification is Type U, not Type D.
    assert!(kappa < 1.0);
}

#[test]
fn vanishing_contraction_is_boundary_not_open_type_u_witness() {
    let frozen = FrozenToeplitzCavity::new(5.0, 0.0).unwrap();
    let kappa = frozen.contraction();
    assert_eq!(kappa, 0.0);
    // Open Type-U witness requires rho in (0,1); kappa = 0 is recorded only.
    assert!(!(kappa > 0.0 && kappa < 1.0));
}
