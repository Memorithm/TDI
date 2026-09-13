//! TDI-10.20 — EXACT three-block family concatenation and interleaved schedules.
//!
//! Finite three-block product identities for F_U / F_D / F_S (mixed
//! permutations and same-family products under index restart); interleaved
//! pair-schedules versus blocked concatenations; Toeplitz-sourced ρ reuse
//! from TDI-10.14; REFUTED that three-block {U,D,S} order is immaterial;
//! REFUTED that interleaving vs blocking is immaterial; REFUTED that
//! index-restart bookkeeping is immaterial for Type-D / Type-S; REFUTED
//! that a Type-S middle erases Type-U prefix decay uniformly in ℓ; REFUTED
//! that a finite Type-D prefix plus Type-S middle forces liminf 0 under a
//! Type-U suffix of fixed n. Does not prove slowly-varying Jacobi
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

fn family_closed(family: BlockFamily, steps: usize, rho: f64) -> f64 {
    match family {
        BlockFamily::FU => rho.powi(steps as i32),
        BlockFamily::FD => 1.0 / ((steps + 1) as f64),
        BlockFamily::FS => ((steps + 2) as f64) / (2.0 * ((steps + 1) as f64)),
    }
}

fn closed_three(
    first: BlockFamily,
    second: BlockFamily,
    third: BlockFamily,
    ell: usize,
    m: usize,
    n: usize,
    rho: f64,
) -> f64 {
    family_closed(first, ell, rho) * family_closed(second, m, rho) * family_closed(third, n, rho)
}

fn closed_interleave(first: BlockFamily, second: BlockFamily, pairs: usize, rho: f64) -> f64 {
    (family_closed(first, 1, rho) * family_closed(second, 1, rho)).powi(pairs as i32)
}

fn closed_blocked(first: BlockFamily, second: BlockFamily, k: usize, rho: f64) -> f64 {
    family_closed(first, k, rho) * family_closed(second, k, rho)
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
        .expect("TDI-10.20 composed family construction must be admissible");

        assert_close(step.transport_factor(), alpha, 4.0e-15);
        assert_close(step.drift(), 0.0, 4.0e-15);

        *cavity = step.current_cavity();
        *reference = step.current_reference();
        out.push(step);
    }
}

fn realize_three_blocks(
    first: BlockFamily,
    second: BlockFamily,
    third: BlockFamily,
    ell: usize,
    m: usize,
    n: usize,
    rho: f64,
) -> Vec<CavityTransportStep> {
    let mut out = Vec::with_capacity(ell + m + n);
    let mut cavity = 2.0;
    let mut reference = 1.0;
    append_block(&mut out, first, ell, rho, &mut cavity, &mut reference);
    append_block(&mut out, second, m, rho, &mut cavity, &mut reference);
    append_block(&mut out, third, n, rho, &mut cavity, &mut reference);
    out
}

fn realize_two_blocks(
    first: BlockFamily,
    second: BlockFamily,
    m: usize,
    n: usize,
    rho: f64,
) -> Vec<CavityTransportStep> {
    let mut out = Vec::with_capacity(m + n);
    let mut cavity = 2.0;
    let mut reference = 1.0;
    append_block(&mut out, first, m, rho, &mut cavity, &mut reference);
    append_block(&mut out, second, n, rho, &mut cavity, &mut reference);
    out
}

fn realize_interleave(
    first: BlockFamily,
    second: BlockFamily,
    pairs: usize,
    rho: f64,
) -> Vec<CavityTransportStep> {
    let mut out = Vec::with_capacity(2 * pairs);
    let mut cavity = 2.0;
    let mut reference = 1.0;
    for _ in 0..pairs {
        append_block(&mut out, first, 1, rho, &mut cavity, &mut reference);
        append_block(&mut out, second, 1, rho, &mut cavity, &mut reference);
    }
    out
}

fn algebraic_three(
    first: BlockFamily,
    second: BlockFamily,
    third: BlockFamily,
    ell: usize,
    m: usize,
    n: usize,
    rho: f64,
) -> f64 {
    let mut product = 1.0;
    for local in 1..=ell {
        product *= block_alpha(first, local, rho);
    }
    for local in 1..=m {
        product *= block_alpha(second, local, rho);
    }
    for local in 1..=n {
        product *= block_alpha(third, local, rho);
    }
    product
}

fn algebraic_interleave(first: BlockFamily, second: BlockFamily, pairs: usize, rho: f64) -> f64 {
    let mut product = 1.0;
    for _ in 0..pairs {
        product *= block_alpha(first, 1, rho);
        product *= block_alpha(second, 1, rho);
    }
    product
}

const MIXED_PERMUTATIONS: [(BlockFamily, BlockFamily, BlockFamily); 6] = [
    (BlockFamily::FU, BlockFamily::FD, BlockFamily::FS),
    (BlockFamily::FU, BlockFamily::FS, BlockFamily::FD),
    (BlockFamily::FD, BlockFamily::FU, BlockFamily::FS),
    (BlockFamily::FD, BlockFamily::FS, BlockFamily::FU),
    (BlockFamily::FS, BlockFamily::FU, BlockFamily::FD),
    (BlockFamily::FS, BlockFamily::FD, BlockFamily::FU),
];

#[test]
fn algebraic_three_block_matches_closed_product() {
    for &rho in &[0.5_f64, 0.8, 0.9] {
        for &(first, second, third) in &MIXED_PERMUTATIONS {
            for ell in [1_usize, 4, 16] {
                for m in [1_usize, 2, 8] {
                    for n in [1_usize, 8] {
                        assert_close(
                            algebraic_three(first, second, third, ell, m, n, rho),
                            closed_three(first, second, third, ell, m, n, rho),
                            2.0e-14,
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn algebraic_same_family_three_block_matches_closed_product() {
    for &rho in &[0.5_f64, 0.75] {
        for family in [BlockFamily::FU, BlockFamily::FD, BlockFamily::FS] {
            for ell in [1_usize, 2, 8] {
                for m in [1_usize, 3] {
                    for n in [1_usize, 4] {
                        assert_close(
                            algebraic_three(family, family, family, ell, m, n, rho),
                            closed_three(family, family, family, ell, m, n, rho),
                            2.0e-14,
                        );
                    }
                }
            }
        }
    }
    // Type-U three-block equals one Type-U block of total length.
    let rho = 0.8_f64;
    assert_close(
        closed_three(
            BlockFamily::FU,
            BlockFamily::FU,
            BlockFamily::FU,
            2,
            3,
            5,
            rho,
        ),
        family_closed(BlockFamily::FU, 10, rho),
        1.0e-15,
    );
}

#[test]
fn algebraic_three_block_product_is_associative() {
    let rho = 0.7_f64;
    let ell = 3_usize;
    let m = 5_usize;
    let n = 4_usize;
    for &(first, second, third) in &MIXED_PERMUTATIONS {
        let left = (family_closed(first, ell, rho) * family_closed(second, m, rho))
            * family_closed(third, n, rho);
        let right = family_closed(first, ell, rho)
            * (family_closed(second, m, rho) * family_closed(third, n, rho));
        assert_close(left, right, 1.0e-15);
        assert_close(
            left,
            closed_three(first, second, third, ell, m, n, rho),
            1.0e-15,
        );
    }
}

#[test]
fn cavity_chain_realizes_mixed_three_blocks() {
    let rho = 0.8_f64;
    for &(first, second, third) in &MIXED_PERMUTATIONS {
        for (ell, m, n) in [(2_usize, 3, 4), (4, 2, 8)] {
            let realized = realize_three_blocks(first, second, third, ell, m, n, rho);
            assert_eq!(realized.len(), ell + m + n);
            let chain = CavityTransportChain::from_steps(&realized)
                .expect("three-block schedule must form one contiguous cavity chain");
            assert_eq!(chain.steps(), ell + m + n);
            assert_close(
                chain.cumulative_transport_factor(),
                closed_three(first, second, third, ell, m, n, rho),
                2.0e-12,
            );
            assert_close(chain.accumulated_drift(), 0.0, 2.0e-12);
        }
    }
}

#[test]
fn cavity_chain_realizes_same_family_three_blocks() {
    let rho = 0.75_f64;
    for family in [BlockFamily::FU, BlockFamily::FD, BlockFamily::FS] {
        let realized = realize_three_blocks(family, family, family, 2, 2, 2, rho);
        let chain = CavityTransportChain::from_steps(&realized)
            .expect("same-family three-block schedule must form one contiguous chain");
        assert_close(
            chain.cumulative_transport_factor(),
            closed_three(family, family, family, 2, 2, 2, rho),
            2.0e-12,
        );
        assert_close(chain.accumulated_drift(), 0.0, 2.0e-12);
    }
}

#[test]
fn algebraic_and_chain_interleave_matches_closed_product() {
    for &rho in &[0.5_f64, 0.8] {
        for &(first, second) in &[
            (BlockFamily::FU, BlockFamily::FD),
            (BlockFamily::FU, BlockFamily::FS),
            (BlockFamily::FD, BlockFamily::FS),
        ] {
            for pairs in [1_usize, 2, 4, 8] {
                assert_close(
                    algebraic_interleave(first, second, pairs, rho),
                    closed_interleave(first, second, pairs, rho),
                    2.0e-14,
                );
                let realized = realize_interleave(first, second, pairs, rho);
                assert_eq!(realized.len(), 2 * pairs);
                let chain = CavityTransportChain::from_steps(&realized)
                    .expect("interleaved pair-schedule must form one contiguous chain");
                assert_close(
                    chain.cumulative_transport_factor(),
                    closed_interleave(first, second, pairs, rho),
                    2.0e-12,
                );
                assert_close(chain.accumulated_drift(), 0.0, 2.0e-12);
            }
        }
    }
}

#[test]
fn blocked_two_block_products_match_15_19_closed_forms() {
    let rho = 0.6_f64;
    for k in [1_usize, 2, 8, 16] {
        assert_close(
            closed_blocked(BlockFamily::FU, BlockFamily::FD, k, rho),
            rho.powi(k as i32) / ((k + 1) as f64),
            1.0e-15,
        );
        assert_close(
            closed_blocked(BlockFamily::FU, BlockFamily::FS, k, rho),
            rho.powi(k as i32) * ((k + 2) as f64) / (2.0 * ((k + 1) as f64)),
            1.0e-15,
        );
        assert_close(
            closed_blocked(BlockFamily::FD, BlockFamily::FS, k, 0.5),
            (1.0 / ((k + 1) as f64)) * ((k + 2) as f64) / (2.0 * ((k + 1) as f64)),
            1.0e-15,
        );
        let ud = realize_two_blocks(BlockFamily::FU, BlockFamily::FD, k, k, rho);
        let chain = CavityTransportChain::from_steps(&ud).unwrap();
        assert_close(
            chain.cumulative_transport_factor(),
            closed_blocked(BlockFamily::FU, BlockFamily::FD, k, rho),
            2.0e-12,
        );
    }
}

#[test]
fn toeplitz_sourced_rho_composes_u_then_d_then_s() {
    let frozen = FrozenToeplitzCavity::new(3.0, 1.0)
        .expect("TDI-10.1 frozen Toeplitz example (3,1) must be admissible");
    let rho = frozen.contraction();
    assert!(rho > 0.0 && rho < 1.0);

    let ell = 6_usize;
    let m = 4_usize;
    let n = 8_usize;
    let realized = realize_three_blocks(
        BlockFamily::FU,
        BlockFamily::FD,
        BlockFamily::FS,
        ell,
        m,
        n,
        rho,
    );
    let chain = CavityTransportChain::from_steps(&realized)
        .expect("Toeplitz-sourced U→D→S must form one contiguous chain");

    assert_close(
        chain.cumulative_transport_factor(),
        closed_three(
            BlockFamily::FU,
            BlockFamily::FD,
            BlockFamily::FS,
            ell,
            m,
            n,
            rho,
        ),
        2.0e-12,
    );
    assert_close(chain.accumulated_drift(), 0.0, 2.0e-12);
}

#[test]
fn three_block_uds_order_is_not_immaterial() {
    // REFUTED: the six mixed {U,D,S} products are equal for all lengths.
    let ell = 2_usize;
    let m = 2_usize;
    let n = 1_usize;
    let rho = 0.5_f64;
    let uds = closed_three(
        BlockFamily::FU,
        BlockFamily::FD,
        BlockFamily::FS,
        ell,
        m,
        n,
        rho,
    );
    let usd = closed_three(
        BlockFamily::FU,
        BlockFamily::FS,
        BlockFamily::FD,
        ell,
        m,
        n,
        rho,
    );
    let dsu = closed_three(
        BlockFamily::FD,
        BlockFamily::FS,
        BlockFamily::FU,
        ell,
        m,
        n,
        rho,
    );
    assert_close(uds, 1.0 / 16.0, 1.0e-15);
    assert_close(usd, 1.0 / 12.0, 1.0e-15);
    assert_close(dsu, 1.0 / 9.0, 1.0e-15);
    assert!((uds - usd).abs() > 0.01);
    assert!((usd - dsu).abs() > 0.01);
    assert!((uds - dsu).abs() > 0.01);

    let chain_uds = CavityTransportChain::from_steps(&realize_three_blocks(
        BlockFamily::FU,
        BlockFamily::FD,
        BlockFamily::FS,
        ell,
        m,
        n,
        rho,
    ))
    .unwrap();
    let chain_usd = CavityTransportChain::from_steps(&realize_three_blocks(
        BlockFamily::FU,
        BlockFamily::FS,
        BlockFamily::FD,
        ell,
        m,
        n,
        rho,
    ))
    .unwrap();
    let chain_dsu = CavityTransportChain::from_steps(&realize_three_blocks(
        BlockFamily::FD,
        BlockFamily::FS,
        BlockFamily::FU,
        ell,
        m,
        n,
        rho,
    ))
    .unwrap();
    assert_close(chain_uds.cumulative_transport_factor(), uds, 2.0e-12);
    assert_close(chain_usd.cumulative_transport_factor(), usd, 2.0e-12);
    assert_close(chain_dsu.cumulative_transport_factor(), dsu, 2.0e-12);
}

#[test]
fn interleaving_versus_blocking_is_not_immaterial() {
    // REFUTED: only the multiset of family step counts matters.
    let k = 2_usize;
    let rho = 0.5_f64;
    let interleaved = closed_interleave(BlockFamily::FU, BlockFamily::FD, k, rho);
    let blocked = closed_blocked(BlockFamily::FU, BlockFamily::FD, k, rho);
    assert_close(interleaved, 1.0 / 16.0, 1.0e-15);
    assert_close(blocked, 1.0 / 12.0, 1.0e-15);
    assert!((interleaved - blocked).abs() > 0.01);

    let chain_i = CavityTransportChain::from_steps(&realize_interleave(
        BlockFamily::FU,
        BlockFamily::FD,
        k,
        rho,
    ))
    .unwrap();
    let chain_b = CavityTransportChain::from_steps(&realize_two_blocks(
        BlockFamily::FU,
        BlockFamily::FD,
        k,
        k,
        rho,
    ))
    .unwrap();
    assert_close(chain_i.cumulative_transport_factor(), interleaved, 2.0e-12);
    assert_close(chain_b.cumulative_transport_factor(), blocked, 2.0e-12);

    let us_i = closed_interleave(BlockFamily::FU, BlockFamily::FS, k, rho);
    let us_b = closed_blocked(BlockFamily::FU, BlockFamily::FS, k, rho);
    assert_close(us_i, (3.0_f64 / 8.0).powi(2), 1.0e-15);
    assert_close(us_b, 0.25 * 4.0 / 6.0, 1.0e-15);
    assert!((us_i - us_b).abs() > 0.01);
}

#[test]
fn index_restart_is_not_immaterial_for_type_d_or_type_s() {
    // REFUTED: three F_D (resp. F_S) blocks of lengths ℓ,m,n equal one block
    // of total length ℓ+m+n.
    let three_d = closed_three(
        BlockFamily::FD,
        BlockFamily::FD,
        BlockFamily::FD,
        2,
        2,
        2,
        0.5,
    );
    let one_d = family_closed(BlockFamily::FD, 6, 0.5);
    assert_close(three_d, 1.0 / 27.0, 1.0e-15);
    assert_close(one_d, 1.0 / 7.0, 1.0e-15);
    assert!((three_d - one_d).abs() > 0.05);

    let three_s = closed_three(
        BlockFamily::FS,
        BlockFamily::FS,
        BlockFamily::FS,
        2,
        2,
        2,
        0.5,
    );
    let one_s = family_closed(BlockFamily::FS, 6, 0.5);
    assert_close(three_s, 8.0 / 27.0, 1.0e-15);
    assert_close(one_s, 4.0 / 7.0, 1.0e-15);
    assert!((three_s - one_s).abs() > 0.2);

    let chain_d = CavityTransportChain::from_steps(&realize_three_blocks(
        BlockFamily::FD,
        BlockFamily::FD,
        BlockFamily::FD,
        2,
        2,
        2,
        0.5,
    ))
    .unwrap();
    let chain_s = CavityTransportChain::from_steps(&realize_three_blocks(
        BlockFamily::FS,
        BlockFamily::FS,
        BlockFamily::FS,
        2,
        2,
        2,
        0.5,
    ))
    .unwrap();
    assert_close(chain_d.cumulative_transport_factor(), three_d, 2.0e-12);
    assert_close(chain_s.cumulative_transport_factor(), three_s, 2.0e-12);

    // Type-U is the exception: restart is immaterial because alpha is constant.
    let rho = 0.8_f64;
    assert_close(
        closed_three(
            BlockFamily::FU,
            BlockFamily::FU,
            BlockFamily::FU,
            2,
            2,
            2,
            rho,
        ),
        family_closed(BlockFamily::FU, 6, rho),
        1.0e-15,
    );
}

#[test]
fn type_s_middle_does_not_erase_type_u_prefix_decay() {
    // REFUTED: a Type-S middle produces a positive lower bound independent of ℓ.
    let rho = 0.5_f64;
    let m = 8_usize;
    let n = 4_usize;
    let short = closed_three(
        BlockFamily::FU,
        BlockFamily::FS,
        BlockFamily::FD,
        2,
        m,
        n,
        rho,
    );
    let long = closed_three(
        BlockFamily::FU,
        BlockFamily::FS,
        BlockFamily::FD,
        16,
        m,
        n,
        rho,
    );
    assert!(long < short);
    assert!(long < 1.0e-5);
    assert!(short > 1.0e-3);

    let realized = realize_three_blocks(
        BlockFamily::FU,
        BlockFamily::FS,
        BlockFamily::FD,
        16,
        m,
        n,
        rho,
    );
    let chain = CavityTransportChain::from_steps(&realized).unwrap();
    assert!(chain.cumulative_transport_factor() < 1.0e-5);
    assert_close(chain.cumulative_transport_factor(), long, 2.0e-12);
}

#[test]
fn finite_type_d_prefix_plus_type_s_middle_does_not_force_liminf_zero_under_fixed_u() {
    // REFUTED: every finite Type-D prefix forces lim_m P_{D→S→U}(ℓ,m,n;ρ) = 0
    // for fixed suffix length n.
    let ell = 4_usize;
    let n = 2_usize;
    let rho = 0.8_f64;
    let liminf = (1.0 / (2.0 * ((ell + 1) as f64))) * rho.powi(n as i32);
    assert!(liminf > 0.05);

    let m = 1024_usize;
    let product = closed_three(
        BlockFamily::FD,
        BlockFamily::FS,
        BlockFamily::FU,
        ell,
        m,
        n,
        rho,
    );
    assert_close(product, liminf, 5.0e-3);
    assert!(product > 0.05);

    // Contrast: only sending ℓ → ∞ (or n → ∞) drives the limit to 0.
    let large_ell = 256_usize;
    assert!(1.0 / (2.0 * ((large_ell + 1) as f64)) * rho.powi(n as i32) < 0.01);
}

#[test]
fn documented_length_one_pair_factors() {
    let rho = 0.5_f64;
    assert_close(family_closed(BlockFamily::FU, 1, rho), rho, 1.0e-15);
    assert_close(family_closed(BlockFamily::FD, 1, 0.5), 0.5, 1.0e-15);
    assert_close(family_closed(BlockFamily::FS, 1, 0.5), 0.75, 1.0e-15);
    assert_close(
        closed_interleave(BlockFamily::FU, BlockFamily::FD, 1, rho),
        rho / 2.0,
        1.0e-15,
    );
    assert_close(
        closed_interleave(BlockFamily::FU, BlockFamily::FS, 1, rho),
        3.0 * rho / 4.0,
        1.0e-15,
    );
    assert_close(
        closed_interleave(BlockFamily::FD, BlockFamily::FS, 1, 0.5),
        3.0 / 8.0,
        1.0e-15,
    );
}
