use tdi_operator::{CavityTransportChain, CavityTransportStep};

fn assert_close(left: f64, right: f64, tolerance: f64) {
    let scale = 1.0_f64.max(left.abs()).max(right.abs());
    assert!(
        (left - right).abs() <= tolerance * scale,
        "left={left:.17e}, right={right:.17e}, |delta|={:.3e}",
        (left - right).abs()
    );
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

/// Alternating-block witness: x_k = lambda on odd k, x_k = 0 on even k.
fn alternating_alpha(step_index: usize, lambda: f64) -> f64 {
    assert!(step_index >= 1);
    assert!(lambda > 0.0 && lambda < 1.0);
    if step_index % 2 == 1 {
        1.0 - lambda
    } else {
        1.0
    }
}

fn realized_factor_chain(
    steps: usize,
    alpha_at: impl Fn(usize) -> f64,
) -> Vec<CavityTransportStep> {
    assert!(steps >= 1);

    let mut out = Vec::with_capacity(steps);
    let mut cavity = 2.0;
    let mut reference = 1.0;

    for step_index in 1..=steps {
        let alpha = alpha_at(step_index);
        assert!(alpha > 0.0 && alpha <= 1.0);
        let requested_edge_squared = alpha * cavity * reference;
        let edge = requested_edge_squared.sqrt();
        let realized_edge_squared = edge * edge;
        let current_reference = 1.0;
        let shifted_diagonal = realized_edge_squared / reference + current_reference;

        let step =
            CavityTransportStep::left(shifted_diagonal, edge, cavity, reference, current_reference)
                .expect("the Cesaro-rate witness construction must be admissible");

        assert!(step.transport_factor() > 0.0);
        assert!(step.transport_factor() <= 1.0 + 1.0e-14);
        assert_close(step.transport_factor(), alpha, 4.0e-15);
        assert_close(step.drift(), 0.0, 4.0e-15);

        cavity = step.current_cavity();
        reference = step.current_reference();
        out.push(step);
    }

    out
}

#[test]
fn claim1_type_u_constant_cesaro_exponential_rate() {
    // EXACT: constant x_k = lambda => average = lambda, product <= exp(-(lambda-eps)n).
    let lambda = 0.25_f64;
    let alpha = 1.0 - lambda;
    for &eps in &[0.0_f64, 0.01, 0.05, 0.1] {
        assert!(eps < lambda);
        for steps in [1_usize, 8, 32, 128, 256] {
            let product = alpha.powi(steps as i32);
            let mut sum_x = 0.0;
            for _ in 1..=steps {
                sum_x += lambda;
            }
            let avg = sum_x / (steps as f64);
            assert_close(avg, lambda, 1.0e-15);

            // Via 10.11: product <= exp(-sum x) = exp(-lambda n) <= exp(-(lambda-eps)n).
            let bound_via_sum = (-sum_x).exp();
            let rate_bound = (-(lambda - eps) * (steps as f64)).exp();
            assert!(
                product <= bound_via_sum + 1.0e-12 * bound_via_sum.max(1.0),
                "Type U: product={product:.17e} exceeded exp(-sum x)={bound_via_sum:.17e}"
            );
            assert!(
                product <= rate_bound + 1.0e-12 * rate_bound.max(1.0),
                "Type U: product={product:.17e} exceeded exp(-(lambda-eps)n)={rate_bound:.17e}"
            );
            assert!(bound_via_sum <= rate_bound + 1.0e-12 * rate_bound.max(1.0));
        }
    }
}

#[test]
fn claim1_alternating_blocks_cesaro_liminf_exponential_rate() {
    // Alternating x: lambda on odd, 0 on even => Cesaro -> lambda/2 > 0.
    let lambda = 0.4_f64;
    let liminf_target = lambda / 2.0;
    assert!(liminf_target > 0.0);

    // Finite-n: for large n the average is within eps of lambda/2.
    let eps = 0.05_f64;
    assert!(eps < liminf_target);
    let n_threshold = 40_usize; // after this, |avg - lambda/2| < eps for this witness

    for steps in [n_threshold, 64, 128, 256, 512] {
        let mut product = 1.0;
        let mut sum_x = 0.0;
        for k in 1..=steps {
            let alpha = alternating_alpha(k, lambda);
            let x = 1.0 - alpha;
            product *= alpha;
            sum_x += x;
        }
        let avg = sum_x / (steps as f64);

        // Exact Cesaro for this witness: floor((n+1)/2)*lambda / n -> lambda/2.
        let expected_sum = steps.div_ceil(2) as f64 * lambda;
        assert_close(sum_x, expected_sum, 1.0e-14);
        assert!(
            (avg - liminf_target).abs() < eps,
            "avg={avg:.17e} not within eps={eps} of liminf={liminf_target}"
        );
        assert!(avg >= liminf_target - eps);

        // Claim 1 via 10.11.
        let bound_via_sum = (-sum_x).exp();
        let rate_bound = (-(liminf_target - eps) * (steps as f64)).exp();
        assert!(
            product <= bound_via_sum + 1.0e-12 * bound_via_sum.max(1.0),
            "alt: product={product:.17e} exceeded exp(-sum x)={bound_via_sum:.17e}"
        );
        assert!(
            product <= rate_bound + 1.0e-12 * rate_bound.max(1.0),
            "alt: product={product:.17e} exceeded exp(-(L-eps)n)={rate_bound:.17e}"
        );
    }

    // Liminf proxy: averages at even/odd large n stay near lambda/2.
    let mut last_avg = 0.0;
    for steps in [100_usize, 101, 200, 201, 500, 501] {
        let mut sum_x = 0.0;
        for k in 1..=steps {
            sum_x += 1.0 - alternating_alpha(k, lambda);
        }
        last_avg = sum_x / (steps as f64);
        assert!((last_avg - liminf_target).abs() < 0.02);
    }
    assert!(last_avg > 0.0);
}

#[test]
fn refuted_positive_cesaro_limit_necessary_for_product_decay() {
    // REFUTED: (1/n) sum x -> lambda > 0 is necessary for product -> 0.
    // Type D: product = 1/(n+1) -> 0 while (1/n)(H_{n+1}-1) -> 0.
    for steps in [16_usize, 64, 256, 1024, 4096] {
        let product = type_d_closed_product(steps);
        let sum_x = harmonic(steps + 1) - 1.0;
        let avg = sum_x / (steps as f64);

        assert!(product > 0.0);
        assert!(product < 1.0 / (steps as f64)); // -> 0
        // Cesaro avg ~ (log n)/n -> 0 (slowly).
        if steps >= 256 {
            assert!(
                avg < 0.03,
                "Type D Cesaro avg should tend to 0; avg={avg:.17e} at n={steps}"
            );
        }
        if steps >= 1024 {
            assert!(
                avg < 0.01,
                "Type D avg={avg:.17e} should be << 1 at n={steps}"
            );
        }

        // Product decay with vanishing Cesaro mean: necessity REFUTED.
        assert!(product < 0.1 || steps < 16);
    }

    // Direct comparison: avg decreases toward 0 while product decreases toward 0.
    let avg_64 = (harmonic(65) - 1.0) / 64.0;
    let avg_4096 = (harmonic(4097) - 1.0) / 4096.0;
    assert!(avg_4096 < avg_64);
    assert!(type_d_closed_product(4096) < type_d_closed_product(64));
    assert!(avg_4096 < 0.005);
}

#[test]
fn claim2_cesaro_liminf_gives_exponential_not_mere_decay() {
    // EXACT: positive Cesaro liminf => exponential rate (stronger than Type D ->0).
    let lambda = 0.2_f64;
    let alpha = 1.0 - lambda;
    let steps = 200_usize;

    // Type U: exponential.
    let type_u_product = alpha.powi(steps as i32);
    let type_u_rate = (-(lambda * 0.9) * (steps as f64)).exp(); // eps = 0.1*lambda
    assert!(type_u_product <= type_u_rate + 1.0e-12);

    // Type D at same n: only polynomial.
    let type_d_product = type_d_closed_product(steps);
    // Exponential upper envelope with any fixed positive rate undercuts Type U;
    // Type D is larger than any exponential with rate bounded away from 0? No —
    // 1/(n+1) decays slower than exp(-c n). So type_d_product >> type_u_product.
    assert!(
        type_d_product > type_u_product * 1.0e6,
        "Type D polynomial decay must be vastly slower than Type U exponential; \
         D={type_d_product:.17e}, U={type_u_product:.17e}"
    );

    // Ratio test: -log(product)/n -> lambda for Type U, -> 0 for Type D.
    let u_rate = -type_u_product.ln() / (steps as f64);
    let d_rate = -type_d_product.ln() / (steps as f64);
    assert!((u_rate - (-alpha.ln())).abs() < 1.0e-12);
    assert!(u_rate > lambda); // -log(1-lambda) > lambda
    assert!(
        d_rate < 0.05,
        "Type D normalized log-rate must vanish; got {d_rate}"
    );
}

#[test]
fn admissible_cavity_chains_realize_cesaro_witnesses() {
    // Type U constant.
    {
        let lambda = 0.25_f64;
        let alpha = 1.0 - lambda;
        let steps = 64_usize;
        let realized = realized_factor_chain(steps, |_| alpha);
        let chain = CavityTransportChain::from_steps(&realized)
            .expect("Type U construction must form one contiguous cavity chain");
        assert_eq!(chain.steps(), steps);
        let product = chain.cumulative_transport_factor();
        assert_close(product, alpha.powi(steps as i32), 2.0e-12);
        assert!(product <= (-lambda * (steps as f64)).exp() + 1.0e-12);
        assert_close(chain.accumulated_drift(), 0.0, 2.0e-12);
    }

    // Alternating blocks.
    {
        let lambda = 0.4_f64;
        let steps = 128_usize;
        let realized = realized_factor_chain(steps, |k| alternating_alpha(k, lambda));
        let chain = CavityTransportChain::from_steps(&realized)
            .expect("alternating construction must form one contiguous cavity chain");
        assert_eq!(chain.steps(), steps);
        let product = chain.cumulative_transport_factor();
        let expected_sum = steps.div_ceil(2) as f64 * lambda;
        assert!(product <= (-expected_sum).exp() + 1.0e-12);
        let avg = expected_sum / (steps as f64);
        assert!((avg - lambda / 2.0).abs() < 0.01);
        assert_close(chain.accumulated_drift(), 0.0, 2.0e-12);
    }

    // Type D (REFUTED necessity witness).
    {
        let steps = 128_usize;
        let realized = realized_factor_chain(steps, type_d_factor);
        let chain = CavityTransportChain::from_steps(&realized)
            .expect("Type D construction must form one contiguous cavity chain");
        assert_eq!(chain.steps(), steps);
        assert_close(
            chain.cumulative_transport_factor(),
            type_d_closed_product(steps),
            2.0e-12,
        );
        let avg = (harmonic(steps + 1) - 1.0) / (steps as f64);
        assert!(avg < 0.05);
        assert_close(chain.accumulated_drift(), 0.0, 2.0e-12);
    }
}
