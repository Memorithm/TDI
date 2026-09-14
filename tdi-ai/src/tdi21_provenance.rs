//! Deterministic provenance records for TDI-21 development evidence.

use super::tdi21::{ArchitectureArm, EvidenceRecord};

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

/// Produce a stable text record suitable for hashing or storing in a result
/// artifact. The caller must provide the exact 40-hex commit SHA; TDI-21 does
/// not silently substitute a branch name or mutable ref.
pub fn canonical_evidence_record(
    task_id: &str,
    git_sha: &str,
    evidence: &EvidenceRecord,
) -> Result<String, ProvenanceError> {
    if task_id.is_empty() {
        return Err(ProvenanceError::EmptyTaskId);
    }
    validate_git_sha(git_sha)?;

    let manifest = evidence.manifest;
    let resources = evidence.resources;
    Ok(format!(
        "tdi21-evidence-v1;task={task_id};git_sha={git_sha};arm={};sequence_len={};state_width_bits={};memory_slots={};development_seed={};correct={};boolean_primitive_evals={};route_activations={};memory_reads={};memory_writes={};memory_misses={};memory_collisions={};memory_replacements={};pairwise_comparisons={}",
        arm_label(manifest.arm),
        manifest.sequence_len,
        manifest.state_width_bits,
        manifest.memory_slots,
        manifest.development_seed,
        evidence.correct,
        resources.boolean_primitive_evals,
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
}
