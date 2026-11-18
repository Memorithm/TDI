//! Experimental non-final TDI-22 torsor attention row with online softmax.
//!
//! TDI-22 fixes M(Q) = C - Q x R, with C = M(P) + P x R.
//! A query (v, omega) at Q scores a prepared key (R, C) as
//! (v + Q x omega).R + omega.C. Both the score and the weighted
//! torsor-value reduction therefore need no per-pair Varignon transport.
//! This is an exact algebraic factorization, NOT a claim of improved
//! model quality, reduced asymptotic time, or physical bandwidth.
//!
//! Keys are prepared once; each streamed row uses constant scratch space.
//! For N queries over N keys the time remains O(N^2), excluding preparation.
//! Nothing here changes the frozen TDI-22 evaluator or FLAT routing.

use super::tdi22_torsor::{
    FactorizedTorsorKey, Torsor3, TorsorError, Twist3, Vec3, direct_pairing,
};
use super::tdi24_attention::MaskPolicy;

/// Versioned, non-final reference algorithm (not a production kernel).
pub const STREAMING_CONTRACT: &str = "tdi22-factorized-streaming-row-v1";

/// A validated, origin-reduced key. Construction is only via a torsor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PreparedTorsorKey {
    key: FactorizedTorsorKey,
}

impl PreparedTorsorKey {
    /// Reduce the key once, rejecting non-finite derived values.
    pub fn prepare(torsor: Torsor3) -> Result<Self, StreamError> {
        Ok(Self {
            key: torsor.factorized_key()?,
        })
    }
}

/// Prepare an ordered nonempty key bank without changing its content.
pub fn prepare_keys(keys: &[Torsor3]) -> Result<Vec<PreparedTorsorKey>, StreamError> {
    if keys.is_empty() {
        return Err(StreamError::EmptyKeys);
    }
    keys.iter().copied().map(PreparedTorsorKey::prepare).collect()
}

/// One normalized attention row, expressed at the query position Q.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StreamedTorsorRow {
    /// Weighted sum of resultants.
    pub resultant: Vec3,
    /// Weighted sum of transported moments at Q.
    pub moment_at_query: Vec3,
    /// Natural log of the sum of unnormalized score exponentials.
    pub log_sum_exp: f64,
    /// Number of visible keys, including those whose weight underflows.
    pub visible_keys: usize,
}

/// Explicit non-final software failures; no silent numerical fallback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamError {
    /// No keys were provided.
    EmptyKeys,
    /// The requested score multiplier is not finite or positive.
    InvalidScale,
    /// The existing TDI-22 torsor algebra rejected a value.
    Torsor(TorsorError),
    /// Finite inputs nevertheless overflowed during online normalization.
    NonFiniteDerived,
}

impl From<TorsorError> for StreamError {
    fn from(error: TorsorError) -> Self {
        Self::Torsor(error)
    }
}

fn checked_scale(v: Vec3, scalar: f64) -> Result<Vec3, StreamError> {
    Ok(Vec3::new(v.x * scalar, v.y * scalar, v.z * scalar)?)
}

fn checked_add(a: Vec3, b: Vec3) -> Result<Vec3, StreamError> {
    Ok(Vec3::new(a.x + b.x, a.y + b.y, a.z + b.z)?)
}

/// Compute one complete row using only scalar scratch accumulators.
///
/// The operation order is deterministic. The caller supplies an explicit
/// positive score scale, e.g. 1/sqrt(6) for a six-channel pairing.
/// MaskPolicy::Causal includes key i exactly when i <= query_index.
/// No N-by-N score/probability matrix and no per-pair moment transport
/// are created. Masked keys are still validated at preparation time.
///
/// This sums origin-reduced moments C and resultants R with the SAME
/// online-softmax weights, then transports the sum to Q once. Under exact
/// arithmetic this equals weighting individually transported moments.
pub fn stream_torsor_row(
    query: Twist3,
    query_position: Vec3,
    keys: &[PreparedTorsorKey],
    query_index: usize,
    policy: MaskPolicy,
    score_scale: f64,
) -> Result<StreamedTorsorRow, StreamError> {
    if keys.is_empty() {
        return Err(StreamError::EmptyKeys);
    }
    if !score_scale.is_finite() || score_scale <= 0.0 {
        return Err(StreamError::InvalidScale);
    }

    let prepared_query = query.factorized_at(query_position)?;
    let mut maximum = f64::NEG_INFINITY;
    let mut denominator = 0.0_f64;
    let mut sum_r = Vec3::zero();
    let mut sum_c = Vec3::zero();
    let mut visible = 0_usize;

    for (i, prepared) in keys.iter().enumerate() {
        if matches!(policy, MaskPolicy::Causal) && i > query_index {
            continue;
        }
        let score = prepared_query.pair(prepared.key)? * score_scale;
        if !score.is_finite() {
            return Err(StreamError::NonFiniteDerived);
        }

        // Online softmax: rescale previous accumulators if the max grows.
        let new_max = maximum.max(score);
        let old_weight = if visible == 0 {
            0.0
        } else {
            (maximum - new_max).exp()
        };
        let new_weight = (score - new_max).exp();
        denominator = denominator * old_weight + new_weight;
        if !denominator.is_finite() || denominator <= 0.0 {
            return Err(StreamError::NonFiniteDerived);
        }
        sum_r = checked_add(
            checked_scale(sum_r, old_weight)?,
            checked_scale(prepared.key.resultant, new_weight)?,
        )?;
        sum_c = checked_add(
            checked_scale(sum_c, old_weight)?,
            checked_scale(prepared.key.origin_moment, new_weight)?,
        )?;
        maximum = new_max;
        visible += 1;
    }

    if visible == 0 {
        return Err(StreamError::EmptyKeys);
    }
    let inv = 1.0 / denominator;
    let resultant = checked_scale(sum_r, inv)?;
    let origin_moment = checked_scale(sum_c, inv)?;
    let moment_at_query = Vec3::new(
        origin_moment.x - query_position.cross(resultant).x,
        origin_moment.y - query_position.cross(resultant).y,
        origin_moment.z - query_position.cross(resultant).z,
    )?;
    let log_sum_exp = maximum + denominator.ln();
    if !log_sum_exp.is_finite() {
        return Err(StreamError::NonFiniteDerived);
    }
    Ok(StreamedTorsorRow {
        resultant,
        moment_at_query,
        log_sum_exp,
        visible_keys: visible,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3::new(x, y, z).unwrap()
    }

    fn fixtures() -> (Twist3, Vec3, Vec<Torsor3>) {
        let q = Twist3::new(v(0.2, -0.5, 0.9), v(0.4, 0.7, -0.3)).unwrap();
        let p = v(-1.2, 0.5, 1.7);
        let keys = vec![
            Torsor3::new(v(2.0, 1.5, -0.5), v(0.1, 0.7, 1.2), v(2.0, 0.0, -3.0)).unwrap(),
            Torsor3::new(v(-1.0, 3.0, 2.0), v(-0.3, 0.9, -0.4), v(0.0, 1.0, 0.0)).unwrap(),
            Torsor3::new(v(0.5, 0.5, 4.0), v(2.3, -1.1, 1.0), v(1.0, -2.0, 2.0)).unwrap(),
            Torsor3::new(v(1.5, -3.0, 0.25), v(-2.0, 0.1, 0.6), v(4.0, 5.0, 6.0)).unwrap(),
        ];
        (q, p, keys)
    }

    fn close(a: f64, b: f64) {
        let scale = 1.0_f64.max(a.abs()).max(b.abs());
        assert!((a - b).abs() <= 2e-12 * scale, "{a} != {b}");
    }

    fn close_vec(a: Vec3, b: Vec3) {
        close(a.x, b.x);
        close(a.y, b.y);
        close(a.z, b.z);
    }

    // Independent dense, direct-pairing softmax oracle using the original
    // reduction points and an individually transported moment for each key.
    fn dense_row(
        query: Twist3,
        q: Vec3,
        keys: &[Torsor3],
        visible: usize,
        scale: f64,
    ) -> StreamedTorsorRow {
        let scores: Vec<f64> = keys[..visible]
            .iter()
            .map(|k| direct_pairing(query, *k, q).unwrap() * scale)
            .collect();
        let max = scores.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let weights: Vec<f64> = scores.iter().map(|s| (s - max).exp()).collect();
        let den: f64 = weights.iter().sum();
        let mut r = Vec3::zero();
        let mut m = Vec3::zero();
        for (&weight, key) in weights.iter().zip(&keys[..visible]) {
            r = r.plus(Vec3 {
                x: key.resultant().x * weight,
                y: key.resultant().y * weight,
                z: key.resultant().z * weight,
            });
            let moment = key.moment_at(q).unwrap();
            m = m.plus(Vec3 {
                x: moment.x * weight,
                y: moment.y * weight,
                z: moment.z * weight,
            });
        }
        StreamedTorsorRow {
            resultant: v(r.x / den, r.y / den, r.z / den),
            moment_at_query: v(m.x / den, m.y / den, m.z / den),
            log_sum_exp: max + den.ln(),
            visible_keys: visible,
        }
    }

    #[test]
    fn factorized_stream_matches_direct_dense_full_and_causal() {
        let (query, p, keys) = fixtures();
        let prepared = prepare_keys(&keys).unwrap();
        for (policy, index, count) in [
            (MaskPolicy::Full, 0, 4),
            (MaskPolicy::Causal, 0, 1),
            (MaskPolicy::Causal, 1, 2),
            (MaskPolicy::Causal, 3, 4),
        ] {
            let actual = stream_torsor_row(query, p, &prepared, index, policy, 0.5).unwrap();
            let oracle = dense_row(query, p, &keys, count, 0.5);
            assert_eq!(actual.visible_keys, count);
            close_vec(actual.resultant, oracle.resultant);
            close_vec(actual.moment_at_query, oracle.moment_at_query);
            close(actual.log_sum_exp, oracle.log_sum_exp);
        }
    }

    #[test]
    fn source_reduction_and_common_translation_preserve_output() {
        let (query, p, keys) = fixtures();
        let original = stream_torsor_row(
            query, p, &prepare_keys(&keys).unwrap(), 0, MaskPolicy::Full, 1.0,
        ).unwrap();
        let t = v(11.0, -17.0, 23.0);
        let transformed: Vec<_> = keys
            .iter()
            .map(|key| {
                // Transport first to a different reduction point, then
                // translate all coordinate origins without changing M there.
                let moved = key.transport(key.reference().plus(v(0.75, 2.0, -1.0))).unwrap();
                Torsor3::new(moved.resultant(), moved.moment(), moved.reference().plus(t)).unwrap()
            })
            .collect();
        let relocated = stream_torsor_row(
            query, p.plus(t), &prepare_keys(&transformed).unwrap(),
            0, MaskPolicy::Full, 1.0,
        ).unwrap();
        close_vec(original.resultant, relocated.resultant);
        close_vec(original.moment_at_query, relocated.moment_at_query);
        close(original.log_sum_exp, relocated.log_sum_exp);
    }

    #[test]
    fn large_logits_remain_stable_and_future_keys_are_excluded() {
        let (query, p, mut keys) = fixtures();
        // Extreme finite scores: the exponentials of raw logits would overflow.
        let q = Twist3::new(v(50_000.0, 0.0, 0.0), query.angular()).unwrap();
        let prepared = prepare_keys(&keys).unwrap();
        let stable = stream_torsor_row(q, p, &prepared, 3, MaskPolicy::Full, 1.0).unwrap();
        assert!(stable.log_sum_exp.is_finite());
        assert!(stable.resultant.x.is_finite());
        let causal = stream_torsor_row(q, p, &prepared, 0, MaskPolicy::Causal, 1.0).unwrap();
        close_vec(causal.resultant, keys[0].resultant());
        close_vec(causal.moment_at_query, keys[0].moment_at(p).unwrap());
        assert_eq!(causal.visible_keys, 1);
        // Changing a masked future key cannot alter the causal result.
        keys[3] = Torsor3::new(v(1e12, 0.0, 0.0), v(0.0, 0.0, 0.0), p).unwrap();
        let altered = stream_torsor_row(
            q, p, &prepare_keys(&keys).unwrap(), 0, MaskPolicy::Causal, 1.0,
        ).unwrap();
        assert_eq!(causal, altered);
    }

    #[test]
    fn invalid_or_overflowing_inputs_fail_closed() {
        let (query, p, keys) = fixtures();
        assert_eq!(prepare_keys(&[]), Err(StreamError::EmptyKeys));
        let prepared = prepare_keys(&keys).unwrap();
        assert_eq!(
            stream_torsor_row(query, p, &prepared, 0, MaskPolicy::Full, f64::NAN),
            Err(StreamError::InvalidScale)
        );
        assert_eq!(
            stream_torsor_row(query, p, &prepared, 0, MaskPolicy::Full, 0.0),
            Err(StreamError::InvalidScale)
        );
        let huge = Torsor3::new(v(f64::MAX, 0.0, 0.0), v(0.0, 1.0, 0.0), v(0.0, 0.0, 2.0)).unwrap();
        assert!(prepare_keys(&[huge]).is_err());
        assert!(stream_torsor_row(
            query, p, &prepared, 0, MaskPolicy::Full, f64::MAX
        ).is_err());
    }
}
