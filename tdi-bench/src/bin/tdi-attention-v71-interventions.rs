//! Task-label-preserving intervention fixtures for TDI-7.1.
//!
//! The task tokens and target are immutable under intervention. Only a declared
//! mechanistic activation site changes once at depth zero.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InterventionSite {
    EarlyToken,
    LateToken,
}

#[derive(Clone, Debug, PartialEq)]
struct MechanisticState {
    tokens: Vec<u16>,
    target: Vec<u16>,
    activations: Vec<f64>,
}

impl MechanisticState {
    fn new(tokens: Vec<u16>, target: Vec<u16>) -> Self {
        let activations = tokens.iter().map(|token| f64::from(*token) / 256.0).collect();
        Self {
            tokens,
            target,
            activations,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct SingleSiteIntervention {
    site: InterventionSite,
    amplitude: f64,
}

impl SingleSiteIntervention {
    fn index(self, len: usize) -> Option<usize> {
        match self.site {
            InterventionSite::EarlyToken => (len >= 2).then_some(1),
            InterventionSite::LateToken => (len >= 2).then_some(len - 2),
        }
    }

    fn apply(self, reference: &MechanisticState) -> Result<MechanisticState, InterventionError> {
        if !self.amplitude.is_finite() {
            return Err(InterventionError::NonFiniteAmplitude);
        }
        let index = self
            .index(reference.activations.len())
            .ok_or(InterventionError::StateTooShort)?;
        let mut perturbed = reference.clone();
        perturbed.activations[index] += self.amplitude;
        if !perturbed.activations[index].is_finite() {
            return Err(InterventionError::NonFiniteResult);
        }
        Ok(perturbed)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InterventionError {
    StateTooShort,
    NonFiniteAmplitude,
    NonFiniteResult,
}

fn advance(state: &MechanisticState) -> MechanisticState {
    let len = state.activations.len();
    let mut next = state.clone();
    for index in 0..len {
        let left = if index == 0 {
            state.activations[index]
        } else {
            state.activations[index - 1]
        };
        let center = state.activations[index];
        let right = if index + 1 == len {
            state.activations[index]
        } else {
            state.activations[index + 1]
        };
        next.activations[index] = 0.25 * left + 0.5 * center + 0.25 * right;
    }
    next
}

fn linf_distance(left: &MechanisticState, right: &MechanisticState) -> f64 {
    left.activations
        .iter()
        .zip(&right.activations)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0, f64::max)
}

fn main() {
    let reference = MechanisticState::new(vec![10, 20, 30, 40, 50, 60], vec![30]);
    let intervention = SingleSiteIntervention {
        site: InterventionSite::EarlyToken,
        amplitude: 0.25,
    };
    let perturbed = intervention.apply(&reference).expect("valid fixture");
    let reference_next = advance(&reference);
    let perturbed_next = advance(&perturbed);
    println!("TDI-7.1 intervention preflight: PASS");
    println!(
        "distance_after_one_step={:.12}",
        linf_distance(&reference_next, &perturbed_next)
    );
    println!("task target preserved={}", reference.target == perturbed.target);
    println!("TDI-7.2 final holdout: NOT ACCESSED");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> MechanisticState {
        MechanisticState::new(vec![11, 22, 33, 44, 55, 66], vec![33])
    }

    #[test]
    fn intervention_changes_exactly_one_activation() {
        let reference = fixture();
        let intervention = SingleSiteIntervention {
            site: InterventionSite::EarlyToken,
            amplitude: 0.5,
        };
        let perturbed = intervention.apply(&reference).unwrap();
        let changed: Vec<_> = reference
            .activations
            .iter()
            .zip(&perturbed.activations)
            .enumerate()
            .filter_map(|(index, (left, right))| (left != right).then_some(index))
            .collect();
        assert_eq!(changed, vec![1]);
    }

    #[test]
    fn intervention_never_changes_tokens_or_target() {
        for site in [InterventionSite::EarlyToken, InterventionSite::LateToken] {
            let reference = fixture();
            let perturbed = SingleSiteIntervention {
                site,
                amplitude: 0.125,
            }
            .apply(&reference)
            .unwrap();
            assert_eq!(perturbed.tokens, reference.tokens);
            assert_eq!(perturbed.target, reference.target);
        }
    }

    #[test]
    fn two_preregisterable_locations_are_distinct() {
        let len = fixture().activations.len();
        let early = SingleSiteIntervention {
            site: InterventionSite::EarlyToken,
            amplitude: 0.1,
        }
        .index(len)
        .unwrap();
        let late = SingleSiteIntervention {
            site: InterventionSite::LateToken,
            amplitude: 0.1,
        }
        .index(len)
        .unwrap();
        assert_ne!(early, late);
    }

    #[test]
    fn intervention_is_applied_once_then_shared_dynamics_advance() {
        let reference_0 = fixture();
        let intervention = SingleSiteIntervention {
            site: InterventionSite::LateToken,
            amplitude: 0.4,
        };
        let perturbed_0 = intervention.apply(&reference_0).unwrap();
        let reference_1 = advance(&reference_0);
        let perturbed_1 = advance(&perturbed_0);
        let reference_2 = advance(&reference_1);
        let perturbed_2 = advance(&perturbed_1);
        assert!(linf_distance(&reference_1, &perturbed_1) > 0.0);
        assert!(linf_distance(&reference_2, &perturbed_2) > 0.0);
        assert_eq!(reference_2.tokens, perturbed_2.tokens);
        assert_eq!(reference_2.target, perturbed_2.target);
    }

    #[test]
    fn identical_input_and_intervention_are_deterministic() {
        let reference = fixture();
        let intervention = SingleSiteIntervention {
            site: InterventionSite::EarlyToken,
            amplitude: 0.25,
        };
        assert_eq!(intervention.apply(&reference), intervention.apply(&reference));
    }

    #[test]
    fn invalid_amplitudes_fail_closed() {
        let reference = fixture();
        let intervention = SingleSiteIntervention {
            site: InterventionSite::EarlyToken,
            amplitude: f64::NAN,
        };
        assert_eq!(
            intervention.apply(&reference),
            Err(InterventionError::NonFiniteAmplitude)
        );
    }
}
