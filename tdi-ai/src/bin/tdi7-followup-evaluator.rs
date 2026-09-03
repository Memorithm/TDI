#![forbid(unsafe_code)]

//! Development/validation evaluator primitives for TDI-7.3 through TDI-7.10.
//!
//! This binary deliberately has no final-holdout execution path. It implements
//! only protocol-defined deterministic diagnostics and fail-closed evidence
//! contracts. Confirmatory authorization remains external to this program.

use std::collections::{BTreeMap, BTreeSet};

use tdi_ai::RecoveryProfile;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tdi7Stage {
    Tdi73,
    Tdi74,
    Tdi75,
    Tdi76,
    Tdi77,
    Tdi78,
    Tdi79,
    Tdi710,
}

impl Tdi7Stage {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Tdi73 => "TDI-7.3",
            Self::Tdi74 => "TDI-7.4",
            Self::Tdi75 => "TDI-7.5",
            Self::Tdi76 => "TDI-7.6",
            Self::Tdi77 => "TDI-7.7",
            Self::Tdi78 => "TDI-7.8",
            Self::Tdi79 => "TDI-7.9",
            Self::Tdi710 => "TDI-7.10",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NonHoldoutSplit {
    Training,
    Development,
    Validation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskFamily {
    AssociativeRecall,
    Copy,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EvaluationError {
    EmptyDataset,
    EmptyEvidenceReference,
    EmptySemanticField(&'static str),
    NonFiniteValue(&'static str),
    OutOfRangeOverlap,
    ZeroDepth,
    SameInterventionSite,
    MismatchedLength,
    MismatchedDepth,
    ZeroDenominator,
    InvalidTolerance,
    DuplicateStage(Tdi7Stage),
    MissingStage(Tdi7Stage),
    InvalidSynthesisStage(Tdi7Stage),
    UnknownContradictionStage(Tdi7Stage),
}

impl core::fmt::Display for EvaluationError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyDataset => formatter.write_str("dataset must not be empty"),
            Self::EmptyEvidenceReference => formatter.write_str("evidence reference must not be empty"),
            Self::EmptySemanticField(field) => write!(formatter, "semantic field {field} must not be empty"),
            Self::NonFiniteValue(field) => write!(formatter, "{field} must be finite"),
            Self::OutOfRangeOverlap => formatter.write_str("recovery overlap must be in [0, 1]"),
            Self::ZeroDepth => formatter.write_str("evaluation depth must be positive"),
            Self::SameInterventionSite => formatter.write_str("coupled intervention sites must be distinct"),
            Self::MismatchedLength => formatter.write_str("compared vectors must have equal non-zero length"),
            Self::MismatchedDepth => formatter.write_str("recovery profiles must use identical depths"),
            Self::ZeroDenominator => formatter.write_str("transfer source score must be non-zero"),
            Self::InvalidTolerance => formatter.write_str("tolerance must be finite and non-negative"),
            Self::DuplicateStage(stage) => write!(formatter, "duplicate synthesis record for {}", stage.label()),
            Self::MissingStage(stage) => write!(formatter, "missing synthesis record for {}", stage.label()),
            Self::InvalidSynthesisStage(stage) => write!(formatter, "{} cannot be an input to its own synthesis", stage.label()),
            Self::UnknownContradictionStage(stage) => write!(formatter, "contradiction references absent stage {}", stage.label()),
        }
    }
}

impl std::error::Error for EvaluationError {}

fn checked_overlap(value: f64) -> Result<f64, EvaluationError> {
    if !value.is_finite() {
        return Err(EvaluationError::NonFiniteValue("overlap"));
    }
    if !(0.0..=1.0).contains(&value) {
        return Err(EvaluationError::OutOfRangeOverlap);
    }
    Ok(value)
}

fn checked_finite(value: f64, field: &'static str) -> Result<f64, EvaluationError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(EvaluationError::NonFiniteValue(field))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CouplingObservation {
    pub split: NonHoldoutSplit,
    pub seed: u64,
    pub task: TaskFamily,
    pub depth: usize,
    pub site_a: String,
    pub site_b: String,
    pub magnitude_a: f64,
    pub magnitude_b: f64,
    pub single_a_overlap: f64,
    pub single_b_overlap: f64,
    pub joint_overlap: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CouplingDiagnostic {
    pub single_a_deficit: f64,
    pub single_b_deficit: f64,
    pub joint_deficit: f64,
    pub excess_coupling: f64,
}

pub fn coupling_diagnostic(observation: &CouplingObservation) -> Result<CouplingDiagnostic, EvaluationError> {
    if observation.depth == 0 {
        return Err(EvaluationError::ZeroDepth);
    }
    if observation.site_a.trim().is_empty() || observation.site_b.trim().is_empty() {
        return Err(EvaluationError::EmptyEvidenceReference);
    }
    if observation.site_a == observation.site_b {
        return Err(EvaluationError::SameInterventionSite);
    }
    let magnitude_a = checked_finite(observation.magnitude_a, "magnitude_a")?;
    let magnitude_b = checked_finite(observation.magnitude_b, "magnitude_b")?;
    if magnitude_a <= 0.0 || magnitude_b <= 0.0 {
        return Err(EvaluationError::NonFiniteValue("intervention magnitude must be positive"));
    }
    let single_a = checked_overlap(observation.single_a_overlap)?;
    let single_b = checked_overlap(observation.single_b_overlap)?;
    let joint = checked_overlap(observation.joint_overlap)?;
    let single_a_deficit = 1.0 - single_a;
    let single_b_deficit = 1.0 - single_b;
    let joint_deficit = 1.0 - joint;
    Ok(CouplingDiagnostic {
        single_a_deficit,
        single_b_deficit,
        joint_deficit,
        excess_coupling: joint_deficit - single_a_deficit - single_b_deficit,
    })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CouplingSummary {
    pub observations: usize,
    pub mean_excess_coupling: f64,
    pub max_abs_excess_coupling: f64,
}

pub fn summarize_coupling(observations: &[CouplingObservation]) -> Result<CouplingSummary, EvaluationError> {
    if observations.is_empty() {
        return Err(EvaluationError::EmptyDataset);
    }
    let diagnostics = observations
        .iter()
        .map(coupling_diagnostic)
        .collect::<Result<Vec<_>, _>>()?;
    let mean = diagnostics.iter().map(|item| item.excess_coupling).sum::<f64>() / diagnostics.len() as f64;
    let max_abs = diagnostics
        .iter()
        .map(|item| item.excess_coupling.abs())
        .fold(0.0_f64, f64::max);
    Ok(CouplingSummary {
        observations: diagnostics.len(),
        mean_excess_coupling: mean,
        max_abs_excess_coupling: max_abs,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DiscreteInformationObservation {
    pub static_bin: u32,
    pub recovery_bin: u32,
    pub outcome_bin: u32,
}

fn empirical_mutual_information<X: Ord + Clone>(pairs: &[(X, u32)]) -> Result<f64, EvaluationError> {
    if pairs.is_empty() {
        return Err(EvaluationError::EmptyDataset);
    }
    let mut joint = BTreeMap::<(X, u32), usize>::new();
    let mut predictor = BTreeMap::<X, usize>::new();
    let mut outcome = BTreeMap::<u32, usize>::new();
    for (x, y) in pairs {
        *joint.entry((x.clone(), *y)).or_default() += 1;
        *predictor.entry(x.clone()).or_default() += 1;
        *outcome.entry(*y).or_default() += 1;
    }
    let n = pairs.len() as f64;
    let mut information = 0.0;
    for ((x, y), count) in joint {
        let p_xy = count as f64 / n;
        let p_x = predictor[&x] as f64 / n;
        let p_y = outcome[&y] as f64 / n;
        information += p_xy * (p_xy / (p_x * p_y)).log2();
    }
    Ok(information)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JointInformationSummary {
    pub static_information_bits: f64,
    pub recovery_information_bits: f64,
    pub joint_information_bits: f64,
    pub joint_increment_over_best_single_bits: f64,
}

pub fn joint_information_summary(records: &[DiscreteInformationObservation]) -> Result<JointInformationSummary, EvaluationError> {
    let static_pairs = records.iter().map(|r| (r.static_bin, r.outcome_bin)).collect::<Vec<_>>();
    let recovery_pairs = records.iter().map(|r| (r.recovery_bin, r.outcome_bin)).collect::<Vec<_>>();
    let joint_pairs = records
        .iter()
        .map(|r| ((r.static_bin, r.recovery_bin), r.outcome_bin))
        .collect::<Vec<_>>();
    let static_information_bits = empirical_mutual_information(&static_pairs)?;
    let recovery_information_bits = empirical_mutual_information(&recovery_pairs)?;
    let joint_information_bits = empirical_mutual_information(&joint_pairs)?;
    Ok(JointInformationSummary {
        static_information_bits,
        recovery_information_bits,
        joint_information_bits,
        joint_increment_over_best_single_bits: joint_information_bits
            - static_information_bits.max(recovery_information_bits),
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemanticSpecification {
    pub label: String,
    pub mathematical_specification_ref: String,
    pub scalar_oracle_ref: String,
    pub numerical_policy_ref: String,
    pub invariance_ref: String,
    pub failure_mode_ref: String,
}

pub fn validate_semantic_specification(specification: &SemanticSpecification) -> Result<(), EvaluationError> {
    for (name, value) in [
        ("label", specification.label.as_str()),
        ("mathematical_specification_ref", specification.mathematical_specification_ref.as_str()),
        ("scalar_oracle_ref", specification.scalar_oracle_ref.as_str()),
        ("numerical_policy_ref", specification.numerical_policy_ref.as_str()),
        ("invariance_ref", specification.invariance_ref.as_str()),
        ("failure_mode_ref", specification.failure_mode_ref.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(EvaluationError::EmptySemanticField(name));
        }
    }
    Ok(())
}

pub fn recovery_profile_distance(left: &RecoveryProfile<f64>, right: &RecoveryProfile<f64>) -> Result<f64, EvaluationError> {
    if left.is_empty() || left.horizon() != right.horizon() {
        return Err(EvaluationError::MismatchedLength);
    }
    let mut squared = 0.0;
    for (left_point, right_point) in left.points().iter().zip(right.points()) {
        if left_point.depth() != right_point.depth() {
            return Err(EvaluationError::MismatchedDepth);
        }
        let left_overlap = checked_overlap(*left_point.overlap())?;
        let right_overlap = checked_overlap(*right_point.overlap())?;
        squared += (left_overlap - right_overlap).powi(2);
    }
    Ok((squared / left.horizon() as f64).sqrt())
}

#[derive(Clone, Debug, PartialEq)]
pub struct EvidenceJustifiedComparison {
    pub motivating_evidence_ref: String,
    pub baseline_metric: f64,
    pub variant_metric: f64,
}

pub fn evidence_justified_delta(comparison: &EvidenceJustifiedComparison) -> Result<f64, EvaluationError> {
    if comparison.motivating_evidence_ref.trim().is_empty() {
        return Err(EvaluationError::EmptyEvidenceReference);
    }
    let baseline = checked_finite(comparison.baseline_metric, "baseline_metric")?;
    let variant = checked_finite(comparison.variant_metric, "variant_metric")?;
    Ok(variant - baseline)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransferSummary {
    pub forward_efficiency_ratio: f64,
    pub reverse_efficiency_ratio: f64,
    pub asymmetry_index: f64,
    pub joint_training_benefit: f64,
}

pub fn transfer_summary(
    forward_source: f64,
    forward_target: f64,
    reverse_source: f64,
    reverse_target: f64,
    joint_target: f64,
) -> Result<TransferSummary, EvaluationError> {
    let forward_source = checked_finite(forward_source, "forward_source")?;
    let forward_target = checked_finite(forward_target, "forward_target")?;
    let reverse_source = checked_finite(reverse_source, "reverse_source")?;
    let reverse_target = checked_finite(reverse_target, "reverse_target")?;
    let joint_target = checked_finite(joint_target, "joint_target")?;
    if forward_source == 0.0 || reverse_source == 0.0 {
        return Err(EvaluationError::ZeroDenominator);
    }
    let forward_efficiency_ratio = forward_target / forward_source;
    let reverse_efficiency_ratio = reverse_target / reverse_source;
    Ok(TransferSummary {
        forward_efficiency_ratio,
        reverse_efficiency_ratio,
        asymmetry_index: forward_efficiency_ratio - reverse_efficiency_ratio,
        joint_training_benefit: joint_target - forward_target.max(reverse_target),
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReexecutionVerdict {
    BitExact,
    ToleranceBound,
    Divergent,
}

pub fn compare_reexecution(left: &[f64], right: &[f64], tolerance: f64) -> Result<ReexecutionVerdict, EvaluationError> {
    if left.is_empty() || left.len() != right.len() {
        return Err(EvaluationError::MismatchedLength);
    }
    if !tolerance.is_finite() || tolerance < 0.0 {
        return Err(EvaluationError::InvalidTolerance);
    }
    let mut bit_exact = true;
    let mut tolerance_bound = true;
    for (left_value, right_value) in left.iter().zip(right) {
        let left_value = checked_finite(*left_value, "left_reexecution_value")?;
        let right_value = checked_finite(*right_value, "right_reexecution_value")?;
        bit_exact &= left_value.to_bits() == right_value.to_bits();
        tolerance_bound &= (left_value - right_value).abs() <= tolerance;
    }
    Ok(if bit_exact {
        ReexecutionVerdict::BitExact
    } else if tolerance_bound {
        ReexecutionVerdict::ToleranceBound
    } else {
        ReexecutionVerdict::Divergent
    })
}

pub fn numerical_sensitivity(left: &[f64], right: &[f64]) -> Result<f64, EvaluationError> {
    if left.is_empty() || left.len() != right.len() {
        return Err(EvaluationError::MismatchedLength);
    }
    left.iter().zip(right).try_fold(0.0_f64, |maximum, (left_value, right_value)| {
        let left_value = checked_finite(*left_value, "left_precision_value")?;
        let right_value = checked_finite(*right_value, "right_precision_value")?;
        Ok(maximum.max((left_value - right_value).abs()))
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvidenceOutcome {
    Positive,
    Negative,
    Inconclusive,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StageEvidence {
    pub stage: Tdi7Stage,
    pub evidence_ref: String,
    pub provenance_ref: String,
    pub outcome: EvidenceOutcome,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContradictionRecord {
    pub left_stage: Tdi7Stage,
    pub right_stage: Tdi7Stage,
    pub resolved: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArchiveSynthesis {
    pub positive_count: usize,
    pub negative_count: usize,
    pub inconclusive_count: usize,
    pub unresolved_contradictions: usize,
}

pub fn synthesize_archive(
    evidence: &[StageEvidence],
    contradictions: &[ContradictionRecord],
) -> Result<ArchiveSynthesis, EvaluationError> {
    const REQUIRED: [Tdi7Stage; 7] = [
        Tdi7Stage::Tdi73,
        Tdi7Stage::Tdi74,
        Tdi7Stage::Tdi75,
        Tdi7Stage::Tdi76,
        Tdi7Stage::Tdi77,
        Tdi7Stage::Tdi78,
        Tdi7Stage::Tdi79,
    ];
    let mut seen = BTreeSet::new();
    let mut positive_count = 0;
    let mut negative_count = 0;
    let mut inconclusive_count = 0;
    for record in evidence {
        if record.stage == Tdi7Stage::Tdi710 {
            return Err(EvaluationError::InvalidSynthesisStage(record.stage));
        }
        if record.evidence_ref.trim().is_empty() || record.provenance_ref.trim().is_empty() {
            return Err(EvaluationError::EmptyEvidenceReference);
        }
        if !seen.insert(record.stage) {
            return Err(EvaluationError::DuplicateStage(record.stage));
        }
        match record.outcome {
            EvidenceOutcome::Positive => positive_count += 1,
            EvidenceOutcome::Negative => negative_count += 1,
            EvidenceOutcome::Inconclusive => inconclusive_count += 1,
        }
    }
    for stage in REQUIRED {
        if !seen.contains(&stage) {
            return Err(EvaluationError::MissingStage(stage));
        }
    }
    let mut unresolved_contradictions = 0;
    for contradiction in contradictions {
        for stage in [contradiction.left_stage, contradiction.right_stage] {
            if !seen.contains(&stage) {
                return Err(EvaluationError::UnknownContradictionStage(stage));
            }
        }
        if !contradiction.resolved {
            unresolved_contradictions += 1;
        }
    }
    Ok(ArchiveSynthesis {
        positive_count,
        negative_count,
        inconclusive_count,
        unresolved_contradictions,
    })
}

fn main() {
    println!("TDI-7.3..7.10 follow-up evaluator foundation");
    println!("mode=development-validation-only");
    println!("final_holdout=NOT_AUTHORIZED");
    for stage in [
        Tdi7Stage::Tdi73,
        Tdi7Stage::Tdi74,
        Tdi7Stage::Tdi75,
        Tdi7Stage::Tdi76,
        Tdi7Stage::Tdi77,
        Tdi7Stage::Tdi78,
        Tdi7Stage::Tdi79,
        Tdi7Stage::Tdi710,
    ] {
        println!("stage={} implementation=primitive-ready confirmatory=locked", stage.label());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn coupling(joint_overlap: f64) -> CouplingObservation {
        CouplingObservation {
            split: NonHoldoutSplit::Validation,
            seed: 42,
            task: TaskFamily::AssociativeRecall,
            depth: 3,
            site_a: "early".to_owned(),
            site_b: "late".to_owned(),
            magnitude_a: 0.25,
            magnitude_b: 0.25,
            single_a_overlap: 0.9,
            single_b_overlap: 0.8,
            joint_overlap,
        }
    }

    #[test]
    fn tdi73_excess_coupling_comes_from_observed_deficits() {
        let additive = coupling_diagnostic(&coupling(0.7)).expect("valid additive fixture");
        assert!(additive.excess_coupling.abs() < 1.0e-12);
        let interacting = coupling_diagnostic(&coupling(0.6)).expect("valid interacting fixture");
        assert!((interacting.excess_coupling - 0.1).abs() < 1.0e-12);
    }

    #[test]
    fn tdi73_coupling_fails_closed_on_invalid_design() {
        let mut invalid = coupling(0.7);
        invalid.site_b = invalid.site_a.clone();
        assert_eq!(coupling_diagnostic(&invalid), Err(EvaluationError::SameInterventionSite));
        invalid.site_b = "late".to_owned();
        invalid.joint_overlap = 1.5;
        assert_eq!(coupling_diagnostic(&invalid), Err(EvaluationError::OutOfRangeOverlap));
    }

    #[test]
    fn tdi74_discrete_joint_information_detects_xor_without_proxy_scores() {
        let records = [
            DiscreteInformationObservation { static_bin: 0, recovery_bin: 0, outcome_bin: 0 },
            DiscreteInformationObservation { static_bin: 0, recovery_bin: 1, outcome_bin: 1 },
            DiscreteInformationObservation { static_bin: 1, recovery_bin: 0, outcome_bin: 1 },
            DiscreteInformationObservation { static_bin: 1, recovery_bin: 1, outcome_bin: 0 },
        ];
        let summary = joint_information_summary(&records).expect("valid contingency table");
        assert!(summary.static_information_bits.abs() < 1.0e-12);
        assert!(summary.recovery_information_bits.abs() < 1.0e-12);
        assert!((summary.joint_information_bits - 1.0).abs() < 1.0e-12);
        assert!((summary.joint_increment_over_best_single_bits - 1.0).abs() < 1.0e-12);
    }

    #[test]
    fn tdi75_semantic_admission_requires_all_protocol_artifacts() {
        let specification = SemanticSpecification {
            label: "semantic-a".to_owned(),
            mathematical_specification_ref: "spec-A".to_owned(),
            scalar_oracle_ref: "oracle-A".to_owned(),
            numerical_policy_ref: "numeric-A".to_owned(),
            invariance_ref: "invariants-A".to_owned(),
            failure_mode_ref: String::new(),
        };
        assert_eq!(
            validate_semantic_specification(&specification),
            Err(EvaluationError::EmptySemanticField("failure_mode_ref"))
        );
    }

    #[test]
    fn tdi75_recovery_distance_uses_measured_profiles() {
        let left = RecoveryProfile::from_overlaps([0.2, 0.4, 0.8]);
        let right = RecoveryProfile::from_overlaps([0.2, 0.5, 0.6]);
        let distance = recovery_profile_distance(&left, &right).expect("aligned profiles");
        assert!(distance > 0.0);
    }

    #[test]
    fn tdi76_and_tdi78_require_motivating_frozen_evidence() {
        let comparison = EvidenceJustifiedComparison {
            motivating_evidence_ref: String::new(),
            baseline_metric: 0.4,
            variant_metric: 0.5,
        };
        assert_eq!(
            evidence_justified_delta(&comparison),
            Err(EvaluationError::EmptyEvidenceReference)
        );
    }

    #[test]
    fn tdi77_transfer_metrics_are_data_derived_and_bidirectional() {
        let summary = transfer_summary(0.8, 0.6, 0.5, 0.45, 0.7).expect("valid scores");
        assert!((summary.forward_efficiency_ratio - 0.75).abs() < 1.0e-12);
        assert!((summary.reverse_efficiency_ratio - 0.9).abs() < 1.0e-12);
        assert!((summary.asymmetry_index + 0.15).abs() < 1.0e-12);
        assert!((summary.joint_training_benefit - 0.1).abs() < 1.0e-12);
    }

    #[test]
    fn tdi79_distinguishes_bit_exact_tolerance_bound_and_divergent_runs() {
        let baseline = [0.1, 0.2, 0.3];
        assert_eq!(compare_reexecution(&baseline, &baseline, 0.0).unwrap(), ReexecutionVerdict::BitExact);
        assert_eq!(
            compare_reexecution(&baseline, &[0.1, 0.200_000_1, 0.3], 1.0e-6).unwrap(),
            ReexecutionVerdict::ToleranceBound
        );
        assert_eq!(
            compare_reexecution(&baseline, &[0.1, 0.25, 0.3], 1.0e-6).unwrap(),
            ReexecutionVerdict::Divergent
        );
    }

    fn complete_evidence() -> Vec<StageEvidence> {
        [
            Tdi7Stage::Tdi73,
            Tdi7Stage::Tdi74,
            Tdi7Stage::Tdi75,
            Tdi7Stage::Tdi76,
            Tdi7Stage::Tdi77,
            Tdi7Stage::Tdi78,
            Tdi7Stage::Tdi79,
        ]
        .into_iter()
        .map(|stage| StageEvidence {
            stage,
            evidence_ref: format!("{}-evidence", stage.label()),
            provenance_ref: format!("{}-provenance", stage.label()),
            outcome: EvidenceOutcome::Inconclusive,
        })
        .collect()
    }

    #[test]
    fn tdi710_synthesis_consumes_complete_evidence_and_never_invents_stage_outcomes() {
        let evidence = complete_evidence();
        let summary = synthesize_archive(&evidence, &[]).expect("complete frozen input set");
        assert_eq!(summary.positive_count, 0);
        assert_eq!(summary.negative_count, 0);
        assert_eq!(summary.inconclusive_count, 7);
        let incomplete = &evidence[..6];
        assert_eq!(
            synthesize_archive(incomplete, &[]),
            Err(EvaluationError::MissingStage(Tdi7Stage::Tdi79))
        );
    }

    #[test]
    fn source_contains_no_final_holdout_authorization_secret() {
        let source = include_str!("tdi7-followup-evaluator.rs");
        let confirmation = ["I_ACCEPT_THE_TDI7_", "HOLDOUT_FREEZE"].concat();
        let environment = ["TDI7_CONFIRM_FINAL_", "HOLDOUT"].concat();
        assert!(!source.contains(&confirmation));
        assert!(!source.contains(&environment));
    }
}
