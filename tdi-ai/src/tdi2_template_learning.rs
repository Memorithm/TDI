//! Evidence-gated template learning primitives for TDI-2.2.
//!
//! This module operates only on bounded structural terms derived from declared
//! observations. Expected labels, evaluator annotations, final data and latency
//! are not induction inputs.

use super::tdi2_incremental_anti_unification::{
    IncrementalAntiUnificationError, incremental_anti_unify,
};
use super::tdi2_structural_terms::StructuralTerm;

/// Versioned identity for a positive-only induced template candidate.
pub const POSITIVE_TEMPLATE_SCHEMA: &str = "tdi2.2-positive-template-v1";

/// A candidate generalized from positive structural examples only.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PositiveTemplateCandidate {
    generalization: StructuralTerm,
    support: usize,
    source_records: Vec<String>,
}

impl PositiveTemplateCandidate {
    /// Generalized structure induced from the positive examples.
    #[must_use]
    pub const fn generalization(&self) -> &StructuralTerm {
        &self.generalization
    }

    /// Number of retained positive evidence rows, including duplicates.
    #[must_use]
    pub const fn support(&self) -> usize {
        self.support
    }

    /// Canonically ordered source structures retained for replay/provenance.
    #[must_use]
    pub fn source_records(&self) -> &[String] {
        &self.source_records
    }

    /// Canonical non-final evidence record.
    #[must_use]
    pub fn canonical_record(&self) -> String {
        use core::fmt::Write as _;
        let generalization = self.generalization.canonical_record();
        let mut output = format!(
            "{POSITIVE_TEMPLATE_SCHEMA};support={};generalization={}:{};sources=[",
            self.support,
            generalization.len(),
            generalization
        );
        for record in &self.source_records {
            write!(output, "{}:{};", record.len(), record).expect("write to string");
        }
        output.push_str("];\n");
        output
    }
}

/// Induce one positive-only template candidate from bounded structural examples.
pub fn induce_positive_template(
    positives: &[StructuralTerm],
) -> Result<PositiveTemplateCandidate, IncrementalAntiUnificationError> {
    let result = incremental_anti_unify(positives)?;
    let mut source_records = positives
        .iter()
        .map(StructuralTerm::canonical_record)
        .collect::<Vec<_>>();
    source_records.sort_unstable();
    Ok(PositiveTemplateCandidate {
        generalization: result.generalization().clone(),
        support: positives.len(),
        source_records,
    })
}

#[cfg(test)]
mod positive_tests {
    use super::*;
    use crate::experimental::tdi2_structural_terms::{StructuralSymbol, StructuralTerm};

    fn atom(id: u32) -> StructuralTerm {
        StructuralTerm::atom(StructuralSymbol::constructor(10_000 + id))
    }

    fn app(id: u32, arguments: Vec<StructuralTerm>) -> StructuralTerm {
        StructuralTerm::application(StructuralSymbol::constructor(id), arguments).expect("term")
    }

    #[test]
    fn positive_candidate_generalizes_surface_difference_and_keeps_support() {
        let positives = [
            app(1, vec![atom(10), atom(20)]),
            app(1, vec![atom(11), atom(20)]),
            app(1, vec![atom(12), atom(20)]),
        ];
        let candidate = induce_positive_template(&positives).expect("candidate");
        assert_eq!(candidate.support(), 3);
        assert!(
            candidate
                .generalization()
                .canonical_record()
                .contains("v0;")
        );
        assert_eq!(candidate.source_records().len(), 3);
    }

    #[test]
    fn empty_positive_set_fails_closed() {
        assert_eq!(
            induce_positive_template(&[]),
            Err(IncrementalAntiUnificationError::EmptyInput)
        );
    }
}

use std::collections::BTreeMap;
use super::tdi2_structural_terms::{StructuralTermKind, StructuralVariableId};

/// Exact first-order variable binding recovered while matching a candidate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemplateMatch {
    bindings: Vec<(StructuralVariableId, StructuralTerm)>,
}

impl TemplateMatch {
    #[must_use]
    pub fn bindings(&self) -> &[(StructuralVariableId, StructuralTerm)] {
        &self.bindings
    }

    #[must_use]
    pub fn binding(&self, variable: StructuralVariableId) -> Option<&StructuralTerm> {
        self.bindings
            .binary_search_by_key(&variable, |(id, _)| *id)
            .ok()
            .map(|index| &self.bindings[index].1)
    }
}

/// Match a generalized structural term against one concrete structural example.
#[must_use]
pub fn match_induced_template(
    template: &StructuralTerm,
    example: &StructuralTerm,
) -> Option<TemplateMatch> {
    let mut bindings = BTreeMap::new();
    if !match_term(template, example, &mut bindings) {
        return None;
    }
    Some(TemplateMatch {
        bindings: bindings.into_iter().collect(),
    })
}

fn match_term(
    template: &StructuralTerm,
    example: &StructuralTerm,
    bindings: &mut BTreeMap<StructuralVariableId, StructuralTerm>,
) -> bool {
    match (template.kind(), example.kind()) {
        (StructuralTermKind::Variable(variable), _) => match bindings.get(variable) {
            Some(existing) => existing == example,
            None => {
                bindings.insert(*variable, example.clone());
                true
            }
        },
        (StructuralTermKind::Atom(left), StructuralTermKind::Atom(right)) => left == right,
        (
            StructuralTermKind::Application {
                symbol: left_symbol,
                arguments: left_arguments,
            },
            StructuralTermKind::Application {
                symbol: right_symbol,
                arguments: right_arguments,
            },
        ) => {
            left_symbol == right_symbol
                && left_arguments.len() == right_arguments.len()
                && left_arguments
                    .iter()
                    .zip(right_arguments)
                    .all(|(left, right)| match_term(left, right, bindings))
        }
        _ => false,
    }
}

/// Held-out positive/negative accounting for one induced template candidate.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CandidateConstraintEvaluation {
    pub positive_total: usize,
    pub positive_admitted: usize,
    pub negative_total: usize,
    pub negative_admitted: usize,
}

impl CandidateConstraintEvaluation {
    #[must_use]
    pub const fn preserves_all_positives(self) -> bool {
        self.positive_total == self.positive_admitted
    }

    #[must_use]
    pub const fn rejects_all_negatives(self) -> bool {
        self.negative_admitted == 0
    }

    #[must_use]
    pub const fn passes(self) -> bool {
        self.preserves_all_positives() && self.rejects_all_negatives()
    }
}

/// Evaluate a candidate on explicitly supplied positive and negative structures.
#[must_use]
pub fn evaluate_candidate_constraints(
    candidate: &PositiveTemplateCandidate,
    positives: &[StructuralTerm],
    negatives: &[StructuralTerm],
) -> CandidateConstraintEvaluation {
    CandidateConstraintEvaluation {
        positive_total: positives.len(),
        positive_admitted: positives
            .iter()
            .filter(|example| match_induced_template(candidate.generalization(), example).is_some())
            .count(),
        negative_total: negatives.len(),
        negative_admitted: negatives
            .iter()
            .filter(|example| match_induced_template(candidate.generalization(), example).is_some())
            .count(),
    }
}

#[cfg(test)]
mod constraint_tests {
    use super::*;
    use crate::experimental::tdi2_structural_terms::{
        StructuralSymbol, StructuralVariableId,
    };

    fn atom(id: u32) -> StructuralTerm {
        StructuralTerm::atom(StructuralSymbol::constructor(10_000 + id))
    }

    fn app(id: u32, arguments: Vec<StructuralTerm>) -> StructuralTerm {
        StructuralTerm::application(StructuralSymbol::constructor(id), arguments).expect("term")
    }

    #[test]
    fn repeated_variable_requires_consistent_binding() {
        let variable = StructuralTerm::variable(StructuralVariableId::new(3));
        let pattern = app(1, vec![variable.clone(), variable]);
        assert!(match_induced_template(&pattern, &app(1, vec![atom(4), atom(4)])).is_some());
        assert!(match_induced_template(&pattern, &app(1, vec![atom(4), atom(5)])).is_none());
    }

    #[test]
    fn counterexample_remains_visible() {
        let positives = [app(1, vec![atom(1)]), app(1, vec![atom(2)])];
        let candidate = induce_positive_template(&positives).expect("candidate");
        let evaluation = evaluate_candidate_constraints(
            &candidate,
            &positives,
            &[app(2, vec![atom(3)]), app(1, vec![atom(9)])],
        );
        assert_eq!(evaluation.negative_admitted, 1);
        assert!(!evaluation.passes());
    }
}


/// Stable structural path from a template root to a term node.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StructuralPath(Vec<u16>);

impl StructuralPath {
    #[must_use]
    pub fn indices(&self) -> &[u16] {
        &self.0
    }
}

/// Explicit partition of fixed structural nodes and contingent variables.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TemplatePartition {
    fixed_nodes: Vec<StructuralPath>,
    contingent_variables: Vec<(StructuralPath, StructuralVariableId)>,
}

impl TemplatePartition {
    #[must_use]
    pub fn fixed_nodes(&self) -> &[StructuralPath] {
        &self.fixed_nodes
    }

    #[must_use]
    pub fn contingent_variables(&self) -> &[(StructuralPath, StructuralVariableId)] {
        &self.contingent_variables
    }
}

/// Partition an induced term without assigning semantics to variable positions.
#[must_use]
pub fn partition_template(template: &StructuralTerm) -> TemplatePartition {
    fn visit(term: &StructuralTerm, path: &mut Vec<u16>, out: &mut TemplatePartition) {
        match term.kind() {
            StructuralTermKind::Variable(variable) => out
                .contingent_variables
                .push((StructuralPath(path.clone()), *variable)),
            StructuralTermKind::Atom(_) => out.fixed_nodes.push(StructuralPath(path.clone())),
            StructuralTermKind::Application { arguments, .. } => {
                out.fixed_nodes.push(StructuralPath(path.clone()));
                for (index, child) in arguments.iter().enumerate() {
                    path.push(u16::try_from(index).expect("bounded arity fits u16"));
                    visit(child, path, out);
                    path.pop();
                }
            }
        }
    }
    let mut out = TemplatePartition::default();
    visit(template, &mut Vec::new(), &mut out);
    out
}

#[cfg(test)]
mod partition_tests {
    use super::*;
    use crate::experimental::tdi2_structural_terms::StructuralSymbol;

    #[test]
    fn variables_are_contingent_while_constructor_is_fixed() {
        let term = StructuralTerm::application(
            StructuralSymbol::constructor(1),
            vec![
                StructuralTerm::variable(StructuralVariableId::new(7)),
                StructuralTerm::atom(StructuralSymbol::constructor(2)),
            ],
        )
        .expect("term");
        let partition = partition_template(&term);
        assert_eq!(partition.contingent_variables().len(), 1);
        assert_eq!(partition.fixed_nodes().len(), 2);
        assert_eq!(partition.contingent_variables()[0].0.indices(), &[0]);
    }
}


use std::collections::BTreeSet;
use super::tdi2_structural_terms::StructuralSymbolNamespace;

/// One variable whose bindings are concrete episode-local entities in every example.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InducedRole {
    variable: StructuralVariableId,
    binding_records: Vec<String>,
}

impl InducedRole {
    #[must_use]
    pub const fn variable(&self) -> StructuralVariableId {
        self.variable
    }

    #[must_use]
    pub fn binding_records(&self) -> &[String] {
        &self.binding_records
    }

    #[must_use]
    pub fn distinct_entity_count(&self) -> usize {
        self.binding_records.iter().collect::<BTreeSet<_>>().len()
    }
}

/// Promote a contingent variable to a role candidate only when every supplied
/// positive example binds it to an observable entity constant.
#[must_use]
pub fn induce_roles(
    candidate: &PositiveTemplateCandidate,
    positives: &[StructuralTerm],
) -> Vec<InducedRole> {
    let variables = partition_template(candidate.generalization())
        .contingent_variables
        .into_iter()
        .map(|(_, variable)| variable)
        .collect::<BTreeSet<_>>();

    variables
        .into_iter()
        .filter_map(|variable| {
            let mut bindings = Vec::with_capacity(positives.len());
            for example in positives {
                let matched = match_induced_template(candidate.generalization(), example)?;
                let bound = matched.binding(variable)?;
                match bound.kind() {
                    StructuralTermKind::Atom(symbol)
                        if symbol.namespace() == StructuralSymbolNamespace::Entity =>
                    {
                        bindings.push(bound.canonical_record());
                    }
                    _ => return None,
                }
            }
            Some(InducedRole {
                variable,
                binding_records: bindings,
            })
        })
        .collect()
}

#[cfg(test)]
mod role_tests {
    use super::*;
    use crate::experimental::tdi2_observation_graph::{ObservedEntityId, ObservedRelationId};
    use crate::experimental::tdi2_structural_terms::{StructuralSymbol, StructuralTerm};
    use crate::experimental::tdi2_template_induction::EpisodeId;

    fn relation(episode: u64, entity: u32) -> StructuralTerm {
        StructuralTerm::application(
            StructuralSymbol::relation(ObservedRelationId::new(3)),
            vec![StructuralTerm::atom(StructuralSymbol::entity(
                EpisodeId::new(episode),
                ObservedEntityId::new(entity),
            ))],
        )
        .expect("term")
    }

    #[test]
    fn changing_entity_constants_induce_a_role() {
        let positives = [relation(1, 10), relation(2, 20), relation(3, 30)];
        let candidate = induce_positive_template(&positives).expect("candidate");
        let roles = induce_roles(&candidate, &positives);
        assert_eq!(roles.len(), 1);
        assert_eq!(roles[0].distinct_entity_count(), 3);
    }
}


/// Descriptive relation-system diagnostic; deliberately not a single quality score.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RelationSystematicity {
    pub relation_applications: usize,
    pub role_linked_relations: usize,
    pub distinct_roles_linked: usize,
    pub roles_reused_across_relations: usize,
}

/// Count whether induced roles participate in a connected relation system.
#[must_use]
pub fn relation_systematicity(
    template: &StructuralTerm,
    roles: &[InducedRole],
) -> RelationSystematicity {
    let role_ids = roles
        .iter()
        .map(InducedRole::variable)
        .collect::<BTreeSet<_>>();
    let mut relation_applications = 0usize;
    let mut role_linked_relations = 0usize;
    let mut occurrences = BTreeMap::<StructuralVariableId, usize>::new();

    fn walk(
        term: &StructuralTerm,
        role_ids: &BTreeSet<StructuralVariableId>,
        relation_applications: &mut usize,
        role_linked_relations: &mut usize,
        occurrences: &mut BTreeMap<StructuralVariableId, usize>,
    ) {
        if let StructuralTermKind::Application { symbol, arguments } = term.kind() {
            if symbol.namespace() == StructuralSymbolNamespace::Relation {
                *relation_applications += 1;
                let linked = arguments
                    .iter()
                    .filter_map(|argument| match argument.kind() {
                        StructuralTermKind::Variable(variable) if role_ids.contains(variable) => {
                            Some(*variable)
                        }
                        _ => None,
                    })
                    .collect::<BTreeSet<_>>();
                if linked.len() >= 2 {
                    *role_linked_relations += 1;
                }
                for variable in linked {
                    *occurrences.entry(variable).or_default() += 1;
                }
            }
            for argument in arguments {
                walk(
                    argument,
                    role_ids,
                    relation_applications,
                    role_linked_relations,
                    occurrences,
                );
            }
        }
    }

    walk(
        template,
        &role_ids,
        &mut relation_applications,
        &mut role_linked_relations,
        &mut occurrences,
    );
    RelationSystematicity {
        relation_applications,
        role_linked_relations,
        distinct_roles_linked: occurrences.len(),
        roles_reused_across_relations: occurrences.values().filter(|count| **count > 1).count(),
    }
}

#[cfg(test)]
mod systematicity_tests {
    use super::*;
    use crate::experimental::tdi2_observation_graph::ObservedRelationId;
    use crate::experimental::tdi2_structural_terms::StructuralSymbol;

    #[test]
    fn linked_relation_system_is_counted_without_collapsing_to_one_score() {
        let v0 = StructuralVariableId::new(0);
        let v1 = StructuralVariableId::new(1);
        let v2 = StructuralVariableId::new(2);
        let r1 = StructuralTerm::application(
            StructuralSymbol::relation(ObservedRelationId::new(1)),
            vec![StructuralTerm::variable(v0), StructuralTerm::variable(v1)],
        )
        .expect("r1");
        let r2 = StructuralTerm::application(
            StructuralSymbol::relation(ObservedRelationId::new(2)),
            vec![StructuralTerm::variable(v1), StructuralTerm::variable(v2)],
        )
        .expect("r2");
        let root = StructuralTerm::application(
            StructuralSymbol::constructor(99),
            vec![r1, r2],
        )
        .expect("root");
        let roles = [v0, v1, v2]
            .into_iter()
            .map(|variable| InducedRole {
                variable,
                binding_records: vec!["entity".to_owned()],
            })
            .collect::<Vec<_>>();
        let diagnostic = relation_systematicity(&root, &roles);
        assert_eq!(diagnostic.relation_applications, 2);
        assert_eq!(diagnostic.role_linked_relations, 2);
        assert_eq!(diagnostic.roles_reused_across_relations, 1);
    }
}


/// Separate evidence and complexity dimensions for a template candidate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TemplateCandidateScore {
    pub positive_total: usize,
    pub positive_admitted: usize,
    pub negative_total: usize,
    pub negative_admitted: usize,
    pub fixed_nodes: usize,
    pub contingent_variables: usize,
    pub role_linked_relations: usize,
}

#[must_use]
pub fn score_template_candidate(
    candidate: &PositiveTemplateCandidate,
    positives: &[StructuralTerm],
    negatives: &[StructuralTerm],
) -> TemplateCandidateScore {
    let evaluation = evaluate_candidate_constraints(candidate, positives, negatives);
    let partition = partition_template(candidate.generalization());
    let roles = induce_roles(candidate, positives);
    let systematicity = relation_systematicity(candidate.generalization(), &roles);
    TemplateCandidateScore {
        positive_total: evaluation.positive_total,
        positive_admitted: evaluation.positive_admitted,
        negative_total: evaluation.negative_total,
        negative_admitted: evaluation.negative_admitted,
        fixed_nodes: partition.fixed_nodes().len(),
        contingent_variables: partition.contingent_variables().len(),
        role_linked_relations: systematicity.role_linked_relations,
    }
}

#[cfg(test)]
mod score_tests {
    use super::*;
    use crate::experimental::tdi2_structural_terms::{StructuralSymbol, StructuralTerm};

    fn atom(id: u32) -> StructuralTerm {
        StructuralTerm::atom(StructuralSymbol::constructor(id))
    }
    fn fact(id: u32, x: StructuralTerm) -> StructuralTerm {
        StructuralTerm::application(StructuralSymbol::constructor(id), vec![x]).expect("term")
    }

    #[test]
    fn fit_and_complexity_remain_separate_fields() {
        let positives = [fact(1, atom(10)), fact(1, atom(11))];
        let candidate = induce_positive_template(&positives).expect("candidate");
        let score = score_template_candidate(&candidate, &positives, &[fact(2, atom(12))]);
        assert_eq!(score.positive_admitted, 2);
        assert_eq!(score.negative_admitted, 0);
        assert!(score.fixed_nodes > 0);
        assert!(score.contingent_variables > 0);
    }
}


/// Frozen lexicographic Development/Validation candidate selection policy.
pub const TEMPLATE_SELECTION_POLICY: &str =
    "tdi2.2-template-selection-v1:pos-desc;neg-asc;vars-asc;record-asc";

/// Select a candidate without fitting weights on the evaluated population.
#[must_use]
pub fn select_template_candidate<'a>(
    candidates: &'a [PositiveTemplateCandidate],
    positives: &[StructuralTerm],
    negatives: &[StructuralTerm],
) -> Option<&'a PositiveTemplateCandidate> {
    candidates.iter().min_by(|left, right| {
        let left_score = score_template_candidate(left, positives, negatives);
        let right_score = score_template_candidate(right, positives, negatives);
        right_score
            .positive_admitted
            .cmp(&left_score.positive_admitted)
            .then_with(|| left_score.negative_admitted.cmp(&right_score.negative_admitted))
            .then_with(|| {
                left_score
                    .contingent_variables
                    .cmp(&right_score.contingent_variables)
            })
            .then_with(|| left.canonical_record().cmp(&right.canonical_record()))
    })
}

#[cfg(test)]
mod selection_tests {
    use super::*;
    use crate::experimental::tdi2_structural_terms::{StructuralSymbol, StructuralTerm};

    fn atom(id: u32) -> StructuralTerm {
        StructuralTerm::atom(StructuralSymbol::constructor(id))
    }
    fn fact(id: u32, x: StructuralTerm) -> StructuralTerm {
        StructuralTerm::application(StructuralSymbol::constructor(id), vec![x]).expect("term")
    }

    #[test]
    fn frozen_policy_prefers_negative_rejection_after_equal_positive_fit() {
        let broad_positives = [fact(1, atom(1)), fact(1, atom(2))];
        let broad = induce_positive_template(&broad_positives).expect("broad");
        let narrow = induce_positive_template(&[fact(1, atom(1))]).expect("narrow");
        let candidates = [broad.clone(), narrow];
        let validation_positives = [fact(1, atom(1))];
        let validation_negatives = [fact(1, atom(9))];
        let chosen = select_template_candidate(
            &candidates,
            &validation_positives,
            &validation_negatives,
        )
        .expect("chosen");
        assert_ne!(chosen.canonical_record(), broad.canonical_record());
        assert!(TEMPLATE_SELECTION_POLICY.contains("neg-asc"));
    }
}


use super::tdi2_structural_terms::StructuralTermError;

/// Alpha-normalize variables by deterministic first occurrence.
pub fn alpha_normalize_template(
    term: &StructuralTerm,
) -> Result<StructuralTerm, StructuralTermError> {
    fn rewrite(
        term: &StructuralTerm,
        map: &mut BTreeMap<StructuralVariableId, StructuralVariableId>,
        next: &mut u32,
    ) -> Result<StructuralTerm, StructuralTermError> {
        match term.kind() {
            StructuralTermKind::Variable(old) => {
                let new = *map.entry(*old).or_insert_with(|| {
                    let id = StructuralVariableId::new(*next);
                    *next += 1;
                    id
                });
                Ok(StructuralTerm::variable(new))
            }
            StructuralTermKind::Atom(symbol) => Ok(StructuralTerm::atom(*symbol)),
            StructuralTermKind::Application { symbol, arguments } => StructuralTerm::application(
                *symbol,
                arguments
                    .iter()
                    .map(|argument| rewrite(argument, map, next))
                    .collect::<Result<Vec<_>, _>>()?,
            ),
        }
    }
    rewrite(term, &mut BTreeMap::new(), &mut 0)
}

pub fn alpha_equivalent(
    left: &StructuralTerm,
    right: &StructuralTerm,
) -> Result<bool, StructuralTermError> {
    Ok(alpha_normalize_template(left)? == alpha_normalize_template(right)?)
}

#[cfg(test)]
mod equivalence_tests {
    use super::*;
    use crate::experimental::tdi2_structural_terms::{StructuralSymbol, StructuralTerm};

    #[test]
    fn variable_names_do_not_define_template_identity() {
        let left = StructuralTerm::application(
            StructuralSymbol::constructor(1),
            vec![StructuralTerm::variable(StructuralVariableId::new(3))],
        )
        .expect("left");
        let right = StructuralTerm::application(
            StructuralSymbol::constructor(1),
            vec![StructuralTerm::variable(StructuralVariableId::new(99))],
        )
        .expect("right");
        assert!(alpha_equivalent(&left, &right).expect("equivalence"));
    }
}
