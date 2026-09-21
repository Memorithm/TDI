//! TDI-25 Phase-B balanced task generators.
//!
//! Slice 11 introduces deterministic torsor-favorable transport tasks while
//! keeping the oracle separate from inference inputs.

use core::fmt;

use super::tdi22_torsor::{Torsor3, Twist3, Vec3};
use super::tdi25_torsor_chiral::{TaskFamily, Tdi25Error, torsor_arm_score};

/// Versioned TDI-25 torsor-favorable transport task contract.
pub const TORSOR_TRANSPORT_TASK_CONTRACT: &str = "tdi25-torsor-transport-task-v1";

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
    fn case_id_overflow_fails_closed() {
        assert_eq!(
            torsor_transport_pair(u64::MAX),
            Err(Tdi25TaskError::CaseIdOverflow)
        );
    }
}
