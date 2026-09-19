//! Bounded first-order structural term IR for TDI-2.2 template induction.
//!
//! Terms are derived only from observable relational structure. Concrete entity
//! identifiers remain episode-local constants; variables are explicit induction
//! objects and are never inferred from labels, outcomes, evaluator state, or
//! latency. The representation is deliberately small and dependency-free so the
//! next anti-unification slices can use an exact oracle over a frozen syntax.

use super::tdi2_intuition::PredicateId;
use super::tdi2_observation_graph::{
    ObservationGraph, ObservedEntityId, ObservedRelation, ObservedRelationId,
};

/// Versioned syntax identity for structural terms emitted by this slice.
pub const STRUCTURAL_TERM_SCHEMA: &str = "tdi2.2-structural-term-v1";
/// Maximum recursive term depth, counting a leaf as depth one.
pub const MAX_STRUCTURAL_TERM_DEPTH: usize = 32;
/// Maximum direct arguments accepted by one application node.
pub const MAX_STRUCTURAL_TERM_ARITY: usize = 64;
/// Maximum total nodes accepted by one structural term.
pub const MAX_STRUCTURAL_TERM_NODES: usize = 4_096;
/// Maximum canonical facts derived from one observation graph.
pub const MAX_STRUCTURAL_FACTS: usize = 16_384;

/// Stable namespace for first-order symbols.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StructuralSymbolNamespace {
    Entity,
    Predicate,
    Relation,
    Constructor,
}

impl StructuralSymbolNamespace {
    const fn key(self) -> &'static str {
        match self {
            Self::Entity => "entity",
            Self::Predicate => "predicate",
            Self::Relation => "relation",
            Self::Constructor => "constructor",
        }
    }
}

/// Stable first-order symbol identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StructuralSymbol {
    namespace: StructuralSymbolNamespace,
    id: u32,
}

impl StructuralSymbol {
    #[must_use]
    pub const fn new(namespace: StructuralSymbolNamespace, id: u32) -> Self {
        Self { namespace, id }
    }

    #[must_use]
    pub const fn entity(id: ObservedEntityId) -> Self {
        Self::new(StructuralSymbolNamespace::Entity, id.raw())
    }

    #[must_use]
    pub const fn predicate(id: PredicateId) -> Self {
        Self::new(StructuralSymbolNamespace::Predicate, id.raw())
    }

    #[must_use]
    pub const fn relation(id: ObservedRelationId) -> Self {
        Self::new(StructuralSymbolNamespace::Relation, id.raw())
    }

    #[must_use]
    pub const fn constructor(id: u32) -> Self {
        Self::new(StructuralSymbolNamespace::Constructor, id)
    }

    #[must_use]
    pub const fn namespace(self) -> StructuralSymbolNamespace {
        self.namespace
    }

    #[must_use]
    pub const fn id(self) -> u32 {
        self.id
    }
}

/// Explicit variable identity used by later anti-unification.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StructuralVariableId(u32);

impl StructuralVariableId {
    #[must_use]
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// One bounded first-order term.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StructuralTerm {
    Variable(StructuralVariableId),
    Atom(StructuralSymbol),
    Application {
        symbol: StructuralSymbol,
        arguments: Vec<Self>,
    },
}

/// Fail-closed syntax/derivation errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StructuralTermError {
    EmptyApplication,
    ArityLimitExceeded { maximum: usize },
    DepthLimitExceeded { maximum: usize },
    NodeLimitExceeded { maximum: usize },
    FactLimitExceeded { maximum: usize },
}

impl StructuralTerm {
    #[must_use]
    pub const fn variable(id: StructuralVariableId) -> Self {
        Self::Variable(id)
    }

    #[must_use]
    pub const fn atom(symbol: StructuralSymbol) -> Self {
        Self::Atom(symbol)
    }

    /// Construct and validate an application. Zero-arity constants must use
    /// [`Self::Atom`] so one semantic term has one canonical representation.
    pub fn application(
        symbol: StructuralSymbol,
        arguments: Vec<Self>,
    ) -> Result<Self, StructuralTermError> {
        if arguments.is_empty() {
            return Err(StructuralTermError::EmptyApplication);
        }
        if arguments.len() > MAX_STRUCTURAL_TERM_ARITY {
            return Err(StructuralTermError::ArityLimitExceeded {
                maximum: MAX_STRUCTURAL_TERM_ARITY,
            });
        }
        let term = Self::Application { symbol, arguments };
        term.validate()?;
        Ok(term)
    }

    /// Validate all public structural bounds.
    pub fn validate(&self) -> Result<(), StructuralTermError> {
        let (nodes, depth) = self.stats()?;
        if nodes > MAX_STRUCTURAL_TERM_NODES {
            return Err(StructuralTermError::NodeLimitExceeded {
                maximum: MAX_STRUCTURAL_TERM_NODES,
            });
        }
        if depth > MAX_STRUCTURAL_TERM_DEPTH {
            return Err(StructuralTermError::DepthLimitExceeded {
                maximum: MAX_STRUCTURAL_TERM_DEPTH,
            });
        }
        Ok(())
    }

    fn stats(&self) -> Result<(usize, usize), StructuralTermError> {
        match self {
            Self::Variable(_) | Self::Atom(_) => Ok((1, 1)),
            Self::Application { arguments, .. } => {
                if arguments.is_empty() {
                    return Err(StructuralTermError::EmptyApplication);
                }
                if arguments.len() > MAX_STRUCTURAL_TERM_ARITY {
                    return Err(StructuralTermError::ArityLimitExceeded {
                        maximum: MAX_STRUCTURAL_TERM_ARITY,
                    });
                }
                let mut nodes = 1usize;
                let mut child_depth = 0usize;
                for argument in arguments {
                    let (argument_nodes, argument_depth) = argument.stats()?;
                    nodes = nodes.checked_add(argument_nodes).ok_or(
                        StructuralTermError::NodeLimitExceeded {
                            maximum: MAX_STRUCTURAL_TERM_NODES,
                        },
                    )?;
                    if nodes > MAX_STRUCTURAL_TERM_NODES {
                        return Err(StructuralTermError::NodeLimitExceeded {
                            maximum: MAX_STRUCTURAL_TERM_NODES,
                        });
                    }
                    child_depth = child_depth.max(argument_depth);
                }
                let depth = child_depth + 1;
                if depth > MAX_STRUCTURAL_TERM_DEPTH {
                    return Err(StructuralTermError::DepthLimitExceeded {
                        maximum: MAX_STRUCTURAL_TERM_DEPTH,
                    });
                }
                Ok((nodes, depth))
            }
        }
    }

    /// Deterministic, unambiguous record for manifests, diffs and exact tests.
    #[must_use]
    pub fn canonical_record(&self) -> String {
        let mut output = String::from(STRUCTURAL_TERM_SCHEMA);
        output.push(':');
        self.write_canonical(&mut output);
        output
    }

    fn write_canonical(&self, output: &mut String) {
        use core::fmt::Write as _;
        match self {
            Self::Variable(id) => {
                write!(output, "v{};", id.raw()).expect("write to string");
            }
            Self::Atom(symbol) => {
                write!(output, "a:{}:{};", symbol.namespace().key(), symbol.id())
                    .expect("write to string");
            }
            Self::Application { symbol, arguments } => {
                write!(
                    output,
                    "f:{}:{}:{}[",
                    symbol.namespace().key(),
                    symbol.id(),
                    arguments.len()
                )
                .expect("write to string");
                for argument in arguments {
                    argument.write_canonical(output);
                }
                output.push_str("];");
            }
        }
    }
}

/// Convert one concrete relation observation into a binary first-order fact.
pub fn relation_fact(relation: ObservedRelation) -> Result<StructuralTerm, StructuralTermError> {
    StructuralTerm::application(
        StructuralSymbol::relation(relation.relation()),
        vec![
            StructuralTerm::atom(StructuralSymbol::entity(relation.left())),
            StructuralTerm::atom(StructuralSymbol::entity(relation.right())),
        ],
    )
}

/// Convert one unary predicate observation into a first-order fact.
pub fn predicate_fact(
    entity: ObservedEntityId,
    predicate: PredicateId,
) -> Result<StructuralTerm, StructuralTermError> {
    StructuralTerm::application(
        StructuralSymbol::predicate(predicate),
        vec![StructuralTerm::atom(StructuralSymbol::entity(entity))],
    )
}

/// Derive a canonical set of structural facts from one label-free graph.
///
/// Concrete entity identifiers stay local constants at this stage. Introducing
/// recurring role variables is owned by later slices and cannot happen here.
pub fn structural_facts_from_graph(
    graph: &ObservationGraph,
) -> Result<Vec<StructuralTerm>, StructuralTermError> {
    let predicate_count = graph
        .entities()
        .iter()
        .try_fold(0usize, |count, entity| {
            count.checked_add(entity.predicates().len())
        })
        .ok_or(StructuralTermError::FactLimitExceeded {
            maximum: MAX_STRUCTURAL_FACTS,
        })?;
    let count = predicate_count.checked_add(graph.relations().len()).ok_or(
        StructuralTermError::FactLimitExceeded {
            maximum: MAX_STRUCTURAL_FACTS,
        },
    )?;
    if count > MAX_STRUCTURAL_FACTS {
        return Err(StructuralTermError::FactLimitExceeded {
            maximum: MAX_STRUCTURAL_FACTS,
        });
    }

    let mut facts = Vec::with_capacity(count);
    for entity in graph.entities() {
        for &predicate in entity.predicates().predicates() {
            facts.push(predicate_fact(entity.id(), predicate)?);
        }
    }
    for &relation in graph.relations() {
        facts.push(relation_fact(relation)?);
    }
    facts.sort_unstable();
    facts.dedup();
    Ok(facts)
}

impl core::fmt::Display for StructuralTermError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyApplication => {
                formatter.write_str("structural application must have arguments")
            }
            Self::ArityLimitExceeded { maximum } => {
                write!(formatter, "structural arity exceeds {maximum}")
            }
            Self::DepthLimitExceeded { maximum } => {
                write!(formatter, "structural depth exceeds {maximum}")
            }
            Self::NodeLimitExceeded { maximum } => {
                write!(formatter, "structural node count exceeds {maximum}")
            }
            Self::FactLimitExceeded { maximum } => {
                write!(formatter, "structural fact count exceeds {maximum}")
            }
        }
    }
}

impl std::error::Error for StructuralTermError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::experimental::tdi2_intuition::BooleanState;
    use crate::experimental::tdi2_observation_graph::ObservedEntity;
    use crate::experimental::tdi2_template_induction::EpisodeId;

    fn graph() -> ObservationGraph {
        ObservationGraph::new(
            EpisodeId::new(9),
            vec![
                ObservedEntity::new(
                    ObservedEntityId::new(2),
                    BooleanState::new(vec![PredicateId::new(7)]),
                ),
                ObservedEntity::new(
                    ObservedEntityId::new(1),
                    BooleanState::new(vec![PredicateId::new(3), PredicateId::new(2)]),
                ),
            ],
            vec![ObservedRelation::new(
                ObservedEntityId::new(1),
                ObservedRelationId::new(5),
                ObservedEntityId::new(2),
            )],
        )
        .expect("graph")
    }

    #[test]
    fn graph_derivation_is_canonical_and_label_free() {
        let facts = structural_facts_from_graph(&graph()).expect("facts");
        assert_eq!(facts.len(), 4);
        let records: Vec<_> = facts.iter().map(StructuralTerm::canonical_record).collect();
        assert!(
            records
                .iter()
                .any(|record| record.contains("f:relation:5:2["))
        );
        assert!(
            records
                .iter()
                .any(|record| record.contains("f:predicate:2:1["))
        );
        assert!(records.iter().any(|record| record.contains("a:entity:1;")));
    }

    #[test]
    fn variables_constants_and_applications_have_distinct_exact_records() {
        let variable = StructuralTerm::variable(StructuralVariableId::new(4));
        let atom = StructuralTerm::atom(StructuralSymbol::constructor(4));
        let application =
            StructuralTerm::application(StructuralSymbol::constructor(4), vec![atom.clone()])
                .expect("application");
        assert_ne!(variable.canonical_record(), atom.canonical_record());
        assert_ne!(atom.canonical_record(), application.canonical_record());
        assert_ne!(variable.canonical_record(), application.canonical_record());
    }

    #[test]
    fn zero_arity_and_oversized_arity_fail_closed() {
        assert_eq!(
            StructuralTerm::application(StructuralSymbol::constructor(1), Vec::new()),
            Err(StructuralTermError::EmptyApplication)
        );
        let arguments = vec![
            StructuralTerm::atom(StructuralSymbol::constructor(2));
            MAX_STRUCTURAL_TERM_ARITY + 1
        ];
        assert_eq!(
            StructuralTerm::application(StructuralSymbol::constructor(1), arguments),
            Err(StructuralTermError::ArityLimitExceeded {
                maximum: MAX_STRUCTURAL_TERM_ARITY
            })
        );
    }

    #[test]
    fn recursive_depth_is_bounded_before_induction() {
        let mut term = StructuralTerm::atom(StructuralSymbol::constructor(0));
        for level in 1..MAX_STRUCTURAL_TERM_DEPTH {
            term = StructuralTerm::application(
                StructuralSymbol::constructor(level as u32),
                vec![term],
            )
            .expect("within depth bound");
        }
        assert_eq!(
            StructuralTerm::application(StructuralSymbol::constructor(99), vec![term]),
            Err(StructuralTermError::DepthLimitExceeded {
                maximum: MAX_STRUCTURAL_TERM_DEPTH
            })
        );
    }
}
