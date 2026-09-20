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
