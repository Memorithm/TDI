//! Experimental TDI-22 sequence-block hierarchy with bounded score approximation.
//!
//! A binary sequence tree is chosen intentionally: no spatial Octree or
//! privileged language-token R^3 geometry is assumed. Each node contains
//! its key mean (R,C), component extrema, and count. Fully eligible nodes
//! may substitute one mass-corrected score (mean_score + ln(count)) when
//! their score deviation envelope does not exceed the caller's tolerance.
//!
//! Real-arithmetic guarantee: if all accepted blocks have individual score
//! deviations <= delta, the approximate normalized distribution differs
//! from exact softmax by total variation <= tanh(delta). Component
//! expectation error is then bounded by (global_max - global_min)*tanh(delta).
//! These bounds are analytical; they do NOT include IEEE-754 rounding error.
//! No production, hardware, model-quality, or O(N log N) claim is implied.

use super::tdi22_streaming_attention::{StreamError, StreamedTorsorRow};
use super::tdi22_torsor::{
    FactorizedTorsorKey, FactorizedTwistQuery, Torsor3, TorsorError, Twist3, Vec3,
};
use super::tdi24_attention::MaskPolicy;

/// Opt-in non-final scalar algorithm contract.
pub const HIERARCHICAL_CONTRACT: &str = "tdi22-sequence-block-envelope-v1";

/// Explicit cap to keep software-reference construction bounded.
pub const MAX_HIERARCHICAL_KEYS: usize = 65_536;

/// Bounded failures, never silently falling back or silently promoting a result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HierarchyError {
    EmptyKeys,
    TooManyKeys,
    InvalidScale,
    InvalidTolerance,
    Torsor(TorsorError),
    NonFiniteDerived,
}

impl From<TorsorError> for HierarchyError {
    fn from(value: TorsorError) -> Self {
        Self::Torsor(value)
    }
}

impl From<StreamError> for HierarchyError {
    fn from(value: StreamError) -> Self {
        match value {
            StreamError::Torsor(error) => Self::Torsor(error),
            _ => Self::NonFiniteDerived,
        }
    }
}

/// An analytical, real-arithmetic envelope for each component of the output.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AlgebraicErrorBounds {
    /// Maximum score error of any accepted group, before normalization.
    pub score_deviation: f64,
    /// Bound on |log Z_exact - log Z_grouped|.
    pub log_sum_exp: f64,
    /// Upper bound on the total variation distance for token probabilities.
    pub total_variation: f64,
    /// Per-coordinate absolute error bound for the resultant.
    pub resultant: Vec3,
    /// Per-coordinate absolute error bound for the moment evaluated at Q.
    pub moment_at_query: Vec3,
}

/// Observable deterministic amount of block work; not physical speed evidence.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HierarchyWork {
    /// Nodes examined in the eligible traversal, including partial nodes.
    pub visited_nodes: usize,
    /// Actual six-component pairing scores evaluated.
    pub scored_nodes: usize,
    /// Groups of two or more token keys substituted by a single score.
    pub collapsed_blocks: usize,
    /// Individually scored token leaves.
    pub exact_leaves: usize,
}

/// Approximate row and its explicit algebraic error envelope.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HierarchicalTorsorRow {
    pub row: StreamedTorsorRow,
    pub bounds: AlgebraicErrorBounds,
    pub work: HierarchyWork,
}

#[derive(Clone, Copy, Debug)]
struct Extrema {
    min_r: Vec3,
    max_r: Vec3,
    min_c: Vec3,
    max_c: Vec3,
}

impl Extrema {
    fn singleton(key: FactorizedTorsorKey) -> Self {
        Self {
            min_r: key.resultant,
            max_r: key.resultant,
            min_c: key.origin_moment,
            max_c: key.origin_moment,
        }
    }

    fn union(lhs: Self, rhs: Self) -> Self {
        fn min(a: Vec3, b: Vec3) -> Vec3 {
            Vec3 {
                x: a.x.min(b.x),
                y: a.y.min(b.y),
                z: a.z.min(b.z),
            }
        }
        fn max(a: Vec3, b: Vec3) -> Vec3 {
            Vec3 {
                x: a.x.max(b.x),
                y: a.y.max(b.y),
                z: a.z.max(b.z),
            }
        }
        Self {
            min_r: min(lhs.min_r, rhs.min_r),
            max_r: max(lhs.max_r, rhs.max_r),
            min_c: min(lhs.min_c, rhs.min_c),
            max_c: max(lhs.max_c, rhs.max_c),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct BlockNode {
    start: usize,
    end: usize,
    mean: FactorizedTorsorKey,
    extrema: Extrema,
    children: Option<(usize, usize)>,
}

impl BlockNode {
    fn count(self) -> usize {
        self.end - self.start
    }

    fn leaf(self) -> bool {
        self.children.is_none()
    }
}

/// Immutable, contiguous, postorder binary hierarchy over key sequence order.
#[derive(Clone, Debug)]
pub struct HierarchicalTorsorBank {
    nodes: Vec<BlockNode>,
    root: usize,
}

fn checked_vec(x: f64, y: f64, z: f64) -> Result<Vec3, HierarchyError> {
    Vec3::new(x, y, z).map_err(HierarchyError::from)
}

fn weighted_mean(a: Vec3, b: Vec3, wa: f64, wb: f64) -> Result<Vec3, HierarchyError> {
    checked_vec(
        a.x * wa + b.x * wb,
        a.y * wa + b.y * wb,
        a.z * wa + b.z * wb,
    )
}

fn checked_linear_update(
    old: Vec3,
    old_weight: f64,
    next: Vec3,
    next_weight: f64,
) -> Result<Vec3, HierarchyError> {
    checked_vec(
        old.x * old_weight + next.x * next_weight,
        old.y * old_weight + next.y * next_weight,
        old.z * old_weight + next.z * next_weight,
    )
}

fn push_subtree(
    nodes: &mut Vec<BlockNode>,
    keys: &[FactorizedTorsorKey],
    start: usize,
    end: usize,
) -> Result<usize, HierarchyError> {
    if end - start == 1 {
        let key = keys[start];
        let index = nodes.len();
        nodes.push(BlockNode {
            start,
            end,
            mean: key,
            extrema: Extrema::singleton(key),
            children: None,
        });
        return Ok(index);
    }

    let mid = start + (end - start) / 2;
    let li = push_subtree(nodes, keys, start, mid)?;
    let ri = push_subtree(nodes, keys, mid, end)?;
    let left = nodes[li];
    let right = nodes[ri];
    let weight_left = left.count() as f64 / (end - start) as f64;
    let weight_right = right.count() as f64 / (end - start) as f64;
    let resultant = weighted_mean(
        left.mean.resultant,
        right.mean.resultant,
        weight_left,
        weight_right,
    )?;
    let origin_moment = weighted_mean(
        left.mean.origin_moment,
        right.mean.origin_moment,
        weight_left,
        weight_right,
    )?;
    let index = nodes.len();
    nodes.push(BlockNode {
        start,
        end,
        mean: FactorizedTorsorKey {
            resultant,
            origin_moment,
        },
        extrema: Extrema::union(left.extrema, right.extrema),
        children: Some((li, ri)),
    });
    Ok(index)
}

impl HierarchicalTorsorBank {
    /// Prepare once, then reuse the immutable sequence hierarchy for many queries.
    pub fn build(keys: &[Torsor3]) -> Result<Self, HierarchyError> {
        if keys.is_empty() {
            return Err(HierarchyError::EmptyKeys);
        }
        if keys.len() > MAX_HIERARCHICAL_KEYS {
            return Err(HierarchyError::TooManyKeys);
        }
        let prepared = keys
            .iter()
            .copied()
            .map(Torsor3::factorized_key)
            .collect::<Result<Vec<_>, _>>()?;
        let mut nodes = Vec::with_capacity(2 * keys.len() - 1);
        let root = push_subtree(&mut nodes, &prepared, 0, keys.len())?;
        Ok(Self { nodes, root })
    }

    pub fn key_count(&self) -> usize {
        self.nodes[self.root].count()
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Query with an explicitly bounded per-key score-deviation threshold.
    ///
    /// Zero requests the exact score-homogeneous grouping limit.
    /// Causal queries only accept complete prefix blocks; a partial node
    /// is descended regardless of tolerance and never scores future keys.
    /// Real-arithmetic error bounds are returned; rounding is not enclosed.
    pub fn row(
        &self,
        query: Twist3,
        query_position: Vec3,
        query_index: usize,
        policy: MaskPolicy,
        score_scale: f64,
        max_score_deviation: f64,
    ) -> Result<HierarchicalTorsorRow, HierarchyError> {
        if !score_scale.is_finite() || score_scale <= 0.0 {
            return Err(HierarchyError::InvalidScale);
        }
        if !max_score_deviation.is_finite() || max_score_deviation < 0.0 {
            return Err(HierarchyError::InvalidTolerance);
        }
        let prepared_query = query.factorized_at(query_position)?;
        let prefix = match policy {
            MaskPolicy::Full => self.key_count(),
            MaskPolicy::Causal => query_index.saturating_add(1).min(self.key_count()),
        };

        let mut stack = vec![self.root];
        let mut max_score = f64::NEG_INFINITY;
        let mut weight_sum = 0.0_f64;
        let mut weighted_r = Vec3::zero();
        let mut weighted_c = Vec3::zero();
        let mut visible = 0_usize;
        let mut max_actual_deviation = 0.0_f64;
        let mut eligible_extrema: Option<Extrema> = None;
        let mut work = HierarchyWork::default();

        while let Some(index) = stack.pop() {
            let node = self.nodes[index];
            work.visited_nodes += 1;
            if node.start >= prefix {
                continue;
            }
            let fully_visible = node.end <= prefix;

            let deviation = if fully_visible && !node.leaf() {
                score_deviation(prepared_query, node, score_scale)?
            } else {
                f64::INFINITY
            };
            let accepted = fully_visible && (node.leaf() || deviation <= max_score_deviation);

            if !accepted {
                if let Some((left, right)) = node.children {
                    stack.push(right);
                    stack.push(left);
                } else {
                    return Err(HierarchyError::NonFiniteDerived);
                }
                continue;
            }

            // A collapsed group represents count identical surrogate scores.
            // log(count) preserves total softmax mass, unlike naive centroid
            // attention, which erroneously gives each group mass one.
            let score = prepared_query.pair(node.mean)? * score_scale;
            let log_mass = score + (node.count() as f64).ln();
            if !log_mass.is_finite() {
                return Err(HierarchyError::NonFiniteDerived);
            }
            let next_max = max_score.max(log_mass);
            let old_factor = if visible == 0 {
                0.0
            } else {
                (max_score - next_max).exp()
            };
            let new_factor = (log_mass - next_max).exp();
            weight_sum = weight_sum * old_factor + new_factor;
            if !weight_sum.is_finite() || weight_sum <= 0.0 {
                return Err(HierarchyError::NonFiniteDerived);
            }
            weighted_r =
                checked_linear_update(weighted_r, old_factor, node.mean.resultant, new_factor)?;
            weighted_c =
                checked_linear_update(weighted_c, old_factor, node.mean.origin_moment, new_factor)?;
            max_score = next_max;
            visible += node.count();
            // Only visible accepted blocks may influence a causal error bound.
            eligible_extrema = Some(match eligible_extrema {
                Some(previous) => Extrema::union(previous, node.extrema),
                None => node.extrema,
            });
            work.scored_nodes += 1;
            if node.leaf() {
                work.exact_leaves += 1;
            } else {
                work.collapsed_blocks += 1;
                max_actual_deviation = max_actual_deviation.max(deviation);
            }
        }

        if visible == 0 {
            return Err(HierarchyError::EmptyKeys);
        }
        let inv = 1.0 / weight_sum;
        let r = checked_vec(weighted_r.x * inv, weighted_r.y * inv, weighted_r.z * inv)?;
        let c = checked_vec(weighted_c.x * inv, weighted_c.y * inv, weighted_c.z * inv)?;
        let qxr = query_position.cross(r);
        let moment = checked_vec(c.x - qxr.x, c.y - qxr.y, c.z - qxr.z)?;
        let lse = max_score + weight_sum.ln();
        if !lse.is_finite() {
            return Err(HierarchyError::NonFiniteDerived);
        }

        let tv = max_actual_deviation.tanh();
        // An exact row has zero approximation error even if the range of
        // finite key values would overflow f64 when subtracted. In particular,
        // masked future keys must not invalidate an otherwise finite prefix.
        let (resultant_error, moment_c_error) = if max_actual_deviation == 0.0 {
            (Vec3::zero(), Vec3::zero())
        } else {
            let extrema = eligible_extrema.ok_or(HierarchyError::NonFiniteDerived)?;
            (
                component_bound(extrema.min_r, extrema.max_r, tv)?,
                component_bound(extrema.min_c, extrema.max_c, tv)?,
            )
        };
        let absolute_q = Vec3 {
            x: query_position.x.abs(),
            y: query_position.y.abs(),
            z: query_position.z.abs(),
        };
        let moment_error = checked_vec(
            moment_c_error.x + absolute_q.y * resultant_error.z + absolute_q.z * resultant_error.y,
            moment_c_error.y + absolute_q.z * resultant_error.x + absolute_q.x * resultant_error.z,
            moment_c_error.z + absolute_q.x * resultant_error.y + absolute_q.y * resultant_error.x,
        )?;
        Ok(HierarchicalTorsorRow {
            row: StreamedTorsorRow {
                resultant: r,
                moment_at_query: moment,
                log_sum_exp: lse,
                visible_keys: visible,
            },
            bounds: AlgebraicErrorBounds {
                score_deviation: max_actual_deviation,
                log_sum_exp: max_actual_deviation,
                total_variation: tv,
                resultant: resultant_error,
                moment_at_query: moment_error,
            },
            work,
        })
    }
}

fn component_bound(min: Vec3, max: Vec3, tv: f64) -> Result<Vec3, HierarchyError> {
    checked_vec(
        (max.x - min.x) * tv,
        (max.y - min.y) * tv,
        (max.z - min.z) * tv,
    )
}

/// Max score deviation of any key from the block mean under exact arithmetic.
///
/// Scalar triple-product factorization makes score linear in (R,C).
/// Signed coefficients only affect the center, while extrema differences
/// weighted by their absolute coefficients bound every individual deviation.
fn score_deviation(
    query: FactorizedTwistQuery,
    node: BlockNode,
    scale: f64,
) -> Result<f64, HierarchyError> {
    fn contribution(coeff: Vec3, center: Vec3, min: Vec3, max: Vec3) -> f64 {
        coeff.x.abs() * (center.x - min.x).abs().max((max.x - center.x).abs())
            + coeff.y.abs() * (center.y - min.y).abs().max((max.y - center.y).abs())
            + coeff.z.abs() * (center.z - min.z).abs().max((max.z - center.z).abs())
    }
    let delta = (contribution(
        query.resultant_dual,
        node.mean.resultant,
        node.extrema.min_r,
        node.extrema.max_r,
    ) + contribution(
        query.moment_dual,
        node.mean.origin_moment,
        node.extrema.min_c,
        node.extrema.max_c,
    )) * scale;
    if delta.is_finite() {
        Ok(delta)
    } else {
        Err(HierarchyError::NonFiniteDerived)
    }
}

#[cfg(test)]
mod tests {
    use super::super::tdi22_streaming_attention::{prepare_keys, stream_torsor_row};
    use super::*;

    fn v(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3::new(x, y, z).unwrap()
    }

    fn fixture(n: usize) -> Vec<Torsor3> {
        (0..n)
            .map(|i| {
                let t = i as f64;
                Torsor3::new(
                    v(0.12 * t, (t * 0.17).sin(), (t * 0.33).cos()),
                    v(t * 0.04, (t * 0.11).cos(), -(t * 0.13).sin()),
                    v(t * 0.1, -t * 0.02, 0.5 * t),
                )
                .unwrap()
            })
            .collect()
    }

    fn query() -> (Twist3, Vec3) {
        (
            Twist3::new(v(0.7, -0.3, 0.2), v(-0.4, 0.25, 0.3)).unwrap(),
            v(0.1, -0.2, 0.3),
        )
    }

    fn close(a: f64, b: f64, tolerance: f64) {
        assert!(
            (a - b).abs() <= tolerance,
            "{a} != {b} with tolerance {tolerance}"
        );
    }

    #[test]
    fn zero_tolerance_matches_streaming_reference_full_and_causal() {
        let keys = fixture(19);
        let bank = HierarchicalTorsorBank::build(&keys).unwrap();
        let prepared = prepare_keys(&keys).unwrap();
        let (q, p) = query();
        assert_eq!(bank.node_count(), 37);

        for (policy, index) in [
            (MaskPolicy::Full, 0),
            (MaskPolicy::Causal, 0),
            (MaskPolicy::Causal, 2),
            (MaskPolicy::Causal, 11),
            (MaskPolicy::Causal, 18),
        ] {
            let actual = bank.row(q, p, index, policy, 0.5, 0.0).unwrap();
            let oracle = stream_torsor_row(q, p, &prepared, index, policy, 0.5).unwrap();
            let tol = 1e-11;
            close(actual.row.resultant.x, oracle.resultant.x, tol);
            close(actual.row.resultant.y, oracle.resultant.y, tol);
            close(actual.row.resultant.z, oracle.resultant.z, tol);
            close(actual.row.moment_at_query.x, oracle.moment_at_query.x, tol);
            close(actual.row.moment_at_query.y, oracle.moment_at_query.y, tol);
            close(actual.row.moment_at_query.z, oracle.moment_at_query.z, tol);
            close(actual.row.log_sum_exp, oracle.log_sum_exp, tol);
            assert_eq!(actual.row.visible_keys, oracle.visible_keys);
            assert_eq!(actual.bounds.score_deviation, 0.0);
            assert_eq!(actual.work.exact_leaves, oracle.visible_keys);
        }
    }

    #[test]
    fn homogeneous_keys_collapse_to_one_node_with_correct_multiplicity() {
        let key = Torsor3::new(v(1.0, 2.0, -1.0), v(0.25, 0.1, 0.75), v(1.0, 0.0, 2.0)).unwrap();
        let keys = vec![key; 32];
        let bank = HierarchicalTorsorBank::build(&keys).unwrap();
        let (q, p) = query();
        let result = bank.row(q, p, 0, MaskPolicy::Full, 1.0, 0.0).unwrap();
        let direct = stream_torsor_row(
            q,
            p,
            &prepare_keys(&keys).unwrap(),
            0,
            MaskPolicy::Full,
            1.0,
        )
        .unwrap();
        assert_eq!(result.work.scored_nodes, 1);
        assert_eq!(result.work.collapsed_blocks, 1);
        assert_eq!(result.row.visible_keys, 32);
        close(result.row.resultant.x, direct.resultant.x, 1e-12);
        close(
            result.row.moment_at_query.z,
            direct.moment_at_query.z,
            1e-12,
        );
        close(result.row.log_sum_exp, direct.log_sum_exp, 1e-12);
        assert_eq!(result.bounds.total_variation, 0.0);
    }

    #[test]
    fn causal_queries_never_read_future_block_values() {
        let keys = fixture(16);
        let mut changed = keys.clone();
        let (q, p) = query();
        let a = HierarchicalTorsorBank::build(&keys).unwrap();
        let x = a.row(q, p, 3, MaskPolicy::Causal, 1.0, 100.0).unwrap();
        for future in changed.iter_mut().skip(4) {
            *future = Torsor3::new(v(10_000.0, -1.0, 1.0), v(-300.0, 200.0, 500.0), p).unwrap();
        }
        let b = HierarchicalTorsorBank::build(&changed).unwrap();
        let y = b.row(q, p, 3, MaskPolicy::Causal, 1.0, 100.0).unwrap();
        assert_eq!(x.row.visible_keys, 4);
        assert_eq!(y.row.visible_keys, 4);
        close(x.row.resultant.x, y.row.resultant.x, 1e-12);
        close(x.row.moment_at_query.y, y.row.moment_at_query.y, 1e-12);
        close(x.row.log_sum_exp, y.row.log_sum_exp, 1e-12);
    }

    #[test]
    fn reported_bounds_enclose_deviation_from_dense_reference() {
        let keys = fixture(64);
        let bank = HierarchicalTorsorBank::build(&keys).unwrap();
        let (q, p) = query();
        let direct = stream_torsor_row(
            q,
            p,
            &prepare_keys(&keys).unwrap(),
            0,
            MaskPolicy::Full,
            0.25,
        )
        .unwrap();
        let approximate = bank.row(q, p, 0, MaskPolicy::Full, 0.25, 3.0).unwrap();
        assert!(approximate.work.collapsed_blocks > 0);
        assert!(approximate.work.scored_nodes < keys.len());
        let b = approximate.bounds;
        assert!(b.score_deviation <= 3.0);
        close(
            approximate.row.resultant.x,
            direct.resultant.x,
            b.resultant.x + 1e-10,
        );
        close(
            approximate.row.resultant.y,
            direct.resultant.y,
            b.resultant.y + 1e-10,
        );
        close(
            approximate.row.resultant.z,
            direct.resultant.z,
            b.resultant.z + 1e-10,
        );
        close(
            approximate.row.moment_at_query.x,
            direct.moment_at_query.x,
            b.moment_at_query.x + 1e-10,
        );
        close(
            approximate.row.moment_at_query.y,
            direct.moment_at_query.y,
            b.moment_at_query.y + 1e-10,
        );
        close(
            approximate.row.moment_at_query.z,
            direct.moment_at_query.z,
            b.moment_at_query.z + 1e-10,
        );
        close(
            approximate.row.log_sum_exp,
            direct.log_sum_exp,
            b.log_sum_exp + 1e-10,
        );
    }

    #[test]
    fn adversarial_scores_cannot_collapse_at_zero_tolerance() {
        let k = [
            Torsor3::new(v(1.0, 0.0, 0.0), Vec3::zero(), Vec3::zero()).unwrap(),
            Torsor3::new(v(-1.0, 0.0, 0.0), Vec3::zero(), Vec3::zero()).unwrap(),
        ];
        let bank = HierarchicalTorsorBank::build(&k).unwrap();
        let q = Twist3::new(v(100.0, 0.0, 0.0), Vec3::zero()).unwrap();
        let exact = bank
            .row(q, Vec3::zero(), 0, MaskPolicy::Full, 1.0, 0.0)
            .unwrap();
        assert_eq!(exact.work.exact_leaves, 2);
        assert!(exact.row.resultant.x > 0.99);
        let approximate = bank
            .row(q, Vec3::zero(), 0, MaskPolicy::Full, 1.0, 100.0)
            .unwrap();
        assert_eq!(approximate.work.scored_nodes, 1);
        assert_eq!(approximate.row.resultant.x, 0.0);
        assert!(approximate.bounds.resultant.x >= 1.0);
        assert!(
            approximate.bounds.log_sum_exp
                >= (approximate.row.log_sum_exp - exact.row.log_sum_exp).abs()
        );
    }

    #[test]
    fn masked_extreme_future_values_do_not_break_exact_or_approximate_prefixes() {
        let zero = Vec3::zero();
        let q = Twist3::new(v(1.0, 0.0, 0.0), zero).unwrap();
        let keys = vec![
            Torsor3::new(v(1.0, 0.0, 0.0), zero, zero).unwrap(),
            Torsor3::new(v(2.0, 0.0, 0.0), zero, zero).unwrap(),
            Torsor3::new(v(1.0e308, 0.0, 0.0), zero, zero).unwrap(),
            Torsor3::new(v(-1.0e308, 0.0, 0.0), zero, zero).unwrap(),
        ];
        let bank = HierarchicalTorsorBank::build(&keys).unwrap();
        let prepared = prepare_keys(&keys).unwrap();

        // The full-bank x-range is infinite in binary64. It is irrelevant
        // to the exact one-key causal row and its zero error envelope.
        let exact = bank.row(q, zero, 0, MaskPolicy::Causal, 1.0, 0.0).unwrap();
        let oracle = stream_torsor_row(q, zero, &prepared, 0, MaskPolicy::Causal, 1.0).unwrap();
        assert_eq!(exact.row, oracle);
        assert_eq!(exact.bounds.resultant, zero);
        assert_eq!(exact.bounds.moment_at_query, zero);

        // The first two visible keys may be summarized with nonzero error.
        // Future extreme values still must not enter the error-range bound.
        let grouped = bank.row(q, zero, 1, MaskPolicy::Causal, 1.0, 1.0).unwrap();
        let dense = stream_torsor_row(q, zero, &prepared, 1, MaskPolicy::Causal, 1.0).unwrap();
        assert_eq!(grouped.row.visible_keys, 2);
        assert_eq!(grouped.work.collapsed_blocks, 1);
        assert!(grouped.bounds.resultant.x.is_finite());
        assert!(
            (grouped.row.resultant.x - dense.resultant.x).abs()
                <= grouped.bounds.resultant.x + 1e-12
        );
    }

    #[test]
    fn malformed_configuration_is_rejected() {
        assert_eq!(
            HierarchicalTorsorBank::build(&[]).unwrap_err(),
            HierarchyError::EmptyKeys
        );
        let keys = fixture(2);
        let bank = HierarchicalTorsorBank::build(&keys).unwrap();
        let (q, p) = query();
        for bad in [f64::NAN, f64::INFINITY, -1.0] {
            assert_eq!(
                bank.row(q, p, 0, MaskPolicy::Full, 1.0, bad).unwrap_err(),
                HierarchyError::InvalidTolerance,
            );
        }
        assert_eq!(
            bank.row(q, p, 0, MaskPolicy::Full, 0.0, 1.0).unwrap_err(),
            HierarchyError::InvalidScale,
        );
        let unstable =
            Torsor3::new(v(f64::MAX, 0.0, 0.0), v(0.0, 1.0, 0.0), v(0.0, 0.0, 2.0)).unwrap();
        assert!(HierarchicalTorsorBank::build(&[unstable]).is_err());
    }
}
