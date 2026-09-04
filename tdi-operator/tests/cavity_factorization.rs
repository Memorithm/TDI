use tdi_operator::{
    CavityDriftFactorization, CavityFactorizationError, CavityTransportStep, FrozenToeplitzCavity,
};

fn assert_close(left: f64, right: f64, tolerance: f64) {
    let scale = 1.0_f64.max(left.abs()).max(right.abs());
    assert!(
        (left - right).abs() <= tolerance * scale,
        "left={left:.17e}, right={right:.17e}, |delta|={:.3e}",
        (left - right).abs()
    );
}

#[test]
fn factorization_reconstructs_tdi10_2_drift_and_transport_exactly_numerically() {
    for (a, edge, cavity, q_neighbor, q_current, reference_edge) in [
        (4.0, 0.8, 3.4, 3.1, 3.2, 0.7),
        (5.0, -0.6, 4.2, 3.8, 4.1, -0.9),
        (3.5, 0.0, 3.3, 3.0, 3.2, 0.0),
    ] {
        let step = CavityTransportStep::left(a, edge, cavity, q_neighbor, q_current).unwrap();
        let factors = CavityDriftFactorization::new(step, reference_edge).unwrap();

        assert_close(factors.reconstructed_drift(), step.drift(), 8.0e-15);
        assert_close(
            factors.reconstructed_transport_factor(),
            step.transport_factor(),
            8.0e-15,
        );
    }
}

#[test]
fn constant_toeplitz_reference_zeroes_all_drift_components_when_exactly_matched() {
    let frozen = FrozenToeplitzCavity::new(3.0, 1.0).unwrap();
    let q = frozen.cavity();
    let step = CavityTransportStep::left(3.0, 1.0, q, q, q).unwrap();
    let factors = CavityDriftFactorization::new(step, 1.0).unwrap();

    assert_close(step.drift(), 0.0, 3.0e-15);
    assert_close(factors.reference_defect(), 0.0, 3.0e-15);
    assert_close(factors.edge_drift(), 0.0, 3.0e-15);
    assert_close(factors.reference_drift(), 0.0, 3.0e-15);
    assert_close(factors.cavity_correction(), 1.0, 2.0e-15);
    assert_close(
        factors.normalized_edge_square(),
        frozen.contraction(),
        3.0e-15,
    );
    assert_close(
        factors.reconstructed_transport_factor(),
        frozen.contraction(),
        3.0e-15,
    );
}

#[test]
fn local_reference_defect_is_not_silently_absorbed_into_drift() {
    let step = CavityTransportStep::left(4.0, 0.75, 3.5, 3.0, 3.25).unwrap();
    let factors = CavityDriftFactorization::new(step, 1.1).unwrap();

    let expected_defect = 4.0 - 1.1_f64.powi(2) / 3.25 - 3.25;
    assert_close(factors.reference_defect(), expected_defect, 2.0e-15);
    assert!(factors.reference_defect().abs() > 1.0e-3);
    assert_close(factors.reconstructed_drift(), step.drift(), 5.0e-15);
}

#[test]
fn matched_local_reference_isolates_edge_and_reference_variation() {
    let frozen = FrozenToeplitzCavity::new(4.0, 0.9).unwrap();
    let q_current = frozen.cavity();
    let q_neighbor = q_current * 1.02;
    let actual_edge = 0.8;
    let step = CavityTransportStep::left(4.0, actual_edge, 3.7, q_neighbor, q_current).unwrap();
    let factors = CavityDriftFactorization::new(step, 0.9).unwrap();

    assert_close(factors.reference_defect(), 0.0, 4.0e-15);
    assert_close(
        factors.edge_drift(),
        (0.9_f64.powi(2) - actual_edge.powi(2)) / q_current,
        3.0e-15,
    );
    assert_close(
        factors.reference_drift(),
        actual_edge.powi(2) * (1.0 / q_current - 1.0 / q_neighbor),
        3.0e-15,
    );
    assert_close(factors.reconstructed_drift(), step.drift(), 5.0e-15);
}

#[test]
fn zero_reference_edge_is_a_valid_explicit_reference_choice() {
    let step = CavityTransportStep::left(2.5, 0.0, 2.0, 2.1, 2.2).unwrap();
    let factors = CavityDriftFactorization::new(step, 0.0).unwrap();
    assert_eq!(factors.local_reference_edge(), 0.0);
    assert_eq!(factors.normalized_edge_square(), 0.0);
    assert_eq!(factors.reconstructed_transport_factor(), 0.0);
    assert_close(factors.reconstructed_drift(), step.drift(), 2.0e-15);
}

#[test]
fn factorization_fails_closed_on_nonfinite_or_unrepresentable_reference_metadata() {
    let step = CavityTransportStep::left(4.0, 0.8, 3.4, 3.1, 3.2).unwrap();
    assert!(matches!(
        CavityDriftFactorization::new(step, f64::NAN),
        Err(CavityFactorizationError::NonFiniteReferenceEdge { .. })
    ));
    assert!(matches!(
        CavityDriftFactorization::new(step, 1.0e308),
        Err(CavityFactorizationError::NonFiniteDerivedQuantity)
    ));
}
