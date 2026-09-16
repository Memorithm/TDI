//! TDI-23.1 typed Categorical Attention IR development scaffold.
//!
//! This module introduces a bounded graph IR over the TDI-23 real finite-
//! dimensional dagger carrier. It keeps object identity, direct sum versus
//! tensor product, nonlinear boundaries, and coordinate-reduction annotations
//! explicit. It is a development grammar and legality oracle, not a rewrite
//! search engine, not a complete attention implementation, and not evidence of
//! quality, compression, asymptotic, or hardware-performance improvement.

use core::fmt;

use super::tdi23_categorical::{DaggerError, RealLinearMap};
use super::tdi23_reduction::{
    CoordinateReduction, ReductionError, reduce_linear_map,
    reduction_composition_defect_max_abs,
};

/// Versioned non-final TDI-23.1 IR contract.
pub const CATEGORICAL_IR_CONTRACT: &str = "tdi23.1-categorical-attention-ir-v1";

/// Stable object handle inside one [`CategoricalAttentionIr`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ObjectId(usize);

impl ObjectId {
    /// Zero-based object index, useful for deterministic provenance records.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0
    }
}

/// Stable node handle inside one [`CategoricalAttentionIr`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(usize);

impl NodeId {
    /// Zero-based node index, useful for deterministic provenance records.
    #[must_use]
    pub const fn index(self) -> usize {
        self.0
    }
}

/// Construction of one finite-dimensional real Hilbert object.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HilbertObjectKind {
    /// One atomic declared real Hilbert space.
    Atomic,
    /// Direct sum / biproduct-like concatenation of declared objects.
    DirectSum(Vec<ObjectId>),
    /// Tensor product of declared objects with multiplicative dimension.
    TensorProduct(Vec<ObjectId>),
}

/// One named finite-dimensional real Hilbert object in the IR.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HilbertObject {
    name: String,
    dimension: usize,
    kind: HilbertObjectKind,
}

impl HilbertObject {
    /// Stable human-readable object name within this IR instance.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Declared finite Hilbert-space dimension.
    #[must_use]
    pub const fn dimension(&self) -> usize {
        self.dimension
    }

    /// Structural construction of the object.
    #[must_use]
    pub fn kind(&self) -> &HilbertObjectKind {
        &self.kind
    }
}

/// Opaque semantic boundary that is deliberately not a `FdHilb` morphism.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NonlinearBoundaryKind {
    /// Softmax or equivalent normalization boundary.
    Softmax,
    /// Boolean-logic boundary.
    Boolean,
    /// Finite-field `F2` boundary.
    F2,
    /// Algebraic-normal-form / Zhegalkin boundary.
    Anf,
    /// Max-plus / tropical boundary.
    MaxPlus,
}

impl fmt::Display for NonlinearBoundaryKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Softmax => "softmax",
            Self::Boolean => "boolean",
            Self::F2 => "F2",
            Self::Anf => "ANF/Zhegalkin",
            Self::MaxPlus => "max-plus",
        };
        formatter.write_str(name)
    }
}

/// One typed IR node.
#[derive(Clone, Debug, PartialEq)]
pub enum IrNodeKind {
    /// Concrete Stage-0 real linear map.
    LinearMap(RealLinearMap),
    /// Identity morphism on one object.
    Identity,
    /// Typed composition `outer o inner`.
    Compose {
        /// Outer/left node.
        outer: NodeId,
        /// Inner/right node.
        inner: NodeId,
    },
    /// Dagger of a linear subgraph.
    Dagger {
        /// Source node whose domain/codomain are reversed.
        source: NodeId,
    },
    /// Explicit opaque non-`FdHilb` operation.
    NonlinearBoundary {
        /// Algebraic/nonlinear domain crossed by this operation.
        boundary: NonlinearBoundaryKind,
    },
}

/// One node with explicit source and target objects.
#[derive(Clone, Debug, PartialEq)]
pub struct IrNode {
    domain: ObjectId,
    codomain: ObjectId,
    kind: IrNodeKind,
}

impl IrNode {
    /// Exact declared domain object.
    #[must_use]
    pub const fn domain(&self) -> ObjectId {
        self.domain
    }

    /// Exact declared codomain object.
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

/// Fail-closed errors for the TDI-23.1 graph grammar and legality oracle.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IrError {
    /// Object names are unique within one IR instance.
    DuplicateObjectName {
        /// Reused name.
        name: String,
    },
    /// Composite objects require at least one factor/summand.
    EmptyObjectConstruction {
        /// Construction being attempted.
        construction: &'static str,
    },
    /// A dimension was zero.
    ZeroDimension,
    /// Direct-sum addition or tensor multiplication overflowed `usize`.
    DimensionOverflow,
    /// Object handle does not belong to this IR.
    UnknownObject {
        /// Invalid handle.
        object: ObjectId,
    },
    /// Node handle does not belong to this IR.
    UnknownNode {
        /// Invalid handle.
        node: NodeId,
    },
    /// Concrete map domain does not match the declared object dimension.
    MapDomainDimensionMismatch {
        /// Object dimension.
        object_dimension: usize,
        /// Map dimension.
        map_dimension: usize,
    },
    /// Concrete map codomain does not match the declared object dimension.
    MapCodomainDimensionMismatch {
        /// Object dimension.
        object_dimension: usize,
        /// Map dimension.
        map_dimension: usize,
    },
    /// Composition requires exact middle-object identity, not only equal size.
    CompositionObjectMismatch {
        /// Domain object expected by the outer node.
        outer_domain: ObjectId,
        /// Codomain object produced by the inner node.
        inner_codomain: ObjectId,
    },
    /// Dagger is illegal when the selected subgraph crosses a nonlinear domain.
    DaggerCrossesNonlinearBoundary {
        /// First detected opaque boundary.
        boundary: NonlinearBoundaryKind,
    },
    /// Linear evaluation reached an explicitly nonlinear boundary.
    LinearEvaluationCrossesBoundary {
        /// Boundary that prevented linear evaluation.
        boundary: NonlinearBoundaryKind,
    },
    /// Coordinate reduction ambient size does not match the annotated object.
    ReductionAmbientDimensionMismatch {
        /// Object dimension.
        object_dimension: usize,
        /// Reduction ambient dimension.
        reduction_dimension: usize,
    },
    /// Reduced lowering requires an explicit object annotation.
    MissingReductionAnnotation {
        /// Object lacking a reduction annotation.
        object: ObjectId,
    },
    /// Structural composition-defect auditing requires a composition node.
    NotCompositionNode {
        /// Node that was requested.
        node: NodeId,
    },
    /// Underlying Stage-0 linear carrier rejected an operation.
    Linear(DaggerError),
    /// Underlying Stage-0 reduction carrier rejected an operation.
    Reduction(ReductionError),
}

impl fmt::Display for IrError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateObjectName { name } => {
                write!(formatter, "object name {name:?} is already declared")
            }
            Self::EmptyObjectConstruction { construction } => {
                write!(formatter, "{construction} requires at least one object")
            }
            Self::ZeroDimension => formatter.write_str("object dimension must be positive"),
            Self::DimensionOverflow => formatter.write_str("object dimension overflowed usize"),
            Self::UnknownObject { object } => {
                write!(formatter, "unknown object id {}", object.index())
            }
            Self::UnknownNode { node } => write!(formatter, "unknown node id {}", node.index()),
            Self::MapDomainDimensionMismatch {
                object_dimension,
                map_dimension,
            } => write!(
                formatter,
                "map domain dimension mismatch: object={object_dimension}, map={map_dimension}"
            ),
            Self::MapCodomainDimensionMismatch {
                object_dimension,
                map_dimension,
            } => write!(
                formatter,
                "map codomain dimension mismatch: object={object_dimension}, map={map_dimension}"
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
            Self::DaggerCrossesNonlinearBoundary { boundary } => {
                write!(formatter, "dagger cannot cross {boundary} boundary")
            }
            Self::LinearEvaluationCrossesBoundary { boundary } => {
                write!(formatter, "linear evaluation cannot cross {boundary} boundary")
            }
            Self::ReductionAmbientDimensionMismatch {
                object_dimension,
                reduction_dimension,
            } => write!(
                formatter,
                "reduction ambient dimension mismatch: object={object_dimension}, reduction={reduction_dimension}"
            ),
            Self::MissingReductionAnnotation { object } => {
                write!(
                    formatter,
                    "object {} has no coordinate-reduction annotation",
                    object.index()
                )
            }
            Self::NotCompositionNode { node } => {
                write!(formatter, "node {} is not a composition", node.index())
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

/// Bounded typed DAG for the TDI-23.1 categorical-attention development grammar.
#[derive(Clone, Debug, Default)]
pub struct CategoricalAttentionIr {
    objects: Vec<HilbertObject>,
    reductions: Vec<Option<CoordinateReduction>>,
    nodes: Vec<IrNode>,
}

impl CategoricalAttentionIr {
    /// Create an empty IR.
    #[must_use]
    pub const fn new() -> Self {
        Self {
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

    /// Add an explicit direct-sum object with additive dimension.
    pub fn add_direct_sum_object(
        &mut self,
        name: impl Into<String>,
        summands: &[ObjectId],
    ) -> Result<ObjectId, IrError> {
        if summands.is_empty() {
            return Err(IrError::EmptyObjectConstruction {
                construction: "direct sum",
            });
        }
        let mut dimension = 0usize;
        for object in summands {
            dimension = dimension
                .checked_add(self.object(*object)?.dimension())
                .ok_or(IrError::DimensionOverflow)?;
        }
        self.push_object(
            name.into(),
            dimension,
            HilbertObjectKind::DirectSum(summands.to_vec()),
        )
    }

    /// Add an explicit tensor-product object with multiplicative dimension.
    pub fn add_tensor_product_object(
        &mut self,
        name: impl Into<String>,
        factors: &[ObjectId],
    ) -> Result<ObjectId, IrError> {
        if factors.is_empty() {
            return Err(IrError::EmptyObjectConstruction {
                construction: "tensor product",
            });
        }
        let mut dimension = 1usize;
        for object in factors {
            dimension = dimension
                .checked_mul(self.object(*object)?.dimension())
                .ok_or(IrError::DimensionOverflow)?;
        }
        self.push_object(
            name.into(),
            dimension,
            HilbertObjectKind::TensorProduct(factors.to_vec()),
        )
    }

    /// Add a concrete Stage-0 real linear map between exact declared objects.
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

    /// Add the identity morphism for one exact object.
    pub fn add_identity(&mut self, object: ObjectId) -> Result<NodeId, IrError> {
        self.object(object)?;
        Ok(self.push_node(object, object, IrNodeKind::Identity))
    }

    /// Add an explicit opaque non-`FdHilb` boundary.
    ///
    /// Such a node can participate in a typed pipeline, but linear evaluation
    /// and dagger construction fail closed when they encounter it.
    pub fn add_nonlinear_boundary(
        &mut self,
        domain: ObjectId,
        codomain: ObjectId,
        boundary: NonlinearBoundaryKind,
    ) -> Result<NodeId, IrError> {
        self.object(domain)?;
        self.object(codomain)?;
        Ok(self.push_node(
            domain,
            codomain,
            IrNodeKind::NonlinearBoundary { boundary },
        ))
    }

    /// Compose `outer o inner` when the exact middle object is identical.
    pub fn compose(&mut self, outer: NodeId, inner: NodeId) -> Result<NodeId, IrError> {
        let outer_node = self.node(outer)?;
        let inner_node = self.node(inner)?;
        if outer_node.domain != inner_node.codomain {
            return Err(IrError::CompositionObjectMismatch {
                outer_domain: outer_node.domain,
                inner_codomain: inner_node.codomain,
            });
        }
        let domain = inner_node.domain;
        let codomain = outer_node.codomain;
        Ok(self.push_node(domain, codomain, IrNodeKind::Compose { outer, inner }))
    }

    /// Construct the dagger of a linear subgraph.
    ///
    /// The operation is rejected before graph mutation if any nonlinear boundary
    /// is reachable from the source expression.
    pub fn dagger(&mut self, source: NodeId) -> Result<NodeId, IrError> {
        let source_node = self.node(source)?;
        if let Some(boundary) = self.first_nonlinear_boundary(source)? {
            return Err(IrError::DaggerCrossesNonlinearBoundary { boundary });
        }
        Ok(self.push_node(
            source_node.codomain,
            source_node.domain,
            IrNodeKind::Dagger { source },
        ))
    }

    /// Attach or replace the explicit coordinate reduction for one object.
    pub fn annotate_coordinate_reduction(
        &mut self,
        object: ObjectId,
        reduction: CoordinateReduction,
    ) -> Result<(), IrError> {
        let dimension = self.object(object)?.dimension();
        if reduction.ambient_dim() != dimension {
            return Err(IrError::ReductionAmbientDimensionMismatch {
                object_dimension: dimension,
                reduction_dimension: reduction.ambient_dim(),
            });
        }
        self.reductions[object.index()] = Some(reduction);
        Ok(())
    }

    /// Remove the reduction annotation for one object.
    pub fn clear_coordinate_reduction(&mut self, object: ObjectId) -> Result<(), IrError> {
        self.object(object)?;
        self.reductions[object.index()] = None;
        Ok(())
    }

    /// Read one object by stable handle.
    pub fn object(&self, object: ObjectId) -> Result<&HilbertObject, IrError> {
        self.objects
            .get(object.index())
            .ok_or(IrError::UnknownObject { object })
    }

    /// Read one node by stable handle.
    pub fn node(&self, node: NodeId) -> Result<&IrNode, IrError> {
        self.nodes.get(node.index()).ok_or(IrError::UnknownNode { node })
    }

    /// Read the optional coordinate-reduction annotation for one object.
    pub fn coordinate_reduction(
        &self,
        object: ObjectId,
    ) -> Result<Option<&CoordinateReduction>, IrError> {
        self.object(object)?;
        Ok(self.reductions[object.index()].as_ref())
    }

    /// Evaluate a boundary-free IR subgraph into the Stage-0 linear carrier.
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
                Err(IrError::LinearEvaluationCrossesBoundary {
                    boundary: *boundary,
                })
            }
        }
    }

    /// Evaluate one boundary-free subgraph and then apply the declared object
    /// reductions to its source and target.
    pub fn evaluate_reduced_linear(&self, node: NodeId) -> Result<RealLinearMap, IrError> {
        let node_value = self.node(node)?;
        let map = self.evaluate_linear(node)?;
        let domain = self.required_reduction(node_value.domain)?;
        let codomain = self.required_reduction(node_value.codomain)?;
        Ok(reduce_linear_map(&map, domain, codomain)?)
    }

    /// Audit the Stage-0 omitted-middle-path composition defect for one compose node.
    ///
    /// This uses the exact object annotations on the composition domain, middle,
    /// and codomain. It reports the Stage-0 structural diagnostic and does not
    /// claim that the complete IR is functorially reducible.
    pub fn composition_reduction_defect_max_abs(&self, node: NodeId) -> Result<f64, IrError> {
        let node_value = self.node(node)?;
        let (outer, inner) = match node_value.kind {
            IrNodeKind::Compose { outer, inner } => (outer, inner),
            _ => return Err(IrError::NotCompositionNode { node }),
        };
        let outer_node = self.node(outer)?;
        let inner_node = self.node(inner)?;
        let first = self.evaluate_linear(inner)?;
        let second = self.evaluate_linear(outer)?;
        let domain = self.required_reduction(inner_node.domain)?;
        let middle = self.required_reduction(inner_node.codomain)?;
        let codomain = self.required_reduction(outer_node.codomain)?;
        Ok(reduction_composition_defect_max_abs(
            &first, &second, domain, middle, codomain,
        )?)
    }

    fn push_object(
        &mut self,
        name: String,
        dimension: usize,
        kind: HilbertObjectKind,
    ) -> Result<ObjectId, IrError> {
        if self.objects.iter().any(|object| object.name == name) {
            return Err(IrError::DuplicateObjectName { name });
        }
        let id = ObjectId(self.objects.len());
        self.objects.push(HilbertObject {
            name,
            dimension,
            kind,
        });
        self.reductions.push(None);
        Ok(id)
    }

    fn push_node(&mut self, domain: ObjectId, codomain: ObjectId, kind: IrNodeKind) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(IrNode {
            domain,
            codomain,
            kind,
        });
        id
    }

    fn required_reduction(&self, object: ObjectId) -> Result<&CoordinateReduction, IrError> {
        self.coordinate_reduction(object)?
            .ok_or(IrError::MissingReductionAnnotation { object })
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
    fn tdi23_1_direct_sum_and_tensor_product_are_distinct() {
        let mut ir = CategoricalAttentionIr::new();
        let h2 = ir.add_atomic_object("H2", 2).expect("H2");
        let h3 = ir.add_atomic_object("H3", 3).expect("H3");
        let direct_sum = ir
            .add_direct_sum_object("H2_plus_H3", &[h2, h3])
            .expect("direct sum");
        let tensor = ir
            .add_tensor_product_object("H2_tensor_H3", &[h2, h3])
            .expect("tensor");

        assert_eq!(ir.object(direct_sum).expect("sum").dimension(), 5);
        assert_eq!(ir.object(tensor).expect("tensor").dimension(), 6);
        assert!(matches!(
            ir.object(direct_sum).expect("sum").kind(),
            HilbertObjectKind::DirectSum(_)
        ));
        assert!(matches!(
            ir.object(tensor).expect("tensor").kind(),
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
    fn tdi23_1_dagger_and_composition_match_stage0_linear_semantics() {
        let mut ir = CategoricalAttentionIr::new();
        let a = ir.add_atomic_object("A", 2).expect("A");
        let b = ir.add_atomic_object("B", 3).expect("B");
        let c = ir.add_atomic_object("C", 2).expect("C");
        let f_map = RealLinearMap::new(2, 3, vec![1.0, 2.0, 0.0, 1.0, 3.0, -1.0])
            .expect("f");
        let g_map = RealLinearMap::new(3, 2, vec![2.0, 0.0, 1.0, -1.0, 4.0, 2.0])
            .expect("g");
        let f = ir.add_linear_map(a, b, f_map.clone()).expect("f node");
        let g = ir.add_linear_map(b, c, g_map.clone()).expect("g node");
        let composed = ir.compose(g, f).expect("g o f");
        let dagger = ir.dagger(composed).expect("dagger");

        let actual = ir.evaluate_linear(dagger).expect("evaluate");
        let expected = f_map
            .dagger()
            .compose(&g_map.dagger())
            .expect("f^dagger o g^dagger");
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
        let right_identity = ir.compose(f, id_a).expect("f o id_A");
        let left_identity = ir.compose(id_b, f).expect("id_B o f");

        assert_eq!(ir.evaluate_linear(right_identity).expect("eval"), map);
        assert_eq!(ir.evaluate_linear(left_identity).expect("eval"), map);
    }

    #[test]
    fn tdi23_1_nonlinear_boundaries_are_explicit_and_fail_closed_for_dagger() {
        let mut ir = CategoricalAttentionIr::new();
        let h = ir.add_atomic_object("H", 3).expect("H");
        let softmax = ir
            .add_nonlinear_boundary(h, h, NonlinearBoundaryKind::Softmax)
            .expect("softmax boundary");
        let identity = ir.add_identity(h).expect("identity");
        let pipeline = ir.compose(identity, softmax).expect("typed pipeline");

        assert_eq!(
            ir.evaluate_linear(pipeline),
            Err(IrError::LinearEvaluationCrossesBoundary {
                boundary: NonlinearBoundaryKind::Softmax,
            })
        );
        assert_eq!(
            ir.dagger(pipeline),
            Err(IrError::DaggerCrossesNonlinearBoundary {
                boundary: NonlinearBoundaryKind::Softmax,
            })
        );
    }

    #[test]
    fn tdi23_1_reduced_lowering_uses_explicit_object_annotations() {
        let mut ir = CategoricalAttentionIr::new();
        let h = ir.add_atomic_object("H", 3).expect("H");
        let k = ir.add_atomic_object("K", 2).expect("K");
        let map = RealLinearMap::new(3, 2, vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0])
            .expect("map");
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

        let reduced = ir.evaluate_reduced_linear(node).expect("reduced map");
        assert_eq!(reduced.domain_dim(), 2);
        assert_eq!(reduced.codomain_dim(), 1);
        assert_eq!(reduced.entries(), &[6.0, 4.0]);
    }

    #[test]
    fn tdi23_1_reduced_lowering_fails_when_annotation_is_missing() {
        let mut ir = CategoricalAttentionIr::new();
        let h = ir.add_atomic_object("H", 2).expect("H");
        let node = ir.add_identity(h).expect("identity");

        assert_eq!(
            ir.evaluate_reduced_linear(node),
            Err(IrError::MissingReductionAnnotation { object: h })
        );
    }

    #[test]
    fn tdi23_1_composition_defect_uses_middle_object_annotation() {
        let mut ir = CategoricalAttentionIr::new();
        let a = ir.add_atomic_object("A", 1).expect("A");
        let b = ir.add_atomic_object("B", 2).expect("B");
        let c = ir.add_atomic_object("C", 1).expect("C");
        let first = ir
            .add_linear_map(
                a,
                b,
                RealLinearMap::new(1, 2, vec![1.0, 1.0]).expect("first"),
            )
            .expect("first node");
        let second = ir
            .add_linear_map(
                b,
                c,
                RealLinearMap::new(2, 1, vec![1.0, 1.0]).expect("second"),
            )
            .expect("second node");
        let composed = ir.compose(second, first).expect("composition");
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
                .expect("dropped-path defect"),
            1.0
        );

        ir.annotate_coordinate_reduction(
            b,
            CoordinateReduction::new(2, vec![0, 1]).expect("full B reduction"),
        )
        .expect("replace B reduction");
        assert_eq!(
            ir.composition_reduction_defect_max_abs(composed)
                .expect("full-space defect"),
            0.0
        );
    }
}
