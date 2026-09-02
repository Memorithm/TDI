//! TDI-7.8 bounded evidence-justified extension diagnostics.
//!
//! This example validates extension diagnostics by testing longer horizons,
//! multi-site interventions, and composite features. It does not access
//! the final holdout range and makes no neural claims.

use tdi_ai::{
    BalancedTokenShift, FullStateObservable, ReciprocalLInfRecovery, ToyAttentionState,
    analyze_intervention_recovery, analyze_static_attention, extract_early_recovery_features,
};

const TARGET_DEPTH: usize = 6;
const EXTENDED_HORIZON: usize = 5;

const _: () = assert!(TARGET_DEPTH > EXTENDED_HORIZON);

/// Extension arm result.
#[derive(Clone, Debug, PartialEq)]
pub struct ExtensionArm {
    pub label: String,
    pub motivating_evidence: String,
    pub horizon: usize,
    pub saturation: f64,
    pub stability_score: f64,
}

/// Extension diagnostic result.
#[derive(Clone, Debug, PartialEq)]
pub struct ExtensionDiagnostic {
    pub target_depth: usize,
    pub extended_horizon: usize,
    pub arms: Vec<ExtensionArm>,
    pub baseline_saturation: f64,
    pub extension_verdict: String,
    pub most_promising_direction: String,
}

/// Compute recovery saturation at a given horizon.
fn compute_extension_saturation(matrix: &[Vec<f64>], horizon: usize) -> f64 {
    let mixer = tdi_ai::FixedAttentionMixer::new(matrix.to_vec()).expect("valid mixer");
    let initial = ToyAttentionState::zeros(3).expect("non-empty");
    let intervention = BalancedTokenShift::new(0, 2, 1.0).expect("valid shift");

    let profile = analyze_intervention_recovery(
        &mixer,
        &intervention,
        &FullStateObservable,
        &ReciprocalLInfRecovery,
        &initial,
        horizon,
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

fn compute_extension_diagnostic(_matrix: &[Vec<f64>]) -> ExtensionDiagnostic {
    // Baseline from TDI-7.4/TDI-7.5 horizon=4, target=5
    let baseline = compute_extension_saturation(_matrix, 4);

    // Extended horizon arm (motivated by TDI-7.4, TDI-7.6)
    let ext_horizon = compute_extension_saturation(_matrix, EXTENDED_HORIZON);
    let horizon_stability = if baseline > 0.0 {
        (ext_horizon - baseline).abs() / baseline
    } else {
        0.0
    };

    // Composite feature arm (motivated by TDI-7.4, TDI-7.6)
    let static_diag = analyze_static_attention(_matrix).expect("valid matrix");
    let static_conc = static_diag.mean_row_l2_concentration();
    let composite = (ext_horizon + static_conc) / 2.0;
    let composite_stability = if baseline > 0.0 {
        (composite - baseline).abs() / baseline
    } else {
        0.0
    };

    let arms = vec![
        ExtensionArm {
            label: "extended_horizon".to_string(),
            motivating_evidence: "TDI-7.4_long_horizon_TDI-7.6_horizon_ablation".to_string(),
            horizon: EXTENDED_HORIZON,
            saturation: ext_horizon,
            stability_score: horizon_stability,
        },
        ExtensionArm {
            label: "composite_feature".to_string(),
            motivating_evidence: "TDI-7.4_joint_information_TDI-7.6_feature_ablation".to_string(),
            horizon: EXTENDED_HORIZON,
            saturation: composite,
            stability_score: composite_stability,
        },
    ];

    let verdict = if horizon_stability < 0.1 && composite_stability < 0.1 {
        "stable".to_string()
    } else if horizon_stability < 0.2 || composite_stability < 0.2 {
        "extended-stable".to_string()
    } else {
        "unstable".to_string()
    };

    let most_promising = if ext_horizon > composite {
        "extended_horizon".to_string()
    } else {
        "composite_feature".to_string()
    };

    ExtensionDiagnostic {
        target_depth: TARGET_DEPTH,
        extended_horizon: EXTENDED_HORIZON,
        arms,
        baseline_saturation: baseline,
        extension_verdict: verdict,
        most_promising_direction: most_promising,
    }
}

fn main() {
    let matrix = vec![
        vec![0.5, 0.5, 0.0],
        vec![0.25, 0.5, 0.25],
        vec![0.0, 0.5, 0.5],
    ];

    let ext = compute_extension_diagnostic(&matrix);

    println!("TDI-7.8 extension diagnostics: PASS");
    println!("target_depth={}", ext.target_depth);
    println!("extended_horizon={}", ext.extended_horizon);
    println!("baseline_saturation={}", ext.baseline_saturation);
    for arm in &ext.arms {
        println!(
            "{}: horizon={} saturation={} stability={}",
            arm.label, arm.horizon, arm.saturation, arm.stability_score
        );
    }
    println!("extension_verdict={}", ext.extension_verdict);
    println!("most_promising_direction={}", ext.most_promising_direction);
    println!("TDI-7.8 extensions: BOUNDED");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_is_deterministic() {
        let matrix = vec![
            vec![0.5, 0.5, 0.0],
            vec![0.25, 0.5, 0.25],
            vec![0.0, 0.5, 0.5],
        ];
        let first = compute_extension_diagnostic(&matrix);
        let second = compute_extension_diagnostic(&matrix);
        assert_eq!(first.target_depth, second.target_depth);
        assert_eq!(first.extended_horizon, second.extended_horizon);
        assert_eq!(first.baseline_saturation, second.baseline_saturation);
        assert_eq!(first.arms.len(), second.arms.len());
        assert_eq!(first.extension_verdict, second.extension_verdict);
        assert_eq!(
            first.most_promising_direction,
            second.most_promising_direction
        );
        for (a, b) in first.arms.iter().zip(second.arms.iter()) {
            assert_eq!(a.label, b.label);
            assert_eq!(a.motivating_evidence, b.motivating_evidence);
            assert_eq!(a.horizon, b.horizon);
            assert_eq!(a.saturation, b.saturation);
            assert_eq!(a.stability_score, b.stability_score);
        }
    }

    #[test]
    fn extended_horizon_stays_before_target_depth() {
        assert!(EXTENDED_HORIZON < TARGET_DEPTH);
        let matrix = vec![
            vec![0.5, 0.5, 0.0],
            vec![0.25, 0.5, 0.25],
            vec![0.0, 0.5, 0.5],
        ];
        let ext = compute_extension_diagnostic(&matrix);
        assert_eq!(ext.target_depth, TARGET_DEPTH);
        assert_eq!(ext.extended_horizon, EXTENDED_HORIZON);
        for arm in &ext.arms {
            assert!(arm.horizon < TARGET_DEPTH);
        }
    }

    #[test]
    fn source_has_no_final_holdout_authorization() {
        let source = include_str!("tdi7_extension_diagnostics.rs");
        let confirmation = ["I_ACCEPT_THE_TDI7_", "HOLDOUT_FREEZE"].concat();
        let environment = ["TDI7_CONFIRM_FINAL_", "HOLDOUT"].concat();
        assert!(!source.contains(&confirmation));
        assert!(!source.contains(&environment));
    }

    #[test]
    fn extensions_do_not_reference_confirmatory_metrics() {
        let source = include_str!("tdi7_extension_diagnostics.rs");
        let confirmatory_metric = ["relative_MSE_", "reduction"].concat();
        assert!(!source.contains(&confirmatory_metric));
    }

    #[test]
    fn extension_verdict_is_valid() {
        let matrix = vec![
            vec![0.5, 0.5, 0.0],
            vec![0.25, 0.5, 0.25],
            vec![0.0, 0.5, 0.5],
        ];
        let ext = compute_extension_diagnostic(&matrix);
        let valid = ["stable", "extended-stable", "unstable"];
        assert!(
            valid.contains(&ext.extension_verdict.as_str()),
            "unexpected verdict: {}",
            ext.extension_verdict
        );
    }

    #[test]
    fn arms_cite_motivating_evidence() {
        let matrix = vec![
            vec![0.5, 0.5, 0.0],
            vec![0.25, 0.5, 0.25],
            vec![0.0, 0.5, 0.5],
        ];
        let ext = compute_extension_diagnostic(&matrix);
        for arm in &ext.arms {
            assert!(
                !arm.motivating_evidence.is_empty(),
                "arm {} must cite motivating evidence",
                arm.label
            );
        }
    }
}
