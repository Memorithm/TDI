//! TDI-24 Phase-B deterministic task generators.
//!
//! Slice 11 introduces reflection-discriminative paired handedness tasks.
//! Later slices extend this module with additional frozen task families.

use core::fmt;

use super::tdi24_chiral::{Chiral6, ChiralError};

/// Versioned Slice-11 reflection-discriminative generator contract.
pub const REFLECTION_DISCRIMINATIVE_CONTRACT: &str = "tdi24-reflection-discriminative-generator-v1";

/// Versioned Slice-12 reflection-nuisance generator contract.
pub const REFLECTION_NUISANCE_CONTRACT: &str = "tdi24-reflection-nuisance-generator-v1";

/// Versioned Slice-13 ordered direction/reversal generator contract.
pub const DIRECTION_REVERSAL_CONTRACT: &str = "tdi24-direction-reversal-generator-v1";

/// Versioned Slice-14 non-chiral negative-control generator contract.
pub const NON_CHIRAL_CONTROL_CONTRACT: &str = "tdi24-non-chiral-control-generator-v1";

const NUISANCE_CASE_ID_PREFIX: u64 = 1_u64 << 63;

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

/// Reflection-invariant binary oracle for the nuisance family.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReflectionInvariantTarget {
    /// Positive even-sector class.
    ClassA,
    /// Negative even-sector class.
    ClassB,
}

/// One case where mirror reflection is a nuisance transformation and must not
/// alter the oracle target.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReflectionNuisanceCase {
    pub pair_id: u64,
    pub case_id: u64,
    pub query: Chiral6,
    pub key: Chiral6,
    pub target: ReflectionInvariantTarget,
    pub generator_contract: &'static str,
}

/// Exact mirrored nuisance pair with one shared target.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReflectionNuisancePair {
    pub canonical: ReflectionNuisanceCase,
    pub reflected: ReflectionNuisanceCase,
}

/// Deterministically materialize one reflection-nuisance pair.
pub fn reflection_nuisance_pair(pair_id: u64) -> Result<ReflectionNuisancePair, Tdi24TaskError> {
    let local_base = pair_id
        .checked_mul(2)
        .filter(|value| *value < (u64::MAX >> 1))
        .ok_or(Tdi24TaskError::CaseIdOverflow)?;
    let canonical_id = NUISANCE_CASE_ID_PREFIX | local_base;
    let reflected_id = canonical_id
        .checked_add(1)
        .ok_or(Tdi24TaskError::CaseIdOverflow)?;

    let target = if pair_id % 2 == 0 {
        ReflectionInvariantTarget::ClassA
    } else {
        ReflectionInvariantTarget::ClassB
    };
    let sign = match target {
        ReflectionInvariantTarget::ClassA => 1.0,
        ReflectionInvariantTarget::ClassB => -1.0,
    };
    let offset = ((pair_id % 31) as f64 + 1.0) / 80.0;
    let query = Chiral6::new(
        [sign * (1.0 + offset), 0.5, -0.25],
        [0.75 + offset, -0.625, 1.125],
    )
    .map_err(Tdi24TaskError::Chiral)?;
    let key = Chiral6::new([1.0, -0.5 + offset, 0.875], [-1.25, 0.375 + offset, 0.5])
        .map_err(Tdi24TaskError::Chiral)?;

    let canonical = ReflectionNuisanceCase {
        pair_id,
        case_id: canonical_id,
        query,
        key,
        target,
        generator_contract: REFLECTION_NUISANCE_CONTRACT,
    };
    let reflected = ReflectionNuisanceCase {
        pair_id,
        case_id: reflected_id,
        query: query.mirror(),
        key: key.mirror(),
        target,
        generator_contract: REFLECTION_NUISANCE_CONTRACT,
    };
    Ok(ReflectionNuisancePair {
        canonical,
        reflected,
    })
}

/// Ordered relation oracle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DirectionTarget {
    Forward,
    Reverse,
}

/// One ordered query-key relation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DirectionReversalCase {
    pub pair_id: u64,
    pub case_id: u64,
    pub query: Chiral6,
    pub key: Chiral6,
    pub target: DirectionTarget,
    pub generator_contract: &'static str,
}

/// Exact order-reversal pair.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DirectionReversalPair {
    pub forward: DirectionReversalCase,
    pub reverse: DirectionReversalCase,
}

/// Deterministically materialize one ordered relation and its reversal.
pub fn direction_reversal_pair(pair_id: u64) -> Result<DirectionReversalPair, Tdi24TaskError> {
    let local_base = pair_id
        .checked_mul(2)
        .filter(|value| *value < (1_u64 << 62))
        .ok_or(Tdi24TaskError::CaseIdOverflow)?;
    let prefix = 1_u64 << 62;
    let forward_id = prefix | local_base;
    let reverse_id = forward_id
        .checked_add(1)
        .ok_or(Tdi24TaskError::CaseIdOverflow)?;

    let offset = ((pair_id % 37) as f64 + 1.0) / 96.0;
    let query = Chiral6::new([1.0 + offset, -0.5, 0.75], [0.25, -1.125 - offset, 1.5])
        .map_err(Tdi24TaskError::Chiral)?;
    let key = Chiral6::new(
        [-0.625, 1.375 + offset, -1.25],
        [1.625 + offset, 0.5, -0.375],
    )
    .map_err(Tdi24TaskError::Chiral)?;

    Ok(DirectionReversalPair {
        forward: DirectionReversalCase {
            pair_id,
            case_id: forward_id,
            query,
            key,
            target: DirectionTarget::Forward,
            generator_contract: DIRECTION_REVERSAL_CONTRACT,
        },
        reverse: DirectionReversalCase {
            pair_id,
            case_id: reverse_id,
            query: key,
            key: query,
            target: DirectionTarget::Reverse,
            generator_contract: DIRECTION_REVERSAL_CONTRACT,
        },
    })
}

/// Target defined exclusively from parity-even construction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NonChiralTarget {
    ClassA,
    ClassB,
}

/// Negative-control case whose odd sectors are nuisance-only.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NonChiralControlCase {
    pub case_id: u64,
    pub nuisance_id: u64,
    pub query: Chiral6,
    pub key: Chiral6,
    pub target: NonChiralTarget,
    pub generator_contract: &'static str,
}

/// Deterministically materialize one negative-control case.
///
/// Consecutive cases 2*n and 2*n+1 have opposite targets but exactly identical
/// query/key odd sectors. Only parity-even coordinates depend on the target.
pub fn non_chiral_control_case(case_id: u64) -> Result<NonChiralControlCase, Tdi24TaskError> {
    let nuisance_id = case_id / 2;
    let target = if case_id % 2 == 0 {
        NonChiralTarget::ClassA
    } else {
        NonChiralTarget::ClassB
    };
    let sign = match target {
        NonChiralTarget::ClassA => 1.0,
        NonChiralTarget::ClassB => -1.0,
    };
    let nuisance = ((nuisance_id % 43) as f64 + 1.0) / 112.0;

    let query_odd = [0.375 + nuisance, -0.875, 1.25 - nuisance];
    let key_odd = [-1.125, 0.625 + nuisance, 0.5];

    let query = Chiral6::new([sign * (1.0 + nuisance), 0.5, -0.75], query_odd)
        .map_err(Tdi24TaskError::Chiral)?;
    let key = Chiral6::new([1.0, sign * (0.625 + nuisance), 0.25], key_odd)
        .map_err(Tdi24TaskError::Chiral)?;

    Ok(NonChiralControlCase {
        case_id,
        nuisance_id,
        query,
        key,
        target,
        generator_contract: NON_CHIRAL_CONTROL_CONTRACT,
    })
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
    #[test]
    fn reflection_nuisance_pairs_are_deterministic_target_invariant_and_namespaced() {
        for pair_id in 0..64 {
            let pair = reflection_nuisance_pair(pair_id).unwrap();
            assert_eq!(pair, reflection_nuisance_pair(pair_id).unwrap());
            assert_eq!(pair.canonical.target, pair.reflected.target);
            assert_eq!(pair.reflected.query, pair.canonical.query.mirror());
            assert_eq!(pair.reflected.key, pair.canonical.key.mirror());
            assert_ne!(pair.canonical.case_id, pair.reflected.case_id);
            assert_ne!(
                pair.canonical.case_id,
                reflection_discriminative_pair(pair_id)
                    .unwrap()
                    .right
                    .case_id
            );
            assert_ne!(pair.canonical.case_id & NUISANCE_CASE_ID_PREFIX, 0);
        }
    }

    #[test]
    fn nuisance_reflection_changes_only_the_odd_observable_not_the_target() {
        for pair_id in 0..32 {
            let pair = reflection_nuisance_pair(pair_id).unwrap();
            let canonical = observables(pair.canonical.query, pair.canonical.key).unwrap();
            let reflected = observables(pair.reflected.query, pair.reflected.key).unwrap();
            assert_eq!(canonical.direct, reflected.direct);
            assert_eq!(canonical.mirrored, reflected.mirrored);
            assert_eq!(canonical.chiral, -reflected.chiral);
            assert_eq!(pair.canonical.target, pair.reflected.target);
        }
    }

    #[test]
    fn nuisance_case_id_overflow_fails_closed() {
        assert_eq!(
            reflection_nuisance_pair(u64::MAX),
            Err(Tdi24TaskError::CaseIdOverflow)
        );
    }
    #[test]
    fn direction_reversal_pairs_swap_order_and_oracle_without_mutating_values() {
        for pair_id in 0..64 {
            let pair = direction_reversal_pair(pair_id).unwrap();
            assert_eq!(pair.reverse.query, pair.forward.key);
            assert_eq!(pair.reverse.key, pair.forward.query);
            assert_eq!(pair.forward.target, DirectionTarget::Forward);
            assert_eq!(pair.reverse.target, DirectionTarget::Reverse);
            assert_ne!(pair.forward.case_id, pair.reverse.case_id);

            let forward = observables(pair.forward.query, pair.forward.key).unwrap();
            let reverse = observables(pair.reverse.query, pair.reverse.key).unwrap();
            assert_eq!(forward.direct, reverse.direct);
            assert_eq!(forward.mirrored, reverse.mirrored);
            assert_eq!(forward.chiral, -reverse.chiral);
            assert_ne!(forward.chiral, 0.0);
        }
    }

    #[test]
    fn direction_reversal_generator_is_deterministic_and_fails_closed_on_id_overflow() {
        assert_eq!(
            direction_reversal_pair(19).unwrap(),
            direction_reversal_pair(19).unwrap()
        );
        assert_eq!(
            direction_reversal_pair(u64::MAX),
            Err(Tdi24TaskError::CaseIdOverflow)
        );
        assert_eq!(
            direction_reversal_pair(1)
                .unwrap()
                .forward
                .generator_contract,
            DIRECTION_REVERSAL_CONTRACT
        );
    }
    #[test]
    fn non_chiral_control_pairs_opposite_targets_with_identical_odd_sectors() {
        for nuisance_id in 0..64 {
            let a = non_chiral_control_case(2 * nuisance_id).unwrap();
            let b = non_chiral_control_case(2 * nuisance_id + 1).unwrap();
            assert_eq!(a.nuisance_id, b.nuisance_id);
            assert_eq!(a.query.odd(), b.query.odd());
            assert_eq!(a.key.odd(), b.key.odd());
            assert_ne!(a.target, b.target);
            assert_ne!(a.query.even(), b.query.even());
            assert_eq!(a.generator_contract, NON_CHIRAL_CONTROL_CONTRACT);
            assert_eq!(b.generator_contract, NON_CHIRAL_CONTROL_CONTRACT);
        }
    }

    #[test]
    fn non_chiral_target_is_reflection_invariant_by_construction() {
        for case_id in 0..128 {
            let case = non_chiral_control_case(case_id).unwrap();
            assert_eq!(case.query.mirror().even(), case.query.even());
            assert_eq!(case.key.mirror().even(), case.key.even());
            assert_eq!(
                case.target,
                non_chiral_control_case(case_id).unwrap().target
            );
        }
    }

    #[test]
    fn odd_sector_carries_no_target_bit_within_paired_nuisance_strata() {
        for nuisance_id in 0..128 {
            let class_a = non_chiral_control_case(2 * nuisance_id).unwrap();
            let class_b = non_chiral_control_case(2 * nuisance_id + 1).unwrap();
            assert_eq!(class_a.query.odd(), class_b.query.odd());
            assert_eq!(class_a.key.odd(), class_b.key.odd());
        }
    }
}
