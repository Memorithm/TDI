//! Deterministic observation-only numeric threshold candidates for TDI-2.2.
//!
//! Candidate thresholds are taken only from finite values already present in a
//! validated `InductionBatch`. No expected labels, outcomes or evaluator state
//! participate in generation.

use std::collections::BTreeMap;

use super::tdi2_induction_input::InductionBatch;
use super::tdi2_predicate_candidates::{
    CandidateProvenance, CandidateProvenanceError, CandidateSource, PredicateCandidateFamily,
};

/// Maximum numeric feature width accepted by this bounded generator.
pub const MAX_NUMERIC_FEATURES: usize = 128;
/// Maximum number of emitted directional threshold candidates.
pub const MAX_SCALAR_THRESHOLD_CANDIDATES: usize = 4096;
/// Maximum raw candidate-source references accepted before provenance de-duplication.
///
/// This is an engineering memory-safety bound, independent of the unique-candidate
/// limit: repeated observations of the same value must not grow temporary provenance
/// vectors without bound.
pub const MAX_SCALAR_THRESHOLD_SOURCE_REFERENCES: usize = 65_536;

/// Direction of a scalar threshold predicate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ThresholdDirection {
    LessEqual,
    GreaterEqual,
}

impl ThresholdDirection {
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::LessEqual => "le",
            Self::GreaterEqual => "ge",
        }
    }
}

/// One canonical scalar threshold candidate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NumericThresholdCandidate {
    feature_index: u32,
    threshold_bits: u64,
    direction: ThresholdDirection,
    provenance: CandidateProvenance,
}

impl NumericThresholdCandidate {
    #[must_use]
    pub const fn feature_index(&self) -> u32 {
        self.feature_index
    }

    #[must_use]
    pub const fn threshold_bits(&self) -> u64 {
        self.threshold_bits
    }

    #[must_use]
    pub const fn threshold(&self) -> f64 {
        f64::from_bits(self.threshold_bits)
    }

    #[must_use]
    pub const fn direction(&self) -> ThresholdDirection {
        self.direction
    }

    #[must_use]
    pub const fn provenance(&self) -> &CandidateProvenance {
        &self.provenance
    }

    /// Evaluate this candidate against one observed scalar value.
    #[must_use]
    pub fn evaluate(&self, value: f64) -> Option<bool> {
        value.is_finite().then(|| match self.direction {
            ThresholdDirection::LessEqual => value <= self.threshold(),
            ThresholdDirection::GreaterEqual => value >= self.threshold(),
        })
    }
}

/// Bounded generation failures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumericThresholdError {
    FeatureWidthExceeded { actual: usize, maximum: usize },
    CandidateLimitExceeded { maximum: usize },
    SourceReferenceLimitExceeded { maximum: usize },
    Provenance(CandidateProvenanceError),
}

impl From<CandidateProvenanceError> for NumericThresholdError {
    fn from(error: CandidateProvenanceError) -> Self {
        Self::Provenance(error)
    }
}

fn canonical_bits(value: f64) -> u64 {
    if value == 0.0 {
        0.0_f64.to_bits()
    } else {
        value.to_bits()
    }
}

/// Generate deterministic `<=` and `>=` candidates from observed numeric values.
///
/// Exact repeated values are deduplicated while all originating observation
/// sources remain in provenance. Candidate order depends only on canonical keys,
/// not on episode order inside the batch.
pub fn generate_numeric_threshold_candidates(
    batch: &InductionBatch,
) -> Result<Vec<NumericThresholdCandidate>, NumericThresholdError> {
    type Key = (u32, u64, ThresholdDirection);

    // Preflight the complete raw source-reference budget before allocating any
    // per-candidate provenance vectors. Repeated values can otherwise keep the
    // unique-candidate count small while growing those vectors without bound.
    let mut source_references = 0usize;
    for episode in batch.episodes() {
        for frame in episode.frames() {
            let width = frame.numeric().len();
            if width > MAX_NUMERIC_FEATURES {
                return Err(NumericThresholdError::FeatureWidthExceeded {
                    actual: width,
                    maximum: MAX_NUMERIC_FEATURES,
                });
            }
            let additional = width.checked_mul(2).ok_or(
                NumericThresholdError::SourceReferenceLimitExceeded {
                    maximum: MAX_SCALAR_THRESHOLD_SOURCE_REFERENCES,
                },
            )?;
            source_references = source_references.checked_add(additional).ok_or(
                NumericThresholdError::SourceReferenceLimitExceeded {
                    maximum: MAX_SCALAR_THRESHOLD_SOURCE_REFERENCES,
                },
            )?;
            if source_references > MAX_SCALAR_THRESHOLD_SOURCE_REFERENCES {
                return Err(NumericThresholdError::SourceReferenceLimitExceeded {
                    maximum: MAX_SCALAR_THRESHOLD_SOURCE_REFERENCES,
                });
            }
        }
    }

    let mut sources_by_candidate: BTreeMap<Key, Vec<CandidateSource>> = BTreeMap::new();
    for episode in batch.episodes() {
        for frame in episode.frames() {
            let values = frame.numeric().values();
            for (feature_index, &value) in values.iter().enumerate() {
                let feature_index = u32::try_from(feature_index).map_err(|_| {
                    NumericThresholdError::FeatureWidthExceeded {
                        actual: values.len(),
                        maximum: MAX_NUMERIC_FEATURES,
                    }
                })?;
                let source = CandidateSource::new(episode.id(), frame.ordinal());
                let bits = canonical_bits(value);
                for direction in [
                    ThresholdDirection::LessEqual,
                    ThresholdDirection::GreaterEqual,
                ] {
                    sources_by_candidate
                        .entry((feature_index, bits, direction))
                        .or_default()
                        .push(source);
                    if sources_by_candidate.len() > MAX_SCALAR_THRESHOLD_CANDIDATES {
                        return Err(NumericThresholdError::CandidateLimitExceeded {
                            maximum: MAX_SCALAR_THRESHOLD_CANDIDATES,
                        });
                    }
                }
            }
        }
    }

    sources_by_candidate
        .into_iter()
        .map(|((feature_index, threshold_bits, direction), sources)| {
            let provenance = CandidateProvenance::new_in_batch(
                PredicateCandidateFamily::ScalarThreshold,
                sources,
                batch,
            )?;
            Ok(NumericThresholdCandidate {
                feature_index,
                threshold_bits,
                direction,
                provenance,
            })
        })
        .collect()
}

impl core::fmt::Display for NumericThresholdError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::FeatureWidthExceeded { actual, maximum } => write!(
                formatter,
                "numeric feature width {actual} exceeds bounded maximum {maximum}"
            ),
            Self::CandidateLimitExceeded { maximum } => write!(
                formatter,
                "numeric threshold candidate count exceeds bounded maximum {maximum}"
            ),
            Self::SourceReferenceLimitExceeded { maximum } => write!(
                formatter,
                "numeric threshold source-reference count exceeds bounded maximum {maximum}"
            ),
            Self::Provenance(error) => write!(formatter, "invalid threshold provenance: {error}"),
        }
    }
}

impl std::error::Error for NumericThresholdError {}

#[cfg(test)]
mod tests {
    use super::{
        MAX_NUMERIC_FEATURES, MAX_SCALAR_THRESHOLD_SOURCE_REFERENCES, NumericThresholdError,
        ThresholdDirection, generate_numeric_threshold_candidates,
    };
    use crate::experimental::tdi2_induction_input::InductionBatch;
    use crate::experimental::tdi2_induction_split::{DEVELOPMENT_START, InductionDomain};
    use crate::experimental::tdi2_intuition::{BooleanState, NumericState};
    use crate::experimental::tdi2_observation_graph::ObservationGraph;
    use crate::experimental::tdi2_template_induction::{
        EpisodeId, ExperienceEpisode, ObservationFrame,
    };

    fn episode(id: u64, values: Vec<f64>) -> ExperienceEpisode {
        ExperienceEpisode::new(
            EpisodeId::new(id),
            vec![ObservationFrame::new(
                0,
                NumericState::new(values).expect("finite"),
                BooleanState::default(),
            )],
        )
        .expect("episode")
    }

    fn graph(id: u64) -> ObservationGraph {
        ObservationGraph::new(EpisodeId::new(id), Vec::new(), Vec::new()).expect("graph")
    }

    fn batch(order: [u64; 2]) -> InductionBatch {
        InductionBatch::new(
            InductionDomain::Development,
            order
                .iter()
                .map(|&id| episode(id, vec![(id - DEVELOPMENT_START) as f64]))
                .collect(),
            order.iter().map(|&id| graph(id)).collect(),
        )
        .expect("batch")
    }

    #[test]
    fn generation_is_label_free_deterministic_and_order_independent() {
        let first = DEVELOPMENT_START;
        let second = DEVELOPMENT_START + 1;
        let forward =
            generate_numeric_threshold_candidates(&batch([first, second])).expect("candidates");
        let reverse =
            generate_numeric_threshold_candidates(&batch([second, first])).expect("candidates");
        assert_eq!(forward, reverse);
        assert_eq!(forward.len(), 4);
        assert!(
            forward
                .iter()
                .all(|candidate| candidate.feature_index() == 0)
        );
    }

    #[test]
    fn repeated_value_is_one_threshold_identity_with_complete_provenance() {
        let first = DEVELOPMENT_START;
        let second = DEVELOPMENT_START + 1;
        let batch = InductionBatch::new(
            InductionDomain::Development,
            vec![episode(first, vec![3.0]), episode(second, vec![3.0])],
            vec![graph(first), graph(second)],
        )
        .expect("batch");
        let candidates = generate_numeric_threshold_candidates(&batch).expect("candidates");
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].provenance().sources().len(), 2);
        assert_eq!(candidates[1].provenance().sources().len(), 2);
        assert_eq!(candidates[0].threshold(), 3.0);
    }

    #[test]
    fn predicate_evaluation_rejects_nonfinite_inputs() {
        let candidates = generate_numeric_threshold_candidates(&batch([
            DEVELOPMENT_START,
            DEVELOPMENT_START + 1,
        ]))
        .expect("candidates");
        let le = candidates
            .iter()
            .find(|candidate| {
                candidate.threshold() == 0.0
                    && candidate.direction() == ThresholdDirection::LessEqual
            })
            .expect("zero threshold");
        assert_eq!(le.evaluate(-1.0), Some(true));
        assert_eq!(le.evaluate(f64::NAN), None);
    }

    #[test]
    fn repeated_observations_cannot_bypass_source_reference_bound() {
        let id = DEVELOPMENT_START;
        let references_per_frame = MAX_NUMERIC_FEATURES * 2;
        let frame_count = MAX_SCALAR_THRESHOLD_SOURCE_REFERENCES / references_per_frame + 1;
        let frames = (0..frame_count)
            .map(|ordinal| {
                ObservationFrame::new(
                    u32::try_from(ordinal).expect("bounded ordinal"),
                    NumericState::new(vec![1.0; MAX_NUMERIC_FEATURES]).expect("finite"),
                    BooleanState::default(),
                )
            })
            .collect();
        let episode = ExperienceEpisode::new(EpisodeId::new(id), frames).expect("episode");
        let batch =
            InductionBatch::new(InductionDomain::Development, vec![episode], vec![graph(id)])
                .expect("batch");

        assert_eq!(
            generate_numeric_threshold_candidates(&batch),
            Err(NumericThresholdError::SourceReferenceLimitExceeded {
                maximum: MAX_SCALAR_THRESHOLD_SOURCE_REFERENCES,
            })
        );
    }

    #[test]
    fn excessive_feature_width_fails_closed() {
        let id = DEVELOPMENT_START;
        let batch = InductionBatch::new(
            InductionDomain::Development,
            vec![episode(id, vec![0.0; MAX_NUMERIC_FEATURES + 1])],
            vec![graph(id)],
        )
        .expect("batch");
        assert_eq!(
            generate_numeric_threshold_candidates(&batch),
            Err(NumericThresholdError::FeatureWidthExceeded {
                actual: MAX_NUMERIC_FEATURES + 1,
                maximum: MAX_NUMERIC_FEATURES,
            })
        );
    }
}
