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


use super::tdi2_observation_graph::ObservedRelationId;

/// Relation between two abstract role variables.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RoleRelationPattern {
    pub left: StructuralVariableId,
    pub relation: ObservedRelationId,
    pub right: StructuralVariableId,
}

/// Exact relation-preservation accounting for one candidate mapping.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RelationPreservation {
    pub expected: usize,
    pub matched: usize,
    pub missing: usize,
}

impl RelationPreservation {
    #[must_use]
    pub const fn is_exact(self) -> bool { self.expected > 0 && self.missing == 0 }
}

#[must_use]
pub fn relation_preservation(
    patterns: &[RoleRelationPattern],
    mapping: &RoleEntityMap,
    target: &ObservationGraph,
) -> RelationPreservation {
    let target_relations = target
        .relations()
        .iter()
        .map(|relation| (relation.left(), relation.relation(), relation.right()))
        .collect::<BTreeSet<_>>();
    let mut matched = 0usize;
    for pattern in patterns {
        if let (Some(left), Some(right)) = (
            mapping.resolve(pattern.left),
            mapping.resolve(pattern.right),
        ) {
            if target_relations.contains(&(left, pattern.relation, right)) {
                matched += 1;
            }
        }
    }
    RelationPreservation {
        expected: patterns.len(),
        matched,
        missing: patterns.len().saturating_sub(matched),
    }
}

#[cfg(test)]
mod preservation_tests {
    use super::*;
    use crate::experimental::tdi2_intuition::BooleanState;
    use crate::experimental::tdi2_observation_graph::{ObservedEntity, ObservedRelation};
    use crate::experimental::tdi2_template_induction::EpisodeId;

    #[test]
    fn exact_mapping_preserves_typed_directional_relation() {
        let roles = [StructuralVariableId::new(1), StructuralVariableId::new(2)];
        let relation = ObservedRelationId::new(7);
        let graph = ObservationGraph::new(
            EpisodeId::new(2),
            vec![
                ObservedEntity::new(ObservedEntityId::new(100), BooleanState::default()),
                ObservedEntity::new(ObservedEntityId::new(200), BooleanState::default()),
            ],
            vec![ObservedRelation::new(
                ObservedEntityId::new(100),
                relation,
                ObservedEntityId::new(200),
            )],
        )
        .expect("graph");
        let mapping = RoleEntityMap::new(
            &roles,
            &graph,
            vec![
                RoleEntityBinding::new(roles[0], ObservedEntityId::new(100)),
                RoleEntityBinding::new(roles[1], ObservedEntityId::new(200)),
            ],
        )
        .expect("mapping");
        let score = relation_preservation(
            &[RoleRelationPattern { left: roles[0], relation, right: roles[1] }],
            &mapping,
            &graph,
        );
        assert_eq!(score, RelationPreservation { expected: 1, matched: 1, missing: 0 });
        assert!(score.is_exact());
    }
}


/// Higher-order consistency accounting for shared-role relation systems.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MappingSystematicity {
    pub shared_role_pairs: usize,
    pub jointly_preserved_pairs: usize,
}

#[must_use]
pub fn mapping_systematicity(
    patterns: &[RoleRelationPattern],
    mapping: &RoleEntityMap,
    target: &ObservationGraph,
) -> MappingSystematicity {
    let target_relations = target
        .relations()
        .iter()
        .map(|relation| (relation.left(), relation.relation(), relation.right()))
        .collect::<BTreeSet<_>>();
    let preserved = |pattern: &RoleRelationPattern| {
        match (mapping.resolve(pattern.left), mapping.resolve(pattern.right)) {
            (Some(left), Some(right)) => {
                target_relations.contains(&(left, pattern.relation, right))
            }
            _ => false,
        }
    };
    let shares_role = |a: &RoleRelationPattern, b: &RoleRelationPattern| {
        [a.left, a.right]
            .into_iter()
            .any(|role| role == b.left || role == b.right)
    };

    let mut result = MappingSystematicity::default();
    for left in 0..patterns.len() {
        for right in (left + 1)..patterns.len() {
            if shares_role(&patterns[left], &patterns[right]) {
                result.shared_role_pairs += 1;
                if preserved(&patterns[left]) && preserved(&patterns[right]) {
                    result.jointly_preserved_pairs += 1;
                }
            }
        }
    }
    result
}

#[cfg(test)]
mod mapping_systematicity_tests {
    use super::*;
    use crate::experimental::tdi2_intuition::BooleanState;
    use crate::experimental::tdi2_observation_graph::{ObservedEntity, ObservedRelation};
    use crate::experimental::tdi2_template_induction::EpisodeId;

    #[test]
    fn shared_middle_role_is_jointly_preserved() {
        let r = [StructuralVariableId::new(0), StructuralVariableId::new(1), StructuralVariableId::new(2)];
        let rel1 = ObservedRelationId::new(1);
        let rel2 = ObservedRelationId::new(2);
        let graph = ObservationGraph::new(
            EpisodeId::new(3),
            vec![10,20,30].into_iter().map(|id| ObservedEntity::new(ObservedEntityId::new(id), BooleanState::default())).collect(),
            vec![
                ObservedRelation::new(ObservedEntityId::new(10), rel1, ObservedEntityId::new(20)),
                ObservedRelation::new(ObservedEntityId::new(20), rel2, ObservedEntityId::new(30)),
            ],
        ).expect("graph");
        let mapping = RoleEntityMap::new(
            &r,
            &graph,
            vec![
                RoleEntityBinding::new(r[0], ObservedEntityId::new(10)),
                RoleEntityBinding::new(r[1], ObservedEntityId::new(20)),
                RoleEntityBinding::new(r[2], ObservedEntityId::new(30)),
            ],
        ).expect("mapping");
        let patterns = [
            RoleRelationPattern { left:r[0], relation:rel1, right:r[1] },
            RoleRelationPattern { left:r[1], relation:rel2, right:r[2] },
        ];
        let diagnostic = mapping_systematicity(&patterns, &mapping, &graph);
        assert_eq!(diagnostic.shared_role_pairs, 1);
        assert_eq!(diagnostic.jointly_preserved_pairs, 1);
    }
}


/// Hard bounds for the exact mapping oracle.
pub const MAX_EXACT_MAPPING_ROLES: usize = 7;
pub const MAX_EXACT_MAPPING_ENTITIES: usize = 9;
pub const MAX_RETAINED_EXACT_MAPPINGS: usize = 128;

/// Exact bounded search evidence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExactMappingSearch {
    pub expected_relations: usize,
    pub best_matched: usize,
    pub total_best_solutions: usize,
    pub retained_solutions: Vec<RoleEntityMap>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MappingSearchError {
    EmptyStructure,
    TooManyRoles,
    TooManyEntities,
    NotEnoughEntities,
    MappingConstraint(MappingConstraintError),
}

impl From<MappingConstraintError> for MappingSearchError {
    fn from(error: MappingConstraintError) -> Self {
        Self::MappingConstraint(error)
    }
}

/// Enumerate all injective mappings within a deliberately small exact domain.
pub fn exact_mapping_search(
    roles: &[StructuralVariableId],
    patterns: &[RoleRelationPattern],
    target: &ObservationGraph,
) -> Result<ExactMappingSearch, MappingSearchError> {
    if patterns.is_empty() || roles.is_empty() {
        return Err(MappingSearchError::EmptyStructure);
    }
    let mut roles = roles.to_vec();
    roles.sort_unstable();
    roles.dedup();
    if roles.len() > MAX_EXACT_MAPPING_ROLES {
        return Err(MappingSearchError::TooManyRoles);
    }
    if target.entities().len() > MAX_EXACT_MAPPING_ENTITIES {
        return Err(MappingSearchError::TooManyEntities);
    }
    if roles.len() > target.entities().len() {
        return Err(MappingSearchError::NotEnoughEntities);
    }

    let entities = target
        .entities()
        .iter()
        .map(|entity| entity.id())
        .collect::<Vec<_>>();
    let mut best_matched = 0usize;
    let mut total_best_solutions = 0usize;
    let mut retained_solutions = Vec::new();
    let mut current = Vec::<RoleEntityBinding>::with_capacity(roles.len());
    let mut used = BTreeSet::<ObservedEntityId>::new();

    struct SearchContext<'a> {
        roles: &'a [StructuralVariableId],
        patterns: &'a [RoleRelationPattern],
        target: &'a ObservationGraph,
        entities: &'a [ObservedEntityId],
        best_matched: &'a mut usize,
        total_best_solutions: &'a mut usize,
        retained_solutions: &'a mut Vec<RoleEntityMap>,
    }

    fn visit(
        index: usize,
        current: &mut Vec<RoleEntityBinding>,
        used: &mut BTreeSet<ObservedEntityId>,
        context: &mut SearchContext<'_>,
    ) -> Result<(), MappingSearchError> {
        if index == context.roles.len() {
            let mapping = RoleEntityMap::new(context.roles, context.target, current.clone())?;
            let score = relation_preservation(context.patterns, &mapping, context.target);
            if score.matched > *context.best_matched {
                *context.best_matched = score.matched;
                *context.total_best_solutions = 1;
                context.retained_solutions.clear();
                context.retained_solutions.push(mapping);
            } else if score.matched == *context.best_matched {
                *context.total_best_solutions = context.total_best_solutions.saturating_add(1);
                if context.retained_solutions.len() < MAX_RETAINED_EXACT_MAPPINGS {
                    context.retained_solutions.push(mapping);
                }
            }
            return Ok(());
        }

        for &entity in context.entities {
            if used.insert(entity) {
                current.push(RoleEntityBinding::new(context.roles[index], entity));
                visit(index + 1, current, used, context)?;
                current.pop();
                used.remove(&entity);
            }
        }
        Ok(())
    }

    let mut context = SearchContext {
        roles: &roles,
        patterns,
        target,
        entities: &entities,
        best_matched: &mut best_matched,
        total_best_solutions: &mut total_best_solutions,
        retained_solutions: &mut retained_solutions,
    };
    visit(0, &mut current, &mut used, &mut context)?;

    Ok(ExactMappingSearch {
        expected_relations: patterns.len(),
        best_matched,
        total_best_solutions,
        retained_solutions,
    })
}

#[cfg(test)]
mod exact_mapping_tests {
    use super::*;
    use crate::experimental::tdi2_intuition::BooleanState;
    use crate::experimental::tdi2_observation_graph::{ObservedEntity, ObservedRelation};
    use crate::experimental::tdi2_template_induction::EpisodeId;

    #[test]
    fn typed_relation_triangle_has_one_exact_mapping() {
        let roles = [StructuralVariableId::new(0), StructuralVariableId::new(1), StructuralVariableId::new(2)];
        let relation = [ObservedRelationId::new(1), ObservedRelationId::new(2), ObservedRelationId::new(3)];
        let graph = ObservationGraph::new(
            EpisodeId::new(4),
            vec![10,20,30].into_iter().map(|id| ObservedEntity::new(ObservedEntityId::new(id), BooleanState::default())).collect(),
            vec![
                ObservedRelation::new(ObservedEntityId::new(20), relation[0], ObservedEntityId::new(30)),
                ObservedRelation::new(ObservedEntityId::new(30), relation[1], ObservedEntityId::new(10)),
                ObservedRelation::new(ObservedEntityId::new(20), relation[2], ObservedEntityId::new(10)),
            ],
        ).expect("graph");
        let patterns = [
            RoleRelationPattern { left:roles[0], relation:relation[0], right:roles[1] },
            RoleRelationPattern { left:roles[1], relation:relation[1], right:roles[2] },
            RoleRelationPattern { left:roles[0], relation:relation[2], right:roles[2] },
        ];
        let result = exact_mapping_search(&roles, &patterns, &graph).expect("search");
        assert_eq!(result.best_matched, 3);
        assert_eq!(result.total_best_solutions, 1);
        assert_eq!(result.retained_solutions[0].resolve(roles[0]), Some(ObservedEntityId::new(20)));
    }
}


/// Deterministic bounded structural approximation using role/entity degree signatures.
pub fn approximate_mapping(
    roles: &[StructuralVariableId],
    patterns: &[RoleRelationPattern],
    target: &ObservationGraph,
) -> Result<RoleEntityMap, MappingSearchError> {
    if roles.is_empty() || patterns.is_empty() {
        return Err(MappingSearchError::EmptyStructure);
    }
    if roles.len() > target.entities().len() {
        return Err(MappingSearchError::NotEnoughEntities);
    }
    let mut role_degree = BTreeMap::<StructuralVariableId, usize>::new();
    for pattern in patterns {
        *role_degree.entry(pattern.left).or_default() += 1;
        *role_degree.entry(pattern.right).or_default() += 1;
    }
    let mut entity_degree = BTreeMap::<ObservedEntityId, usize>::new();
    for relation in target.relations() {
        *entity_degree.entry(relation.left()).or_default() += 1;
        *entity_degree.entry(relation.right()).or_default() += 1;
    }

    let mut ordered_roles = roles.to_vec();
    ordered_roles.sort_unstable_by_key(|role| {
        (core::cmp::Reverse(*role_degree.get(role).unwrap_or(&0)), role.raw())
    });
    let mut available = target.entities().iter().map(|entity| entity.id()).collect::<Vec<_>>();
    let mut bindings = Vec::with_capacity(ordered_roles.len());

    for role in ordered_roles {
        let role_d = *role_degree.get(&role).unwrap_or(&0);
        let (index, entity) = available
            .iter()
            .enumerate()
            .min_by_key(|(_, entity)| {
                let entity_d = *entity_degree.get(entity).unwrap_or(&0);
                (role_d.abs_diff(entity_d), entity.raw())
            })
            .map(|(index, entity)| (index, *entity))
            .ok_or(MappingSearchError::NotEnoughEntities)?;
        available.remove(index);
        bindings.push(RoleEntityBinding::new(role, entity));
    }
    RoleEntityMap::new(roles, target, bindings).map_err(Into::into)
}

#[cfg(test)]
mod approximate_mapping_tests {
    use super::*;
    use crate::experimental::tdi2_intuition::BooleanState;
    use crate::experimental::tdi2_observation_graph::{ObservedEntity, ObservedRelation};
    use crate::experimental::tdi2_template_induction::EpisodeId;

    #[test]
    fn structural_degree_mapper_is_deterministic() {
        let roles = [StructuralVariableId::new(0), StructuralVariableId::new(1)];
        let relation = ObservedRelationId::new(8);
        let graph = ObservationGraph::new(
            EpisodeId::new(5),
            vec![10,20].into_iter().map(|id| ObservedEntity::new(ObservedEntityId::new(id), BooleanState::default())).collect(),
            vec![ObservedRelation::new(ObservedEntityId::new(10), relation, ObservedEntityId::new(20))],
        ).expect("graph");
        let patterns = [RoleRelationPattern { left:roles[0], relation, right:roles[1] }];
        let first = approximate_mapping(&roles, &patterns, &graph).expect("first");
        let second = approximate_mapping(&roles, &patterns, &graph).expect("second");
        assert_eq!(first, second);
    }
}


/// Decision surface for mapping search; ambiguity is not silently tie-broken.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MappingDecision {
    Selected(RoleEntityMap),
    Ambiguous { exact_solutions: usize },
    InsufficientStructure { matched: usize, expected: usize },
}

#[must_use]
pub fn decide_exact_mapping(search: &ExactMappingSearch) -> MappingDecision {
    if search.best_matched < search.expected_relations {
        return MappingDecision::InsufficientStructure {
            matched: search.best_matched,
            expected: search.expected_relations,
        };
    }
    if search.total_best_solutions != 1 {
        return MappingDecision::Ambiguous {
            exact_solutions: search.total_best_solutions,
        };
    }
    match search.retained_solutions.first() {
        Some(mapping) => MappingDecision::Selected(mapping.clone()),
        None => MappingDecision::InsufficientStructure {
            matched: search.best_matched,
            expected: search.expected_relations,
        },
    }
}

#[cfg(test)]
mod mapping_decision_tests {
    use super::*;

    #[test]
    fn exact_ties_are_ambiguous_not_arbitrarily_selected() {
        let search = ExactMappingSearch {
            expected_relations: 2,
            best_matched: 2,
            total_best_solutions: 3,
            retained_solutions: Vec::new(),
        };
        assert_eq!(
            decide_exact_mapping(&search),
            MappingDecision::Ambiguous { exact_solutions: 3 }
        );
    }
}


/// Surface-identity control: role raw ids are treated as target entity raw ids.
#[must_use]
pub fn surface_identity_control(
    roles: &[StructuralVariableId],
    target: &ObservationGraph,
) -> Option<RoleEntityMap> {
    if roles.is_empty() {
        return None;
    }
    let bindings = roles
        .iter()
        .copied()
        .map(|role| RoleEntityBinding::new(role, ObservedEntityId::new(role.raw())))
        .collect::<Vec<_>>();
    RoleEntityMap::new(roles, target, bindings).ok()
}

/// Deterministic arbitrary-permutation control independent of relation structure.
#[must_use]
pub fn rotated_entity_control(
    roles: &[StructuralVariableId],
    target: &ObservationGraph,
    offset: usize,
) -> Option<RoleEntityMap> {
    if roles.is_empty() || roles.len() > target.entities().len() {
        return None;
    }
    let mut roles = roles.to_vec();
    roles.sort_unstable();
    let entities = target.entities().iter().map(|entity| entity.id()).collect::<Vec<_>>();
    let shift = offset % entities.len();
    let bindings = roles
        .iter()
        .enumerate()
        .map(|(index, role)| {
            RoleEntityBinding::new(*role, entities[(index + shift) % entities.len()])
        })
        .collect::<Vec<_>>();
    RoleEntityMap::new(&roles, target, bindings).ok()
}

#[cfg(test)]
mod mapping_control_tests {
    use super::*;
    use crate::experimental::tdi2_intuition::BooleanState;
    use crate::experimental::tdi2_observation_graph::ObservedEntity;
    use crate::experimental::tdi2_template_induction::EpisodeId;

    #[test]
    fn controls_are_explicitly_surface_or_arbitrary() {
        let graph = ObservationGraph::new(
            EpisodeId::new(6),
            vec![0,1,10].into_iter().map(|id| ObservedEntity::new(ObservedEntityId::new(id), BooleanState::default())).collect(),
            Vec::new(),
        ).expect("graph");
        let roles = [StructuralVariableId::new(0), StructuralVariableId::new(1)];
        assert!(surface_identity_control(&roles, &graph).is_some());
        let rotated = rotated_entity_control(&roles, &graph, 1).expect("rotated");
        assert_ne!(rotated.resolve(roles[0]), Some(ObservedEntityId::new(0)));
    }
}
