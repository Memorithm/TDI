use tdi_operator::{CavityChainError, CavityTransportChain, CavityTransportStep};

fn assert_close(left: f64, right: f64, tolerance: f64) {
    let scale = 1.0_f64.max(left.abs()).max(right.abs());
    assert!(
        (left - right).abs() <= tolerance * scale,
        "left={left:.17e}, right={right:.17e}, |delta|={:.3e}",
        (left - right).abs()
    );
}

#[test]
fn left_chain_matches_exact_affine_unrolling_numerically() {
    let first = CavityTransportStep::left(4.0, 0.75, 3.2, 3.0, 3.1).unwrap();
    let second = CavityTransportStep::left(
        4.2,
        0.8,
        first.current_cavity(),
        first.current_reference(),
        3.3,
    )
    .unwrap();
    let third = CavityTransportStep::left(
        4.4,
        0.7,
        second.current_cavity(),
        second.current_reference(),
        3.5,
    )
    .unwrap();

    let chain = CavityTransportChain::from_steps(&[first, second, third]).unwrap();
    let expected_factor =
        third.transport_factor() * second.transport_factor() * first.transport_factor();
    let expected_drift = third.drift()
        + third.transport_factor() * second.drift()
        + third.transport_factor() * second.transport_factor() * first.drift();

    assert_eq!(chain.steps(), 3);
    assert_eq!(chain.initial_error(), first.neighbor_error());
    assert_eq!(chain.observed_final_error(), third.current_error());
    assert_close(
        chain.cumulative_transport_factor(),
        expected_factor,
        3.0e-15,
    );
    assert_close(chain.accumulated_drift(), expected_drift, 4.0e-15);
    assert_close(
        chain.reconstructed_final_error(),
        third.current_error(),
        8.0e-15,
    );
}

#[test]
fn right_chain_uses_the_same_exact_composition_in_propagation_order() {
    let first = CavityTransportStep::right(5.0, -0.6, 4.0, 3.8, 3.9).unwrap();
    let second = CavityTransportStep::right(
        4.8,
        0.5,
        first.current_cavity(),
        first.current_reference(),
        3.7,
    )
    .unwrap();

    let chain = CavityTransportChain::from_steps(&[first, second]).unwrap();

    assert_close(
        chain.cumulative_transport_factor(),
        second.transport_factor() * first.transport_factor(),
        2.0e-15,
    );
    assert_close(
        chain.accumulated_drift(),
        second.transport_factor() * first.drift() + second.drift(),
        3.0e-15,
    );
    assert_close(
        chain.reconstructed_final_error(),
        second.current_error(),
        6.0e-15,
    );
}

#[test]
fn single_step_chain_reduces_to_tdi10_2_identity() {
    let step = CavityTransportStep::left(3.5, 0.9, 3.0, 2.8, 2.9).unwrap();
    let chain = CavityTransportChain::from_steps(&[step]).unwrap();

    assert_eq!(chain.steps(), 1);
    assert_eq!(chain.cumulative_transport_factor(), step.transport_factor());
    assert_eq!(chain.accumulated_drift(), step.drift());
    assert_close(
        chain.reconstructed_final_error(),
        step.reconstructed_error(),
        2.0e-15,
    );
}

#[test]
fn empty_chain_fails_closed() {
    assert_eq!(
        CavityTransportChain::from_steps(&[]),
        Err(CavityChainError::EmptyChain)
    );
}

#[test]
fn discontinuous_chain_metadata_is_rejected() {
    let first = CavityTransportStep::left(4.0, 0.5, 3.0, 2.9, 3.1).unwrap();
    let wrong_cavity = CavityTransportStep::left(
        4.2,
        0.6,
        first.current_cavity() + 0.25,
        first.current_reference(),
        3.2,
    )
    .unwrap();
    assert!(matches!(
        CavityTransportChain::from_steps(&[first, wrong_cavity]),
        Err(CavityChainError::DiscontinuousCavity { index: 1, .. })
    ));

    let wrong_reference = CavityTransportStep::left(
        4.2,
        0.6,
        first.current_cavity(),
        first.current_reference() + 0.25,
        3.2,
    )
    .unwrap();
    assert!(matches!(
        CavityTransportChain::from_steps(&[first, wrong_reference]),
        Err(CavityChainError::DiscontinuousReference { index: 1, .. })
    ));
}

#[test]
fn finite_steps_fail_closed_when_chain_accumulation_overflows() {
    let tiny_reference = 1.0e-200;
    let first = CavityTransportStep::left(2.0, 1.0, 1.0, tiny_reference, tiny_reference).unwrap();
    let second = CavityTransportStep::left(
        2.0,
        1.0,
        first.current_cavity(),
        first.current_reference(),
        tiny_reference,
    )
    .unwrap();

    assert!(first.transport_factor().is_finite());
    assert!(second.transport_factor().is_finite());
    assert!(matches!(
        CavityTransportChain::from_steps(&[first, second]),
        Err(CavityChainError::NonFiniteAccumulation { index: 1 })
    ));
}