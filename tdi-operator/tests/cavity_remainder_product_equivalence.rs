use tdi_operator::{CavityTransportChain, CavityTransportStep};

fn assert_close(left: f64, right: f64, tolerance: f64) {
    let scale = 1.0_f64.max(left.abs()).max(right.abs());
    assert!(
        (left - right).abs() <= tolerance * scale,
        "left={left:.17e}, right={right:.17e}, |delta|={:.3e}",
        (left - right).abs()
    );
}

fn type_s_factor(step_index: usize) -> f64 {
    assert!(step_index >= 1);
    let denominator = (step_index + 1) as f64;
    1.0 - 1.0 / (denominator * denominator)
}

fn type_s_closed_product(steps: usize) -> f64 {
    ((steps + 2) as f64) / (2.0 * ((steps + 1) as f64))
}

fn type_d_factor(step_index: usize) -> f64 {
    assert!(step_index >= 1);
    (step_index as f64) / ((step_index + 1) as f64)
}

fn type_d_closed_product(steps: usize) -> f64 {
    1.0 / ((steps + 1) as f64)
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
                .expect("the equivalence witness construction must be admissible");

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

fn realized_constant_factor_chain(steps: usize, rho: f64) -> Vec<CavityTransportStep> {
    assert!(steps >= 1);
    assert!(rho > 0.0 && rho < 1.0);

    let mut out = Vec::with_capacity(steps);
    let mut cavity = 2.0;
    let mut reference = 1.0;

    for _step_index in 1..=steps {
        let requested_edge_squared = rho * cavity * reference;
        let edge = requested_edge_squared.sqrt();
        let realized_edge_squared = edge * edge;
        let current_reference = 1.0;
        let shifted_diagonal = realized_edge_squared / reference + current_reference;

        let step =
            CavityTransportStep::left(shifted_diagonal, edge, cavity, reference, current_reference)
                .expect("the Type U constant-factor construction must be admissible");

        assert!(step.transport_factor() > 0.0);
        assert!(step.transport_factor() <= rho + 1.0e-14);
        assert_close(step.transport_factor(), rho, 4.0e-15);
        assert_close(step.drift(), 0.0, 4.0e-15);

        cavity = step.current_cavity();
        reference = step.current_reference();
        out.push(step);
    }

    out
}

#[test]
fn sandwich_x_le_minus_log_le_x_over_one_minus_x() {
    // EXACT: for x in [0,1), x <= -log(1-x) <= x/(1-x).
    for &x in &[
        0.0_f64, 1.0e-12, 1.0e-6, 0.01, 0.25, 0.5, 0.75, 0.9, 0.99, 0.999,
    ] {
        assert!((0.0..1.0).contains(&x));
        let minus_log = -(1.0 - x).ln();
        let upper = if x == 0.0 { 0.0 } else { x / (1.0 - x) };
        assert!(
            minus_log + 1.0e-15 >= x,
            "-log(1-x)={minus_log:.17e} fell below x={x:.17e}"
        );
        assert!(
            minus_log <= upper + 1.0e-12,
            "-log(1-x)={minus_log:.17e} exceeded x/(1-x)={upper:.17e} at x={x:.17e}"
        );
    }
}

#[test]
fn type_s_all_three_fail_together() {
    // Summable remainder, bounded -log sum, product -> 1/2 != 0.
    let mut remainder = 0.0;
    let mut neg_log = 0.0;
    let mut product = 1.0;
    for step_index in 1..=4096 {
        let alpha = type_s_factor(step_index);
        assert!(alpha > 0.0 && alpha <= 1.0);
        remainder += 1.0 - alpha;
        neg_log += -alpha.ln();
        product *= alpha;
    }

    assert!(remainder < 1.0);
    assert!(neg_log < 1.0);
    assert_close(product, type_s_closed_product(4096), 2.0e-14);
    assert!(product > 0.5);
    assert!((type_s_closed_product(4096) - 0.5).abs() < 2.0e-4);
}

#[test]
fn type_d_all_three_hold_together() {
    // Divergent remainder, divergent -log sum, product -> 0.
    for steps in [256_usize, 1024, 4096] {
        let mut remainder = 0.0;
        let mut neg_log = 0.0;
        let mut product = 1.0;
        for step_index in 1..=steps {
            let alpha = type_d_factor(step_index);
            assert!(alpha > 0.0 && alpha <= 1.0);
            remainder += 1.0 - alpha;
            neg_log += -alpha.ln();
            product *= alpha;
        }

        let harmonic_lower = ((steps + 2) as f64 / 2.0).ln();
        assert!(remainder + 1.0e-12 >= harmonic_lower);
        // -log(1-x) >= x => neg_log >= remainder.
        assert!(neg_log + 1.0e-12 >= remainder);
        assert_close(product, type_d_closed_product(steps), 2.0e-14);
        assert!(product < 1.0 / (steps as f64));
        if steps >= 1024 {
            assert!(product < 1.0e-3);
            assert!(type_d_factor(steps) > 0.999);
        }
    }
}

#[test]
fn type_u_all_three_hold_without_alpha_approaching_one() {
    // REFUTED carrier for "alpha -> 1 required": constant rho stays away from 1.
    let rho = 0.8_f64;
    for steps in [16_usize, 64, 256] {
        let remainder = (1.0 - rho) * (steps as f64);
        let neg_log = -rho.ln() * (steps as f64);
        let product = rho.powi(steps as i32);

        assert!(remainder > 1.0);
        assert!(neg_log + 1.0e-12 >= remainder);
        assert!(product < 1.0e-1 || steps < 16);
        if steps >= 64 {
            assert!(product < 1.0e-5);
        }
        // Factors never approach 1.
        assert!((rho - 0.8).abs() < 1.0e-15);
        assert!(rho < 0.85);
    }
}

#[test]
fn zero_factor_refutes_necessity_without_alpha_positive() {
    // REFUTED: product -> 0 => divergent remainder, if alpha=0 is allowed.
    // alpha_1 = 0, alpha_k = 1 for k >= 2: product vanishes, remainder = 1.
    let alphas = [0.0_f64, 1.0, 1.0, 1.0, 1.0];
    let mut product = 1.0;
    let mut remainder = 0.0;
    for &alpha in &alphas {
        product *= alpha;
        remainder += 1.0 - alpha;
    }
    assert_eq!(product, 0.0);
    assert_close(remainder, 1.0, 1.0e-15);
    assert!(remainder < 2.0);
}

#[test]
fn log_sum_lower_bounds_remainder_on_positive_factors() {
    // EXACT consequence of -log(1-x) >= x for sample factors in (0,1].
    for &alpha in &[1.0_f64, 0.999, 0.9, 0.5, 0.1, 1.0e-3] {
        assert!(alpha > 0.0 && alpha <= 1.0);
        let x = 1.0 - alpha;
        let neg_log = -alpha.ln();
        assert!(neg_log + 1.0e-15 >= x);
    }
}

#[test]
fn admissible_cavity_chains_realize_equivalence_witnesses() {
    // Type S: product not to 0, summable remainder
    {
        let steps = 64_usize;
        let realized = realized_zero_drift_chain(steps, type_s_factor);
        let chain = CavityTransportChain::from_steps(&realized)
            .expect("Type S construction must form one contiguous cavity chain");
        assert_eq!(chain.steps(), steps);
        assert_close(
            chain.cumulative_transport_factor(),
            type_s_closed_product(steps),
            2.0e-12,
        );
        assert!(chain.cumulative_transport_factor() > 0.5);
        assert_close(chain.accumulated_drift(), 0.0, 2.0e-12);
    }

    // Type D: product -> 0, divergent remainder
    {
        let steps = 128_usize;
        let realized = realized_zero_drift_chain(steps, type_d_factor);
        let chain = CavityTransportChain::from_steps(&realized)
            .expect("Type D construction must form one contiguous cavity chain");
        assert_eq!(chain.steps(), steps);
        assert_close(
            chain.cumulative_transport_factor(),
            type_d_closed_product(steps),
            2.0e-12,
        );
        assert!(chain.cumulative_transport_factor() < 1.0e-2);
        assert_close(chain.accumulated_drift(), 0.0, 2.0e-12);
    }

    // Type U: product -> 0 without alpha -> 1
    {
        let rho = 0.8_f64;
        let steps = 32_usize;
        let realized = realized_constant_factor_chain(steps, rho);
        let chain = CavityTransportChain::from_steps(&realized)
            .expect("Type U construction must form one contiguous cavity chain");
        assert_eq!(chain.steps(), steps);
        assert_close(
            chain.cumulative_transport_factor(),
            rho.powi(steps as i32),
            2.0e-12,
        );
        assert!(chain.cumulative_transport_factor() < 1.0e-2);
        assert_close(chain.accumulated_drift(), 0.0, 2.0e-12);
    }
}
