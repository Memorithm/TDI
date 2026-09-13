//! TDI-10.18 — EXACT operator-family finite hypothesis checklist.
//!
//! Item D via TDI-10.9; Item T = TDI-10.2 one-step identity on family-realized
//! steps; REFUTED that informal alpha→1 meets Item D; REFUTED that a finite
//! harmonic window alone forces product → 0. Does not prove slowly-varying
//! Jacobi asymptotics.

use tdi_operator::{CavityTransportChain, CavityTransportStep, FrozenToeplitzCavity};

fn assert_close(left: f64, right: f64, tolerance: f64) {
    let scale = 1.0_f64.max(left.abs()).max(right.abs());
    assert!(
        (left - right).abs() <= tolerance * scale,
        "left={left:.17e}, right={right:.17e}, |delta|={:.3e}",
        (left - right).abs()
    );
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FamilyId {
    /// Type S: alpha_k = 1 - 1/(k+1)^2.
    FS,
    /// Type U: constant alpha_k = rho.
    FU,
    /// Type D: alpha_k = k/(k+1).
    FD,
}

fn family_alpha(family: FamilyId, step_index: usize, rho: f64) -> f64 {
    assert!(step_index >= 1);
    match family {
        FamilyId::FS => {
            let denominator = (step_index + 1) as f64;
            1.0 - 1.0 / (denominator * denominator)
        }
        FamilyId::FU => {
            assert!(rho > 0.0 && rho < 1.0);
            rho
        }
        FamilyId::FD => (step_index as f64) / ((step_index + 1) as f64),
    }
}

fn closed_product(family: FamilyId, steps: usize, rho: f64) -> f64 {
    match family {
        FamilyId::FS => ((steps + 2) as f64) / (2.0 * ((steps + 1) as f64)),
        FamilyId::FU => rho.powi(steps as i32),
        FamilyId::FD => 1.0 / ((steps + 1) as f64),
    }
}

/// Checklist Item D: exists L > 0 and K such that x_k >= L/k for all k >= K
/// on the inspected range [K, max_k]. Returns the first failing k, if any.
fn item_d_holds(alphas: &[(usize, f64)], ell: f64, k0: usize) -> Result<(), usize> {
    assert!(ell > 0.0);
    assert!(k0 >= 1);
    for &(k, alpha) in alphas {
        if k < k0 {
            continue;
        }
        let x = 1.0 - alpha;
        if x < ell / (k as f64) {
            return Err(k);
        }
    }
    Ok(())
}

fn realize_zero_drift_alphas(
    steps: usize,
    alpha_at: impl Fn(usize) -> f64,
) -> Vec<CavityTransportStep> {
    assert!(steps >= 1);

    let mut out = Vec::with_capacity(steps);
    let mut cavity = 2.0;
    let mut reference = 1.0;

    for step_index in 1..=steps {
        let alpha = alpha_at(step_index);
        assert!(alpha > 0.0 && alpha <= 1.0);
        let requested_edge_squared = alpha * cavity * reference;
        let edge = requested_edge_squared.sqrt();
        let realized_edge_squared = edge * edge;
        let current_reference = 1.0;
        let shifted_diagonal = realized_edge_squared / reference + current_reference;

        let step =
            CavityTransportStep::left(shifted_diagonal, edge, cavity, reference, current_reference)
                .expect("TDI-10.18 family construction must be admissible");

        assert!(step.transport_factor() > 0.0);
        assert!(step.transport_factor() <= 1.0 + 1.0e-14);
        assert_close(step.transport_factor(), alpha, 4.0e-15);
        assert_close(step.drift(), 0.0, 4.0e-15);

        // Item T — TDI-10.2 one-step identity on the public accessors.
        assert_close(
            step.reconstructed_error(),
            step.transport_factor() * step.neighbor_error() + step.drift(),
            4.0e-15,
        );
        assert_close(step.reconstructed_error(), step.current_error(), 4.0e-15);

        cavity = step.current_cavity();
        reference = step.current_reference();
        out.push(step);
    }

    out
}

fn realize_family(family: FamilyId, steps: usize, rho: f64) -> Vec<CavityTransportStep> {
    realize_zero_drift_alphas(steps, |k| family_alpha(family, k, rho))
}

#[test]
fn item_d_holds_for_family_fd_and_product_vanishes() {
    let ell = 0.5;
    let k0 = 1_usize;
    let steps = 512_usize;
    let mut alphas = Vec::with_capacity(steps);
    for k in 1..=steps {
        alphas.push((k, family_alpha(FamilyId::FD, k, 0.0)));
    }
    assert!(
        item_d_holds(&alphas, ell, k0).is_ok(),
        "F_D must meet checklist Item D with L=1/2"
    );

    let chain_steps = realize_family(FamilyId::FD, steps, 0.0);
    let chain = CavityTransportChain::from_steps(&chain_steps).expect("admissible chain");
    assert_close(
        chain.cumulative_transport_factor(),
        closed_product(FamilyId::FD, steps, 0.0),
        4.0e-15,
    );
    assert!(chain.cumulative_transport_factor() < 1.0 / (steps as f64));
}

#[test]
fn item_d_fails_for_family_fs_despite_alpha_to_one() {
    // For F_S, k * x_k = k/(k+1)^2 → 0, so every L > 0 eventually fails Item D.
    // Search far enough that each tested L has an explicit failing index.
    let ells = [1.0_f64, 0.5, 0.1, 1.0e-3, 1.0e-6];
    let search_limit = 2_000_000_usize; // covers L=1e-6 (need k ≳ 1/L)

    for &ell in &ells {
        let mut failed_at = None;
        for k in 1..=search_limit {
            let alpha = family_alpha(FamilyId::FS, k, 0.0);
            let x = 1.0 - alpha;
            if x < ell / (k as f64) {
                failed_at = Some(k);
                break;
            }
        }
        assert!(
            failed_at.is_some(),
            "F_S must fail Item D for L={ell} within {search_limit} (alpha→1 alone is not Item D)"
        );
    }

    // Informal "slow variation" slogan: alpha_k → 1, yet product stays away from 0.
    let product_steps = 1024_usize;
    assert!(family_alpha(FamilyId::FS, product_steps, 0.0) > 0.999);
    let chain_steps = realize_family(FamilyId::FS, product_steps, 0.0);
    let chain = CavityTransportChain::from_steps(&chain_steps).expect("admissible chain");
    assert_close(
        chain.cumulative_transport_factor(),
        closed_product(FamilyId::FS, product_steps, 0.0),
        4.0e-15,
    );
    assert!(chain.cumulative_transport_factor() > 0.49);
}

#[test]
fn item_d_and_item_u_hold_for_family_fu_including_toeplitz_rho() {
    let rho = 0.7_f64;
    let steps = 256_usize;
    let mut alphas = Vec::with_capacity(steps);
    for k in 1..=steps {
        alphas.push((k, family_alpha(FamilyId::FU, k, rho)));
    }
    // Item U: uniform bound.
    assert!(alphas.iter().all(|&(_, a)| a <= rho + 1.0e-15));
    // Item D eventually: 1-rho >= L/k for large k.
    let ell = (1.0 - rho) * 0.5;
    let k0 = ((ell / (1.0 - rho)).ceil() as usize).max(1);
    assert!(item_d_holds(&alphas, ell, k0).is_ok());

    let chain_steps = realize_family(FamilyId::FU, steps, rho);
    let chain = CavityTransportChain::from_steps(&chain_steps).expect("admissible chain");
    assert_close(
        chain.cumulative_transport_factor(),
        closed_product(FamilyId::FU, steps, rho),
        4.0e-15,
    );

    // Toeplitz-sourced rho (TDI-10.14 vocabulary reused, not reinvented).
    let frozen = FrozenToeplitzCavity::new(3.0, 1.0).expect("admissible frozen symbol");
    let toeplitz_rho = frozen.contraction();
    assert!(toeplitz_rho > 0.0 && toeplitz_rho < 1.0);
    let toeplitz_steps = realize_family(FamilyId::FU, 64, toeplitz_rho);
    let toeplitz_chain =
        CavityTransportChain::from_steps(&toeplitz_steps).expect("admissible chain");
    assert_close(
        toeplitz_chain.cumulative_transport_factor(),
        toeplitz_rho.powi(64),
        4.0e-15,
    );
}

#[test]
fn item_t_tdi10_2_identity_on_all_named_family_steps() {
    for family in [FamilyId::FS, FamilyId::FU, FamilyId::FD] {
        let rho = if family == FamilyId::FU { 0.55 } else { 0.0 };
        let steps = realize_family(family, 128, rho);
        for (idx, step) in steps.iter().enumerate() {
            let k = idx + 1;
            assert_close(
                step.transport_factor(),
                family_alpha(family, k, rho),
                4.0e-15,
            );
            assert_close(step.drift(), 0.0, 4.0e-15);
            assert_close(
                step.reconstructed_error(),
                step.transport_factor() * step.neighbor_error() + step.drift(),
                4.0e-15,
            );
            assert_close(step.reconstructed_error(), step.current_error(), 4.0e-15);
        }
    }
}

#[test]
fn finite_window_harmonic_bound_does_not_force_product_decay() {
    // REFUTED candidate: harmonic lower bound only on [1, N] ⇒ product → 0.
    let n_prefix = 8_usize;
    let n_total = 512_usize;
    let ell = 0.5_f64;

    let alpha_at = |k: usize| -> f64 {
        if k <= n_prefix {
            family_alpha(FamilyId::FD, k, 0.0)
        } else {
            family_alpha(FamilyId::FS, k, 0.0)
        }
    };

    // Finite window [1, N] meets a harmonic lower bound.
    let mut prefix_alphas = Vec::with_capacity(n_prefix);
    for k in 1..=n_prefix {
        prefix_alphas.push((k, alpha_at(k)));
    }
    assert!(
        item_d_holds(&prefix_alphas, ell, 1).is_ok(),
        "prefix must meet a harmonic bound on the finite window"
    );

    // Full sequence fails Item D eventually forever (Type-S tail).
    let mut all_alphas = Vec::with_capacity(n_total);
    for k in 1..=n_total {
        all_alphas.push((k, alpha_at(k)));
    }
    assert!(
        item_d_holds(&all_alphas, ell, 1).is_err(),
        "Type-S tail must break Item D on the infinite checklist reading"
    );

    let steps = realize_zero_drift_alphas(n_total, alpha_at);
    let chain = CavityTransportChain::from_steps(&steps).expect("admissible chain");

    // Closed form: prefix 1/(N+1) times F_S-style telescoping from N+1.
    // ∏_{k=N+1}^n (1 - 1/(k+1)^2) = (N+1)(n+2)/((N+2)(n+1))
    // full product → 1/(N+2).
    let n = n_total as f64;
    let n_p = n_prefix as f64;
    // prefix 1/(N+1) * tail (N+1)(n+2)/((N+2)(n+1)) = (n+2)/((N+2)(n+1)) → 1/(N+2)
    let expected = (n + 2.0) / ((n_p + 2.0) * (n + 1.0));
    assert_close(chain.cumulative_transport_factor(), expected, 4.0e-15);
    let limit = 1.0 / (n_p + 2.0);
    assert!(
        chain.cumulative_transport_factor() > 0.5 * limit,
        "finite-window harmonic bound must not force product → 0; got {}",
        chain.cumulative_transport_factor()
    );
    assert!((expected - limit).abs() < 1.0 / (n + 1.0));
}
