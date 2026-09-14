//! Deterministic provenance records for TDI-21 development evidence.
//!
//! SHA validation is syntactic only: the runner must independently bind its
//! executable, source revision, input data and configuration to the record.

use super::tdi21::{ArchitectureArm, EvidenceRecord, ROUTE_SEMANTICS};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProvenanceError {
    EmptyTaskId,
    InvalidGitShaLength,
    InvalidGitShaCharacter,
}

fn arm_label(arm: ArchitectureArm) -> &'static str {
    match arm {
        ArchitectureArm::B0AttentionReference => "B0-attention-reference",
        ArchitectureArm::B1BinaryAttentionReference => "B1-binary-attention-reference",
        ArchitectureArm::B2BooleanDirect => "B2-boolean-direct",
        ArchitectureArm::B3BooleanAssociative => "B3-boolean-associative",
        ArchitectureArm::B4BooleanAnf => "B4-boolean-anf",
        ArchitectureArm::B5BooleanRelational => "B5-boolean-relational",
    }
}

fn validate_git_sha(git_sha: &str) -> Result<(), ProvenanceError> {
    if git_sha.len() != 40 {
        return Err(ProvenanceError::InvalidGitShaLength);
    }
    if !git_sha.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(ProvenanceError::InvalidGitShaCharacter);
    }
    Ok(())
}

/// V2 hex-encodes the UTF-8 task identifier, normalizes SHA case, and records
/// the newly separated counters. V1 remains historical and must not be
/// interpreted as V2. This serializes declarations, not execution attestation.
/// Negative or structurally invalid observations remain serializable.
pub fn canonical_evidence_record(
    task_id: &str,
    git_sha: &str,
    evidence: &EvidenceRecord,
) -> Result<String, ProvenanceError> {
    if task_id.is_empty() {
        return Err(ProvenanceError::EmptyTaskId);
    }
    validate_git_sha(git_sha)?;

    let task_hex: String = task_id.bytes().map(|byte| format!("{byte:02x}")).collect();
    let git_sha = git_sha.to_ascii_lowercase();
    let manifest = evidence.manifest;
    let resources = evidence.resources;
    Ok(format!(
        "tdi21-evidence-v2;task_hex={task_hex};git_sha={git_sha};route_semantics={ROUTE_SEMANTICS};arm={};sequence_len={};state_width_bits={};memory_slots={};development_seed={};correct={};candidate_declaration_valid={};boolean_primitive_evals={};word_boolean_evals={};address_derivations={};tag_equality_checks={};anf_term_evals={};route_activations={};memory_reads={};memory_writes={};memory_misses={};memory_collisions={};memory_replacements={};pairwise_comparisons={}",
        arm_label(manifest.arm),
        manifest.sequence_len,
        manifest.state_width_bits,
        manifest.memory_slots,
        manifest.development_seed,
        evidence.correct,
        evidence.candidate_structurally_valid(),
        resources.boolean_primitive_evals,
        resources.word_boolean_evals,
        resources.address_derivations,
        resources.tag_equality_checks,
        resources.anf_term_evals,
        resources.route_activations,
        resources.memory_reads,
        resources.memory_writes,
        resources.memory_misses,
        resources.memory_collisions,
        resources.memory_replacements,
        resources.pairwise_comparisons,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::experimental::tdi21::delayed_bit_recall;

    const SHA: &str = "0123456789abcdef0123456789abcdef01234567";

    #[test]
    fn canonical_record_is_stable_and_contains_structural_guard() {
        let evidence = delayed_bit_recall::<8>(7, true);
        let first = canonical_evidence_record("delayed-bit-recall", SHA, &evidence).unwrap();
        let second = canonical_evidence_record("delayed-bit-recall", SHA, &evidence).unwrap();
        assert_eq!(first, second);
        assert!(first.contains("arm=B2-boolean-direct"));
        assert!(first.contains("pairwise_comparisons=0"));
        assert!(first.starts_with("tdi21-evidence-v2;"));
        assert!(first.contains("address_derivations=1"));
    }

    #[test]
    fn provenance_rejects_mutable_or_malformed_git_identifiers() {
        let evidence = delayed_bit_recall::<8>(7, true);
        assert_eq!(
            canonical_evidence_record("delayed-bit-recall", "main", &evidence),
            Err(ProvenanceError::InvalidGitShaLength)
        );
        assert_eq!(
            canonical_evidence_record(
                "delayed-bit-recall",
                "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz",
                &evidence,
            ),
            Err(ProvenanceError::InvalidGitShaCharacter)
        );
    }

    #[test]
    fn provenance_requires_explicit_task_identity() {
        let evidence = delayed_bit_recall::<8>(7, true);
        assert_eq!(
            canonical_evidence_record("", SHA, &evidence),
            Err(ProvenanceError::EmptyTaskId)
        );
    }

    #[test]
    fn task_identifiers_cannot_inject_fields_or_records() {
        let evidence = delayed_bit_recall::<8>(7, true);
        let record = canonical_evidence_record("x;correct=false\n\r=é", SHA, &evidence).unwrap();
        assert!(!record.contains('\n'));
        assert!(!record.contains('\r'));
        assert_eq!(record.matches(";correct=").count(), 1);
        assert!(record.contains(";task_hex=783b636f72726563743d66616c73650a0d3dc3a9;"));
    }

    #[test]
    fn sha_case_is_canonical_and_negative_evidence_is_retained() {
        let evidence = delayed_bit_recall::<0>(7, true);
        let lower = canonical_evidence_record("zero-capacity", SHA, &evidence).unwrap();
        let upper =
            canonical_evidence_record("zero-capacity", &SHA.to_uppercase(), &evidence).unwrap();
        assert_eq!(lower, upper);
        assert!(lower.contains(";correct=false;candidate_declaration_valid=false;"));
    }
}
