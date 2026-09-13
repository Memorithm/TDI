//! TDI-10.13 — named operator families F_S / F_U / F_D realizing the
//! subunit-product trichotomy through public cavity-transport APIs.
//!
//! Scientific status: EXACT bookkeeping of already-qualified witnesses
//! (TDI-10.5–10.8) as operator families; REFUTED that every cavity family
//! forces product decay. Does not prove slowly-varying Jacobi asymptotics.

use tdi_operator::{CavityTransportChain, CavityTransportStep, FrozenToeplitzCavity};

fn assert_close(left: f64, right: f64, tolerance: f64) {
    let scale = 1.0_f64.max(left.abs()).max(right.abs());
    assert!(
        (left - right).abs() <= tolerance * scale,
        "left={left:.17e}, right={right:.17e}, |delta|={:.3e}",
        (left - right).abs()
    );
}

/// Named operator-family identifiers for the TDI-10.13 registry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OperatorFamilyId {
    /// Type S: alpha_k = 1 - 1/(k+1)^2 (TDI-10.5).
    FS,
    /// Type U: constant alpha_k = rho (TDI-10.6 / frozen contraction).
    FU,
    /// Type D: alpha_k = k/(k+1) (TDI-10.7).
    FD,
}

/// Declared map producing alpha_k in (0,1] for cavity transport.
///
/// An operator family (this stage) is exactly such a map, realized by
/// zero-drift `CavityTransportStep` chains. No new physical parameters.
fn family_alpha(family: OperatorFamilyId, step_index: usize, rho: f64) -> f64 {
    assert!(step_index >= 1);
    match family {
        OperatorFamilyId::FS => {
            let denominator = (step_index + 1) as f64;
            1.0 - 1.0 / (denominator * denominator)
        }
        OperatorFamilyId::FU => {
            assert!(rho > 0.0 && rho < 1.0);
            rho
        }
        OperatorFamilyId::FD => (step_index as f64) / ((step_index + 1) as f64),
    }
}

fn family_closed_product(family: OperatorFamilyId, steps: usize, rho: f64) -> f64 {
    match family {
        OperatorFamilyId::FS => ((steps + 2) as f64) / (2.0 * ((steps + 1) as f64)),
        OperatorFamilyId::FU => rho.powi(steps as i32),
        OperatorFamilyId::FD => 1.0 / ((steps + 1) as f64),
    }
}

/// Zero-drift cavity realization of a declared alpha map (TDI-10.2 / 10.4).
fn realize_family_chain(
    family: OperatorFamilyId,
    steps: usize,
    rho: f64,
) -> Vec<CavityTransportStep> {
    assert!(steps >= 1);

    let mut out = Vec::with_capacity(steps);
    let mut cavity = 2.0;
    let mut reference = 1.0;

    for step_index in 1..=steps {
        let alpha = family_alpha(family, step_index, rho);
        assert!(alpha > 0.0 && alpha <= 1.0);
        let requested_edge_squared = alpha * cavity * reference;
        let edge = requested_edge_squared.sqrt();
        let realized_edge_squared = edge * edge;
        let current_reference = 1.0;
        let shifted_diagonal = realized_edge_squared / reference + current_reference;

        let step =
            CavityTransportStep::left(shifted_diagonal, edge, cavity, reference, current_reference)
                .expect("TDI-10.13 family construction must be admissible");

        assert!(step.transport_factor() > 0.0);
        assert!(step.transport_factor() <= 1.0 + 1.0e-14);
        assert_close(step.transport_factor(), alpha, 4.0e-15);
        assert_close(step.drift(), 0.0, 4.0e-15);

        cavity = step.current_cavity();
        reference = step.current_reference();
        out.push(step);
    }

    out
}

#[test]
fn family_fs_is_type_s_summable_nondecay() {
    let family = OperatorFamilyId::FS;
    for steps in [1_usize, 16, 256, 1024] {
        let mut product = 1.0;
        let mut remainder = 0.0;
        for step_index in 1..=steps {
            let alpha = family_alpha(family, step_index, 0.0);
            assert!(alpha > 0.0 && alpha < 1.0);
            product *= alpha;
            remainder += 1.0 - alpha;
        }
        assert_close(product, family_closed_product(family, steps, 0.0), 2.0e-14);
        assert!(product > 0.5);
        assert!(remainder < 1.0);
    }
}

#[test]
fn family_fu_is_type_u_uniform_geometric_decay() {
    let family = OperatorFamilyId::FU;
    for &rho in &[0.5_f64, 0.8, 0.9] {
        for steps in [1_usize, 8, 32, 128] {
            let mut product = 1.0;
            for step_index in 1..=steps {
                product *= family_alpha(family, step_index, rho);
            }
            assert_close(product, family_closed_product(family, steps, rho), 2.0e-14);
            assert!(product <= rho);
            if steps >= 32 && rho <= 0.9 {
                assert!(product < 1.0e-1);
            }
        }
    }
}

#[test]
fn family_fd_is_type_d_divergent_remainder_decay() {
    let family = OperatorFamilyId::FD;
    for steps in [1_usize, 16, 256, 1024] {
        let mut product = 1.0;
        let mut remainder = 0.0;
        for step_index in 1..=steps {
            let alpha = family_alpha(family, step_index, 0.0);
            assert!(alpha > 0.0 && alpha < 1.0);
            product *= alpha;
            remainder += 1.0 - alpha;
        }
        assert_close(product, family_closed_product(family, steps, 0.0), 2.0e-14);
        if steps >= 256 {
            assert!(product < 1.0e-2);
            assert!(family_alpha(family, steps, 0.0) > 0.99);
        }
        let lower_bound = ((steps + 2) as f64 / 2.0).ln();
        assert!(remainder + 1.0e-12 >= lower_bound);
    }
}

#[test]
fn cavity_chains_realize_named_families_fs_fu_fd() {
    // F_S
    {
        let steps = 64_usize;
        let realized = realize_family_chain(OperatorFamilyId::FS, steps, 0.0);
        let chain = CavityTransportChain::from_steps(&realized)
            .expect("F_S must form one contiguous cavity chain");
        assert_eq!(chain.steps(), steps);
        assert_close(
            chain.cumulative_transport_factor(),
            family_closed_product(OperatorFamilyId::FS, steps, 0.0),
            2.0e-12,
        );
        assert!(chain.cumulative_transport_factor() > 0.5);
        assert_close(chain.accumulated_drift(), 0.0, 2.0e-12);
    }

    // F_U (declared constant rho)
    {
        let rho = 0.8_f64;
        let steps = 32_usize;
        let realized = realize_family_chain(OperatorFamilyId::FU, steps, rho);
        let chain = CavityTransportChain::from_steps(&realized)
            .expect("F_U must form one contiguous cavity chain");
        assert_eq!(chain.steps(), steps);
        assert_close(
            chain.cumulative_transport_factor(),
            family_closed_product(OperatorFamilyId::FU, steps, rho),
            2.0e-12,
        );
        assert!(chain.cumulative_transport_factor() < 1.0e-2);
        assert_close(chain.accumulated_drift(), 0.0, 2.0e-12);
    }

    // F_D
    {
        let steps = 128_usize;
        let realized = realize_family_chain(OperatorFamilyId::FD, steps, 0.0);
        let chain = CavityTransportChain::from_steps(&realized)
            .expect("F_D must form one contiguous cavity chain");
        assert_eq!(chain.steps(), steps);
        assert_close(
            chain.cumulative_transport_factor(),
            family_closed_product(OperatorFamilyId::FD, steps, 0.0),
            2.0e-12,
        );
        assert!(chain.cumulative_transport_factor() < 1.0e-2);
        assert_close(chain.accumulated_drift(), 0.0, 2.0e-12);
    }
}

#[test]
fn family_fu_sources_rho_from_frozen_toeplitz_contraction() {
    // Declared mathematical example already in the TDI-10.1 vocabulary:
    // constant symbol (a,b)=(3,1) with a > 2|b|.
    let frozen = FrozenToeplitzCavity::new(3.0, 1.0)
        .expect("TDI-10.1 frozen Toeplitz example (3,1) must be admissible");
    let rho = frozen.contraction();
    assert!(rho > 0.0 && rho < 1.0);
    // Exact identity: kappa = b^2 / q^2.
    assert_close(rho, (frozen.edge() / frozen.cavity()).powi(2), 2.0e-15);

    let steps = 40_usize;
    let realized = realize_family_chain(OperatorFamilyId::FU, steps, rho);
    let chain = CavityTransportChain::from_steps(&realized)
        .expect("frozen-sourced F_U must form one contiguous cavity chain");

    assert_eq!(chain.steps(), steps);
    for step in &realized {
        assert_close(step.transport_factor(), rho, 4.0e-15);
        assert_close(step.drift(), 0.0, 4.0e-15);
    }
    assert_close(
        chain.cumulative_transport_factor(),
        rho.powi(steps as i32),
        2.0e-12,
    );
    assert!(chain.cumulative_transport_factor() < 1.0e-3);
    assert_close(chain.accumulated_drift(), 0.0, 2.0e-12);
}

#[test]
fn family_fs_refutes_universal_cavity_decay() {
    // REFUTED: every admissible cavity family with 0 < alpha_k < 1 forces
    // product -> 0. F_S is the 10.5 witness packaged as a cavity family.
    let steps = 1024_usize;
    let realized = realize_family_chain(OperatorFamilyId::FS, steps, 0.0);
    let chain =
        CavityTransportChain::from_steps(&realized).expect("F_S cavity realization must succeed");
    assert!(chain.cumulative_transport_factor() > 0.5);
    assert_close(
        chain.cumulative_transport_factor(),
        family_closed_product(OperatorFamilyId::FS, steps, 0.0),
        2.0e-12,
    );
    // Observed boundary error stays away from zero under zero drift.
    assert!(chain.observed_final_error() > 0.49);
}

#[test]
fn named_families_are_mutually_exclusive_on_product_limits() {
    let type_s = family_closed_product(OperatorFamilyId::FS, 1024, 0.0);
    let type_u = family_closed_product(OperatorFamilyId::FU, 1024, 0.9);
    let type_d = family_closed_product(OperatorFamilyId::FD, 1024, 0.0);

    assert!(type_s > 0.5);
    assert!(type_u < 1.0e-40 || type_u == 0.0 || type_u < 1.0e-20);
    assert!(type_d < 1.0e-3);
    assert!(type_s > 100.0 * type_d.max(type_u.max(1.0e-300)));
}
