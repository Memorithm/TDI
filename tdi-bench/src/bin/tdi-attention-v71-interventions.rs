//! Task-label-preserving intervention oracle for TDI-7.1.
//!
//! The mechanics now live in `tdi_bench::attention_v7` so follow-up stages can
//! reuse the exact bounded semantics. Task tokens and targets remain immutable.

use tdi_bench::attention_v7::{
    InterventionSite, MechanisticState, SingleSiteIntervention, advance, linf_distance,
};

fn main() {
    let reference = MechanisticState::new(vec![10, 20, 30, 40, 50, 60], vec![30]);
    for site in [InterventionSite::EarlyToken, InterventionSite::LateToken] {
        let intervention = SingleSiteIntervention::new(site, 0.25);
        let perturbed = intervention.apply(&reference).expect("valid fixture");
        let reference_next = advance(&reference);
        let perturbed_next = advance(&perturbed);
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

    fn fixture() -> MechanisticState {
        MechanisticState::new(vec![11, 22, 33, 44, 55, 66], vec![33])
    }

    #[test]
    fn intervention_changes_exactly_one_activation() {
        let reference = fixture();
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
        assert_eq!(changed, vec![1]);
    }

    #[test]
    fn intervention_never_changes_tokens_or_target() {
        for site in [InterventionSite::EarlyToken, InterventionSite::LateToken] {
            let reference = fixture();
            let perturbed = SingleSiteIntervention::new(site, 0.125)
                .apply(&reference)
                .unwrap();
            assert_eq!(perturbed.tokens(), reference.tokens());
            assert_eq!(perturbed.target(), reference.target());
        }
    }

    #[test]
    fn two_preregisterable_locations_are_distinct() {
        let len = fixture().activations().len();
        let early = SingleSiteIntervention::new(InterventionSite::EarlyToken, 0.1)
            .index(len)
            .unwrap();
        let late = SingleSiteIntervention::new(InterventionSite::LateToken, 0.1)
            .index(len)
            .unwrap();
        assert_ne!(early, late);
    }

    #[test]
    fn intervention_is_applied_once_then_shared_dynamics_advance() {
        let reference_0 = fixture();
        let perturbed_0 = SingleSiteIntervention::new(InterventionSite::LateToken, 0.4)
            .apply(&reference_0)
            .unwrap();
        let reference_1 = advance(&reference_0);
        let perturbed_1 = advance(&perturbed_0);
        let reference_2 = advance(&reference_1);
        let perturbed_2 = advance(&perturbed_1);
        assert!(linf_distance(&reference_1, &perturbed_1) > 0.0);
        assert!(linf_distance(&reference_2, &perturbed_2) > 0.0);
        assert_eq!(reference_2.tokens(), perturbed_2.tokens());
        assert_eq!(reference_2.target(), perturbed_2.target());
    }

    #[test]
    fn identical_input_and_intervention_are_deterministic() {
        let reference = fixture();
        let intervention = SingleSiteIntervention::new(InterventionSite::EarlyToken, 0.25);
        assert_eq!(
            intervention.apply(&reference),
            intervention.apply(&reference)
        );
    }

    #[test]
    fn invalid_amplitudes_fail_closed() {
        let reference = fixture();
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
