//! Synthetic Development launch for TDI-27 concept geometry.

use tdi_bench::concept_geometry_v27::{
    causal_novelty_gap, interaction_residual, mean_difference, residualize,
    sequential_innovations, ConceptGeometryError, DEFAULT_TOLERANCE,
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

    println!("schema=tdi27-concept-geometry-development/v1");
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
    println!("orthogonal_interaction_residual={interaction:.12}");
    println!(
        "sequential_innovation_ratios={:.12},{:.12},{:.12}",
        sequence[0].innovation_energy_ratio(),
        sequence[1].innovation_energy_ratio(),
        sequence[2].innovation_energy_ratio()
    );
    println!("sequential_accepted_rank={accepted_rank}");
    println!("confirmatory_result=false");
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("tdi27 development runner failed: {error:?}");
        std::process::exit(1);
    }
}
