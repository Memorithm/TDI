//! Calibration diagnostics for TDI-2.1 experience-conditioned confidence.

/// One observed prediction with a probability-like reliability estimate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CalibrationObservation {
    /// Frozen predicted reliability in `[0, 1]`.
    pub predicted: f64,
    /// Whether the prediction was correct.
    pub correct: bool,
}

/// One equal-width calibration bin.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CalibrationBin {
    /// Number of observations in this bin.
    pub count: usize,
    /// Mean predicted reliability.
    pub mean_prediction: f64,
    /// Empirical correctness rate.
    pub empirical_accuracy: f64,
}

/// Calibration validation errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CalibrationError {
    /// At least one prediction was non-finite or outside `[0, 1]`.
    InvalidPrediction,
    /// Bin count must be positive.
    ZeroBins,
}

impl core::fmt::Display for CalibrationError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidPrediction => {
                formatter.write_str("calibration predictions must be finite values in [0, 1]")
            }
            Self::ZeroBins => formatter.write_str("calibration bin count must be positive"),
        }
    }
}
impl std::error::Error for CalibrationError {}

fn validate_observations(observations: &[CalibrationObservation]) -> Result<(), CalibrationError> {
    if observations.iter().any(|observation| {
        !observation.predicted.is_finite()
            || observation.predicted < 0.0
            || observation.predicted > 1.0
    }) {
        return Err(CalibrationError::InvalidPrediction);
    }
    Ok(())
}

/// Compute non-empty equal-width calibration bins.
pub fn calibration_bins(
    observations: &[CalibrationObservation],
    bins: usize,
) -> Result<Vec<CalibrationBin>, CalibrationError> {
    if bins == 0 {
        return Err(CalibrationError::ZeroBins);
    }
    validate_observations(observations)?;

    let mut counts = vec![0usize; bins];
    let mut prediction_sums = vec![0.0f64; bins];
    let mut correct_sums = vec![0usize; bins];
    for observation in observations {
        let mut index = (observation.predicted * bins as f64).floor() as usize;
        if index == bins {
            index = bins - 1;
        }
        counts[index] += 1;
        prediction_sums[index] += observation.predicted;
        correct_sums[index] += usize::from(observation.correct);
    }

    Ok((0..bins)
        .filter(|&index| counts[index] > 0)
        .map(|index| CalibrationBin {
            count: counts[index],
            mean_prediction: prediction_sums[index] / counts[index] as f64,
            empirical_accuracy: correct_sums[index] as f64 / counts[index] as f64,
        })
        .collect())
}

/// Expected calibration error weighted by observed bin mass.
pub fn expected_calibration_error(
    observations: &[CalibrationObservation],
    bins: usize,
) -> Result<Option<f64>, CalibrationError> {
    if observations.is_empty() {
        if bins == 0 {
            return Err(CalibrationError::ZeroBins);
        }
        return Ok(None);
    }
    let summaries = calibration_bins(observations, bins)?;
    let total = observations.len() as f64;
    Ok(Some(
        summaries
            .iter()
            .map(|bin| {
                (bin.count as f64 / total) * (bin.mean_prediction - bin.empirical_accuracy).abs()
            })
            .sum(),
    ))
}

/// Mean Brier score for probability-like reliability estimates.
pub fn brier_score(
    observations: &[CalibrationObservation],
) -> Result<Option<f64>, CalibrationError> {
    validate_observations(observations)?;
    if observations.is_empty() {
        return Ok(None);
    }
    let sum = observations
        .iter()
        .map(|observation| {
            let target = f64::from(u8::from(observation.correct));
            let error = observation.predicted - target;
            error * error
        })
        .sum::<f64>();
    Ok(Some(sum / observations.len() as f64))
}

#[cfg(test)]
mod tests {
    use super::{CalibrationObservation, brier_score, expected_calibration_error};

    #[test]
    fn perfectly_calibrated_two_bin_fixture_has_zero_error() {
        let observations = [
            CalibrationObservation {
                predicted: 0.0,
                correct: false,
            },
            CalibrationObservation {
                predicted: 1.0,
                correct: true,
            },
        ];
        assert_eq!(expected_calibration_error(&observations, 2), Ok(Some(0.0)));
        assert_eq!(brier_score(&observations), Ok(Some(0.0)));
    }

    #[test]
    fn half_confidence_has_quarter_brier_error() {
        let observations = [CalibrationObservation {
            predicted: 0.5,
            correct: true,
        }];
        assert_eq!(brier_score(&observations), Ok(Some(0.25)));
    }
}
