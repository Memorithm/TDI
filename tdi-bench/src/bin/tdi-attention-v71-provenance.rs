//! Explicit provenance record for the bounded TDI-7.1 evaluator.
//!
//! This binary records frozen/preflight configuration only. It does not execute
//! a final holdout or infer a scientific verdict.

const SEMANTIC_ID: &str = "deterministic_local_row_stochastic_v1";
const TASK_GENERATOR_VERSION: &str = "tdi7-hai1-generators-v1";
const TRAINING_RANGE: &str = "7100000000..=7100009999";
const DEVELOPMENT_RANGE: &str = "7100010000..=7100019999";
const VALIDATION_RANGE: &str = "7100020000..=7100029999";
const FINAL_HOLDOUT_RANGE: &str = "7100030000..=7100039999";
const TRAINING_GENERATOR_COUNT: usize = 96;
const DEVELOPMENT_GENERATOR_COUNT: usize = 48;
const VALIDATION_GENERATOR_COUNT: usize = 48;
const FINAL_HOLDOUT_GENERATOR_COUNT_STATUS: &str = "UNFROZEN";
const INTERVENTION_SITES: &str = "early-token,late-token";
const INTERVENTION_POLICY: &str = "single-site-one-shot-task-label-preserving";
const INTERVENTION_AGGREGATION: &str = "two_sites_per_generator_equal_record_weighting";
const INTERVENTION_AMPLITUDE: f64 = 0.25;
const EARLY_OBSERVATION_DEPTHS: &str = "1,2";
const TARGET_DEPTH: usize = 5;
const TARGET_DEFINITION: &str = "bounded_retrieval_deficit:d/(1+d)";
const STATIC_FEATURE_SCHEMA: &str = "sequence_length,distractor_count,retrieval_distance,mean_row_entropy_nats,mean_normalized_row_entropy,mean_row_max_weight,mean_row_l2_concentration,mean_row_effective_support,frobenius_norm,intervention_site_indicator";
const RECOVERY_FEATURE_SCHEMA: &str = "recovery_depth_1,recovery_depth_2";
const MODEL_CLASS: &str = "ridge-linear-with-intercept";
const LAMBDA_GRID: &str = "0,1e-6,1e-3,1e-1";
const PRIMARY_LOSS: &str = "mse";
const BOOTSTRAP_SEED: &str = "0x5444493745324501";
const BOOTSTRAP_REPLICATES: usize = 2_000;
const NUMERICAL_POLICY: &str = "rust-f64-scalar;pivot_tolerance=1e-12";
const RELEVANCE_MARGIN: f64 = 0.02;
const CLASSIFIER_POLICY: &str = "beneficial_then_harmful_then_equivalent_then_inconclusive";

#[derive(Clone, Debug, PartialEq, Eq)]
struct Provenance {
    tdi_commit: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ProvenanceError {
    MissingCommit,
    InvalidCommit,
}

impl Provenance {
    fn new(tdi_commit: &str) -> Result<Self, ProvenanceError> {
        if tdi_commit.is_empty() {
            return Err(ProvenanceError::MissingCommit);
        }
        if tdi_commit.len() != 40 || !tdi_commit.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(ProvenanceError::InvalidCommit);
        }
        Ok(Self {
            tdi_commit: tdi_commit.to_ascii_lowercase(),
        })
    }

    fn lines(&self) -> Vec<String> {
        vec![
            format!("tdi_commit={}", self.tdi_commit),
            format!("semantic_id={SEMANTIC_ID}"),
            format!("task_generator_version={TASK_GENERATOR_VERSION}"),
            format!("training_seed_range={TRAINING_RANGE}"),
            format!("development_seed_range={DEVELOPMENT_RANGE}"),
            format!("validation_seed_range={VALIDATION_RANGE}"),
            format!("final_holdout_seed_range={FINAL_HOLDOUT_RANGE}"),
            format!("training_generator_count={TRAINING_GENERATOR_COUNT}"),
            format!("development_generator_count={DEVELOPMENT_GENERATOR_COUNT}"),
            format!("validation_generator_count={VALIDATION_GENERATOR_COUNT}"),
            format!(
                "final_holdout_generator_count={FINAL_HOLDOUT_GENERATOR_COUNT_STATUS}"
            ),
            format!("intervention_sites={INTERVENTION_SITES}"),
            format!("intervention_policy={INTERVENTION_POLICY}"),
            format!("intervention_aggregation={INTERVENTION_AGGREGATION}"),
            format!("intervention_amplitude={INTERVENTION_AMPLITUDE}"),
            format!("early_observation_depths={EARLY_OBSERVATION_DEPTHS}"),
            format!("target_depth={TARGET_DEPTH}"),
            format!("target_definition={TARGET_DEFINITION}"),
            format!("static_feature_schema={STATIC_FEATURE_SCHEMA}"),
            format!("recovery_feature_schema={RECOVERY_FEATURE_SCHEMA}"),
            format!("model_class={MODEL_CLASS}"),
            format!("lambda_grid={LAMBDA_GRID}"),
            format!("primary_loss={PRIMARY_LOSS}"),
            format!("bootstrap_seed={BOOTSTRAP_SEED}"),
            format!("bootstrap_replicates={BOOTSTRAP_REPLICATES}"),
            format!("numerical_policy={NUMERICAL_POLICY}"),
            format!("relevance_margin={RELEVANCE_MARGIN}"),
            format!("classifier_policy={CLASSIFIER_POLICY}"),
            "final_holdout_status=NOT_ACCESSED".to_string(),
        ]
    }
}

fn parse_commit_argument() -> Result<String, ProvenanceError> {
    let mut args = std::env::args().skip(1);
    match (args.next().as_deref(), args.next(), args.next()) {
        (Some("--tdi-commit"), Some(value), None) => Ok(value),
        _ => Err(ProvenanceError::MissingCommit),
    }
}

fn main() {
    let commit = parse_commit_argument().and_then(|value| Provenance::new(&value));
    let provenance = match commit {
        Ok(value) => value,
        Err(error) => {
            eprintln!("TDI-7.1 provenance error: {error:?}");
            std::process::exit(2);
        }
    };

    for line in provenance.lines() {
        println!("{line}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines() -> String {
        Provenance::new("0123456789abcdef0123456789abcdef01234567")
            .unwrap()
            .lines()
            .join("\n")
    }

    #[test]
    fn valid_sha_is_normalized_and_emitted() {
        let provenance = Provenance::new("ABCDEF0123456789ABCDEF0123456789ABCDEF01").unwrap();
        assert_eq!(
            provenance.tdi_commit,
            "abcdef0123456789abcdef0123456789abcdef01"
        );
        assert!(
            provenance
                .lines()
                .iter()
                .any(|line| line.starts_with("semantic_id="))
        );
    }

    #[test]
    fn malformed_sha_is_rejected() {
        assert_eq!(Provenance::new(""), Err(ProvenanceError::MissingCommit));
        assert_eq!(Provenance::new("abc"), Err(ProvenanceError::InvalidCommit));
        assert_eq!(
            Provenance::new("zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz"),
            Err(ProvenanceError::InvalidCommit)
        );
    }

    #[test]
    fn all_frozen_split_ranges_are_reported() {
        let output = lines();
        assert!(output.contains(TRAINING_RANGE));
        assert!(output.contains(DEVELOPMENT_RANGE));
        assert!(output.contains(VALIDATION_RANGE));
        assert!(output.contains(FINAL_HOLDOUT_RANGE));
    }

    #[test]
    fn bounded_population_counts_match_end_to_end_evaluator() {
        let output = lines();
        assert!(output.contains("training_generator_count=96"));
        assert!(output.contains("development_generator_count=48"));
        assert!(output.contains("validation_generator_count=48"));
    }

    #[test]
    fn primary_bootstrap_seed_matches_end_to_end_specification() {
        assert_eq!(BOOTSTRAP_SEED, "0x5444493745324501");
        assert!(lines().contains("bootstrap_seed=0x5444493745324501"));
    }

    #[test]
    fn semantic_and_feature_schema_match_primary_evaluator() {
        let output = lines();
        assert!(output.contains("semantic_id=deterministic_local_row_stochastic_v1"));
        assert!(output.contains("early_observation_depths=1,2"));
        assert!(output.contains("target_depth=5"));
        assert!(output.contains("target_definition=bounded_retrieval_deficit:d/(1+d)"));
        assert!(output.contains("intervention_amplitude=0.25"));
        assert!(output.contains("intervention_site_indicator"));
        assert!(output.contains("recovery_feature_schema=recovery_depth_1,recovery_depth_2"));
    }

    #[test]
    fn intervention_aggregation_and_classifier_precedence_are_explicit() {
        let output = lines();
        assert!(output.contains(
            "intervention_aggregation=two_sites_per_generator_equal_record_weighting"
        ));
        assert!(output.contains(
            "classifier_policy=beneficial_then_harmful_then_equivalent_then_inconclusive"
        ));
    }

    #[test]
    fn unresolved_final_population_is_explicit_and_fail_closed() {
        assert_eq!(FINAL_HOLDOUT_GENERATOR_COUNT_STATUS, "UNFROZEN");
        assert!(lines().contains("final_holdout_generator_count=UNFROZEN"));
    }

    #[test]
    fn provenance_reports_no_final_holdout_access() {
        let provenance = Provenance::new("0123456789abcdef0123456789abcdef01234567").unwrap();
        assert!(
            provenance
                .lines()
                .contains(&"final_holdout_status=NOT_ACCESSED".to_string())
        );
    }

    #[test]
    fn source_has_no_holdout_authorization_token() {
        let source = include_str!("tdi-attention-v71-provenance.rs");
        let confirmation = ["I_ACCEPT_THE_TDI7_", "HOLDOUT_FREEZE"].concat();
        let environment = ["TDI7_CONFIRM_FINAL_", "HOLDOUT"].concat();
        assert!(!source.contains(&confirmation));
        assert!(!source.contains(&environment));
    }
}
