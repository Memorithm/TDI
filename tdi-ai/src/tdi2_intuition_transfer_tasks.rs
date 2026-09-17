//! Deterministic novel-identity transfer fixtures for TDI-2.1.

use super::tdi2_intuition::RoleId;
use super::tdi2_intuition_transfer::{EntityId, RoleBinding};

/// Build an injective role binding whose concrete identities depend only on the case id.
///
/// Roles are sorted before assignment so construction order cannot alter the fixture.
#[must_use]
pub fn novel_identity_bindings(roles: &[RoleId], case_id: u32) -> Vec<RoleBinding> {
    let mut canonical_roles = roles.to_vec();
    canonical_roles.sort_unstable();
    canonical_roles.dedup();
    let count = canonical_roles.len();
    if count == 0 {
        return Vec::new();
    }
    let rotation = case_id as usize % count;
    canonical_roles
        .into_iter()
        .enumerate()
        .map(|(index, role)| {
            let slot = (index + rotation) % count;
            let entity = 100_000u32
                .wrapping_add(case_id.wrapping_mul(1_000))
                .wrapping_add(slot as u32);
            RoleBinding::new(role, EntityId::new(entity))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::novel_identity_bindings;
    use crate::experimental::tdi2_intuition::RoleId;

    #[test]
    fn bindings_are_injective_and_case_specific() {
        let roles = [RoleId::new(3), RoleId::new(1), RoleId::new(2)];
        let first = novel_identity_bindings(&roles, 7);
        let second = novel_identity_bindings(&roles, 8);
        assert_eq!(first.len(), 3);
        assert_ne!(first, second);
        let mut entities = first
            .iter()
            .map(|binding| binding.entity())
            .collect::<Vec<_>>();
        entities.sort_unstable();
        entities.dedup();
        assert_eq!(entities.len(), 3);
    }
}
