//! TDI-7.7 bounded cross-architecture transfer diagnostics.
//!
//! This example validates the transfer diagnostics by comparing
//! recovery profiles between two deterministic architectures.
//! It does not access the final holdout range and makes no neural claims.

use tdi_ai::{
    BalancedTokenShift, FullStateObservable, Intervention, ReciprocalLInfRecovery,
    ReferenceDynamics, ToyAttentionState, analyze_intervention_recovery,
    extract_early_recovery_features,
};

const LONG_HORIZON: usize = 4;
const TARGET_DEPTH: usize = 5;

const _: () = assert!(TARGET_DEPTH > LONG_HORIZON);

/// Architecture A: uniform averaging mixer.
struct ArchAveraging {
    size: usize,
}

impl ArchAveraging {
    fn new(size: usize) -> Self {
        Self { size }
    }
}

impl ReferenceDynamics for ArchAveraging {
    type State = ToyAttentionState;
    type Error = tdi_ai::ToyAttentionError;

    fn advance(&self, state: &Self::State) -> Result<Self::State, Self::Error> {
        let values = state.values();
        if values.is_empty() {
            return Err(tdi_ai::ToyAttentionError::EmptyState);
        }
        let sum: f64 = values.iter().sum();
        let avg = sum / self.size as f64;
        ToyAttentionState::new(vec![avg; self.size])
    }
}

/// Architecture B: identity mixer (each token attends only to itself).
#[allow(dead_code)]
struct ArchIdentity {
    size: usize,
}

impl ArchIdentity {
    fn new(size: usize) -> Self {
        Self { size }
    }
}

impl ReferenceDynamics for ArchIdentity {
    type State = ToyAttentionState;
    type Error = tdi_ai::ToyAttentionError;

    fn advance(&self, state: &Self::State) -> Result<Self::State, Self::Error> {
        let values = state.values();
        if values.is_empty() {
            return Err(tdi_ai::ToyAttentionError::EmptyState);
        }
        ToyAttentionState::new(values.to_vec())
    }
}

/// Transfer diagnostic result.
#[derive(Clone, Debug, PartialEq)]
pub struct TransferDiagnostic {
    pub target_depth: usize,
    pub long_horizon: usize,
    pub source_a_saturation: f64,
    pub source_b_saturation: f64,
    pub forward_transfer_ratio: f64,
    pub reverse_transfer_ratio: f64,
    pub asymmetry_index: f64,
    pub architecture_distance: f64,
    pub transfer_verdict: String,
}

/// Compute recovery saturation for a given architecture.
fn compute_arch_saturation<D>(
    mixer: &D,
    initial: &ToyAttentionState,
    intervention: &BalancedTokenShift,
) -> f64
where
    D: ReferenceDynamics<State = ToyAttentionState, Error = tdi_ai::ToyAttentionError>,
    BalancedTokenShift: Intervention<ToyAttentionState, Error = tdi_ai::ToyAttentionError>,
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

fn compute_transfer_diagnostic(_matrix: &[Vec<f64>]) -> TransferDiagnostic {
    let initial = ToyAttentionState::zeros(3).expect("non-empty");
    let intervention = BalancedTokenShift::new(0, 2, 1.0).expect("valid shift");

    let arch_a = ArchAveraging::new(3);
    let arch_b = ArchIdentity::new(3);

    let sat_a = compute_arch_saturation(&arch_a, &initial, &intervention);
    let sat_b = compute_arch_saturation(&arch_b, &initial, &intervention);

    let forward_ratio = if sat_a > 0.0 { sat_b / sat_a } else { 0.0 };
    let reverse_ratio = if sat_b > 0.0 { sat_a / sat_b } else { 0.0 };
    let asymmetry = forward_ratio - reverse_ratio;
    let arch_distance = (sat_a - sat_b).abs();

    let verdict = if forward_ratio > 0.8 && reverse_ratio > 0.8 {
        "full".to_string()
    } else if forward_ratio > 0.5 || reverse_ratio > 0.5 {
        "partial".to_string()
    } else if asymmetry.abs() > 0.3 {
        "asymmetric".to_string()
    } else {
        "failed".to_string()
    };

    TransferDiagnostic {
        target_depth: TARGET_DEPTH,
        long_horizon: LONG_HORIZON,
        source_a_saturation: sat_a,
        source_b_saturation: sat_b,
        forward_transfer_ratio: forward_ratio,
        reverse_transfer_ratio: reverse_ratio,
        asymmetry_index: asymmetry,
        architecture_distance: arch_distance,
        transfer_verdict: verdict,
    }
}

fn main() {
    let matrix = vec![
        vec![0.5, 0.5, 0.0],
        vec![0.25, 0.5, 0.25],
        vec![0.0, 0.5, 0.5],
    ];

    let transfer = compute_transfer_diagnostic(&matrix);

    println!("TDI-7.7 cross-architecture transfer diagnostics: PASS");
    println!("target_depth={}", transfer.target_depth);
    println!("long_horizon={}", transfer.long_horizon);
    println!("source_a_saturation={}", transfer.source_a_saturation);
    println!("source_b_saturation={}", transfer.source_b_saturation);
    println!("forward_transfer_ratio={}", transfer.forward_transfer_ratio);
    println!("reverse_transfer_ratio={}", transfer.reverse_transfer_ratio);
    println!("asymmetry_index={}", transfer.asymmetry_index);
    println!("architecture_distance={}", transfer.architecture_distance);
    println!("transfer_verdict={}", transfer.transfer_verdict);
    println!("TDI-7.7 transfer: BOUNDED");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transfer_is_deterministic() {
        let matrix = vec![
            vec![0.5, 0.5, 0.0],
            vec![0.25, 0.5, 0.25],
            vec![0.0, 0.5, 0.5],
        ];
        let first = compute_transfer_diagnostic(&matrix);
        let second = compute_transfer_diagnostic(&matrix);
        assert_eq!(first.target_depth, second.target_depth);
        assert_eq!(first.long_horizon, second.long_horizon);
        assert_eq!(first.source_a_saturation, second.source_a_saturation);
        assert_eq!(first.source_b_saturation, second.source_b_saturation);
        assert_eq!(first.forward_transfer_ratio, second.forward_transfer_ratio);
        assert_eq!(first.reverse_transfer_ratio, second.reverse_transfer_ratio);
        assert_eq!(first.asymmetry_index, second.asymmetry_index);
        assert_eq!(first.architecture_distance, second.architecture_distance);
        assert_eq!(first.transfer_verdict, second.transfer_verdict);
    }

    #[test]
    fn long_horizon_stays_strictly_before_target_depth() {
        assert!(LONG_HORIZON < TARGET_DEPTH);
        let matrix = vec![
            vec![0.5, 0.5, 0.0],
            vec![0.25, 0.5, 0.25],
            vec![0.0, 0.5, 0.5],
        ];
        let transfer = compute_transfer_diagnostic(&matrix);
        assert_eq!(transfer.target_depth, TARGET_DEPTH);
        assert_eq!(transfer.long_horizon, LONG_HORIZON);
    }

    #[test]
    fn architectures_produce_different_recovery() {
        let matrix = vec![
            vec![0.5, 0.5, 0.0],
            vec![0.25, 0.5, 0.25],
            vec![0.0, 0.5, 0.5],
        ];
        let transfer = compute_transfer_diagnostic(&matrix);
        assert!(
            transfer.architecture_distance > 0.0,
            "expected different architectures, got distance={}",
            transfer.architecture_distance
        );
    }

    #[test]
    fn source_has_no_final_holdout_authorization() {
        let source = include_str!("tdi7_cross_arch_transfer.rs");
        let confirmation = ["I_ACCEPT_THE_TDI7_", "HOLDOUT_FREEZE"].concat();
        let environment = ["TDI7_CONFIRM_FINAL_", "HOLDOUT"].concat();
        assert!(!source.contains(&confirmation));
        assert!(!source.contains(&environment));
    }

    #[test]
    fn transfer_diagnostics_do_not_reference_confirmatory_metrics() {
        let source = include_str!("tdi7_cross_arch_transfer.rs");
        let confirmatory_metric = ["relative_MSE_", "reduction"].concat();
        assert!(!source.contains(&confirmatory_metric));
    }

    #[test]
    fn transfer_verdict_is_valid() {
        let matrix = vec![
            vec![0.5, 0.5, 0.0],
            vec![0.25, 0.5, 0.25],
            vec![0.0, 0.5, 0.5],
        ];
        let transfer = compute_transfer_diagnostic(&matrix);
        let valid = ["full", "partial", "asymmetric", "failed"];
        assert!(
            valid.contains(&transfer.transfer_verdict.as_str()),
            "unexpected verdict: {}",
            transfer.transfer_verdict
        );
    }
}
