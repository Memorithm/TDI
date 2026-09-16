//! TDI-23.0 global-reduction development scaffold.
//!
//! The Stage-0 reduction is intentionally conservative: each Hilbert-space
//! object is reduced by selecting an ordered, duplicate-free subset of its
//! declared orthonormal coordinate basis. A morphism is reduced to the
//! corresponding row/column submatrix. This gives an exact development model
//! for studying which dagger/category laws survive reduction and which do not.
//!
//! In particular, coordinate reduction commutes with dagger exactly, while
//! composition is not assumed to be preserved. The exposed composition-defect
//! metric makes loss through omitted intermediate coordinates explicit.

use core::fmt;

use super::tdi23_categorical::{DaggerError, RealLinearMap};

/// Versioned non-final Stage-0 global-reduction contract.
pub const GLOBAL_REDUCTION_CONTRACT: &str = "tdi23-coordinate-reduction-v1";

/// Fail-closed errors for the Stage-0 coordinate-reduction scaffold.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReductionError {
    /// A coordinate-space dimension was zero.
    ZeroAmbientDimension,
    /// A reduction attempted to retain no coordinates.
    EmptySelection,
    /// One requested coordinate lies outside the ambient space.
    CoordinateOutOfBounds {
        /// Invalid coordinate.
        coordinate: usize,
        /// Ambient space dimension.
        ambient_dim: usize,
    },
    /// A coordinate was selected more than once.
    DuplicateCoordinate {
        /// Duplicated coordinate.
        coordinate: usize,
    },
    /// A reduction was applied to a map side with a different ambient dimension.
    AmbientDimensionMismatch {
        /// Semantic side being validated.
        role: &'static str,
        /// Dimension declared by the map.
        expected: usize,
        /// Dimension declared by the reduction.
        actual: usize,
    },
    /// A reduced map does not match the selected source/target dimensions.
    ReducedMapDimensionMismatch {
        /// Expected reduced domain dimension.
        expected_domain: usize,
        /// Actual reduced domain dimension.
        actual_domain: usize,
        /// Expected reduced codomain dimension.
        expected_codomain: usize,
        /// Actual reduced codomain dimension.
        actual_codomain: usize,
    },
    /// Two maps being compared do not have identical dimensions.
    ComparisonDimensionMismatch,
    /// A coordinate lookup failed despite validated dimensions.
    CoordinateLookupFailure {
        /// Requested row.
        row: usize,
        /// Requested column.
        column: usize,
    },
    /// Finite operands produced a non-finite defect value.
    NonFiniteDerivedValue {
        /// Operation producing the invalid value.
        operation: &'static str,
    },
    /// Underlying bounded linear-map contract rejected an operation.
    Linear(DaggerError),
}

impl fmt::Display for ReductionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroAmbientDimension => formatter.write_str("ambient dimension must be positive"),
            Self::EmptySelection => formatter.write_str("coordinate selection must not be empty"),
            Self::CoordinateOutOfBounds {
                coordinate,
                ambient_dim,
            } => write!(
                formatter,
                "coordinate {coordinate} is outside ambient dimension {ambient_dim}"
            ),
            Self::DuplicateCoordinate { coordinate } => {
                write!(
                    formatter,
                    "coordinate {coordinate} is selected more than once"
                )
            }
            Self::AmbientDimensionMismatch {
                role,
                expected,
                actual,
            } => write!(
                formatter,
                "{role} ambient dimension mismatch: expected {expected}, got {actual}"
            ),
            Self::ReducedMapDimensionMismatch {
                expected_domain,
                actual_domain,
                expected_codomain,
                actual_codomain,
            } => write!(
                formatter,
                "reduced map dimension mismatch: expected {expected_domain}->{expected_codomain}, got {actual_domain}->{actual_codomain}"
            ),
            Self::ComparisonDimensionMismatch => {
                formatter.write_str("reduction defect comparison requires identical map dimensions")
            }
            Self::CoordinateLookupFailure { row, column } => {
                write!(
                    formatter,
                    "validated coordinate lookup failed at ({row}, {column})"
                )
            }
            Self::NonFiniteDerivedValue { operation } => {
                write!(formatter, "{operation} produced a non-finite value")
            }
            Self::Linear(error) => write!(formatter, "linear-map error: {error}"),
        }
    }
}

impl std::error::Error for ReductionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Linear(error) => Some(error),
            _ => None,
        }
    }
}

impl From<DaggerError> for ReductionError {
    fn from(value: DaggerError) -> Self {
        Self::Linear(value)
    }
}

/// Ordered coordinate-subspace reduction for one finite-dimensional object.
///
/// The retained coordinates are unique and preserve the caller-declared order.
/// Stage 0 interprets this as the coordinate isometry `E : R^r -> R^n` whose
/// columns are the selected standard-basis vectors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoordinateReduction {
    ambient_dim: usize,
    kept: Vec<usize>,
}

impl CoordinateReduction {
    /// Construct a validated non-empty coordinate reduction.
    pub fn new(ambient_dim: usize, kept: Vec<usize>) -> Result<Self, ReductionError> {
        if ambient_dim == 0 {
            return Err(ReductionError::ZeroAmbientDimension);
        }
        if kept.is_empty() {
            return Err(ReductionError::EmptySelection);
        }
        for (position, coordinate) in kept.iter().copied().enumerate() {
            if coordinate >= ambient_dim {
                return Err(ReductionError::CoordinateOutOfBounds {
                    coordinate,
                    ambient_dim,
                });
            }
            if kept[..position].contains(&coordinate) {
                return Err(ReductionError::DuplicateCoordinate { coordinate });
            }
        }
        Ok(Self { ambient_dim, kept })
    }

    /// Ambient Hilbert-space dimension.
    #[must_use]
    pub const fn ambient_dim(&self) -> usize {
        self.ambient_dim
    }

    /// Reduced Hilbert-space dimension.
    #[must_use]
    pub fn reduced_dim(&self) -> usize {
        self.kept.len()
    }

    /// Ordered retained ambient coordinates.
    #[must_use]
    pub fn kept_coordinates(&self) -> &[usize] {
        &self.kept
    }

    fn validate_map_side(&self, expected: usize, role: &'static str) -> Result<(), ReductionError> {
        if self.ambient_dim != expected {
            return Err(ReductionError::AmbientDimensionMismatch {
                role,
                expected,
                actual: self.ambient_dim,
            });
        }
        Ok(())
    }
}

/// Reduce `map : H -> K` to `E_K^dagger map E_H` by coordinate selection.
pub fn reduce_linear_map(
    map: &RealLinearMap,
    domain: &CoordinateReduction,
    codomain: &CoordinateReduction,
) -> Result<RealLinearMap, ReductionError> {
    domain.validate_map_side(map.domain_dim(), "domain")?;
    codomain.validate_map_side(map.codomain_dim(), "codomain")?;

    let len = domain
        .reduced_dim()
        .checked_mul(codomain.reduced_dim())
        .ok_or(DaggerError::DimensionOverflow)?;
    let mut entries = Vec::with_capacity(len);
    for row in codomain.kept_coordinates().iter().copied() {
        for column in domain.kept_coordinates().iter().copied() {
            let value = map
                .entry(row, column)
                .ok_or(ReductionError::CoordinateLookupFailure { row, column })?;
            entries.push(value);
        }
    }

    Ok(RealLinearMap::new(
        domain.reduced_dim(),
        codomain.reduced_dim(),
        entries,
    )?)
}

/// Lift a reduced map back to the ambient spaces, zeroing omitted coordinates.
///
/// This realizes `E_K reduced E_H^dagger` for the Stage-0 coordinate isometries.
pub fn lift_linear_map(
    reduced: &RealLinearMap,
    domain: &CoordinateReduction,
    codomain: &CoordinateReduction,
) -> Result<RealLinearMap, ReductionError> {
    let expected_domain = domain.reduced_dim();
    let expected_codomain = codomain.reduced_dim();
    if reduced.domain_dim() != expected_domain || reduced.codomain_dim() != expected_codomain {
        return Err(ReductionError::ReducedMapDimensionMismatch {
            expected_domain,
            actual_domain: reduced.domain_dim(),
            expected_codomain,
            actual_codomain: reduced.codomain_dim(),
        });
    }

    let len = domain
        .ambient_dim()
        .checked_mul(codomain.ambient_dim())
        .ok_or(DaggerError::DimensionOverflow)?;
    let mut entries = vec![0.0; len];
    for (reduced_row, ambient_row) in codomain.kept_coordinates().iter().copied().enumerate() {
        for (reduced_column, ambient_column) in
            domain.kept_coordinates().iter().copied().enumerate()
        {
            let value = reduced.entry(reduced_row, reduced_column).ok_or(
                ReductionError::CoordinateLookupFailure {
                    row: reduced_row,
                    column: reduced_column,
                },
            )?;
            entries[ambient_row * domain.ambient_dim() + ambient_column] = value;
        }
    }

    Ok(RealLinearMap::new(
        domain.ambient_dim(),
        codomain.ambient_dim(),
        entries,
    )?)
}

/// Maximum absolute entrywise residual between a map and its reduce/lift image.
pub fn reduction_residual_max_abs(
    map: &RealLinearMap,
    domain: &CoordinateReduction,
    codomain: &CoordinateReduction,
) -> Result<f64, ReductionError> {
    let reduced = reduce_linear_map(map, domain, codomain)?;
    let lifted = lift_linear_map(&reduced, domain, codomain)?;
    max_abs_difference(map, &lifted, "reduction residual")
}

/// Maximum absolute defect between reducing a composite and composing reductions.
///
/// For `first : A -> B` and `second : B -> C`, this compares
/// `R(second o first)` with `R(second) o R(first)`. A non-zero value is expected
/// whenever omitted coordinates in the intermediate object carry a contributing
/// path. Therefore this function is an audit metric, not a functoriality claim.
pub fn reduction_composition_defect_max_abs(
    first: &RealLinearMap,
    second: &RealLinearMap,
    domain: &CoordinateReduction,
    middle: &CoordinateReduction,
    codomain: &CoordinateReduction,
) -> Result<f64, ReductionError> {
    let reduced_first = reduce_linear_map(first, domain, middle)?;
    let reduced_second = reduce_linear_map(second, middle, codomain)?;
    let composed_reductions = reduced_second.compose(&reduced_first)?;

    let full_composite = second.compose(first)?;
    let reduced_composite = reduce_linear_map(&full_composite, domain, codomain)?;

    max_abs_difference(
        &reduced_composite,
        &composed_reductions,
        "reduction composition defect",
    )
}

fn max_abs_difference(
    left: &RealLinearMap,
    right: &RealLinearMap,
    operation: &'static str,
) -> Result<f64, ReductionError> {
    if left.domain_dim() != right.domain_dim() || left.codomain_dim() != right.codomain_dim() {
        return Err(ReductionError::ComparisonDimensionMismatch);
    }

    let mut maximum = 0.0_f64;
    for (left_value, right_value) in left.entries().iter().zip(right.entries()) {
        let difference = (left_value - right_value).abs();
        if !difference.is_finite() {
            return Err(ReductionError::NonFiniteDerivedValue { operation });
        }
        maximum = maximum.max(difference);
    }
    Ok(maximum)
}

#[cfg(test)]
mod tests {
    use super::{
        CoordinateReduction, GLOBAL_REDUCTION_CONTRACT, ReductionError, lift_linear_map,
        reduce_linear_map, reduction_composition_defect_max_abs, reduction_residual_max_abs,
    };
    use crate::experimental::tdi23_categorical::RealLinearMap;

    #[test]
    fn tdi23_global_reduction_contract_is_versioned() {
        assert_eq!(GLOBAL_REDUCTION_CONTRACT, "tdi23-coordinate-reduction-v1");
    }

    #[test]
    fn tdi23_coordinate_selection_fails_closed() {
        assert_eq!(
            CoordinateReduction::new(3, vec![]),
            Err(ReductionError::EmptySelection)
        );
        assert_eq!(
            CoordinateReduction::new(3, vec![0, 3]),
            Err(ReductionError::CoordinateOutOfBounds {
                coordinate: 3,
                ambient_dim: 3,
            })
        );
        assert_eq!(
            CoordinateReduction::new(3, vec![1, 1]),
            Err(ReductionError::DuplicateCoordinate { coordinate: 1 })
        );
    }

    #[test]
    fn tdi23_coordinate_reduction_commutes_with_dagger_exactly() {
        let map = RealLinearMap::new(
            4,
            3,
            vec![
                1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
            ],
        )
        .expect("fixture map");
        let domain = CoordinateReduction::new(4, vec![0, 2]).expect("domain reduction");
        let codomain = CoordinateReduction::new(3, vec![1, 2]).expect("codomain reduction");

        let reduced_then_dagger = reduce_linear_map(&map, &domain, &codomain)
            .expect("reduce")
            .dagger();
        let dagger_then_reduced =
            reduce_linear_map(&map.dagger(), &codomain, &domain).expect("reduce dagger");

        assert_eq!(reduced_then_dagger, dagger_then_reduced);
    }

    #[test]
    fn tdi23_reduce_lift_reduce_is_idempotent_on_selected_block() {
        let map = RealLinearMap::new(3, 3, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0])
            .expect("fixture map");
        let reduction = CoordinateReduction::new(3, vec![2, 0]).expect("reduction");
        let reduced = reduce_linear_map(&map, &reduction, &reduction).expect("reduce");
        let lifted = lift_linear_map(&reduced, &reduction, &reduction).expect("lift");
        let reduced_again =
            reduce_linear_map(&lifted, &reduction, &reduction).expect("reduce again");

        assert_eq!(reduced_again, reduced);
    }

    #[test]
    fn tdi23_full_middle_reduction_has_zero_composition_defect_fixture() {
        let first = RealLinearMap::new(2, 3, vec![1.0, 0.0, 2.0, 1.0, -1.0, 3.0]).expect("first");
        let second = RealLinearMap::new(3, 2, vec![2.0, 1.0, 0.0, -1.0, 4.0, 2.0]).expect("second");
        let domain = CoordinateReduction::new(2, vec![0, 1]).expect("domain");
        let middle = CoordinateReduction::new(3, vec![0, 1, 2]).expect("middle");
        let codomain = CoordinateReduction::new(2, vec![0, 1]).expect("codomain");

        let defect =
            reduction_composition_defect_max_abs(&first, &second, &domain, &middle, &codomain)
                .expect("defect");
        assert_eq!(defect, 0.0);
    }

    #[test]
    fn tdi23_dropped_middle_path_produces_explicit_composition_defect() {
        let first = RealLinearMap::new(1, 2, vec![1.0, 1.0]).expect("first");
        let second = RealLinearMap::new(2, 1, vec![1.0, 2.0]).expect("second");
        let endpoint = CoordinateReduction::new(1, vec![0]).expect("endpoint");
        let middle = CoordinateReduction::new(2, vec![0]).expect("middle");

        let defect =
            reduction_composition_defect_max_abs(&first, &second, &endpoint, &middle, &endpoint)
                .expect("defect");
        assert_eq!(defect, 2.0);
    }

    #[test]
    fn tdi23_reduce_lift_residual_measures_discarded_entries() {
        let map = RealLinearMap::new(2, 2, vec![1.0, 2.0, 3.0, 4.0]).expect("map");
        let reduction = CoordinateReduction::new(2, vec![0]).expect("reduction");
        let residual = reduction_residual_max_abs(&map, &reduction, &reduction).expect("residual");
        assert_eq!(residual, 4.0);
    }
}
