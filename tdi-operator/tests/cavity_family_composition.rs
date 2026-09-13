//! TDI-10.15 — EXACT operator-family composition: F_U then F_D / F_S.
//!
//! Finite concatenation product identities for named cavity families;
//! Toeplitz-sourced ρ reuse from TDI-10.14; REFUTED that a Type-S suffix
//! erases Type-U prefix decay uniformly in the prefix length. Does not prove
//! slowly-varying Jacobi asymptotics.

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
enum BlockFamily {
    /// Constant alpha = rho (Type U).
    FU,
    /// alpha_k = k/(k+1) with block-local index k (Type D).
    FD,
    /// alpha_k = 1 - 1/(k+1)^2 with block-local index k (Type S).
    FS,
}

fn block_alpha(family: BlockFamily, local_index: usize, rho: f64) -> f64 {
    assert!(local_index >= 1);
    match family {
        BlockFamily::FU => {
            assert!(rho > 0.0 && rho < 1.0);
            rho
        }
        BlockFamily::FD => (local_index as f64) / ((local_index + 1) as f64),
        BlockFamily::FS => {
            let denominator = (local_index + 1) as f64;
            1.0 - 1.0 / (denominator * denominator)
        }
    }
}

fn closed_u_then_d(m: usize, n: usize, rho: f64) -> f64 {
    rho.powi(m as i32) / ((n + 1) as f64)
}

fn closed_u_then_s(m: usize, n: usize, rho: f64) -> f64 {
    rho.powi(m as i32) * ((n + 2) as f64) / (2.0 * ((n + 1) as f64))
}

/// Append `steps` zero-drift cavity realizations of `family`, continuing from
/// the trailing cavity/reference state (contiguous chain provenance).
fn append_block(
    out: &mut Vec<CavityTransportStep>,
    family: BlockFamily,
    steps: usize,
    rho: f64,
    cavity: &mut f64,
    reference: &mut f64,
) {
    assert!(steps >= 1);
    for local_index in 1..=steps {
        let alpha = block_alpha(family, local_index, rho);
        assert!(alpha > 0.0 && alpha <= 1.0);
        let requested_edge_squared = alpha * (*cavity) * (*reference);
        let edge = requested_edge_squared.sqrt();
        let realized_edge_squared = edge * edge;
        let current_reference = 1.0;
        let shifted_diagonal = realized_edge_squared / (*reference) + current_reference;

        let step = CavityTransportStep::left(
            shifted_diagonal,
            edge,
            *cavity,
            *reference,
            current_reference,
        )
        .expect("TDI-10.15 composed family construction must be admissible");

        assert_close(step.transport_factor(), alpha, 4.0e-15);
        assert_close(step.drift(), 0.0, 4.0e-15);

        *cavity = step.current_cavity();
        *reference = step.current_reference();
        out.push(step);
    }
}

fn realize_u_then(suffix: BlockFamily, m: usize, n: usize, rho: f64) -> Vec<CavityTransportStep> {
    let mut out = Vec::with_capacity(m + n);
    let mut cavity = 2.0;
    let mut reference = 1.0;
    append_block(
        &mut out,
        BlockFamily::FU,
        m,
        rho,
        &mut cavity,
        &mut reference,
    );
    append_block(&mut out, suffix, n, rho, &mut cavity, &mut reference);
    out
}

#[test]
fn algebraic_u_then_d_matches_closed_product() {
    for &rho in &[0.5_f64, 0.8, 0.9] {
        for m in [1_usize, 4, 16] {
            for n in [1_usize, 8, 64] {
                let mut product = 1.0;
                for local in 1..=m {
                    product *= block_alpha(BlockFamily::FU, local, rho);
                }
                for local in 1..=n {
                    product *= block_alpha(BlockFamily::FD, local, rho);
                }
                assert_close(product, closed_u_then_d(m, n, rho), 2.0e-14);
            }
        }
    }
}

#[test]
fn algebraic_u_then_s_matches_closed_product() {
    for &rho in &[0.5_f64, 0.8, 0.9] {
        for m in [1_usize, 4, 16] {
            for n in [1_usize, 8, 64] {
                let mut product = 1.0;
                for local in 1..=m {
                    product *= block_alpha(BlockFamily::FU, local, rho);
                }
                for local in 1..=n {
                    product *= block_alpha(BlockFamily::FS, local, rho);
                }
                assert_close(product, closed_u_then_s(m, n, rho), 2.0e-14);
            }
        }
    }
}

#[test]
fn cavity_chain_realizes_u_then_d_remainder() {
    let rho = 0.8_f64;
    for (m, n) in [(2_usize, 4), (8, 16), (16, 32)] {
        let realized = realize_u_then(BlockFamily::FD, m, n, rho);
        assert_eq!(realized.len(), m + n);
        let chain = CavityTransportChain::from_steps(&realized)
            .expect("F_U then F_D must form one contiguous cavity chain");
        assert_eq!(chain.steps(), m + n);
        assert_close(
            chain.cumulative_transport_factor(),
            closed_u_then_d(m, n, rho),
            2.0e-12,
        );
        assert_close(chain.accumulated_drift(), 0.0, 2.0e-12);
        // Type-D remainder forces decay in n for fixed m.
        if n >= 32 {
            assert!(chain.cumulative_transport_factor() < rho.powi(m as i32) * 0.05);
        }
    }
}

#[test]
fn cavity_chain_realizes_u_then_s_remainder() {
    let rho = 0.75_f64;
    for (m, n) in [(2_usize, 4), (8, 64), (12, 256)] {
        let realized = realize_u_then(BlockFamily::FS, m, n, rho);
        let chain = CavityTransportChain::from_steps(&realized)
            .expect("F_U then F_S must form one contiguous cavity chain");
        assert_eq!(chain.steps(), m + n);
        assert_close(
            chain.cumulative_transport_factor(),
            closed_u_then_s(m, n, rho),
            2.0e-12,
        );
        assert_close(chain.accumulated_drift(), 0.0, 2.0e-12);
        // Type-S remainder saturates near rho^m / 2 for large n.
        if n >= 256 {
            assert_close(
                chain.cumulative_transport_factor(),
                rho.powi(m as i32) / 2.0,
                1.0e-2,
            );
            assert!(chain.cumulative_transport_factor() > 0.4 * rho.powi(m as i32));
        }
    }
}

#[test]
fn toeplitz_sourced_rho_composes_u_then_d() {
    let frozen = FrozenToeplitzCavity::new(3.0, 1.0)
        .expect("TDI-10.1 frozen Toeplitz example (3,1) must be admissible");
    let rho = frozen.contraction();
    assert!(rho > 0.0 && rho < 1.0);

    let m = 10_usize;
    let n = 40_usize;
    let realized = realize_u_then(BlockFamily::FD, m, n, rho);
    let chain = CavityTransportChain::from_steps(&realized)
        .expect("Toeplitz-sourced F_U then F_D must form one contiguous chain");

    assert_close(
        chain.cumulative_transport_factor(),
        closed_u_then_d(m, n, rho),
        2.0e-12,
    );
    assert!(chain.cumulative_transport_factor() < 1.0e-4);
    assert_close(chain.accumulated_drift(), 0.0, 2.0e-12);
}

#[test]
fn type_s_suffix_does_not_erase_type_u_prefix_decay_uniformly_in_m() {
    // REFUTED: appending Type S after any Type-U prefix erases Type-U decay
    // so that lim_n P_{U→S}(m,n;ρ) is bounded away from 0 independently of m.
    let rho = 0.85_f64;
    let epsilon = 1.0e-4_f64;

    // Choose m with rho^m < epsilon.
    let mut m = 1_usize;
    while rho.powi(m as i32) >= epsilon {
        m += 1;
        assert!(m < 10_000);
    }
    assert!(rho.powi(m as i32) < epsilon);

    let n = 512_usize;
    let limit_proxy = closed_u_then_s(m, n, rho);
    assert!(limit_proxy < epsilon);
    assert_close(limit_proxy, rho.powi(m as i32) / 2.0, 5.0e-3);

    let realized = realize_u_then(BlockFamily::FS, m, n, rho);
    let chain = CavityTransportChain::from_steps(&realized).unwrap();
    assert!(chain.cumulative_transport_factor() < epsilon);
    assert_close(
        chain.cumulative_transport_factor(),
        closed_u_then_s(m, n, rho),
        2.0e-12,
    );

    // Contrast: short prefix leaves a larger composed liminf.
    let m_short = 2_usize;
    let short_limit = rho.powi(m_short as i32) / 2.0;
    assert!(short_limit > 10.0 * epsilon);
    assert!(rho.powi(m as i32) / 2.0 < short_limit);
}

#[test]
fn u_then_d_decays_when_either_block_lengthens() {
    let rho = 0.9_f64;
    let base = closed_u_then_d(4, 4, rho);
    assert!(closed_u_then_d(32, 4, rho) < base);
    assert!(closed_u_then_d(4, 256, rho) < base);
    assert!(closed_u_then_d(64, 256, rho) < 1.0e-4);
    assert!(closed_u_then_d(32, 256, rho) < base);
}
