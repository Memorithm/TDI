use tdi_operator::{CavityTransportChain, CavityTransportStep};

fn assert_close(left: f64, right: f64, tolerance: f64) {
    let scale = 1.0_f64.max(left.abs()).max(right.abs());
    assert!(
        (left - right).abs() <= tolerance * scale,
        "left={left:.17e}, right={right:.17e}, |delta|={:.3e}",
        (left - right).abs()
    );
}

fn realized_constant_factor_chain(steps: usize, rho: f64) -> Vec<CavityTransportStep> {
    assert!(steps >= 1);
    assert!(rho > 0.0 && rho < 1.0);

    let mut out = Vec::with_capacity(steps);
    let mut cavity = 2.0;
    let mut reference = 1.0;

    for _ in 1..=steps {
        let requested_edge_squared = rho * cavity * reference;
        let edge = requested_edge_squared.sqrt();
        let realized_edge_squared = edge * edge;
        let current_reference = 1.0;
        let shifted_diagonal = realized_edge_squared / reference + current_reference;

        let step =
            CavityTransportStep::left(shifted_diagonal, edge, cavity, reference, current_reference)
                .expect("constant uniform-subunit construction must be admissible");

        assert!(step.transport_factor() > 0.0);
        assert!(step.transport_factor() <= rho + 1.0e-14);
        assert_close(step.drift(), 0.0, 4.0e-15);

        cavity = step.current_cavity();
        reference = step.current_reference();
        out.push(step);
    }

    out
}

#[test]
fn uniform_geometric_bound_forces_product_to_zero() {
    for &rho in &[0.5_f64, 0.9, 0.99] {
        for steps in [1_usize, 8, 64, 256] {
            let mut product = 1.0;
            for _ in 0..steps {
                product *= rho;
            }
            assert_close(product, rho.powi(steps as i32), 2.0e-14);
            assert!(product <= rho);
            if steps >= 64 {
                assert!(product < 1.0e-2 || rho >= 0.99);
            }
        }
    }
}

#[test]
fn admissible_cavity_chain_realizes_uniform_subunit_decay() {
    for &rho in &[0.5_f64, 0.8] {
        for steps in [1_usize, 16, 128] {
            let realized_steps = realized_constant_factor_chain(steps, rho);
            let chain = CavityTransportChain::from_steps(&realized_steps)
                .expect("constant-factor construction must form one contiguous cavity chain");

            assert_eq!(chain.steps(), steps);
            assert_close(
                chain.cumulative_transport_factor(),
                rho.powi(steps as i32),
                2.0e-12,
            );
            assert!(chain.cumulative_transport_factor() <= rho + 1.0e-12);
            assert_close(chain.accumulated_drift(), 0.0, 2.0e-12);
            assert_close(
                chain.reconstructed_final_error(),
                chain.observed_final_error(),
                2.0e-12,
            );
        }
    }
}
