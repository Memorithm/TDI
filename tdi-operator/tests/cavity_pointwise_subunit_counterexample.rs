use tdi_operator::{CavityTransportChain, CavityTransportStep};

fn assert_close(left: f64, right: f64, tolerance: f64) {
    let scale = 1.0_f64.max(left.abs()).max(right.abs());
    assert!(
        (left - right).abs() <= tolerance * scale,
        "left={left:.17e}, right={right:.17e}, |delta|={:.3e}",
        (left - right).abs()
    );
}

fn subunit_factor(step_index: usize) -> f64 {
    assert!(step_index >= 1);
    let denominator = (step_index + 1) as f64;
    1.0 - 1.0 / (denominator * denominator)
}

fn closed_product(steps: usize) -> f64 {
    let n = steps as f64;
    (n + 2.0) / (2.0 * (n + 1.0))
}

fn realized_zero_drift_chain(steps: usize) -> Vec<CavityTransportStep> {
    assert!(steps >= 1);

    let mut out = Vec::with_capacity(steps);
    let mut cavity = 2.0;
    let mut reference = 1.0;

    for step_index in 1..=steps {
        let alpha = subunit_factor(step_index);
        let requested_edge_squared = alpha * cavity * reference;
        let edge = requested_edge_squared.sqrt();
        let realized_edge_squared = edge * edge;
        let current_reference = 1.0;
        let shifted_diagonal = realized_edge_squared / reference + current_reference;

        let step =
            CavityTransportStep::left(shifted_diagonal, edge, cavity, reference, current_reference)
                .expect("the explicit positive counterexample construction must be admissible");

        assert!(step.transport_factor() > 0.0);
        assert!(step.transport_factor() < 1.0);
        assert_close(step.transport_factor(), alpha, 4.0e-15);
        assert_close(step.drift(), 0.0, 4.0e-15);

        cavity = step.current_cavity();
        reference = step.current_reference();
        out.push(step);
    }

    out
}

#[test]
fn telescoping_subunit_product_stays_bounded_away_from_zero() {
    for steps in [1_usize, 4, 16, 256, 1024] {
        let mut product = 1.0;
        for step_index in 1..=steps {
            let alpha = subunit_factor(step_index);
            assert!(alpha > 0.0 && alpha < 1.0);
            product *= alpha;
        }

        assert_close(product, closed_product(steps), 2.0e-14);
        assert!(product > 0.5);
    }

    assert!(subunit_factor(1024) > 0.999_999);
}

#[test]
fn admissible_cavity_chain_realizes_the_nondecaying_boundary_error_witness() {
    for steps in [1_usize, 8, 64, 1024] {
        let realized_steps = realized_zero_drift_chain(steps);
        let chain = CavityTransportChain::from_steps(&realized_steps)
            .expect("the explicit construction must form one contiguous cavity chain");

        assert_eq!(chain.steps(), steps);
        assert_close(chain.initial_error(), 1.0, 2.0e-14);
        assert_close(
            chain.cumulative_transport_factor(),
            closed_product(steps),
            2.0e-12,
        );
        assert_close(chain.accumulated_drift(), 0.0, 2.0e-12);
        assert_close(
            chain.reconstructed_final_error(),
            chain.observed_final_error(),
            2.0e-12,
        );

        // The boundary contribution does not approach zero for this family:
        // its exact real-arithmetic limit is one half of the initial error.
        assert!(chain.observed_final_error() > 0.49);
    }
}
