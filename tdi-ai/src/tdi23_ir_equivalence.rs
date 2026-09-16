//! Boundary-aware exact equivalence and reduction-contract summaries for TDI-23.1.
//!
//! This module is deliberately narrower than a rewrite engine. It compares two
//! already-built rooted fragments only when they share the same exact IR
//! endpoints and both lower through the Stage-0 real linear carrier. Explicit
//! nonlinear/algebraic boundaries fail closed. No tolerance, approximate
//! equivalence, rewrite enumeration, or performance claim is introduced here.

use core::fmt;
use std::collections::{BTreeMap, BTreeSet};

use super::tdi23_ir::{
    CategoricalAttentionIr, HilbertObjectKind, IrError, IrNodeKind, NodeId,
    NonlinearBoundaryKind, ObjectId,
};
use super::tdi23_ir_provenance::{ProvenanceError, validate_rooted_subgraph};

/// Versioned exact boundary-aware equivalence contract.
pub const EXACT_EQUIVALENCE_CONTRACT: &str =
    "tdi23.1-boundary-aware-exact-equivalence-v1";
/// Versioned explicit reduction-summary contract.
pub const REDUCTION_CONTRACT_SUMMARY: &str = "tdi23.1-reduction-contract-summary-v1";

/// Side of an exact comparison that failed to lower through the linear carrier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComparisonSide {
    /// Left comparison root.
    Left,
    /// Right comparison root.
    Right,
}

impl fmt::Display for ComparisonSide {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Left => "left",
            Self::Right => "right",
        })
    }
}

/// Fail-closed errors for exact rooted-fragment comparison and reduction summaries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EquivalenceError {
    /// Independent rooted structural validation failed.
    Provenance(ProvenanceError),
    /// Underlying IR lookup or bounded linear evaluation failed.
    Ir(IrError),
    /// Equality of morphisms requires the same exact source and target objects.
    EndpointMismatch {
        /// Left domain object index.
        left_domain: usize,
        /// Right domain object index.
        right_domain: usize,
        /// Left codomain object index.
        left_codomain: usize,
        /// Right codomain object index.
        right_codomain: usize,
    },
    /// Exact linear comparison cannot cross an explicit nonlinear/algebraic boundary.
    NonlinearBoundary {
        /// Side that contains the boundary.
        side: ComparisonSide,
        /// First boundary reported by the Stage-0 lowering path.
        boundary: NonlinearBoundaryKind,
    },
}

impl fmt::Display for EquivalenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Provenance(error) => write!(formatter, "rooted validation failed: {error}"),
            Self::Ir(error) => write!(formatter, "IR equivalence error: {error}"),
            Self::EndpointMismatch {
                left_domain,
                right_domain,
                left_codomain,
                right_codomain,
            } => write!(
                formatter,
                "exact endpoint mismatch: domain {left_domain}!={right_domain}, codomain {left_codomain}!={right_codomain}"
            ),
            Self::NonlinearBoundary { side, boundary } => write!(
                formatter,
                "{side} rooted fragment crosses explicit {boundary} boundary"
            ),
        }
    }
}

impl std::error::Error for EquivalenceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Provenance(error) => Some(error),
            Self::Ir(error) => Some(error),
            _ => None,
        }
    }
}

impl From<ProvenanceError> for EquivalenceError {
    fn from(value: ProvenanceError) -> Self {
        Self::Provenance(value)
    }
}

impl From<IrError> for EquivalenceError {
    fn from(value: IrError) -> Self {
        Self::Ir(value)
    }
}

/// Exact Stage-0 comparison result for two boundary-free roots with identical endpoints.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExactLinearEquivalenceReport {
    left: NodeId,
    right: NodeId,
    entry_count: usize,
    values_equal: bool,
    bitwise_identical: bool,
    first_value_mismatch: Option<usize>,
    first_bit_mismatch: Option<usize>,
}

impl ExactLinearEquivalenceReport {
    /// Left validated root.
    #[must_use]
    pub const fn left(&self) -> NodeId {
        self.left
    }

    /// Right validated root.
    #[must_use]
    pub const fn right(&self) -> NodeId {
        self.right
    }

    /// Number of scalar entries compared.
    #[must_use]
    pub const fn entry_count(&self) -> usize {
        self.entry_count
    }

    /// Whether all finite scalar values compare equal under ordinary `f64` equality.
    ///
    /// In particular, `+0.0 == -0.0` is true here.
    #[must_use]
    pub const fn values_equal(&self) -> bool {
        self.values_equal
    }

    /// Whether every scalar has the exact same IEEE-754 bit pattern.
    #[must_use]
    pub const fn bitwise_identical(&self) -> bool {
        self.bitwise_identical
    }

    /// First scalar position whose ordinary `f64` values differ.
    #[must_use]
    pub const fn first_value_mismatch(&self) -> Option<usize> {
        self.first_value_mismatch
    }

    /// First scalar position whose IEEE-754 bit patterns differ.
    #[must_use]
    pub const fn first_bit_mismatch(&self) -> Option<usize> {
        self.first_bit_mismatch
    }
}

/// Compare two rooted linear fragments after independent structural validation.
///
/// Both roots must belong to the same IR and use the same exact domain and
/// codomain objects. Any explicit softmax/Boolean/F2/ANF/max-plus boundary is
/// rejected by the Stage-0 linear lowering path. The returned report separates
/// ordinary finite-value equality from stronger IEEE-754 bit identity.
pub fn compare_exact_linear_semantics(
    ir: &CategoricalAttentionIr,
    left: NodeId,
    right: NodeId,
) -> Result<ExactLinearEquivalenceReport, EquivalenceError> {
    validate_rooted_subgraph(ir, left)?;
    validate_rooted_subgraph(ir, right)?;

    let left_node = ir.node(left)?;
    let right_node = ir.node(right)?;
    if left_node.domain() != right_node.domain() || left_node.codomain() != right_node.codomain() {
        return Err(EquivalenceError::EndpointMismatch {
            left_domain: left_node.domain().index(),
            right_domain: right_node.domain().index(),
            left_codomain: left_node.codomain().index(),
            right_codomain: right_node.codomain().index(),
        });
    }

    let left_map = evaluate_side(ir, left, ComparisonSide::Left)?;
    let right_map = evaluate_side(ir, right, ComparisonSide::Right)?;

    let first_value_mismatch = left_map
        .entries()
        .iter()
        .zip(right_map.entries())
        .position(|(left_value, right_value)| left_value != right_value);
    let first_bit_mismatch = left_map
        .entries()
        .iter()
        .zip(right_map.entries())
        .position(|(left_value, right_value)| left_value.to_bits() != right_value.to_bits());

    Ok(ExactLinearEquivalenceReport {
        left,
        right,
        entry_count: left_map.entries().len(),
        values_equal: first_value_mismatch.is_none(),
        bitwise_identical: first_bit_mismatch.is_none(),
        first_value_mismatch,
        first_bit_mismatch,
    })
}

fn evaluate_side(
    ir: &CategoricalAttentionIr,
    root: NodeId,
    side: ComparisonSide,
) -> Result<super::tdi23_categorical::RealLinearMap, EquivalenceError> {
    match ir.evaluate_linear(root) {
        Ok(map) => Ok(map),
        Err(IrError::LinearEvaluationCrossesBoundary(boundary)) => {
            Err(EquivalenceError::NonlinearBoundary { side, boundary })
        }
        Err(error) => Err(EquivalenceError::Ir(error)),
    }
}

/// Reduction annotation attached to one reachable object.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReductionObjectContract {
    object_index: usize,
    name: String,
    ambient_dimension: usize,
    kept_coordinates: Option<Vec<usize>>,
}

impl ReductionObjectContract {
    /// IR-local object index used only as an in-process diagnostic.
    #[must_use]
    pub const fn object_index(&self) -> usize {
        self.object_index
    }

    /// Declared object name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Declared ambient object dimension.
    #[must_use]
    pub const fn ambient_dimension(&self) -> usize {
        self.ambient_dimension
    }

    /// Ordered retained coordinates, or `None` when no reduction is annotated.
    #[must_use]
    pub fn kept_coordinates(&self) -> Option<&[usize]> {
        self.kept_coordinates.as_deref()
    }

    /// Reduced dimension when an explicit annotation is present.
    #[must_use]
    pub fn reduced_dimension(&self) -> Option<usize> {
        self.kept_coordinates.as_ref().map(Vec::len)
    }
}

/// Explicit reduction contract over all objects reachable from one root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReductionContractSummary {
    root: NodeId,
    objects: Vec<ReductionObjectContract>,
    annotated_object_count: usize,
    complete: bool,
    root_domain_reduced_dimension: Option<usize>,
    root_codomain_reduced_dimension: Option<usize>,
}

impl ReductionContractSummary {
    /// Root whose reachable reduction contract was summarized.
    #[must_use]
    pub const fn root(&self) -> NodeId {
        self.root
    }

    /// Reachable objects in ascending IR-local construction index.
    #[must_use]
    pub fn objects(&self) -> &[ReductionObjectContract] {
        &self.objects
    }

    /// Number of reachable objects carrying an explicit coordinate reduction.
    #[must_use]
    pub const fn annotated_object_count(&self) -> usize {
        self.annotated_object_count
    }

    /// Whether every reachable object has an explicit reduction annotation.
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        self.complete
    }

    /// Reduced dimension of the root domain when explicitly annotated.
    #[must_use]
    pub const fn root_domain_reduced_dimension(&self) -> Option<usize> {
        self.root_domain_reduced_dimension
    }

    /// Reduced dimension of the root codomain when explicitly annotated.
    #[must_use]
    pub const fn root_codomain_reduced_dimension(&self) -> Option<usize> {
        self.root_codomain_reduced_dimension
    }
}

/// Summarize explicit coordinate reductions over one independently validated root.
///
/// Missing annotations are reported as missing rather than filled implicitly.
/// Completeness is a bookkeeping property only; it does not claim that the
/// reduction is lossless or functorial.
pub fn summarize_reduction_contract(
    ir: &CategoricalAttentionIr,
    root: NodeId,
) -> Result<ReductionContractSummary, EquivalenceError> {
    validate_rooted_subgraph(ir, root)?;

    let mut nodes = BTreeSet::new();
    let mut objects = BTreeMap::new();
    collect_reachable_objects(ir, root, &mut nodes, &mut objects)?;

    let root_node = ir.node(root)?;
    let root_domain_reduced_dimension = ir
        .coordinate_reduction(root_node.domain())?
        .map(|reduction| reduction.reduced_dim());
    let root_codomain_reduced_dimension = ir
        .coordinate_reduction(root_node.codomain())?
        .map(|reduction| reduction.reduced_dim());

    let mut contracts = Vec::with_capacity(objects.len());
    let mut annotated_object_count = 0usize;
    for (index, object_id) in objects {
        let object = ir.object(object_id)?;
        let kept_coordinates = ir.coordinate_reduction(object_id)?.map(|reduction| {
            annotated_object_count += 1;
            reduction.kept_coordinates().to_vec()
        });
        contracts.push(ReductionObjectContract {
            object_index: index,
            name: object.name().to_owned(),
            ambient_dimension: object.dimension(),
            kept_coordinates,
        });
    }

    Ok(ReductionContractSummary {
        root,
        complete: annotated_object_count == contracts.len(),
        objects: contracts,
        annotated_object_count,
        root_domain_reduced_dimension,
        root_codomain_reduced_dimension,
    })
}

fn collect_reachable_objects(
    ir: &CategoricalAttentionIr,
    node_id: NodeId,
    nodes: &mut BTreeSet<usize>,
    objects: &mut BTreeMap<usize, ObjectId>,
) -> Result<(), IrError> {
    if !nodes.insert(node_id.index()) {
        return Ok(());
    }
    let node = ir.node(node_id)?;
    collect_object_dependencies(ir, node.domain(), objects)?;
    collect_object_dependencies(ir, node.codomain(), objects)?;
    match node.kind() {
        IrNodeKind::Compose { outer, inner } => {
            collect_reachable_objects(ir, *inner, nodes, objects)?;
            collect_reachable_objects(ir, *outer, nodes, objects)?;
        }
        IrNodeKind::Dagger { source } => {
            collect_reachable_objects(ir, *source, nodes, objects)?;
        }
        IrNodeKind::LinearMap(_) | IrNodeKind::Identity | IrNodeKind::NonlinearBoundary { .. } => {}
    }
    Ok(())
}

fn collect_object_dependencies(
    ir: &CategoricalAttentionIr,
    object_id: ObjectId,
    objects: &mut BTreeMap<usize, ObjectId>,
) -> Result<(), IrError> {
    if objects.contains_key(&object_id.index()) {
        return Ok(());
    }
    objects.insert(object_id.index(), object_id);
    let object = ir.object(object_id)?;
    match object.kind() {
        HilbertObjectKind::Atomic => {}
        HilbertObjectKind::DirectSum(parts) | HilbertObjectKind::TensorProduct(parts) => {
            for part in parts {
                collect_object_dependencies(ir, *part, objects)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        ComparisonSide, EXACT_EQUIVALENCE_CONTRACT, EquivalenceError, REDUCTION_CONTRACT_SUMMARY,
        compare_exact_linear_semantics, summarize_reduction_contract,
    };
    use crate::experimental::tdi23_categorical::RealLinearMap;
    use crate::experimental::tdi23_ir::{CategoricalAttentionIr, NonlinearBoundaryKind};
    use crate::experimental::tdi23_reduction::CoordinateReduction;

    #[test]
    fn tdi23_1_equivalence_contracts_are_versioned() {
        assert_eq!(
            EXACT_EQUIVALENCE_CONTRACT,
            "tdi23.1-boundary-aware-exact-equivalence-v1"
        );
        assert_eq!(
            REDUCTION_CONTRACT_SUMMARY,
            "tdi23.1-reduction-contract-summary-v1"
        );
    }

    #[test]
    fn tdi23_1_equivalence_identity_compositions_are_bitwise_identical() {
        let mut ir = CategoricalAttentionIr::new();
        let a = ir.add_atomic_object("A", 2).expect("A");
        let b = ir.add_atomic_object("B", 2).expect("B");
        let map = RealLinearMap::new(2, 2, vec![1.0, 2.0, 3.0, 4.0]).expect("map");
        let f = ir.add_linear_map(a, b, map).expect("f");
        let id_a = ir.add_identity(a).expect("id A");
        let id_b = ir.add_identity(b).expect("id B");
        let right_identity = ir.compose(f, id_a).expect("f o id_A");
        let left_identity = ir.compose(id_b, f).expect("id_B o f");

        for candidate in [right_identity, left_identity] {
            let report = compare_exact_linear_semantics(&ir, candidate, f).expect("comparison");
            assert!(report.values_equal());
            assert!(report.bitwise_identical());
            assert_eq!(report.first_value_mismatch(), None);
            assert_eq!(report.first_bit_mismatch(), None);
        }
    }

    #[test]
    fn tdi23_1_equivalence_double_dagger_is_bitwise_identical() {
        let mut ir = CategoricalAttentionIr::new();
        let a = ir.add_atomic_object("A", 2).expect("A");
        let b = ir.add_atomic_object("B", 3).expect("B");
        let f = ir
            .add_linear_map(
                a,
                b,
                RealLinearMap::new(2, 3, vec![1.0, -2.0, 3.0, 4.0, 5.0, -6.0])
                    .expect("map"),
            )
            .expect("f");
        let dagger = ir.dagger(f).expect("dagger");
        let double_dagger = ir.dagger(dagger).expect("double dagger");
        let report = compare_exact_linear_semantics(&ir, double_dagger, f).expect("comparison");
        assert!(report.values_equal());
        assert!(report.bitwise_identical());
    }

    #[test]
    fn tdi23_1_equivalence_requires_exact_endpoint_identity() {
        let mut ir = CategoricalAttentionIr::new();
        let left_object = ir.add_atomic_object("Left", 2).expect("Left");
        let right_object = ir.add_atomic_object("Right", 2).expect("Right");
        let left = ir.add_identity(left_object).expect("left identity");
        let right = ir.add_identity(right_object).expect("right identity");
        assert_eq!(
            compare_exact_linear_semantics(&ir, left, right),
            Err(EquivalenceError::EndpointMismatch {
                left_domain: left_object.index(),
                right_domain: right_object.index(),
                left_codomain: left_object.index(),
                right_codomain: right_object.index(),
            })
        );
    }

    #[test]
    fn tdi23_1_equivalence_fails_closed_at_nonlinear_boundaries() {
        let mut ir = CategoricalAttentionIr::new();
        let h = ir.add_atomic_object("H", 2).expect("H");
        let softmax = ir
            .add_nonlinear_boundary(h, h, NonlinearBoundaryKind::Softmax)
            .expect("softmax");
        let identity = ir.add_identity(h).expect("identity");
        assert_eq!(
            compare_exact_linear_semantics(&ir, softmax, identity),
            Err(EquivalenceError::NonlinearBoundary {
                side: ComparisonSide::Left,
                boundary: NonlinearBoundaryKind::Softmax,
            })
        );
    }

    #[test]
    fn tdi23_1_equivalence_separates_value_and_bit_identity() {
        let mut ir = CategoricalAttentionIr::new();
        let h = ir.add_atomic_object("H", 1).expect("H");
        let positive = ir
            .add_linear_map(
                h,
                h,
                RealLinearMap::new(1, 1, vec![0.0]).expect("positive zero"),
            )
            .expect("positive");
        let negative = ir
            .add_linear_map(
                h,
                h,
                RealLinearMap::new(1, 1, vec![-0.0]).expect("negative zero"),
            )
            .expect("negative");
        let report = compare_exact_linear_semantics(&ir, positive, negative).expect("comparison");
        assert!(report.values_equal());
        assert!(!report.bitwise_identical());
        assert_eq!(report.first_value_mismatch(), None);
        assert_eq!(report.first_bit_mismatch(), Some(0));
    }

    #[test]
    fn tdi23_1_equivalence_reduction_summary_is_explicit_and_complete_only_when_annotated() {
        let mut ir = CategoricalAttentionIr::new();
        let a = ir.add_atomic_object("A", 1).expect("A");
        let b = ir.add_atomic_object("B", 2).expect("B");
        let c = ir.add_atomic_object("C", 1).expect("C");
        let first = ir
            .add_linear_map(
                a,
                b,
                RealLinearMap::new(1, 2, vec![1.0, 2.0]).expect("first map"),
            )
            .expect("first");
        let second = ir
            .add_linear_map(
                b,
                c,
                RealLinearMap::new(2, 1, vec![3.0, 4.0]).expect("second map"),
            )
            .expect("second");
        let root = ir.compose(second, first).expect("root");

        ir.annotate_coordinate_reduction(
            a,
            CoordinateReduction::new(1, vec![0]).expect("A reduction"),
        )
        .expect("annotate A");
        ir.annotate_coordinate_reduction(
            c,
            CoordinateReduction::new(1, vec![0]).expect("C reduction"),
        )
        .expect("annotate C");

        let incomplete = summarize_reduction_contract(&ir, root).expect("summary");
        assert!(!incomplete.is_complete());
        assert_eq!(incomplete.annotated_object_count(), 2);
        assert_eq!(incomplete.objects().len(), 3);
        assert_eq!(incomplete.root_domain_reduced_dimension(), Some(1));
        assert_eq!(incomplete.root_codomain_reduced_dimension(), Some(1));
        let middle = incomplete
            .objects()
            .iter()
            .find(|contract| contract.name() == "B")
            .expect("middle contract");
        assert_eq!(middle.kept_coordinates(), None);

        ir.annotate_coordinate_reduction(
            b,
            CoordinateReduction::new(2, vec![1]).expect("B reduction"),
        )
        .expect("annotate B");
        let complete = summarize_reduction_contract(&ir, root).expect("complete summary");
        assert!(complete.is_complete());
        assert_eq!(complete.annotated_object_count(), 3);
        let middle = complete
            .objects()
            .iter()
            .find(|contract| contract.name() == "B")
            .expect("middle contract");
        assert_eq!(middle.kept_coordinates(), Some(&[1][..]));
        assert_eq!(middle.reduced_dimension(), Some(1));
    }
}
