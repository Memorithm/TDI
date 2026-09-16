//! Bounded local rewrite calculus for TDI-23.2.
//!
//! This module deliberately implements only four explicit local rules over the
//! existing TDI-23.1 typed IR: left-identity elimination, right-identity
//! elimination, dagger-of-identity elimination, and double-dagger elimination.
//!
//! Every candidate is independently validated and checked by the TDI-23.1 exact
//! equivalence oracle before it is accepted. There is no recursive rewriting,
//! no rewrite search, no reassociation, no tensor/direct-sum distribution, and
//! no nonlinear-boundary semantics in this slice.

use core::fmt;

use super::tdi23_ir::{CategoricalAttentionIr, IrError, IrNodeKind, NodeId};
use super::tdi23_ir_equivalence::{
    EquivalenceError, ExactLinearEquivalenceReport, compare_exact_linear_semantics,
};
use super::tdi23_ir_provenance::{ProvenanceError, validate_rooted_subgraph};

/// Versioned non-final TDI-23.2 local rewrite contract.
pub const LOCAL_REWRITE_CONTRACT: &str = "tdi23.2-local-rewrite-calculus-v1";

/// Local rewrite rules admitted by the first bounded TDI-23.2 slice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RewriteRule {
    /// `id_B o f -> f`.
    LeftIdentity,
    /// `f o id_A -> f`.
    RightIdentity,
    /// `id_A^dagger -> id_A`.
    DaggerIdentity,
    /// `(f^dagger)^dagger -> f`.
    DoubleDagger,
}

impl RewriteRule {
    /// Exactness guarantee required from the Stage-0 verifier.
    #[must_use]
    pub const fn exactness_class(self) -> RewriteExactnessClass {
        match self {
            Self::LeftIdentity | Self::RightIdentity => RewriteExactnessClass::FiniteValueExact,
            Self::DaggerIdentity | Self::DoubleDagger => RewriteExactnessClass::BitwiseExact,
        }
    }
}

impl fmt::Display for RewriteRule {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::LeftIdentity => "left-identity elimination",
            Self::RightIdentity => "right-identity elimination",
            Self::DaggerIdentity => "dagger-of-identity elimination",
            Self::DoubleDagger => "double-dagger elimination",
        })
    }
}

/// Exactness class required for one local rewrite.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RewriteExactnessClass {
    /// Every finite scalar compares equal under ordinary `f64` equality.
    ///
    /// This intentionally treats `+0.0` and `-0.0` as value-equal while still
    /// exposing their bit-level difference through the verification report.
    FiniteValueExact,
    /// Every scalar must retain the exact IEEE-754 bit pattern.
    BitwiseExact,
}

/// Fail-closed errors produced by the bounded local rewrite verifier.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RewriteError {
    /// Independent rooted validation failed before pattern matching.
    Provenance(ProvenanceError),
    /// Underlying typed-IR lookup failed.
    Ir(IrError),
    /// The requested rule did not match the root shape exactly.
    PatternMismatch {
        /// Requested rule.
        rule: RewriteRule,
        /// IR-local root index for diagnostics only.
        root_index: usize,
    },
    /// Exact equivalence validation failed closed.
    Equivalence(EquivalenceError),
    /// The candidate lowered successfully but did not satisfy the rule's
    /// declared exactness class.
    VerificationFailed {
        /// Rule being verified.
        rule: RewriteRule,
        /// Ordinary finite-value equality result.
        values_equal: bool,
        /// IEEE-754 bit-identity result.
        bitwise_identical: bool,
    },
}

impl fmt::Display for RewriteError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Provenance(error) => write!(formatter, "rewrite provenance error: {error}"),
            Self::Ir(error) => write!(formatter, "rewrite IR error: {error}"),
            Self::PatternMismatch { rule, root_index } => {
                write!(formatter, "{rule} does not match root node {root_index}")
            }
            Self::Equivalence(error) => write!(formatter, "rewrite equivalence error: {error}"),
            Self::VerificationFailed {
                rule,
                values_equal,
                bitwise_identical,
            } => write!(
                formatter,
                "{rule} failed exactness verification: values_equal={values_equal}, bitwise_identical={bitwise_identical}"
            ),
        }
    }
}

impl std::error::Error for RewriteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Provenance(error) => Some(error),
            Self::Ir(error) => Some(error),
            Self::Equivalence(error) => Some(error),
            Self::PatternMismatch { .. } | Self::VerificationFailed { .. } => None,
        }
    }
}

impl From<ProvenanceError> for RewriteError {
    fn from(value: ProvenanceError) -> Self {
        Self::Provenance(value)
    }
}

impl From<IrError> for RewriteError {
    fn from(value: IrError) -> Self {
        Self::Ir(value)
    }
}

impl From<EquivalenceError> for RewriteError {
    fn from(value: EquivalenceError) -> Self {
        Self::Equivalence(value)
    }
}

/// One accepted local rewrite together with its exact Stage-0 verification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewriteApplication {
    rule: RewriteRule,
    original: NodeId,
    replacement: NodeId,
    exactness_class: RewriteExactnessClass,
    verification: ExactLinearEquivalenceReport,
}

impl RewriteApplication {
    /// Applied rule.
    #[must_use]
    pub const fn rule(&self) -> RewriteRule {
        self.rule
    }

    /// Original rooted fragment.
    #[must_use]
    pub const fn original(&self) -> NodeId {
        self.original
    }

    /// Existing node selected as the replacement.
    #[must_use]
    pub const fn replacement(&self) -> NodeId {
        self.replacement
    }

    /// Exactness guarantee required for acceptance.
    #[must_use]
    pub const fn exactness_class(&self) -> RewriteExactnessClass {
        self.exactness_class
    }

    /// Full exact-equivalence report used for acceptance.
    #[must_use]
    pub const fn verification(&self) -> &ExactLinearEquivalenceReport {
        &self.verification
    }
}

/// Apply one explicitly requested local rewrite after independent validation.
///
/// This function never mutates the graph and never searches for a rule. The
/// caller supplies both the root and the rule. The returned replacement is an
/// already-existing node in the same IR instance.
pub fn apply_local_rewrite(
    ir: &CategoricalAttentionIr,
    root: NodeId,
    rule: RewriteRule,
) -> Result<RewriteApplication, RewriteError> {
    validate_rooted_subgraph(ir, root)?;

    let replacement = match rule {
        RewriteRule::LeftIdentity => match ir.node(root)?.kind() {
            IrNodeKind::Compose { outer, inner }
                if matches!(ir.node(*outer)?.kind(), IrNodeKind::Identity) =>
            {
                *inner
            }
            _ => return Err(pattern_mismatch(rule, root)),
        },
        RewriteRule::RightIdentity => match ir.node(root)?.kind() {
            IrNodeKind::Compose { outer, inner }
                if matches!(ir.node(*inner)?.kind(), IrNodeKind::Identity) =>
            {
                *outer
            }
            _ => return Err(pattern_mismatch(rule, root)),
        },
        RewriteRule::DaggerIdentity => match ir.node(root)?.kind() {
            IrNodeKind::Dagger { source }
                if matches!(ir.node(*source)?.kind(), IrNodeKind::Identity) =>
            {
                *source
            }
            _ => return Err(pattern_mismatch(rule, root)),
        },
        RewriteRule::DoubleDagger => match ir.node(root)?.kind() {
            IrNodeKind::Dagger { source } => match ir.node(*source)?.kind() {
                IrNodeKind::Dagger { source: original } => *original,
                _ => return Err(pattern_mismatch(rule, root)),
            },
            _ => return Err(pattern_mismatch(rule, root)),
        },
    };

    let verification = compare_exact_linear_semantics(ir, root, replacement)?;
    let exactness_class = rule.exactness_class();
    let accepted = match exactness_class {
        RewriteExactnessClass::FiniteValueExact => verification.values_equal(),
        RewriteExactnessClass::BitwiseExact => verification.bitwise_identical(),
    };

    if !accepted {
        return Err(RewriteError::VerificationFailed {
            rule,
            values_equal: verification.values_equal(),
            bitwise_identical: verification.bitwise_identical(),
        });
    }

    Ok(RewriteApplication {
        rule,
        original: root,
        replacement,
        exactness_class,
        verification,
    })
}

fn pattern_mismatch(rule: RewriteRule, root: NodeId) -> RewriteError {
    RewriteError::PatternMismatch {
        rule,
        root_index: root.index(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        LOCAL_REWRITE_CONTRACT, RewriteError, RewriteExactnessClass, RewriteRule,
        apply_local_rewrite,
    };
    use crate::experimental::tdi23_categorical::RealLinearMap;
    use crate::experimental::tdi23_ir::{CategoricalAttentionIr, NonlinearBoundaryKind};
    use crate::experimental::tdi23_ir_equivalence::{ComparisonSide, EquivalenceError};

    #[test]
    fn tdi23_2_rewrite_contract_is_versioned() {
        assert_eq!(
            LOCAL_REWRITE_CONTRACT,
            "tdi23.2-local-rewrite-calculus-v1"
        );
    }

    #[test]
    fn tdi23_2_rewrite_left_and_right_identity_are_value_exact() {
        let mut ir = CategoricalAttentionIr::new();
        let a = ir.add_atomic_object("A", 2).expect("A");
        let b = ir.add_atomic_object("B", 2).expect("B");
        let f = ir
            .add_linear_map(
                a,
                b,
                RealLinearMap::new(2, 2, vec![1.0, 2.0, 3.0, 4.0]).expect("map"),
            )
            .expect("f");
        let id_a = ir.add_identity(a).expect("id A");
        let id_b = ir.add_identity(b).expect("id B");
        let left = ir.compose(id_b, f).expect("id_B o f");
        let right = ir.compose(f, id_a).expect("f o id_A");

        let left_application =
            apply_local_rewrite(&ir, left, RewriteRule::LeftIdentity).expect("left identity");
        let right_application = apply_local_rewrite(&ir, right, RewriteRule::RightIdentity)
            .expect("right identity");

        assert_eq!(left_application.replacement(), f);
        assert_eq!(right_application.replacement(), f);
        assert_eq!(
            left_application.exactness_class(),
            RewriteExactnessClass::FiniteValueExact
        );
        assert!(left_application.verification().values_equal());
        assert!(right_application.verification().values_equal());
    }

    #[test]
    fn tdi23_2_rewrite_identity_class_does_not_hide_signed_zero_bit_drift() {
        let mut ir = CategoricalAttentionIr::new();
        let a = ir.add_atomic_object("A", 1).expect("A");
        let f = ir
            .add_linear_map(
                a,
                a,
                RealLinearMap::new(1, 1, vec![-0.0]).expect("signed-zero map"),
            )
            .expect("f");
        let identity = ir.add_identity(a).expect("identity");
        let composed = ir.compose(identity, f).expect("id o f");

        let application = apply_local_rewrite(&ir, composed, RewriteRule::LeftIdentity)
            .expect("value-exact identity elimination");

        assert!(application.verification().values_equal());
        assert!(!application.verification().bitwise_identical());
    }

    #[test]
    fn tdi23_2_rewrite_dagger_identity_is_bitwise_exact() {
        let mut ir = CategoricalAttentionIr::new();
        let a = ir.add_atomic_object("A", 3).expect("A");
        let identity = ir.add_identity(a).expect("identity");
        let dagger = ir.dagger(identity).expect("dagger identity");

        let application = apply_local_rewrite(&ir, dagger, RewriteRule::DaggerIdentity)
            .expect("dagger identity rewrite");

        assert_eq!(application.replacement(), identity);
        assert_eq!(
            application.exactness_class(),
            RewriteExactnessClass::BitwiseExact
        );
        assert!(application.verification().bitwise_identical());
    }

    #[test]
    fn tdi23_2_rewrite_double_dagger_is_bitwise_exact() {
        let mut ir = CategoricalAttentionIr::new();
        let a = ir.add_atomic_object("A", 2).expect("A");
        let b = ir.add_atomic_object("B", 2).expect("B");
        let f = ir
            .add_linear_map(
                a,
                b,
                RealLinearMap::new(2, 2, vec![-0.0, 2.0, 3.0, 4.0]).expect("map"),
            )
            .expect("f");
        let first = ir.dagger(f).expect("first dagger");
        let second = ir.dagger(first).expect("second dagger");

        let application = apply_local_rewrite(&ir, second, RewriteRule::DoubleDagger)
            .expect("double dagger rewrite");

        assert_eq!(application.replacement(), f);
        assert!(application.verification().bitwise_identical());
    }

    #[test]
    fn tdi23_2_rewrite_pattern_mismatch_fails_closed() {
        let mut ir = CategoricalAttentionIr::new();
        let a = ir.add_atomic_object("A", 1).expect("A");
        let identity = ir.add_identity(a).expect("identity");

        assert!(matches!(
            apply_local_rewrite(&ir, identity, RewriteRule::DoubleDagger),
            Err(RewriteError::PatternMismatch {
                rule: RewriteRule::DoubleDagger,
                ..
            })
        ));
    }

    #[test]
    fn tdi23_2_rewrite_refuses_identity_elimination_across_softmax_boundary() {
        let mut ir = CategoricalAttentionIr::new();
        let a = ir.add_atomic_object("A", 2).expect("A");
        let boundary = ir
            .add_nonlinear_boundary(a, a, NonlinearBoundaryKind::Softmax)
            .expect("softmax boundary");
        let identity = ir.add_identity(a).expect("identity");
        let composed = ir.compose(identity, boundary).expect("id o softmax");

        assert!(matches!(
            apply_local_rewrite(&ir, composed, RewriteRule::LeftIdentity),
            Err(RewriteError::Equivalence(EquivalenceError::NonlinearBoundary {
                side: ComparisonSide::Left,
                boundary: NonlinearBoundaryKind::Softmax,
            }))
        ));
    }

    #[test]
    fn tdi23_2_rewrite_distinct_same_dimension_objects_do_not_create_rules() {
        let mut ir = CategoricalAttentionIr::new();
        let a = ir.add_atomic_object("A", 2).expect("A");
        let b = ir.add_atomic_object("B", 2).expect("B");
        let f = ir
            .add_linear_map(
                a,
                b,
                RealLinearMap::identity(2).expect("carrier identity"),
            )
            .expect("A to B map");

        assert!(matches!(
            apply_local_rewrite(&ir, f, RewriteRule::LeftIdentity),
            Err(RewriteError::PatternMismatch { .. })
        ));
    }
}
