//! Role-preserving structural transfer for TDI-2.1 experiential templates.

use super::tdi2_intuition::RoleId;
use super::tdi2_intuition_relations::{RelationId, RelationalTemplate};

/// Identifier for one concrete entity in a novel situation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityId(u32);

impl EntityId {
    /// Construct an entity identifier.
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

/// Mapping of one abstract template role onto one concrete entity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RoleBinding {
    role: RoleId,
    entity: EntityId,
}

impl RoleBinding {
    /// Construct one role-to-entity binding.
    #[must_use]
    pub const fn new(role: RoleId, entity: EntityId) -> Self {
        Self { role, entity }
    }

    /// Abstract role.
    #[must_use]
    pub const fn role(self) -> RoleId {
        self.role
    }

    /// Concrete entity.
    #[must_use]
    pub const fn entity(self) -> EntityId {
        self.entity
    }
}

/// Complete injective role mapping for one template application.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoleMap {
    bindings: Vec<RoleBinding>,
}

/// Validation errors for structural transfer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransferError {
    /// A binding names a role absent from the template.
    UnknownRole { role: RoleId },
    /// A role is bound more than once.
    DuplicateRole { role: RoleId },
    /// Two abstract roles collapse onto the same concrete entity.
    DuplicateEntity { entity: EntityId },
    /// A declared role has no concrete binding.
    MissingRole { role: RoleId },
}

impl core::fmt::Display for TransferError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnknownRole { role } => write!(formatter, "unknown transfer role {}", role.raw()),
            Self::DuplicateRole { role } => {
                write!(formatter, "transfer role {} is bound twice", role.raw())
            }
            Self::DuplicateEntity { entity } => write!(
                formatter,
                "concrete entity {} receives multiple roles",
                entity.raw()
            ),
            Self::MissingRole { role } => {
                write!(formatter, "transfer role {} has no binding", role.raw())
            }
        }
    }
}

impl std::error::Error for TransferError {}

impl RoleMap {
    /// Validate a complete injective mapping for the supplied template.
    pub fn for_template(
        template: &RelationalTemplate,
        mut bindings: Vec<RoleBinding>,
    ) -> Result<Self, TransferError> {
        for binding in &bindings {
            if template
                .base()
                .roles()
                .binary_search(&binding.role())
                .is_err()
            {
                return Err(TransferError::UnknownRole {
                    role: binding.role(),
                });
            }
        }

        bindings.sort_unstable_by_key(|binding| binding.role());
        if let Some(role) = bindings
            .windows(2)
            .find_map(|pair| (pair[0].role() == pair[1].role()).then_some(pair[0].role()))
        {
            return Err(TransferError::DuplicateRole { role });
        }

        let mut entities = bindings
            .iter()
            .map(|binding| binding.entity())
            .collect::<Vec<_>>();
        entities.sort_unstable();
        if let Some(entity) = entities
            .windows(2)
            .find_map(|pair| (pair[0] == pair[1]).then_some(pair[0]))
        {
            return Err(TransferError::DuplicateEntity { entity });
        }

        for role in template.base().roles() {
            if bindings
                .binary_search_by_key(role, |binding| binding.role())
                .is_err()
            {
                return Err(TransferError::MissingRole { role: *role });
            }
        }

        Ok(Self { bindings })
    }

    /// Resolve one role to its concrete entity.
    #[must_use]
    pub fn resolve(&self, role: RoleId) -> Option<EntityId> {
        self.bindings
            .binary_search_by_key(&role, |binding| binding.role())
            .ok()
            .map(|index| self.bindings[index].entity())
    }

    /// Canonical role bindings.
    #[must_use]
    pub fn bindings(&self) -> &[RoleBinding] {
        &self.bindings
    }
}

/// One transferred concrete relation in the novel situation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EntityRelation {
    left: EntityId,
    relation: RelationId,
    right: EntityId,
}

impl EntityRelation {
    /// Source entity.
    #[must_use]
    pub const fn left(self) -> EntityId {
        self.left
    }

    /// Preserved relation kind.
    #[must_use]
    pub const fn relation(self) -> RelationId {
        self.relation
    }

    /// Destination entity.
    #[must_use]
    pub const fn right(self) -> EntityId {
        self.right
    }
}

/// Apply a validated role map while preserving all relation identities.
#[must_use]
pub fn transfer_relations(
    template: &RelationalTemplate,
    role_map: &RoleMap,
) -> Vec<EntityRelation> {
    template
        .relations()
        .iter()
        .map(|relation| EntityRelation {
            left: role_map
                .resolve(relation.left())
                .expect("RoleMap was validated against this template"),
            relation: relation.relation(),
            right: role_map
                .resolve(relation.right())
                .expect("RoleMap was validated against this template"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{EntityId, RoleBinding, RoleMap, TransferError, transfer_relations};
    use crate::experimental::tdi2_intuition::{PredicateId, RoleId, Template, TemplateId};
    use crate::experimental::tdi2_intuition_relations::{
        RelationId, RelationalTemplate, RoleRelation,
    };

    fn relational_template() -> RelationalTemplate {
        let base = Template::new(
            TemplateId::new(7),
            vec![PredicateId::new(1)],
            Vec::new(),
            vec![RoleId::new(1), RoleId::new(2)],
        )
        .expect("valid template");
        RelationalTemplate::new(
            base,
            vec![RoleRelation::new(
                RoleId::new(1),
                RelationId::new(42),
                RoleId::new(2),
            )],
        )
        .expect("valid relational template")
    }

    #[test]
    fn transfer_preserves_relation_kind() {
        let template = relational_template();
        let map = RoleMap::for_template(
            &template,
            vec![
                RoleBinding::new(RoleId::new(1), EntityId::new(100)),
                RoleBinding::new(RoleId::new(2), EntityId::new(200)),
            ],
        )
        .expect("valid mapping");
        let transferred = transfer_relations(&template, &map);
        assert_eq!(transferred.len(), 1);
        assert_eq!(transferred[0].left(), EntityId::new(100));
        assert_eq!(transferred[0].relation(), RelationId::new(42));
        assert_eq!(transferred[0].right(), EntityId::new(200));
    }

    #[test]
    fn transfer_is_injective() {
        let template = relational_template();
        let error = RoleMap::for_template(
            &template,
            vec![
                RoleBinding::new(RoleId::new(1), EntityId::new(100)),
                RoleBinding::new(RoleId::new(2), EntityId::new(100)),
            ],
        )
        .expect_err("role collapse must be rejected");
        assert_eq!(
            error,
            TransferError::DuplicateEntity {
                entity: EntityId::new(100)
            }
        );
    }
}
