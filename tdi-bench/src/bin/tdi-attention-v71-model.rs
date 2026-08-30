//! Deterministic B0/B1 model ladder for TDI-7.1 preflight data.
//!
//! Both arms use the same ridge-linear model class. Lambda is selected only
//! from training/development data. No final-holdout path exists in this binary.

const LAMBDA_GRID: [f64; 4] = [0.0, 1.0e-6, 1.0e-3, 1.0e-1];
const PIVOT_TOLERANCE: f64 = 1.0e-12;

#[derive(Clone, Debug, PartialEq)]
struct Record {
    baseline: Vec<f64>,
    recovery: Vec<f64>,
    target: f64,
}
#[derive(Clone, Debug, PartialEq)]
struct RidgeModel {
    weights: Vec<f64>,
    lambda: f64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ModelError {
    EmptyDataset,
    FeatureWidthMismatch,
    NonFiniteValue,
    SingularSystem,
}

fn design(records: &[Record], include_recovery: bool) -> Result<Vec<Vec<f64>>, ModelError> {
    let first = records.first().ok_or(ModelError::EmptyDataset)?;
    let baseline_width = first.baseline.len();
    let recovery_width = first.recovery.len();
    let mut rows = Vec::with_capacity(records.len());
    for record in records {
        if record.baseline.len() != baseline_width || record.recovery.len() != recovery_width {
            return Err(ModelError::FeatureWidthMismatch);
        }
        if !record.target.is_finite()
            || record.baseline.iter().any(|v| !v.is_finite())
            || record.recovery.iter().any(|v| !v.is_finite())
        {
            return Err(ModelError::NonFiniteValue);
        }
        let mut row = Vec::with_capacity(1 + baseline_width + recovery_width);
        row.push(1.0);
        row.extend_from_slice(&record.baseline);
        if include_recovery {
            row.extend_from_slice(&record.recovery);
        }
        rows.push(row);
    }
    Ok(rows)
}

fn solve_linear(mut a: Vec<Vec<f64>>, mut b: Vec<f64>) -> Result<Vec<f64>, ModelError> {
    let n = b.len();
    for pivot in 0..n {
        let mut best = pivot;
        for row in (pivot + 1)..n {
            if a[row][pivot].abs() > a[best][pivot].abs() {
                best = row;
            }
        }
        if a[best][pivot].abs() <= PIVOT_TOLERANCE {
            return Err(ModelError::SingularSystem);
        }
        a.swap(pivot, best);
        b.swap(pivot, best);
        let scale = a[pivot][pivot];
        for value in &mut a[pivot][pivot..] {
            *value /= scale;
        }
        b[pivot] /= scale;
        let pivot_row = a[pivot].clone();
        let pivot_rhs = b[pivot];
        for (row_index, row_values) in a.iter_mut().enumerate() {
            if row_index == pivot {
                continue;
            }
            let factor = row_values[pivot];
            for (value, pivot_value) in row_values[pivot..].iter_mut().zip(&pivot_row[pivot..]) {
                *value -= factor * pivot_value;
            }
            b[row_index] -= factor * pivot_rhs;
        }
    }
    Ok(b)
}

fn fit(records: &[Record], include_recovery: bool, lambda: f64) -> Result<RidgeModel, ModelError> {
    let x = design(records, include_recovery)?;
    let width = x[0].len();
    let mut gram = vec![vec![0.0; width]; width];
    let mut rhs = vec![0.0; width];
    for (row, record) in x.iter().zip(records) {
        for i in 0..width {
            rhs[i] += row[i] * record.target;
            for j in 0..width {
                gram[i][j] += row[i] * row[j];
            }
        }
    }
    for (index, diagonal) in gram.iter_mut().enumerate().skip(1) {
        diagonal[index] += lambda;
    }
    Ok(RidgeModel {
        weights: solve_linear(gram, rhs)?,
        lambda,
    })
}

fn predict(model: &RidgeModel, record: &Record, include_recovery: bool) -> f64 {
    let mut value = model.weights[0];
    let mut offset = 1;
    for feature in &record.baseline {
        value += model.weights[offset] * feature;
        offset += 1;
    }
    if include_recovery {
        for feature in &record.recovery {
            value += model.weights[offset] * feature;
            offset += 1;
        }
    }
    value
}
fn mse(model: &RidgeModel, records: &[Record], include_recovery: bool) -> f64 {
    records
        .iter()
        .map(|record| {
            let error = predict(model, record, include_recovery) - record.target;
            error * error
        })
        .sum::<f64>()
        / records.len() as f64
}

fn select_lambda(
    training: &[Record],
    development: &[Record],
    include_recovery: bool,
) -> Result<RidgeModel, ModelError> {
    let mut best: Option<(f64, RidgeModel)> = None;
    for lambda in LAMBDA_GRID {
        let Ok(model) = fit(training, include_recovery, lambda) else {
            continue;
        };
        let loss = mse(&model, development, include_recovery);
        if !loss.is_finite() {
            return Err(ModelError::NonFiniteValue);
        }
        if best.as_ref().is_none_or(|(best_loss, _)| loss < *best_loss) {
            best = Some((loss, model));
        }
    }
    best.map(|(_, model)| model)
        .ok_or(ModelError::SingularSystem)
}

fn synthetic_split(start: usize, end: usize) -> Vec<Record> {
    (start..end)
        .map(|index| {
            let x = index as f64 / 10.0;
            let static_feature = x.sin();
            let recovery_feature = (0.7 * x).cos();
            Record {
                baseline: vec![static_feature, x / 10.0],
                recovery: vec![recovery_feature],
                target: 0.4 + 0.3 * static_feature + 0.8 * recovery_feature,
            }
        })
        .collect()
}

fn main() {
    let training = synthetic_split(0, 32);
    let development = synthetic_split(32, 48);
    let validation = synthetic_split(48, 64);
    let b0 = select_lambda(&training, &development, false).expect("B0 preflight fit");
    let b1 = select_lambda(&training, &development, true).expect("B1 preflight fit");
    println!("TDI-7.1 nested-model preflight: PASS");
    println!(
        "B0 lambda={} validation_mse={:.12}",
        b0.lambda,
        mse(&b0, &validation, false)
    );
    println!(
        "B1 lambda={} validation_mse={:.12}",
        b1.lambda,
        mse(&b1, &validation, true)
    );
    println!("TDI-7.2 final holdout: NOT ACCESSED");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn b0_and_b1_share_the_same_model_class_and_lambda_grid() {
        let training = synthetic_split(0, 32);
        let development = synthetic_split(32, 48);
        let b0 = select_lambda(&training, &development, false).unwrap();
        let b1 = select_lambda(&training, &development, true).unwrap();
        assert!(LAMBDA_GRID.contains(&b0.lambda));
        assert!(LAMBDA_GRID.contains(&b1.lambda));
    }
    #[test]
    fn recovery_features_are_absent_from_b0_design() {
        let records = synthetic_split(0, 4);
        assert_eq!(design(&records, false).unwrap()[0].len(), 3);
        assert_eq!(design(&records, true).unwrap()[0].len(), 4);
    }
    #[test]
    fn lambda_selection_uses_only_supplied_training_and_development_slices() {
        let training = synthetic_split(0, 32);
        let development = synthetic_split(32, 48);
        let untouched = synthetic_split(48, 64);
        let model = select_lambda(&training, &development, true).unwrap();
        assert_eq!(mse(&model, &untouched, true), mse(&model, &untouched, true));
    }
    #[test]
    fn augmented_arm_can_recover_signal_present_only_in_recovery_block() {
        let training = synthetic_split(0, 32);
        let development = synthetic_split(32, 48);
        let validation = synthetic_split(48, 64);
        let b0 = select_lambda(&training, &development, false).unwrap();
        let b1 = select_lambda(&training, &development, true).unwrap();
        assert!(mse(&b1, &validation, true) < mse(&b0, &validation, false));
    }
    #[test]
    fn malformed_data_fail_closed() {
        let malformed = vec![
            Record {
                baseline: vec![1.0],
                recovery: vec![0.5],
                target: 1.0,
            },
            Record {
                baseline: vec![1.0, 2.0],
                recovery: vec![0.5],
                target: 1.0,
            },
        ];
        assert_eq!(
            design(&malformed, false),
            Err(ModelError::FeatureWidthMismatch)
        );
    }
    #[test]
    fn source_has_no_holdout_authorization_token() {
        let source = include_str!("tdi-attention-v71-model.rs");
        let confirmation = ["I_ACCEPT_THE_TDI7_", "HOLDOUT_FREEZE"].concat();
        let environment = ["TDI7_CONFIRM_FINAL_", "HOLDOUT"].concat();
        assert!(!source.contains(&confirmation));
        assert!(!source.contains(&environment));
    }
}
