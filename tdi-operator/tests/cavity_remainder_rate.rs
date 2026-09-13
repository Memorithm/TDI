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
                .expect("the remainder-rate witness construction must be admissible");

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
fn type_d_satisfies_harmonic_rate_lower_bound() {
    // EXACT: 1 - alpha_k = 1/(k+1) >= c/k for c in (0,1] and large k.
    let c = 0.5_f64;
    let k0 = 2_usize;
    for k in k0..=4096 {
        let remainder = 1.0 - type_d_factor(k);
        assert!(remainder + 1.0e-15 >= c / (k as f64));
        assert_close(remainder, 1.0 / ((k + 1) as f64), 2.0e-15);
    }
}

#[test]
fn harmonic_comparison_lower_bounds_type_d_remainder_sums() {
    // sum_{k=K}^m (1-alpha_k) >= c * sum_{k=K}^m 1/k with K=2, c=1/2.
    let c = 0.5_f64;
    let k0 = 2_usize;
    for m in [16_usize, 256, 1024, 4096] {
        let mut remainder = 0.0;
        let mut harmonic = 0.0;
        for k in k0..=m {
            remainder += 1.0 - type_d_factor(k);
            harmonic += 1.0 / (k as f64);
        }
        assert!(remainder + 1.0e-12 >= c * harmonic);
        // Full product from k=1 still matches the closed form.
        assert_close(type_d_closed_product(m), 1.0 / ((m + 1) as f64), 2.0e-14);
        assert!(type_d_closed_product(m) < 1.0 / (m as f64));
    }
}

#[test]
fn type_d_closed_product_decays_at_harmonic_rate() {
    for steps in [1_usize, 16, 256, 1024] {
        let mut product = 1.0;
        for step_index in 1..=steps {
            product *= type_d_factor(step_index);
        }
        assert_close(product, type_d_closed_product(steps), 2.0e-14);
        if steps >= 256 {
            assert!(product < 1.0e-2);
            assert!(type_d_factor(steps) > 0.99);
        }
    }
}

#[test]
fn superharmonic_lower_bound_is_not_sufficient_for_decay() {
    // REFUTED: 1-alpha_k >= c/k^{1+eps} (eps=1) => product -> 0.
    // Type S: 1-alpha_k = 1/(k+1)^2 >= c/k^2 for c=1/4 and large k, yet P_n -> 1/2.
    let c = 0.25_f64;
    let k0 = 2_usize;
    for k in k0..=4096 {
        let remainder = 1.0 - type_s_factor(k);
        assert!(remainder + 1.0e-15 >= c / ((k as f64) * (k as f64)));
    }

    for steps in [1_usize, 16, 256, 1024, 4096] {
        let mut product = 1.0;
        for step_index in 1..=steps {
            product *= type_s_factor(step_index);
        }
        assert_close(product, type_s_closed_product(steps), 2.0e-14);
        assert!(product > 0.5);
    }
    assert!((type_s_closed_product(4096) - 0.5).abs() < 2.0e-4);
}

#[test]
fn closed_form_witness_calculus_matches_all_three_types() {
    // Type S
    assert_close(type_s_closed_product(1), 3.0 / 4.0, 2.0e-15);
    assert_close(type_s_closed_product(8), 10.0 / 18.0, 2.0e-15);

    // Type U
    for &rho in &[0.5_f64, 0.8, 0.9] {
        for steps in [1_usize, 8, 32] {
            let mut product = 1.0;
            for _ in 0..steps {
                product *= rho;
            }
            assert_close(product, rho.powi(steps as i32), 2.0e-14);
        }
    }

    // Type D
    assert_close(type_d_closed_product(1), 0.5, 2.0e-15);
    assert_close(type_d_closed_product(99), 0.01, 2.0e-15);
}

#[test]
fn admissible_cavity_chains_realize_rate_witnesses() {
    // Type D harmonic-rate witness
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

    // Type S superharmonic / summable witness (refutation carrier)
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

    // Type U constant-rate witness
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
