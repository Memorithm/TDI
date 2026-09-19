//! Label-free relational observation graph for TDI-2.2.
//!
//! Concrete entity identifiers are provenance-local observations. They are not
//! template roles and must not be reused as privileged cross-episode features.

use super::tdi2_intuition::BooleanState;
use super::tdi2_template_induction::EpisodeId;

/// Concrete entity identifier local to one observation graph.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObservedEntityId(u32);

impl ObservedEntityId {
    /// Construct an observed entity identifier.
    #[must_use]
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    /// Canonical integer representation.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// Relation vocabulary identifier supplied by the observation schema.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObservedRelationId(u32);

impl ObservedRelationId {
    /// Construct a relation identifier.
    #[must_use]
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    /// Canonical integer representation.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// One concrete observed entity with unary Boolean observations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedEntity {
    id: ObservedEntityId,
    predicates: BooleanState,
}

impl ObservedEntity {
    /// Construct one observed entity.
    #[must_use]
    pub fn new(id: ObservedEntityId, predicates: BooleanState) -> Self {
        Self { id, predicates }
    }

    /// Concrete local identity.
    #[must_use]
    pub const fn id(&self) -> ObservedEntityId {
        self.id
    }

    /// Unary observations on this entity.
    #[must_use]
    pub const fn predicates(&self) -> &BooleanState {
        &self.predicates
    }
}

/// One directed typed observed relation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObservedRelation {
    left: ObservedEntityId,
    relation: ObservedRelationId,
    right: ObservedEntityId,
}

impl ObservedRelation {
    /// Construct one directed relation observation.
    #[must_use]
    pub const fn new(
        left: ObservedEntityId,
        relation: ObservedRelationId,
        right: ObservedEntityId,
    ) -> Self {
        Self {
            left,
            relation,
            right,
        }
    }

    /// Source entity.
    #[must_use]
    pub const fn left(self) -> ObservedEntityId {
        self.left
    }

    /// Relation kind.
    #[must_use]
    pub const fn relation(self) -> ObservedRelationId {
        self.relation
    }

    /// Destination entity.
    #[must_use]
    pub const fn right(self) -> ObservedEntityId {
        self.right
    }
}

/// Canonical concrete relational observation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservationGraph {
    episode: EpisodeId,
    entities: Vec<ObservedEntity>,
    relations: Vec<ObservedRelation>,
}

/// Observation-graph validation failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObservationGraphError {
    /// Entity ids must be unique within a graph.
    DuplicateEntity { entity: ObservedEntityId },
    /// Every relation endpoint must exist in the graph.
    UnknownRelationEndpoint { entity: ObservedEntityId },
    /// Identical relation triples are not repeated evidence.
    DuplicateRelation,
}

impl ObservationGraph {
    /// Validate and canonicalize one concrete relational observation.
    pub fn new(
        episode: EpisodeId,
        mut entities: Vec<ObservedEntity>,
        mut relations: Vec<ObservedRelation>,
    ) -> Result<Self, ObservationGraphError> {
        entities.sort_unstable_by_key(ObservedEntity::id);
        if let Some(entity) = entities
            .windows(2)
            .find_map(|pair| (pair[0].id() == pair[1].id()).then_some(pair[0].id()))
        {
            return Err(ObservationGraphError::DuplicateEntity { entity });
        }

        for relation in &relations {
            for endpoint in [relation.left(), relation.right()] {
                if entities
                    .binary_search_by_key(&endpoint, ObservedEntity::id)
                    .is_err()
                {
                    return Err(ObservationGraphError::UnknownRelationEndpoint {
                        entity: endpoint,
                    });
                }
            }
        }
        relations.sort_unstable();
        if relations.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(ObservationGraphError::DuplicateRelation);
        }
        Ok(Self {
            episode,
            entities,
            relations,
        })
    }

    /// Episode whose frame observations this graph describes.
    #[must_use]
    pub const fn episode(&self) -> EpisodeId {
        self.episode
    }

    /// Canonically ordered entities.
    #[must_use]
    pub fn entities(&self) -> &[ObservedEntity] {
        &self.entities
    }

    /// Canonically ordered directed relations.
    #[must_use]
    pub fn relations(&self) -> &[ObservedRelation] {
        &self.relations
    }
}

impl core::fmt::Display for ObservationGraphError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::DuplicateEntity { entity } => {
                write!(formatter, "observed entity {} appears twice", entity.raw())
            }
            Self::UnknownRelationEndpoint { entity } => write!(
                formatter,
                "relation endpoint {} is absent from observation graph",
                entity.raw()
            ),
            Self::DuplicateRelation => formatter.write_str("duplicate observed relation"),
        }
    }
}

impl std::error::Error for ObservationGraphError {}

#[cfg(test)]
mod tests {
    use super::{
        ObservationGraph, ObservationGraphError, ObservedEntity, ObservedEntityId,
        ObservedRelation, ObservedRelationId,
    };
    use crate::experimental::tdi2_intuition::{BooleanState, PredicateId};
    use crate::experimental::tdi2_template_induction::EpisodeId;

    fn entity(id: u32, predicate: u32) -> ObservedEntity {
        ObservedEntity::new(
            ObservedEntityId::new(id),
            BooleanState::new(vec![PredicateId::new(predicate)]),
        )
    }

    #[test]
    fn graph_canonicalizes_entities_and_relations_without_role_labels() {
        let graph = ObservationGraph::new(
            EpisodeId::new(7),
            vec![entity(20, 2), entity(10, 1)],
            vec![ObservedRelation::new(
                ObservedEntityId::new(10),
                ObservedRelationId::new(7),
                ObservedEntityId::new(20),
            )],
        )
        .expect("graph");
        assert_eq!(graph.episode(), EpisodeId::new(7));
        assert_eq!(graph.entities()[0].id(), ObservedEntityId::new(10));
        assert_eq!(graph.relations()[0].relation(), ObservedRelationId::new(7));
    }

    #[test]
    fn graph_rejects_unknown_relation_endpoint() {
        let result = ObservationGraph::new(
            EpisodeId::new(1),
            vec![entity(1, 10)],
            vec![ObservedRelation::new(
                ObservedEntityId::new(1),
                ObservedRelationId::new(3),
                ObservedEntityId::new(2),
            )],
        );
        assert_eq!(
            result,
            Err(ObservationGraphError::UnknownRelationEndpoint {
                entity: ObservedEntityId::new(2)
            })
        );
    }
}
