//! Surface-disjoint cross-domain transfer fixtures for TDI-2.2.
//!
//! Entity ids, relation ids and Boolean surface predicates differ across domains.
//! Only constructor-level relational topology is shared. Expected evaluation
//! outcomes remain outside induction inputs.

use super::tdi2_intuition::{BooleanState, PredicateId};
use super::tdi2_observation_graph::{ObservedEntityId, ObservedRelationId};
use super::tdi2_structural_terms::{StructuralSymbol, StructuralTerm};
use super::tdi2_template_induction::EpisodeId;
use super::tdi2_template_learning::{PositiveTemplateCandidate, induce_positive_template, match_induced_template};
use super::tdi2_incremental_anti_unification::IncrementalAntiUnificationError;

/// Constructor identities shared across domains. They encode syntax, not domain vocabulary.
const GRAPH_CONSTRUCTOR: u32 = 70_000;
const EDGE_CONSTRUCTOR: u32 = 70_001;

/// One complete structural observation in a domain-specific vocabulary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrossDomainObservation {
    pub domain: u32,
    pub case_id: u32,
    pub structure: StructuralTerm,
    pub surface_predicates: BooleanState,
}

/// Build one topology-equivalent observation with domain-specific entity/relation vocabulary.
pub fn cross_domain_observation(
    domain: u32,
    case_id: u32,
) -> Result<CrossDomainObservation, CrossDomainError> {
    let domain_rel = domain
        .checked_mul(1_000)
        .ok_or(CrossDomainError::IdentifierOverflow)?;
    let entity_base = domain
        .checked_mul(1_000_000)
        .and_then(|base| base.checked_add(case_id.checked_mul(10)?))
        .ok_or(CrossDomainError::IdentifierOverflow)?;
    let episode = EpisodeId::new((u64::from(domain) << 32) | u64::from(case_id));
    let entities = [
        StructuralTerm::atom(StructuralSymbol::entity(
            episode,
            ObservedEntityId::new(entity_base + 1),
        )),
        StructuralTerm::atom(StructuralSymbol::entity(
            episode,
            ObservedEntityId::new(entity_base + 2),
        )),
        StructuralTerm::atom(StructuralSymbol::entity(
            episode,
            ObservedEntityId::new(entity_base + 3),
        )),
    ];
    let relations = [
        StructuralTerm::atom(StructuralSymbol::relation(ObservedRelationId::new(domain_rel + 1))),
        StructuralTerm::atom(StructuralSymbol::relation(ObservedRelationId::new(domain_rel + 2))),
        StructuralTerm::atom(StructuralSymbol::relation(ObservedRelationId::new(domain_rel + 3))),
    ];
    let edge = |relation: StructuralTerm, left: StructuralTerm, right: StructuralTerm| {
        StructuralTerm::application(
            StructuralSymbol::constructor(EDGE_CONSTRUCTOR),
            vec![relation, left, right],
        )
    };
    let edges = vec![
        edge(relations[0].clone(), entities[0].clone(), entities[1].clone())?,
        edge(relations[1].clone(), entities[1].clone(), entities[2].clone())?,
        edge(relations[2].clone(), entities[0].clone(), entities[2].clone())?,
    ];
    let structure = StructuralTerm::application(
        StructuralSymbol::constructor(GRAPH_CONSTRUCTOR),
        edges,
    )?;

    let surface_base = domain
        .checked_mul(100_000)
        .ok_or(CrossDomainError::IdentifierOverflow)?;
    let surface_predicates = BooleanState::new(vec![
        PredicateId::new(surface_base + 1),
        PredicateId::new(surface_base + 2),
        PredicateId::new(surface_base + 3),
    ]);
    Ok(CrossDomainObservation {
        domain,
        case_id,
        structure,
        surface_predicates,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrossDomainError {
    IdentifierOverflow,
    Structural(super::tdi2_structural_terms::StructuralTermError),
    Induction(IncrementalAntiUnificationError),
}

impl From<super::tdi2_structural_terms::StructuralTermError> for CrossDomainError {
    fn from(error: super::tdi2_structural_terms::StructuralTermError) -> Self {
        Self::Structural(error)
    }
}
impl From<IncrementalAntiUnificationError> for CrossDomainError {
    fn from(error: IncrementalAntiUnificationError) -> Self {
        Self::Induction(error)
    }
}

/// Induce from two disjoint surface domains; the third domain is never an induction input.
pub fn induce_cross_domain_template(
    first_domain: u32,
    second_domain: u32,
    case_id: u32,
) -> Result<PositiveTemplateCandidate, CrossDomainError> {
    let first = cross_domain_observation(first_domain, case_id)?;
    let second = cross_domain_observation(second_domain, case_id)?;
    Ok(induce_positive_template(&[first.structure, second.structure])?)
}

/// Evaluate structural transfer to another disjoint domain.
pub fn transfers_to_domain(
    candidate: &PositiveTemplateCandidate,
    target_domain: u32,
    case_id: u32,
) -> Result<bool, CrossDomainError> {
    let target = cross_domain_observation(target_domain, case_id)?;
    Ok(match_induced_template(candidate.generalization(), &target.structure).is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relation_entity_and_boolean_vocabularies_are_disjoint_across_domains() {
        let a = cross_domain_observation(1, 7).expect("a");
        let b = cross_domain_observation(2, 7).expect("b");
        assert_ne!(a.structure.canonical_record(), b.structure.canonical_record());
        assert!(a
            .surface_predicates
            .predicates()
            .iter()
            .all(|predicate| !b.surface_predicates.contains(*predicate)));
    }

    #[test]
    fn two_domains_induce_topology_that_matches_a_third_domain() {
        let candidate = induce_cross_domain_template(1, 2, 9).expect("candidate");
        assert!(transfers_to_domain(&candidate, 3, 9).expect("transfer"));
        let record = candidate.generalization().canonical_record();
        assert!(!record.contains("relation:1001"));
        assert!(!record.contains("relation:2001"));
    }
}


/// Explicit corruption accounting for one Boolean observation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PredicateCorruption {
    pub original: usize,
    pub retained: usize,
    pub dropped: usize,
    pub added_noise: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CorruptionError {
    ZeroDropModulus,
    NoiseIdentifierOverflow,
}

/// Deterministically drop a declared subset and inject reserved novel predicates.
pub fn corrupt_surface_predicates(
    state: &BooleanState,
    drop_modulus: u32,
    noise_base: u32,
    noise_count: u16,
) -> Result<(BooleanState, PredicateCorruption), CorruptionError> {
    if drop_modulus == 0 {
        return Err(CorruptionError::ZeroDropModulus);
    }
    let retained = state
        .predicates()
        .iter()
        .copied()
        .filter(|predicate| predicate.raw() % drop_modulus != 0)
        .collect::<Vec<_>>();
    let retained_count = retained.len();
    let mut output = retained;
    for offset in 0..noise_count {
        let raw = noise_base
            .checked_add(u32::from(offset))
            .ok_or(CorruptionError::NoiseIdentifierOverflow)?;
        output.push(PredicateId::new(raw));
    }
    let output = BooleanState::new(output);
    Ok((
        output,
        PredicateCorruption {
            original: state.len(),
            retained: retained_count,
            dropped: state.len().saturating_sub(retained_count),
            added_noise: usize::from(noise_count),
        },
    ))
}

#[cfg(test)]
mod corruption_tests {
    use super::*;

    #[test]
    fn corruption_is_deterministic_and_accounted() {
        let state = BooleanState::new(vec![
            PredicateId::new(10),
            PredicateId::new(11),
            PredicateId::new(12),
            PredicateId::new(13),
        ]);
        let first = corrupt_surface_predicates(&state, 2, 90_000, 3).expect("first");
        let second = corrupt_surface_predicates(&state, 2, 90_000, 3).expect("second");
        assert_eq!(first, second);
        assert_eq!(first.1.dropped, 2);
        assert_eq!(first.1.added_noise, 3);
    }
}
