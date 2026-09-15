//! TDI-22 Stage-0 exact torsor/twist algebra scaffolding.
//!
//! This module is deliberately limited to deterministic mathematical identities.
//! It is not an attention evaluator, not a FLAT-ATTENTION kernel, not a model
//! implementation, and not evidence of quality, efficiency, novelty, or hardware
//! performance.
//!
//! Sign convention for transport from source point `P` to target point `Q`:
//!
//! `M(Q) = M(P) + (P - Q) x R`.
//!
//! With the origin-reduced moment
//!
//! `C = M(P) + P x R`,
//!
//! the same torsor evaluated at query point `Q` is
//!
//! `M(Q) = C - Q x R`.
//!
//! For a twist-like query `xi = (v, omega)`, the direct dual pairing
//!
//! `s = v . R + omega . M(Q)`
//!
//! is exactly equivalent, up to floating-point roundoff, to
//!
//! `s = (v + Q x omega) . R + omega . C`.

use core::fmt;

/// Versioned mathematical contract implemented by this Stage-0 module.
pub const TORSOR_CONTRACT: &str = "tdi22-torsor-dual-pairing-v1";

/// Three-dimensional finite vector used by the bounded Stage-0 torsor model.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3 {
    /// X component.
    pub x: f64,
    /// Y component.
    pub y: f64,
    /// Z component.
    pub z: f64,
}

impl Vec3 {
    /// Construct a finite vector.
    pub fn new(x: f64, y: f64, z: f64) -> Result<Self, TorsorError> {
        let value = Self { x, y, z };
        value.validate("vector")?;
        Ok(value)
    }

    /// Zero vector.
    #[must_use]
    pub const fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    /// Dot product.
    #[must_use]
    pub fn dot(self, rhs: Self) -> f64 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    /// Three-dimensional cross product.
    #[must_use]
    pub fn cross(self, rhs: Self) -> Self {
        Self {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    /// Squared Euclidean norm.
    #[must_use]
    pub fn norm_squared(self) -> f64 {
        self.dot(self)
    }

    /// Component-wise addition.
    #[must_use]
    pub fn plus(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }

    /// Component-wise subtraction.
    #[must_use]
    pub fn minus(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }

    fn validate(self, field: &'static str) -> Result<(), TorsorError> {
        if self.x.is_finite() && self.y.is_finite() && self.z.is_finite() {
            Ok(())
        } else {
            Err(TorsorError::NonFiniteVector { field })
        }
    }
}

/// Wrench-like three-dimensional torsor reduced at one explicit reference point.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Torsor3 {
    resultant: Vec3,
    moment: Vec3,
    reference: Vec3,
}

impl Torsor3 {
    /// Construct a finite torsor `(R, M(P))` reduced at `P`.
    pub fn new(resultant: Vec3, moment: Vec3, reference: Vec3) -> Result<Self, TorsorError> {
        resultant.validate("resultant")?;
        moment.validate("moment")?;
        reference.validate("reference")?;
        Ok(Self {
            resultant,
            moment,
            reference,
        })
    }

    /// Resultant `R`, invariant under change of reduction point.
    #[must_use]
    pub const fn resultant(self) -> Vec3 {
        self.resultant
    }

    /// Moment at the stored reduction point.
    #[must_use]
    pub const fn moment(self) -> Vec3 {
        self.moment
    }

    /// Stored reduction point.
    #[must_use]
    pub const fn reference(self) -> Vec3 {
        self.reference
    }

    /// Evaluate the moment at another point using the declared Varignon convention.
    pub fn moment_at(self, target: Vec3) -> Result<Vec3, TorsorError> {
        target.validate("target")?;
        let arm = self.reference.minus(target);
        let transported = self.moment.plus(arm.cross(self.resultant));
        transported.validate("transported_moment")?;
        Ok(transported)
    }

    /// Return the same physical torsor reduced at `target`.
    pub fn transport(self, target: Vec3) -> Result<Self, TorsorError> {
        let moment = self.moment_at(target)?;
        Self::new(self.resultant, moment, target)
    }

    /// Origin-reduced moment `C = M(P) + P x R`.
    pub fn origin_moment(self) -> Result<Vec3, TorsorError> {
        let value = self.moment.plus(self.reference.cross(self.resultant));
        value.validate("origin_moment")?;
        Ok(value)
    }

    /// First reduction-point invariant used by Stage 0: `||R||^2`.
    ///
    /// Finite vector components can still overflow during the dot product, so a
    /// non-finite derived scalar is rejected rather than exposed as an invariant.
    pub fn resultant_norm_squared(self) -> Result<f64, TorsorError> {
        let value = self.resultant.norm_squared();
        if value.is_finite() {
            Ok(value)
        } else {
            Err(TorsorError::NonFiniteScalar {
                field: "resultant_norm_squared",
            })
        }
    }

    /// Second reduction-point invariant used by Stage 0: `R . M(P)`.
    ///
    /// As with the norm invariant, arithmetic overflow is a typed failure even
    /// when every input component is individually finite.
    pub fn scalar_invariant(self) -> Result<f64, TorsorError> {
        let value = self.resultant.dot(self.moment);
        if value.is_finite() {
            Ok(value)
        } else {
            Err(TorsorError::NonFiniteScalar {
                field: "scalar_invariant",
            })
        }
    }

    /// Produce the factorized key `(R, C)` used by the TDI-22 identity tests.
    pub fn factorized_key(self) -> Result<FactorizedTorsorKey, TorsorError> {
        Ok(FactorizedTorsorKey {
            resultant: self.resultant,
            origin_moment: self.origin_moment()?,
        })
    }
}

/// Twist-like query carrier `(v, omega)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Twist3 {
    linear: Vec3,
    angular: Vec3,
}

impl Twist3 {
    /// Construct a finite query twist.
    pub fn new(linear: Vec3, angular: Vec3) -> Result<Self, TorsorError> {
        linear.validate("linear")?;
        angular.validate("angular")?;
        Ok(Self { linear, angular })
    }

    /// Linear component `v`.
    #[must_use]
    pub const fn linear(self) -> Vec3 {
        self.linear
    }

    /// Angular component `omega`.
    #[must_use]
    pub const fn angular(self) -> Vec3 {
        self.angular
    }

    /// Build the six-component factorized query at position `Q`.
    pub fn factorized_at(self, query_position: Vec3) -> Result<FactorizedTwistQuery, TorsorError> {
        query_position.validate("query_position")?;
        let resultant_dual = self.linear.plus(query_position.cross(self.angular));
        resultant_dual.validate("factorized_resultant_dual")?;
        Ok(FactorizedTwistQuery {
            resultant_dual,
            moment_dual: self.angular,
        })
    }
}

/// Position-independent key representation `(R, C)` for the declared convention.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FactorizedTorsorKey {
    /// Resultant component.
    pub resultant: Vec3,
    /// Origin-reduced moment `C`.
    pub origin_moment: Vec3,
}

/// Query-side six-component representation at one query position.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FactorizedTwistQuery {
    /// Coefficient paired with the torsor resultant.
    pub resultant_dual: Vec3,
    /// Coefficient paired with the origin-reduced moment.
    pub moment_dual: Vec3,
}

impl FactorizedTwistQuery {
    /// Pair a factorized query with a factorized torsor key.
    pub fn pair(self, key: FactorizedTorsorKey) -> Result<f64, TorsorError> {
        let score =
            self.resultant_dual.dot(key.resultant) + self.moment_dual.dot(key.origin_moment);
        if score.is_finite() {
            Ok(score)
        } else {
            Err(TorsorError::NonFiniteScalar { field: "pairing" })
        }
    }
}

/// Direct dual pairing `v.R + omega.M(Q)`.
pub fn direct_pairing(
    query: Twist3,
    key: Torsor3,
    query_position: Vec3,
) -> Result<f64, TorsorError> {
    let score = query.linear.dot(key.resultant) + query.angular.dot(key.moment_at(query_position)?);
    if score.is_finite() {
        Ok(score)
    } else {
        Err(TorsorError::NonFiniteScalar { field: "pairing" })
    }
}

/// Factorized pairing `(v + Q x omega).R + omega.C`.
pub fn factorized_pairing(
    query: Twist3,
    key: Torsor3,
    query_position: Vec3,
) -> Result<f64, TorsorError> {
    query
        .factorized_at(query_position)?
        .pair(key.factorized_key()?)
}

/// Explicit invalid-input/arithmetic errors; Stage 0 has no silent fallback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TorsorError {
    /// A three-component input or derived vector contains NaN or infinity.
    NonFiniteVector {
        /// Name of the rejected field.
        field: &'static str,
    },
    /// A derived scalar contains NaN or infinity.
    NonFiniteScalar {
        /// Name of the rejected field.
        field: &'static str,
    },
}

impl fmt::Display for TorsorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteVector { field } => write!(formatter, "{field} must be finite"),
            Self::NonFiniteScalar { field } => write!(formatter, "{field} must be finite"),
        }
    }
}

impl std::error::Error for TorsorError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3::new(x, y, z).unwrap()
    }

    fn close(lhs: f64, rhs: f64) {
        let scale = 1.0_f64.max(lhs.abs()).max(rhs.abs());
        assert!(
            (lhs - rhs).abs() <= 64.0 * f64::EPSILON * scale,
            "lhs={lhs:?}, rhs={rhs:?}"
        );
    }

    fn close_vec(lhs: Vec3, rhs: Vec3) {
        close(lhs.x, rhs.x);
        close(lhs.y, rhs.y);
        close(lhs.z, rhs.z);
    }

    #[test]
    fn transport_round_trip_recovers_original_reduction() {
        let source = v(2.0, -3.0, 5.0);
        let target = v(-7.0, 11.0, 13.0);
        let torsor = Torsor3::new(v(3.0, 4.0, -2.0), v(5.0, -1.0, 7.0), source).unwrap();
        let round_trip = torsor.transport(target).unwrap().transport(source).unwrap();
        close_vec(round_trip.resultant(), torsor.resultant());
        close_vec(round_trip.moment(), torsor.moment());
        close_vec(round_trip.reference(), torsor.reference());
    }

    #[test]
    fn reduction_point_invariants_survive_transport() {
        let torsor =
            Torsor3::new(v(1.5, -2.0, 0.75), v(4.0, 3.0, -5.0), v(2.0, 1.0, -3.0)).unwrap();
        let moved = torsor.transport(v(-4.0, 7.0, 9.0)).unwrap();
        close(
            torsor.resultant_norm_squared().unwrap(),
            moved.resultant_norm_squared().unwrap(),
        );
        close(
            torsor.scalar_invariant().unwrap(),
            moved.scalar_invariant().unwrap(),
        );
        close_vec(
            torsor.origin_moment().unwrap(),
            moved.origin_moment().unwrap(),
        );
    }

    #[test]
    fn derived_invariants_reject_finite_inputs_that_overflow() {
        let huge = v(f64::MAX, 0.0, 0.0);
        let torsor = Torsor3::new(huge, huge, Vec3::zero()).unwrap();
        assert_eq!(
            torsor.resultant_norm_squared(),
            Err(TorsorError::NonFiniteScalar {
                field: "resultant_norm_squared",
            })
        );
        assert_eq!(
            torsor.scalar_invariant(),
            Err(TorsorError::NonFiniteScalar {
                field: "scalar_invariant",
            })
        );
    }

    #[test]
    fn direct_and_factorized_pairings_are_equivalent() {
        let cases = [
            (
                Twist3::new(v(1.0, 2.0, 3.0), v(-2.0, 1.0, 4.0)).unwrap(),
                Torsor3::new(v(4.0, -3.0, 2.0), v(5.0, 7.0, -11.0), v(3.0, -5.0, 8.0)).unwrap(),
                v(-7.0, 13.0, 17.0),
            ),
            (
                Twist3::new(v(-0.25, 0.5, 1.25), v(2.5, -3.5, 4.5)).unwrap(),
                Torsor3::new(
                    v(0.75, 1.5, -2.25),
                    v(3.125, -4.25, 5.5),
                    v(-6.0, 7.0, -8.0),
                )
                .unwrap(),
                v(9.0, -10.0, 11.0),
            ),
        ];

        for (query, key, position) in cases {
            close(
                direct_pairing(query, key, position).unwrap(),
                factorized_pairing(query, key, position).unwrap(),
            );
        }
    }

    #[test]
    fn factorized_pairing_is_invariant_to_global_origin_translation() {
        let query = Twist3::new(v(2.0, -1.0, 0.5), v(3.0, 4.0, -2.0)).unwrap();
        let source = v(5.0, -7.0, 11.0);
        let query_position = v(-13.0, 17.0, 19.0);
        let key = Torsor3::new(v(7.0, 3.0, -5.0), v(2.0, -11.0, 13.0), source).unwrap();
        let translation = v(101.0, -37.0, 23.0);

        let translated_key =
            Torsor3::new(key.resultant(), key.moment(), source.plus(translation)).unwrap();
        let translated_query_position = query_position.plus(translation);

        close(
            factorized_pairing(query, key, query_position).unwrap(),
            factorized_pairing(query, translated_key, translated_query_position).unwrap(),
        );
    }

    #[test]
    fn factorized_key_is_independent_of_reduction_point() {
        let key = Torsor3::new(v(2.0, 3.0, 5.0), v(7.0, 11.0, 13.0), v(17.0, 19.0, 23.0)).unwrap();
        let moved = key.transport(v(-29.0, 31.0, -37.0)).unwrap();
        let lhs = key.factorized_key().unwrap();
        let rhs = moved.factorized_key().unwrap();
        close_vec(lhs.resultant, rhs.resultant);
        close_vec(lhs.origin_moment, rhs.origin_moment);
    }

    #[test]
    fn non_finite_inputs_fail_closed() {
        assert_eq!(
            Vec3::new(f64::NAN, 0.0, 0.0),
            Err(TorsorError::NonFiniteVector { field: "vector" })
        );
        assert_eq!(
            Vec3::new(0.0, f64::INFINITY, 0.0),
            Err(TorsorError::NonFiniteVector { field: "vector" })
        );
    }
}
