//! TDI-26 Stage-0 BANC v888 sparse-recurrent research contract.
//!
//! Stage 0 defines only source identity, topology-control identities, matched
//! resource accounting, and future intervention vocabulary. It does not run a
//! scientific comparison and does not authorize confirmatory execution.

use core::fmt;

/// Versioned TDI-26 Stage-0 contract identifier.
pub const TDI26_STAGE0_CONTRACT: &str = "tdi26-v888-stage0-v1";

/// The only connectome dataset authorized by this research line.
pub const TDI26_DATASET: &str = "banc";

/// The only materialization authorized by this research line.
pub const TDI26_MATERIALIZATION: u32 = 888;

/// Confirmatory execution is deliberately disabled at Stage 0.
pub const TDI26_CONFIRMATORY_AUTHORIZED: bool = false;

/// Ordered topology arms used by the future matched-control programme.
///
/// No arm carries a scientific result. These are identities for later
/// preregistration and resource-matching only.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TopologyArm {
    DenseReference,
    RandomSparse,
    DegreeMatched,
    DegreeReciprocityMatched,
    DegreeReciprocityModularityMatched,
    V888StructuralPrior,
}

impl TopologyArm {
    /// Stable machine-readable identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DenseReference => "c0_dense_reference",
            Self::RandomSparse => "c1_random_sparse",
            Self::DegreeMatched => "c2_degree_matched",
            Self::DegreeReciprocityMatched => "c3_degree_reciprocity_matched",
            Self::DegreeReciprocityModularityMatched => "c4_degree_reciprocity_modularity_matched",
            Self::V888StructuralPrior => "c5_v888_structural_prior",
        }
    }

    /// Whether this arm is a sparse matched-budget arm.
    #[must_use]
    pub const fn is_sparse(self) -> bool {
        !matches!(self, Self::DenseReference)
    }
}

/// Frozen Stage-0 arm order.
///
/// The stronger controls are explicit and cannot be silently omitted from a
/// later preregistration.
#[must_use]
pub const fn stage0_topology_arms() -> [TopologyArm; 6] {
    [
        TopologyArm::DenseReference,
        TopologyArm::RandomSparse,
        TopologyArm::DegreeMatched,
        TopologyArm::DegreeReciprocityMatched,
        TopologyArm::DegreeReciprocityModularityMatched,
        TopologyArm::V888StructuralPrior,
    ]
}

/// Exact logical resource budget used to reject unmatched topology comparisons.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TopologyBudget {
    pub nodes: u32,
    pub directed_edges: u64,
    pub dynamic_memory_bits: u64,
}

impl TopologyBudget {
    /// Construct a fail-closed budget for simple directed graphs without autapses.
    pub fn new(
        nodes: u32,
        directed_edges: u64,
        dynamic_memory_bits: u64,
    ) -> Result<Self, Tdi26Error> {
        if nodes < 2 {
            return Err(Tdi26Error::TooFewNodes);
        }
        if directed_edges == 0 {
            return Err(Tdi26Error::ZeroEdges);
        }
        let max_edges = u64::from(nodes)
            .checked_mul(u64::from(nodes - 1))
            .ok_or(Tdi26Error::BudgetOverflow)?;
        if directed_edges > max_edges {
            return Err(Tdi26Error::TooManyDirectedEdges {
                directed_edges,
                max_edges,
            });
        }
        Ok(Self {
            nodes,
            directed_edges,
            dynamic_memory_bits,
        })
    }
}

/// One arm plus the exact resource budget charged to it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BudgetedTopologyArm {
    pub arm: TopologyArm,
    pub budget: TopologyBudget,
}

/// Validate that all sparse arms in a declared comparison use exactly the same
/// node, edge, and dynamic-memory budgets.
///
/// The dense reference is intentionally excluded from this equality rule because
/// it is a contextual full-connectivity control rather than a sparse matched arm.
pub fn validate_sparse_matched_budgets(
    arms: &[BudgetedTopologyArm],
) -> Result<TopologyBudget, Tdi26Error> {
    let mut expected = None;
    let mut seen_v888 = false;
    let mut seen_random = false;
    let mut seen_degree = false;
    let mut seen_reciprocity = false;
    let mut seen_modularity = false;

    for entry in arms {
        match entry.arm {
            TopologyArm::DenseReference => continue,
            TopologyArm::RandomSparse => seen_random = true,
            TopologyArm::DegreeMatched => seen_degree = true,
            TopologyArm::DegreeReciprocityMatched => seen_reciprocity = true,
            TopologyArm::DegreeReciprocityModularityMatched => seen_modularity = true,
            TopologyArm::V888StructuralPrior => seen_v888 = true,
        }

        match expected {
            None => expected = Some(entry.budget),
            Some(reference) if reference == entry.budget => {}
            Some(reference) => {
                return Err(Tdi26Error::UnmatchedSparseBudget {
                    expected: reference,
                    found: entry.budget,
                    arm: entry.arm,
                });
            }
        }
    }

    if !(seen_v888 && seen_random && seen_degree && seen_reciprocity && seen_modularity) {
        return Err(Tdi26Error::MissingRequiredControl);
    }

    expected.ok_or(Tdi26Error::MissingRequiredControl)
}

/// Future intervention identities admitted by the Stage-0 scope.
///
/// Their presence here authorizes vocabulary and accounting only, not execution
/// on a final or confirmatory population.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InterventionKind {
    RandomNodeSilencing,
    RandomEdgeRemoval,
    TargetedHighDegreeNodeSilencing,
    InterModuleCut,
    InputPerturbation,
    StatePerturbation,
}

impl InterventionKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RandomNodeSilencing => "random_node_silencing",
            Self::RandomEdgeRemoval => "random_edge_removal",
            Self::TargetedHighDegreeNodeSilencing => "targeted_high_degree_node_silencing",
            Self::InterModuleCut => "inter_module_cut",
            Self::InputPerturbation => "input_perturbation",
            Self::StatePerturbation => "state_perturbation",
        }
    }
}

/// Immutable Stage-0 identity record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stage0Identity {
    pub contract: &'static str,
    pub dataset: &'static str,
    pub materialization: u32,
    pub confirmatory_authorized: bool,
}

/// Return the only Stage-0 identity accepted by TDI-26.
#[must_use]
pub const fn stage0_identity() -> Stage0Identity {
    Stage0Identity {
        contract: TDI26_STAGE0_CONTRACT,
        dataset: TDI26_DATASET,
        materialization: TDI26_MATERIALIZATION,
        confirmatory_authorized: TDI26_CONFIRMATORY_AUTHORIZED,
    }
}

/// TDI-26 Stage-0 validation failures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tdi26Error {
    TooFewNodes,
    ZeroEdges,
    BudgetOverflow,
    TooManyDirectedEdges {
        directed_edges: u64,
        max_edges: u64,
    },
    MissingRequiredControl,
    UnmatchedSparseBudget {
        expected: TopologyBudget,
        found: TopologyBudget,
        arm: TopologyArm,
    },
}

impl fmt::Display for Tdi26Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooFewNodes => formatter.write_str("TDI-26 requires at least two nodes"),
            Self::ZeroEdges => {
                formatter.write_str("TDI-26 sparse budget requires at least one edge")
            }
            Self::BudgetOverflow => formatter.write_str("TDI-26 budget arithmetic overflow"),
            Self::TooManyDirectedEdges {
                directed_edges,
                max_edges,
            } => write!(
                formatter,
                "TDI-26 directed edge budget {directed_edges} exceeds simple-graph maximum {max_edges}"
            ),
            Self::MissingRequiredControl => formatter
                .write_str("TDI-26 matched comparison is missing a required sparse control"),
            Self::UnmatchedSparseBudget {
                expected,
                found,
                arm,
            } => write!(
                formatter,
                "TDI-26 arm {} has unmatched sparse budget: expected {:?}, found {:?}",
                arm.as_str(),
                expected,
                found
            ),
        }
    }
}

impl std::error::Error for Tdi26Error {}

#[cfg(test)]
mod tests {
    use super::*;

    fn budget() -> TopologyBudget {
        TopologyBudget::new(128, 512, 16_384).unwrap()
    }

    #[test]
    fn stage0_identity_is_v888_only_and_non_confirmatory() {
        let identity = stage0_identity();
        assert_eq!(identity.contract, TDI26_STAGE0_CONTRACT);
        assert_eq!(identity.dataset, "banc");
        assert_eq!(identity.materialization, 888);
        assert!(!identity.confirmatory_authorized);
    }

    #[test]
    fn topology_arm_order_preserves_strong_controls() {
        assert_eq!(
            stage0_topology_arms(),
            [
                TopologyArm::DenseReference,
                TopologyArm::RandomSparse,
                TopologyArm::DegreeMatched,
                TopologyArm::DegreeReciprocityMatched,
                TopologyArm::DegreeReciprocityModularityMatched,
                TopologyArm::V888StructuralPrior,
            ]
        );
    }

    #[test]
    fn valid_sparse_panel_requires_all_controls_and_equal_budgets() {
        let panel = [
            BudgetedTopologyArm {
                arm: TopologyArm::RandomSparse,
                budget: budget(),
            },
            BudgetedTopologyArm {
                arm: TopologyArm::DegreeMatched,
                budget: budget(),
            },
            BudgetedTopologyArm {
                arm: TopologyArm::DegreeReciprocityMatched,
                budget: budget(),
            },
            BudgetedTopologyArm {
                arm: TopologyArm::DegreeReciprocityModularityMatched,
                budget: budget(),
            },
            BudgetedTopologyArm {
                arm: TopologyArm::V888StructuralPrior,
                budget: budget(),
            },
        ];
        assert_eq!(validate_sparse_matched_budgets(&panel).unwrap(), budget());
    }

    #[test]
    fn missing_control_fails_closed() {
        let panel = [
            BudgetedTopologyArm {
                arm: TopologyArm::RandomSparse,
                budget: budget(),
            },
            BudgetedTopologyArm {
                arm: TopologyArm::V888StructuralPrior,
                budget: budget(),
            },
        ];
        assert_eq!(
            validate_sparse_matched_budgets(&panel),
            Err(Tdi26Error::MissingRequiredControl)
        );
    }

    #[test]
    fn unmatched_sparse_budget_fails_closed() {
        let different = TopologyBudget::new(128, 513, 16_384).unwrap();
        let panel = [
            BudgetedTopologyArm {
                arm: TopologyArm::RandomSparse,
                budget: budget(),
            },
            BudgetedTopologyArm {
                arm: TopologyArm::DegreeMatched,
                budget: budget(),
            },
            BudgetedTopologyArm {
                arm: TopologyArm::DegreeReciprocityMatched,
                budget: budget(),
            },
            BudgetedTopologyArm {
                arm: TopologyArm::DegreeReciprocityModularityMatched,
                budget: different,
            },
            BudgetedTopologyArm {
                arm: TopologyArm::V888StructuralPrior,
                budget: budget(),
            },
        ];
        assert!(matches!(
            validate_sparse_matched_budgets(&panel),
            Err(Tdi26Error::UnmatchedSparseBudget {
                arm: TopologyArm::DegreeReciprocityModularityMatched,
                ..
            })
        ));
    }

    #[test]
    fn impossible_simple_graph_budget_is_rejected() {
        assert_eq!(TopologyBudget::new(1, 1, 0), Err(Tdi26Error::TooFewNodes));
        assert_eq!(TopologyBudget::new(4, 0, 0), Err(Tdi26Error::ZeroEdges));
        assert!(matches!(
            TopologyBudget::new(4, 13, 0),
            Err(Tdi26Error::TooManyDirectedEdges {
                directed_edges: 13,
                max_edges: 12,
            })
        ));
    }

    #[test]
    fn intervention_names_are_stable() {
        assert_eq!(
            InterventionKind::TargetedHighDegreeNodeSilencing.as_str(),
            "targeted_high_degree_node_silencing"
        );
        assert_eq!(
            InterventionKind::InterModuleCut.as_str(),
            "inter_module_cut"
        );
    }
}
