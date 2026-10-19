//! Synthetic Development launch for TDI-27 concept geometry.

use tdi_bench::concept_geometry_v27::{
    ConceptGeometryError, DEFAULT_TOLERANCE, DevelopmentResamplingPlan, DevelopmentSampleSizePoint,
    ProcrustesStabilityGrid, SubspacePersistenceGrid, bootstrap_direction_stability,
    bootstrap_innovation_energy_summary, bootstrap_projection_method_differentials,
    causal_novelty_gap, compare_innovation_energy_to_shuffled_null,
    cross_layer_direction_transport, decompose_symmetric_intervention_causal_contrasts,
    interaction_residual, matched_intervention_causal_contrasts,
    matched_intervention_dose_response_curve, matched_intervention_dose_states, mean_difference,
    orthogonal_procrustes_alignment, procrustes_anchor_permutation_control,
    procrustes_stability_grid, projection_method_differential, residualize,
    sample_size_sensitivity_grid, sequential_accepted_rank_summary, sequential_innovations,
    subspace_persistence_controls,
};

fn response(state: &[f64]) -> f64 {
    state[0] + state[1] + 0.5 * state[0] * state[1]
}

fn intervention_effect(base: &[f64], direction: &[f64], alpha: f64) -> f64 {
    let mut intervened = base.to_vec();
    for (value, delta) in intervened.iter_mut().zip(direction) {
        *value += alpha * delta;
    }
    response(&intervened) - response(base)
}

fn run() -> Result<(), ConceptGeometryError> {
    let control = vec![
        vec![0.2, -0.1, 0.0],
        vec![-0.2, 0.1, 0.0],
        vec![0.1, 0.2, -0.1],
        vec![-0.1, -0.2, 0.1],
    ];
    let positive = control
        .iter()
        .map(|row| vec![row[0] + 3.0, row[1] + 4.0, row[2]])
        .collect::<Vec<_>>();

    let contrast = mean_difference(&positive, &control)?;
    let known_basis = vec![vec![1.0, 0.0, 0.0]];
    let residual = residualize(&contrast, &known_basis, DEFAULT_TOLERANCE)?;
    let unit = residual
        .unit_direction()
        .ok_or(ConceptGeometryError::ZeroNorm)?;

    let base = [0.0, 0.0, 0.0];
    let target_effect = intervention_effect(&base, unit, 1.0);
    let control_effect = intervention_effect(&base, &[0.0, 0.0, 1.0], 1.0);
    let causal_gap = causal_novelty_gap(target_effect, &[control_effect])?;
    let intervention_doses = matched_intervention_dose_states(
        &base,
        unit,
        &[vec![0.0, 0.0, 1.0]],
        &[-1.0, 0.0, 1.0],
        DEFAULT_TOLERANCE,
        DEFAULT_TOLERANCE,
    )?;
    let intervention_curve = matched_intervention_dose_response_curve(
        &base,
        unit,
        &[vec![0.0, 0.0, 1.0]],
        &[-1.0, 0.0, 1.0],
        0,
        &[1],
        DEFAULT_TOLERANCE,
        DEFAULT_TOLERANCE,
        |state| Ok(vec![response(state), state[0] - state[1]]),
    )?;

    let intervention_contrasts = matched_intervention_causal_contrasts(&intervention_curve)?;
    let signed_dose_decomposition =
        decompose_symmetric_intervention_causal_contrasts(&intervention_contrasts)?;

    let effect_a = intervention_effect(&base, &[1.0, 0.0, 0.0], 1.0);
    let effect_b = intervention_effect(&base, &[0.0, 1.0, 0.0], 1.0);
    let effect_ab = intervention_effect(&base, &[1.0, 1.0, 0.0], 1.0);
    let interaction = interaction_residual(effect_a, effect_b, effect_ab)?;

    let sequence = sequential_innovations(
        &[
            vec![1.0, 0.0, 0.0],
            vec![1.0, 1.0, 0.0],
            vec![2.0, 2.0, 1.0],
        ],
        DEFAULT_TOLERANCE,
    )?;
    let accepted_rank = sequence.iter().filter(|step| step.accepted()).count();

    let resampling_positive = vec![vec![3.0, 4.0, 0.0]; 4];
    let resampling_control = vec![vec![0.0, 0.0, 0.0]; 4];
    let resampling_plan = DevelopmentResamplingPlan::new(8, 0x2701_1501)?;
    let bootstrap_summary = bootstrap_innovation_energy_summary(
        &resampling_positive,
        &resampling_control,
        &known_basis,
        DEFAULT_TOLERANCE,
        resampling_plan,
        1,
        6,
    )?;
    let direction_stability = bootstrap_direction_stability(
        &resampling_positive,
        &resampling_control,
        &known_basis,
        DEFAULT_TOLERANCE,
        resampling_plan,
    )?;
    let differential_basis = vec![vec![1.0, 1.0, 0.0], vec![1.0, 1.0 + 1.0e-10, 0.0]];
    let projection_vector = [3.0, 4.0, 2.0];
    let projection_differential = projection_method_differential(
        &projection_vector,
        &differential_basis,
        1.0e-12,
        1.0e-8,
        1.0e-8,
    )?;
    let projection_positive = vec![projection_vector.to_vec(); 4];
    let projection_control = vec![vec![0.0, 0.0, 0.0]; 4];
    let bootstrap_projection_differentials = bootstrap_projection_method_differentials(
        &projection_positive,
        &projection_control,
        &differential_basis,
        1.0e-12,
        1.0e-8,
        1.0e-8,
        resampling_plan,
    )?;
    let null_summary = compare_innovation_energy_to_shuffled_null(
        &resampling_positive,
        &resampling_control,
        &known_basis,
        DEFAULT_TOLERANCE,
        resampling_plan,
    )?;
    let sample_size_grid = [
        DevelopmentSampleSizePoint::new(2, 2)?,
        DevelopmentSampleSizePoint::new(3, 3)?,
    ];
    let sample_size_cells = sample_size_sensitivity_grid(
        &resampling_positive,
        &resampling_control,
        &known_basis,
        DEFAULT_TOLERANCE,
        resampling_plan,
        &sample_size_grid,
    )?;
    let rank_replicates = vec![
        vec![vec![1.0, 0.0, 0.0], vec![2.0, 0.0, 0.0]],
        vec![vec![1.0, 0.0, 0.0], vec![1.0, 1.0, 0.0]],
        vec![
            vec![1.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0],
            vec![0.0, 0.0, 1.0],
        ],
    ];
    let rank_summary = sequential_accepted_rank_summary(
        &[
            vec![1.0, 0.0, 0.0],
            vec![1.0, 1.0, 0.0],
            vec![1.0, 1.0, 1.0],
        ],
        &rank_replicates,
        DEFAULT_TOLERANCE,
        0,
        2,
    )?;

    // TDI-27.4 first slice: Procrustes alignment between two synthetic
    // layers whose paired anchors differ by a known +90-degree rotation.
    let layer_source = vec![
        vec![1.0, 0.0, 0.0],
        vec![0.0, 1.0, 0.0],
        vec![0.0, 0.0, 1.0],
        vec![1.0, 1.0, 0.5],
    ];
    let layer_target: Vec<Vec<f64>> = layer_source
        .iter()
        .map(|row| vec![-row[1], row[0], row[2]])
        .collect();
    let procrustes =
        orthogonal_procrustes_alignment(&layer_source, &layer_target, DEFAULT_TOLERANCE)?;
    let procrustes_transport = cross_layer_direction_transport(
        &[1.0, 0.0, 0.0],
        &[0.0, 1.0, 0.0],
        &procrustes,
        DEFAULT_TOLERANCE,
    )?;

    // TDI-27.4 slice 2: shuffled-anchor negative control (descriptive only).
    let permutation_source = vec![
        vec![1.0, 0.0, 0.0],
        vec![0.0, 1.0, 0.0],
        vec![0.0, 0.0, 1.0],
        vec![1.0, 1.0, 0.5],
        vec![-0.5, 2.0, 1.0],
        vec![0.25, -1.0, 2.0],
    ];
    let permutation_target: Vec<Vec<f64>> = permutation_source
        .iter()
        .map(|row| vec![-row[1], row[0], row[2]])
        .collect();
    let permutation_control = procrustes_anchor_permutation_control(
        &permutation_source,
        &permutation_target,
        &[1.0, 0.0, 0.0],
        &[0.0, 1.0, 0.0],
        DEFAULT_TOLERANCE,
        27_402,
        8,
    )?;

    // TDI-27.4 slice 3: Q stability across synthetic anchor sets. The grid
    // is declared for this descriptive slice, not a frozen threshold.
    let stability_cells = procrustes_stability_grid(
        &[
            vec![0.0, 1.0, 0.0],
            vec![-1.0, 0.0, 0.0],
            vec![0.0, 0.0, 1.0],
        ],
        &ProcrustesStabilityGrid {
            anchor_counts: vec![8, 32],
            noise_scales: vec![0.01, 0.1],
            replicates: 8,
            seed: 27_403,
        },
        DEFAULT_TOLERANCE,
    )?;

    // TDI-27.4 slice 4: subspace persistence via principal angles with
    // known-truth controls. Grid declared for this descriptive slice only.
    let subspace_cells = subspace_persistence_controls(
        &SubspacePersistenceGrid {
            width: 6,
            subspace_dims: vec![1, 2, 4],
            anchor_count: 12,
            random_draws: 8,
            seed: 27_404,
        },
        DEFAULT_TOLERANCE,
    )?;

    println!("schema=tdi27-concept-geometry-development/v9");
    println!("scope=synthetic_development_only");
    println!(
        "mean_contrast={:.12},{:.12},{:.12}",
        contrast[0], contrast[1], contrast[2]
    );
    println!("known_basis_rank={}", residual.basis_rank());
    println!(
        "innovation_energy_ratio={:.12}",
        residual.innovation_energy_ratio()
    );
    println!(
        "residual_unit={:.12},{:.12},{:.12}",
        unit[0], unit[1], unit[2]
    );
    println!("target_effect={target_effect:.12}");
    println!("matched_control_effect={control_effect:.12}");
    println!("causal_novelty_gap={causal_gap:.12}");
    println!("intervention_dose_count={}", intervention_doses.len());
    println!(
        "intervention_control_count={}",
        intervention_doses[0].matched_control_states().len()
    );
    println!("intervention_alpha_grid=-1.000000000000,0.000000000000,1.000000000000");
    println!(
        "intervention_curve_target_outcome_index={}",
        intervention_curve.target_outcome_index()
    );
    println!("intervention_curve_non_target_outcome_indices=1");
    println!(
        "intervention_curve_target_effects={:.12},{:.12},{:.12}",
        intervention_curve.points()[0].target_state_effects()[0],
        intervention_curve.points()[1].target_state_effects()[0],
        intervention_curve.points()[2].target_state_effects()[0]
    );
    println!(
        "intervention_curve_matched_control_effects={:.12},{:.12},{:.12}",
        intervention_curve.points()[0].matched_control_effects()[0][0],
        intervention_curve.points()[1].matched_control_effects()[0][0],
        intervention_curve.points()[2].matched_control_effects()[0][0]
    );
    println!(
        "intervention_contrast_target_gaps={:.12},{:.12},{:.12}",
        intervention_contrasts.points()[0].target_outcome_gap(),
        intervention_contrasts.points()[1].target_outcome_gap(),
        intervention_contrasts.points()[2].target_outcome_gap()
    );
    println!(
        "intervention_contrast_non_target_gaps={:.12},{:.12},{:.12}",
        intervention_contrasts.points()[0].non_target_outcome_gaps()[0],
        intervention_contrasts.points()[1].non_target_outcome_gaps()[0],
        intervention_contrasts.points()[2].non_target_outcome_gaps()[0]
    );
    println!(
        "intervention_symmetric_pair_count={}",
        signed_dose_decomposition.pairs().len()
    );
    println!(
        "intervention_symmetric_pair_alpha={:.12}",
        signed_dose_decomposition.pairs()[0].alpha_magnitude()
    );
    println!(
        "intervention_symmetric_target_odd_even={:.12},{:.12}",
        signed_dose_decomposition.pairs()[0].target_odd_component(),
        signed_dose_decomposition.pairs()[0].target_even_component()
    );
    println!(
        "intervention_symmetric_non_target_odd_even={:.12},{:.12}",
        signed_dose_decomposition.pairs()[0].non_target_odd_components()[0],
        signed_dose_decomposition.pairs()[0].non_target_even_components()[0]
    );
    println!(
        "intervention_zero_point_present={}",
        signed_dose_decomposition.zero_point().is_some()
    );
    println!("intervention_zero_identity_enforced=true");
    println!(
        "intervention_unpaired_point_count={}",
        signed_dose_decomposition.unpaired_points().len()
    );
    println!("orthogonal_interaction_residual={interaction:.12}");
    println!(
        "sequential_innovation_ratios={:.12},{:.12},{:.12}",
        sequence[0].innovation_energy_ratio(),
        sequence[1].innovation_energy_ratio(),
        sequence[2].innovation_energy_ratio()
    );
    println!("sequential_accepted_rank={accepted_rank}");
    println!("resampling_replicates={}", resampling_plan.replicates());
    println!(
        "bootstrap_innovation_full={:.12}",
        bootstrap_summary.full_sample_value()
    );
    println!(
        "bootstrap_innovation_order_bounds={:.12},{:.12}",
        bootstrap_summary.order_interval().lower_value(),
        bootstrap_summary.order_interval().upper_value()
    );
    let direction_stability_values = direction_stability
        .replicate_cosines()
        .iter()
        .map(|value| match value {
            Some(value) => format!("{value:.12}"),
            None => "undefined".to_string(),
        })
        .collect::<Vec<_>>()
        .join(",");
    println!("bootstrap_direction_stability={direction_stability_values}");
    println!(
        "bootstrap_direction_stability_undefined={}",
        direction_stability
            .replicate_cosines()
            .iter()
            .filter(|value| value.is_none())
            .count()
    );
    println!(
        "projection_method_max_abs_residual_difference={:.17e}",
        projection_differential.max_abs_residual_difference()
    );
    match projection_differential.max_abs_unit_direction_difference() {
        Some(difference) => {
            println!("projection_method_max_abs_unit_direction_difference={difference:.17e}")
        }
        None => println!("projection_method_max_abs_unit_direction_difference=undefined"),
    }
    println!(
        "projection_method_bootstrap_replicates={}",
        bootstrap_projection_differentials.len()
    );
    println!(
        "projection_method_bootstrap_max_abs_residual_difference={:.17e}",
        bootstrap_projection_differentials
            .iter()
            .map(|report| report.differential().max_abs_residual_difference())
            .fold(0.0, f64::max)
    );
    let bootstrap_direction_difference = bootstrap_projection_differentials
        .iter()
        .filter_map(|report| report.differential().max_abs_unit_direction_difference())
        .fold(None::<f64>, |maximum, difference| {
            Some(maximum.map_or(difference, |current| current.max(difference)))
        });
    match bootstrap_direction_difference {
        Some(difference) => println!(
            "projection_method_bootstrap_max_abs_unit_direction_difference={difference:.17e}"
        ),
        None => println!("projection_method_bootstrap_max_abs_unit_direction_difference=undefined"),
    }
    println!("null_total_replicates={}", null_summary.total_replicates());
    println!(
        "null_defined_replicates={}",
        null_summary.defined_null_values().len()
    );
    println!(
        "null_undefined_replicates={}",
        null_summary.undefined_replicates()
    );
    println!(
        "null_greater_or_equal_count={}",
        null_summary.greater_or_equal_count()
    );
    println!("sample_size_cells={}", sample_size_cells.len());
    println!("sequential_rank_full={}", rank_summary.full_sample_rank());
    println!(
        "sequential_rank_order_bounds={},{}",
        rank_summary.lower_value(),
        rank_summary.upper_value()
    );
    println!(
        "procrustes_singular_values={:.12},{:.12},{:.12}",
        procrustes.singular_values()[0],
        procrustes.singular_values()[1],
        procrustes.singular_values()[2]
    );
    println!(
        "procrustes_orthogonality_defect={:.17e}",
        procrustes.orthogonality_defect()
    );
    println!(
        "procrustes_relative_alignment_residual={:.17e}",
        procrustes.relative_alignment_residual()
    );
    println!(
        "procrustes_transported_cosine={:.12}",
        procrustes_transport.aligned_cosine()
    );
    println!(
        "procrustes_raw_cosine={:.12}",
        procrustes_transport.raw_cosine()
    );
    println!(
        "procrustes_permutation_true_residual={:.17e}",
        permutation_control
            .true_pairing()
            .relative_alignment_residual
    );
    println!(
        "procrustes_permutation_true_cosine={:.12}",
        permutation_control.true_pairing().transported_cosine
    );
    println!("procrustes_permutation_seed={}", permutation_control.seed());
    for draw in permutation_control.draws() {
        match draw.outcome() {
            Ok(outcome) => println!(
                "procrustes_permutation_draw={},perm={:?},fixed={},residual={:.12},cosine={:.12}",
                draw.draw_index(),
                draw.permutation(),
                draw.fixed_point_count(),
                outcome.relative_alignment_residual,
                outcome.transported_cosine
            ),
            Err(error) => println!(
                "procrustes_permutation_draw={},perm={:?},fixed={},failed={error:?}",
                draw.draw_index(),
                draw.permutation(),
                draw.fixed_point_count()
            ),
        }
    }
    for cell in &stability_cells {
        let failed = cell.replicates().iter().filter(|r| r.is_err()).count();
        match (
            cell.distance_to_truth(),
            cell.pairwise_frobenius_distance(),
            cell.max_axis_angle_to_truth(),
        ) {
            (Some(distance), Some(pairwise), Some(angle)) => println!(
                "procrustes_stability_cell=anchors:{},noise:{},replicates:{},failed:{},dist_truth_min_mean_max={:.6e},{:.6e},{:.6e},pairwise_min_mean_max={:.6e},{:.6e},{:.6e},max_axis_angle_rad_min_mean_max={:.6e},{:.6e},{:.6e}",
                cell.anchor_count(),
                cell.noise_scale(),
                cell.replicates().len(),
                failed,
                distance.min,
                distance.mean,
                distance.max,
                pairwise.min,
                pairwise.mean,
                pairwise.max,
                angle.min,
                angle.mean,
                angle.max
            ),
            _ => println!(
                "procrustes_stability_cell=anchors:{},noise:{},replicates:{},failed:{},spread=undefined",
                cell.anchor_count(),
                cell.noise_scale(),
                cell.replicates().len(),
                failed
            ),
        }
    }
    for cell in &subspace_cells {
        println!(
            "subspace_persistence_dim={},rotation_aligned_angles_rad={:?},rotation_raw_angles_rad={:?}",
            cell.subspace_dim(),
            cell.rotation_aligned_angles(),
            cell.rotation_raw_angles()
        );
        for (draw, angles) in cell.random_aligned_angles().iter().enumerate() {
            println!(
                "subspace_persistence_random=dim:{},draw:{draw},angles_rad={angles:?}",
                cell.subspace_dim()
            );
        }
    }
    println!("statistical_decision_pinned=false");
    println!("confirmatory_result=false");
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("tdi27 development runner failed: {error:?}");
        std::process::exit(1);
    }
}
