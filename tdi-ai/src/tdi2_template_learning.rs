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
