//! Automatic analogical role/entity mapping for TDI-2.2.
//!
//! Target entity identifiers are episode-local observations. Mapping candidates
//! are generated from structure only; no expected mapping, target label, latency,
//! or PrimaryHoldout information is accepted by this module.

use super::tdi2_observation_graph::{ObservationGraph, ObservedEntityId};
use super::tdi2_structural_terms::StructuralVariableId;

/// One possible correspondence between an induced role variable and a target entity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RoleEntityCandidate {
    role: StructuralVariableId,
    entity: ObservedEntityId,
}

impl RoleEntityCandidate {
    #[must_use]
    pub const fn role(self) -> StructuralVariableId {
        self.role
    }

    #[must_use]
    pub const fn entity(self) -> ObservedEntityId {
        self.entity
    }
}

/// Generate the full bounded role/entity candidate relation.
///
/// Duplicate role ids are removed. Candidate order is deterministic by role then
/// canonical target entity order. This stage deliberately does not rank candidates.
#[must_use]
pub fn correspondence_candidates(
    roles: &[StructuralVariableId],
    target: &ObservationGraph,
) -> Vec<RoleEntityCandidate> {
    let mut roles = roles.to_vec();
    roles.sort_unstable();
    roles.dedup();
    let mut candidates = Vec::with_capacity(roles.len().saturating_mul(target.entities().len()));
    for role in roles {
        for entity in target.entities() {
            candidates.push(RoleEntityCandidate {
                role,
                entity: entity.id(),
            });
        }
    }
    candidates
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::experimental::tdi2_intuition::BooleanState;
    use crate::experimental::tdi2_observation_graph::{ObservationGraph, ObservedEntity};
    use crate::experimental::tdi2_template_induction::EpisodeId;

    #[test]
    fn candidate_generation_is_complete_deterministic_and_label_free() {
        let graph = ObservationGraph::new(
            EpisodeId::new(9),
            vec![
                ObservedEntity::new(ObservedEntityId::new(20), BooleanState::default()),
                ObservedEntity::new(ObservedEntityId::new(10), BooleanState::default()),
            ],
            Vec::new(),
        )
        .expect("graph");
        let candidates = correspondence_candidates(
            &[StructuralVariableId::new(2), StructuralVariableId::new(1)],
            &graph,
        );
        assert_eq!(candidates.len(), 4);
        assert_eq!(candidates[0].role(), StructuralVariableId::new(1));
        assert_eq!(candidates[0].entity(), ObservedEntityId::new(10));
        assert_eq!(candidates[3].role(), StructuralVariableId::new(2));
        assert_eq!(candidates[3].entity(), ObservedEntityId::new(20));
    }
}


use std::collections::BTreeSet;

/// One selected role/entity binding.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RoleEntityBinding {
    role: StructuralVariableId,
    entity: ObservedEntityId,
}

impl RoleEntityBinding {
    #[must_use]
    pub const fn new(role: StructuralVariableId, entity: ObservedEntityId) -> Self {
        Self { role, entity }
    }
    #[must_use]
    pub const fn role(self) -> StructuralVariableId { self.role }
    #[must_use]
    pub const fn entity(self) -> ObservedEntityId { self.entity }
}

/// Complete injective structural mapping.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoleEntityMap {
    bindings: Vec<RoleEntityBinding>,
}

impl RoleEntityMap {
    pub fn new(
        roles: &[StructuralVariableId],
        target: &ObservationGraph,
        mut bindings: Vec<RoleEntityBinding>,
    ) -> Result<Self, MappingConstraintError> {
        let role_set = roles.iter().copied().collect::<BTreeSet<_>>();
        if role_set.len() != roles.len() {
            return Err(MappingConstraintError::DuplicateDeclaredRole);
        }
        let entity_set = target
            .entities()
            .iter()
            .map(|entity| entity.id())
            .collect::<BTreeSet<_>>();
        for binding in &bindings {
            if !role_set.contains(&binding.role()) {
                return Err(MappingConstraintError::UnknownRole(binding.role()));
            }
            if !entity_set.contains(&binding.entity()) {
                return Err(MappingConstraintError::UnknownEntity(binding.entity()));
            }
        }
        bindings.sort_unstable();
        if let Some(role) = bindings
            .windows(2)
            .find_map(|pair| (pair[0].role() == pair[1].role()).then_some(pair[0].role()))
        {
            return Err(MappingConstraintError::DuplicateRole(role));
        }
        let mut mapped_entities = bindings.iter().map(|binding| binding.entity()).collect::<Vec<_>>();
        mapped_entities.sort_unstable();
        if let Some(entity) = mapped_entities
            .windows(2)
            .find_map(|pair| (pair[0] == pair[1]).then_some(pair[0]))
        {
            return Err(MappingConstraintError::DuplicateEntity(entity));
        }
        if bindings.len() != roles.len() {
            let bound_roles = bindings.iter().map(|binding| binding.role()).collect::<BTreeSet<_>>();
            if let Some(role) = roles.iter().copied().find(|role| !bound_roles.contains(role)) {
                return Err(MappingConstraintError::MissingRole(role));
            }
        }
        Ok(Self { bindings })
    }

    #[must_use]
    pub fn bindings(&self) -> &[RoleEntityBinding] { &self.bindings }

    #[must_use]
    pub fn resolve(&self, role: StructuralVariableId) -> Option<ObservedEntityId> {
        self.bindings
            .binary_search_by_key(&role, |binding| binding.role())
            .ok()
            .map(|index| self.bindings[index].entity())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MappingConstraintError {
    DuplicateDeclaredRole,
    UnknownRole(StructuralVariableId),
    UnknownEntity(ObservedEntityId),
    DuplicateRole(StructuralVariableId),
    DuplicateEntity(ObservedEntityId),
    MissingRole(StructuralVariableId),
}

#[cfg(test)]
mod mapping_constraint_tests {
    use super::*;
    use crate::experimental::tdi2_intuition::BooleanState;
    use crate::experimental::tdi2_observation_graph::ObservedEntity;
    use crate::experimental::tdi2_template_induction::EpisodeId;

    fn graph() -> ObservationGraph {
        ObservationGraph::new(
            EpisodeId::new(1),
            vec![
                ObservedEntity::new(ObservedEntityId::new(10), BooleanState::default()),
                ObservedEntity::new(ObservedEntityId::new(20), BooleanState::default()),
            ],
            Vec::new(),
        )
        .expect("graph")
    }

    #[test]
    fn role_map_is_complete_and_injective() {
        let roles = [StructuralVariableId::new(1), StructuralVariableId::new(2)];
        let map = RoleEntityMap::new(
            &roles,
            &graph(),
            vec![
                RoleEntityBinding::new(roles[0], ObservedEntityId::new(10)),
                RoleEntityBinding::new(roles[1], ObservedEntityId::new(20)),
            ],
        )
        .expect("map");
        assert_eq!(map.resolve(roles[1]), Some(ObservedEntityId::new(20)));
        assert!(matches!(
            RoleEntityMap::new(
                &roles,
                &graph(),
                vec![
                    RoleEntityBinding::new(roles[0], ObservedEntityId::new(10)),
                    RoleEntityBinding::new(roles[1], ObservedEntityId::new(10)),
                ],
            ),
            Err(MappingConstraintError::DuplicateEntity(_))
        ));
    }
}
