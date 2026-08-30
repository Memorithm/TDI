//! Paired generator-level bootstrap mechanics for TDI-7.1.
//!
//! The same sampled generator indices are used for B0 and B1 in every replicate.
//! This binary operates only on bounded preflight squared-error fixtures.

const BOOTSTRAP_REPLICATES: usize = 2_000;
const BOOTSTRAP_SEED: u64 = 0x5444_4937_4253_0001;

#[derive(Clone, Copy, Debug, PartialEq)]
struct PairedError {
    generator_id: u64,
    b0_squared_error: f64,
    b1_squared_error: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct BootstrapSummary {
    relative_mse_reduction: f64,
    lower_95: f64,
    upper_95: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BootstrapError {
    EmptyInput,
    NonFiniteError,
    NonPositiveBaselineMse,
}

#[derive(Clone, Copy, Debug)]
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn index(&mut self, len: usize) -> usize {
        (self.next_u64() % len as u64) as usize
    }
}

fn relative_reduction(b0_mse: f64, b1_mse: f64) -> Result<f64, BootstrapError> {
    if !b0_mse.is_finite() || !b1_mse.is_finite() {
        return Err(BootstrapError::NonFiniteError);
    }
    if b0_mse <= 0.0 {
        return Err(BootstrapError::NonPositiveBaselineMse);
    }
    Ok((b0_mse - b1_mse) / b0_mse)
}

fn mean_errors(records: &[PairedError]) -> Result<(f64, f64), BootstrapError> {
    if records.is_empty() {
        return Err(BootstrapError::EmptyInput);
    }
    if records
        .iter()
        .any(|record| !record.b0_squared_error.is_finite() || !record.b1_squared_error.is_finite())
    {
        return Err(BootstrapError::NonFiniteError);
    }
    let count = records.len() as f64;
    Ok((
        records
            .iter()
            .map(|record| record.b0_squared_error)
            .sum::<f64>()
            / count,
        records
            .iter()
            .map(|record| record.b1_squared_error)
            .sum::<f64>()
            / count,
    ))
}

fn percentile(sorted: &[f64], probability: f64) -> f64 {
    assert!(!sorted.is_empty());
    let position = probability * (sorted.len() - 1) as f64;
    let lower = position.floor() as usize;
    let upper = position.ceil() as usize;
    if lower == upper {
        sorted[lower]
    } else {
        let weight = position - lower as f64;
        sorted[lower] * (1.0 - weight) + sorted[upper] * weight
    }
}

fn paired_bootstrap(records: &[PairedError]) -> Result<BootstrapSummary, BootstrapError> {
    let (b0_mse, b1_mse) = mean_errors(records)?;
    let point = relative_reduction(b0_mse, b1_mse)?;
    let mut rng = SplitMix64::new(BOOTSTRAP_SEED);
    let mut reductions = Vec::with_capacity(BOOTSTRAP_REPLICATES);

    for _ in 0..BOOTSTRAP_REPLICATES {
        let mut b0_sum = 0.0;
        let mut b1_sum = 0.0;
        for _ in 0..records.len() {
            let record = records[rng.index(records.len())];
            b0_sum += record.b0_squared_error;
            b1_sum += record.b1_squared_error;
        }
        let count = records.len() as f64;
        reductions.push(relative_reduction(b0_sum / count, b1_sum / count)?);
    }

    reductions.sort_by(f64::total_cmp);
    Ok(BootstrapSummary {
        relative_mse_reduction: point,
        lower_95: percentile(&reductions, 0.025),
        upper_95: percentile(&reductions, 0.975),
    })
}

fn preflight_fixture() -> Vec<PairedError> {
    (0_u32..32)
        .map(|index| {
            let scale = 1.0 + f64::from(index) / 64.0;
            PairedError {
                generator_id: 7_100_000_000 + u64::from(index),
                b0_squared_error: scale,
                b1_squared_error: 0.9 * scale,
            }
        })
        .collect()
}

fn main() {
    let summary = paired_bootstrap(&preflight_fixture()).expect("valid bounded fixture");
    println!("TDI-7.1 paired-bootstrap preflight: PASS");
    println!("relative_reduction={:.12}", summary.relative_mse_reduction);
    println!(
        "interval=[{:.12}, {:.12}]",
        summary.lower_95, summary.upper_95
    );
    println!("TDI-7.2 final holdout: NOT ACCESSED");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_arms_have_zero_reduction_and_zero_interval() {
        let records: Vec<_> = (0..16)
            .map(|index| PairedError {
                generator_id: index,
                b0_squared_error: 1.0 + index as f64,
                b1_squared_error: 1.0 + index as f64,
            })
            .collect();
        let summary = paired_bootstrap(&records).unwrap();
        assert_eq!(summary.relative_mse_reduction, 0.0);
        assert_eq!(summary.lower_95, 0.0);
        assert_eq!(summary.upper_95, 0.0);
    }

    #[test]
    fn proportional_improvement_is_preserved_by_every_paired_resample() {
        let summary = paired_bootstrap(&preflight_fixture()).unwrap();
        assert!((summary.relative_mse_reduction - 0.1).abs() <= 1.0e-12);
        assert!((summary.lower_95 - 0.1).abs() <= 1.0e-12);
        assert!((summary.upper_95 - 0.1).abs() <= 1.0e-12);
    }

    #[test]
    fn bootstrap_is_bitwise_deterministic_for_same_input() {
        let records = preflight_fixture();
        assert_eq!(paired_bootstrap(&records), paired_bootstrap(&records));
    }

    #[test]
    fn pairing_is_generator_level_not_arm_level() {
        let records = preflight_fixture();
        for record in &records {
            assert_eq!(record.b1_squared_error, 0.9 * record.b0_squared_error);
        }
        let summary = paired_bootstrap(&records).unwrap();
        assert!((summary.lower_95 - summary.upper_95).abs() <= 1.0e-12);
    }

    #[test]
    fn invalid_baseline_mse_fails_closed() {
        let records = vec![PairedError {
            generator_id: 1,
            b0_squared_error: 0.0,
            b1_squared_error: 0.0,
        }];
        assert_eq!(
            paired_bootstrap(&records),
            Err(BootstrapError::NonPositiveBaselineMse)
        );
    }

    #[test]
    fn nonfinite_errors_fail_closed() {
        let records = vec![PairedError {
            generator_id: 1,
            b0_squared_error: f64::NAN,
            b1_squared_error: 1.0,
        }];
        assert_eq!(
            paired_bootstrap(&records),
            Err(BootstrapError::NonFiniteError)
        );
    }

    #[test]
    fn source_has_no_holdout_authorization_token() {
        let source = include_str!("tdi-attention-v71-bootstrap.rs");
        let confirmation = ["I_ACCEPT_THE_TDI7_", "HOLDOUT_FREEZE"].concat();
        let environment = ["TDI7_CONFIRM_FINAL_", "HOLDOUT"].concat();
        assert!(!source.contains(&confirmation));
        assert!(!source.contains(&environment));
    }
}
