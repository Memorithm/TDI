//! Development-only deterministic comparison, not a confirmatory experiment.
//! Outputs logical operation counts and numerical deviations, never speed claims.

use tdi_ai::experimental::tdi22_hierarchical_attention::HierarchicalTorsorBank;
use tdi_ai::experimental::tdi22_torsor::{Torsor3, Twist3, Vec3, direct_pairing};
use tdi_ai::experimental::tdi24_attention::MaskPolicy;

fn v(x: f64, y: f64, z: f64) -> Vec3 {
    Vec3::new(x, y, z).expect("finite deterministic fixture")
}

fn fixture(n: usize) -> Vec<Torsor3> {
    (0..n)
        .map(|i| {
            let x = i as f64;
            Torsor3::new(
                v(0.1 * (x * 0.21).sin(), x * 0.003, (x * 0.07).cos()),
                v(x * 0.002, (x * 0.09).sin(), 0.3 * (x * 0.13).cos()),
                v(x * 0.015, -x * 0.01, 0.1 * (x * 0.02).sin()),
            )
            .expect("finite deterministic torsor")
        })
        .collect()
}

fn independent_dense(
    query: Twist3,
    position: Vec3,
    keys: &[Torsor3],
    visible: usize,
    scale: f64,
) -> (Vec3, Vec3, f64) {
    let scores: Vec<f64> = keys[..visible]
        .iter()
        .map(|key| direct_pairing(query, *key, position).unwrap() * scale)
        .collect();
    let maximum = scores.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let weights: Vec<f64> = scores.iter().map(|score| (score - maximum).exp()).collect();
    let sum: f64 = weights.iter().sum();
    let mut resulting = Vec3::zero();
    let mut moments = Vec3::zero();
    for (key, weight) in keys[..visible].iter().zip(weights) {
        let moment = key.moment_at(position).unwrap();
        resulting = resulting.plus(v(
            key.resultant().x * weight,
            key.resultant().y * weight,
            key.resultant().z * weight,
        ));
        moments = moments.plus(v(moment.x * weight, moment.y * weight, moment.z * weight));
    }
    (
        v(resulting.x / sum, resulting.y / sum, resulting.z / sum),
        v(moments.x / sum, moments.y / sum, moments.z / sum),
        maximum + sum.ln(),
    )
}

fn component_error(a: Vec3, b: Vec3) -> Vec3 {
    v((a.x - b.x).abs(), (a.y - b.y).abs(), (a.z - b.z).abs())
}

fn max_component(a: Vec3) -> f64 {
    a.x.max(a.y).max(a.z)
}

fn main() {
    let keys = fixture(256);
    let bank = HierarchicalTorsorBank::build(&keys).expect("valid finite bank");
    let scale = 0.5;
    let positions = [0, 1, 7, 15, 31, 63, 127, 255];

    println!(
        "tdi22-hierarchy-dev-v1: n={} nodes={}",
        keys.len(),
        bank.node_count()
    );
    println!(
        "policy,score_tolerance,rows,eligible_keys,score_evaluations,collapsed_blocks,max_resultant_abs_error,max_moment_abs_error,max_lse_abs_error"
    );

    for policy in [MaskPolicy::Full, MaskPolicy::Causal] {
        for tolerance in [0.0, 0.01, 0.05, 0.25, 1.0] {
            let mut visible_total = 0;
            let mut score_total = 0;
            let mut collapsed_total = 0;
            let mut max_r: f64 = 0.0;
            let mut max_m: f64 = 0.0;
            let mut max_lse: f64 = 0.0;

            for &i in &positions {
                let x = i as f64;
                let position = v(x * 0.001, -x * 0.002, 0.3);
                let query = Twist3::new(
                    v((x * 0.04).cos(), 0.6, -0.25),
                    v(-0.3, (x * 0.08).sin(), 0.1),
                )
                .unwrap();
                let row = bank
                    .row(query, position, i, policy, scale, tolerance)
                    .expect("bounded hierarchical row");
                let visible = if matches!(policy, MaskPolicy::Causal) {
                    i + 1
                } else {
                    keys.len()
                };
                let (reference_r, reference_m, reference_lse) =
                    independent_dense(query, position, &keys, visible, scale);
                assert_eq!(row.row.visible_keys, visible);
                let error_r = component_error(reference_r, row.row.resultant);
                let error_m = component_error(reference_m, row.row.moment_at_query);
                let error_lse = (row.row.log_sum_exp - reference_lse).abs();

                // These are analytical envelopes, not interval-certified
                // floating point error bounds. The small additive slack
                // accommodates the independent binary64 implementations.
                let slack = 1e-10;
                assert!(error_r.x <= row.bounds.resultant.x + slack);
                assert!(error_r.y <= row.bounds.resultant.y + slack);
                assert!(error_r.z <= row.bounds.resultant.z + slack);
                assert!(error_m.x <= row.bounds.moment_at_query.x + slack);
                assert!(error_m.y <= row.bounds.moment_at_query.y + slack);
                assert!(error_m.z <= row.bounds.moment_at_query.z + slack);
                assert!(error_lse <= row.bounds.log_sum_exp + slack);

                visible_total += visible;
                score_total += row.work.scored_nodes;
                collapsed_total += row.work.collapsed_blocks;
                max_r = max_r.max(max_component(error_r));
                max_m = max_m.max(max_component(error_m));
                max_lse = max_lse.max(error_lse);
            }

            println!(
                "{policy:?},{tolerance:.4},{},{visible_total},{score_total},{collapsed_total},{max_r:.10e},{max_m:.10e},{max_lse:.10e}",
                positions.len()
            );
        }
    }
}
