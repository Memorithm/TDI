//! TDI-25 Stage-0 comparison scaffold for torsor versus chiral attention.
//!
//! This module does not reimplement either representation.  It binds the
//! existing TDI-22 torsor/twist contract to the shared TDI-24 chiral contract
//! and supplies only a matched generic six-component attribution control.
//!
//! No model training, softmax/normalizer choice, task result, confirmatory
//! execution or superiority claim is implemented here.

use core::fmt;

use super::tdi22_torsor::{
    TORSOR_CONTRACT, Torsor3, TorsorError, Twist3, Vec3, factorized_pairing,
};
use super::tdi24_attention::{
    MASKING_CONTRACT, MaskPolicy, NORMALIZER_CONTRACT, NormalizerError, normalize_with_policy,
};
use super::tdi24_chiral::{
    CHIRAL_CONTRACT, Chiral6, ChiralError, ChiralObservables, ChiralScoreWeights, chiral_score,
    observables,
};

/// Versioned Stage-0 TDI-25 comparison contract.
pub const TDI25_CONTRACT: &str = "tdi25-torsor-vs-chiral-v1";

/// Version of the TDI-25 source-contract provenance pin.
pub const SOURCE_CONTRACT_PIN_VERSION: &str = "tdi25-source-contract-pin-v1";

/// Exact upstream semantic contract identities accepted by this TDI-25 tranche.
///
/// A later upstream contract revision must update this pin explicitly before
/// TDI-25 comparison machinery can consume it.
pub const PINNED_SOURCE_CONTRACTS: SourceContracts = SourceContracts {
    torsor: "tdi22-torsor-dual-pairing-v1",
    chiral: "tdi24-mirror-coupled-chiral-v1",
};

/// The torsor/twist representation consumed by TDI-25 has six scalar
/// components: three linear/resultant and three angular/moment components.
pub const TORSOR_WIDTH: usize = 6;

/// Versioned generic six-component attribution-control contract.
pub const GENERIC6_CONTRACT: &str = "tdi25-generic6-control-v1";

/// Fixed generic control width.
pub const GENERIC6_WIDTH: usize = 6;

/// Versioned carrier/accounting contract for T6/C6/G6.
pub const CARRIER_ACCOUNTING_CONTRACT: &str = "tdi25-carrier-accounting-v1";

/// Versioned common scalar score-scale contract.
pub const SCORE_SCALE_CONTRACT: &str = "tdi25-common-score-scale-v1";

/// Versioned TDI-22 torsor-invariant bridge contract.
pub const TORSOR_INVARIANT_BRIDGE_CONTRACT: &str = "tdi25-torsor-invariant-bridge-v1";

/// Versioned TDI-24 chiral-invariant bridge contract.
pub const CHIRAL_INVARIANT_BRIDGE_CONTRACT: &str = "tdi25-chiral-invariant-bridge-v1";

/// Versioned shared masking/normalization bridge contract.
pub const MASK_NORMALIZER_BRIDGE_CONTRACT: &str = "tdi25-shared-mask-normalizer-v1";

const _GENERIC_MATCH_TORSOR: [(); TORSOR_WIDTH] = [(); GENERIC6_WIDTH];
const _GENERIC_MATCH_CHIRAL: [(); super::tdi24_chiral::CHIRAL_WIDTH] = [(); GENERIC6_WIDTH];

/// TDI-25 comparison arm identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComparisonArm {
    /// TDI-22 torsor/twist arm.
    T6,
    /// TDI-24 chiral arm.
    C6,
    /// Generic six-component attribution control.
    G6,
}

/// Static score-carrier accounting for one arm.
///
/// The six score-pairing components are matched across arms. T6 additionally
/// accounts for its three-component stored key reference point and the
/// three-component query position required before factorized pairing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CarrierAccounting {
    /// Arm being described.
    pub arm: ComparisonArm,
    /// Scalar query components presented to the score pairing.
    pub query_components: usize,
    /// Scalar key components presented to the score pairing.
    pub key_components: usize,
    /// Scalar score outputs per query-key pair.
    pub score_components: usize,
    /// Explicit external geometry components required before score pairing.
    pub external_geometry_components: usize,
    /// f64 bytes occupied by the query score carrier.
    pub query_bytes: usize,
    /// f64 bytes occupied by the key score carrier.
    pub key_bytes: usize,
    /// Contract defining this accounting surface.
    pub accounting_contract: &'static str,
}

/// Return the declared static carrier accounting for an arm.
#[must_use]
pub const fn carrier_accounting(arm: ComparisonArm) -> CarrierAccounting {
    let (key_components, external_geometry_components) = match arm {
        ComparisonArm::T6 => (9, 3),
        ComparisonArm::C6 | ComparisonArm::G6 => (6, 0),
    };
    CarrierAccounting {
        arm,
        query_components: 6,
        key_components,
        score_components: 1,
        external_geometry_components,
        query_bytes: 6 * core::mem::size_of::<f64>(),
        key_bytes: key_components * core::mem::size_of::<f64>(),
        accounting_contract: CARRIER_ACCOUNTING_CONTRACT,
    }
}

/// Source contracts consumed by this comparison scaffold.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceContracts {
    /// TDI-22 torsor/twist contract identity.
    pub torsor: &'static str,
    /// TDI-24 mirror-coupled chiral contract identity.
    pub chiral: &'static str,
}

/// Return the exact upstream contracts used by this build.
#[must_use]
pub const fn source_contracts() -> SourceContracts {
    SourceContracts {
        torsor: TORSOR_CONTRACT,
        chiral: CHIRAL_CONTRACT,
    }
}

/// Verify that the source contracts compiled into this build are exactly the
/// versions pinned by TDI-25 Slice 02.
///
/// This turns upstream contract drift into a typed failure instead of silently
/// changing the comparison semantics.
pub fn validate_source_contracts() -> Result<SourceContracts, Tdi25Error> {
    validate_source_contracts_against(source_contracts())
}

fn validate_source_contracts_against(
    actual: SourceContracts,
) -> Result<SourceContracts, Tdi25Error> {
    if actual.torsor != PINNED_SOURCE_CONTRACTS.torsor {
        return Err(Tdi25Error::SourceContractMismatch {
            source: "TDI-22 torsor",
            expected: PINNED_SOURCE_CONTRACTS.torsor,
            actual: actual.torsor,
        });
    }
    if actual.chiral != PINNED_SOURCE_CONTRACTS.chiral {
        return Err(Tdi25Error::SourceContractMismatch {
            source: "TDI-24 chiral",
            expected: PINNED_SOURCE_CONTRACTS.chiral,
            actual: actual.chiral,
        });
    }
    Ok(actual)
}

/// Generic finite six-component control with no torsor or chiral semantics.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Generic6 {
    data: [f64; GENERIC6_WIDTH],
}

impl Generic6 {
    /// Construct a finite generic six-component carrier.
    pub fn new(data: [f64; GENERIC6_WIDTH]) -> Result<Self, Tdi25Error> {
        if data.iter().all(|value| value.is_finite()) {
            Ok(Self { data })
        } else {
            Err(Tdi25Error::NonFiniteGeneric)
        }
    }

    /// Expose the raw matched-capacity carrier.
    #[must_use]
    pub const fn as_array(self) -> [f64; GENERIC6_WIDTH] {
        self.data
    }

    /// Generic dot-product attribution score with fail-closed overflow handling.
    pub fn dot(self, rhs: Self) -> Result<f64, Tdi25Error> {
        let mut accumulator = 0.0;
        for (lhs, rhs) in self.data.iter().zip(rhs.data.iter()) {
            let product = finite_mul(*lhs, *rhs, "generic_product")?;
            accumulator = finite_add(accumulator, product, "generic_accumulator")?;
        }
        Ok(accumulator)
    }
}

/// TDI-22 invariants observed through the TDI-25 bridge.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TorsorInvariantSnapshot {
    /// Reduction-point invariant squared resultant norm.
    pub resultant_norm_squared: f64,
    /// Reduction-point invariant scalar product R.M.
    pub scalar_invariant: f64,
    /// Origin-reduced moment C.
    pub origin_moment: Vec3,
    /// Contract defining this bridge diagnostic.
    pub bridge_contract: &'static str,
}

/// Observe the upstream TDI-22 invariants without reimplementing them.
pub fn torsor_invariant_snapshot(key: Torsor3) -> Result<TorsorInvariantSnapshot, Tdi25Error> {
    Ok(TorsorInvariantSnapshot {
        resultant_norm_squared: key.resultant_norm_squared().map_err(Tdi25Error::Torsor)?,
        scalar_invariant: key.scalar_invariant().map_err(Tdi25Error::Torsor)?,
        origin_moment: key.origin_moment().map_err(Tdi25Error::Torsor)?,
        bridge_contract: TORSOR_INVARIANT_BRIDGE_CONTRACT,
    })
}

/// Return invariant snapshots before and after reducing the same torsor at a
/// new point. The caller retains the numerical tolerance policy.
pub fn transported_torsor_invariants(
    key: Torsor3,
    target: Vec3,
) -> Result<(TorsorInvariantSnapshot, TorsorInvariantSnapshot), Tdi25Error> {
    let before = torsor_invariant_snapshot(key)?;
    let transported = key.transport(target).map_err(Tdi25Error::Torsor)?;
    let after = torsor_invariant_snapshot(transported)?;
    Ok((before, after))
}

/// Chiral observables before and after simultaneous mirror reflection.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChiralInvariantSnapshot {
    /// Original `s/m/chi` observables.
    pub base: ChiralObservables,
    /// Observables after `(q,k) -> (Mq,Mk)`.
    pub reflected: ChiralObservables,
    /// Upstream chiral algebra contract.
    pub chiral_contract: &'static str,
    /// TDI-25 bridge contract.
    pub bridge_contract: &'static str,
}

/// Validate TDI-24 carrier and parity identities through the TDI-25 adapter.
///
/// This consumes the upstream operations directly. It does not reimplement the
/// mirror or complex structure. Any contract drift or identity failure is a
/// typed error and blocks later comparison slices.
pub fn validate_chiral_invariants(
    query: Chiral6,
    key: Chiral6,
) -> Result<ChiralInvariantSnapshot, Tdi25Error> {
    validate_source_contracts()?;

    if query.mirror().mirror() != query || key.mirror().mirror() != key {
        return Err(Tdi25Error::ChiralInvariantViolation {
            field: "mirror_squared",
        });
    }
    if query.complex_structure().complex_structure() != query.negate()
        || key.complex_structure().complex_structure() != key.negate()
    {
        return Err(Tdi25Error::ChiralInvariantViolation { field: "j_squared" });
    }
    if query.mirror().complex_structure().mirror() != query.complex_structure().negate()
        || key.mirror().complex_structure().mirror() != key.complex_structure().negate()
    {
        return Err(Tdi25Error::ChiralInvariantViolation { field: "mjm" });
    }

    let base = observables(query, key).map_err(Tdi25Error::Chiral)?;
    let reflected = observables(query.mirror(), key.mirror()).map_err(Tdi25Error::Chiral)?;
    if reflected.direct != base.direct {
        return Err(Tdi25Error::ChiralInvariantViolation {
            field: "direct_even",
        });
    }
    if reflected.mirrored != base.mirrored {
        return Err(Tdi25Error::ChiralInvariantViolation {
            field: "mirror_even",
        });
    }
    if reflected.chiral != -base.chiral {
        return Err(Tdi25Error::ChiralInvariantViolation {
            field: "chiral_odd",
        });
    }

    Ok(ChiralInvariantSnapshot {
        base,
        reflected,
        chiral_contract: CHIRAL_CONTRACT,
        bridge_contract: CHIRAL_INVARIANT_BRIDGE_CONTRACT,
    })
}

/// Score the TDI-22 torsor arm through the upstream factorized pairing.
///
/// The wrapper exists so later comparison machinery can depend on a TDI-25
/// interface without copying any TDI-22 transport or pairing logic.
pub fn torsor_arm_score(
    query: Twist3,
    key: Torsor3,
    query_position: Vec3,
) -> Result<f64, Tdi25Error> {
    factorized_pairing(query, key, query_position).map_err(Tdi25Error::Torsor)
}

/// Score the TDI-24 chiral arm through the upstream shared contract.
pub fn chiral_arm_score(
    query: Chiral6,
    key: Chiral6,
    weights: ChiralScoreWeights,
) -> Result<f64, Tdi25Error> {
    chiral_score(query, key, weights).map_err(Tdi25Error::Chiral)
}

/// Score the generic six-component attribution control.
pub fn generic_arm_score(query: Generic6, key: Generic6) -> Result<f64, Tdi25Error> {
    query.dot(key)
}

/// Common positive finite score divisor applied identically to all arms.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScoreScale {
    divisor: f64,
}

impl ScoreScale {
    /// Construct a finite positive scale.
    pub fn new(divisor: f64) -> Result<Self, Tdi25Error> {
        if divisor.is_finite() && divisor > 0.0 {
            Ok(Self { divisor })
        } else {
            Err(Tdi25Error::InvalidScoreScale)
        }
    }

    /// Common `sqrt(6)` divisor for the matched six-component carriers.
    pub fn matched_six_component() -> Self {
        Self {
            divisor: (GENERIC6_WIDTH as f64).sqrt(),
        }
    }

    /// Expose the frozen divisor.
    #[must_use]
    pub const fn divisor(self) -> f64 {
        self.divisor
    }

    /// Apply this scale to one already-valid scalar score.
    pub fn apply(self, score: f64) -> Result<f64, Tdi25Error> {
        finite_scalar(score / self.divisor, "scaled_score")
    }
}

/// One normalized score row with immutable arm and upstream-contract provenance.
#[derive(Clone, Debug, PartialEq)]
pub struct NormalizedArmRow {
    /// Arm whose logits were normalized.
    pub arm: ComparisonArm,
    /// Shared normalized probabilities.
    pub probabilities: Vec<f64>,
    /// Upstream TDI-24 mask contract consumed unchanged.
    pub masking_contract: &'static str,
    /// Upstream TDI-24 normalizer contract consumed unchanged.
    pub normalizer_contract: &'static str,
    /// TDI-25 bridge contract proving all arms share the same path.
    pub bridge_contract: &'static str,
}

/// Normalize one arm's already-computed logits through the exact shared TDI-24
/// mask/normalizer implementation. No arm-specific branch is permitted here.
pub fn normalize_arm_row(
    arm: ComparisonArm,
    logits: &[f64],
    policy: MaskPolicy,
    query_index: usize,
) -> Result<NormalizedArmRow, Tdi25Error> {
    let probabilities =
        normalize_with_policy(logits, policy, query_index).map_err(Tdi25Error::Normalizer)?;
    Ok(NormalizedArmRow {
        arm,
        probabilities,
        masking_contract: MASKING_CONTRACT,
        normalizer_contract: NORMALIZER_CONTRACT,
        bridge_contract: MASK_NORMALIZER_BRIDGE_CONTRACT,
    })
}

/// Stage-0 score bundle. Values are kept separate; this type deliberately does
/// not compute a winner, rank, aggregate metric or statistical conclusion.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ArmScores {
    /// TDI-22 torsor arm score.
    pub torsor: f64,
    /// TDI-24 chiral arm score.
    pub chiral: f64,
    /// Generic six-component attribution control score.
    pub generic: f64,
}

/// Apply one identical scalar scale to all three arms.
pub fn scaled_arm_scores(scores: ArmScores, scale: ScoreScale) -> Result<ArmScores, Tdi25Error> {
    Ok(ArmScores {
        torsor: scale.apply(scores.torsor)?,
        chiral: scale.apply(scores.chiral)?,
        generic: scale.apply(scores.generic)?,
    })
}

/// Evaluate the three Stage-0 scalar score paths for one already-materialized
/// fixture.  Encoding/task fairness is a later preregistered responsibility;
/// this function only proves that TDI-25 calls the upstream contracts.
#[allow(
    clippy::too_many_arguments,
    reason = "Stage-0 bridge keeps each arm's independently materialized fixture explicit"
)]
pub fn score_fixture(
    torsor_query: Twist3,
    torsor_key: Torsor3,
    torsor_query_position: Vec3,
    chiral_query: Chiral6,
    chiral_key: Chiral6,
    chiral_weights: ChiralScoreWeights,
    generic_query: Generic6,
    generic_key: Generic6,
) -> Result<ArmScores, Tdi25Error> {
    Ok(ArmScores {
        torsor: torsor_arm_score(torsor_query, torsor_key, torsor_query_position)?,
        chiral: chiral_arm_score(chiral_query, chiral_key, chiral_weights)?,
        generic: generic_arm_score(generic_query, generic_key)?,
    })
}

fn finite_mul(lhs: f64, rhs: f64, field: &'static str) -> Result<f64, Tdi25Error> {
    finite_scalar(lhs * rhs, field)
}

fn finite_add(lhs: f64, rhs: f64, field: &'static str) -> Result<f64, Tdi25Error> {
    finite_scalar(lhs + rhs, field)
}

fn finite_scalar(value: f64, field: &'static str) -> Result<f64, Tdi25Error> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(Tdi25Error::NonFiniteScalar { field })
    }
}

/// Stage-0 binding/control failures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tdi25Error {
    /// Shared TDI-24 mask/normalizer rejected the row.
    Normalizer(NormalizerError),
    /// A required TDI-24 mirror/parity identity failed through the bridge.
    ChiralInvariantViolation {
        /// Identity or parity channel that failed.
        field: &'static str,
    },
    /// Score normalization divisor must be finite and strictly positive.
    InvalidScoreScale,
    /// One upstream semantic contract no longer matches the frozen TDI-25 pin.
    SourceContractMismatch {
        /// Human-readable upstream source identifier.
        source: &'static str,
        /// Contract identity required by this tranche.
        expected: &'static str,
        /// Contract identity actually compiled into the build.
        actual: &'static str,
    },
    /// Upstream TDI-22 torsor contract rejected the fixture/arithmetic.
    Torsor(TorsorError),
    /// Upstream TDI-24 chiral contract rejected the fixture/arithmetic.
    Chiral(ChiralError),
    /// Generic control received NaN or infinity.
    NonFiniteGeneric,
    /// Generic derived arithmetic became non-finite.
    NonFiniteScalar {
        /// Name of the rejected quantity.
        field: &'static str,
    },
}

impl fmt::Display for Tdi25Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Normalizer(error) => {
                write!(formatter, "shared normalizer rejected row: {error:?}")
            }
            Self::ChiralInvariantViolation { field } => {
                write!(formatter, "chiral bridge invariant failed: {field}")
            }
            Self::InvalidScoreScale => {
                formatter.write_str("score scale must be finite and positive")
            }
            Self::SourceContractMismatch {
                source,
                expected,
                actual,
            } => write!(
                formatter,
                "{source} contract mismatch: expected {expected}, compiled {actual}"
            ),
            Self::Torsor(error) => write!(formatter, "torsor arm rejected fixture: {error}"),
            Self::Chiral(error) => write!(formatter, "chiral arm rejected fixture: {error}"),
            Self::NonFiniteGeneric => {
                formatter.write_str("generic six-component carrier must be finite")
            }
            Self::NonFiniteScalar { field } => write!(formatter, "{field} must be finite"),
        }
    }
}

impl std::error::Error for Tdi25Error {}

#[cfg(test)]
mod tests {
    use super::super::tdi22_torsor::direct_pairing;
    use super::super::tdi24_chiral::{CHIRAL_WIDTH, observables};
    use super::*;

    fn v(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3::new(x, y, z).unwrap()
    }

    fn c(even: [f64; 3], odd: [f64; 3]) -> Chiral6 {
        Chiral6::new(even, odd).unwrap()
    }

    fn close(lhs: f64, rhs: f64) {
        let scale = 1.0_f64.max(lhs.abs()).max(rhs.abs());
        assert!(
            (lhs - rhs).abs() <= 64.0 * f64::EPSILON * scale,
            "lhs={lhs:?}, rhs={rhs:?}"
        );
    }

    #[test]
    fn all_arms_share_identical_mask_and_normalization_path() {
        let logits = [1.0, 2.0, 30.0, 40.0];
        let mut rows = Vec::new();
        for arm in [ComparisonArm::T6, ComparisonArm::C6, ComparisonArm::G6] {
            rows.push(normalize_arm_row(arm, &logits, MaskPolicy::Causal, 1).unwrap());
        }
        assert_eq!(rows[0].probabilities, rows[1].probabilities);
        assert_eq!(rows[1].probabilities, rows[2].probabilities);
        for row in rows {
            assert_eq!(row.probabilities[2], 0.0);
            assert_eq!(row.probabilities[3], 0.0);
            assert_eq!(row.masking_contract, MASKING_CONTRACT);
            assert_eq!(row.normalizer_contract, NORMALIZER_CONTRACT);
            assert_eq!(row.bridge_contract, MASK_NORMALIZER_BRIDGE_CONTRACT);
        }
    }

    #[test]
    fn invalid_normalization_rows_fail_identically_for_every_arm() {
        for arm in [ComparisonArm::T6, ComparisonArm::C6, ComparisonArm::G6] {
            assert_eq!(
                normalize_arm_row(arm, &[], MaskPolicy::Full, 0),
                Err(Tdi25Error::Normalizer(NormalizerError::EmptyInput))
            );
            assert_eq!(
                normalize_arm_row(arm, &[f64::NAN], MaskPolicy::Full, 0),
                Err(Tdi25Error::Normalizer(NormalizerError::NonFiniteLogit))
            );
        }
    }

    #[test]
    fn both_primary_carriers_have_six_components() {
        assert_eq!(TORSOR_WIDTH, CHIRAL_WIDTH);
        assert_eq!(GENERIC6_WIDTH, CHIRAL_WIDTH);
        assert_eq!(TORSOR_WIDTH, 6);
    }

    #[test]
    fn chiral_and_generic_carrier_accounting_is_six_by_six() {
        for arm in [ComparisonArm::C6, ComparisonArm::G6] {
            let accounting = carrier_accounting(arm);
            assert_eq!(accounting.query_components, 6);
            assert_eq!(accounting.key_components, 6);
            assert_eq!(accounting.score_components, 1);
            assert_eq!(accounting.query_bytes, 6 * core::mem::size_of::<f64>());
            assert_eq!(accounting.key_bytes, 6 * core::mem::size_of::<f64>());
            assert_eq!(accounting.accounting_contract, CARRIER_ACCOUNTING_CONTRACT);
        }
    }

    #[test]
    fn torsor_external_geometry_is_accounted_not_hidden() {
        let torsor = carrier_accounting(ComparisonArm::T6);
        assert_eq!(torsor.query_components, 6);
        assert_eq!(torsor.key_components, 9);
        assert_eq!(torsor.query_bytes, 6 * core::mem::size_of::<f64>());
        assert_eq!(torsor.key_bytes, 9 * core::mem::size_of::<f64>());
        assert_eq!(torsor.external_geometry_components, 3);
        assert_eq!(
            carrier_accounting(ComparisonArm::C6).external_geometry_components,
            0
        );
        assert_eq!(
            carrier_accounting(ComparisonArm::G6).external_geometry_components,
            0
        );
    }

    #[test]
    fn common_score_scale_is_identical_for_all_arms() {
        let scale = ScoreScale::matched_six_component();
        close(scale.divisor(), (6.0_f64).sqrt());
        let scores = ArmScores {
            torsor: 6.0,
            chiral: 12.0,
            generic: -3.0,
        };
        let scaled = scaled_arm_scores(scores, scale).unwrap();
        close(scaled.torsor, scores.torsor / scale.divisor());
        close(scaled.chiral, scores.chiral / scale.divisor());
        close(scaled.generic, scores.generic / scale.divisor());
        assert_eq!(SCORE_SCALE_CONTRACT, "tdi25-common-score-scale-v1");
    }

    #[test]
    fn invalid_score_scales_fail_closed() {
        for invalid in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert_eq!(ScoreScale::new(invalid), Err(Tdi25Error::InvalidScoreScale));
        }
    }

    #[test]
    fn source_contract_ids_are_bound_not_redeclared() {
        let contracts = source_contracts();
        assert_eq!(contracts.torsor, TORSOR_CONTRACT);
        assert_eq!(contracts.chiral, CHIRAL_CONTRACT);
        assert_ne!(contracts.torsor, contracts.chiral);
    }

    #[test]
    fn source_contract_pin_accepts_only_the_declared_upstream_versions() {
        assert_eq!(SOURCE_CONTRACT_PIN_VERSION, "tdi25-source-contract-pin-v1");
        assert_eq!(
            PINNED_SOURCE_CONTRACTS,
            SourceContracts {
                torsor: "tdi22-torsor-dual-pairing-v1",
                chiral: "tdi24-mirror-coupled-chiral-v1",
            }
        );
        assert_eq!(validate_source_contracts().unwrap(), source_contracts());
    }

    #[test]
    fn source_contract_pin_fails_closed_on_simulated_drift() {
        let wrong_torsor = SourceContracts {
            torsor: "tdi22-torsor-dual-pairing-v2",
            chiral: PINNED_SOURCE_CONTRACTS.chiral,
        };
        assert_eq!(
            validate_source_contracts_against(wrong_torsor),
            Err(Tdi25Error::SourceContractMismatch {
                source: "TDI-22 torsor",
                expected: "tdi22-torsor-dual-pairing-v1",
                actual: "tdi22-torsor-dual-pairing-v2",
            })
        );

        let wrong_chiral = SourceContracts {
            torsor: PINNED_SOURCE_CONTRACTS.torsor,
            chiral: "tdi24-mirror-coupled-chiral-v2",
        };
        assert_eq!(
            validate_source_contracts_against(wrong_chiral),
            Err(Tdi25Error::SourceContractMismatch {
                source: "TDI-24 chiral",
                expected: "tdi24-mirror-coupled-chiral-v1",
                actual: "tdi24-mirror-coupled-chiral-v2",
            })
        );
    }

    #[test]
    fn chiral_bridge_validates_upstream_carrier_and_reflection_identities() {
        let query = c([1.0, 2.0, -3.0], [0.5, -1.5, 2.5]);
        let key = c([-4.0, 1.0, 2.0], [3.0, 0.25, -0.75]);
        let snapshot = validate_chiral_invariants(query, key).unwrap();
        assert_eq!(snapshot.chiral_contract, CHIRAL_CONTRACT);
        assert_eq!(snapshot.bridge_contract, CHIRAL_INVARIANT_BRIDGE_CONTRACT);
        close(snapshot.reflected.direct, snapshot.base.direct);
        close(snapshot.reflected.mirrored, snapshot.base.mirrored);
        close(snapshot.reflected.chiral, -snapshot.base.chiral);
    }

    #[test]
    fn torsor_transport_bridge_preserves_upstream_invariants_and_score() {
        let query = Twist3::new(v(1.0, -2.0, 3.0), v(0.5, 4.0, -1.0)).unwrap();
        let key =
            Torsor3::new(v(2.0, 3.0, -4.0), v(-5.0, 7.0, 11.0), v(13.0, -17.0, 19.0)).unwrap();
        let target = v(-2.0, 5.0, 7.0);
        let query_position = v(-23.0, 29.0, 31.0);
        let (before, after) = transported_torsor_invariants(key, target).unwrap();
        assert_eq!(before.bridge_contract, TORSOR_INVARIANT_BRIDGE_CONTRACT);
        assert_eq!(after.bridge_contract, TORSOR_INVARIANT_BRIDGE_CONTRACT);
        close(before.resultant_norm_squared, after.resultant_norm_squared);
        close(before.scalar_invariant, after.scalar_invariant);
        close(before.origin_moment.x, after.origin_moment.x);
        close(before.origin_moment.y, after.origin_moment.y);
        close(before.origin_moment.z, after.origin_moment.z);

        let transported = key.transport(target).unwrap();
        close(
            torsor_arm_score(query, key, query_position).unwrap(),
            torsor_arm_score(query, transported, query_position).unwrap(),
        );
    }

    #[test]
    fn torsor_bridge_matches_upstream_factorized_and_direct_pairings() {
        let query = Twist3::new(v(1.0, -2.0, 3.0), v(0.5, 4.0, -1.0)).unwrap();
        let key =
            Torsor3::new(v(2.0, 3.0, -4.0), v(-5.0, 7.0, 11.0), v(13.0, -17.0, 19.0)).unwrap();
        let position = v(-23.0, 29.0, 31.0);
        let bridged = torsor_arm_score(query, key, position).unwrap();
        close(bridged, factorized_pairing(query, key, position).unwrap());
        close(bridged, direct_pairing(query, key, position).unwrap());
    }

    #[test]
    fn chiral_bridge_preserves_upstream_parity_odd_channel() {
        let query = c([1.0, 2.0, -3.0], [0.5, -1.5, 2.5]);
        let key = c([-4.0, 1.0, 2.0], [3.0, 0.25, -0.75]);
        let base = observables(query, key).unwrap();
        let mirrored = observables(query.mirror(), key.mirror()).unwrap();
        close(mirrored.chiral, -base.chiral);
    }

    #[test]
    fn generic_control_contract_is_explicit_and_width_matched() {
        assert_eq!(GENERIC6_CONTRACT, "tdi25-generic6-control-v1");
        assert_eq!(GENERIC6_WIDTH, TORSOR_WIDTH);
        assert_eq!(GENERIC6_WIDTH, CHIRAL_WIDTH);
    }

    #[test]
    fn generic_control_is_plain_six_dimensional_dot_product() {
        let q = Generic6::new([1.0, 2.0, 3.0, 4.0, 5.0, 6.0]).unwrap();
        let k = Generic6::new([-1.0, 0.5, 2.0, -3.0, 4.0, 1.5]).unwrap();
        close(
            generic_arm_score(q, k).unwrap(),
            -1.0 + 2.0 * 0.5 + 3.0 * 2.0 - 4.0 * 3.0 + 5.0 * 4.0 + 6.0 * 1.5,
        );
    }

    #[test]
    fn generic_control_rejects_non_finite_input_and_overflow() {
        assert_eq!(
            Generic6::new([f64::NAN, 0.0, 0.0, 0.0, 0.0, 0.0]),
            Err(Tdi25Error::NonFiniteGeneric)
        );
        let huge = Generic6::new([f64::MAX, 0.0, 0.0, 0.0, 0.0, 0.0]).unwrap();
        assert_eq!(
            huge.dot(huge),
            Err(Tdi25Error::NonFiniteScalar {
                field: "generic_product"
            })
        );

        let accumulator = Generic6::new([f64::MAX, f64::MAX, 0.0, 0.0, 0.0, 0.0]).unwrap();
        let ones = Generic6::new([1.0, 1.0, 0.0, 0.0, 0.0, 0.0]).unwrap();
        assert_eq!(
            accumulator.dot(ones),
            Err(Tdi25Error::NonFiniteScalar {
                field: "generic_accumulator"
            })
        );
    }
}
