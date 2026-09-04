use tdi_operator::{
    CavityTransportError, CavityTransportStep, FrozenToeplitzCavity, JacobiMatrix, SchurCavities,
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
fn left_and_right_steps_reconstruct_the_exact_error_identity() {
    let left = CavityTransportStep::left(4.0, 1.0, 3.0, 2.5, 3.2).unwrap();
    assert_close(left.current_cavity(), 4.0 - 1.0 / 3.0, 1.0e-15);
    assert_close(left.current_error(), left.reconstructed_error(), 2.0e-15);

    let right = CavityTransportStep::right(5.0, -0.75, 4.2, 3.9, 4.1).unwrap();
    assert_close(
        right.current_cavity(),
        5.0 - 0.75_f64.powi(2) / 4.2,
        1.0e-15,
    );
    assert_close(right.current_error(), right.reconstructed_error(), 2.0e-15);
}

#[test]
fn transport_matches_finite_schur_cavities_for_arbitrary_positive_references() {
    let matrix = JacobiMatrix::new(
        vec![3.0, 3.2, 3.4, 3.6, 3.8],
        vec![0.5, -0.7, 0.6, 0.4],
    )
    .unwrap();
    let shift = 0.4;
    let cavities = SchurCavities::compute(&matrix, shift).unwrap();
    let references = [2.7, 2.9, 3.1, 3.3, 3.5];

    for i in 1..matrix.len() {
        let step = CavityTransportStep::left(
            matrix.diagonal()[i] + shift,
            matrix.off_diagonal()[i - 1],
            cavities.left()[i - 1],
            references[i - 1],
            references[i],
        )
        .unwrap();
        assert_close(step.current_cavity(), cavities.left()[i], 2.0e-15);
        assert_close(step.current_error(), step.reconstructed_error(), 3.0e-15);
    }

    for i in 0..matrix.len() - 1 {
        let step = CavityTransportStep::right(
            matrix.diagonal()[i] + shift,
            matrix.off_diagonal()[i],
            cavities.right()[i + 1],
            references[i + 1],
            references[i],
        )
        .unwrap();
        assert_close(step.current_cavity(), cavities.right()[i], 2.0e-15);
        assert_close(step.current_error(), step.reconstructed_error(), 3.0e-15);
    }
}

#[test]
fn recurrence_consistent_references_make_the_drift_exactly_zero_up_to_roundoff() {
    let shifted_diagonal = 4.25;
    let edge = 0.8;
    let neighbor_reference = 3.1;
    let current_reference = shifted_diagonal - edge * edge / neighbor_reference;
    let neighbor_cavity = 3.4;

    let step = CavityTransportStep::left(
        shifted_diagonal,
        edge,
        neighbor_cavity,
        neighbor_reference,
        current_reference,
    )
    .unwrap();

    assert_close(step.drift(), 0.0, 2.0e-15);
    assert_close(
        step.current_error(),
        step.transport_factor() * step.neighbor_error(),
        3.0e-15,
    );
}

#[test]
fn constant_toeplitz_fixed_point_is_a_zero_drift_reference() {
    let frozen = FrozenToeplitzCavity::new(3.0, 1.0).unwrap();
    let q = frozen.cavity();
    let neighbor_cavity = q * 1.03;

    let step = CavityTransportStep::left(3.0, 1.0, neighbor_cavity, q, q).unwrap();
    assert_close(step.drift(), 0.0, 3.0e-15);
    assert_close(
        step.transport_factor(),
        1.0 / (neighbor_cavity * q),
        2.0e-15,
    );
    assert_close(step.current_error(), step.reconstructed_error(), 3.0e-15);
}

#[test]
fn transport_fails_closed_on_invalid_or_unrepresentable_inputs() {
    assert!(matches!(
        CavityTransportStep::left(f64::NAN, 1.0, 2.0, 2.0, 2.0),
        Err(CavityTransportError::NonFiniteInput { .. })
    ));
    assert!(matches!(
        CavityTransportStep::left(3.0, 1.0, 0.0, 2.0, 2.0),
        Err(CavityTransportError::NonPositiveNeighborCavity { .. })
    ));
    assert!(matches!(
        CavityTransportStep::left(3.0, 1.0, 2.0, -1.0, 2.0),
        Err(CavityTransportError::NonPositiveReference { .. })
    ));
    assert!(matches!(
        CavityTransportStep::left(1.0, 2.0, 1.0, 1.0, 1.0),
        Err(CavityTransportError::NonPositiveCurrentCavity { .. })
    ));
    assert!(matches!(
        CavityTransportStep::left(3.0, 1.0e308, 2.0, 2.0, 2.0),
        Err(CavityTransportError::NonFiniteDerivedQuantity)
    ));
}
