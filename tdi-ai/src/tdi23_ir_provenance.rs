//! Deterministic rooted-subgraph validation and provenance for TDI-23.1.
//!
//! The manifest is a provenance format, not a claim of graph-isomorphism
//! canonicalization. It is deterministic for the relative construction order
//! of the reachable subgraph and excludes the process-local IR ownership token.

use core::fmt;
use std::collections::{BTreeMap, BTreeSet};

use super::tdi23_ir::{
    CATEGORICAL_IR_CONTRACT, CategoricalAttentionIr, HilbertObjectKind, IrError, IrNodeKind,
    NodeId, NonlinearBoundaryKind, ObjectId,
};

/// Versioned deterministic rooted-subgraph provenance format.
pub const ROOTED_MANIFEST_CONTRACT: &str = "tdi23.1-rooted-ir-manifest-v1";

/// Independent structural/provenance validation failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProvenanceError {
    /// Underlying IR lookup or carrier operation failed closed.
    Ir(IrError),
    /// A composite object references a component created at or after itself.
    ForwardObjectReference {
        /// Composite object index.
        object: usize,
        /// Referenced component index.
        referenced: usize,
    },
    /// A node references another node created at or after itself.
    ForwardNodeReference {
        /// Referencing node index.
        node: usize,
        /// Referenced node index.
        referenced: usize,
    },
    /// A direct-sum object's stored dimension differs from its components.
    DirectSumDimensionMismatch {
        /// Object index.
        object: usize,
        /// Stored dimension.
        stored: usize,
        /// Recomputed dimension.
        recomputed: usize,
    },
    /// A tensor-product object's stored dimension differs from its factors.
    TensorProductDimensionMismatch {
        /// Object index.
        object: usize,
        /// Stored dimension.
        stored: usize,
        /// Recomputed dimension.
        recomputed: usize,
    },
    /// An identity node does not use one exact object on both ends.
    IdentityEndpointMismatch {
        /// Node index.
        node: usize,
    },
    /// A linear-map node's carrier dimensions disagree with its objects.
    LinearMapDimensionMismatch {
        /// Node index.
        node: usize,
    },
    /// A composition node has inconsistent middle or outer endpoints.
    CompositionEndpointMismatch {
        /// Node index.
        node: usize,
    },
    /// A dagger node does not reverse the source endpoints.
    DaggerEndpointMismatch {
        /// Node index.
        node: usize,
    },
    /// A dagger source subgraph crosses an explicit nonlinear boundary.
    DaggerContainsNonlinearBoundary {
        /// Dagger node index.
        node: usize,
        /// First detected boundary.
        boundary: NonlinearBoundaryKind,
    },
    /// An attached coordinate reduction disagrees with the object's ambient size.
    ReductionAmbientMismatch {
        /// Object index.
        object: usize,
        /// Object dimension.
        object_dimension: usize,
        /// Reduction ambient dimension.
        reduction_dimension: usize,
    },
    /// An object name is duplicated inside the rooted reachable object set.
    DuplicateReachableObjectName {
        /// Duplicated name.
        name: String,
    },
    /// Dimension arithmetic overflowed while independently revalidating an object.
    DimensionOverflow {
        /// Object index.
        object: usize,
    },
    /// An independently validated reachable object was missing from manifest remapping.
    MissingObjectOrdinal {
        /// Original IR-local object index.
        object: usize,
    },
    /// An independently validated reachable node was missing from manifest remapping.
    MissingNodeOrdinal {
        /// Original IR-local node index.
        node: usize,
    },
}

impl fmt::Display for ProvenanceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ir(error) => write!(formatter, "IR validation error: {error}"),
            Self::ForwardObjectReference { object, referenced } => write!(
                formatter,
                "object {object} references non-prior object {referenced}"
            ),
            Self::ForwardNodeReference { node, referenced } => {
                write!(
                    formatter,
                    "node {node} references non-prior node {referenced}"
                )
            }
            Self::DirectSumDimensionMismatch {
                object,
                stored,
                recomputed,
            } => write!(
                formatter,
                "direct-sum object {object} dimension mismatch: stored={stored}, recomputed={recomputed}"
            ),
            Self::TensorProductDimensionMismatch {
                object,
                stored,
                recomputed,
            } => write!(
                formatter,
                "tensor object {object} dimension mismatch: stored={stored}, recomputed={recomputed}"
            ),
            Self::IdentityEndpointMismatch { node } => {
                write!(formatter, "identity node {node} has unequal endpoints")
            }
            Self::LinearMapDimensionMismatch { node } => {
                write!(
                    formatter,
                    "linear-map node {node} disagrees with object dimensions"
                )
            }
            Self::CompositionEndpointMismatch { node } => {
                write!(
                    formatter,
                    "composition node {node} has inconsistent endpoints"
                )
            }
            Self::DaggerEndpointMismatch { node } => {
                write!(
                    formatter,
                    "dagger node {node} does not reverse source endpoints"
                )
            }
            Self::DaggerContainsNonlinearBoundary { node, boundary } => {
                write!(formatter, "dagger node {node} crosses {boundary} boundary")
            }
            Self::ReductionAmbientMismatch {
                object,
                object_dimension,
                reduction_dimension,
            } => write!(
                formatter,
                "object {object} reduction ambient mismatch: object={object_dimension}, reduction={reduction_dimension}"
            ),
            Self::DuplicateReachableObjectName { name } => {
                write!(formatter, "reachable object name {name:?} is duplicated")
            }
            Self::DimensionOverflow { object } => {
                write!(
                    formatter,
                    "dimension overflow while validating object {object}"
                )
            }
            Self::MissingObjectOrdinal { object } => {
                write!(
                    formatter,
                    "reachable object {object} has no manifest ordinal"
                )
            }
            Self::MissingNodeOrdinal { node } => {
                write!(formatter, "reachable node {node} has no manifest ordinal")
            }
        }
    }
}

impl std::error::Error for ProvenanceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Ir(error) => Some(error),
            _ => None,
        }
    }
}

impl From<IrError> for ProvenanceError {
    fn from(value: IrError) -> Self {
        Self::Ir(value)
    }
}

/// Deterministic summary of one validated rooted subgraph.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RootedValidationSummary {
    root: NodeId,
    object_count: usize,
    node_count: usize,
    nonlinear_boundary_count: usize,
    reduction_annotation_count: usize,
}

impl RootedValidationSummary {
    /// Validated root node.
    #[must_use]
    pub const fn root(&self) -> NodeId {
        self.root
    }

    /// Number of reachable objects, including composite-object dependencies.
    #[must_use]
    pub const fn object_count(&self) -> usize {
        self.object_count
    }

    /// Number of reachable IR nodes.
    #[must_use]
    pub const fn node_count(&self) -> usize {
        self.node_count
    }

    /// Number of reachable explicit nonlinear-boundary nodes.
    #[must_use]
    pub const fn nonlinear_boundary_count(&self) -> usize {
        self.nonlinear_boundary_count
    }

    /// Number of reachable objects carrying coordinate-reduction annotations.
    #[must_use]
    pub const fn reduction_annotation_count(&self) -> usize {
        self.reduction_annotation_count
    }
}

#[derive(Default)]
struct Reachable {
    objects: BTreeMap<usize, ObjectId>,
    nodes: BTreeMap<usize, NodeId>,
    nonlinear_boundaries: usize,
}

/// Independently validate one rooted IR subgraph using the public IR surface.
///
/// This rechecks object construction dimensions, reference ordering, node
/// endpoints, dagger boundary discipline, and reduction ambient dimensions.
pub fn validate_rooted_subgraph(
    ir: &CategoricalAttentionIr,
    root: NodeId,
) -> Result<RootedValidationSummary, ProvenanceError> {
    let reachable = collect_and_validate(ir, root)?;

    let reduction_annotation_count = reachable
        .objects
        .values()
        .copied()
        .map(|object| ir.coordinate_reduction(object))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(Option::is_some)
        .count();

    Ok(RootedValidationSummary {
        root,
        object_count: reachable.objects.len(),
        node_count: reachable.nodes.len(),
        nonlinear_boundary_count: reachable.nonlinear_boundaries,
        reduction_annotation_count,
    })
}

/// Serialize one validated rooted subgraph into a deterministic textual manifest.
///
/// Runtime owner tokens and absolute IR-local indices are excluded. Reachable
/// objects and nodes are remapped to manifest-local ordinals in relative
/// construction order, so unrelated unreachable prefixes do not perturb the
/// manifest. This is provenance, not graph-isomorphism canonicalization.
pub fn canonical_rooted_manifest(
    ir: &CategoricalAttentionIr,
    root: NodeId,
) -> Result<String, ProvenanceError> {
    let reachable = collect_and_validate(ir, root)?;
    let object_ordinals = ordinals(reachable.objects.keys().copied());
    let node_ordinals = ordinals(reachable.nodes.keys().copied());
    let root_ordinal = mapped_node_ordinal(&node_ordinals, root)?;

    let mut output = String::new();
    output.push_str(ROOTED_MANIFEST_CONTRACT);
    output.push('\n');
    output.push_str("ir_contract=");
    output.push_str(CATEGORICAL_IR_CONTRACT);
    output.push('\n');
    output.push_str(&format!("root={root_ordinal}\n"));

    for (source_index, object_id) in &reachable.objects {
        let ordinal = object_ordinals.get(source_index).copied().ok_or(
            ProvenanceError::MissingObjectOrdinal {
                object: *source_index,
            },
        )?;
        let object = ir.object(*object_id)?;
        output.push_str(&format!(
            "object|{ordinal}|name_hex={}|dim={}|kind={}",
            hex_bytes(object.name().as_bytes()),
            object.dimension(),
            object_kind_token(object.kind(), &object_ordinals)?
        ));
        if let Some(reduction) = ir.coordinate_reduction(*object_id)? {
            output.push_str("|reduction=");
            output.push_str(&reduction.ambient_dim().to_string());
            output.push(':');
            push_usize_list(&mut output, reduction.kept_coordinates());
        } else {
            output.push_str("|reduction=none");
        }
        output.push('\n');
    }

    for (source_index, node_id) in &reachable.nodes {
        let ordinal = node_ordinals.get(source_index).copied().ok_or(
            ProvenanceError::MissingNodeOrdinal {
                node: *source_index,
            },
        )?;
        let node = ir.node(*node_id)?;
        output.push_str(&format!(
            "node|{ordinal}|domain={}|codomain={}|op=",
            mapped_object_ordinal(&object_ordinals, node.domain())?,
            mapped_object_ordinal(&object_ordinals, node.codomain())?
        ));
        match node.kind() {
            IrNodeKind::LinearMap(map) => {
                output.push_str("linear|bits=");
                for (position, value) in map.entries().iter().enumerate() {
                    if position > 0 {
                        output.push(',');
                    }
                    output.push_str(&format!("{:016x}", value.to_bits()));
                }
            }
            IrNodeKind::Identity => output.push_str("identity"),
            IrNodeKind::Compose { outer, inner } => {
                output.push_str(&format!(
                    "compose:{}:{}",
                    mapped_node_ordinal(&node_ordinals, *outer)?,
                    mapped_node_ordinal(&node_ordinals, *inner)?
                ));
            }
            IrNodeKind::Dagger { source } => {
                output.push_str(&format!(
                    "dagger:{}",
                    mapped_node_ordinal(&node_ordinals, *source)?
                ));
            }
            IrNodeKind::NonlinearBoundary { boundary } => {
                output.push_str("boundary:");
                output.push_str(boundary_token(*boundary));
            }
        }
        output.push('\n');
    }

    Ok(output)
}

fn collect_and_validate(
    ir: &CategoricalAttentionIr,
    root: NodeId,
) -> Result<Reachable, ProvenanceError> {
    let mut reachable = Reachable::default();
    let mut visiting = BTreeSet::new();
    let mut validated_nodes = BTreeMap::new();
    validate_node(
        ir,
        root,
        &mut reachable,
        &mut visiting,
        &mut validated_nodes,
    )?;

    let mut names = BTreeMap::<String, usize>::new();
    let mut validated_objects = BTreeSet::new();
    let object_ids = reachable.objects.values().copied().collect::<Vec<_>>();
    for object in object_ids {
        validate_object(
            ir,
            object,
            &mut reachable,
            &mut validated_objects,
            &mut names,
        )?;
    }
    Ok(reachable)
}

fn validate_node(
    ir: &CategoricalAttentionIr,
    node_id: NodeId,
    reachable: &mut Reachable,
    visiting: &mut BTreeSet<usize>,
    validated: &mut BTreeMap<usize, bool>,
) -> Result<bool, ProvenanceError> {
    let index = node_id.index();
    if let Some(has_boundary) = validated.get(&index) {
        return Ok(*has_boundary);
    }
    if !visiting.insert(index) {
        return Err(ProvenanceError::ForwardNodeReference {
            node: index,
            referenced: index,
        });
    }

    let node = ir.node(node_id)?;
    reachable.nodes.insert(index, node_id);
    reachable
        .objects
        .insert(node.domain().index(), node.domain());
    reachable
        .objects
        .insert(node.codomain().index(), node.codomain());
    let domain = ir.object(node.domain())?;
    let codomain = ir.object(node.codomain())?;

    let has_boundary = match node.kind() {
        IrNodeKind::LinearMap(map) => {
            if map.domain_dim() != domain.dimension() || map.codomain_dim() != codomain.dimension()
            {
                return Err(ProvenanceError::LinearMapDimensionMismatch { node: index });
            }
            false
        }
        IrNodeKind::Identity => {
            if node.domain() != node.codomain() {
                return Err(ProvenanceError::IdentityEndpointMismatch { node: index });
            }
            false
        }
        IrNodeKind::Compose { outer, inner } => {
            validate_prior_node_reference(index, *outer)?;
            validate_prior_node_reference(index, *inner)?;
            let outer_node = ir.node(*outer)?;
            let inner_node = ir.node(*inner)?;
            if outer_node.domain() != inner_node.codomain()
                || node.domain() != inner_node.domain()
                || node.codomain() != outer_node.codomain()
            {
                return Err(ProvenanceError::CompositionEndpointMismatch { node: index });
            }
            let outer_boundary = validate_node(ir, *outer, reachable, visiting, validated)?;
            let inner_boundary = validate_node(ir, *inner, reachable, visiting, validated)?;
            outer_boundary || inner_boundary
        }
        IrNodeKind::Dagger { source } => {
            validate_prior_node_reference(index, *source)?;
            let source_node = ir.node(*source)?;
            if node.domain() != source_node.codomain() || node.codomain() != source_node.domain() {
                return Err(ProvenanceError::DaggerEndpointMismatch { node: index });
            }
            let source_has_boundary = validate_node(ir, *source, reachable, visiting, validated)?;
            if source_has_boundary {
                let boundary = first_boundary(ir, *source, &mut BTreeSet::new())?
                    .expect("validated source reported a boundary");
                return Err(ProvenanceError::DaggerContainsNonlinearBoundary {
                    node: index,
                    boundary,
                });
            }
            false
        }
        IrNodeKind::NonlinearBoundary { .. } => {
            reachable.nonlinear_boundaries += 1;
            true
        }
    };

    visiting.remove(&index);
    validated.insert(index, has_boundary);
    Ok(has_boundary)
}

fn validate_object(
    ir: &CategoricalAttentionIr,
    object_id: ObjectId,
    reachable: &mut Reachable,
    validated: &mut BTreeSet<usize>,
    names: &mut BTreeMap<String, usize>,
) -> Result<(), ProvenanceError> {
    let index = object_id.index();
    if validated.contains(&index) {
        return Ok(());
    }
    let object = ir.object(object_id)?;
    if let Some(previous) = names.insert(object.name().to_owned(), index) {
        if previous != index {
            return Err(ProvenanceError::DuplicateReachableObjectName {
                name: object.name().to_owned(),
            });
        }
    }

    match object.kind() {
        HilbertObjectKind::Atomic => {}
        HilbertObjectKind::DirectSum(summands) => {
            if summands.is_empty() {
                return Err(ProvenanceError::DirectSumDimensionMismatch {
                    object: index,
                    stored: object.dimension(),
                    recomputed: 0,
                });
            }
            let mut recomputed = 0usize;
            for summand in summands {
                validate_prior_object_reference(index, *summand)?;
                let child = ir.object(*summand)?;
                reachable.objects.insert(summand.index(), *summand);
                recomputed = recomputed
                    .checked_add(child.dimension())
                    .ok_or(ProvenanceError::DimensionOverflow { object: index })?;
                validate_object(ir, *summand, reachable, validated, names)?;
            }
            if recomputed != object.dimension() {
                return Err(ProvenanceError::DirectSumDimensionMismatch {
                    object: index,
                    stored: object.dimension(),
                    recomputed,
                });
            }
        }
        HilbertObjectKind::TensorProduct(factors) => {
            if factors.is_empty() {
                return Err(ProvenanceError::TensorProductDimensionMismatch {
                    object: index,
                    stored: object.dimension(),
                    recomputed: 1,
                });
            }
            let mut recomputed = 1usize;
            for factor in factors {
                validate_prior_object_reference(index, *factor)?;
                let child = ir.object(*factor)?;
                reachable.objects.insert(factor.index(), *factor);
                recomputed = recomputed
                    .checked_mul(child.dimension())
                    .ok_or(ProvenanceError::DimensionOverflow { object: index })?;
                validate_object(ir, *factor, reachable, validated, names)?;
            }
            if recomputed != object.dimension() {
                return Err(ProvenanceError::TensorProductDimensionMismatch {
                    object: index,
                    stored: object.dimension(),
                    recomputed,
                });
            }
        }
    }

    if let Some(reduction) = ir.coordinate_reduction(object_id)? {
        if reduction.ambient_dim() != object.dimension() {
            return Err(ProvenanceError::ReductionAmbientMismatch {
                object: index,
                object_dimension: object.dimension(),
                reduction_dimension: reduction.ambient_dim(),
            });
        }
    }

    validated.insert(index);
    Ok(())
}

fn validate_prior_node_reference(node: usize, referenced: NodeId) -> Result<(), ProvenanceError> {
    if referenced.index() >= node {
        return Err(ProvenanceError::ForwardNodeReference {
            node,
            referenced: referenced.index(),
        });
    }
    Ok(())
}

fn validate_prior_object_reference(
    object: usize,
    referenced: ObjectId,
) -> Result<(), ProvenanceError> {
    if referenced.index() >= object {
        return Err(ProvenanceError::ForwardObjectReference {
            object,
            referenced: referenced.index(),
        });
    }
    Ok(())
}

fn first_boundary(
    ir: &CategoricalAttentionIr,
    node_id: NodeId,
    visiting: &mut BTreeSet<usize>,
) -> Result<Option<NonlinearBoundaryKind>, ProvenanceError> {
    if !visiting.insert(node_id.index()) {
        return Err(ProvenanceError::ForwardNodeReference {
            node: node_id.index(),
            referenced: node_id.index(),
        });
    }
    let result = match ir.node(node_id)?.kind() {
        IrNodeKind::LinearMap(_) | IrNodeKind::Identity => None,
        IrNodeKind::NonlinearBoundary { boundary } => Some(*boundary),
        IrNodeKind::Compose { outer, inner } => {
            first_boundary(ir, *outer, visiting)?.or(first_boundary(ir, *inner, visiting)?)
        }
        IrNodeKind::Dagger { source } => first_boundary(ir, *source, visiting)?,
    };
    visiting.remove(&node_id.index());
    Ok(result)
}

fn ordinals(indices: impl IntoIterator<Item = usize>) -> BTreeMap<usize, usize> {
    indices
        .into_iter()
        .enumerate()
        .map(|(ordinal, index)| (index, ordinal))
        .collect()
}

fn mapped_object_ordinal(
    ordinals: &BTreeMap<usize, usize>,
    object: ObjectId,
) -> Result<usize, ProvenanceError> {
    ordinals
        .get(&object.index())
        .copied()
        .ok_or(ProvenanceError::MissingObjectOrdinal {
            object: object.index(),
        })
}

fn mapped_node_ordinal(
    ordinals: &BTreeMap<usize, usize>,
    node: NodeId,
) -> Result<usize, ProvenanceError> {
    ordinals
        .get(&node.index())
        .copied()
        .ok_or(ProvenanceError::MissingNodeOrdinal { node: node.index() })
}

fn object_kind_token(
    kind: &HilbertObjectKind,
    object_ordinals: &BTreeMap<usize, usize>,
) -> Result<String, ProvenanceError> {
    match kind {
        HilbertObjectKind::Atomic => Ok("atomic".to_owned()),
        HilbertObjectKind::DirectSum(parts) => {
            let mut token = "direct_sum:".to_owned();
            push_mapped_object_list(&mut token, parts, object_ordinals)?;
            Ok(token)
        }
        HilbertObjectKind::TensorProduct(parts) => {
            let mut token = "tensor_product:".to_owned();
            push_mapped_object_list(&mut token, parts, object_ordinals)?;
            Ok(token)
        }
    }
}

fn push_mapped_object_list(
    output: &mut String,
    objects: &[ObjectId],
    ordinals: &BTreeMap<usize, usize>,
) -> Result<(), ProvenanceError> {
    for (position, object) in objects.iter().enumerate() {
        if position > 0 {
            output.push(',');
        }
        output.push_str(&mapped_object_ordinal(ordinals, *object)?.to_string());
    }
    Ok(())
}

fn boundary_token(boundary: NonlinearBoundaryKind) -> &'static str {
    match boundary {
        NonlinearBoundaryKind::Softmax => "softmax",
        NonlinearBoundaryKind::Boolean => "boolean",
        NonlinearBoundaryKind::F2 => "f2",
        NonlinearBoundaryKind::Anf => "anf_zhegalkin",
        NonlinearBoundaryKind::MaxPlus => "max_plus",
    }
}

fn hex_bytes(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push_str(&format!("{byte:02x}"));
    }
    encoded
}

fn push_usize_list(output: &mut String, values: &[usize]) {
    for (position, value) in values.iter().enumerate() {
        if position > 0 {
            output.push(',');
        }
        output.push_str(&value.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ProvenanceError, ROOTED_MANIFEST_CONTRACT, canonical_rooted_manifest,
        validate_rooted_subgraph,
    };
    use crate::experimental::tdi23_categorical::RealLinearMap;
    use crate::experimental::tdi23_ir::{CategoricalAttentionIr, NodeId, NonlinearBoundaryKind};
    use crate::experimental::tdi23_reduction::CoordinateReduction;

    fn build_linear_fixture(value: f64) -> (CategoricalAttentionIr, NodeId) {
        let mut ir = CategoricalAttentionIr::new();
        let h = ir.add_atomic_object("H", 2).expect("H");
        let k = ir.add_atomic_object("K", 1).expect("K");
        let map = RealLinearMap::new(2, 1, vec![value, 2.0]).expect("map");
        let node = ir.add_linear_map(h, k, map).expect("linear node");
        (ir, node)
    }

    fn build_linear_fixture_with_unreachable_prefix(
        value: f64,
    ) -> (CategoricalAttentionIr, NodeId) {
        let mut ir = CategoricalAttentionIr::new();
        let unused = ir.add_atomic_object("UNUSED", 3).expect("unused");
        let _unused_identity = ir.add_identity(unused).expect("unused identity");
        let h = ir.add_atomic_object("H", 2).expect("H");
        let k = ir.add_atomic_object("K", 1).expect("K");
        let map = RealLinearMap::new(2, 1, vec![value, 2.0]).expect("map");
        let node = ir.add_linear_map(h, k, map).expect("linear node");
        (ir, node)
    }

    #[test]
    fn tdi23_1_manifest_contract_is_versioned() {
        assert_eq!(ROOTED_MANIFEST_CONTRACT, "tdi23.1-rooted-ir-manifest-v1");
    }

    #[test]
    fn tdi23_1_manifest_ignores_runtime_owner_token() {
        let (left, left_root) = build_linear_fixture(1.0);
        let (right, right_root) = build_linear_fixture(1.0);
        let left_manifest = canonical_rooted_manifest(&left, left_root).expect("left manifest");
        let right_manifest = canonical_rooted_manifest(&right, right_root).expect("right manifest");
        assert_eq!(left_manifest, right_manifest);
        assert!(left_manifest.starts_with(ROOTED_MANIFEST_CONTRACT));
    }

    #[test]
    fn tdi23_1_manifest_ignores_unreachable_prefix() {
        let (plain, plain_root) = build_linear_fixture(1.0);
        let (prefixed, prefixed_root) = build_linear_fixture_with_unreachable_prefix(1.0);
        let plain_manifest = canonical_rooted_manifest(&plain, plain_root).expect("plain manifest");
        let prefixed_manifest =
            canonical_rooted_manifest(&prefixed, prefixed_root).expect("prefixed manifest");
        assert_eq!(plain_manifest, prefixed_manifest);
        assert!(!prefixed_manifest.contains("UNUSED"));
    }

    #[test]
    fn tdi23_1_manifest_preserves_f64_bit_patterns() {
        let (positive, positive_root) = build_linear_fixture(0.0);
        let (negative, negative_root) = build_linear_fixture(-0.0);
        let positive_manifest =
            canonical_rooted_manifest(&positive, positive_root).expect("positive zero");
        let negative_manifest =
            canonical_rooted_manifest(&negative, negative_root).expect("negative zero");
        assert_ne!(positive_manifest, negative_manifest);
        assert!(positive_manifest.contains("0000000000000000"));
        assert!(negative_manifest.contains("8000000000000000"));
    }

    #[test]
    fn tdi23_1_manifest_records_reduction_annotations() {
        let (mut reduced, root) = build_linear_fixture(1.0);
        let domain = reduced.node(root).expect("root").domain();
        reduced
            .annotate_coordinate_reduction(
                domain,
                CoordinateReduction::new(2, vec![1]).expect("reduction"),
            )
            .expect("annotate");
        let reduced_manifest = canonical_rooted_manifest(&reduced, root).expect("reduced manifest");

        let (plain, plain_root) = build_linear_fixture(1.0);
        let plain_manifest = canonical_rooted_manifest(&plain, plain_root).expect("plain manifest");
        assert_ne!(reduced_manifest, plain_manifest);
        assert!(reduced_manifest.contains("reduction=2:1"));
    }

    #[test]
    fn tdi23_1_manifest_distinguishes_direct_sum_and_tensor_product() {
        let mut sum_ir = CategoricalAttentionIr::new();
        let a = sum_ir.add_atomic_object("A", 2).expect("A");
        let b = sum_ir.add_atomic_object("B", 2).expect("B");
        let sum = sum_ir.add_direct_sum_object("AB", &[a, b]).expect("sum");
        let sum_root = sum_ir.add_identity(sum).expect("sum identity");

        let mut tensor_ir = CategoricalAttentionIr::new();
        let a = tensor_ir.add_atomic_object("A", 2).expect("A");
        let b = tensor_ir.add_atomic_object("B", 2).expect("B");
        let tensor = tensor_ir
            .add_tensor_product_object("AB", &[a, b])
            .expect("tensor");
        let tensor_root = tensor_ir.add_identity(tensor).expect("tensor identity");

        let sum_manifest = canonical_rooted_manifest(&sum_ir, sum_root).expect("sum manifest");
        let tensor_manifest =
            canonical_rooted_manifest(&tensor_ir, tensor_root).expect("tensor manifest");
        assert_ne!(sum_manifest, tensor_manifest);
        assert!(sum_manifest.contains("kind=direct_sum:0,1"));
        assert!(tensor_manifest.contains("kind=tensor_product:0,1"));
    }

    #[test]
    fn tdi23_1_root_validation_counts_boundaries_and_reductions() {
        let mut ir = CategoricalAttentionIr::new();
        let h = ir.add_atomic_object("H", 2).expect("H");
        ir.annotate_coordinate_reduction(
            h,
            CoordinateReduction::new(2, vec![0]).expect("reduction"),
        )
        .expect("annotate");
        let boundary = ir
            .add_nonlinear_boundary(h, h, NonlinearBoundaryKind::Boolean)
            .expect("boundary");
        let summary = validate_rooted_subgraph(&ir, boundary).expect("summary");
        assert_eq!(summary.object_count(), 1);
        assert_eq!(summary.node_count(), 1);
        assert_eq!(summary.nonlinear_boundary_count(), 1);
        assert_eq!(summary.reduction_annotation_count(), 1);
    }

    #[test]
    fn tdi23_1_root_validation_memoizes_reused_dag_nodes() {
        let mut ir = CategoricalAttentionIr::new();
        let object = ir.add_atomic_object("H", 1).expect("H");
        let mut node = ir.add_identity(object).expect("identity");
        for _ in 0..40 {
            node = ir.compose(node, node).expect("self composition");
        }

        let summary = validate_rooted_subgraph(&ir, node).expect("memoized DAG validation");
        assert_eq!(summary.node_count(), 41);
        assert_eq!(summary.nonlinear_boundary_count(), 0);
    }

    #[test]
    fn tdi23_1_root_validation_rejects_foreign_root() {
        let (left, left_root) = build_linear_fixture(1.0);
        let (right, _) = build_linear_fixture(1.0);
        assert_eq!(
            validate_rooted_subgraph(&right, left_root),
            Err(ProvenanceError::Ir(
                crate::experimental::tdi23_ir::IrError::ForeignNodeHandle(left_root)
            ))
        );
        drop(left);
    }
}
