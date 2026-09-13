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

fn type_d_factor(step_index: usize) -> f64 {
    assert!(step_index >= 1);
    (step_index as f64) / ((step_index + 1) as f64)
}

fn type_d_closed_product(steps: usize) -> f64 {
    1.0 / ((steps + 1) as f64)
}

fn harmonic(m: usize) -> f64 {
    assert!(m >= 1);
    let mut h = 0.0;
    for j in 1..=m {
        h += 1.0 / (j as f64);
    }
    h
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
                .expect("the product-exp-bounds witness construction must be admissible");

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
fn claim1_product_le_exp_minus_sum_x() {
    // EXACT: product alpha <= exp(-sum x) from log(1-x) <= -x.
    for steps in [1_usize, 8, 64, 256] {
        // Type D
        {
            let mut product = 1.0;
            let mut sum_x = 0.0;
            for k in 1..=steps {
                let alpha = type_d_factor(k);
                let x = 1.0 - alpha;
                product *= alpha;
                sum_x += x;
            }
            let upper = (-sum_x).exp();
            assert!(
                product <= upper + 1.0e-12 * upper.max(1.0),
                "Type D: product={product:.17e} exceeded exp(-sum x)={upper:.17e}"
            );
        }

        // Type S
        {
            let mut product = 1.0;
            let mut sum_x = 0.0;
            for k in 1..=steps {
                let alpha = type_s_factor(k);
                let x = 1.0 - alpha;
                product *= alpha;
                sum_x += x;
            }
            let upper = (-sum_x).exp();
            assert!(
                product <= upper + 1.0e-12 * upper.max(1.0),
                "Type S: product={product:.17e} exceeded exp(-sum x)={upper:.17e}"
            );
        }

        // Type U
        {
            let rho = 0.8_f64;
            let product = rho.powi(steps as i32);
            let sum_x = (1.0 - rho) * (steps as f64);
            let upper = (-sum_x).exp();
            assert!(
                product <= upper + 1.0e-12 * upper.max(1.0),
                "Type U: product={product:.17e} exceeded exp(-sum x)={upper:.17e}"
            );
        }
    }
}

#[test]
fn claim2_cutoff_lower_bound_type_d_k_equals_one() {
    // EXACT: for Type D, every x_k = 1/(k+1) <= 1/2, so declare K=1, C_1=0:
    // product >= exp(-2 sum_{k=1}^n x_k).
    for steps in [1_usize, 8, 64, 256, 1024] {
        let mut product = 1.0;
        let mut sum_x = 0.0;
        for k in 1..=steps {
            let alpha = type_d_factor(k);
            let x = 1.0 - alpha;
            assert!(x <= 0.5 + 1.0e-15);
            product *= alpha;
            sum_x += x;
        }
        let lower = (-2.0 * sum_x).exp();
        assert!(
            product + 1.0e-12 * product.max(1.0) >= lower,
            "Type D K=1: product={product:.17e} fell below exp(-2 sum x)={lower:.17e}"
        );
        assert_close(product, type_d_closed_product(steps), 2.0e-14);
    }
}

#[test]
fn claim2_cutoff_lower_bound_mixed_prefix() {
    // Declared cutoff K=3 on a mixed sample: prefix factors may be far from 1,
    // then a Type D-style near-one tail with x_k <= 1/2.
    // alphas: 0.25, 0.4, then Type D from step index 3 onward (x_3=1/4<=1/2).
    let k_cutoff = 3_usize;
    let prefix = [0.25_f64, 0.4];
    assert_eq!(prefix.len(), k_cutoff - 1);

    for n in [3_usize, 16, 64, 256] {
        assert!(n >= k_cutoff);

        let mut product = 1.0;
        let mut c_k = 0.0; // C_K = -log P_{K-1}
        let mut sum_x_tail = 0.0;

        for (idx, &alpha) in prefix.iter().enumerate() {
            let step = idx + 1;
            assert!(step < k_cutoff);
            assert!(alpha > 0.0 && alpha <= 1.0);
            product *= alpha;
            c_k += -alpha.ln();
        }

        for k in k_cutoff..=n {
            let alpha = type_d_factor(k);
            let x = 1.0 - alpha;
            assert!(x <= 0.5 + 1.0e-15);
            product *= alpha;
            sum_x_tail += x;
        }

        let lower = (-c_k - 2.0 * sum_x_tail).exp();
        assert!(
            product + 1.0e-12 * product.max(1.0) >= lower,
            "mixed K={k_cutoff}: product={product:.17e} fell below exp(-C-2 sum_tail)={lower:.17e}"
        );

        // Also verify the tail-only form product_tail >= exp(-2 sum_x_tail).
        let mut product_tail = 1.0;
        for k in k_cutoff..=n {
            product_tail *= type_d_factor(k);
        }
        let tail_lower = (-2.0 * sum_x_tail).exp();
        assert!(product_tail + 1.0e-12 * product_tail.max(1.0) >= tail_lower);
    }
}

#[test]
fn claim3_type_d_exact_sandwich() {
    // EXACT: product = 1/(n+1), sum x = H_{n+1}-1, and
    // exp(-2(H_{n+1}-1)) <= 1/(n+1) <= exp(-(H_{n+1}-1)).
    for steps in [1_usize, 2, 8, 64, 256, 1024] {
        let mut product = 1.0;
        let mut sum_x = 0.0;
        for k in 1..=steps {
            let alpha = type_d_factor(k);
            product *= alpha;
            sum_x += 1.0 - alpha;
        }

        let closed_product = type_d_closed_product(steps);
        let h = harmonic(steps + 1);
        let remainder = h - 1.0;

        assert_close(product, closed_product, 2.0e-14);
        assert_close(sum_x, remainder, 2.0e-14);

        let upper = (-remainder).exp();
        let lower = (-2.0 * remainder).exp();
        assert!(
            closed_product <= upper + 1.0e-12 * upper.max(1.0),
            "1/(n+1)={closed_product:.17e} exceeded exp(-(H-1))={upper:.17e}"
        );
        assert!(
            closed_product + 1.0e-12 * closed_product.max(1.0) >= lower,
            "1/(n+1)={closed_product:.17e} fell below exp(-2(H-1))={lower:.17e}"
        );
    }
}

#[test]
fn refuted_equality_sharpness_type_d_and_u() {
    // REFUTED: product = exp(-sum x) for all sequences.
    // Equality in log(1-x)=-x only at x=0; Type D and Type U have some x_k>0.
    for steps in [1_usize, 8, 64, 256] {
        // Type D
        {
            let mut product = 1.0;
            let mut sum_x = 0.0;
            for k in 1..=steps {
                let alpha = type_d_factor(k);
                product *= alpha;
                sum_x += 1.0 - alpha;
            }
            let upper = (-sum_x).exp();
            assert!(
                product < upper,
                "Type D must be strictly below exp(-sum x); product={product:.17e}, upper={upper:.17e}"
            );
            // Exact closed-form gap witness for n>=1: x_1 = 1/2 > 0.
            assert!(type_d_factor(1) < 1.0);
            assert!((1.0 - type_d_factor(1) - 0.5).abs() < 1.0e-15);
        }

        // Type U
        {
            let rho = 0.8_f64;
            let product = rho.powi(steps as i32);
            let sum_x = (1.0 - rho) * (steps as f64);
            let upper = (-sum_x).exp();
            assert!(
                product < upper,
                "Type U must be strictly below exp(-sum x); product={product:.17e}, upper={upper:.17e}"
            );
            assert!(1.0 - rho > 0.0);
        }
    }

    // Pointwise: equality in log(1-x)=-x only at x=0.
    let f = |x: f64| -> f64 { -x - (1.0 - x).ln() };
    assert_close(f(0.0), 0.0, 1.0e-15);
    for &x in &[1.0e-6_f64, 0.01, 0.25, 0.5, 0.9] {
        assert!(f(x) > 0.0, "f(x)=-x-log(1-x) must be >0 for x={x}");
    }
}

#[test]
fn admissible_cavity_chains_realize_bound_witnesses() {
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
        let sum_x = harmonic(steps + 1) - 1.0;
        let product = chain.cumulative_transport_factor();
        assert!(product <= (-sum_x).exp() + 1.0e-12);
        assert!(product + 1.0e-12 >= (-2.0 * sum_x).exp());
        assert!(product < (-sum_x).exp());
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
        let product = chain.cumulative_transport_factor();
        let sum_x = (1.0 - rho) * (steps as f64);
        assert_close(product, rho.powi(steps as i32), 2.0e-12);
        assert!(product < (-sum_x).exp());
        assert_close(chain.accumulated_drift(), 0.0, 2.0e-12);
    }
}
