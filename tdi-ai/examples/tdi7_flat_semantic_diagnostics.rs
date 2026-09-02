//! TDI-7.5 bounded FLAT semantic discrimination diagnostics.
//!
//! This example validates the semantic discrimination diagnostics by comparing
//! two deterministic FLAT attention semantics. It does not access the final
//! holdout range and makes no GPU performance claims.

use tdi_ai::{
    BalancedTokenShift, FullStateObservable, Intervention, ReciprocalLInfRecovery,
    ReferenceDynamics, ToyAttentionError, ToyAttentionState, analyze_intervention_recovery,
    analyze_static_attention, extract_early_recovery_features,
};

const LONG_HORIZON: usize = 4;
const TARGET_DEPTH: usize = 5;

const _: () = assert!(TARGET_DEPTH > LONG_HORIZON);

/// A simple deterministic FLAT semantic: uniform averaging mixer.
struct AveragingMixer {
    size: usize,
}

impl AveragingMixer {
    fn new(size: usize) -> Self {
        Self { size }
    }
}

impl ReferenceDynamics for AveragingMixer {
    type State = ToyAttentionState;
    type Error = ToyAttentionError;

    fn advance(&self, state: &Self::State) -> Result<Self::State, Self::Error> {
        let values = state.values();
        if values.is_empty() {
            return Err(ToyAttentionError::EmptyState);
        }
        let sum: f64 = values.iter().sum();
        let avg = sum / self.size as f64;
        ToyAttentionState::new(vec![avg; self.size])
    }
}

/// A simple deterministic FLAT semantic: exponential decay mixer.
struct DecayMixer {
    size: usize,
    decay: f64,
}

impl DecayMixer {
    fn new(size: usize, decay: f64) -> Self {
        Self { size, decay }
    }
}

impl ReferenceDynamics for DecayMixer {
    type State = ToyAttentionState;
    type Error = ToyAttentionError;

    fn advance(&self, state: &Self::State) -> Result<Self::State, Self::Error> {
        let values = state.values();
        if values.is_empty() {
            return Err(ToyAttentionError::EmptyState);
        }
        let result: Vec<f64> = (0..self.size)
            .map(|i| {
                let (weighted_sum, weight_sum) =
                    values
                        .iter()
                        .enumerate()
                        .fold((0.0, 0.0), |(acc, w_sum), (j, val)| {
                            let distance = (i as i64 - j as i64).abs() as i32;
                            let weight = self.decay.powi(distance);
                            (acc + val * weight, w_sum + weight)
                        });
                if weight_sum > 0.0 {
                    weighted_sum / weight_sum
                } else {
                    0.0
                }
            })
            .collect();
        ToyAttentionState::new(result)
    }
}

/// FLAT semantic discrimination diagnostic.
#[derive(Clone, Debug, PartialEq)]
pub struct SemanticDiscriminationDiagnostic {
    pub target_depth: usize,
    pub long_horizon: usize,
    pub semantic_a_saturation: f64,
    pub semantic_b_saturation: f64,
    pub recovery_distance: f64,
    pub static_discrimination: f64,
    pub joint_discrimination: f64,
    pub insufficiency_indicator: bool,
}

/// Compute recovery saturation for a given semantic using generics.
fn compute_semantic_saturation<D>(
    mixer: &D,
    initial: &ToyAttentionState,
    intervention: &BalancedTokenShift,
) -> f64
where
    D: ReferenceDynamics<State = ToyAttentionState, Error = ToyAttentionError>,
    BalancedTokenShift: Intervention<ToyAttentionState, Error = ToyAttentionError>,
{
    let profile = analyze_intervention_recovery(
        mixer,
        intervention,
        &FullStateObservable,
        &ReciprocalLInfRecovery,
        initial,
        LONG_HORIZON,
    )
    .expect("deterministic recovery succeeds");

    let early =
        extract_early_recovery_features(&profile, TARGET_DEPTH).expect("early features exist");

    let overlaps = early.overlaps();
    let len = overlaps.len();
    if len == 0 {
        return 0.0;
    }
    let second_half = len / 2;
    if second_half == 0 {
        return overlaps[len - 1];
    }
    let sum: f64 = overlaps[second_half..].iter().sum();
    sum / (len - second_half) as f64
}

fn compute_semantic_discrimination(matrix: &[Vec<f64>]) -> SemanticDiscriminationDiagnostic {
    let initial = ToyAttentionState::zeros(3).expect("non-empty");
    let intervention = BalancedTokenShift::new(0, 2, 1.0).expect("valid shift");

    let mixer_a = AveragingMixer::new(3);
    let saturation_a = compute_semantic_saturation(&mixer_a, &initial, &intervention);

    let mixer_b = DecayMixer::new(3, 0.5);
    let saturation_b = compute_semantic_saturation(&mixer_b, &initial, &intervention);

    let recovery_distance = (saturation_a - saturation_b).abs();

    let static_diag = analyze_static_attention(matrix).expect("valid matrix");
    let static_concentration = static_diag.mean_row_l2_concentration();
    let static_discrimination = (static_concentration - 0.5).abs() * 2.0;

    let joint_discrimination = (static_discrimination + recovery_distance) / 2.0;
    let insufficiency_indicator = static_discrimination < 0.1 && recovery_distance > 0.01;

    SemanticDiscriminationDiagnostic {
        target_depth: TARGET_DEPTH,
        long_horizon: LONG_HORIZON,
        semantic_a_saturation: saturation_a,
        semantic_b_saturation: saturation_b,
        recovery_distance,
        static_discrimination,
        joint_discrimination,
        insufficiency_indicator,
    }
}

fn main() {
    let matrix = vec![
        vec![0.5, 0.5, 0.0],
        vec![0.25, 0.5, 0.25],
        vec![0.0, 0.5, 0.5],
    ];

    let disc = compute_semantic_discrimination(&matrix);

    println!("TDI-7.5 FLAT semantic discrimination diagnostics: PASS");
    println!("target_depth={}", disc.target_depth);
    println!("long_horizon={}", disc.long_horizon);
    println!("semantic_a_saturation={}", disc.semantic_a_saturation);
    println!("semantic_b_saturation={}", disc.semantic_b_saturation);
    println!("recovery_distance={}", disc.recovery_distance);
    println!("static_discrimination={}", disc.static_discrimination);
    println!("joint_discrimination={}", disc.joint_discrimination);
    println!("insufficiency_indicator={}", disc.insufficiency_indicator);
    println!("TDI-7.5 semantic discrimination: BOUNDED");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_discrimination_is_deterministic() {
        let matrix = vec![
            vec![0.5, 0.5, 0.0],
            vec![0.25, 0.5, 0.25],
            vec![0.0, 0.5, 0.5],
        ];
        let first = compute_semantic_discrimination(&matrix);
        let second = compute_semantic_discrimination(&matrix);
        assert_eq!(first.target_depth, second.target_depth);
        assert_eq!(first.long_horizon, second.long_horizon);
        assert_eq!(first.semantic_a_saturation, second.semantic_a_saturation);
        assert_eq!(first.semantic_b_saturation, second.semantic_b_saturation);
        assert_eq!(first.recovery_distance, second.recovery_distance);
        assert_eq!(first.static_discrimination, second.static_discrimination);
        assert_eq!(first.joint_discrimination, second.joint_discrimination);
        assert_eq!(
            first.insufficiency_indicator,
            second.insufficiency_indicator
        );
    }

    #[test]
    fn long_horizon_stays_strictly_before_target_depth() {
        assert!(LONG_HORIZON < TARGET_DEPTH);
        let matrix = vec![
            vec![0.5, 0.5, 0.0],
            vec![0.25, 0.5, 0.25],
            vec![0.0, 0.5, 0.5],
        ];
        let disc = compute_semantic_discrimination(&matrix);
        assert_eq!(disc.target_depth, TARGET_DEPTH);
        assert_eq!(disc.long_horizon, LONG_HORIZON);
    }

    #[test]
    fn two_semantics_produce_different_recovery() {
        let matrix = vec![
            vec![0.5, 0.5, 0.0],
            vec![0.25, 0.5, 0.25],
            vec![0.0, 0.5, 0.5],
        ];
        let disc = compute_semantic_discrimination(&matrix);
        assert!(
            disc.recovery_distance > 0.0,
            "expected different recovery profiles, got distance={}",
            disc.recovery_distance
        );
    }

    #[test]
    fn source_has_no_final_holdout_authorization() {
        let source = include_str!("tdi7_flat_semantic_diagnostics.rs");
        let confirmation = ["I_ACCEPT_THE_TDI7_", "HOLDOUT_FREEZE"].concat();
        let environment = ["TDI7_CONFIRM_FINAL_", "HOLDOUT"].concat();
        assert!(!source.contains(&confirmation));
        assert!(!source.contains(&environment));
    }

    #[test]
    fn semantic_diagnostics_do_not_reference_confirmatory_metrics() {
        let source = include_str!("tdi7_flat_semantic_diagnostics.rs");
        let confirmatory_metric = ["relative_MSE_", "reduction"].concat();
        assert!(!source.contains(&confirmatory_metric));
    }

    #[test]
    fn static_discrimination_is_bounded() {
        let matrix = vec![
            vec![0.5, 0.5, 0.0],
            vec![0.25, 0.5, 0.25],
            vec![0.0, 0.5, 0.5],
        ];
        let disc = compute_semantic_discrimination(&matrix);
        assert!(disc.static_discrimination >= 0.0);
        assert!(disc.static_discrimination <= 1.0);
    }
}
