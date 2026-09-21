//! TDI-25 Phase-B balanced task generators.
//!
//! Slice 11 introduces deterministic torsor-favorable transport tasks while
//! keeping the oracle separate from inference inputs.
//! Slice 12 adds chiral-favorable mirrored handedness pairs with the same
//! oracle/input separation.
//! Slice 13 adds mixed geometry tasks that require both a transported
//! relation and a parity-sensitive relation.
//! Slice 14 adds neutral six-component controls that privilege neither
//! torsor nor chirality in the target construction.
//! Slice 15 registers explicit position-geometry arms (linear/helical/
//! learned/external) so no geometry is an implicit default.
//! Slice 16 adds bounded deterministic difficulty strata independent of
//! model output.

use core::fmt;

use super::tdi22_torsor::{Torsor3, Twist3, Vec3};
use super::tdi24_chiral::{Chiral6, ChiralScoreWeights};
use super::tdi25_torsor_chiral::{
    Generic6, TaskFamily, Tdi25Error, chiral_arm_score, generic_arm_score, torsor_arm_score,
};

/// Versioned TDI-25 torsor-favorable transport task contract.
pub const TORSOR_TRANSPORT_TASK_CONTRACT: &str = "tdi25-torsor-transport-task-v1";

/// Versioned TDI-25 chiral-favorable reflection task contract.
pub const CHIRAL_REFLECTION_TASK_CONTRACT: &str = "tdi25-chiral-reflection-task-v1";

/// Versioned TDI-25 mixed-geometry task contract.
pub const MIXED_GEOMETRY_TASK_CONTRACT: &str = "tdi25-mixed-geometry-task-v1";

/// Versioned TDI-25 neutral six-component control task contract.
pub const NEUTRAL_CONTROL_TASK_CONTRACT: &str = "tdi25-neutral-control-task-v1";

/// Versioned TDI-25 position-geometry arm registry contract.
pub const POSITION_GEOMETRY_ARM_CONTRACT: &str = "tdi25-position-geometry-arm-v1";

/// Inclusive upper bound on admissible geometry indices.
pub const POSITION_GEOMETRY_INDEX_MAX: u64 = 1024;

/// Versioned TDI-25 difficulty-strata contract.
pub const DIFFICULTY_STRATA_CONTRACT: &str = "tdi25-difficulty-strata-v1";

/// Inclusive upper bound on admissible difficulty levels (`0..=MAX`).
pub const DIFFICULTY_LEVEL_MAX: u8 = 3;

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

/// Structure-agnostic binary target for neutral controls.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NeutralTarget {
    /// Positive class under the deterministic sign schedule.
    ClassA,
    /// Negative class under the deterministic sign schedule.
    ClassB,
}

/// Inference-visible input for one neutral Generic6 control case.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NeutralControlInput {
    pub case_id: u64,
    pub query: Generic6,
    pub key: Generic6,
    pub task_family: TaskFamily,
    pub generator_contract: &'static str,
}

/// Target/score oracle retained separately from inference input.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NeutralControlOracle {
    pub pair_id: u64,
    pub target: NeutralTarget,
    pub expected_score: f64,
    pub generator_contract: &'static str,
}

/// Opposite-target neutral pair that privileges neither torsor nor chirality.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NeutralControlPair {
    pub class_a: NeutralControlInput,
    pub class_b: NeutralControlInput,
    pub class_a_oracle: NeutralControlOracle,
    pub class_b_oracle: NeutralControlOracle,
}

/// Deterministically generate one neutral six-component control pair.
///
/// Both members use the matched Generic6 capacity only. The class label is
/// driven by a global sign on generic components, not by torsor transport or
/// chiral parity structure.
pub fn neutral_control_pair(pair_id: u64) -> Result<NeutralControlPair, Tdi25TaskError> {
    let class_a_id = pair_id
        .checked_mul(2)
        .ok_or(Tdi25TaskError::CaseIdOverflow)?;
    let class_b_id = class_a_id
        .checked_add(1)
        .ok_or(Tdi25TaskError::CaseIdOverflow)?;

    let offset = ((pair_id % 43) as f64 + 1.0) / 112.0;
    let make_carriers = |sign: f64| -> Result<(Generic6, Generic6), Tdi25TaskError> {
        let query = Generic6::new([
            sign * (1.0 + offset),
            0.5,
            -0.75,
            0.375 + offset,
            -0.875,
            1.25 - offset,
        ])
        .map_err(Tdi25TaskError::Bridge)?;
        let key = Generic6::new([
            1.0,
            sign * (0.625 + offset),
            0.25,
            -1.125,
            0.625 + offset,
            0.5,
        ])
        .map_err(Tdi25TaskError::Bridge)?;
        Ok((query, key))
    };

    let (query_a, key_a) = make_carriers(1.0)?;
    let (query_b, key_b) = make_carriers(-1.0)?;
    let score_a = generic_arm_score(query_a, key_a).map_err(Tdi25TaskError::Bridge)?;
    let score_b = generic_arm_score(query_b, key_b).map_err(Tdi25TaskError::Bridge)?;

    let make_input = |case_id, query, key| NeutralControlInput {
        case_id,
        query,
        key,
        task_family: TaskFamily::Neutral,
        generator_contract: NEUTRAL_CONTROL_TASK_CONTRACT,
    };

    Ok(NeutralControlPair {
        class_a: make_input(class_a_id, query_a, key_a),
        class_b: make_input(class_b_id, query_b, key_b),
        class_a_oracle: NeutralControlOracle {
            pair_id,
            target: NeutralTarget::ClassA,
            expected_score: score_a,
            generator_contract: NEUTRAL_CONTROL_TASK_CONTRACT,
        },
        class_b_oracle: NeutralControlOracle {
            pair_id,
            target: NeutralTarget::ClassB,
            expected_score: score_b,
            generator_contract: NEUTRAL_CONTROL_TASK_CONTRACT,
        },
    })
}

/// Explicit position-geometry experimental arm.
///
/// A physical 3D coordinate is never an implicit token assumption; every
/// geometry choice carries provenance through this registry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PositionGeometryArm {
    /// Uniform linear abscissa along x.
    Linear,
    /// Discrete helical sample on a bounded octagon with linear z.
    Helical,
    /// Deterministic frozen lookup table (not a trained embedding).
    Learned,
    /// Caller-supplied external coordinate with fail-closed validation.
    External,
}

impl PositionGeometryArm {
    /// Stable lowercase label for manifests and audits.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Linear => "linear",
            Self::Helical => "helical",
            Self::Learned => "learned",
            Self::External => "external",
        }
    }

    /// Human-readable provenance retained beside every sample.
    #[must_use]
    pub const fn provenance(self) -> &'static str {
        match self {
            Self::Linear => "tdi25-geometry-linear-abscissa-v1",
            Self::Helical => "tdi25-geometry-helical-octagon-v1",
            Self::Learned => "tdi25-geometry-learned-table-v1",
            Self::External => "tdi25-geometry-external-supplied-v1",
        }
    }

    /// Fail-closed parse of an arm label.
    pub fn parse(label: &str) -> Result<Self, Tdi25TaskError> {
        match label {
            "linear" => Ok(Self::Linear),
            "helical" => Ok(Self::Helical),
            "learned" => Ok(Self::Learned),
            "external" => Ok(Self::External),
            _ => Err(Tdi25TaskError::UnknownGeometryArm),
        }
    }
}

/// One materialized position sample with explicit arm provenance.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PositionGeometrySample {
    pub arm: PositionGeometryArm,
    pub index: u64,
    pub point: Vec3,
    pub arm_provenance: &'static str,
    pub generator_contract: &'static str,
}

/// Resolve a position for the declared geometry arm.
///
/// `external` is required only for [`PositionGeometryArm::External`].
pub fn position_geometry_point(
    arm: PositionGeometryArm,
    index: u64,
    external: Option<Vec3>,
) -> Result<PositionGeometrySample, Tdi25TaskError> {
    if index > POSITION_GEOMETRY_INDEX_MAX {
        return Err(Tdi25TaskError::GeometryIndexOutOfRange);
    }

    let point = match arm {
        PositionGeometryArm::Linear => Vec3::new(index as f64 / 64.0, 0.0, 0.0)
            .map_err(|error| Tdi25TaskError::Bridge(Tdi25Error::Torsor(error)))?,
        PositionGeometryArm::Helical => {
            let (x, y) = match index % 8 {
                0 => (1.0, 0.0),
                1 => (1.0, 1.0),
                2 => (0.0, 1.0),
                3 => (-1.0, 1.0),
                4 => (-1.0, 0.0),
                5 => (-1.0, -1.0),
                6 => (0.0, -1.0),
                _ => (1.0, -1.0),
            };
            Vec3::new(x, y, index as f64 / 64.0)
                .map_err(|error| Tdi25TaskError::Bridge(Tdi25Error::Torsor(error)))?
        }
        PositionGeometryArm::Learned => {
            // Frozen deterministic table: cyclic unit-ish offsets, not trainable.
            let table = [
                (0.25, -0.5, 0.75),
                (-0.75, 0.25, 0.5),
                (0.5, 0.75, -0.25),
                (-0.25, -0.75, 0.125),
            ];
            let (x, y, z) = table[(index as usize) % table.len()];
            let scale = 1.0 + (index / 4) as f64 / 64.0;
            Vec3::new(x * scale, y * scale, z * scale)
                .map_err(|error| Tdi25TaskError::Bridge(Tdi25Error::Torsor(error)))?
        }
        PositionGeometryArm::External => {
            external.ok_or(Tdi25TaskError::ExternalPositionRequired)?
        }
    };

    Ok(PositionGeometrySample {
        arm,
        index,
        point,
        arm_provenance: arm.provenance(),
        generator_contract: POSITION_GEOMETRY_ARM_CONTRACT,
    })
}

/// Enumerate the frozen registry in stable order.
#[must_use]
pub const fn position_geometry_registry() -> [PositionGeometryArm; 4] {
    [
        PositionGeometryArm::Linear,
        PositionGeometryArm::Helical,
        PositionGeometryArm::Learned,
        PositionGeometryArm::External,
    ]
}

/// Bounded difficulty level independent of any model output.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DifficultyLevel {
    /// Discrete level in `0..=DIFFICULTY_LEVEL_MAX`.
    pub level: u8,
}

impl DifficultyLevel {
    /// Construct a fail-closed level.
    pub fn new(level: u8) -> Result<Self, Tdi25TaskError> {
        if level > DIFFICULTY_LEVEL_MAX {
            return Err(Tdi25TaskError::DifficultyOutOfRange);
        }
        Ok(Self { level })
    }

    /// Deterministic stratum for a seed; never consults model output.
    #[must_use]
    pub fn from_seed(seed: u64) -> Self {
        Self {
            level: (seed % u64::from(DIFFICULTY_LEVEL_MAX + 1)) as u8,
        }
    }

    /// Positive finite magnitude scale for the stratum.
    #[must_use]
    pub fn magnitude_scale(self) -> f64 {
        1.0 + f64::from(self.level) * 0.25
    }
}

/// Difficulty metadata attached to an existing Phase-B family seed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DifficultyStratum {
    pub seed: u64,
    pub level: DifficultyLevel,
    pub generator_contract: &'static str,
}

/// Assign a bounded difficulty stratum from a deterministic seed.
#[must_use]
pub fn difficulty_stratum(seed: u64) -> DifficultyStratum {
    DifficultyStratum {
        seed,
        level: DifficultyLevel::from_seed(seed),
        generator_contract: DIFFICULTY_STRATA_CONTRACT,
    }
}

/// TDI-25 task-generation failures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tdi25TaskError {
    CaseIdOverflow,
    GeometryIndexOutOfRange,
    ExternalPositionRequired,
    UnknownGeometryArm,
    DifficultyOutOfRange,
    Bridge(Tdi25Error),
}

impl fmt::Display for Tdi25TaskError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CaseIdOverflow => formatter.write_str("TDI-25 task case id overflow"),
            Self::GeometryIndexOutOfRange => {
                formatter.write_str("TDI-25 position geometry index out of range")
            }
            Self::ExternalPositionRequired => {
                formatter.write_str("TDI-25 external position geometry requires a supplied point")
            }
            Self::UnknownGeometryArm => formatter.write_str("TDI-25 unknown position geometry arm"),
            Self::DifficultyOutOfRange => {
                formatter.write_str("TDI-25 difficulty level out of admissible range")
            }
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

    #[test]
    fn neutral_pairs_are_deterministic_and_keep_oracle_outside_inputs() {
        let pair = neutral_control_pair(5).unwrap();
        assert_eq!(pair, neutral_control_pair(5).unwrap());
        assert_ne!(pair.class_a.case_id, pair.class_b.case_id);
        assert_eq!(pair.class_a.task_family, TaskFamily::Neutral);
        assert_eq!(pair.class_b.task_family, TaskFamily::Neutral);
        assert_eq!(
            pair.class_a_oracle.generator_contract,
            NEUTRAL_CONTROL_TASK_CONTRACT
        );
        assert_eq!(pair.class_a_oracle.target, NeutralTarget::ClassA);
        assert_eq!(pair.class_b_oracle.target, NeutralTarget::ClassB);
        assert_ne!(
            pair.class_a_oracle.expected_score,
            pair.class_b_oracle.expected_score
        );
    }

    #[test]
    fn neutral_targets_use_generic_capacity_without_torsor_or_chiral_privilege() {
        for pair_id in 0..24 {
            let pair = neutral_control_pair(pair_id).unwrap();
            let score_a = generic_arm_score(pair.class_a.query, pair.class_a.key).unwrap();
            let score_b = generic_arm_score(pair.class_b.query, pair.class_b.key).unwrap();
            close(score_a, pair.class_a_oracle.expected_score);
            close(score_b, pair.class_b_oracle.expected_score);

            // Class label tracks only the global generic sign schedule.
            assert_eq!(pair.class_a.query.as_array()[0].signum(), 1.0);
            assert_eq!(pair.class_b.query.as_array()[0].signum(), -1.0);
            // Shared nuisance coordinates (indices 2,3,5 on query; 0,2,5 on key)
            // stay identical across classes — no chirality/torsor channeling.
            let qa = pair.class_a.query.as_array();
            let qb = pair.class_b.query.as_array();
            let ka = pair.class_a.key.as_array();
            let kb = pair.class_b.key.as_array();
            close(qa[2], qb[2]);
            close(qa[3], qb[3]);
            close(qa[5], qb[5]);
            close(ka[0], kb[0]);
            close(ka[2], kb[2]);
            close(ka[5], kb[5]);
        }
    }

    #[test]
    fn neutral_case_id_overflow_fails_closed() {
        assert_eq!(
            neutral_control_pair(u64::MAX),
            Err(Tdi25TaskError::CaseIdOverflow)
        );
    }

    #[test]
    fn position_geometry_registry_is_explicit_and_complete() {
        let registry = position_geometry_registry();
        assert_eq!(registry.len(), 4);
        assert_eq!(registry[0], PositionGeometryArm::Linear);
        assert_eq!(registry[1], PositionGeometryArm::Helical);
        assert_eq!(registry[2], PositionGeometryArm::Learned);
        assert_eq!(registry[3], PositionGeometryArm::External);
        for arm in registry {
            assert_eq!(PositionGeometryArm::parse(arm.as_str()).unwrap(), arm);
            assert!(!arm.provenance().is_empty());
        }
        assert_eq!(
            PositionGeometryArm::parse("implicit"),
            Err(Tdi25TaskError::UnknownGeometryArm)
        );
    }

    #[test]
    fn position_geometry_points_are_deterministic_with_provenance() {
        for index in 0..32 {
            let linear = position_geometry_point(PositionGeometryArm::Linear, index, None).unwrap();
            assert_eq!(
                linear,
                position_geometry_point(PositionGeometryArm::Linear, index, None).unwrap()
            );
            assert_eq!(linear.generator_contract, POSITION_GEOMETRY_ARM_CONTRACT);
            assert_eq!(
                linear.arm_provenance,
                PositionGeometryArm::Linear.provenance()
            );
            close(linear.point.y, 0.0);
            close(linear.point.z, 0.0);

            let helical =
                position_geometry_point(PositionGeometryArm::Helical, index, None).unwrap();
            assert_eq!(helical.arm, PositionGeometryArm::Helical);

            let learned =
                position_geometry_point(PositionGeometryArm::Learned, index, None).unwrap();
            assert_eq!(learned.arm, PositionGeometryArm::Learned);
            assert_ne!(learned.point, linear.point);
        }

        let supplied = Vec3::new(1.0, -2.0, 0.5).unwrap();
        let external =
            position_geometry_point(PositionGeometryArm::External, 3, Some(supplied)).unwrap();
        assert_eq!(external.point, supplied);
        assert_eq!(
            position_geometry_point(PositionGeometryArm::External, 3, None),
            Err(Tdi25TaskError::ExternalPositionRequired)
        );
        assert_eq!(
            position_geometry_point(
                PositionGeometryArm::Linear,
                POSITION_GEOMETRY_INDEX_MAX + 1,
                None
            ),
            Err(Tdi25TaskError::GeometryIndexOutOfRange)
        );
    }

    #[test]
    fn difficulty_strata_are_deterministic_bounded_and_model_independent() {
        for seed in 0..64 {
            let stratum = difficulty_stratum(seed);
            assert_eq!(stratum, difficulty_stratum(seed));
            assert!(stratum.level.level <= DIFFICULTY_LEVEL_MAX);
            assert_eq!(stratum.generator_contract, DIFFICULTY_STRATA_CONTRACT);
            assert_eq!(stratum.level, DifficultyLevel::from_seed(seed));
            let scale = stratum.level.magnitude_scale();
            assert!(scale.is_finite() && scale >= 1.0);
            assert!(scale <= 1.0 + f64::from(DIFFICULTY_LEVEL_MAX) * 0.25);
        }
        assert_eq!(
            DifficultyLevel::new(DIFFICULTY_LEVEL_MAX + 1),
            Err(Tdi25TaskError::DifficultyOutOfRange)
        );
        let mut seen = [false; (DIFFICULTY_LEVEL_MAX as usize) + 1];
        for seed in 0..32 {
            seen[difficulty_stratum(seed).level.level as usize] = true;
        }
        assert!(seen.iter().all(|hit| *hit));
    }
}
