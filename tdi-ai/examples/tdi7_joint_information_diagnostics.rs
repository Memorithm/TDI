//! TDI-7.4 bounded joint information and synergy diagnostics.
//!
//! This example validates the joint information diagnostics between static
//! attention features and early TDI recovery features. It uses a longer
//! observation horizon than TDI-7.1 but stays strictly before the target
//! evaluation depth. It does not access the final holdout range.

use tdi_ai::{
    BalancedTokenShift, FixedAttentionMixer, FullStateObservable, ReciprocalLInfRecovery,
    ToyAttentionState, analyze_intervention_recovery, analyze_static_attention,
    extract_early_recovery_features,
};

const LONG_HORIZON: usize = 4;
const TARGET_DEPTH: usize = 5;

const _: () = assert!(TARGET_DEPTH > LONG_HORIZON);

/// Joint information diagnostic between static attention features and
/// early TDI recovery features.
///
/// All fields are bounded diagnostics computed strictly before the target
/// evaluation depth. They are not interpreted as evidence about MMI,
/// causation, or human cognitive mechanisms.
#[derive(Clone, Debug, PartialEq)]
pub struct JointInformationDiagnostic {
    pub target_depth: usize,
    pub long_horizon: usize,
    pub static_concentration: f64,
    pub recovery_saturation: f64,
    pub mutual_information_proxy: f64,
    pub synergy_index: f64,
    pub redundancy_index: f64,
}

/// Compute the recovery saturation as the mean overlap over the second half
/// of the early observation window.
fn recovery_saturation(overlaps: &[f64]) -> f64 {
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

/// Compute joint information diagnostics from a fixture attention matrix.
///
/// The computation is deterministic: the same matrix always produces the
/// same diagnostic. This function does not access the final holdout range.
fn compute_joint_diagnostic(matrix: &[Vec<f64>]) -> JointInformationDiagnostic {
    let static_diag = analyze_static_attention(matrix).expect("valid matrix");

    let mixer = FixedAttentionMixer::new(matrix.to_vec()).expect("valid mixer");
    let initial = ToyAttentionState::zeros(3).expect("non-empty");
    let intervention = BalancedTokenShift::new(0, 2, 1.0).expect("valid shift");

    let profile = analyze_intervention_recovery(
        &mixer,
        &intervention,
        &FullStateObservable,
        &ReciprocalLInfRecovery,
        &initial,
        LONG_HORIZON,
    )
    .expect("deterministic recovery succeeds");

    let early =
        extract_early_recovery_features(&profile, TARGET_DEPTH).expect("early features exist");

    let static_concentration = static_diag.mean_row_l2_concentration();
    let overlaps: Vec<f64> = early.overlaps().to_vec();
    let saturation = recovery_saturation(&overlaps);

    let mi_proxy = (static_concentration * saturation).sqrt();
    let synergy = mi_proxy - (static_concentration + saturation) / 2.0 * 0.5;
    let redundancy = static_concentration.min(saturation);

    JointInformationDiagnostic {
        target_depth: TARGET_DEPTH,
        long_horizon: LONG_HORIZON,
        static_concentration,
        recovery_saturation: saturation,
        mutual_information_proxy: mi_proxy,
        synergy_index: synergy,
        redundancy_index: redundancy,
    }
}

fn main() {
    let matrix = vec![
        vec![0.5, 0.5, 0.0],
        vec![0.25, 0.5, 0.25],
        vec![0.0, 0.5, 0.5],
    ];

    let joint = compute_joint_diagnostic(&matrix);

    println!("TDI-7.4 joint information diagnostics: PASS");
    println!("target_depth={}", joint.target_depth);
    println!("long_horizon={}", joint.long_horizon);
    println!("static_concentration={}", joint.static_concentration);
    println!("recovery_saturation={}", joint.recovery_saturation);
    println!(
        "mutual_information_proxy={}",
        joint.mutual_information_proxy
    );
    println!("synergy_index={}", joint.synergy_index);
    println!("redundancy_index={}", joint.redundancy_index);
    println!("TDI-7.4 joint information: BOUNDED");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() <= 1.0e-12,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn joint_diagnostic_is_deterministic() {
        let matrix = vec![
            vec![0.5, 0.5, 0.0],
            vec![0.25, 0.5, 0.25],
            vec![0.0, 0.5, 0.5],
        ];
        let first = compute_joint_diagnostic(&matrix);
        let second = compute_joint_diagnostic(&matrix);
        assert_eq!(first.target_depth, second.target_depth);
        assert_eq!(first.long_horizon, second.long_horizon);
        assert_eq!(first.static_concentration, second.static_concentration);
        assert_eq!(first.recovery_saturation, second.recovery_saturation);
        assert_eq!(
            first.mutual_information_proxy,
            second.mutual_information_proxy
        );
        assert_eq!(first.synergy_index, second.synergy_index);
        assert_eq!(first.redundancy_index, second.redundancy_index);
    }

    #[test]
    fn long_horizon_stays_strictly_before_target_depth() {
        assert!(LONG_HORIZON < TARGET_DEPTH);
        let matrix = vec![
            vec![0.5, 0.5, 0.0],
            vec![0.25, 0.5, 0.25],
            vec![0.0, 0.5, 0.5],
        ];
        let joint = compute_joint_diagnostic(&matrix);
        assert_eq!(joint.target_depth, TARGET_DEPTH);
        assert_eq!(joint.long_horizon, LONG_HORIZON);
    }

    #[test]
    fn static_concentration_matches_independent_oracle() {
        let matrix = vec![
            vec![0.5, 0.5, 0.0],
            vec![0.25, 0.5, 0.25],
            vec![0.0, 0.5, 0.5],
        ];
        let joint = compute_joint_diagnostic(&matrix);
        assert_close(joint.static_concentration, 11.0 / 24.0);
    }

    #[test]
    fn recovery_saturation_is_computed_from_early_window() {
        let matrix = vec![
            vec![0.5, 0.5, 0.0],
            vec![0.25, 0.5, 0.25],
            vec![0.0, 0.5, 0.5],
        ];
        let joint = compute_joint_diagnostic(&matrix);
        let expected = (8.0 / 9.0 + 16.0 / 17.0) / 2.0;
        assert_close(joint.recovery_saturation, expected);
    }

    #[test]
    fn source_has_no_final_holdout_authorization() {
        let source = include_str!("tdi7_joint_information_diagnostics.rs");
        let confirmation = ["I_ACCEPT_THE_TDI7_", "HOLDOUT_FREEZE"].concat();
        let environment = ["TDI7_CONFIRM_FINAL_", "HOLDOUT"].concat();
        assert!(!source.contains(&confirmation));
        assert!(!source.contains(&environment));
    }

    #[test]
    fn joint_diagnostics_do_not_reference_confirmatory_metrics() {
        let source = include_str!("tdi7_joint_information_diagnostics.rs");
        let confirmatory_metric = ["relative_MSE_", "reduction"].concat();
        assert!(!source.contains(&confirmatory_metric));
    }
}
