//! Deterministic bounded multi-example anti-unification for TDI-2.2.
//!
//! Inputs are validated, bounded and canonically ordered before the exact
//! pairwise baseline is folded over them. The retained step witnesses make the
//! entire fold replayable without introducing labels, outcomes, evaluator
//! state, latency, or protected/final information.

use super::tdi2_anti_unification::{AntiUnificationError, AntiUnificationResult, anti_unify};
use super::tdi2_structural_terms::{StructuralTerm, StructuralTermError, StructuralTermKind};

/// Versioned identity for the incremental multi-example baseline.
pub const INCREMENTAL_ANTI_UNIFICATION_SCHEMA: &str = "tdi2.2-incremental-anti-unification-v2";
/// Maximum number of examples in one incremental anti-unification batch.
pub const MAX_INCREMENTAL_ANTI_UNIFICATION_TERMS: usize = 256;
/// Maximum aggregate structural nodes accepted across one input batch.
pub const MAX_INCREMENTAL_ANTI_UNIFICATION_INPUT_NODES: usize = 65_536;

/// Replayable multi-example anti-unification result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IncrementalAntiUnificationResult {
    canonical_inputs: Vec<StructuralTerm>,
    seed: StructuralTerm,
    steps: Vec<AntiUnificationResult>,
    generalization: StructuralTerm,
}

impl IncrementalAntiUnificationResult {
    #[must_use]
    pub fn generalization(&self) -> &StructuralTerm {
        &self.generalization
    }

    #[must_use]
    pub fn steps(&self) -> &[AntiUnificationResult] {
        &self.steps
    }

    #[must_use]
    pub fn input_count(&self) -> usize {
        self.canonical_inputs.len()
    }

    /// Recover the exact canonical input sequence from the retained witnesses.
    ///
    /// Duplicate inputs remain explicit evidence but do not create another fold
    /// step. Each distinct input after the seed must have exactly one executable
    /// witness, so neither missing nor surplus steps can be hidden.
    pub fn reconstruct_canonical_inputs(
        &self,
    ) -> Result<Vec<StructuralTerm>, IncrementalAntiUnificationError> {
        if self.canonical_inputs.first() != Some(&self.seed) {
            return Err(IncrementalAntiUnificationError::ReplayMismatch);
        }

        let mut current = self.seed.clone();
        let mut previous_input: Option<&StructuralTerm> = None;
        let mut steps = self.steps.iter();
        for input in &self.canonical_inputs {
            if previous_input.is_some_and(|previous| previous != input) {
                let step = steps
                    .next()
                    .ok_or(IncrementalAntiUnificationError::ReplayMismatch)?;
                let replay_left = step
                    .reconstruct_left()
                    .map_err(IncrementalAntiUnificationError::Pairwise)?;
                let replay_right = step
                    .reconstruct_right()
                    .map_err(IncrementalAntiUnificationError::Pairwise)?;
                if replay_left != current || replay_right != *input {
                    return Err(IncrementalAntiUnificationError::ReplayMismatch);
                }
                current = step.generalization().clone();
            }
            previous_input = Some(input);
        }

        if steps.next().is_some() || current != self.generalization {
            return Err(IncrementalAntiUnificationError::ReplayMismatch);
        }
        Ok(self.canonical_inputs.clone())
    }

    /// Deterministic record binding every input, the distinct fold and witnesses.
    #[must_use]
    pub fn canonical_record(&self) -> String {
        use core::fmt::Write as _;
        let seed = self.seed.canonical_record();
        let final_term = self.generalization.canonical_record();
        let mut output = format!(
            "{INCREMENTAL_ANTI_UNIFICATION_SCHEMA};inputs={}:[",
            self.canonical_inputs.len()
        );
        for input in &self.canonical_inputs {
            let record = input.canonical_record();
            write!(output, "{}:{};", record.len(), record).expect("write to string");
        }
        write!(
            output,
            "];seed={}:{};steps={}:[",
            seed.len(),
            seed,
            self.steps.len()
        )
        .expect("write to string");
        for step in &self.steps {
            let record = step.canonical_record();
            write!(output, "{}:{};", record.len(), record).expect("write to string");
        }
        write!(output, "];final={}:{};", final_term.len(), final_term).expect("write to string");
        output
    }
}

/// Fail-closed multi-example anti-unification errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IncrementalAntiUnificationError {
    EmptyInput,
    TermCountLimitExceeded {
        maximum: usize,
    },
    InputInvalid {
        index: usize,
        error: StructuralTermError,
    },
    AggregateNodeLimitExceeded {
        maximum: usize,
    },
    Pairwise(AntiUnificationError),
    ReplayMismatch,
}

/// Incrementally anti-unify a bounded set of structural examples.
///
/// Canonical sorting removes caller-order effects from this baseline while
/// retaining duplicates as explicit evidence entries. A one-term batch is the
/// identity case and therefore has no pairwise steps.
pub fn incremental_anti_unify(
    terms: &[StructuralTerm],
) -> Result<IncrementalAntiUnificationResult, IncrementalAntiUnificationError> {
    if terms.is_empty() {
        return Err(IncrementalAntiUnificationError::EmptyInput);
    }
    if terms.len() > MAX_INCREMENTAL_ANTI_UNIFICATION_TERMS {
        return Err(IncrementalAntiUnificationError::TermCountLimitExceeded {
            maximum: MAX_INCREMENTAL_ANTI_UNIFICATION_TERMS,
        });
    }

    let mut aggregate_nodes = 0usize;
    for (index, term) in terms.iter().enumerate() {
        term.validate()
            .map_err(|error| IncrementalAntiUnificationError::InputInvalid { index, error })?;
        aggregate_nodes = aggregate_nodes.checked_add(node_count(term)).ok_or(
            IncrementalAntiUnificationError::AggregateNodeLimitExceeded {
                maximum: MAX_INCREMENTAL_ANTI_UNIFICATION_INPUT_NODES,
            },
        )?;
        if aggregate_nodes > MAX_INCREMENTAL_ANTI_UNIFICATION_INPUT_NODES {
            return Err(
                IncrementalAntiUnificationError::AggregateNodeLimitExceeded {
                    maximum: MAX_INCREMENTAL_ANTI_UNIFICATION_INPUT_NODES,
                },
            );
        }
    }

    let mut ordered = terms.to_vec();
    ordered.sort_unstable();

    let seed = ordered[0].clone();
    let mut distinct_inputs = ordered.clone();
    distinct_inputs.dedup();

    let mut generalization = seed.clone();
    let mut steps = Vec::with_capacity(distinct_inputs.len().saturating_sub(1));
    for term in distinct_inputs.iter().skip(1) {
        let step =
            anti_unify(&generalization, term).map_err(IncrementalAntiUnificationError::Pairwise)?;
        generalization = step.generalization().clone();
        steps.push(step);
    }

    let result = IncrementalAntiUnificationResult {
        canonical_inputs: ordered.clone(),
        seed,
        steps,
        generalization,
    };
    if result.reconstruct_canonical_inputs()? != ordered {
        return Err(IncrementalAntiUnificationError::ReplayMismatch);
    }
    Ok(result)
}

fn node_count(term: &StructuralTerm) -> usize {
    match term.kind() {
        StructuralTermKind::Variable(_) | StructuralTermKind::Atom(_) => 1,
        StructuralTermKind::Application { arguments, .. } => {
            1 + arguments.iter().map(node_count).sum::<usize>()
        }
    }
}

impl core::fmt::Display for IncrementalAntiUnificationError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyInput => {
                formatter.write_str("incremental anti-unification requires at least one term")
            }
            Self::TermCountLimitExceeded { maximum } => {
                write!(
                    formatter,
                    "incremental anti-unification term count exceeds {maximum}"
                )
            }
            Self::InputInvalid { index, error } => {
                write!(formatter, "structural input {index} is invalid: {error}")
            }
            Self::AggregateNodeLimitExceeded { maximum } => {
                write!(
                    formatter,
                    "incremental anti-unification input nodes exceed {maximum}"
                )
            }
            Self::Pairwise(error) => write!(formatter, "pairwise anti-unification failed: {error}"),
            Self::ReplayMismatch => {
                formatter.write_str("incremental anti-unification replay mismatch")
            }
        }
    }
}

impl std::error::Error for IncrementalAntiUnificationError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::experimental::tdi2_structural_terms::{StructuralSymbol, StructuralVariableId};

    fn atom(id: u32) -> StructuralTerm {
        StructuralTerm::atom(StructuralSymbol::constructor(10_000 + id))
    }

    fn app(id: u32, arguments: Vec<StructuralTerm>) -> StructuralTerm {
        StructuralTerm::application(StructuralSymbol::constructor(id), arguments).expect("term")
    }

    #[test]
    fn one_example_is_the_identity_case() {
        let term = app(1, vec![atom(2)]);
        let result = incremental_anti_unify(core::slice::from_ref(&term)).expect("result");
        assert_eq!(result.generalization(), &term);
        assert!(result.steps().is_empty());
        assert_eq!(
            result.reconstruct_canonical_inputs().expect("replay"),
            vec![term]
        );
    }

    #[test]
    fn three_examples_generalize_incrementally_and_replay_exactly() {
        let terms = vec![
            app(7, vec![atom(1), atom(4)]),
            app(7, vec![atom(1), atom(2)]),
            app(7, vec![atom(1), atom(3)]),
        ];
        let result = incremental_anti_unify(&terms).expect("result");
        assert_eq!(result.input_count(), 3);
        assert_eq!(result.steps().len(), 2);
        assert_eq!(
            result.generalization(),
            &app(
                7,
                vec![
                    atom(1),
                    StructuralTerm::variable(StructuralVariableId::new(1)),
                ],
            )
        );
        let mut expected = terms;
        expected.sort_unstable();
        assert_eq!(
            result.reconstruct_canonical_inputs().expect("replay"),
            expected
        );
    }

    #[test]
    fn canonical_sorting_makes_input_permutation_irrelevant() {
        let a = app(2, vec![atom(8), atom(1)]);
        let b = app(2, vec![atom(8), atom(2)]);
        let c = app(2, vec![atom(8), atom(3)]);
        let first = incremental_anti_unify(&[a.clone(), b.clone(), c.clone()]).expect("first");
        let second = incremental_anti_unify(&[c, a, b]).expect("second");
        assert_eq!(first, second);
        assert_eq!(first.canonical_record(), second.canonical_record());
    }

    #[test]
    fn duplicates_are_retained_as_evidence_but_do_not_change_structure() {
        let a = app(3, vec![atom(1)]);
        let b = app(3, vec![atom(2)]);
        let unique = incremental_anti_unify(&[a.clone(), b.clone()]).expect("unique");
        let repeated = incremental_anti_unify(&[a.clone(), a, b]).expect("repeated");
        assert_eq!(unique.generalization(), repeated.generalization());
        assert_eq!(repeated.input_count(), 3);
        assert_eq!(unique.steps(), repeated.steps());
        assert_eq!(repeated.steps().len(), 1);
        assert_eq!(
            repeated
                .reconstruct_canonical_inputs()
                .expect("replay")
                .len(),
            3
        );
        assert_ne!(unique.canonical_record(), repeated.canonical_record());
    }

    #[test]
    fn duplicate_after_a_distinct_term_preserves_the_lgg_and_variable_budget() {
        let retained = StructuralTerm::variable(StructuralVariableId::new(u32::MAX - 1));
        let a = app(3, vec![retained.clone(), atom(1)]);
        let b = app(3, vec![retained, atom(2)]);

        let unique = incremental_anti_unify(&[a.clone(), b.clone()]).expect("unique");
        let repeated = incremental_anti_unify(&[a, b.clone(), b]).expect("repeated");

        assert_eq!(unique.generalization(), repeated.generalization());
        assert_eq!(unique.steps(), repeated.steps());
        assert_eq!(repeated.input_count(), 3);
        assert_eq!(
            repeated
                .reconstruct_canonical_inputs()
                .expect("replay")
                .len(),
            3
        );
    }

    #[test]
    fn empty_and_oversized_batches_fail_closed() {
        assert_eq!(
            incremental_anti_unify(&[]),
            Err(IncrementalAntiUnificationError::EmptyInput)
        );
        let terms = vec![atom(1); MAX_INCREMENTAL_ANTI_UNIFICATION_TERMS + 1];
        assert_eq!(
            incremental_anti_unify(&terms),
            Err(IncrementalAntiUnificationError::TermCountLimitExceeded {
                maximum: MAX_INCREMENTAL_ANTI_UNIFICATION_TERMS
            })
        );
    }

    #[test]
    fn aggregate_node_budget_is_checked_before_canonical_clone() {
        let child = app(5, vec![atom(1); 4]);
        let wide = app(4, vec![child; 64]);
        let terms = vec![wide; MAX_INCREMENTAL_ANTI_UNIFICATION_TERMS];
        assert_eq!(
            incremental_anti_unify(&terms),
            Err(
                IncrementalAntiUnificationError::AggregateNodeLimitExceeded {
                    maximum: MAX_INCREMENTAL_ANTI_UNIFICATION_INPUT_NODES
                }
            )
        );
    }
}
