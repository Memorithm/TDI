//! Canonical observation/output provenance for TDI-2.2 induction runs.

use core::fmt::Write as _;

use super::tdi2_induction_input::InductionBatch;
use super::tdi2_induction_split::InductionDomain;

/// Provenance construction failures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InductionProvenanceError {
    /// Algorithm identity must be explicit.
    EmptyAlgorithmIdentity,
    /// Algorithm output must have a declared canonical representation.
    EmptyOutputRecord,
}

fn append_hex(output: &mut String, bytes: &[u8]) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in bytes {
        output.push(char::from(HEX[(byte >> 4) as usize]));
        output.push(char::from(HEX[(byte & 0x0f) as usize]));
    }
}

/// Canonical, label-free representation of the exact induction input.
#[must_use]
pub fn canonical_batch_record(batch: &InductionBatch) -> String {
    let domain = match batch.domain() {
        InductionDomain::Development => "development",
        InductionDomain::Validation => "validation",
    };
    let mut record = format!("tdi2.2-induction-batch-v2;domain={domain};");
    for (episode, graph) in batch.episodes().iter().zip(batch.graphs()) {
        let _ = write!(record, "episode={}[", episode.id().raw());
        for frame in episode.frames() {
            let _ = write!(record, "frame={}:n=", frame.ordinal());
            for value in frame.numeric().values() {
                let _ = write!(record, "{:016x},", value.to_bits());
            }
            record.push_str(":p=");
            for predicate in frame.observed_predicates().predicates() {
                let _ = write!(record, "{},", predicate.raw());
            }
            record.push(';');
        }
        record.push_str("]graph=[");
        for entity in graph.entities() {
            let _ = write!(record, "entity={}:p=", entity.id().raw());
            for predicate in entity.predicates().predicates() {
                let _ = write!(record, "{},", predicate.raw());
            }
            record.push(';');
        }
        for relation in graph.relations() {
            let _ = write!(
                record,
                "rel={},{},{};",
                relation.left().raw(),
                relation.relation().raw(),
                relation.right().raw()
            );
        }
        record.push_str("];|");
    }
    record
}

/// Canonical binding between observation-only input and one declared induction output.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InductionProvenance {
    canonical: String,
}

impl InductionProvenance {
    /// Bind a batch, algorithm identity and algorithm-specific canonical output.
    pub fn new(
        batch: &InductionBatch,
        algorithm_identity: &str,
        output_record: &str,
    ) -> Result<Self, InductionProvenanceError> {
        if algorithm_identity.is_empty() {
            return Err(InductionProvenanceError::EmptyAlgorithmIdentity);
        }
        if output_record.is_empty() {
            return Err(InductionProvenanceError::EmptyOutputRecord);
        }
        let mut canonical = canonical_batch_record(batch);
        canonical.push_str("algorithm_hex=");
        append_hex(&mut canonical, algorithm_identity.as_bytes());
        canonical.push_str(";output_hex=");
        append_hex(&mut canonical, output_record.as_bytes());
        canonical.push(';');
        Ok(Self { canonical })
    }

    /// Exact canonical record. No evaluator label is appended by this type.
    #[must_use]
    pub fn canonical_record(&self) -> &str {
        &self.canonical
    }
}

impl core::fmt::Display for InductionProvenanceError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyAlgorithmIdentity => {
                formatter.write_str("induction algorithm identity is empty")
            }
            Self::EmptyOutputRecord => formatter.write_str("induction output record is empty"),
        }
    }
}

impl std::error::Error for InductionProvenanceError {}

#[cfg(test)]
mod tests {
    use super::{InductionProvenance, canonical_batch_record};
    use crate::experimental::tdi2_induction_input::InductionBatch;
    use crate::experimental::tdi2_induction_split::{DEVELOPMENT_START, InductionDomain};
    use crate::experimental::tdi2_intuition::{BooleanState, NumericState, PredicateId};
    use crate::experimental::tdi2_observation_graph::{
        ObservationGraph, ObservedEntity, ObservedEntityId,
    };
    use crate::experimental::tdi2_template_induction::{
        EpisodeId, ExperienceEpisode, ObservationFrame,
    };

    fn batch(value: f64) -> InductionBatch {
        let id = DEVELOPMENT_START;
        let episode = ExperienceEpisode::new(
            EpisodeId::new(id),
            vec![ObservationFrame::new(
                0,
                NumericState::new(vec![value]).expect("finite"),
                BooleanState::new(vec![PredicateId::new(5)]),
            )],
        )
        .expect("episode");
        let graph = ObservationGraph::new(
            EpisodeId::new(id),
            vec![ObservedEntity::new(
                ObservedEntityId::new(9),
                BooleanState::new(vec![PredicateId::new(7)]),
            )],
            Vec::new(),
        )
        .expect("graph");
        InductionBatch::new(InductionDomain::Development, vec![episode], vec![graph])
            .expect("batch")
    }

    #[test]
    fn canonical_input_binds_exact_numeric_bits_and_observed_structure() {
        let first = canonical_batch_record(&batch(1.0));
        let second = canonical_batch_record(&batch(1.5));
        assert_ne!(first, second);
        assert!(first.contains("3ff0000000000000"));
        assert!(first.contains("entity=9:p=7,"));
    }

    #[test]
    fn provenance_binds_algorithm_and_output_without_target_label_field() {
        let provenance =
            InductionProvenance::new(&batch(2.0), "anti-unify-v1", "term(X)").expect("provenance");
        let record = provenance.canonical_record();
        assert!(record.contains("algorithm_hex="));
        assert!(record.contains("output_hex="));
        assert!(!record.contains("expected_label"));
    }
}
