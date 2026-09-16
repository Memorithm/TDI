//! Relational structure carried by TDI-2.1 experiential templates.

use super::tdi2_intuition::{RoleId, Template};

/// Stable identifier for one relation kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RelationId(u32);

impl RelationId {
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

/// One directed typed relation between two abstract roles.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RoleRelation {
    left: RoleId,
    relation: RelationId,
    right: RoleId,
}

impl RoleRelation {
    /// Construct a role relation.
    #[must_use]
    pub const fn new(left: RoleId, relation: RelationId, right: RoleId) -> Self {
        Self {
            left,
            relation,
            right,
        }
    }

    /// Source role.
    #[must_use]
    pub const fn left(self) -> RoleId {
        self.left
    }

    /// Relation kind.
    #[must_use]
    pub const fn relation(self) -> RelationId {
        self.relation
    }

    /// Destination role.
    #[must_use]
    pub const fn right(self) -> RoleId {
        self.right
    }
}

/// A Boolean template enriched with role-to-role structural relations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationalTemplate {
    base: Template,
    relations: Vec<RoleRelation>,
}

/// Validation errors for relational template structure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelationError {
    /// A relation refers to a role not declared by the base template.
    UnknownRole { role: RoleId },
    /// The same directed typed relation appears more than once.
    DuplicateRelation { relation: RoleRelation },
}

impl core::fmt::Display for RelationError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnknownRole { role } => {
                write!(formatter, "relation references undeclared role {}", role.raw())
            }
            Self::DuplicateRelation { relation } => write!(
                formatter,
                "duplicate relation {}:{}:{}",
                relation.left().raw(),
                relation.relation().raw(),
                relation.right().raw()
            ),
        }
    }
}

impl std::error::Error for RelationError {}

impl RelationalTemplate {
    /// Validate relations against the role declarations of a base template.
    pub fn new(base: Template, mut relations: Vec<RoleRelation>) -> Result<Self, RelationError> {
        for relation in &relations {
            for role in [relation.left(), relation.right()] {
                if base.roles().binary_search(&role).is_err() {
                    return Err(RelationError::UnknownRole { role });
                }
            }
        }

        relations.sort_unstable();
        if let Some(relation) = relations
            .windows(2)
            .find_map(|pair| (pair[0] == pair[1]).then_some(pair[0]))
        {
            return Err(RelationError::DuplicateRelation { relation });
        }

        Ok(Self { base, relations })
    }

    /// Underlying Boolean template.
    #[must_use]
    pub const fn base(&self) -> &Template {
        &self.base
    }

    /// Canonically ordered role relations.
    #[must_use]
    pub fn relations(&self) -> &[RoleRelation] {
        &self.relations
    }
}

#[cfg(test)]
mod tests {
    use super::{RelationError, RelationId, RelationalTemplate, RoleRelation};
    use crate::experimental::tdi2_intuition::{PredicateId, RoleId, Template, TemplateId};

    fn base_template() -> Template {
        Template::new(
            TemplateId::new(4),
            vec![PredicateId::new(1)],
            Vec::new(),
            vec![RoleId::new(10), RoleId::new(20)],
        )
        .expect("valid template")
    }

    #[test]
    fn relation_requires_declared_roles() {
        let error = RelationalTemplate::new(
            base_template(),
            vec![RoleRelation::new(
                RoleId::new(10),
                RelationId::new(5),
                RoleId::new(99),
            )],
        )
        .expect_err("unknown role must be rejected");
        assert_eq!(error, RelationError::UnknownRole { role: RoleId::new(99) });
    }

    #[test]
    fn relation_order_is_canonical() {
        let first = RoleRelation::new(RoleId::new(10), RelationId::new(2), RoleId::new(20));
        let second = RoleRelation::new(RoleId::new(20), RelationId::new(1), RoleId::new(10));
        let template = RelationalTemplate::new(base_template(), vec![second, first])
            .expect("valid relational template");
        assert_eq!(template.relations(), &[first, second]);
    }
}
