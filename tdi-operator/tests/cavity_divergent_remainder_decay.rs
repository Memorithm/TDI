use tdi_operator::{CavityTransportChain, CavityTransportStep};

fn assert_close(left: f64, right: f64, tolerance: f64) {
    let scale = 1.0_f64.max(left.abs()).max(right.abs());
    assert!(
        (left - right).abs() <= tolerance * scale,
        "left={left:.17e}, right={right:.17e}, |delta|={:.3e}",
        (left - right).abs()
    );
}

fn harmonic_factor(step_index: usize) -> f64 {
    assert!(step_index >= 1);
    (step_index as f64) / ((step_index + 1) as f64)
}

fn closed_product(steps: usize) -> f64 {
    1.0 / ((steps + 1) as f64)
}

fn ten_five_factor(step_index: usize) -> f64 {
    assert!(step_index >= 1);
    let denominator = (step_index + 1) as f64;
    1.0 - 1.0 / (denominator * denominator)
}

fn realized_zero_drift_chain(steps: usize, factor: fn(usize) -> f64) -> Vec<CavityTransportStep> {
    assert!(steps >= 1);

    let mut out = Vec::with_capacity(steps);
    let mut cavity = 2.0;
    let mut reference = 1.0;

    for step_index in 1..=steps {
        let alpha = factor(step_index);
        let requested_edge_squared = alpha * cavity * reference;
        let edge = requested_edge_squared.sqrt();
        let realized_edge_squared = edge * edge;
        let current_reference = 1.0;
        let shifted_diagonal = realized_edge_squared / reference + current_reference;

        let step =
            CavityTransportStep::left(shifted_diagonal, edge, cavity, reference, current_reference)
                .expect("the explicit remainder-family construction must be admissible");

        assert!(step.transport_factor() > 0.0);
        assert!(step.transport_factor() <= 1.0);
        assert_close(step.transport_factor(), alpha, 4.0e-15);
        assert_close(step.drift(), 0.0, 4.0e-15);

        cavity = step.current_cavity();
        reference = step.current_reference();
        out.push(step);
    }

    out
}

#[test]
fn log_one_minus_x_is_at_most_minus_x_on_the_unit_interval() {
    for &x in &[
        0.0_f64, 1.0e-12, 1.0e-6, 0.01, 0.25, 0.5, 0.75, 0.9, 0.99, 0.999,
    ] {
        assert!((0.0..1.0).contains(&x));
        let log_term = (1.0 - x).ln();
        assert!(
            log_term <= -x + 1.0e-15,
            "log(1-x)={log_term:.17e} exceeded -x={:.17e} at x={x:.17e}",
            -x
        );
    }
}

#[test]
fn harmonic_remainder_product_telescopes_to_one_over_n_plus_one() {
    for steps in [1_usize, 4, 16, 256, 1024] {
        let mut product = 1.0;
        let mut remainder = 0.0;
        for step_index in 1..=steps {
            let alpha = harmonic_factor(step_index);
            assert!(alpha > 0.0 && alpha < 1.0);
            product *= alpha;
            remainder += 1.0 - alpha;
        }

        assert_close(product, closed_product(steps), 2.0e-14);
        assert!(product < 1.0);
        if steps >= 256 {
            assert!(product < 1.0e-2);
        }

        // Exact integral comparison: sum_{k=1}^n 1/(k+1) >= log((n+2)/2).
        let lower_bound = ((steps + 2) as f64 / 2.0).ln();
        assert!(remainder + 1.0e-12 >= lower_bound);
    }

    assert!(harmonic_factor(1024) > 0.999);
}

#[test]
fn approaching_one_does_not_prevent_product_decay() {
    let mut previous = 1.0;
    for steps in [8_usize, 64, 256, 1024] {
        let alpha = harmonic_factor(steps);
        let product = closed_product(steps);
        assert!(alpha > 0.99 || steps < 256);
        assert!(product < previous);
        previous = product;
    }
    assert!(closed_product(1024) < 1.0e-3);
}

#[test]
fn ten_five_remainder_is_summable_and_product_stays_positive() {
    let mut remainder = 0.0;
    let mut product = 1.0;
    for step_index in 1..=1024 {
        let alpha = ten_five_factor(step_index);
        remainder += 1.0 - alpha;
        product *= alpha;
    }

    // 1/(k+1)^2 < 1/(k(k+1)) and sum 1/(k(k+1)) = 1 - 1/(n+1) < 1.
    assert!(remainder < 1.0);
    assert!(product > 0.5);
}

#[test]
fn admissible_cavity_chain_realizes_divergent_remainder_decay() {
    for steps in [1_usize, 16, 128] {
        let realized_steps = realized_zero_drift_chain(steps, harmonic_factor);
        let chain = CavityTransportChain::from_steps(&realized_steps)
            .expect("the harmonic remainder construction must form one contiguous cavity chain");

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
        assert!(chain.observed_final_error() < closed_product(steps) + 2.0e-12);
        if steps >= 128 {
            assert!(chain.observed_final_error() < 1.0e-2);
        }
    }
}
