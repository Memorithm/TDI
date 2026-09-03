//! Task-label-preserving balanced intervention oracle for TDI-7.1.
//!
//! The mechanics live in `tdi_bench::attention_v7` so follow-up stages reuse
//! the same frozen non-holdout semantics. Each intervention adds amplitude to
//! the declared site and subtracts the same amplitude from its compensating
//! boundary coordinate before both trajectories advance through the same
//! retrieval-distance-conditioned local mixer.

use tdi_bench::attention_v7::{
    DeterministicLocalMixer, InterventionSite, MechanisticState, SingleSiteIntervention, TaskKind,
    generate_task, linf_distance,
};

fn main() {
    let task = generate_task(TaskKind::AssociativeRecall, 7_100_000_000);
    let reference = MechanisticState::from_task(&task);
    let mixer = DeterministicLocalMixer::from_task(&task);
    for site in [InterventionSite::EarlyToken, InterventionSite::LateToken] {
        let intervention = SingleSiteIntervention::new(site, 0.25);
        let perturbed = intervention.apply(&reference).expect("valid fixture");
        let reference_next = mixer.advance(&reference).expect("reference advance");
        let perturbed_next = mixer.advance(&perturbed).expect("perturbed advance");
        println!(
            "site={site:?} distance_after_one_step={:.12}",
            linf_distance(&reference_next, &perturbed_next)
        );
        assert_eq!(reference.tokens(), perturbed.tokens());
        assert_eq!(reference.target(), perturbed.target());
    }
    println!("TDI-7.1 intervention preflight: PASS");
    println!("TDI-7.2 final holdout: NOT ACCESSED");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tdi_bench::attention_v7::InterventionError;

    fn fixture() -> (MechanisticState, DeterministicLocalMixer) {
        let task = generate_task(TaskKind::AssociativeRecall, 7_100_000_001);
        (
            MechanisticState::from_task(&task),
            DeterministicLocalMixer::from_task(&task),
        )
    }

    #[test]
    fn intervention_changes_exactly_two_balancing_coordinates() {
        let (reference, _) = fixture();
        let perturbed = SingleSiteIntervention::new(InterventionSite::EarlyToken, 0.5)
            .apply(&reference)
            .unwrap();
        let changed: Vec<_> = reference
            .activations()
            .iter()
            .zip(perturbed.activations())
            .enumerate()
            .filter_map(|(index, (left, right))| (left != right).then_some(index))
            .collect();
        assert_eq!(changed, vec![0, 1]);
        let before: f64 = reference.activations().iter().sum();
        let after: f64 = perturbed.activations().iter().sum();
        assert!((before - after).abs() <= 1.0e-12);
    }

    #[test]
    fn intervention_never_changes_tokens_or_target() {
        for site in [InterventionSite::EarlyToken, InterventionSite::LateToken] {
            let (reference, _) = fixture();
            let perturbed = SingleSiteIntervention::new(site, 0.125)
                .apply(&reference)
                .unwrap();
            assert_eq!(perturbed.tokens(), reference.tokens());
            assert_eq!(perturbed.target(), reference.target());
        }
    }

    #[test]
    fn two_preregisterable_locations_are_distinct_and_disjoint() {
        let (reference, _) = fixture();
        let len = reference.activations().len();
        let early = SingleSiteIntervention::new(InterventionSite::EarlyToken, 0.1)
            .indices(len)
            .unwrap();
        let late = SingleSiteIntervention::new(InterventionSite::LateToken, 0.1)
            .indices(len)
            .unwrap();
        assert_ne!(early, late);
        assert!(![early.0, early.1].contains(&late.0));
        assert!(![early.0, early.1].contains(&late.1));
    }

    #[test]
    fn intervention_is_applied_once_then_shared_dynamics_advance() {
        let (reference_0, mixer) = fixture();
        let perturbed_0 = SingleSiteIntervention::new(InterventionSite::LateToken, 0.4)
            .apply(&reference_0)
            .unwrap();
        let reference_1 = mixer.advance(&reference_0).unwrap();
        let perturbed_1 = mixer.advance(&perturbed_0).unwrap();
        let reference_2 = mixer.advance(&reference_1).unwrap();
        let perturbed_2 = mixer.advance(&perturbed_1).unwrap();
        assert!(linf_distance(&reference_1, &perturbed_1) > 0.0);
        assert!(linf_distance(&reference_2, &perturbed_2) > 0.0);
        assert_eq!(reference_2.tokens(), perturbed_2.tokens());
        assert_eq!(reference_2.target(), perturbed_2.target());
    }

    #[test]
    fn identical_input_and_intervention_are_deterministic() {
        let (reference, _) = fixture();
        let intervention = SingleSiteIntervention::new(InterventionSite::EarlyToken, 0.25);
        assert_eq!(
            intervention.apply(&reference),
            intervention.apply(&reference)
        );
    }

    #[test]
    fn invalid_amplitudes_fail_closed() {
        let (reference, _) = fixture();
        let intervention = SingleSiteIntervention::new(InterventionSite::EarlyToken, f64::NAN);
        assert_eq!(
            intervention.apply(&reference),
            Err(InterventionError::NonFiniteAmplitude)
        );
    }

    #[test]
    fn binary_source_has_no_final_holdout_authorization_secret() {
        let source = include_str!("tdi-attention-v71-interventions.rs");
        let confirmation = ["I_ACCEPT_THE_TDI7_", "HOLDOUT_FREEZE"].concat();
        let environment = ["TDI7_CONFIRM_FINAL_", "HOLDOUT"].concat();
        assert!(!source.contains(&confirmation));
        assert!(!source.contains(&environment));
    }
}
