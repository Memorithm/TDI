//! Deterministic source-subspace diagnostics for bounded TDI-7.x audits.
//!
//! This module is diagnostic only. It does not classify a scientific hypothesis
//! and has no final-holdout execution or authorization surface.

pub const SUBSPACE_RANK_RESIDUAL_SCALE: f64 = 1.0e-10;
pub const SUBSPACE_EQUIVALENCE_TOLERANCE: f64 = 1.0e-10;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubspaceError {
    EmptyDataset,
    TooFewRecords,
    EmptyFeatureBlock,
    FeatureWidthMismatch,
    NonFiniteValue,
    ZeroRank,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TwoBlockSubspaceAudit {
    left_rank: usize,
    right_rank: usize,
    joint_rank: usize,
    right_outside_left_rms_ratio: f64,
    right_outside_left_max_ratio: f64,
    left_outside_right_rms_ratio: f64,
    left_outside_right_max_ratio: f64,
    equivalent_within_tolerance: bool,
}

impl TwoBlockSubspaceAudit {
    #[must_use]
    pub const fn left_rank(self) -> usize {
        self.left_rank
    }

    #[must_use]
    pub const fn right_rank(self) -> usize {
        self.right_rank
    }

    #[must_use]
    pub const fn joint_rank(self) -> usize {
        self.joint_rank
    }

    #[must_use]
    pub const fn right_outside_left_rms_ratio(self) -> f64 {
        self.right_outside_left_rms_ratio
    }

    #[must_use]
    pub const fn right_outside_left_max_ratio(self) -> f64 {
        self.right_outside_left_max_ratio
    }

    #[must_use]
    pub const fn left_outside_right_rms_ratio(self) -> f64 {
        self.left_outside_right_rms_ratio
    }

    #[must_use]
    pub const fn left_outside_right_max_ratio(self) -> f64 {
        self.left_outside_right_max_ratio
    }

    #[must_use]
    pub const fn equivalent_within_tolerance(self) -> bool {
        self.equivalent_within_tolerance
    }

    #[must_use]
    pub fn canonical_report(self) -> String {
        format!(
            concat!(
                "left_rank={};right_rank={};joint_rank={};",
                "right_outside_left_rms_ratio_bits={:016x};",
                "right_outside_left_max_ratio_bits={:016x};",
                "left_outside_right_rms_ratio_bits={:016x};",
                "left_outside_right_max_ratio_bits={:016x};",
                "equivalent_within_tolerance={}"
            ),
            self.left_rank,
            self.right_rank,
            self.joint_rank,
            self.right_outside_left_rms_ratio.to_bits(),
            self.right_outside_left_max_ratio.to_bits(),
            self.left_outside_right_rms_ratio.to_bits(),
            self.left_outside_right_max_ratio.to_bits(),
            self.equivalent_within_tolerance,
        )
    }
}

fn validate_rows(rows: &[Vec<f64>]) -> Result<usize, SubspaceError> {
    let first = rows.first().ok_or(SubspaceError::EmptyDataset)?;
    if rows.len() < 3 {
        return Err(SubspaceError::TooFewRecords);
    }
    if first.is_empty() {
        return Err(SubspaceError::EmptyFeatureBlock);
    }
    let width = first.len();
    for row in rows {
        if row.len() != width {
            return Err(SubspaceError::FeatureWidthMismatch);
        }
        if row.iter().any(|value| !value.is_finite()) {
            return Err(SubspaceError::NonFiniteValue);
        }
    }
    Ok(width)
}

fn projection(left: &[f64], right: &[f64]) -> f64 {
    left.iter().zip(right).map(|(a, b)| a * b).sum()
}

fn standardized_columns(rows: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, SubspaceError> {
    let width = validate_rows(rows)?;
    let n = rows.len();
    let mut columns = Vec::with_capacity(width);
    for column in 0..width {
        let mean = rows.iter().map(|row| row[column]).sum::<f64>() / n as f64;
        let mut values = rows
            .iter()
            .map(|row| row[column] - mean)
            .collect::<Vec<_>>();
        let rms = (projection(&values, &values) / n as f64).sqrt();
        if !rms.is_finite() {
            return Err(SubspaceError::NonFiniteValue);
        }
        if rms == 0.0 {
            continue;
        }
        for value in &mut values {
            *value /= rms;
        }
        columns.push(values);
    }
    if columns.is_empty() {
        return Err(SubspaceError::ZeroRank);
    }
    Ok(columns)
}

fn remove_basis_components(vector: &mut [f64], basis: &[Vec<f64>]) {
    for _ in 0..2 {
        for direction in basis {
            let coefficient = projection(vector, direction);
            for (value, direction_value) in vector.iter_mut().zip(direction) {
                *value -= coefficient * direction_value;
            }
        }
    }
}

fn orthonormal_basis(columns: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, SubspaceError> {
    let n = columns.first().ok_or(SubspaceError::ZeroRank)?.len();
    if n < 3 || columns.iter().any(|column| column.len() != n) {
        return Err(SubspaceError::FeatureWidthMismatch);
    }
    let threshold = SUBSPACE_RANK_RESIDUAL_SCALE * (n as f64).sqrt();
    let mut basis = Vec::<Vec<f64>>::new();
    for column in columns {
        let mut candidate = column.clone();
        remove_basis_components(&mut candidate, &basis);
        let norm = projection(&candidate, &candidate).sqrt();
        if !norm.is_finite() {
            return Err(SubspaceError::NonFiniteValue);
        }
        if norm > threshold {
            for value in &mut candidate {
                *value /= norm;
            }
            basis.push(candidate);
        }
    }
    if basis.is_empty() {
        return Err(SubspaceError::ZeroRank);
    }
    Ok(basis)
}

fn residual_ratios(
    columns: &[Vec<f64>],
    basis: &[Vec<f64>],
) -> Result<(f64, f64), SubspaceError> {
    let mut residual_energy = 0.0;
    let mut source_energy = 0.0;
    let mut maximum = 0.0_f64;
    for column in columns {
        let source_norm = projection(column, column).sqrt();
        if !source_norm.is_finite() || source_norm == 0.0 {
            return Err(SubspaceError::NonFiniteValue);
        }
        let mut residual = column.clone();
        remove_basis_components(&mut residual, basis);
        let residual_norm = projection(&residual, &residual).sqrt();
        if !residual_norm.is_finite() {
            return Err(SubspaceError::NonFiniteValue);
        }
        residual_energy += residual_norm * residual_norm;
        source_energy += source_norm * source_norm;
        maximum = maximum.max(residual_norm / source_norm);
    }
    if source_energy == 0.0 || !source_energy.is_finite() {
        return Err(SubspaceError::ZeroRank);
    }
    Ok(((residual_energy / source_energy).sqrt(), maximum))
}

pub fn audit_two_blocks(
    left_rows: &[Vec<f64>],
    right_rows: &[Vec<f64>],
) -> Result<TwoBlockSubspaceAudit, SubspaceError> {
    if left_rows.len() != right_rows.len() {
        return Err(SubspaceError::FeatureWidthMismatch);
    }
    let left_columns = standardized_columns(left_rows)?;
    let right_columns = standardized_columns(right_rows)?;
    let left_basis = orthonormal_basis(&left_columns)?;
    let right_basis = orthonormal_basis(&right_columns)?;
    let mut joint_columns = left_columns.clone();
    joint_columns.extend(right_columns.iter().cloned());
    let joint_basis = orthonormal_basis(&joint_columns)?;
    let (right_rms, right_max) = residual_ratios(&right_columns, &left_basis)?;
    let (left_rms, left_max) = residual_ratios(&left_columns, &right_basis)?;
    let equivalent = left_basis.len() == right_basis.len()
        && left_basis.len() == joint_basis.len()
        && right_max <= SUBSPACE_EQUIVALENCE_TOLERANCE
        && left_max <= SUBSPACE_EQUIVALENCE_TOLERANCE;
    Ok(TwoBlockSubspaceAudit {
        left_rank: left_basis.len(),
        right_rank: right_basis.len(),
        joint_rank: joint_basis.len(),
        right_outside_left_rms_ratio: right_rms,
        right_outside_left_max_ratio: right_max,
        left_outside_right_rms_ratio: left_rms,
        left_outside_right_max_ratio: left_max,
        equivalent_within_tolerance: equivalent,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invertible_feature_reparameterization_has_same_subspace() {
        let mut left = Vec::new();
        let mut right = Vec::new();
        for index in 0..32 {
            let x = index as f64 - 15.5;
            let z = ((index * 7 + 3) % 19) as f64 - 9.0;
            left.push(vec![x, z]);
            right.push(vec![x + z, 2.0 * x - z]);
        }
        let audit = audit_two_blocks(&left, &right).unwrap();
        assert_eq!(audit.left_rank(), 2);
        assert_eq!(audit.right_rank(), 2);
        assert_eq!(audit.joint_rank(), 2);
        assert!(audit.right_outside_left_max_ratio() < 1.0e-12);
        assert!(audit.left_outside_right_max_ratio() < 1.0e-12);
        assert!(audit.equivalent_within_tolerance());
    }

    #[test]
    fn independent_extra_direction_increases_joint_rank() {
        let mut left = Vec::new();
        let mut right = Vec::new();
        for index in 0..32 {
            let x = index as f64 - 15.5;
            let z = ((index * 7 + 3) % 19) as f64 - 9.0;
            left.push(vec![x]);
            right.push(vec![z]);
        }
        let audit = audit_two_blocks(&left, &right).unwrap();
        assert_eq!(audit.left_rank(), 1);
        assert_eq!(audit.right_rank(), 1);
        assert_eq!(audit.joint_rank(), 2);
        assert!(!audit.equivalent_within_tolerance());
    }

    #[test]
    fn redundant_columns_do_not_manufacture_rank() {
        let mut left = Vec::new();
        let mut right = Vec::new();
        for index in 0..32 {
            let x = index as f64 - 15.5;
            let z = ((index * 5 + 1) % 17) as f64 - 8.0;
            left.push(vec![x, 2.0 * x, z]);
            right.push(vec![x + z, x - z]);
        }
        let audit = audit_two_blocks(&left, &right).unwrap();
        assert_eq!(audit.left_rank(), 2);
        assert_eq!(audit.right_rank(), 2);
        assert_eq!(audit.joint_rank(), 2);
        assert!(audit.equivalent_within_tolerance());
    }

    #[test]
    fn constant_blocks_fail_closed() {
        assert_eq!(
            audit_two_blocks(&vec![vec![1.0]; 8], &vec![vec![2.0]; 8]),
            Err(SubspaceError::ZeroRank)
        );
    }

    #[test]
    fn source_has_no_final_holdout_authorization_secret() {
        let source = include_str!("subspace_v7.rs");
        let confirmation = ["I_ACCEPT_THE_TDI7_", "HOLDOUT_FREEZE"].concat();
        let environment = ["TDI7_CONFIRM_FINAL_", "HOLDOUT"].concat();
        assert!(!source.contains(&confirmation));
        assert!(!source.contains(&environment));
    }
}
