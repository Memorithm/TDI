//! Development-only deterministic hierarchy with an exact, typed fallback.
//!
//! The hierarchy may be fast when many keys share similar scores, but a
//! heterogeneous bank may require a near-complete tree traversal. Rather
//! than finishing a costly traversal, the caller can explicitly cap visits;
//! on this one condition only, the row is recomputed with the previously
//! qualified factorized streaming oracle over the same ordered key bank.
//!
//! Fallback is exact in the underlying algebraic attention semantics. It
//! does not certify identical IEEE-754 rounding against direct Varignon,
//! faster wall time, memory traffic savings or model quality. TDI-22.0's
//! frozen stage and FLAT's default routing remain untouched.

use super::tdi22_hierarchical_attention::{
    AlgebraicErrorBounds, HierarchicalRowRequest, HierarchicalTorsorBank, HierarchyError,
    HierarchyWork,
};
use super::tdi22_streaming_attention::{
    PreparedTorsorKey, StreamedTorsorRow, prepare_keys, stream_torsor_row,
};
use super::tdi22_torsor::{Torsor3, Twist3, Vec3};
use super::tdi24_attention::MaskPolicy;

/// Explicit opt-in non-final routing identity.
pub const GUARDED_CONTRACT: &str = "tdi22-visit-limited-exact-fallback-v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuardedPath {
    /// Completed a hierarchy row within the maximum visited-node count.
    Hierarchical,
    /// Traversal cap reached; recomputed the full row from prepared keys.
    ExactStreamingFallback,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GuardedTorsorRow {
    /// Normalized (R,M(Q)) with visible count and LSE.
    pub row: StreamedTorsorRow,
    /// Chosen path for audit and independent performance analysis.
    pub path: GuardedPath,
    /// Hierarchical error envelope, absent for exact streaming fallback.
    pub hierarchy_bounds: Option<AlgebraicErrorBounds>,
    /// Actual hierarchy work only when traversal completed.
    pub hierarchy_work: Option<HierarchyWork>,
    /// Query's explicit hierarchy visit cap, never silently tuned.
    pub visit_budget: usize,
}

#[derive(Clone, Debug)]
pub struct GuardedTorsorBank {
    /// Immutable, ordered validated input to the exact fallback.
    prepared: Vec<PreparedTorsorKey>,
    /// Hierarchy for the very same keys. Neither can be updated alone.
    hierarchy: HierarchicalTorsorBank,
}

impl GuardedTorsorBank {
    /// Construct both paths from one immutable ordered source of torsors.
    /// This currently performs factorization twice and retains both
    /// representations. Cold-build and memory costs must be measured.
    pub fn build(keys: &[Torsor3]) -> Result<Self, HierarchyError> {
        let hierarchy = HierarchicalTorsorBank::build(keys)?;
        let prepared = prepare_keys(keys).map_err(HierarchyError::from)?;
        debug_assert_eq!(prepared.len(), hierarchy.key_count());
        Ok(Self {
            prepared,
            hierarchy,
        })
    }

    pub fn key_count(&self) -> usize {
        self.prepared.len()
    }

    pub fn node_count(&self) -> usize {
        self.hierarchy.node_count()
    }

    /// Run a deterministic bounded hierarchy then, **only if its visit
    /// budget is exhausted**, compute an exact streaming row.
    ///
    /// The same explicit score scale, query, point, causal policy, and
    /// query index apply to both paths. The fallback is not triggered for
    /// invalid tolerance, non-finite arithmetic, or malformed inputs.
    /// max_visited_nodes=0 deliberately forces streaming.
    pub fn row(
        &self,
        request: HierarchicalRowRequest,
        max_visited_nodes: usize,
    ) -> Result<GuardedTorsorRow, HierarchyError> {
        match self
            .hierarchy
            .row_with_visit_budget(request, max_visited_nodes)
        {
            Ok(complete) => Ok(GuardedTorsorRow {
                row: complete.row,
                path: GuardedPath::Hierarchical,
                hierarchy_bounds: Some(complete.bounds),
                hierarchy_work: Some(complete.work),
                visit_budget: max_visited_nodes,
            }),
            Err(HierarchyError::VisitBudgetExceeded) => {
                // No partial hierarchy weights are reused. The second
                // computation has exact normalizer semantics.
                let exact = stream_torsor_row(
                    request.query,
                    request.query_position,
                    &self.prepared,
                    request.query_index,
                    request.policy,
                    request.score_scale,
                )
                .map_err(HierarchyError::from)?;
                Ok(GuardedTorsorRow {
                    row: exact,
                    path: GuardedPath::ExactStreamingFallback,
                    hierarchy_bounds: None,
                    hierarchy_work: None,
                    visit_budget: max_visited_nodes,
                })
            }
            Err(other) => Err(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(x: f64, y: f64, z: f64) -> Vec3 {
        Vec3::new(x, y, z).unwrap()
    }

    fn keys(n: usize) -> Vec<Torsor3> {
        (0..n)
            .map(|i| {
                let x = i as f64;
                Torsor3::new(
                    v((x * 0.13).sin(), x * 0.02, (x * 0.17).cos()),
                    v(-x * 0.03, (x * 0.23).sin(), x * 0.015),
                    v(x * 0.01, -x * 0.02, 0.3),
                )
                .unwrap()
            })
            .collect()
    }

    fn query() -> (Twist3, Vec3) {
        (
            Twist3::new(v(0.7, 0.2, -0.4), v(-0.2, 0.5, 0.3)).unwrap(),
            v(0.1, -0.3, 0.2),
        )
    }

    fn request(
        query: Twist3,
        query_position: Vec3,
        query_index: usize,
        policy: MaskPolicy,
        score_scale: f64,
        max_score_deviation: f64,
    ) -> HierarchicalRowRequest {
        HierarchicalRowRequest {
            query,
            query_position,
            query_index,
            policy,
            score_scale,
            max_score_deviation,
        }
    }

    #[test]
    fn zero_budget_is_exact_streaming_full_and_causal() {
        let input = keys(19);
        let guarded = GuardedTorsorBank::build(&input).unwrap();
        assert_eq!(guarded.key_count(), 19);
        assert_eq!(guarded.node_count(), 37);
        let prepared = prepare_keys(&input).unwrap();
        let (q, point) = query();

        for (policy, position) in [
            (MaskPolicy::Full, 0),
            (MaskPolicy::Causal, 0),
            (MaskPolicy::Causal, 6),
            (MaskPolicy::Causal, 18),
        ] {
            let row = guarded
                .row(request(q, point, position, policy, 0.5, 0.4), 0)
                .unwrap();
            let exact = stream_torsor_row(q, point, &prepared, position, policy, 0.5).unwrap();
            assert_eq!(row.path, GuardedPath::ExactStreamingFallback);
            assert_eq!(row.row, exact);
            assert_eq!(row.hierarchy_work, None);
            assert_eq!(row.hierarchy_bounds, None);
            assert_eq!(row.visit_budget, 0);
        }
    }

    #[test]
    fn enough_budget_preserves_original_hierarchy_bitwise() {
        let input = keys(23);
        let guarded = GuardedTorsorBank::build(&input).unwrap();
        let original = HierarchicalTorsorBank::build(&input).unwrap();
        let (q, point) = query();
        for policy in [MaskPolicy::Full, MaskPolicy::Causal] {
            let expected = original.row(q, point, 12, policy, 0.5, 0.4).unwrap();
            let observed = guarded.row(request(q, point, 12, policy, 0.5, 0.4), 64).unwrap();
            assert_eq!(observed.path, GuardedPath::Hierarchical);
            assert_eq!(observed.row, expected.row);
            assert_eq!(observed.hierarchy_work, Some(expected.work));
            assert_eq!(observed.hierarchy_bounds, Some(expected.bounds));
        }
    }

    #[test]
    fn one_node_can_be_accepted_at_budget_one_when_homogeneous() {
        let key = Torsor3::new(v(1.0, -2.0, 3.0), v(0.3, 0.4, -0.2), v(0.2, 0.1, -0.1)).unwrap();
        let bank = GuardedTorsorBank::build(&vec![key; 32]).unwrap();
        let (q, point) = query();
        let r = bank
            .row(request(q, point, 0, MaskPolicy::Full, 1.0, 0.0), 1)
            .unwrap();
        assert_eq!(r.path, GuardedPath::Hierarchical);
        assert_eq!(r.hierarchy_work.unwrap().scored_nodes, 1);
        assert_eq!(r.row.visible_keys, 32);
        assert_eq!(r.hierarchy_bounds.unwrap().total_variation, 0.0);
    }

    #[test]
    fn alternating_opposite_scores_force_exact_fallback() {
        let keys: Vec<_> = (0..64)
            .map(|i| {
                let sign = if i % 2 == 0 { 1.0 } else { -1.0 };
                Torsor3::new(v(18.0 * sign, 0.0, 0.0), Vec3::zero(), Vec3::zero()).unwrap()
            })
            .collect();
        let bank = GuardedTorsorBank::build(&keys).unwrap();
        let q = Twist3::new(v(1.0, 0.0, 0.0), Vec3::zero()).unwrap();
        let output = bank
            .row(request(q, Vec3::zero(), 0, MaskPolicy::Full, 0.25, 0.5), 8)
            .unwrap();
        assert_eq!(output.path, GuardedPath::ExactStreamingFallback);
        let expected = stream_torsor_row(
            q,
            Vec3::zero(),
            &prepare_keys(&keys).unwrap(),
            0,
            MaskPolicy::Full,
            0.25,
        )
        .unwrap();
        assert_eq!(output.row, expected);
    }

    #[test]
    fn invalid_settings_never_become_successful_fallback() {
        let bank = GuardedTorsorBank::build(&keys(4)).unwrap();
        let (q, point) = query();
        for invalid_scale in [0.0, f64::NAN, f64::INFINITY] {
            assert_eq!(
                bank.row(request(q, point, 0, MaskPolicy::Full, invalid_scale, 0.5), 0)
                    .unwrap_err(),
                HierarchyError::InvalidScale,
            );
        }
        assert_eq!(
            bank.row(request(q, point, 0, MaskPolicy::Full, 0.5, -0.5), 0)
                .unwrap_err(),
            HierarchyError::InvalidTolerance,
        );
        assert_eq!(
            GuardedTorsorBank::build(&[]).unwrap_err(),
            HierarchyError::EmptyKeys,
        );
    }
}
