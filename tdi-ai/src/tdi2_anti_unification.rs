//! Deterministic bounded first-order anti-unification for TDI-2.2.
//!
//! This module provides the Stage-C structural baseline requested by the
//! TDI-2.2 protocol. It computes a least-general generalization (LGG) of two
//! bounded structural terms using only their declared syntax. It never receives
//! labels, outcomes, evaluator state, latency, or protected/final data.

use std::collections::BTreeMap;

use super::tdi2_structural_terms::{
    MAX_STRUCTURAL_TERM_NODES, StructuralTerm, StructuralTermError, StructuralTermKind,
    StructuralVariableId,
};

/// Versioned identity for the exact pairwise anti-unification baseline.
pub const ANTI_UNIFICATION_SCHEMA: &str = "tdi2.2-anti-unification-v1";
/// A result cannot introduce more variables than the bounded term node budget.
pub const MAX_ANTI_UNIFICATION_VARIABLES: usize = MAX_STRUCTURAL_TERM_NODES;

/// One exact pairwise LGG with witness substitutions for both source terms.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AntiUnificationResult {
    generalization: StructuralTerm,
    left_substitution: Vec<(StructuralVariableId, StructuralTerm)>,
    right_substitution: Vec<(StructuralVariableId, StructuralTerm)>,
}

impl AntiUnificationResult {
    #[must_use]
    pub fn generalization(&self) -> &StructuralTerm {
        &self.generalization
    }

    #[must_use]
    pub fn left_substitution(&self) -> &[(StructuralVariableId, StructuralTerm)] {
        &self.left_substitution
    }

    #[must_use]
    pub fn right_substitution(&self) -> &[(StructuralVariableId, StructuralTerm)] {
        &self.right_substitution
    }

    /// Reconstruct the exact left input from the LGG and its witness.
    pub fn reconstruct_left(&self) -> Result<StructuralTerm, AntiUnificationError> {
        apply_substitution(&self.generalization, &self.left_substitution)
    }

    /// Reconstruct the exact right input from the LGG and its witness.
    pub fn reconstruct_right(&self) -> Result<StructuralTerm, AntiUnificationError> {
        apply_substitution(&self.generalization, &self.right_substitution)
    }

    /// Deterministic evidence record binding the LGG and both witnesses.
    #[must_use]
    pub fn canonical_record(&self) -> String {
        let mut output = format!("{ANTI_UNIFICATION_SCHEMA};");
        push_term_record(&mut output, "g", &self.generalization);
        push_substitution_record(&mut output, "l", &self.left_substitution);
        push_substitution_record(&mut output, "r", &self.right_substitution);
        output
    }
}

/// Fail-closed pairwise anti-unification errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AntiUnificationError {
    LeftInputInvalid(StructuralTermError),
    RightInputInvalid(StructuralTermError),
    VariableIdExhausted,
    VariableLimitExceeded { maximum: usize },
    ResultInvalid(StructuralTermError),
    WitnessMismatch,
}

/// Compute a deterministic pairwise first-order LGG.
///
/// Equal subterms are retained exactly. Applications with the same symbol and
/// arity are generalized recursively. Every other ordered source-term pair is
/// replaced by a fresh variable. Repeated occurrences of the same ordered pair
/// reuse that variable, preserving equality structure without incorrectly
/// identifying reversed pairs.
pub fn anti_unify(
    left: &StructuralTerm,
    right: &StructuralTerm,
) -> Result<AntiUnificationResult, AntiUnificationError> {
    left.validate()
        .map_err(AntiUnificationError::LeftInputInvalid)?;
    right
        .validate()
        .map_err(AntiUnificationError::RightInputInvalid)?;

    let maximum_input_variable = max_variable_id(left).max(max_variable_id(right));
    let next_variable = match maximum_input_variable {
        Some(u32::MAX) => None,
        Some(value) => value.checked_add(1),
        None => Some(0),
    };

    let mut context = AntiUnificationContext {
        next_variable,
        mismatch_variables: BTreeMap::new(),
        left_substitution: Vec::new(),
        right_substitution: Vec::new(),
    };
    let generalization = context.generalize(left, right)?;
    generalization
        .validate()
        .map_err(AntiUnificationError::ResultInvalid)?;

    let result = AntiUnificationResult {
        generalization,
        left_substitution: context.left_substitution,
        right_substitution: context.right_substitution,
    };

    // Keep the witnesses executable as part of the contract, rather than
    // trusting construction alone. Any future syntax extension must preserve
    // exact reconstruction or fail closed here.
    if result.reconstruct_left()? != *left || result.reconstruct_right()? != *right {
        return Err(AntiUnificationError::WitnessMismatch);
    }

    Ok(result)
}

struct AntiUnificationContext {
    next_variable: Option<u32>,
    mismatch_variables: BTreeMap<(StructuralTerm, StructuralTerm), StructuralVariableId>,
    left_substitution: Vec<(StructuralVariableId, StructuralTerm)>,
    right_substitution: Vec<(StructuralVariableId, StructuralTerm)>,
}

impl AntiUnificationContext {
    fn generalize(
        &mut self,
        left: &StructuralTerm,
        right: &StructuralTerm,
    ) -> Result<StructuralTerm, AntiUnificationError> {
        if left == right {
            return Ok(left.clone());
        }

        if let (
            StructuralTermKind::Application {
                symbol: left_symbol,
                arguments: left_arguments,
            },
            StructuralTermKind::Application {
                symbol: right_symbol,
                arguments: right_arguments,
            },
        ) = (left.kind(), right.kind())
        {
            if left_symbol == right_symbol && left_arguments.len() == right_arguments.len() {
                let arguments = left_arguments
                    .iter()
                    .zip(right_arguments)
                    .map(|(left_argument, right_argument)| {
                        self.generalize(left_argument, right_argument)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                return StructuralTerm::application(*left_symbol, arguments)
                    .map_err(AntiUnificationError::ResultInvalid);
            }
        }

        self.variable_for_mismatch(left, right)
            .map(StructuralTerm::variable)
    }

    fn variable_for_mismatch(
        &mut self,
        left: &StructuralTerm,
        right: &StructuralTerm,
    ) -> Result<StructuralVariableId, AntiUnificationError> {
        let key = (left.clone(), right.clone());
        if let Some(variable) = self.mismatch_variables.get(&key) {
            return Ok(*variable);
        }
        if self.mismatch_variables.len() >= MAX_ANTI_UNIFICATION_VARIABLES {
            return Err(AntiUnificationError::VariableLimitExceeded {
                maximum: MAX_ANTI_UNIFICATION_VARIABLES,
            });
        }

        let raw = self
            .next_variable
            .ok_or(AntiUnificationError::VariableIdExhausted)?;
        let variable = StructuralVariableId::new(raw);
        self.next_variable = raw.checked_add(1);
        self.mismatch_variables.insert(key, variable);
        self.left_substitution.push((variable, left.clone()));
        self.right_substitution.push((variable, right.clone()));
        Ok(variable)
    }
}

fn max_variable_id(term: &StructuralTerm) -> Option<u32> {
    match term.kind() {
        StructuralTermKind::Variable(variable) => Some(variable.raw()),
        StructuralTermKind::Atom(_) => None,
        StructuralTermKind::Application { arguments, .. } => {
            arguments.iter().filter_map(max_variable_id).max()
        }
    }
}

fn apply_substitution(
    term: &StructuralTerm,
    substitution: &[(StructuralVariableId, StructuralTerm)],
) -> Result<StructuralTerm, AntiUnificationError> {
    match term.kind() {
        StructuralTermKind::Variable(variable) => Ok(substitution
            .binary_search_by_key(variable, |(candidate, _)| *candidate)
            .map_or_else(|_| term.clone(), |index| substitution[index].1.clone())),
        StructuralTermKind::Atom(_) => Ok(term.clone()),
        StructuralTermKind::Application { symbol, arguments } => {
            let arguments = arguments
                .iter()
                .map(|argument| apply_substitution(argument, substitution))
                .collect::<Result<Vec<_>, _>>()?;
            StructuralTerm::application(*symbol, arguments)
                .map_err(AntiUnificationError::ResultInvalid)
        }
    }
}

fn push_term_record(output: &mut String, prefix: &str, term: &StructuralTerm) {
    use core::fmt::Write as _;
    let record = term.canonical_record();
    write!(output, "{prefix}{}:{record};", record.len()).expect("write to string");
}

fn push_substitution_record(
    output: &mut String,
    prefix: &str,
    substitution: &[(StructuralVariableId, StructuralTerm)],
) {
    use core::fmt::Write as _;
    write!(output, "{prefix}{}[", substitution.len()).expect("write to string");
    for (variable, term) in substitution {
        let record = term.canonical_record();
        write!(output, "v{}={}:{record};", variable.raw(), record.len()).expect("write to string");
    }
    output.push_str("];");
}

impl core::fmt::Display for AntiUnificationError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::LeftInputInvalid(error) => {
                write!(formatter, "left structural term is invalid: {error}")
            }
            Self::RightInputInvalid(error) => {
                write!(formatter, "right structural term is invalid: {error}")
            }
            Self::VariableIdExhausted => {
                formatter.write_str("no fresh structural variable id remains")
            }
            Self::VariableLimitExceeded { maximum } => {
                write!(
                    formatter,
                    "anti-unification variable count exceeds {maximum}"
                )
            }
            Self::ResultInvalid(error) => {
                write!(formatter, "anti-unification result is invalid: {error}")
            }
            Self::WitnessMismatch => {
                formatter.write_str("anti-unification witness does not reconstruct its source term")
            }
        }
    }
}

impl std::error::Error for AntiUnificationError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::experimental::tdi2_structural_terms::StructuralSymbol;

    fn atom(id: u32) -> StructuralTerm {
        StructuralTerm::atom(StructuralSymbol::constructor(10_000 + id))
    }

    fn app(id: u32, arguments: Vec<StructuralTerm>) -> StructuralTerm {
        StructuralTerm::application(StructuralSymbol::constructor(id), arguments).expect("term")
    }

    #[test]
    fn identical_terms_remain_exact_without_witness_variables() {
        let term = app(1, vec![atom(2), atom(3)]);
        let result = anti_unify(&term, &term).expect("lgg");
        assert_eq!(result.generalization(), &term);
        assert!(result.left_substitution().is_empty());
        assert!(result.right_substitution().is_empty());
        assert_eq!(result.reconstruct_left().expect("left"), term);
    }

    #[test]
    fn same_head_recurses_and_witnesses_reconstruct_both_inputs() {
        let left = app(7, vec![atom(1), atom(9)]);
        let right = app(7, vec![atom(1), atom(4)]);
        let result = anti_unify(&left, &right).expect("lgg");
        assert_eq!(result.left_substitution().len(), 1);
        assert_eq!(result.right_substitution().len(), 1);
        assert_eq!(result.reconstruct_left().expect("left"), left);
        assert_eq!(result.reconstruct_right().expect("right"), right);
        assert_eq!(
            result.generalization(),
            &app(
                7,
                vec![
                    atom(1),
                    StructuralTerm::variable(StructuralVariableId::new(0)),
                ],
            )
        );
    }

    #[test]
    fn repeated_ordered_mismatch_reuses_one_variable() {
        let left = app(8, vec![atom(1), atom(1)]);
        let right = app(8, vec![atom(2), atom(2)]);
        let result = anti_unify(&left, &right).expect("lgg");
        let variable = StructuralTerm::variable(StructuralVariableId::new(0));
        assert_eq!(
            result.generalization(),
            &app(8, vec![variable.clone(), variable])
        );
        assert_eq!(result.left_substitution().len(), 1);
        assert_eq!(result.reconstruct_left().expect("left"), left);
        assert_eq!(result.reconstruct_right().expect("right"), right);
    }

    #[test]
    fn reversed_mismatch_pairs_remain_distinct() {
        let left = app(8, vec![atom(1), atom(2)]);
        let right = app(8, vec![atom(2), atom(1)]);
        let result = anti_unify(&left, &right).expect("lgg");
        assert_eq!(result.left_substitution().len(), 2);
        assert_eq!(
            result.generalization(),
            &app(
                8,
                vec![
                    StructuralTerm::variable(StructuralVariableId::new(0)),
                    StructuralTerm::variable(StructuralVariableId::new(1)),
                ],
            )
        );
        assert_eq!(result.reconstruct_left().expect("left"), left);
        assert_eq!(result.reconstruct_right().expect("right"), right);
    }

    #[test]
    fn generated_variables_are_fresh_relative_to_input_variables() {
        let retained = StructuralTerm::variable(StructuralVariableId::new(7));
        let left = app(3, vec![retained.clone(), atom(1)]);
        let right = app(3, vec![retained.clone(), atom(2)]);
        let result = anti_unify(&left, &right).expect("lgg");
        assert_eq!(
            result.generalization(),
            &app(
                3,
                vec![
                    retained,
                    StructuralTerm::variable(StructuralVariableId::new(8)),
                ],
            )
        );
        assert_eq!(result.reconstruct_left().expect("left"), left);
        assert_eq!(result.reconstruct_right().expect("right"), right);
    }

    #[test]
    fn exhausted_variable_namespace_fails_closed_when_generalization_needs_one() {
        let maximum = StructuralTerm::variable(StructuralVariableId::new(u32::MAX));
        let left = app(5, vec![maximum.clone(), atom(1)]);
        let right = app(5, vec![maximum, atom(2)]);
        assert_eq!(
            anti_unify(&left, &right),
            Err(AntiUnificationError::VariableIdExhausted)
        );
    }

    #[test]
    fn canonical_record_is_repeatable() {
        let left = app(9, vec![atom(1), atom(2)]);
        let right = app(9, vec![atom(1), atom(3)]);
        let first = anti_unify(&left, &right).expect("first");
        let second = anti_unify(&left, &right).expect("second");
        assert_eq!(first, second);
        assert_eq!(first.canonical_record(), second.canonical_record());
    }
}
