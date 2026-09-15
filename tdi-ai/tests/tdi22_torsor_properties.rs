#![cfg(feature = "experimental")]

use tdi_ai::experimental::tdi22_torsor::{
    Torsor3, Twist3, Vec3, direct_pairing, factorized_pairing,
};

fn v(x: f64, y: f64, z: f64) -> Vec3 {
    Vec3::new(x, y, z).expect("finite fixture")
}

fn close(lhs: f64, rhs: f64) {
    let scale = 1.0_f64.max(lhs.abs()).max(rhs.abs());
    assert!(
        (lhs - rhs).abs() <= 1024.0 * f64::EPSILON * scale,
        "lhs={lhs:?}, rhs={rhs:?}"
    );
}

fn close_vec(lhs: Vec3, rhs: Vec3) {
    close(lhs.x, rhs.x);
    close(lhs.y, rhs.y);
    close(lhs.z, rhs.z);
}

#[test]
fn tdi22_transport_composes_across_reduction_points() {
    for index in 1_u32..=64 {
        let t = f64::from(index);
        let source = v(t - 17.0, 0.5 * t - 9.0, 3.0 - 0.25 * t);
        let middle = v(11.0 - 0.75 * t, t - 31.0, 0.125 * t + 5.0);
        let target = v(-7.0 + 0.375 * t, 19.0 - 0.2 * t, t - 23.0);
        let torsor = Torsor3::new(
            v(0.5 * t + 1.0, -0.25 * t + 2.0, 0.125 * t - 3.0),
            v(t - 5.0, 7.0 - 0.4 * t, 0.3 * t + 11.0),
            source,
        )
        .expect("valid torsor");

        let direct = torsor.transport(target).expect("direct transport");
        let composed = torsor
            .transport(middle)
            .expect("first transport")
            .transport(target)
            .expect("second transport");

        close_vec(direct.resultant(), composed.resultant());
        close_vec(direct.moment(), composed.moment());
        close_vec(direct.reference(), composed.reference());
    }
}

#[test]
fn tdi22_direct_and_factorized_pairings_match_on_deterministic_sweep() {
    for index in 1_u32..=128 {
        let t = f64::from(index);
        let query = Twist3::new(
            v(0.1 * t - 3.0, 5.0 - 0.07 * t, 0.03 * t + 1.0),
            v(2.0 - 0.04 * t, 0.09 * t - 4.0, 7.0 - 0.02 * t),
        )
        .expect("valid query");
        let key = Torsor3::new(
            v(0.08 * t + 2.0, -0.05 * t - 1.0, 3.0 - 0.06 * t),
            v(4.0 - 0.03 * t, 0.11 * t + 1.0, -2.0 + 0.07 * t),
            v(t - 65.0, 0.2 * t - 13.0, 17.0 - 0.15 * t),
        )
        .expect("valid key");
        let query_position = v(31.0 - 0.4 * t, t - 80.0, 0.125 * t + 9.0);

        close(
            direct_pairing(query, key, query_position).expect("direct pairing"),
            factorized_pairing(query, key, query_position).expect("factorized pairing"),
        );
    }
}
