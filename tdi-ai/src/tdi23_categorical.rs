//! TDI-23 Stage-0 categorical/dagger-attention scaffolding.
//!
//! This module deliberately implements only a bounded real finite-dimensional
//! Hilbert-space analogue. It provides typed dense linear maps, ordinary
//! composition, the real adjoint (transpose), and query/key ket pairing.
//!
//! It is not a complete categorical-attention IR, not a softmax model, not a
//! quantum-computing implementation, and not evidence of quality, memory,
//! asymptotic, or hardware-performance improvement.

use core::fmt;

/// Versioned Stage-0 mathematical contract.
pub const DAGGER_ATTENTION_CONTRACT: &str = "tdi23-real-fdhilb-dagger-v1";

/// Fail-closed errors for the bounded Stage-0 real linear-map carrier.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DaggerError {
    /// A declared Hilbert-space dimension was zero.
    ZeroDimension {
        /// Name of the invalid dimension.
        field: &'static str,
    },
    /// Multiplying dimensions overflowed `usize` while validating storage.
    DimensionOverflow,
    /// Dense row-major storage does not match codomain × domain.
    InvalidStorageLength {
        /// Expected number of scalar entries.
        expected: usize,
        /// Actual number of scalar entries.
        actual: usize,
    },
    /// One stored matrix entry was NaN or infinite.
    NonFiniteEntry {
        /// Row-major entry index.
        index: usize,
    },
    /// One vector entry was NaN or infinite.
    NonFiniteVectorEntry {
        /// Vector entry index.
        index: usize,
    },
    /// A vector length did not match the expected map or pairing dimension.
    VectorDimensionMismatch {
        /// Expected vector length.
        expected: usize,
        /// Actual vector length.
        actual: usize,
    },
    /// Two vectors passed to an inner product had different dimensions.
    InnerProductDimensionMismatch {
        /// Left vector length.
        left: usize,
        /// Right vector length.
        right: usize,
    },
    /// Two linear maps cannot be composed because the middle dimensions differ.
    CompositionDimensionMismatch {
        /// Domain dimension of the left/outer map.
        left_domain: usize,
        /// Codomain dimension of the right/inner map.
        right_codomain: usize,
    },
    /// Finite inputs produced a non-finite derived arithmetic result.
    NonFiniteDerivedValue {
        /// Operation that produced the invalid value.
        operation: &'static str,
    },
}

impl fmt::Display for DaggerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroDimension { field } => write!(formatter, "{field} must be positive"),
            Self::DimensionOverflow => formatter.write_str("dimension product overflowed usize"),
            Self::InvalidStorageLength { expected, actual } => write!(
                formatter,
                "dense storage length mismatch: expected {expected}, got {actual}"
            ),
            Self::NonFiniteEntry { index } => {
                write!(formatter, "matrix entry {index} is not finite")
            }
            Self::NonFiniteVectorEntry { index } => {
                write!(formatter, "vector entry {index} is not finite")
            }
            Self::VectorDimensionMismatch { expected, actual } => write!(
                formatter,
                "vector dimension mismatch: expected {expected}, got {actual}"
            ),
            Self::InnerProductDimensionMismatch { left, right } => write!(
                formatter,
                "inner-product dimension mismatch: left={left}, right={right}"
            ),
            Self::CompositionDimensionMismatch {
                left_domain,
                right_codomain,
            } => write!(
                formatter,
                "composition dimension mismatch: left domain={left_domain}, right codomain={right_codomain}"
            ),
            Self::NonFiniteDerivedValue { operation } => {
                write!(formatter, "{operation} produced a non-finite value")
            }
        }
    }
}

impl std::error::Error for DaggerError {}

/// Dense real linear map `domain -> codomain` in row-major storage.
///
/// Stage 0 assumes declared orthonormal bases and the standard Euclidean inner
/// product. Under those assumptions the dagger is exactly the transpose.
#[derive(Clone, Debug, PartialEq)]
pub struct RealLinearMap {
    domain_dim: usize,
    codomain_dim: usize,
    entries: Vec<f64>,
}

impl RealLinearMap {
    /// Construct a finite row-major dense map with explicit domain/codomain.
    pub fn new(
        domain_dim: usize,
        codomain_dim: usize,
        entries: Vec<f64>,
    ) -> Result<Self, DaggerError> {
        if domain_dim == 0 {
            return Err(DaggerError::ZeroDimension {
                field: "domain_dim",
            });
        }
        if codomain_dim == 0 {
            return Err(DaggerError::ZeroDimension {
                field: "codomain_dim",
            });
        }
        let expected = domain_dim
            .checked_mul(codomain_dim)
            .ok_or(DaggerError::DimensionOverflow)?;
        if entries.len() != expected {
            return Err(DaggerError::InvalidStorageLength {
                expected,
                actual: entries.len(),
            });
        }
        if let Some(index) = entries.iter().position(|value| !value.is_finite()) {
            return Err(DaggerError::NonFiniteEntry { index });
        }
        Ok(Self {
            domain_dim,
            codomain_dim,
            entries,
        })
    }

    /// Construct the identity map on a positive finite dimension.
    pub fn identity(dimension: usize) -> Result<Self, DaggerError> {
        if dimension == 0 {
            return Err(DaggerError::ZeroDimension { field: "dimension" });
        }
        let len = dimension
            .checked_mul(dimension)
            .ok_or(DaggerError::DimensionOverflow)?;
        let mut entries = vec![0.0; len];
        for index in 0..dimension {
            entries[index * dimension + index] = 1.0;
        }
        Self::new(dimension, dimension, entries)
    }

    /// Construct the zero map `domain -> codomain`.
    pub fn zero(domain_dim: usize, codomain_dim: usize) -> Result<Self, DaggerError> {
        let len = domain_dim
            .checked_mul(codomain_dim)
            .ok_or(DaggerError::DimensionOverflow)?;
        Self::new(domain_dim, codomain_dim, vec![0.0; len])
    }

    /// Construct a ket `R -> H` from one finite coordinate vector.
    pub fn ket(vector: &[f64]) -> Result<Self, DaggerError> {
        validate_vector(vector)?;
        Self::new(1, vector.len(), vector.to_vec())
    }

    /// Domain dimension.
    #[must_use]
    pub const fn domain_dim(&self) -> usize {
        self.domain_dim
    }

    /// Codomain dimension.
    #[must_use]
    pub const fn codomain_dim(&self) -> usize {
        self.codomain_dim
    }

    /// Immutable row-major matrix entries.
    #[must_use]
    pub fn entries(&self) -> &[f64] {
        &self.entries
    }

    /// Read one matrix entry by `(row, column)` when both coordinates are valid.
    #[must_use]
    pub fn entry(&self, row: usize, column: usize) -> Option<f64> {
        if row >= self.codomain_dim || column >= self.domain_dim {
            return None;
        }
        Some(self.entries[row * self.domain_dim + column])
    }

    /// Return the real adjoint, i.e. transpose under the Stage-0 basis contract.
    #[must_use]
    pub fn dagger(&self) -> Self {
        let mut transposed = vec![0.0; self.entries.len()];
        for row in 0..self.codomain_dim {
            for column in 0..self.domain_dim {
                transposed[column * self.codomain_dim + row] =
                    self.entries[row * self.domain_dim + column];
            }
        }
        Self {
            domain_dim: self.codomain_dim,
            codomain_dim: self.domain_dim,
            entries: transposed,
        }
    }

    /// Compose `self` after `rhs`, returning `self o rhs`.
    pub fn compose(&self, rhs: &Self) -> Result<Self, DaggerError> {
        if rhs.codomain_dim != self.domain_dim {
            return Err(DaggerError::CompositionDimensionMismatch {
                left_domain: self.domain_dim,
                right_codomain: rhs.codomain_dim,
            });
        }

        let len = rhs
            .domain_dim
            .checked_mul(self.codomain_dim)
            .ok_or(DaggerError::DimensionOverflow)?;
        let mut entries = vec![0.0; len];

        for row in 0..self.codomain_dim {
            for column in 0..rhs.domain_dim {
                let mut sum = 0.0;
                for middle in 0..self.domain_dim {
                    let left = self.entries[row * self.domain_dim + middle];
                    let right = rhs.entries[middle * rhs.domain_dim + column];
                    sum += left * right;
                    if !sum.is_finite() {
                        return Err(DaggerError::NonFiniteDerivedValue {
                            operation: "matrix composition",
                        });
                    }
                }
                entries[row * rhs.domain_dim + column] = sum;
            }
        }

        Self::new(rhs.domain_dim, self.codomain_dim, entries)
    }

    /// Apply the map to one finite vector in its domain.
    pub fn apply(&self, vector: &[f64]) -> Result<Vec<f64>, DaggerError> {
        if vector.len() != self.domain_dim {
            return Err(DaggerError::VectorDimensionMismatch {
                expected: self.domain_dim,
                actual: vector.len(),
            });
        }
        validate_vector(vector)?;

        let mut output = vec![0.0; self.codomain_dim];
        for (row, value) in output.iter_mut().enumerate() {
            let mut sum = 0.0;
            for (column, coordinate) in vector.iter().copied().enumerate() {
                sum += self.entries[row * self.domain_dim + column] * coordinate;
                if !sum.is_finite() {
                    return Err(DaggerError::NonFiniteDerivedValue {
                        operation: "linear-map application",
                    });
                }
            }
            *value = sum;
        }
        Ok(output)
    }
}

/// Compute the declared Stage-0 Euclidean inner product `<left, right>`.
pub fn euclidean_inner_product(left: &[f64], right: &[f64]) -> Result<f64, DaggerError> {
    if left.len() != right.len() {
        return Err(DaggerError::InnerProductDimensionMismatch {
            left: left.len(),
            right: right.len(),
        });
    }
    validate_vector(left)?;
    validate_vector(right)?;

    let mut sum = 0.0;
    for (left_value, right_value) in left.iter().zip(right) {
        sum += left_value * right_value;
        if !sum.is_finite() {
            return Err(DaggerError::NonFiniteDerivedValue {
                operation: "euclidean inner product",
            });
        }
    }
    Ok(sum)
}

/// Compute an attention-style scalar explicitly as `k^dagger o q`.
///
/// This Stage-0 helper exists to test the categorical encoding against the
/// ordinary real dot product. It performs no scaling, masking, or softmax.
pub fn dagger_attention_score(query: &[f64], key: &[f64]) -> Result<f64, DaggerError> {
    let query_ket = RealLinearMap::ket(query)?;
    let key_ket = RealLinearMap::ket(key)?;
    let scalar = key_ket.dagger().compose(&query_ket)?;
    Ok(scalar.entries[0])
}

fn validate_vector(vector: &[f64]) -> Result<(), DaggerError> {
    if vector.is_empty() {
        return Err(DaggerError::ZeroDimension {
            field: "vector_dimension",
        });
    }
    if let Some(index) = vector.iter().position(|value| !value.is_finite()) {
        return Err(DaggerError::NonFiniteVectorEntry { index });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        DAGGER_ATTENTION_CONTRACT, DaggerError, RealLinearMap, dagger_attention_score,
        euclidean_inner_product,
    };

    #[test]
    fn tdi23_contract_is_versioned() {
        assert_eq!(DAGGER_ATTENTION_CONTRACT, "tdi23-real-fdhilb-dagger-v1");
    }

    #[test]
    fn tdi23_dagger_swaps_dimensions_and_is_involutive() {
        let map = RealLinearMap::new(3, 2, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0])
            .expect("finite rectangular fixture");
        let dagger = map.dagger();
        assert_eq!(dagger.domain_dim(), 2);
        assert_eq!(dagger.codomain_dim(), 3);
        assert_eq!(dagger.entries(), &[1.0, 4.0, 2.0, 5.0, 3.0, 6.0]);
        assert_eq!(dagger.dagger(), map);
    }

    #[test]
    fn tdi23_dagger_reverses_composition() {
        let f = RealLinearMap::new(2, 3, vec![1.0, 2.0, 0.0, 1.0, 3.0, -1.0])
            .expect("finite f");
        let g = RealLinearMap::new(3, 2, vec![2.0, 0.0, 1.0, -1.0, 4.0, 2.0])
            .expect("finite g");

        let left = g.compose(&f).expect("g o f").dagger();
        let right = f
            .dagger()
            .compose(&g.dagger())
            .expect("f^dagger o g^dagger");
        assert_eq!(left, right);
    }

    #[test]
    fn tdi23_key_dagger_query_matches_euclidean_dot_product() {
        let query = [1.0, -2.0, 3.0, 4.0];
        let key = [5.0, 6.0, -1.0, 2.0];
        let categorical = dagger_attention_score(&query, &key).expect("valid categorical score");
        let direct = euclidean_inner_product(&key, &query).expect("valid direct score");
        assert_eq!(categorical, direct);
        assert_eq!(categorical, -2.0);
    }

    #[test]
    fn tdi23_identity_zero_and_application_controls_are_explicit() {
        let identity = RealLinearMap::identity(3).expect("identity");
        assert_eq!(identity.apply(&[2.0, -1.0, 4.0]).expect("apply"), [2.0, -1.0, 4.0]);

        let zero = RealLinearMap::zero(3, 2).expect("zero map");
        assert_eq!(zero.apply(&[2.0, -1.0, 4.0]).expect("apply"), [0.0, 0.0]);
    }

    #[test]
    fn tdi23_dimension_mismatch_fails_closed() {
        let left = RealLinearMap::identity(3).expect("left");
        let right = RealLinearMap::identity(2).expect("right");
        assert_eq!(
            left.compose(&right),
            Err(DaggerError::CompositionDimensionMismatch {
                left_domain: 3,
                right_codomain: 2,
            })
        );
    }

    #[test]
    fn tdi23_nonfinite_inputs_fail_closed() {
        assert_eq!(
            RealLinearMap::new(1, 1, vec![f64::NAN]),
            Err(DaggerError::NonFiniteEntry { index: 0 })
        );
        assert_eq!(
            dagger_attention_score(&[1.0, f64::INFINITY], &[1.0, 2.0]),
            Err(DaggerError::NonFiniteVectorEntry { index: 1 })
        );
    }
}
