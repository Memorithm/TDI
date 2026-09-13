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
                .expect("the trichotomy witness construction must be admissible");

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
fn type_s_summable_remainder_product_stays_positive() {
    for steps in [1_usize, 16, 256, 1024] {
        let mut product = 1.0;
        let mut remainder = 0.0;
        for step_index in 1..=steps {
            let alpha = type_s_factor(step_index);
            assert!(alpha > 0.0 && alpha < 1.0);
            product *= alpha;
            remainder += 1.0 - alpha;
        }

        assert_close(product, type_s_closed_product(steps), 2.0e-14);
        assert!(product > 0.5);
        // summable remainder: bounded by the telescoping 1/(k(k+1)) comparison.
        assert!(remainder < 1.0);
    }

    assert!(type_s_closed_product(1024) > 0.5);
    assert!((type_s_closed_product(1024) - 0.5).abs() < 1.0e-3);
}

#[test]
fn type_u_uniform_geometric_product_decays() {
    for &rho in &[0.5_f64, 0.8, 0.9] {
        for steps in [1_usize, 8, 32, 128] {
            let mut product = 1.0;
            for _ in 0..steps {
                product *= rho;
            }
            assert_close(product, rho.powi(steps as i32), 2.0e-14);
            assert!(product <= rho);
            if steps >= 32 && rho <= 0.9 {
                assert!(product < 1.0e-1);
            }
        }
    }
}

#[test]
fn type_d_divergent_remainder_product_decays_while_factors_approach_one() {
    for steps in [1_usize, 16, 256, 1024] {
        let mut product = 1.0;
        let mut remainder = 0.0;
        for step_index in 1..=steps {
            let alpha = type_d_factor(step_index);
            assert!(alpha > 0.0 && alpha < 1.0);
            product *= alpha;
            remainder += 1.0 - alpha;
        }

        assert_close(product, type_d_closed_product(steps), 2.0e-14);
        assert!(product < 1.0);
        if steps >= 256 {
            assert!(product < 1.0e-2);
            assert!(type_d_factor(steps) > 0.99);
        }

        let lower_bound = ((steps + 2) as f64 / 2.0).ln();
        assert!(remainder + 1.0e-12 >= lower_bound);
    }
}

#[test]
fn trichotomy_regimes_are_mutually_exclusive_on_product_limits() {
    let type_s = type_s_closed_product(1024);
    let type_u = 0.9_f64.powi(1024);
    let type_d = type_d_closed_product(1024);

    assert!(type_s > 0.5);
    assert!(type_u < 1.0e-40 || type_u == 0.0 || type_u < 1.0e-20);
    assert!(type_d < 1.0e-3);

    // Type S stays away from zero; U and D vanish.
    assert!(type_s > 100.0 * type_d.max(type_u.max(1.0e-300)));
}

#[test]
fn approaching_one_does_not_imply_type_s() {
    // REFUTED: (alpha_k -> 1) => Type S.
    assert!(type_d_factor(1024) > 0.999);
    assert!(type_d_closed_product(1024) < 1.0e-3);
    assert!(type_s_closed_product(1024) > 0.5);
}

#[test]
fn divergent_remainder_is_not_automatic() {
    // REFUTED: every 0 < alpha_k < 1 family has divergent remainder.
    let mut remainder = 0.0;
    for step_index in 1..=4096 {
        remainder += 1.0 - type_s_factor(step_index);
    }
    assert!(remainder < 1.0);
    assert!(type_s_closed_product(4096) > 0.5);
}

#[test]
fn admissible_cavity_chains_realize_each_trichotomy_type() {
    // Type S
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

    // Type U
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

    // Type D
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
}
