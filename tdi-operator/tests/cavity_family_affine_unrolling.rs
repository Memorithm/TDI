//! TDI-10.17 — EXACT family ↔ TDI-10.4 affine-unrolling link.
//!
//! Zero-drift collapse onto named-family closed products; constant-drift
//! F_U / F_D accumulated-drift closed forms; REFUTED that Type-U product
//! decay alone forces cavity-error → 0 under nonzero constant drift.
//! Does not prove slowly-varying Jacobi asymptotics.

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

fn closed_fu_accumulated_drift(n: usize, rho: f64, delta: f64) -> f64 {
    delta * (1.0 - rho.powi(n as i32)) / (1.0 - rho)
}

fn closed_fd_accumulated_drift(n: usize, delta: f64) -> f64 {
    // B_n = δ/(n+1) * ((n+1)(n+2)/2 - 1)
    let n1 = (n + 1) as f64;
    delta / n1 * (n1 * ((n + 2) as f64) / 2.0 - 1.0)
}

/// Build a contiguous cavity chain with prescribed family alphas and a
/// constant per-step drift `delta` (zero allowed). Uses only public
/// `CavityTransportStep::left` — no new transport API.
fn realize_family_chain_with_drift(
    family: FamilyId,
    steps: usize,
    rho: f64,
    delta: f64,
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
        // delta = a - e^2/q_j - q_i  ⇒  a = delta + e^2/q_j + q_i
        let shifted_diagonal = delta + realized_edge_squared / reference + current_reference;

        let step =
            CavityTransportStep::left(shifted_diagonal, edge, cavity, reference, current_reference)
                .expect("TDI-10.17 family+drift construction must be admissible");

        assert!(step.transport_factor() > 0.0);
        assert!(step.transport_factor() <= 1.0 + 1.0e-14);
        assert_close(step.transport_factor(), alpha, 4.0e-15);
        assert_close(step.drift(), delta, 4.0e-15);
        assert!(step.current_cavity() > 0.0);

        cavity = step.current_cavity();
        reference = step.current_reference();
        out.push(step);
    }

    out
}

#[test]
fn zero_drift_family_chains_collapse_unrolling_onto_closed_products() {
    let cases = [
        (FamilyId::FS, 1.0_f64, &[1_usize, 8, 64][..]),
        (FamilyId::FU, 0.7_f64, &[1, 4, 16][..]),
        (FamilyId::FD, 1.0_f64, &[1, 8, 64][..]),
    ];

    for &(family, rho, lengths) in &cases {
        for &n in lengths {
            let steps = realize_family_chain_with_drift(family, n, rho, 0.0);
            let chain = CavityTransportChain::from_steps(&steps).unwrap();
            let a_n = closed_product(family, n, rho);

            assert_close(chain.cumulative_transport_factor(), a_n, 6.0e-15);
            assert_close(chain.accumulated_drift(), 0.0, 6.0e-15);
            assert_close(
                chain.reconstructed_final_error(),
                a_n * chain.initial_error(),
                8.0e-15,
            );
            assert_close(
                chain.reconstructed_final_error(),
                chain.observed_final_error(),
                8.0e-15,
            );
        }
    }
}

#[test]
fn constant_drift_fu_matches_geometric_unrolling_closed_form() {
    let deltas = [0.05_f64, -0.12, 0.3];
    let rhos = [0.4_f64, 0.75];
    for &rho in &rhos {
        for &delta in &deltas {
            for n in [1_usize, 3, 12, 40] {
                let steps = realize_family_chain_with_drift(FamilyId::FU, n, rho, delta);
                let chain = CavityTransportChain::from_steps(&steps).unwrap();
                let a_n = rho.powi(n as i32);
                let b_n = closed_fu_accumulated_drift(n, rho, delta);
                let e_n = a_n * chain.initial_error() + b_n;

                assert_close(chain.cumulative_transport_factor(), a_n, 6.0e-15);
                assert_close(chain.accumulated_drift(), b_n, 8.0e-14);
                assert_close(chain.reconstructed_final_error(), e_n, 8.0e-14);
                assert_close(
                    chain.reconstructed_final_error(),
                    chain.observed_final_error(),
                    8.0e-14,
                );
            }
        }
    }
}

#[test]
fn constant_drift_fu_accepts_toeplitz_sourced_rho() {
    let frozen = FrozenToeplitzCavity::new(5.0, 1.5).expect("admissible symbol");
    let rho = frozen.contraction();
    assert!(rho > 0.0 && rho < 1.0);

    let delta = 0.08_f64;
    let n = 24_usize;
    let steps = realize_family_chain_with_drift(FamilyId::FU, n, rho, delta);
    let chain = CavityTransportChain::from_steps(&steps).unwrap();
    let a_n = rho.powi(n as i32);
    let b_n = closed_fu_accumulated_drift(n, rho, delta);

    assert_close(chain.cumulative_transport_factor(), a_n, 6.0e-15);
    assert_close(chain.accumulated_drift(), b_n, 1.0e-13);
    assert_close(
        chain.reconstructed_final_error(),
        a_n * chain.initial_error() + b_n,
        1.0e-13,
    );
}

#[test]
fn constant_drift_fd_matches_closed_accumulated_drift() {
    let delta = 0.2_f64;
    for n in [1_usize, 2, 7, 31] {
        let steps = realize_family_chain_with_drift(FamilyId::FD, n, 1.0, delta);
        let chain = CavityTransportChain::from_steps(&steps).unwrap();
        let a_n = 1.0 / ((n + 1) as f64);
        let b_n = closed_fd_accumulated_drift(n, delta);
        let e_n = a_n * chain.initial_error() + b_n;

        assert_close(chain.cumulative_transport_factor(), a_n, 6.0e-15);
        assert_close(chain.accumulated_drift(), b_n, 1.0e-13);
        assert_close(chain.reconstructed_final_error(), e_n, 1.0e-13);
        assert_close(
            chain.reconstructed_final_error(),
            chain.observed_final_error(),
            1.0e-13,
        );
    }
}

#[test]
fn refuted_type_u_product_decay_alone_does_not_force_error_to_zero_under_drift() {
    // Candidate claim: Type-U alpha_k = rho < 1 ⇒ E_n → 0.
    // Counterexample: constant nonzero drift; E_n → delta/(1-rho) ≠ 0.
    let rho = 0.6_f64;
    let delta = 0.25_f64;
    let n = 80_usize;
    let steps = realize_family_chain_with_drift(FamilyId::FU, n, rho, delta);
    let chain = CavityTransportChain::from_steps(&steps).unwrap();

    let product = chain.cumulative_transport_factor();
    let limit = delta / (1.0 - rho);
    let e_n = chain.reconstructed_final_error();

    assert!(
        product < 1.0e-12,
        "Type-U product must be tiny; got {product}"
    );
    assert!(product < 0.01 * limit.abs());
    assert_close(e_n, limit, 1.0e-10);
    assert!(
        e_n.abs() > 0.1,
        "error remains bounded away from 0 under constant drift; got {e_n}"
    );
}
