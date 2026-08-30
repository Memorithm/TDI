//! Internal TDI-7 evidence handoff schema fixture.
//!
//! This is intentionally an example, not yet a public `tdi-ai` API. It freezes
//! validation behavior before any cross-project schema promotion.

const REL_TOLERANCE: f64 = 1.0e-12;
const RELEVANCE_MARGIN: f64 = 0.02;
const REQUIRED_TASKS: [&str; 2] = ["associative_recall", "copy"];
const REQUIRED_SITES: [&str; 2] = ["early-token", "late-token"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Verdict {
    Beneficial,
    Equivalent,
    Harmful,
    Inconclusive,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RejectionReason {
    code: &'static str,
    count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct InterventionSiteSummary {
    site_id: &'static str,
    record_count: usize,
}

#[derive(Clone, Debug, PartialEq)]
struct TaskEvidence {
    task_id: &'static str,
    generator_count: usize,
    intervention_pair_count: usize,
    rejected_count: usize,
    rejection_reasons: Vec<RejectionReason>,
    intervention_sites: Vec<InterventionSiteSummary>,
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
    aggregate_verdict: Verdict,
    provenance: Provenance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EvidenceError {
    MissingTask,
    UnexpectedTaskSet,
    EmptyTaskId,
    EmptyPopulation,
    InterventionCountMismatch,
    InvalidInterventionSites,
    RejectionLedgerMismatch,
    DuplicateRejectionReason,
    InvalidMse,
    InvalidInterval,
    RelativeReductionMismatch,
    VerdictMismatch,
    AggregateVerdictMismatch,
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

fn combine_task_verdicts(left: Verdict, right: Verdict) -> Verdict {
    if left == Verdict::Harmful || right == Verdict::Harmful {
        Verdict::Harmful
    } else if left == Verdict::Equivalent && right == Verdict::Equivalent {
        Verdict::Equivalent
    } else if left == Verdict::Beneficial || right == Verdict::Beneficial {
        Verdict::Beneficial
    } else {
        Verdict::Inconclusive
    }
}

fn valid_sha(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn validate_rejection_ledger(task: &TaskEvidence) -> Result<(), EvidenceError> {
    let mut sum = 0usize;
    for (index, reason) in task.rejection_reasons.iter().enumerate() {
        if reason.code.is_empty()
            || task.rejection_reasons[(index + 1)..]
                .iter()
                .any(|other| other.code == reason.code)
        {
            return Err(EvidenceError::DuplicateRejectionReason);
        }
        sum = sum
            .checked_add(reason.count)
            .ok_or(EvidenceError::RejectionLedgerMismatch)?;
    }
    if sum != task.rejected_count {
        return Err(EvidenceError::RejectionLedgerMismatch);
    }
    Ok(())
}

fn validate_intervention_sites(task: &TaskEvidence) -> Result<(), EvidenceError> {
    if task.intervention_sites.len() != REQUIRED_SITES.len() {
        return Err(EvidenceError::InvalidInterventionSites);
    }
    for required in REQUIRED_SITES {
        let matches: Vec<_> = task
            .intervention_sites
            .iter()
            .filter(|site| site.site_id == required)
            .collect();
        if matches.len() != 1 || matches[0].record_count != task.generator_count {
            return Err(EvidenceError::InvalidInterventionSites);
        }
    }
    Ok(())
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
    validate_intervention_sites(task)?;
    validate_rejection_ledger(task)?;
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

fn validate_task_set(tasks: &[TaskEvidence]) -> Result<(), EvidenceError> {
    if tasks.len() != REQUIRED_TASKS.len() {
        return Err(EvidenceError::UnexpectedTaskSet);
    }
    for required in REQUIRED_TASKS {
        if tasks.iter().filter(|task| task.task_id == required).count() != 1 {
            return Err(EvidenceError::UnexpectedTaskSet);
        }
    }
    Ok(())
}

fn validate(packet: &EvidencePacket) -> Result<(), EvidenceError> {
    if packet.tasks.is_empty() {
        return Err(EvidenceError::MissingTask);
    }
    validate_task_set(&packet.tasks)?;
    validate_provenance(&packet.provenance)?;
    for task in &packet.tasks {
        validate_task(task)?;
    }
    let associative = packet
        .tasks
        .iter()
        .find(|task| task.task_id == "associative_recall")
        .ok_or(EvidenceError::UnexpectedTaskSet)?;
    let copy = packet
        .tasks
        .iter()
        .find(|task| task.task_id == "copy")
        .ok_or(EvidenceError::UnexpectedTaskSet)?;
    if packet.aggregate_verdict != combine_task_verdicts(associative.verdict, copy.verdict) {
        return Err(EvidenceError::AggregateVerdictMismatch);
    }
    Ok(())
}

fn task_fixture(task_id: &'static str, verdict: Verdict) -> TaskEvidence {
    let (b1_mse, relative_mse_reduction, lower_95, upper_95) = match verdict {
        Verdict::Beneficial => (0.9, 0.1, 0.05, 0.15),
        Verdict::Equivalent => (1.0, 0.0, -0.01, 0.01),
        Verdict::Harmful => (1.1, -0.1, -0.15, -0.05),
        Verdict::Inconclusive => (0.97, 0.03, -0.01, 0.06),
    };
    TaskEvidence {
        task_id,
        generator_count: 64,
        intervention_pair_count: 128,
        rejected_count: 0,
        rejection_reasons: Vec::new(),
        intervention_sites: vec![
            InterventionSiteSummary {
                site_id: "early-token",
                record_count: 64,
            },
            InterventionSiteSummary {
                site_id: "late-token",
                record_count: 64,
            },
        ],
        b0_mse: 1.0,
        b1_mse,
        relative_mse_reduction,
        lower_95,
        upper_95,
        verdict,
    }
}

fn fixture() -> EvidencePacket {
    EvidencePacket {
        tasks: vec![
            task_fixture("associative_recall", Verdict::Beneficial),
            task_fixture("copy", Verdict::Equivalent),
        ],
        aggregate_verdict: Verdict::Beneficial,
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
    println!("task_count={}", packet.tasks.len());
    println!("aggregate_verdict={:?}", packet.aggregate_verdict);
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
    fn confirmatory_task_set_is_exact() {
        let mut packet = fixture();
        packet.tasks.pop();
        assert_eq!(validate(&packet), Err(EvidenceError::UnexpectedTaskSet));

        let mut packet = fixture();
        packet.tasks[1].task_id = "associative_recall";
        assert_eq!(validate(&packet), Err(EvidenceError::UnexpectedTaskSet));
    }

    #[test]
    fn aggregate_verdict_is_recomputed_not_trusted() {
        let mut packet = fixture();
        packet.aggregate_verdict = Verdict::Equivalent;
        assert_eq!(validate(&packet), Err(EvidenceError::AggregateVerdictMismatch));
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
    fn multi_task_gate_matches_preregistration() {
        assert_eq!(
            combine_task_verdicts(Verdict::Beneficial, Verdict::Equivalent),
            Verdict::Beneficial
        );
        assert_eq!(
            combine_task_verdicts(Verdict::Beneficial, Verdict::Harmful),
            Verdict::Harmful
        );
        assert_eq!(
            combine_task_verdicts(Verdict::Equivalent, Verdict::Equivalent),
            Verdict::Equivalent
        );
        assert_eq!(
            combine_task_verdicts(Verdict::Equivalent, Verdict::Inconclusive),
            Verdict::Inconclusive
        );
    }

    #[test]
    fn unbalanced_intervention_counts_fail_closed() {
        let mut packet = fixture();
        packet.tasks[0].intervention_pair_count = 127;
        assert_eq!(validate(&packet), Err(EvidenceError::InterventionCountMismatch));
    }

    #[test]
    fn both_frozen_intervention_sites_are_required_and_balanced() {
        let mut packet = fixture();
        packet.tasks[0].intervention_sites[0].record_count = 63;
        assert_eq!(validate(&packet), Err(EvidenceError::InvalidInterventionSites));

        let mut packet = fixture();
        packet.tasks[0].intervention_sites[1].site_id = "other";
        assert_eq!(validate(&packet), Err(EvidenceError::InvalidInterventionSites));
    }

    #[test]
    fn rejection_ledger_must_reconcile_exactly() {
        let mut packet = fixture();
        packet.tasks[0].rejected_count = 2;
        packet.tasks[0].rejection_reasons = vec![RejectionReason {
            code: "invalid_generated_record",
            count: 1,
        }];
        assert_eq!(validate(&packet), Err(EvidenceError::RejectionLedgerMismatch));
    }

    #[test]
    fn rejection_reason_codes_must_be_unique() {
        let mut packet = fixture();
        packet.tasks[0].rejected_count = 2;
        packet.tasks[0].rejection_reasons = vec![
            RejectionReason {
                code: "invalid_generated_record",
                count: 1,
            },
            RejectionReason {
                code: "invalid_generated_record",
                count: 1,
            },
        ];
        assert_eq!(validate(&packet), Err(EvidenceError::DuplicateRejectionReason));
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
