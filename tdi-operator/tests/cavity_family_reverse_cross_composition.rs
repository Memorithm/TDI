//! TDI-10.19 — EXACT reverse and cross operator-family composition.
//!
//! Finite concatenation product identities for F_D/F_S then F_U and for
//! F_D↔F_S cross products; Toeplitz-sourced ρ reuse from TDI-10.14; REFUTED
//! that a Type-S prefix blocks Type-D suffix decay; REFUTED that a finite
//! Type-D prefix forces composed liminf 0 under Type-S; REFUTED that F_D/F_S
//! block order is immaterial. Does not prove slowly-varying Jacobi
//! asymptotics.

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

fn closed_d_then_u(m: usize, n: usize, rho: f64) -> f64 {
    rho.powi(n as i32) / ((m + 1) as f64)
}

fn closed_s_then_u(m: usize, n: usize, rho: f64) -> f64 {
    ((m + 2) as f64) / (2.0 * ((m + 1) as f64)) * rho.powi(n as i32)
}

fn closed_d_then_s(m: usize, n: usize) -> f64 {
    (1.0 / ((m + 1) as f64)) * ((n + 2) as f64) / (2.0 * ((n + 1) as f64))
}

fn closed_s_then_d(m: usize, n: usize) -> f64 {
    ((m + 2) as f64) / (2.0 * ((m + 1) as f64)) / ((n + 1) as f64)
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
        .expect("TDI-10.19 composed family construction must be admissible");

        assert_close(step.transport_factor(), alpha, 4.0e-15);
        assert_close(step.drift(), 0.0, 4.0e-15);

        *cavity = step.current_cavity();
        *reference = step.current_reference();
        out.push(step);
    }
}

fn realize_two_blocks(
    prefix: BlockFamily,
    suffix: BlockFamily,
    m: usize,
    n: usize,
    rho: f64,
) -> Vec<CavityTransportStep> {
    let mut out = Vec::with_capacity(m + n);
    let mut cavity = 2.0;
    let mut reference = 1.0;
    append_block(&mut out, prefix, m, rho, &mut cavity, &mut reference);
    append_block(&mut out, suffix, n, rho, &mut cavity, &mut reference);
    out
}

fn algebraic_product(
    prefix: BlockFamily,
    suffix: BlockFamily,
    m: usize,
    n: usize,
    rho: f64,
) -> f64 {
    let mut product = 1.0;
    for local in 1..=m {
        product *= block_alpha(prefix, local, rho);
    }
    for local in 1..=n {
        product *= block_alpha(suffix, local, rho);
    }
    product
}

#[test]
fn algebraic_d_then_u_matches_closed_product() {
    for &rho in &[0.5_f64, 0.8, 0.9] {
        for m in [1_usize, 4, 16] {
            for n in [1_usize, 8, 64] {
                assert_close(
                    algebraic_product(BlockFamily::FD, BlockFamily::FU, m, n, rho),
                    closed_d_then_u(m, n, rho),
                    2.0e-14,
                );
            }
        }
    }
}

#[test]
fn algebraic_s_then_u_matches_closed_product() {
    for &rho in &[0.5_f64, 0.8, 0.9] {
        for m in [1_usize, 4, 16] {
            for n in [1_usize, 8, 64] {
                assert_close(
                    algebraic_product(BlockFamily::FS, BlockFamily::FU, m, n, rho),
                    closed_s_then_u(m, n, rho),
                    2.0e-14,
                );
            }
        }
    }
}

#[test]
fn algebraic_d_then_s_matches_closed_product() {
    for m in [1_usize, 2, 4, 16] {
        for n in [1_usize, 2, 8, 64] {
            assert_close(
                algebraic_product(BlockFamily::FD, BlockFamily::FS, m, n, 0.5),
                closed_d_then_s(m, n),
                2.0e-14,
            );
        }
    }
}

#[test]
fn algebraic_s_then_d_matches_closed_product() {
    for m in [1_usize, 2, 4, 16] {
        for n in [1_usize, 2, 8, 64] {
            assert_close(
                algebraic_product(BlockFamily::FS, BlockFamily::FD, m, n, 0.5),
                closed_s_then_d(m, n),
                2.0e-14,
            );
        }
    }
}

#[test]
fn cavity_chain_realizes_d_then_u_suffix() {
    let rho = 0.8_f64;
    for (m, n) in [(2_usize, 4), (8, 16), (16, 32)] {
        let realized = realize_two_blocks(BlockFamily::FD, BlockFamily::FU, m, n, rho);
        assert_eq!(realized.len(), m + n);
        let chain = CavityTransportChain::from_steps(&realized)
            .expect("F_D then F_U must form one contiguous cavity chain");
        assert_eq!(chain.steps(), m + n);
        assert_close(
            chain.cumulative_transport_factor(),
            closed_d_then_u(m, n, rho),
            2.0e-12,
        );
        assert_close(chain.accumulated_drift(), 0.0, 2.0e-12);
        if n >= 32 {
            assert!(chain.cumulative_transport_factor() < 1.0 / ((m + 1) as f64) * 0.05);
        }
    }
}

#[test]
fn cavity_chain_realizes_s_then_u_suffix() {
    let rho = 0.75_f64;
    for (m, n) in [(2_usize, 4), (8, 16), (12, 32)] {
        let realized = realize_two_blocks(BlockFamily::FS, BlockFamily::FU, m, n, rho);
        let chain = CavityTransportChain::from_steps(&realized)
            .expect("F_S then F_U must form one contiguous cavity chain");
        assert_eq!(chain.steps(), m + n);
        assert_close(
            chain.cumulative_transport_factor(),
            closed_s_then_u(m, n, rho),
            2.0e-12,
        );
        assert_close(chain.accumulated_drift(), 0.0, 2.0e-12);
        if n >= 32 {
            assert!(chain.cumulative_transport_factor() < 0.05);
        }
    }
}

#[test]
fn cavity_chain_realizes_d_then_s_cross() {
    for (m, n) in [(2_usize, 4), (8, 64), (16, 256)] {
        let realized = realize_two_blocks(BlockFamily::FD, BlockFamily::FS, m, n, 0.5);
        let chain = CavityTransportChain::from_steps(&realized)
            .expect("F_D then F_S must form one contiguous cavity chain");
        assert_close(
            chain.cumulative_transport_factor(),
            closed_d_then_s(m, n),
            2.0e-12,
        );
        assert_close(chain.accumulated_drift(), 0.0, 2.0e-12);
        if n >= 256 {
            assert_close(
                chain.cumulative_transport_factor(),
                1.0 / (2.0 * ((m + 1) as f64)),
                1.0e-2,
            );
            assert!(chain.cumulative_transport_factor() > 0.4 / ((m + 1) as f64));
        }
    }
}

#[test]
fn cavity_chain_realizes_s_then_d_cross() {
    for (m, n) in [(2_usize, 4), (8, 16), (16, 64)] {
        let realized = realize_two_blocks(BlockFamily::FS, BlockFamily::FD, m, n, 0.5);
        let chain = CavityTransportChain::from_steps(&realized)
            .expect("F_S then F_D must form one contiguous cavity chain");
        assert_close(
            chain.cumulative_transport_factor(),
            closed_s_then_d(m, n),
            2.0e-12,
        );
        assert_close(chain.accumulated_drift(), 0.0, 2.0e-12);
        if n >= 64 {
            assert!(chain.cumulative_transport_factor() < 0.02);
        }
    }
}

#[test]
fn toeplitz_sourced_rho_composes_d_then_u() {
    let frozen = FrozenToeplitzCavity::new(3.0, 1.0)
        .expect("TDI-10.1 frozen Toeplitz example (3,1) must be admissible");
    let rho = frozen.contraction();
    assert!(rho > 0.0 && rho < 1.0);

    let m = 10_usize;
    let n = 40_usize;
    let realized = realize_two_blocks(BlockFamily::FD, BlockFamily::FU, m, n, rho);
    let chain = CavityTransportChain::from_steps(&realized)
        .expect("Toeplitz-sourced F_D then F_U must form one contiguous chain");

    assert_close(
        chain.cumulative_transport_factor(),
        closed_d_then_u(m, n, rho),
        2.0e-12,
    );
    assert!(chain.cumulative_transport_factor() < 1.0e-4);
    assert_close(chain.accumulated_drift(), 0.0, 2.0e-12);
}

#[test]
fn type_s_prefix_does_not_block_type_d_suffix_decay() {
    // REFUTED: a Type-S prefix prevents product → 0 when Type-D suffix lengthens.
    let m = 8_usize;
    let prefix_factor = ((m + 2) as f64) / (2.0 * ((m + 1) as f64));
    assert!(prefix_factor > 0.4);

    let n = 512_usize;
    let product = closed_s_then_d(m, n);
    assert!(product < 1.0e-2);
    assert_close(product, prefix_factor / ((n + 1) as f64), 2.0e-14);

    let realized = realize_two_blocks(BlockFamily::FS, BlockFamily::FD, m, n, 0.5);
    let chain = CavityTransportChain::from_steps(&realized).unwrap();
    assert!(chain.cumulative_transport_factor() < 1.0e-2);
    assert_close(
        chain.cumulative_transport_factor(),
        closed_s_then_d(m, n),
        2.0e-12,
    );
}

#[test]
fn finite_type_d_prefix_does_not_force_liminf_zero_under_type_s_suffix() {
    // REFUTED: every finite Type-D prefix forces lim_n P_{D→S}(m,n) = 0.
    let m = 4_usize;
    let liminf = 1.0 / (2.0 * ((m + 1) as f64));
    assert!(liminf > 0.05);

    let n = 1024_usize;
    let product = closed_d_then_s(m, n);
    assert_close(product, liminf, 5.0e-3);
    assert!(product > 0.05);

    let realized = realize_two_blocks(BlockFamily::FD, BlockFamily::FS, m, n, 0.5);
    let chain = CavityTransportChain::from_steps(&realized).unwrap();
    assert!(chain.cumulative_transport_factor() > 0.05);
    assert_close(
        chain.cumulative_transport_factor(),
        closed_d_then_s(m, n),
        2.0e-12,
    );

    // Contrast: only sending m → ∞ drives the liminf to 0.
    let large_m = 256_usize;
    assert!(1.0 / (2.0 * ((large_m + 1) as f64)) < 0.01);
}

#[test]
fn d_s_block_order_is_not_immaterial() {
    // REFUTED: P_{D→S}(m,n) = P_{S→D}(m,n) for all m,n.
    let m = 2_usize;
    let n = 1_usize;
    let d_then_s = closed_d_then_s(m, n);
    let s_then_d = closed_s_then_d(m, n);
    assert_close(d_then_s, 0.25, 1.0e-15);
    assert_close(s_then_d, 1.0 / 3.0, 1.0e-15);
    assert!((d_then_s - s_then_d).abs() > 0.05);

    let chain_ds = CavityTransportChain::from_steps(&realize_two_blocks(
        BlockFamily::FD,
        BlockFamily::FS,
        m,
        n,
        0.5,
    ))
    .unwrap();
    let chain_sd = CavityTransportChain::from_steps(&realize_two_blocks(
        BlockFamily::FS,
        BlockFamily::FD,
        m,
        n,
        0.5,
    ))
    .unwrap();
    assert_close(chain_ds.cumulative_transport_factor(), d_then_s, 2.0e-12);
    assert_close(chain_sd.cumulative_transport_factor(), s_then_d, 2.0e-12);
    assert!(
        (chain_ds.cumulative_transport_factor() - chain_sd.cumulative_transport_factor()).abs()
            > 0.05
    );

    // The accidental m=n=1 agreement does not restore order-independence.
    assert_close(closed_d_then_s(1, 1), 0.375, 1.0e-15);
    assert_close(closed_s_then_d(1, 1), 0.375, 1.0e-15);
}

#[test]
fn reverse_and_cross_decays_when_forcing_block_lengthens() {
    let rho = 0.9_f64;
    let base_du = closed_d_then_u(4, 4, rho);
    assert!(closed_d_then_u(32, 4, rho) < base_du);
    assert!(closed_d_then_u(4, 256, rho) < base_du);
    assert!(closed_d_then_u(64, 256, rho) < 1.0e-4);

    let base_sd = closed_s_then_d(4, 4);
    assert!(closed_s_then_d(4, 256) < base_sd);
    assert!(closed_s_then_d(4, 256) < 1.0e-2);
}
