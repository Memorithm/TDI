//! TDI-25 Phase-B balanced task generators.
//!
//! Slice 11 introduces deterministic torsor-favorable transport tasks while
//! keeping the oracle separate from inference inputs.
//! Slice 12 adds chiral-favorable mirrored handedness pairs with the same
//! oracle/input separation.
//! Slice 13 adds mixed geometry tasks that require both a transported
//! relation and a parity-sensitive relation.

use core::fmt;

use super::tdi22_torsor::{Torsor3, Twist3, Vec3};
use super::tdi24_chiral::{Chiral6, ChiralScoreWeights};
use super::tdi25_torsor_chiral::{TaskFamily, Tdi25Error, chiral_arm_score, torsor_arm_score};

/// Versioned TDI-25 torsor-favorable transport task contract.
pub const TORSOR_TRANSPORT_TASK_CONTRACT: &str = "tdi25-torsor-transport-task-v1";

/// Versioned TDI-25 chiral-favorable reflection task contract.
pub const CHIRAL_REFLECTION_TASK_CONTRACT: &str = "tdi25-chiral-reflection-task-v1";

/// Versioned TDI-25 mixed-geometry task contract.
pub const MIXED_GEOMETRY_TASK_CONTRACT: &str = "tdi25-mixed-geometry-task-v1";

/// Inference-visible input for one torsor transport case.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TorsorTransportInput {
    pub case_id: u64,
    pub query: Twist3,
    pub key: Torsor3,
    pub query_position: Vec3,
    pub task_family: TaskFamily,
    pub generator_contract: &'static str,
}

/// Oracle retained separately from inference input.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TorsorTransportOracle {
    pub pair_id: u64,
    pub expected_score: f64,
    pub generator_contract: &'static str,
}

/// Two reductions of the same physical torsor with one shared oracle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TorsorTransportPair {
    pub original: TorsorTransportInput,
    pub transported: TorsorTransportInput,
    pub oracle: TorsorTransportOracle,
}

/// Deterministically generate one transport-invariance task.
pub fn torsor_transport_pair(pair_id: u64) -> Result<TorsorTransportPair, Tdi25TaskError> {
    let original_id = pair_id
        .checked_mul(2)
        .ok_or(Tdi25TaskError::CaseIdOverflow)?;
    let transported_id = original_id
        .checked_add(1)
        .ok_or(Tdi25TaskError::CaseIdOverflow)?;

    let offset = ((pair_id % 41) as f64 + 1.0) / 128.0;
    let v = |x, y, z| {
        Vec3::new(x, y, z).map_err(|error| Tdi25TaskError::Bridge(Tdi25Error::Torsor(error)))
    };

    let resultant = v(2.0 + offset, 3.0, -4.0)?;
    let moment = v(-5.0, 7.0 + offset, 11.0)?;
    let reference = v(13.0, -17.0, 19.0 + offset)?;
    let target_reference = v(-2.0 - offset, 5.0, 7.0)?;
    let query_position = v(-23.0, 29.0 + offset, 31.0)?;
    let query = Twist3::new(v(1.0, -2.0, 3.0 + offset)?, v(0.5 + offset, 4.0, -1.0)?)
        .map_err(|error| Tdi25TaskError::Bridge(Tdi25Error::Torsor(error)))?;

    let original_key = Torsor3::new(resultant, moment, reference)
        .map_err(|error| Tdi25TaskError::Bridge(Tdi25Error::Torsor(error)))?;
    let transported_key = original_key
        .transport(target_reference)
        .map_err(|error| Tdi25TaskError::Bridge(Tdi25Error::Torsor(error)))?;
    let expected_score =
        torsor_arm_score(query, original_key, query_position).map_err(Tdi25TaskError::Bridge)?;

    let make_input = |case_id, key| TorsorTransportInput {
        case_id,
        query,
        key,
        query_position,
        task_family: TaskFamily::TorsorFavorable,
        generator_contract: TORSOR_TRANSPORT_TASK_CONTRACT,
    };

    Ok(TorsorTransportPair {
        original: make_input(original_id, original_key),
        transported: make_input(transported_id, transported_key),
        oracle: TorsorTransportOracle {
            pair_id,
            expected_score,
            generator_contract: TORSOR_TRANSPORT_TASK_CONTRACT,
        },
    })
}

/// Handedness oracle for one member of a mirrored chiral pair.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HandednessTarget {
    /// Canonical orientation emitted first by the deterministic generator.
    Right,
    /// Simultaneously mirrored partner.
    Left,
}

/// Inference-visible input for one chiral reflection case.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChiralReflectionInput {
    pub case_id: u64,
    pub query: Chiral6,
    pub key: Chiral6,
    pub weights: ChiralScoreWeights,
    pub task_family: TaskFamily,
    pub generator_contract: &'static str,
}

/// Handedness/score oracle retained separately from inference input.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChiralReflectionOracle {
    pub pair_id: u64,
    pub handedness: HandednessTarget,
    pub expected_score: f64,
    pub generator_contract: &'static str,
}

/// Mirrored handedness pair with per-member oracles kept outside inputs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChiralReflectionPair {
    pub right: ChiralReflectionInput,
    pub left: ChiralReflectionInput,
    pub right_oracle: ChiralReflectionOracle,
    pub left_oracle: ChiralReflectionOracle,
}

/// Deterministically generate one chiral-favorable reflection pair.
pub fn chiral_reflection_pair(pair_id: u64) -> Result<ChiralReflectionPair, Tdi25TaskError> {
    let right_id = pair_id
        .checked_mul(2)
        .ok_or(Tdi25TaskError::CaseIdOverflow)?;
    let left_id = right_id
        .checked_add(1)
        .ok_or(Tdi25TaskError::CaseIdOverflow)?;

    // Bounded offset keeps every Stage-A scalar finite and model-independent.
    let offset = ((pair_id % 29) as f64 + 1.0) / 64.0;
    // gamma != 0 so the parity-odd channel makes handedness target-relevant.
    let weights = ChiralScoreWeights::new(0.7, -0.2, 1.3)
        .map_err(|error| Tdi25TaskError::Bridge(Tdi25Error::Chiral(error)))?;
    let query = Chiral6::new([1.0 + offset, -0.75, 0.5], [0.625, -1.0 - offset, 1.5])
        .map_err(|error| Tdi25TaskError::Bridge(Tdi25Error::Chiral(error)))?;
    let key = Chiral6::new([-0.5, 1.25 + offset, -1.0], [1.75 + offset, 0.375, -0.875])
        .map_err(|error| Tdi25TaskError::Bridge(Tdi25Error::Chiral(error)))?;

    let right_score = chiral_arm_score(query, key, weights).map_err(Tdi25TaskError::Bridge)?;
    let left_query = query.mirror();
    let left_key = key.mirror();
    let left_score =
        chiral_arm_score(left_query, left_key, weights).map_err(Tdi25TaskError::Bridge)?;

    let make_input = |case_id, query, key| ChiralReflectionInput {
        case_id,
        query,
        key,
        weights,
        task_family: TaskFamily::ChiralFavorable,
        generator_contract: CHIRAL_REFLECTION_TASK_CONTRACT,
    };

    Ok(ChiralReflectionPair {
        right: make_input(right_id, query, key),
        left: make_input(left_id, left_query, left_key),
        right_oracle: ChiralReflectionOracle {
            pair_id,
            handedness: HandednessTarget::Right,
            expected_score: right_score,
            generator_contract: CHIRAL_REFLECTION_TASK_CONTRACT,
        },
        left_oracle: ChiralReflectionOracle {
            pair_id,
            handedness: HandednessTarget::Left,
            expected_score: left_score,
            generator_contract: CHIRAL_REFLECTION_TASK_CONTRACT,
        },
    })
}

/// Inference-visible input combining torsor transport and chiral reflection.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MixedGeometryInput {
    pub case_id: u64,
    pub torsor_query: Twist3,
    pub torsor_key: Torsor3,
    pub query_position: Vec3,
    pub chiral_query: Chiral6,
    pub chiral_key: Chiral6,
    pub weights: ChiralScoreWeights,
    pub task_family: TaskFamily,
    pub generator_contract: &'static str,
}

/// Oracle requiring both transport invariance and parity-sensitive handedness.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MixedGeometryOracle {
    pub pair_id: u64,
    pub handedness: HandednessTarget,
    pub expected_torsor_score: f64,
    pub expected_chiral_score: f64,
    pub generator_contract: &'static str,
}

/// Base vs jointly-transformed mixed geometry pair.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MixedGeometryPair {
    pub base: MixedGeometryInput,
    pub transformed: MixedGeometryInput,
    pub base_oracle: MixedGeometryOracle,
    pub transformed_oracle: MixedGeometryOracle,
}

/// Deterministically generate one mixed geometry task.
///
/// The transformed member applies torsor reduction-point transport and
/// simultaneous chiral mirror reflection together. The oracle demands the
/// shared torsor score (transport-invariant) and opposite handedness with a
/// parity-odd chiral score (reflection-sensitive).
pub fn mixed_geometry_pair(pair_id: u64) -> Result<MixedGeometryPair, Tdi25TaskError> {
    let base_id = pair_id
        .checked_mul(2)
        .ok_or(Tdi25TaskError::CaseIdOverflow)?;
    let transformed_id = base_id
        .checked_add(1)
        .ok_or(Tdi25TaskError::CaseIdOverflow)?;

    let offset = ((pair_id % 37) as f64 + 1.0) / 96.0;
    let v = |x, y, z| {
        Vec3::new(x, y, z).map_err(|error| Tdi25TaskError::Bridge(Tdi25Error::Torsor(error)))
    };

    let resultant = v(2.0 + offset, 3.0, -4.0)?;
    let moment = v(-5.0, 7.0 + offset, 11.0)?;
    let reference = v(13.0, -17.0, 19.0 + offset)?;
    let target_reference = v(-2.0 - offset, 5.0, 7.0)?;
    let query_position = v(-23.0, 29.0 + offset, 31.0)?;
    let torsor_query = Twist3::new(v(1.0, -2.0, 3.0 + offset)?, v(0.5 + offset, 4.0, -1.0)?)
        .map_err(|error| Tdi25TaskError::Bridge(Tdi25Error::Torsor(error)))?;
    let original_key = Torsor3::new(resultant, moment, reference)
        .map_err(|error| Tdi25TaskError::Bridge(Tdi25Error::Torsor(error)))?;
    let transported_key = original_key
        .transport(target_reference)
        .map_err(|error| Tdi25TaskError::Bridge(Tdi25Error::Torsor(error)))?;
    let expected_torsor_score = torsor_arm_score(torsor_query, original_key, query_position)
        .map_err(Tdi25TaskError::Bridge)?;

    let weights = ChiralScoreWeights::new(0.7, -0.2, 1.3)
        .map_err(|error| Tdi25TaskError::Bridge(Tdi25Error::Chiral(error)))?;
    let chiral_query = Chiral6::new([1.0 + offset, -0.75, 0.5], [0.625, -1.0 - offset, 1.5])
        .map_err(|error| Tdi25TaskError::Bridge(Tdi25Error::Chiral(error)))?;
    let chiral_key = Chiral6::new([-0.5, 1.25 + offset, -1.0], [1.75 + offset, 0.375, -0.875])
        .map_err(|error| Tdi25TaskError::Bridge(Tdi25Error::Chiral(error)))?;
    let base_chiral_score =
        chiral_arm_score(chiral_query, chiral_key, weights).map_err(Tdi25TaskError::Bridge)?;
    let mirrored_query = chiral_query.mirror();
    let mirrored_key = chiral_key.mirror();
    let transformed_chiral_score =
        chiral_arm_score(mirrored_query, mirrored_key, weights).map_err(Tdi25TaskError::Bridge)?;

    let make_input = |case_id, torsor_key, chiral_query, chiral_key| MixedGeometryInput {
        case_id,
        torsor_query,
        torsor_key,
        query_position,
        chiral_query,
        chiral_key,
        weights,
        task_family: TaskFamily::Mixed,
        generator_contract: MIXED_GEOMETRY_TASK_CONTRACT,
    };

    Ok(MixedGeometryPair {
        base: make_input(base_id, original_key, chiral_query, chiral_key),
        transformed: make_input(
            transformed_id,
            transported_key,
            mirrored_query,
            mirrored_key,
        ),
        base_oracle: MixedGeometryOracle {
            pair_id,
            handedness: HandednessTarget::Right,
            expected_torsor_score,
            expected_chiral_score: base_chiral_score,
            generator_contract: MIXED_GEOMETRY_TASK_CONTRACT,
        },
        transformed_oracle: MixedGeometryOracle {
            pair_id,
            handedness: HandednessTarget::Left,
            expected_torsor_score,
            expected_chiral_score: transformed_chiral_score,
            generator_contract: MIXED_GEOMETRY_TASK_CONTRACT,
        },
    })
}

/// TDI-25 task-generation failures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tdi25TaskError {
    CaseIdOverflow,
    Bridge(Tdi25Error),
}

impl fmt::Display for Tdi25TaskError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CaseIdOverflow => formatter.write_str("TDI-25 task case id overflow"),
            Self::Bridge(error) => write!(formatter, "TDI-25 bridge rejected task: {error}"),
        }
    }
}

impl std::error::Error for Tdi25TaskError {}

#[cfg(test)]
mod tests {
    use super::super::tdi24_chiral::observables;
    use super::*;

    fn close(lhs: f64, rhs: f64) {
        let scale = 1.0_f64.max(lhs.abs()).max(rhs.abs());
        assert!((lhs - rhs).abs() <= 128.0 * f64::EPSILON * scale);
    }

    #[test]
    fn transport_pairs_are_deterministic_and_keep_oracle_outside_inputs() {
        let pair = torsor_transport_pair(9).unwrap();
        assert_eq!(pair, torsor_transport_pair(9).unwrap());
        assert_ne!(pair.original.case_id, pair.transported.case_id);
        assert_eq!(pair.original.task_family, TaskFamily::TorsorFavorable);
        assert_eq!(pair.transported.task_family, TaskFamily::TorsorFavorable);
        assert_eq!(
            pair.oracle.generator_contract,
            TORSOR_TRANSPORT_TASK_CONTRACT
        );
    }

    #[test]
    fn alternate_reduction_preserves_physical_invariants_and_oracle_score() {
        for pair_id in 0..32 {
            let pair = torsor_transport_pair(pair_id).unwrap();
            assert_eq!(
                pair.original.key.resultant(),
                pair.transported.key.resultant()
            );
            assert_ne!(
                pair.original.key.reference(),
                pair.transported.key.reference()
            );

            let original_score = torsor_arm_score(
                pair.original.query,
                pair.original.key,
                pair.original.query_position,
            )
            .unwrap();
            let transported_score = torsor_arm_score(
                pair.transported.query,
                pair.transported.key,
                pair.transported.query_position,
            )
            .unwrap();
            close(original_score, pair.oracle.expected_score);
            close(transported_score, pair.oracle.expected_score);

            let original_origin = pair.original.key.origin_moment().unwrap();
            let transported_origin = pair.transported.key.origin_moment().unwrap();
            close(original_origin.x, transported_origin.x);
            close(original_origin.y, transported_origin.y);
            close(original_origin.z, transported_origin.z);
        }
    }

    #[test]
    fn torsor_case_id_overflow_fails_closed() {
        assert_eq!(
            torsor_transport_pair(u64::MAX),
            Err(Tdi25TaskError::CaseIdOverflow)
        );
    }

    #[test]
    fn reflection_pairs_are_deterministic_and_keep_oracle_outside_inputs() {
        let pair = chiral_reflection_pair(11).unwrap();
        assert_eq!(pair, chiral_reflection_pair(11).unwrap());
        assert_ne!(pair.right.case_id, pair.left.case_id);
        assert_eq!(pair.right.task_family, TaskFamily::ChiralFavorable);
        assert_eq!(pair.left.task_family, TaskFamily::ChiralFavorable);
        assert_eq!(
            pair.right_oracle.generator_contract,
            CHIRAL_REFLECTION_TASK_CONTRACT
        );
        assert_eq!(pair.right_oracle.handedness, HandednessTarget::Right);
        assert_eq!(pair.left_oracle.handedness, HandednessTarget::Left);
        assert_ne!(
            pair.right_oracle.expected_score,
            pair.left_oracle.expected_score
        );
    }

    #[test]
    fn mirrored_members_are_exact_mirrors_with_parity_odd_oracle_scores() {
        for pair_id in 0..32 {
            let pair = chiral_reflection_pair(pair_id).unwrap();
            assert_eq!(pair.left.query, pair.right.query.mirror());
            assert_eq!(pair.left.key, pair.right.key.mirror());
            assert_eq!(pair.left.weights, pair.right.weights);
            assert!(pair.right.weights.gamma.abs() > 0.0);

            let right_score =
                chiral_arm_score(pair.right.query, pair.right.key, pair.right.weights).unwrap();
            let left_score =
                chiral_arm_score(pair.left.query, pair.left.key, pair.left.weights).unwrap();
            close(right_score, pair.right_oracle.expected_score);
            close(left_score, pair.left_oracle.expected_score);

            let base = observables(pair.right.query, pair.right.key).unwrap();
            let reflected = observables(pair.left.query, pair.left.key).unwrap();
            close(reflected.direct, base.direct);
            close(reflected.mirrored, base.mirrored);
            close(reflected.chiral, -base.chiral);
            // Even channels cancel in the score difference; only gamma*chi remains.
            close(
                right_score - left_score,
                2.0 * pair.right.weights.gamma * base.chiral,
            );
        }
    }

    #[test]
    fn chiral_case_id_overflow_fails_closed() {
        assert_eq!(
            chiral_reflection_pair(u64::MAX),
            Err(Tdi25TaskError::CaseIdOverflow)
        );
    }

    #[test]
    fn mixed_pairs_are_deterministic_and_keep_oracle_outside_inputs() {
        let pair = mixed_geometry_pair(7).unwrap();
        assert_eq!(pair, mixed_geometry_pair(7).unwrap());
        assert_ne!(pair.base.case_id, pair.transformed.case_id);
        assert_eq!(pair.base.task_family, TaskFamily::Mixed);
        assert_eq!(pair.transformed.task_family, TaskFamily::Mixed);
        assert_eq!(
            pair.base_oracle.generator_contract,
            MIXED_GEOMETRY_TASK_CONTRACT
        );
        assert_eq!(pair.base_oracle.handedness, HandednessTarget::Right);
        assert_eq!(pair.transformed_oracle.handedness, HandednessTarget::Left);
        assert_eq!(
            pair.base_oracle.expected_torsor_score,
            pair.transformed_oracle.expected_torsor_score
        );
        assert_ne!(
            pair.base_oracle.expected_chiral_score,
            pair.transformed_oracle.expected_chiral_score
        );
    }

    #[test]
    fn mixed_transform_preserves_torsor_score_and_flips_chiral_parity() {
        for pair_id in 0..24 {
            let pair = mixed_geometry_pair(pair_id).unwrap();
            assert_eq!(
                pair.base.torsor_key.resultant(),
                pair.transformed.torsor_key.resultant()
            );
            assert_ne!(
                pair.base.torsor_key.reference(),
                pair.transformed.torsor_key.reference()
            );
            assert_eq!(
                pair.transformed.chiral_query,
                pair.base.chiral_query.mirror()
            );
            assert_eq!(pair.transformed.chiral_key, pair.base.chiral_key.mirror());
            assert!(pair.base.weights.gamma.abs() > 0.0);

            let base_torsor = torsor_arm_score(
                pair.base.torsor_query,
                pair.base.torsor_key,
                pair.base.query_position,
            )
            .unwrap();
            let transformed_torsor = torsor_arm_score(
                pair.transformed.torsor_query,
                pair.transformed.torsor_key,
                pair.transformed.query_position,
            )
            .unwrap();
            close(base_torsor, pair.base_oracle.expected_torsor_score);
            close(
                transformed_torsor,
                pair.transformed_oracle.expected_torsor_score,
            );
            close(base_torsor, transformed_torsor);

            let base_chiral = chiral_arm_score(
                pair.base.chiral_query,
                pair.base.chiral_key,
                pair.base.weights,
            )
            .unwrap();
            let transformed_chiral = chiral_arm_score(
                pair.transformed.chiral_query,
                pair.transformed.chiral_key,
                pair.transformed.weights,
            )
            .unwrap();
            close(base_chiral, pair.base_oracle.expected_chiral_score);
            close(
                transformed_chiral,
                pair.transformed_oracle.expected_chiral_score,
            );

            let base_obs = observables(pair.base.chiral_query, pair.base.chiral_key).unwrap();
            let transformed_obs =
                observables(pair.transformed.chiral_query, pair.transformed.chiral_key).unwrap();
            close(transformed_obs.direct, base_obs.direct);
            close(transformed_obs.mirrored, base_obs.mirrored);
            close(transformed_obs.chiral, -base_obs.chiral);
        }
    }

    #[test]
    fn mixed_case_id_overflow_fails_closed() {
        assert_eq!(
            mixed_geometry_pair(u64::MAX),
            Err(Tdi25TaskError::CaseIdOverflow)
        );
    }
}
