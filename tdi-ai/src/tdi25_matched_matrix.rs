//! Fail-closed capability matrix for the TDI-25 primary T6/C6 comparison.
//!
//! This module records only evaluator paths that are reachable from sealed
//! production constructors. It deliberately does not invent carrier
//! conversions, relabel task families, or allow the secondary G6 control to
//! satisfy a missing primary path.

use core::fmt;

use super::tdi25_eval::{
    C6_EVALUATOR_CONTRACT, G6_EVALUATOR_CONTRACT, T6_EVALUATOR_CONTRACT,
};
use super::tdi25_tasks::{
    CHIRAL_REFLECTION_TASK_CONTRACT, MIXED_GEOMETRY_TASK_CONTRACT,
    TORSOR_TRANSPORT_TASK_CONTRACT,
};
use super::tdi25_torsor_chiral::TaskFamily;

/// Versioned contract for the sealed primary evaluator capability matrix.
pub const MATCHED_EVALUATOR_MATRIX_CONTRACT: &str = "tdi25-matched-evaluator-matrix-v1";

/// Primary arm identity. G6 is intentionally absent because it is a secondary
/// attribution control, not a substitute for T6 or C6.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimaryArm {
    T6,
    C6,
}

/// One production-reachable primary evaluator path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PrimaryEvaluatorPath {
    pub family: TaskFamily,
    pub arm: PrimaryArm,
    pub task_contract: &'static str,
    pub evaluator_contract: &'static str,
    pub matrix_contract: &'static str,
}

/// Closed family order required by the Stage-C primary comparison.
pub const REQUIRED_PRIMARY_FAMILIES: [TaskFamily; 4] = [
    TaskFamily::TorsorFavorable,
    TaskFamily::ChiralFavorable,
    TaskFamily::Mixed,
    TaskFamily::Neutral,
];

/// Exact sealed paths reachable on the current default-branch evaluator API.
///
/// T6 reaches torsor-favourable and mixed inputs. C6 reaches
/// chiral-favourable and mixed inputs. Neutral is currently G6-only and G6 is
/// not included here.
pub const CURRENT_PRIMARY_PATHS: [PrimaryEvaluatorPath; 4] = [
    PrimaryEvaluatorPath {
        family: TaskFamily::TorsorFavorable,
        arm: PrimaryArm::T6,
        task_contract: TORSOR_TRANSPORT_TASK_CONTRACT,
        evaluator_contract: T6_EVALUATOR_CONTRACT,
        matrix_contract: MATCHED_EVALUATOR_MATRIX_CONTRACT,
    },
    PrimaryEvaluatorPath {
        family: TaskFamily::Mixed,
        arm: PrimaryArm::T6,
        task_contract: MIXED_GEOMETRY_TASK_CONTRACT,
        evaluator_contract: T6_EVALUATOR_CONTRACT,
        matrix_contract: MATCHED_EVALUATOR_MATRIX_CONTRACT,
    },
    PrimaryEvaluatorPath {
        family: TaskFamily::ChiralFavorable,
        arm: PrimaryArm::C6,
        task_contract: CHIRAL_REFLECTION_TASK_CONTRACT,
        evaluator_contract: C6_EVALUATOR_CONTRACT,
        matrix_contract: MATCHED_EVALUATOR_MATRIX_CONTRACT,
    },
    PrimaryEvaluatorPath {
        family: TaskFamily::Mixed,
        arm: PrimaryArm::C6,
        task_contract: MIXED_GEOMETRY_TASK_CONTRACT,
        evaluator_contract: C6_EVALUATOR_CONTRACT,
        matrix_contract: MATCHED_EVALUATOR_MATRIX_CONTRACT,
    },
];

/// Paired capability view for one declared task family.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MatchedFamilyPaths {
    pub family: TaskFamily,
    pub t6: Option<PrimaryEvaluatorPath>,
    pub c6: Option<PrimaryEvaluatorPath>,
    pub matrix_contract: &'static str,
}

impl MatchedFamilyPaths {
    /// True only when both primary arms have sealed paths for this family.
    #[must_use]
    pub const fn is_complete(self) -> bool {
        self.t6.is_some() && self.c6.is_some()
    }
}

/// Fail-closed matrix admission errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatchedMatrixError {
    /// One declared family lacks a sealed path for one primary arm.
    MissingPrimaryPath {
        family: TaskFamily,
        arm: PrimaryArm,
    },
    /// A G6 path was proposed as a primary T6/C6 replacement.
    SecondaryControlCannotSatisfyPrimary {
        family: TaskFamily,
        evaluator_contract: &'static str,
    },
}

impl fmt::Display for MatchedMatrixError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingPrimaryPath { family, arm } => write!(
                formatter,
                "missing sealed primary evaluator path for family {family:?}, arm {arm:?}"
            ),
            Self::SecondaryControlCannotSatisfyPrimary {
                family,
                evaluator_contract,
            } => write!(
                formatter,
                "secondary control {evaluator_contract} cannot satisfy a primary path for {family:?}"
            ),
        }
    }
}

impl std::error::Error for MatchedMatrixError {}

/// Look up one exact production-reachable primary path.
#[must_use]
pub fn primary_path(family: TaskFamily, arm: PrimaryArm) -> Option<PrimaryEvaluatorPath> {
    CURRENT_PRIMARY_PATHS
        .iter()
        .copied()
        .find(|path| path.family == family && path.arm == arm)
}

/// Return the current paired capability view for one family.
#[must_use]
pub fn matched_family_paths(family: TaskFamily) -> MatchedFamilyPaths {
    MatchedFamilyPaths {
        family,
        t6: primary_path(family, PrimaryArm::T6),
        c6: primary_path(family, PrimaryArm::C6),
        matrix_contract: MATCHED_EVALUATOR_MATRIX_CONTRACT,
    }
}

/// Require a complete four-family T6/C6 matrix.
///
/// The current matrix intentionally returns an error. Callers must not proceed
/// to four-family primary synthesis until real sealed paths have been added.
pub fn require_complete_primary_matrix(
) -> Result<[MatchedFamilyPaths; 4], MatchedMatrixError> {
    let families = REQUIRED_PRIMARY_FAMILIES.map(matched_family_paths);
    for paths in families {
        if paths.t6.is_none() {
            return Err(MatchedMatrixError::MissingPrimaryPath {
                family: paths.family,
                arm: PrimaryArm::T6,
            });
        }
        if paths.c6.is_none() {
            return Err(MatchedMatrixError::MissingPrimaryPath {
                family: paths.family,
                arm: PrimaryArm::C6,
            });
        }
    }
    Ok(families)
}

/// Produce the explicit error used when G6 is proposed as a primary substitute.
#[must_use]
pub const fn reject_g6_primary_substitution(family: TaskFamily) -> MatchedMatrixError {
    MatchedMatrixError::SecondaryControlCannotSatisfyPrimary {
        family,
        evaluator_contract: G6_EVALUATOR_CONTRACT,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_matrix_records_only_real_sealed_paths() {
        assert_eq!(CURRENT_PRIMARY_PATHS.len(), 4);
        assert_eq!(
            primary_path(TaskFamily::TorsorFavorable, PrimaryArm::T6)
                .unwrap()
                .task_contract,
            TORSOR_TRANSPORT_TASK_CONTRACT
        );
        assert_eq!(
            primary_path(TaskFamily::ChiralFavorable, PrimaryArm::C6)
                .unwrap()
                .task_contract,
            CHIRAL_REFLECTION_TASK_CONTRACT
        );
        assert!(matched_family_paths(TaskFamily::Mixed).is_complete());
    }

    #[test]
    fn asymmetric_and_neutral_primary_paths_are_absent() {
        assert!(primary_path(TaskFamily::TorsorFavorable, PrimaryArm::C6).is_none());
        assert!(primary_path(TaskFamily::ChiralFavorable, PrimaryArm::T6).is_none());
        assert!(primary_path(TaskFamily::Neutral, PrimaryArm::T6).is_none());
        assert!(primary_path(TaskFamily::Neutral, PrimaryArm::C6).is_none());
    }

    #[test]
    fn incomplete_matrix_fails_closed_before_synthesis() {
        assert_eq!(
            require_complete_primary_matrix(),
            Err(MatchedMatrixError::MissingPrimaryPath {
                family: TaskFamily::TorsorFavorable,
                arm: PrimaryArm::C6,
            })
        );
    }

    #[test]
    fn g6_cannot_replace_a_missing_primary_arm() {
        assert_eq!(
            reject_g6_primary_substitution(TaskFamily::Neutral),
            MatchedMatrixError::SecondaryControlCannotSatisfyPrimary {
                family: TaskFamily::Neutral,
                evaluator_contract: G6_EVALUATOR_CONTRACT,
            }
        );
    }

    #[test]
    fn every_path_binds_exact_contracts() {
        for path in CURRENT_PRIMARY_PATHS {
            assert_eq!(path.matrix_contract, MATCHED_EVALUATOR_MATRIX_CONTRACT);
            match path.arm {
                PrimaryArm::T6 => assert_eq!(path.evaluator_contract, T6_EVALUATOR_CONTRACT),
                PrimaryArm::C6 => assert_eq!(path.evaluator_contract, C6_EVALUATOR_CONTRACT),
            }
        }
    }
}
