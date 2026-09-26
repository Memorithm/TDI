//! Synthetic Development launch for TDI-27 concept geometry.

use tdi_bench::concept_geometry_v27::{
    ConceptGeometryError, DEFAULT_TOLERANCE, DevelopmentResamplingPlan, DevelopmentSampleSizePoint,
    bootstrap_direction_stability, bootstrap_innovation_energy_summary,
    bootstrap_projection_method_differentials, causal_novelty_gap,
    compare_innovation_energy_to_shuffled_null, interaction_residual,
    matched_intervention_dose_states, mean_difference, projection_method_differential, residualize,
    sample_size_sensitivity_grid, sequential_accepted_rank_summary, sequential_innovations,
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

    println!("schema=tdi27-concept-geometry-development/v2");
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
