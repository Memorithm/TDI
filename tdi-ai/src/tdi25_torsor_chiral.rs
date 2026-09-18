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
use super::tdi24_chiral::{
    CHIRAL_CONTRACT, Chiral6, ChiralError, ChiralScoreWeights, chiral_score,
};

/// Versioned Stage-0 TDI-25 comparison contract.
pub const TDI25_CONTRACT: &str = "tdi25-torsor-vs-chiral-v1";

/// The torsor/twist representation consumed by TDI-25 has six scalar
/// components: three linear/resultant and three angular/moment components.
pub const TORSOR_WIDTH: usize = 6;

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

/// Generic finite six-component control with no torsor or chiral semantics.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Generic6 {
    data: [f64; 6],
}

impl Generic6 {
    /// Construct a finite generic six-component carrier.
    pub fn new(data: [f64; 6]) -> Result<Self, Tdi25Error> {
        if data.iter().all(|value| value.is_finite()) {
            Ok(Self { data })
        } else {
            Err(Tdi25Error::NonFiniteGeneric)
        }
    }

    /// Expose the raw matched-capacity carrier.
    #[must_use]
    pub const fn as_array(self) -> [f64; 6] {
        self.data
    }

    /// Generic dot-product attribution score with fail-closed overflow handling.
    pub fn dot(self, rhs: Self) -> Result<f64, Tdi25Error> {
        let score = self
            .data
            .iter()
            .zip(rhs.data.iter())
            .map(|(lhs, rhs)| lhs * rhs)
            .sum::<f64>();
        finite_scalar(score, "generic_score")
    }
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
    fn both_primary_carriers_have_six_components() {
        assert_eq!(TORSOR_WIDTH, CHIRAL_WIDTH);
        assert_eq!(TORSOR_WIDTH, 6);
    }

    #[test]
    fn source_contract_ids_are_bound_not_redeclared() {
        let contracts = source_contracts();
        assert_eq!(contracts.torsor, TORSOR_CONTRACT);
        assert_eq!(contracts.chiral, CHIRAL_CONTRACT);
        assert_ne!(contracts.torsor, contracts.chiral);
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
                field: "generic_score"
            })
        );
    }
}
