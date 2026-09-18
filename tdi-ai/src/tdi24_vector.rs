//! TDI-24 Slice 03 matched six-dimensional vector reference.
//!
//! This control deliberately carries no mirror, parity or complex-structure
//! semantics. Its only structure is a finite six-component Euclidean vector.
//! Width and arithmetic discipline are matched to the TDI-24 chiral carrier so
//! later V6-vs-C6 comparisons cannot attribute an effect to a hidden width or
//! overflow-policy mismatch.

use core::fmt;

/// Versioned matched-vector reference contract.
pub const VECTOR6_CONTRACT: &str = "tdi24-matched-vector6-v1";

/// Fixed width of the vector reference.
pub const VECTOR6_WIDTH: usize = 6;

/// Compile-time width match against the chiral carrier, without reusing its
/// representation or any mirror/chiral operation.
const _MATCHED_CHIRAL_WIDTH: [(); super::tdi24_chiral::CHIRAL_WIDTH] = [(); VECTOR6_WIDTH];

/// Finite generic six-dimensional vector used by the V6 control arm.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vector6 {
    data: [f64; VECTOR6_WIDTH],
}

impl Vector6 {
    /// Construct a finite V6 carrier.
    pub fn new(data: [f64; VECTOR6_WIDTH]) -> Result<Self, Vector6Error> {
        if data.iter().all(|value| value.is_finite()) {
            Ok(Self { data })
        } else {
            Err(Vector6Error::NonFiniteVector)
        }
    }

    /// Return the complete six-component carrier.
    #[must_use]
    pub const fn as_array(self) -> [f64; VECTOR6_WIDTH] {
        self.data
    }

    /// Checked Euclidean pairing.
    ///
    /// Every product and partial accumulation must remain finite, matching the
    /// fail-closed discipline used by the C6 direct channel.
    pub fn dot(self, rhs: Self) -> Result<f64, Vector6Error> {
        let mut accumulator = 0.0;
        for (lhs, rhs) in self.data.iter().zip(rhs.data.iter()) {
            let product = finite_mul(*lhs, *rhs, "vector6_product")?;
            accumulator = finite_add(accumulator, product, "vector6_accumulator")?;
        }
        Ok(accumulator)
    }
}

/// Score the matched V6 reference. No normalization, mask, learned parameter,
/// mirror operation or hidden state is included in this Stage-A control.
pub fn vector6_score(query: Vector6, key: Vector6) -> Result<f64, Vector6Error> {
    query.dot(key)
}

fn finite_mul(lhs: f64, rhs: f64, field: &'static str) -> Result<f64, Vector6Error> {
    finite_scalar(lhs * rhs, field)
}

fn finite_add(lhs: f64, rhs: f64, field: &'static str) -> Result<f64, Vector6Error> {
    finite_scalar(lhs + rhs, field)
}

fn finite_scalar(value: f64, field: &'static str) -> Result<f64, Vector6Error> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(Vector6Error::NonFiniteScalar { field })
    }
}

/// Fail-closed V6 input and arithmetic errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vector6Error {
    /// The carrier contains NaN or infinity.
    NonFiniteVector,
    /// A derived product or partial sum became non-finite.
    NonFiniteScalar {
        /// Name of the rejected derived quantity.
        field: &'static str,
    },
}

impl fmt::Display for Vector6Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteVector => formatter.write_str("vector6 carrier must be finite"),
            Self::NonFiniteScalar { field } => write!(formatter, "{field} must be finite"),
        }
    }
}

impl std::error::Error for Vector6Error {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::experimental::tdi24_chiral::{Chiral6, vector_score as chiral_direct_score};

    fn v(data: [f64; 6]) -> Vector6 {
        Vector6::new(data).unwrap()
    }

    fn chiral_from_same(data: [f64; 6]) -> Chiral6 {
        Chiral6::new([data[0], data[1], data[2]], [data[3], data[4], data[5]]).unwrap()
    }

    #[test]
    fn vector6_has_exactly_the_matched_six_component_budget() {
        assert_eq!(VECTOR6_WIDTH, super::super::tdi24_chiral::CHIRAL_WIDTH);
        assert_eq!(v([0.0; 6]).as_array().len(), 6);
    }

    #[test]
    fn vector6_rejects_non_finite_inputs() {
        for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut data = [0.0; 6];
            data[2] = invalid;
            assert_eq!(Vector6::new(data), Err(Vector6Error::NonFiniteVector));
        }
    }

    #[test]
    fn vector6_score_matches_c6_direct_channel_for_identical_components() {
        let fixtures = [
            (
                [1.0, -2.0, 3.0, 0.5, -1.5, 2.5],
                [-4.0, 1.0, 2.0, 3.0, 0.25, -0.75],
            ),
            ([0.0; 6], [1.0; 6]),
            (
                [0.125, -8.0, 2.0, 4.0, -0.5, 3.0],
                [2.0, 0.25, -1.0, 0.5, 6.0, -2.0],
            ),
        ];

        for (query, key) in fixtures {
            let v6 = vector6_score(v(query), v(key)).unwrap();
            let c6 = chiral_direct_score(chiral_from_same(query), chiral_from_same(key)).unwrap();
            assert_eq!(v6, c6);
        }
    }

    #[test]
    fn vector6_overflow_policy_matches_fail_closed_stage0_discipline() {
        let huge = v([f64::MAX, 0.0, 0.0, 0.0, 0.0, 0.0]);
        assert_eq!(
            huge.dot(huge),
            Err(Vector6Error::NonFiniteScalar {
                field: "vector6_product",
            })
        );

        let accum = v([f64::MAX, f64::MAX, 0.0, 0.0, 0.0, 0.0]);
        let ones = v([1.0, 1.0, 0.0, 0.0, 0.0, 0.0]);
        assert_eq!(
            accum.dot(ones),
            Err(Vector6Error::NonFiniteScalar {
                field: "vector6_accumulator",
            })
        );
    }

    #[test]
    fn vector6_is_deterministic_and_symmetric_on_finite_fixture() {
        let q = v([1.0, -2.0, 3.0, 0.5, -1.5, 2.5]);
        let k = v([-4.0, 1.0, 2.0, 3.0, 0.25, -0.75]);
        let qk = q.dot(k).unwrap();
        assert_eq!(qk, q.dot(k).unwrap());
        assert_eq!(qk, k.dot(q).unwrap());
    }
}
