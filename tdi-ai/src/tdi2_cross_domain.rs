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


/// Non-mutating diagnosis of how recent evidence relates to an existing template.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConceptDriftDiagnosis {
    Stable,
    Refuted,
    SplitSuggested,
}

#[must_use]
pub fn diagnose_concept_drift(
    candidate: &PositiveTemplateCandidate,
    recent_positives: &[StructuralTerm],
    recent_negatives: &[StructuralTerm],
) -> ConceptDriftDiagnosis {
    let evaluation = super::tdi2_template_learning::evaluate_candidate_constraints(
        candidate,
        recent_positives,
        recent_negatives,
    );
    let loses_positive = evaluation.positive_admitted < evaluation.positive_total;
    let admits_negative = evaluation.negative_admitted > 0;
    match (loses_positive, admits_negative) {
        (false, false) => ConceptDriftDiagnosis::Stable,
        (true, false) => ConceptDriftDiagnosis::Refuted,
        (_, true) => ConceptDriftDiagnosis::SplitSuggested,
    }
}

#[cfg(test)]
mod drift_tests {
    use super::*;

    #[test]
    fn contradictory_recent_evidence_requests_review_not_mutation() {
        let training = [
            cross_domain_observation(1, 3).expect("one").structure,
            cross_domain_observation(2, 3).expect("two").structure,
        ];
        let candidate = induce_positive_template(&training).expect("candidate");
        let old_shape_as_negative = [cross_domain_observation(3, 3).expect("negative").structure];
        let changed_positive = StructuralTerm::application(
            StructuralSymbol::constructor(99_999),
            vec![StructuralTerm::atom(StructuralSymbol::constructor(1))],
        )
        .expect("changed");
        assert_eq!(
            diagnose_concept_drift(&candidate, &[changed_positive], &old_shape_as_negative),
            ConceptDriftDiagnosis::SplitSuggested
        );
    }
}


/// Count exact shared active predicates; no learned metric or hidden weighting.
#[must_use]
pub fn boolean_surface_overlap(left: &BooleanState, right: &BooleanState) -> usize {
    left.predicates()
        .iter()
        .filter(|predicate| right.contains(**predicate))
        .count()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StructuralVsSurface {
    pub structural_transfer: bool,
    pub surface_overlap: usize,
}

pub fn compare_structural_to_surface(
    case_id: u32,
    train_a: u32,
    train_b: u32,
    target_domain: u32,
) -> Result<StructuralVsSurface, CrossDomainError> {
    let candidate = induce_cross_domain_template(train_a, train_b, case_id)?;
    let source_surface = cross_domain_observation(train_b, case_id)?;
    let target = cross_domain_observation(target_domain, case_id)?;
    Ok(StructuralVsSurface {
        structural_transfer: match_induced_template(
            candidate.generalization(),
            &target.structure,
        )
        .is_some(),
        surface_overlap: boolean_surface_overlap(
            &source_surface.surface_predicates,
            &target.surface_predicates,
        ),
    })
}

#[cfg(test)]
mod surface_baseline_tests {
    use super::*;

    #[test]
    fn surface_disjoint_case_has_zero_boolean_overlap_but_structural_match() {
        let result = compare_structural_to_surface(8, 1, 2, 3).expect("comparison");
        assert!(result.structural_transfer);
        assert_eq!(result.surface_overlap, 0);
    }
}


/// Minimal bounded conjunctive rule baseline over active Boolean predicates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConjunctiveRule {
    required: BooleanState,
}

impl ConjunctiveRule {
    #[must_use]
    pub fn required(&self) -> &BooleanState {
        &self.required
    }

    #[must_use]
    pub fn matches(&self, state: &BooleanState) -> bool {
        self.required
            .predicates()
            .iter()
            .all(|predicate| state.contains(*predicate))
    }
}

pub fn induce_conjunctive_rule(positives: &[BooleanState]) -> Option<ConjunctiveRule> {
    let first = positives.first()?;
    let required = first
        .predicates()
        .iter()
        .copied()
        .filter(|predicate| {
            positives
                .iter()
                .skip(1)
                .all(|state| state.contains(*predicate))
        })
        .collect::<Vec<_>>();
    Some(ConjunctiveRule {
        required: BooleanState::new(required),
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SymbolicBaselineComparison {
    pub candidate_matches_target: bool,
    pub candidate_equals_anti_unification: bool,
    pub conjunctive_rule_matches_target: bool,
    pub conjunctive_rule_false_admission: bool,
}

pub fn compare_symbolic_baselines(
    case_id: u32,
    train_a: u32,
    train_b: u32,
    target_domain: u32,
) -> Result<SymbolicBaselineComparison, CrossDomainError> {
    let a = cross_domain_observation(train_a, case_id)?;
    let b = cross_domain_observation(train_b, case_id)?;
    let target = cross_domain_observation(target_domain, case_id)?;
    let candidate = induce_positive_template(&[a.structure.clone(), b.structure.clone()])?;
    let direct = super::tdi2_incremental_anti_unification::incremental_anti_unify(&[
        a.structure,
        b.structure,
    ])?;
    let rule = induce_conjunctive_rule(&[
        cross_domain_observation(train_a, case_id)?.surface_predicates,
        cross_domain_observation(train_b, case_id)?.surface_predicates,
    ])
    .expect("two positive states");
    let unrelated = cross_domain_observation(target_domain + 100, case_id)?;

    Ok(SymbolicBaselineComparison {
        candidate_matches_target: match_induced_template(
            candidate.generalization(),
            &target.structure,
        )
        .is_some(),
        candidate_equals_anti_unification: candidate.generalization() == direct.generalization(),
        conjunctive_rule_matches_target: rule.matches(&target.surface_predicates),
        conjunctive_rule_false_admission: rule.matches(&unrelated.surface_predicates),
    })
}

#[cfg(test)]
mod symbolic_baseline_tests {
    use super::*;

    #[test]
    fn current_candidate_is_honestly_equivalent_to_anti_unification_baseline() {
        let comparison = compare_symbolic_baselines(11, 1, 2, 3).expect("comparison");
        assert!(comparison.candidate_matches_target);
        assert!(comparison.candidate_equals_anti_unification);
        assert!(comparison.conjunctive_rule_matches_target);
        assert!(comparison.conjunctive_rule_false_admission);
    }
}


#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CrossDomainCampaignSummary {
    pub total: usize,
    pub structural_transfer_success: usize,
    pub zero_surface_overlap: usize,
    pub candidate_equals_anti_unification: usize,
    pub conjunctive_rule_false_admission: usize,
}

pub fn run_cross_domain_campaign(
    start_case: u32,
    count: usize,
    train_a: u32,
    train_b: u32,
    target_domain: u32,
) -> Result<CrossDomainCampaignSummary, CrossDomainError> {
    let mut summary = CrossDomainCampaignSummary::default();
    for offset in 0..count {
        let offset = u32::try_from(offset).map_err(|_| CrossDomainError::IdentifierOverflow)?;
        let case_id = start_case
            .checked_add(offset)
            .ok_or(CrossDomainError::IdentifierOverflow)?;
        let surface = compare_structural_to_surface(case_id, train_a, train_b, target_domain)?;
        let symbolic = compare_symbolic_baselines(case_id, train_a, train_b, target_domain)?;
        summary.total += 1;
        summary.structural_transfer_success += usize::from(surface.structural_transfer);
        summary.zero_surface_overlap += usize::from(surface.surface_overlap == 0);
        summary.candidate_equals_anti_unification +=
            usize::from(symbolic.candidate_equals_anti_unification);
        summary.conjunctive_rule_false_admission +=
            usize::from(symbolic.conjunctive_rule_false_admission);
    }
    Ok(summary)
}

pub const CROSS_DOMAIN_DEVELOPMENT_START: u32 = 1_000;
pub const CROSS_DOMAIN_DEVELOPMENT_CASES: usize = 32;

pub fn run_cross_domain_development() -> Result<CrossDomainCampaignSummary, CrossDomainError> {
    run_cross_domain_campaign(
        CROSS_DOMAIN_DEVELOPMENT_START,
        CROSS_DOMAIN_DEVELOPMENT_CASES,
        1,
        2,
        3,
    )
}

#[cfg(test)]
mod cross_domain_development_tests {
    use super::*;

    #[test]
    fn development_retains_the_anti_unification_null_result() {
        let summary = run_cross_domain_development().expect("development");
        assert_eq!(summary.total, CROSS_DOMAIN_DEVELOPMENT_CASES);
        assert_eq!(summary.structural_transfer_success, CROSS_DOMAIN_DEVELOPMENT_CASES);
        assert_eq!(summary.zero_surface_overlap, CROSS_DOMAIN_DEVELOPMENT_CASES);
        assert_eq!(
            summary.candidate_equals_anti_unification,
            CROSS_DOMAIN_DEVELOPMENT_CASES
        );
        assert_eq!(
            summary.conjunctive_rule_false_admission,
            CROSS_DOMAIN_DEVELOPMENT_CASES
        );
    }
}


pub const CROSS_DOMAIN_VALIDATION_START: u32 = 2_000;
pub const CROSS_DOMAIN_VALIDATION_CASES: usize = 32;

pub fn run_cross_domain_validation() -> Result<CrossDomainCampaignSummary, CrossDomainError> {
    run_cross_domain_campaign(
        CROSS_DOMAIN_VALIDATION_START,
        CROSS_DOMAIN_VALIDATION_CASES,
        4,
        5,
        6,
    )
}

#[cfg(test)]
mod cross_domain_validation_tests {
    use super::*;

    #[test]
    fn validation_is_disjoint_and_preserves_null_baseline_finding() {
        assert!(
            CROSS_DOMAIN_DEVELOPMENT_START + CROSS_DOMAIN_DEVELOPMENT_CASES as u32
                <= CROSS_DOMAIN_VALIDATION_START
        );
        let summary = run_cross_domain_validation().expect("validation");
        assert_eq!(summary.total, CROSS_DOMAIN_VALIDATION_CASES);
        assert_eq!(summary.structural_transfer_success, CROSS_DOMAIN_VALIDATION_CASES);
        assert_eq!(summary.zero_surface_overlap, CROSS_DOMAIN_VALIDATION_CASES);
        assert_eq!(
            summary.candidate_equals_anti_unification,
            CROSS_DOMAIN_VALIDATION_CASES
        );
        assert_eq!(
            summary.conjunctive_rule_false_admission,
            CROSS_DOMAIN_VALIDATION_CASES
        );
    }
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CrossDomainLogicalAccounting {
    pub support: usize,
    pub source_records: usize,
    pub template_nodes: usize,
    pub variables: usize,
    pub canonical_bytes: usize,
}

#[must_use]
pub fn cross_domain_logical_accounting(
    candidate: &PositiveTemplateCandidate,
) -> CrossDomainLogicalAccounting {
    fn count(term: &StructuralTerm) -> (usize, usize) {
        match term.kind() {
            super::tdi2_structural_terms::StructuralTermKind::Variable(_) => (1, 1),
            super::tdi2_structural_terms::StructuralTermKind::Atom(_) => (1, 0),
            super::tdi2_structural_terms::StructuralTermKind::Application { arguments, .. } => {
                arguments.iter().fold((1usize, 0usize), |(nodes, variables), child| {
                    let (child_nodes, child_variables) = count(child);
                    (nodes + child_nodes, variables + child_variables)
                })
            }
        }
    }
    let (template_nodes, variables) = count(candidate.generalization());
    CrossDomainLogicalAccounting {
        support: candidate.support(),
        source_records: candidate.source_records().len(),
        template_nodes,
        variables,
        canonical_bytes: candidate.canonical_record().len(),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CrossDomainReplay {
    pub first: CrossDomainCampaignSummary,
    pub second: CrossDomainCampaignSummary,
    pub identical: bool,
    pub accounting: CrossDomainLogicalAccounting,
}

pub fn replay_cross_domain_validation() -> Result<CrossDomainReplay, CrossDomainError> {
    let first = run_cross_domain_validation()?;
    let second = run_cross_domain_validation()?;
    let candidate = induce_cross_domain_template(4, 5, CROSS_DOMAIN_VALIDATION_START)?;
    Ok(CrossDomainReplay {
        first,
        second,
        identical: first == second,
        accounting: cross_domain_logical_accounting(&candidate),
    })
}

#[cfg(test)]
mod replay_tests {
    use super::*;

    #[test]
    fn validation_replays_exactly_and_cost_is_logical_not_wall_clock() {
        let replay = replay_cross_domain_validation().expect("replay");
        assert!(replay.identical);
        assert_eq!(replay.accounting.support, 2);
        assert!(replay.accounting.template_nodes > 0);
        assert!(replay.accounting.variables > 0);
        assert!(replay.accounting.canonical_bytes > 0);
    }
}
