// TDI-7.8 evidence-justified extension features.
// These features are bounded to the TDI-7.1 evaluator and do not access
// the final holdout range (7100030000-7100039999).
// TDI-7.3 uses the frozen decision records from docs/TDI-7.3-*.toml
// TDI-7.4 uses the frozen decision records from docs/TDI-7.4-*.toml
// TDI-7.5 uses the frozen decision records from docs/TDI-7.5-*.toml
// TDI-7.6 uses the frozen decision records from docs/TDI-7.6-*.toml
// TDI-7.7 uses the frozen decision records from docs/TDI-7.7-*.toml
// TDI-7.8 uses the frozen decision records from docs/TDI-7.8-*.toml

//! TDI-7.1 bounded feature extraction over the merged `tdi-ai` contracts.
//!
//! This example keeps static controls and early dynamic recovery descriptors
//! structurally separate and never reads a confirmatory late target.

use tdi_ai::{
    BalancedTokenShift, FixedAttentionMixer, FullStateObservable, ReciprocalLInfRecovery,
    ToyAttentionState, analyze_intervention_recovery, analyze_static_attention,
};

#[derive(Clone, Debug, PartialEq)]
struct StaticFeatureBlock {
    rows: usize,
    columns: usize,
    mean_entropy: f64,
    mean_normalized_entropy: f64,
    mean_max_weight: f64,
    mean_l2_concentration: f64,
    mean_effective_support: f64,
    frobenius_norm: f64,
}

#[derive(Clone, Debug, PartialEq)]
struct RecoveryFeatureBlock {
    early_depths: Vec<usize>,
    recovery: Vec<f64>,
}

fn fixture_matrix() -> Vec<Vec<f64>> {
    vec![
        vec![0.5, 0.5, 0.0],
        vec![0.25, 0.5, 0.25],
        vec![0.0, 0.5, 0.5],
    ]
}

fn static_features(matrix: &[Vec<f64>]) -> StaticFeatureBlock {
    let diagnostics = analyze_static_attention(matrix).expect("fixture is row stochastic");
    StaticFeatureBlock {
        rows: diagnostics.rows(),
        columns: diagnostics.columns(),
        mean_entropy: diagnostics.mean_row_entropy_nats(),
        mean_normalized_entropy: diagnostics.mean_normalized_row_entropy(),
        mean_max_weight: diagnostics.mean_row_max_weight(),
        mean_l2_concentration: diagnostics.mean_row_l2_concentration(),
        mean_effective_support: diagnostics.mean_row_effective_support(),
        frobenius_norm: diagnostics.frobenius_norm(),
    }
}

fn recovery_features(matrix: Vec<Vec<f64>>, early_horizon: usize) -> RecoveryFeatureBlock {
    let mixer = FixedAttentionMixer::new(matrix).expect("fixture mixer is valid");
    let initial = ToyAttentionState::zeros(3).expect("non-empty fixture");
    let intervention = BalancedTokenShift::new(0, 2, 1.0).expect("valid balanced shift");
    let profile = analyze_intervention_recovery(
        &mixer,
        &intervention,
        &FullStateObservable,
        &ReciprocalLInfRecovery,
        &initial,
        early_horizon,
    )
    .expect("deterministic recovery succeeds");

    RecoveryFeatureBlock {
        early_depths: profile.points().iter().map(|point| point.depth()).collect(),
        recovery: profile
            .points()
            .iter()
            .map(|point| *point.overlap())
            .collect(),
    }
}

fn main() {
    let matrix = fixture_matrix();
    let static_block = static_features(&matrix);
    let recovery_block = recovery_features(matrix, 2);
    println!("TDI-7.1 feature preflight: PASS");
    println!("static_feature_count=8");
    println!("early_recovery_depths={:?}", recovery_block.early_depths);
    println!(
        "static_rows={} static_columns={}",
        static_block.rows, static_block.columns
    );
    println!("TDI-7.2 late target: NOT READ");
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
    fn static_and_dynamic_blocks_remain_separate() {
        let matrix = fixture_matrix();
        let static_block = static_features(&matrix);
        let recovery_block = recovery_features(matrix, 2);
        assert_eq!(static_block.rows, 3);
        assert_eq!(static_block.columns, 3);
        assert_eq!(recovery_block.early_depths, vec![1, 2]);
        assert_eq!(recovery_block.recovery.len(), 2);
    }

    #[test]
    fn recovery_fixture_matches_independent_gate_b_oracle() {
        let block = recovery_features(fixture_matrix(), 2);
        assert_close(block.recovery[0], 2.0 / 3.0);
        assert_close(block.recovery[1], 4.0 / 5.0);
    }

    #[test]
    fn early_horizon_is_explicit_and_bounded() {
        let one = recovery_features(fixture_matrix(), 1);
        let three = recovery_features(fixture_matrix(), 3);
        assert_eq!(one.early_depths, vec![1]);
        assert_eq!(three.early_depths, vec![1, 2, 3]);
    }

    #[test]
    fn feature_source_does_not_reference_confirmatory_target_or_holdout_token() {
        let source = include_str!("tdi7_features.rs");
        let confirmation = ["I_ACCEPT_THE_TDI7_", "HOLDOUT_FREEZE"].concat();
        let environment = ["TDI7_CONFIRM_FINAL_", "HOLDOUT"].concat();
        let confirmatory_metric = ["relative_MSE_", "reduction"].concat();
        assert!(!source.contains(&confirmation));
        assert!(!source.contains(&environment));
        assert!(!source.contains(&confirmatory_metric));
    }

    #[test]
    fn static_effective_support_is_not_exposed_as_rank() {
        let block = static_features(&fixture_matrix());
        assert!(block.mean_effective_support > 1.0);
        let source = include_str!("tdi7_features.rs");
        let forbidden_name = ["effective_", "rank"].concat();
        assert!(!source.contains(&forbidden_name));
    }
}
