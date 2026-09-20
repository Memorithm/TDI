//! TDI-24 Phase-B deterministic task generators.
//!
//! Slice 11 introduces reflection-discriminative paired handedness tasks.
//! Later slices extend this module with additional frozen task families.

use core::fmt;

use super::tdi24_chiral::{Chiral6, ChiralError};

/// Versioned Slice-11 reflection-discriminative generator contract.
pub const REFLECTION_DISCRIMINATIVE_CONTRACT: &str = "tdi24-reflection-discriminative-generator-v1";

/// Handedness oracle for one member of a mirrored pair.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HandednessTarget {
    /// Canonical orientation emitted first by the deterministic generator.
    Right,
    /// Simultaneously mirrored partner.
    Left,
}

/// One bounded reflection-discriminative case.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReflectionDiscriminativeCase {
    /// Pair identity shared by both mirrored members.
    pub pair_id: u64,
    /// Globally deterministic member identity (`2*pair_id` or `2*pair_id+1`).
    pub case_id: u64,
    /// Query carrier.
    pub query: Chiral6,
    /// Key carrier.
    pub key: Chiral6,
    /// Handedness oracle label.
    pub target: HandednessTarget,
    /// Generator contract identity.
    pub generator_contract: &'static str,
}

/// Pair of exact mirrored cases.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReflectionDiscriminativePair {
    /// Canonical member.
    pub right: ReflectionDiscriminativeCase,
    /// Simultaneously reflected member.
    pub left: ReflectionDiscriminativeCase,
}

/// Deterministically materialize one handedness pair from a bounded pair id.
pub fn reflection_discriminative_pair(
    pair_id: u64,
) -> Result<ReflectionDiscriminativePair, Tdi24TaskError> {
    let right_id = pair_id
        .checked_mul(2)
        .ok_or(Tdi24TaskError::CaseIdOverflow)?;
    let left_id = right_id
        .checked_add(1)
        .ok_or(Tdi24TaskError::CaseIdOverflow)?;

    // The bounded offset changes examples without ever depending on model output.
    // Magnitudes stay small enough that every Stage-A scalar remains finite.
    let offset = ((pair_id % 29) as f64 + 1.0) / 64.0;
    let query = Chiral6::new([1.0 + offset, -0.75, 0.5], [0.625, -1.0 - offset, 1.5])
        .map_err(Tdi24TaskError::Chiral)?;
    let key = Chiral6::new([-0.5, 1.25 + offset, -1.0], [1.75 + offset, 0.375, -0.875])
        .map_err(Tdi24TaskError::Chiral)?;

    let right = ReflectionDiscriminativeCase {
        pair_id,
        case_id: right_id,
        query,
        key,
        target: HandednessTarget::Right,
        generator_contract: REFLECTION_DISCRIMINATIVE_CONTRACT,
    };
    let left = ReflectionDiscriminativeCase {
        pair_id,
        case_id: left_id,
        query: query.mirror(),
        key: key.mirror(),
        target: HandednessTarget::Left,
        generator_contract: REFLECTION_DISCRIMINATIVE_CONTRACT,
    };
    Ok(ReflectionDiscriminativePair { right, left })
}

/// Task-generation failures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tdi24TaskError {
    /// Pair id cannot be mapped to two disjoint member ids.
    CaseIdOverflow,
    /// The underlying chiral carrier rejected a generated fixture.
    Chiral(ChiralError),
}

impl fmt::Display for Tdi24TaskError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CaseIdOverflow => formatter.write_str("TDI-24 case id overflow"),
            Self::Chiral(error) => write!(formatter, "generated chiral fixture invalid: {error}"),
        }
    }
}

impl std::error::Error for Tdi24TaskError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::experimental::tdi24_chiral::observables;

    #[test]
    fn reflection_discriminative_pairs_are_deterministic_and_disjoint() {
        let first = reflection_discriminative_pair(7).unwrap();
        let second = reflection_discriminative_pair(7).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.right.case_id, 14);
        assert_eq!(first.left.case_id, 15);
        assert_ne!(first.right.case_id, first.left.case_id);
        assert_eq!(first.right.pair_id, first.left.pair_id);
    }

    #[test]
    fn pair_members_are_exact_simultaneous_reflections_with_opposite_targets() {
        for pair_id in 0..64 {
            let pair = reflection_discriminative_pair(pair_id).unwrap();
            assert_eq!(pair.left.query, pair.right.query.mirror());
            assert_eq!(pair.left.key, pair.right.key.mirror());
            assert_eq!(pair.right.target, HandednessTarget::Right);
            assert_eq!(pair.left.target, HandednessTarget::Left);

            let right = observables(pair.right.query, pair.right.key).unwrap();
            let left = observables(pair.left.query, pair.left.key).unwrap();
            assert_eq!(left.direct, right.direct);
            assert_eq!(left.mirrored, right.mirrored);
            assert_eq!(left.chiral, -right.chiral);
            assert_ne!(right.chiral, 0.0);
        }
    }

    #[test]
    fn generator_contract_is_stable_and_overflow_fails_closed() {
        let pair = reflection_discriminative_pair(1).unwrap();
        assert_eq!(
            pair.right.generator_contract,
            REFLECTION_DISCRIMINATIVE_CONTRACT
        );
        assert_eq!(
            reflection_discriminative_pair(u64::MAX),
            Err(Tdi24TaskError::CaseIdOverflow)
        );
    }
}
