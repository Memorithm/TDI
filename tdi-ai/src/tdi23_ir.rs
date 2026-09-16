//! TDI-23.1 typed Categorical Attention IR development scaffold.
//!
//! The graph is deliberately bounded: finite-dimensional real Hilbert objects,
//! Stage-0 real linear maps, identity/composition/dagger, explicit opaque
//! nonlinear boundaries, and explicit coordinate-reduction annotations.
//!
//! This is a legality/equivalence scaffold. It is not a rewrite search engine,
//! not complete softmax attention, and not evidence of quality, compression,
//! asymptotic, or hardware-performance improvement.

use core::fmt;
use core::sync::atomic::{AtomicU64, Ordering};

use super::tdi23_categorical::{DaggerError, RealLinearMap};
use super::tdi23_reduction::{
    CoordinateReduction, ReductionError, reduce_linear_map, reduction_composition_defect_max_abs,
};

/// Versioned non-final TDI-23.1 IR contract.
pub const CATEGORICAL_IR_CONTRACT: &str = "tdi23.1-categorical-attention-ir-v1";

static NEXT_IR_OWNER: AtomicU64 = AtomicU64::new(1);

fn allocate_ir_owner() -> u64 {
    NEXT_IR_OWNER
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |owner| {
            owner.checked_add(1)
        })
        .expect("TDI-23.1 IR owner-id space exhausted")
}

/// Stable object handle bound to one IR instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ObjectId {
    owner: u64,
    index: usize,
}

impl ObjectId {
    /// Zero-based object index for deterministic local provenance.
    #[must_use]
    pub const fn index(self) -> usize {
        self.index
    }
}

/// Stable node handle bound to one IR instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeId {
    owner: u64,
    index: usize,
}

impl NodeId {
    /// Zero-based node index for deterministic local provenance.
    #[must_use]
    pub const fn index(self) -> usize {
        self.index
    }
}

/// Structural construction of one finite-dimensional real Hilbert object.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HilbertObjectKind {
    /// Atomic declared space.
    Atomic,
    /// Direct sum / concatenation with additive dimension.
    DirectSum(Vec<ObjectId>),
    /// Tensor product with multiplicative dimension.
    TensorProduct(Vec<ObjectId>),
}

/// Named finite-dimensional real Hilbert object.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HilbertObject {
    name: String,
    dimension: usize,
    kind: HilbertObjectKind,
}

impl HilbertObject {
    /// Human-readable name, unique within one IR instance.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Declared dimension.
    #[must_use]
    pub const fn dimension(&self) -> usize {
        self.dimension
    }

    /// Object construction kind.
    #[must_use]
    pub fn kind(&self) -> &HilbertObjectKind {
        &self.kind
    }
}

/// Opaque operation/domain boundary that is deliberately not a `FdHilb` morphism.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NonlinearBoundaryKind {
    /// Softmax or equivalent normalization.
    Softmax,
    /// Boolean logic.
    Boolean,
    /// Finite-field `F2` operations.
    F2,
    /// Algebraic-normal-form / Zhegalkin operations.
    Anf,
    /// Max-plus / tropical operations.
    MaxPlus,
}

impl fmt::Display for NonlinearBoundaryKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Softmax => "softmax",
            Self::Boolean => "boolean",
            Self::F2 => "F2",
            Self::Anf => "ANF/Zhegalkin",
            Self::MaxPlus => "max-plus",
        })
    }
}

/// Operation represented by one typed IR node.
#[derive(Clone, Debug, PartialEq)]
pub enum IrNodeKind {
    /// Concrete Stage-0 real linear map.
    LinearMap(RealLinearMap),
    /// Identity on the node's domain/codomain object.
    Identity,
    /// Typed composition `outer o inner`.
    Compose {
        /// Outer/left operand.
        outer: NodeId,
        /// Inner/right operand.
        inner: NodeId,
    },
    /// Dagger of a boundary-free linear subgraph.
    Dagger {
        /// Source node.
        source: NodeId,
    },
    /// Explicit non-linear / non-`FdHilb` boundary.
    NonlinearBoundary {
        /// Boundary kind.
        boundary: NonlinearBoundaryKind,
    },
}

/// One IR node with exact source and target objects.
#[derive(Clone, Debug, PartialEq)]
pub struct IrNode {
    domain: ObjectId,
    codomain: ObjectId,
    kind: IrNodeKind,
}

impl IrNode {
    /// Exact domain object.
    #[must_use]
    pub const fn domain(&self) -> ObjectId {
        self.domain
    }

    /// Exact codomain object.
    #[must_use]
    pub const fn codomain(&self) -> ObjectId {
        self.codomain
    }

    /// Node operation.
    #[must_use]
    pub fn kind(&self) -> &IrNodeKind {
        &self.kind
    }
}

/// Fail-closed IR construction/evaluation errors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IrError {
    /// Object names are unique within one IR instance.
    DuplicateObjectName(String),
    /// Composite objects need at least one component.
    EmptyObjectConstruction(&'static str),
    /// Object dimensions must be positive.
    ZeroDimension,
    /// Additive or multiplicative dimension overflow.
    DimensionOverflow,
    /// Object handle came from another IR instance.
    ForeignObjectHandle(ObjectId),
    /// Node handle came from another IR instance.
    ForeignNodeHandle(NodeId),
    /// Object handle is invalid for this IR.
    UnknownObject(ObjectId),
    /// Node handle is invalid for this IR.
    UnknownNode(NodeId),
    /// Concrete map domain dimension differs from the declared object.
    MapDomainDimensionMismatch {
        /// Object dimension.
        object_dimension: usize,
        /// Map dimension.
        map_dimension: usize,
    },
    /// Concrete map codomain dimension differs from the declared object.
    MapCodomainDimensionMismatch {
        /// Object dimension.
        object_dimension: usize,
        /// Map dimension.
        map_dimension: usize,
    },
    /// Composition requires exact middle-object identity.
    CompositionObjectMismatch {
        /// Domain expected by the outer node.
        outer_domain: ObjectId,
        /// Codomain produced by the inner node.
        inner_codomain: ObjectId,
    },
    /// Dagger cannot cross an opaque nonlinear boundary.
    DaggerCrossesNonlinearBoundary(NonlinearBoundaryKind),
    /// Linear evaluation cannot reinterpret an opaque nonlinear boundary.
    LinearEvaluationCrossesBoundary(NonlinearBoundaryKind),
    /// Reduction ambient dimension differs from the annotated object.
    ReductionAmbientDimensionMismatch {
        /// Object dimension.
        object_dimension: usize,
        /// Reduction ambient dimension.
        reduction_dimension: usize,
    },
    /// Reduced lowering requires an explicit reduction annotation.
    MissingReductionAnnotation(ObjectId),
    /// Composition-defect auditing requires a composition node.
    NotCompositionNode(NodeId),
    /// Stage-0 linear carrier rejected an operation.
    Linear(DaggerError),
    /// Stage-0 reduction carrier rejected an operation.
    Reduction(ReductionError),
}

impl fmt::Display for IrError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateObjectName(name) => write!(formatter, "duplicate object name {name:?}"),
            Self::EmptyObjectConstruction(kind) => {
                write!(formatter, "{kind} requires at least one object")
            }
            Self::ZeroDimension => formatter.write_str("object dimension must be positive"),
            Self::DimensionOverflow => formatter.write_str("object dimension overflowed usize"),
            Self::ForeignObjectHandle(id) => {
                write!(
                    formatter,
                    "object handle {} belongs to another IR",
                    id.index()
                )
            }
            Self::ForeignNodeHandle(id) => {
                write!(
                    formatter,
                    "node handle {} belongs to another IR",
                    id.index()
                )
            }
            Self::UnknownObject(id) => write!(formatter, "unknown object id {}", id.index()),
            Self::UnknownNode(id) => write!(formatter, "unknown node id {}", id.index()),
            Self::MapDomainDimensionMismatch {
                object_dimension,
                map_dimension,
            } => write!(
                formatter,
                "map domain mismatch: object={object_dimension}, map={map_dimension}"
            ),
            Self::MapCodomainDimensionMismatch {
                object_dimension,
                map_dimension,
            } => write!(
                formatter,
                "map codomain mismatch: object={object_dimension}, map={map_dimension}"
            ),
            Self::CompositionObjectMismatch {
                outer_domain,
                inner_codomain,
            } => write!(
                formatter,
                "composition object mismatch: outer domain={}, inner codomain={}",
                outer_domain.index(),
                inner_codomain.index()
            ),
            Self::DaggerCrossesNonlinearBoundary(boundary) => {
                write!(formatter, "dagger cannot cross {boundary} boundary")
            }
            Self::LinearEvaluationCrossesBoundary(boundary) => {
                write!(
                    formatter,
                    "linear evaluation cannot cross {boundary} boundary"
                )
            }
            Self::ReductionAmbientDimensionMismatch {
                object_dimension,
                reduction_dimension,
            } => write!(
                formatter,
                "reduction ambient mismatch: object={object_dimension}, reduction={reduction_dimension}"
            ),
            Self::MissingReductionAnnotation(id) => {
                write!(
                    formatter,
                    "object {} has no reduction annotation",
                    id.index()
                )
            }
            Self::NotCompositionNode(id) => {
                write!(formatter, "node {} is not a composition", id.index())
            }
            Self::Linear(error) => write!(formatter, "linear-map error: {error}"),
            Self::Reduction(error) => write!(formatter, "reduction error: {error}"),
        }
    }
}

impl std::error::Error for IrError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Linear(error) => Some(error),
            Self::Reduction(error) => Some(error),
            _ => None,
        }
    }
}

impl From<DaggerError> for IrError {
    fn from(value: DaggerError) -> Self {
        Self::Linear(value)
    }
}

impl From<ReductionError> for IrError {
    fn from(value: ReductionError) -> Self {
        Self::Reduction(value)
    }
}

/// Bounded typed DAG for TDI-23.1 categorical-attention research.
///
/// Handles are instance-bound. The graph intentionally does not implement
/// `Clone`: duplicating an instance while retaining the same owner token would
/// permit divergent graphs to reinterpret one another's handles.
#[derive(Debug)]
pub struct CategoricalAttentionIr {
    owner: u64,
    objects: Vec<HilbertObject>,
    reductions: Vec<Option<CoordinateReduction>>,
    nodes: Vec<IrNode>,
}

impl Default for CategoricalAttentionIr {
    fn default() -> Self {
        Self::new()
    }
}

impl CategoricalAttentionIr {
    /// Create an empty graph with a process-local ownership token.
    #[must_use]
    pub fn new() -> Self {
        Self {
            owner: allocate_ir_owner(),
            objects: Vec::new(),
            reductions: Vec::new(),
            nodes: Vec::new(),
        }
    }

    /// Add one atomic finite-dimensional real Hilbert object.
    pub fn add_atomic_object(
        &mut self,
        name: impl Into<String>,
        dimension: usize,
    ) -> Result<ObjectId, IrError> {
        if dimension == 0 {
            return Err(IrError::ZeroDimension);
        }
        self.push_object(name.into(), dimension, HilbertObjectKind::Atomic)
    }

    /// Add a direct sum with additive dimension.
    pub fn add_direct_sum_object(
        &mut self,
        name: impl Into<String>,
        summands: &[ObjectId],
    ) -> Result<ObjectId, IrError> {
        if summands.is_empty() {
            return Err(IrError::EmptyObjectConstruction("direct sum"));
        }
        let mut dimension = 0usize;
        for id in summands {
            dimension = dimension
                .checked_add(self.object(*id)?.dimension())
                .ok_or(IrError::DimensionOverflow)?;
        }
        self.push_object(
            name.into(),
            dimension,
            HilbertObjectKind::DirectSum(summands.to_vec()),
        )
    }

    /// Add a tensor product with multiplicative dimension.
    pub fn add_tensor_product_object(
        &mut self,
        name: impl Into<String>,
        factors: &[ObjectId],
    ) -> Result<ObjectId, IrError> {
        if factors.is_empty() {
            return Err(IrError::EmptyObjectConstruction("tensor product"));
        }
        let mut dimension = 1usize;
        for id in factors {
            dimension = dimension
                .checked_mul(self.object(*id)?.dimension())
                .ok_or(IrError::DimensionOverflow)?;
        }
        self.push_object(
            name.into(),
            dimension,
            HilbertObjectKind::TensorProduct(factors.to_vec()),
        )
    }

    /// Add a concrete Stage-0 linear map between exact objects.
    pub fn add_linear_map(
        &mut self,
        domain: ObjectId,
        codomain: ObjectId,
        map: RealLinearMap,
    ) -> Result<NodeId, IrError> {
        let domain_dimension = self.object(domain)?.dimension();
        let codomain_dimension = self.object(codomain)?.dimension();
        if map.domain_dim() != domain_dimension {
            return Err(IrError::MapDomainDimensionMismatch {
                object_dimension: domain_dimension,
                map_dimension: map.domain_dim(),
            });
        }
        if map.codomain_dim() != codomain_dimension {
            return Err(IrError::MapCodomainDimensionMismatch {
                object_dimension: codomain_dimension,
                map_dimension: map.codomain_dim(),
            });
        }
        Ok(self.push_node(domain, codomain, IrNodeKind::LinearMap(map)))
    }

    /// Add an identity node.
    pub fn add_identity(&mut self, object: ObjectId) -> Result<NodeId, IrError> {
        self.object(object)?;
        Ok(self.push_node(object, object, IrNodeKind::Identity))
    }

    /// Add an explicit opaque non-`FdHilb` boundary.
    pub fn add_nonlinear_boundary(
        &mut self,
        domain: ObjectId,
        codomain: ObjectId,
        boundary: NonlinearBoundaryKind,
    ) -> Result<NodeId, IrError> {
        self.object(domain)?;
        self.object(codomain)?;
        Ok(self.push_node(domain, codomain, IrNodeKind::NonlinearBoundary { boundary }))
    }

    /// Compose `outer o inner`; exact middle-object identity is required.
    pub fn compose(&mut self, outer: NodeId, inner: NodeId) -> Result<NodeId, IrError> {
        let outer_node = self.node(outer)?;
        let inner_node = self.node(inner)?;
        let outer_domain = outer_node.domain;
        let inner_codomain = inner_node.codomain;
        if outer_domain != inner_codomain {
            return Err(IrError::CompositionObjectMismatch {
                outer_domain,
                inner_codomain,
            });
        }
        let domain = inner_node.domain;
        let codomain = outer_node.codomain;
        Ok(self.push_node(domain, codomain, IrNodeKind::Compose { outer, inner }))
    }

    /// Add the dagger of a boundary-free linear subgraph.
    pub fn dagger(&mut self, source: NodeId) -> Result<NodeId, IrError> {
        let source_node = self.node(source)?;
        let domain = source_node.codomain;
        let codomain = source_node.domain;
        if let Some(boundary) = self.first_nonlinear_boundary(source)? {
            return Err(IrError::DaggerCrossesNonlinearBoundary(boundary));
        }
        Ok(self.push_node(domain, codomain, IrNodeKind::Dagger { source }))
    }

    /// Attach or replace an explicit coordinate-reduction annotation.
    pub fn annotate_coordinate_reduction(
        &mut self,
        object: ObjectId,
        reduction: CoordinateReduction,
    ) -> Result<(), IrError> {
        let object_dimension = self.object(object)?.dimension();
        if reduction.ambient_dim() != object_dimension {
            return Err(IrError::ReductionAmbientDimensionMismatch {
                object_dimension,
                reduction_dimension: reduction.ambient_dim(),
            });
        }
        self.reductions[object.index()] = Some(reduction);
        Ok(())
    }

    /// Remove an object's reduction annotation.
    pub fn clear_coordinate_reduction(&mut self, object: ObjectId) -> Result<(), IrError> {
        self.object(object)?;
        self.reductions[object.index()] = None;
        Ok(())
    }

    /// Read one object, rejecting a handle created by another IR instance.
    pub fn object(&self, id: ObjectId) -> Result<&HilbertObject, IrError> {
        if id.owner != self.owner {
            return Err(IrError::ForeignObjectHandle(id));
        }
        self.objects
            .get(id.index())
            .ok_or(IrError::UnknownObject(id))
    }

    /// Read one node, rejecting a handle created by another IR instance.
    pub fn node(&self, id: NodeId) -> Result<&IrNode, IrError> {
        if id.owner != self.owner {
            return Err(IrError::ForeignNodeHandle(id));
        }
        self.nodes.get(id.index()).ok_or(IrError::UnknownNode(id))
    }

    /// Read one optional coordinate-reduction annotation.
    pub fn coordinate_reduction(
        &self,
        object: ObjectId,
    ) -> Result<Option<&CoordinateReduction>, IrError> {
        self.object(object)?;
        Ok(self.reductions[object.index()].as_ref())
    }

    /// Evaluate a boundary-free subgraph into the Stage-0 linear carrier.
    pub fn evaluate_linear(&self, node: NodeId) -> Result<RealLinearMap, IrError> {
        let node_value = self.node(node)?;
        match &node_value.kind {
            IrNodeKind::LinearMap(map) => Ok(map.clone()),
            IrNodeKind::Identity => Ok(RealLinearMap::identity(
                self.object(node_value.domain)?.dimension(),
            )?),
            IrNodeKind::Compose { outer, inner } => {
                let outer_map = self.evaluate_linear(*outer)?;
                let inner_map = self.evaluate_linear(*inner)?;
                Ok(outer_map.compose(&inner_map)?)
            }
            IrNodeKind::Dagger { source } => Ok(self.evaluate_linear(*source)?.dagger()),
            IrNodeKind::NonlinearBoundary { boundary } => {
                Err(IrError::LinearEvaluationCrossesBoundary(*boundary))
            }
        }
    }

    /// Evaluate one linear subgraph and explicitly reduce its endpoint objects.
    pub fn evaluate_reduced_linear(&self, node: NodeId) -> Result<RealLinearMap, IrError> {
        let node_value = self.node(node)?;
        let domain = node_value.domain;
        let codomain = node_value.codomain;
        let map = self.evaluate_linear(node)?;
        Ok(reduce_linear_map(
            &map,
            self.required_reduction(domain)?,
            self.required_reduction(codomain)?,
        )?)
    }

    /// Audit the Stage-0 omitted-middle-path structural defect for a compose node.
    pub fn composition_reduction_defect_max_abs(&self, node: NodeId) -> Result<f64, IrError> {
        let node_value = self.node(node)?;
        let (outer, inner) = match &node_value.kind {
            IrNodeKind::Compose { outer, inner } => (*outer, *inner),
            _ => return Err(IrError::NotCompositionNode(node)),
        };
        let outer_node = self.node(outer)?;
        let inner_node = self.node(inner)?;
        let domain = inner_node.domain;
        let middle = inner_node.codomain;
        let codomain = outer_node.codomain;
        let first = self.evaluate_linear(inner)?;
        let second = self.evaluate_linear(outer)?;
        Ok(reduction_composition_defect_max_abs(
            &first,
            &second,
            self.required_reduction(domain)?,
            self.required_reduction(middle)?,
            self.required_reduction(codomain)?,
        )?)
    }

    fn push_object(
        &mut self,
        name: String,
        dimension: usize,
        kind: HilbertObjectKind,
    ) -> Result<ObjectId, IrError> {
        if self
            .objects
            .iter()
            .any(|object| object.name.as_str() == name.as_str())
        {
            return Err(IrError::DuplicateObjectName(name));
        }
        let id = ObjectId {
            owner: self.owner,
            index: self.objects.len(),
        };
        self.objects.push(HilbertObject {
            name,
            dimension,
            kind,
        });
        self.reductions.push(None);
        Ok(id)
    }

    fn push_node(&mut self, domain: ObjectId, codomain: ObjectId, kind: IrNodeKind) -> NodeId {
        let id = NodeId {
            owner: self.owner,
            index: self.nodes.len(),
        };
        self.nodes.push(IrNode {
            domain,
            codomain,
            kind,
        });
        id
    }

    fn required_reduction(&self, object: ObjectId) -> Result<&CoordinateReduction, IrError> {
        self.coordinate_reduction(object)?
            .ok_or(IrError::MissingReductionAnnotation(object))
    }

    fn first_nonlinear_boundary(
        &self,
        node: NodeId,
    ) -> Result<Option<NonlinearBoundaryKind>, IrError> {
        match &self.node(node)?.kind {
            IrNodeKind::LinearMap(_) | IrNodeKind::Identity => Ok(None),
            IrNodeKind::NonlinearBoundary { boundary } => Ok(Some(*boundary)),
            IrNodeKind::Dagger { source } => self.first_nonlinear_boundary(*source),
            IrNodeKind::Compose { outer, inner } => {
                if let Some(boundary) = self.first_nonlinear_boundary(*outer)? {
                    return Ok(Some(boundary));
                }
                self.first_nonlinear_boundary(*inner)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CATEGORICAL_IR_CONTRACT, CategoricalAttentionIr, HilbertObjectKind, IrError,
        NonlinearBoundaryKind,
    };
    use crate::experimental::tdi23_categorical::RealLinearMap;
    use crate::experimental::tdi23_reduction::CoordinateReduction;

    #[test]
    fn tdi23_1_ir_contract_is_versioned() {
        assert_eq!(
            CATEGORICAL_IR_CONTRACT,
            "tdi23.1-categorical-attention-ir-v1"
        );
    }

    #[test]
    fn tdi23_1_handles_are_bound_to_their_owning_ir() {
        let mut left = CategoricalAttentionIr::new();
        let left_object = left.add_atomic_object("H", 2).expect("left object");
        let left_node = left.add_identity(left_object).expect("left node");

        let mut right = CategoricalAttentionIr::new();
        let right_object = right.add_atomic_object("H", 2).expect("right object");
        let right_node = right.add_identity(right_object).expect("right node");

        assert_eq!(
            right.add_identity(left_object),
            Err(IrError::ForeignObjectHandle(left_object))
        );
        assert_eq!(
            right.compose(right_node, left_node),
            Err(IrError::ForeignNodeHandle(left_node))
        );
        assert_eq!(
            left.compose(left_node, right_node),
            Err(IrError::ForeignNodeHandle(right_node))
        );
    }

    #[test]
    fn tdi23_1_direct_sum_and_tensor_product_are_distinct() {
        let mut ir = CategoricalAttentionIr::new();
        let h2 = ir.add_atomic_object("H2", 2).expect("H2");
        let h3 = ir.add_atomic_object("H3", 3).expect("H3");
        let sum = ir
            .add_direct_sum_object("H2_plus_H3", &[h2, h3])
            .expect("sum");
        let tensor = ir
            .add_tensor_product_object("H2_tensor_H3", &[h2, h3])
            .expect("tensor");
        assert_eq!(ir.object(sum).expect("sum object").dimension(), 5);
        assert_eq!(ir.object(tensor).expect("tensor object").dimension(), 6);
        assert!(matches!(
            ir.object(sum).expect("sum object").kind(),
            HilbertObjectKind::DirectSum(_)
        ));
        assert!(matches!(
            ir.object(tensor).expect("tensor object").kind(),
            HilbertObjectKind::TensorProduct(_)
        ));
    }

    #[test]
    fn tdi23_1_composition_requires_exact_middle_object_identity() {
        let mut ir = CategoricalAttentionIr::new();
        let a = ir.add_atomic_object("A", 2).expect("A");
        let b = ir.add_atomic_object("B", 2).expect("B");
        let c = ir.add_atomic_object("C", 2).expect("C");
        let outer = ir
            .add_linear_map(a, b, RealLinearMap::identity(2).expect("outer map"))
            .expect("outer");
        let inner = ir
            .add_linear_map(c, c, RealLinearMap::identity(2).expect("inner map"))
            .expect("inner");
        assert_eq!(
            ir.compose(outer, inner),
            Err(IrError::CompositionObjectMismatch {
                outer_domain: a,
                inner_codomain: c,
            })
        );
    }

    #[test]
    fn tdi23_1_dagger_and_composition_match_stage0_semantics() {
        let mut ir = CategoricalAttentionIr::new();
        let a = ir.add_atomic_object("A", 2).expect("A");
        let b = ir.add_atomic_object("B", 3).expect("B");
        let c = ir.add_atomic_object("C", 2).expect("C");
        let f_map = RealLinearMap::new(2, 3, vec![1.0, 2.0, 0.0, 1.0, 3.0, -1.0]).expect("f");
        let g_map = RealLinearMap::new(3, 2, vec![2.0, 0.0, 1.0, -1.0, 4.0, 2.0]).expect("g");
        let f = ir.add_linear_map(a, b, f_map.clone()).expect("f node");
        let g = ir.add_linear_map(b, c, g_map.clone()).expect("g node");
        let composed = ir.compose(g, f).expect("g o f");
        let dagger = ir.dagger(composed).expect("dagger");
        let actual = ir.evaluate_linear(dagger).expect("evaluate");
        let expected = f_map
            .dagger()
            .compose(&g_map.dagger())
            .expect("expected dagger composition");
        assert_eq!(actual, expected);
    }

    #[test]
    fn tdi23_1_identity_compositions_are_semantically_neutral() {
        let mut ir = CategoricalAttentionIr::new();
        let a = ir.add_atomic_object("A", 2).expect("A");
        let b = ir.add_atomic_object("B", 2).expect("B");
        let map = RealLinearMap::new(2, 2, vec![1.0, 2.0, 3.0, 4.0]).expect("map");
        let f = ir.add_linear_map(a, b, map.clone()).expect("f");
        let id_a = ir.add_identity(a).expect("id A");
        let id_b = ir.add_identity(b).expect("id B");
        let right = ir.compose(f, id_a).expect("f o id_A");
        let left = ir.compose(id_b, f).expect("id_B o f");
        assert_eq!(ir.evaluate_linear(right).expect("right"), map);
        assert_eq!(ir.evaluate_linear(left).expect("left"), map);
    }

    #[test]
    fn tdi23_1_nonlinear_boundary_fails_closed_for_linear_semantics() {
        let mut ir = CategoricalAttentionIr::new();
        let h = ir.add_atomic_object("H", 3).expect("H");
        let softmax = ir
            .add_nonlinear_boundary(h, h, NonlinearBoundaryKind::Softmax)
            .expect("softmax");
        let identity = ir.add_identity(h).expect("identity");
        let pipeline = ir.compose(identity, softmax).expect("pipeline");
        assert_eq!(
            ir.evaluate_linear(pipeline),
            Err(IrError::LinearEvaluationCrossesBoundary(
                NonlinearBoundaryKind::Softmax
            ))
        );
        assert_eq!(
            ir.dagger(pipeline),
            Err(IrError::DaggerCrossesNonlinearBoundary(
                NonlinearBoundaryKind::Softmax
            ))
        );
    }

    #[test]
    fn tdi23_1_reduced_lowering_uses_explicit_object_annotations() {
        let mut ir = CategoricalAttentionIr::new();
        let h = ir.add_atomic_object("H", 3).expect("H");
        let k = ir.add_atomic_object("K", 2).expect("K");
        let map = RealLinearMap::new(3, 2, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0]).expect("map");
        let node = ir.add_linear_map(h, k, map).expect("node");
        ir.annotate_coordinate_reduction(
            h,
            CoordinateReduction::new(3, vec![2, 0]).expect("H reduction"),
        )
        .expect("annotate H");
        ir.annotate_coordinate_reduction(
            k,
            CoordinateReduction::new(2, vec![1]).expect("K reduction"),
        )
        .expect("annotate K");
        let reduced = ir.evaluate_reduced_linear(node).expect("reduced");
        assert_eq!(reduced.domain_dim(), 2);
        assert_eq!(reduced.codomain_dim(), 1);
        assert_eq!(reduced.entries(), &[6.0, 4.0]);
    }

    #[test]
    fn tdi23_1_reduced_lowering_requires_annotations() {
        let mut ir = CategoricalAttentionIr::new();
        let h = ir.add_atomic_object("H", 2).expect("H");
        let identity = ir.add_identity(h).expect("identity");
        assert_eq!(
            ir.evaluate_reduced_linear(identity),
            Err(IrError::MissingReductionAnnotation(h))
        );
    }

    #[test]
    fn tdi23_1_composition_defect_tracks_omitted_middle_paths() {
        let mut ir = CategoricalAttentionIr::new();
        let a = ir.add_atomic_object("A", 1).expect("A");
        let b = ir.add_atomic_object("B", 2).expect("B");
        let c = ir.add_atomic_object("C", 1).expect("C");
        let first = ir
            .add_linear_map(
                a,
                b,
                RealLinearMap::new(1, 2, vec![1.0, 1.0]).expect("first map"),
            )
            .expect("first");
        let second = ir
            .add_linear_map(
                b,
                c,
                RealLinearMap::new(2, 1, vec![1.0, 1.0]).expect("second map"),
            )
            .expect("second");
        let composed = ir.compose(second, first).expect("compose");
        ir.annotate_coordinate_reduction(
            a,
            CoordinateReduction::new(1, vec![0]).expect("A reduction"),
        )
        .expect("annotate A");
        ir.annotate_coordinate_reduction(
            b,
            CoordinateReduction::new(2, vec![0]).expect("B reduction"),
        )
        .expect("annotate B");
        ir.annotate_coordinate_reduction(
            c,
            CoordinateReduction::new(1, vec![0]).expect("C reduction"),
        )
        .expect("annotate C");
        assert_eq!(
            ir.composition_reduction_defect_max_abs(composed)
                .expect("dropped path"),
            1.0
        );
        ir.annotate_coordinate_reduction(
            b,
            CoordinateReduction::new(2, vec![0, 1]).expect("full B reduction"),
        )
        .expect("replace B reduction");
        assert_eq!(
            ir.composition_reduction_defect_max_abs(composed)
                .expect("full middle"),
            0.0
        );
    }
}
