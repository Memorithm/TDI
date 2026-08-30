//! Internal TDI-7 evidence handoff schema fixture.
//!
//! This is intentionally an example, not yet a public `tdi-ai` API. It freezes
//! validation behavior before any cross-project schema promotion.

const REL_TOLERANCE: f64 = 1.0e-12;
const RELEVANCE_MARGIN: f64 = 0.02;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Verdict {
    Beneficial,
    Equivalent,
    Harmful,
    Inconclusive,
}

#[derive(Clone, Debug, PartialEq)]
struct TaskEvidence {
    task_id: &'static str,
    generator_count: usize,
    intervention_pair_count: usize,
    rejected_count: usize,
    b0_mse: f64,
    b1_mse: f64,
    relative_mse_reduction: f64,
    lower_95: f64,
    upper_95: f64,
    verdict: Verdict,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Provenance {
    tdi_commit: String,
    protocol_id: &'static str,
    evaluator_spec: &'static str,
    semantic_id: &'static str,
    generator_version: &'static str,
    seed_range_id: &'static str,
    intervention_id: &'static str,
    intervention_aggregation: &'static str,
    observation_depths: &'static str,
    feature_schema: &'static str,
    model_id: &'static str,
    bootstrap_id: &'static str,
    numerical_policy: &'static str,
    classifier_margin: &'static str,
    classifier_policy: &'static str,
    final_holdout_accessed: bool,
}

#[derive(Clone, Debug, PartialEq)]
struct EvidencePacket {
    tasks: Vec<TaskEvidence>,
    provenance: Provenance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EvidenceError {
    MissingTask,
    EmptyTaskId,
    EmptyPopulation,
    InterventionCountMismatch,
    InvalidMse,
    InvalidInterval,
    RelativeReductionMismatch,
    VerdictMismatch,
    InvalidCommitSha,
    IncompleteProvenance,
}

fn classify(r: f64, lower: f64, upper: f64) -> Verdict {
    if !r.is_finite() || !lower.is_finite() || !upper.is_finite() || lower > upper {
        return Verdict::Inconclusive;
    }
    // Precedence is part of the frozen implementation behavior: at an exact
    // relevance-margin boundary, Beneficial/Harmful is evaluated before the
    // complete-interval Equivalent rule.
    if r >= RELEVANCE_MARGIN && lower > 0.0 {
        Verdict::Beneficial
    } else if r <= -RELEVANCE_MARGIN && upper < 0.0 {
        Verdict::Harmful
    } else if lower >= -RELEVANCE_MARGIN && upper <= RELEVANCE_MARGIN {
        Verdict::Equivalent
    } else {
        Verdict::Inconclusive
    }
}

fn valid_sha(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn validate_task(task: &TaskEvidence) -> Result<(), EvidenceError> {
    if task.task_id.is_empty() {
        return Err(EvidenceError::EmptyTaskId);
    }
    if task.generator_count == 0 || task.intervention_pair_count == 0 {
        return Err(EvidenceError::EmptyPopulation);
    }
    if task.generator_count.checked_mul(2) != Some(task.intervention_pair_count) {
        return Err(EvidenceError::InterventionCountMismatch);
    }
    if !task.b0_mse.is_finite()
        || !task.b1_mse.is_finite()
        || task.b0_mse <= 0.0
        || task.b1_mse < 0.0
    {
        return Err(EvidenceError::InvalidMse);
    }
    if !task.lower_95.is_finite()
        || !task.upper_95.is_finite()
        || task.lower_95 > task.upper_95
    {
        return Err(EvidenceError::InvalidInterval);
    }
    let expected = (task.b0_mse - task.b1_mse) / task.b0_mse;
    if !task.relative_mse_reduction.is_finite()
        || (task.relative_mse_reduction - expected).abs() > REL_TOLERANCE
    {
        return Err(EvidenceError::RelativeReductionMismatch);
    }
    if task.verdict != classify(task.relative_mse_reduction, task.lower_95, task.upper_95) {
        return Err(EvidenceError::VerdictMismatch);
    }
    Ok(())
}

fn validate_provenance(provenance: &Provenance) -> Result<(), EvidenceError> {
    if !valid_sha(&provenance.tdi_commit) {
        return Err(EvidenceError::InvalidCommitSha);
    }
    let required = [
        provenance.protocol_id,
        provenance.evaluator_spec,
        provenance.semantic_id,
        provenance.generator_version,
        provenance.seed_range_id,
        provenance.intervention_id,
        provenance.intervention_aggregation,
        provenance.observation_depths,
        provenance.feature_schema,
        provenance.model_id,
        provenance.bootstrap_id,
        provenance.numerical_policy,
        provenance.classifier_margin,
        provenance.classifier_policy,
    ];
    if required.iter().any(|value| value.is_empty()) {
        return Err(EvidenceError::IncompleteProvenance);
    }
    Ok(())
}

fn validate(packet: &EvidencePacket) -> Result<(), EvidenceError> {
    if packet.tasks.is_empty() {
        return Err(EvidenceError::MissingTask);
    }
    validate_provenance(&packet.provenance)?;
    for task in &packet.tasks {
        validate_task(task)?;
    }
    Ok(())
}

fn fixture() -> EvidencePacket {
    EvidencePacket {
        tasks: vec![TaskEvidence {
            task_id: "associative_recall",
            generator_count: 64,
            intervention_pair_count: 128,
            rejected_count: 0,
            b0_mse: 1.0,
            b1_mse: 0.9,
            relative_mse_reduction: 0.1,
            lower_95: 0.05,
            upper_95: 0.15,
            verdict: Verdict::Beneficial,
        }],
        provenance: Provenance {
            tdi_commit: "0123456789abcdef0123456789abcdef01234567".to_string(),
            protocol_id: "TDI-7.0",
            evaluator_spec: "TDI-7.1-EVALUATOR-SPEC",
            semantic_id: "deterministic_local_row_stochastic_v1",
            generator_version: "tdi7_generator_v1",
            seed_range_id: "non_holdout_fixture",
            intervention_id: "balanced_single_site_amp_0.25",
            intervention_aggregation: "two_sites_per_generator_equal_record_weighting",
            observation_depths: "1,2",
            feature_schema: "static_task_plus_raw_recovery",
            model_id: "ridge_linear_shared_grid",
            bootstrap_id: "paired_generator_2000",
            numerical_policy: "rust_f64_scalar_reference",
            classifier_margin: "+/-0.02",
            classifier_policy: "beneficial_then_harmful_then_equivalent_then_inconclusive",
            final_holdout_accessed: false,
        },
    }
}

fn main() {
    let packet = fixture();
    validate(&packet).expect("bounded evidence fixture must validate");
    println!("TDI-7 evidence-schema preflight: PASS");
    println!("public_api_status=NOT_PROMOTED");
    println!("final_holdout_accessed={}", packet.provenance.final_holdout_accessed);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_packet_passes() {
        assert_eq!(validate(&fixture()), Ok(()));
    }

    #[test]
    fn relative_reduction_is_recomputed_not_trusted() {
        let mut packet = fixture();
        packet.tasks[0].relative_mse_reduction = 0.2;
        assert_eq!(validate(&packet), Err(EvidenceError::RelativeReductionMismatch));
    }

    #[test]
    fn verdict_must_match_frozen_classifier() {
        let mut packet = fixture();
        packet.tasks[0].verdict = Verdict::Equivalent;
        assert_eq!(validate(&packet), Err(EvidenceError::VerdictMismatch));
    }

    #[test]
    fn classifier_precedence_is_explicit_at_exact_relevance_boundaries() {
        assert_eq!(classify(0.02, 0.01, 0.02), Verdict::Beneficial);
        assert_eq!(classify(-0.02, -0.02, -0.01), Verdict::Harmful);
        assert_eq!(classify(0.0, -0.02, 0.02), Verdict::Equivalent);
    }

    #[test]
    fn unbalanced_intervention_counts_fail_closed() {
        let mut packet = fixture();
        packet.tasks[0].intervention_pair_count = 127;
        assert_eq!(validate(&packet), Err(EvidenceError::InterventionCountMismatch));
    }

    #[test]
    fn malformed_interval_fails_closed() {
        let mut packet = fixture();
        packet.tasks[0].lower_95 = 0.2;
        packet.tasks[0].upper_95 = 0.1;
        assert_eq!(validate(&packet), Err(EvidenceError::InvalidInterval));
    }

    #[test]
    fn malformed_commit_identity_fails_closed() {
        let mut packet = fixture();
        packet.provenance.tdi_commit = "deadbeef".to_string();
        assert_eq!(validate(&packet), Err(EvidenceError::InvalidCommitSha));
    }

    #[test]
    fn empty_provenance_field_fails_closed() {
        let mut packet = fixture();
        packet.provenance.semantic_id = "";
        assert_eq!(validate(&packet), Err(EvidenceError::IncompleteProvenance));
    }
}
