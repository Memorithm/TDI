//! Experimental Boolean-policy representation for TDI-9.3.
//!
//! This module is deliberately non-final and is exposed only through the
//! `experimental` feature. It provides a deterministic representation and
//! evaluator for Boolean action rules without selecting TDI-9.1 predicates,
//! thresholds, search algorithms, final seeds, or confirmatory material.

use core::fmt;

use super::adaptive_inference::{InferenceAction, PolicyArm};

/// A compact Boolean expression over an externally supplied predicate vector.
///
/// Predicate meaning is intentionally not encoded here. A TDI experiment must
/// freeze its observation-to-predicate boundary separately before evidence is
/// produced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BooleanExpr {
    Predicate(usize),
    Not(Box<BooleanExpr>),
    And(Box<BooleanExpr>, Box<BooleanExpr>),
    Or(Box<BooleanExpr>, Box<BooleanExpr>),
    Xor(Box<BooleanExpr>, Box<BooleanExpr>),
}

impl BooleanExpr {
    /// Evaluate the expression fail-closed against a predicate vector.
    pub fn evaluate(&self, predicates: &[bool]) -> Result<bool, BooleanPolicyError> {
        match self {
            Self::Predicate(index) => {
                predicates
                    .get(*index)
                    .copied()
                    .ok_or(BooleanPolicyError::PredicateOutOfRange {
                        index: *index,
                        predicate_count: predicates.len(),
                    })
            }
            Self::Not(inner) => Ok(!inner.evaluate(predicates)?),
            Self::And(left, right) => Ok(left.evaluate(predicates)? & right.evaluate(predicates)?),
            Self::Or(left, right) => Ok(left.evaluate(predicates)? | right.evaluate(predicates)?),
            Self::Xor(left, right) => Ok(left.evaluate(predicates)? ^ right.evaluate(predicates)?),
        }
    }

    fn validate_predicates(&self, predicate_count: usize) -> Result<(), BooleanPolicyError> {
        match self {
            Self::Predicate(index) => {
                if *index >= predicate_count {
                    return Err(BooleanPolicyError::PredicateOutOfRange {
                        index: *index,
                        predicate_count,
                    });
                }
                Ok(())
            }
            Self::Not(inner) => inner.validate_predicates(predicate_count),
            Self::And(left, right) | Self::Or(left, right) | Self::Xor(left, right) => {
                left.validate_predicates(predicate_count)?;
                right.validate_predicates(predicate_count)
            }
        }
    }

    /// Exact structural complexity of this expression under the reference IR.
    #[must_use]
    pub fn complexity(&self) -> BooleanComplexity {
        match self {
            Self::Predicate(_) => BooleanComplexity {
                predicate_reads: 1,
                logical_ops: 0,
                depth: 0,
            },
            Self::Not(inner) => {
                let inner = inner.complexity();
                BooleanComplexity {
                    predicate_reads: inner.predicate_reads,
                    logical_ops: inner.logical_ops + 1,
                    depth: inner.depth + 1,
                }
            }
            Self::And(left, right) | Self::Or(left, right) | Self::Xor(left, right) => {
                let left = left.complexity();
                let right = right.complexity();
                BooleanComplexity {
                    predicate_reads: left.predicate_reads + right.predicate_reads,
                    logical_ops: left.logical_ops + right.logical_ops + 1,
                    depth: left.depth.max(right.depth) + 1,
                }
            }
        }
    }
}

/// Exact reference complexity for one Boolean expression.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct BooleanComplexity {
    predicate_reads: u64,
    logical_ops: u64,
    depth: u32,
}

impl BooleanComplexity {
    #[must_use]
    pub const fn predicate_reads(self) -> u64 {
        self.predicate_reads
    }

    #[must_use]
    pub const fn logical_ops(self) -> u64 {
        self.logical_ops
    }

    #[must_use]
    pub const fn depth(self) -> u32 {
        self.depth
    }
}

/// One ordered Boolean rule mapping a predicate expression to an allowed TDI action.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BooleanActionRule {
    expression: BooleanExpr,
    action: InferenceAction,
}

impl BooleanActionRule {
    #[must_use]
    pub const fn new(expression: BooleanExpr, action: InferenceAction) -> Self {
        Self { expression, action }
    }

    #[must_use]
    pub const fn action(&self) -> InferenceAction {
        self.action
    }

    #[must_use]
    pub const fn expression(&self) -> &BooleanExpr {
        &self.expression
    }
}

/// Ordered, deterministic Boolean policy candidate for one frozen TDI policy arm.
///
/// Rules are evaluated in order. The first true rule selects its action; if no
/// rule matches, the declared fallback action is selected. Rule order is part of
/// the policy identity and must be treated as such by any future search layer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BooleanPolicy {
    arm: PolicyArm,
    rules: Vec<BooleanActionRule>,
    fallback: InferenceAction,
}

impl BooleanPolicy {
    pub fn new(
        arm: PolicyArm,
        rules: Vec<BooleanActionRule>,
        fallback: InferenceAction,
    ) -> Result<Self, BooleanPolicyError> {
        if !arm.is_trajectory_adaptive() {
            return Err(BooleanPolicyError::NonAdaptiveArm { arm });
        }
        if !arm.allows(fallback) {
            return Err(BooleanPolicyError::ActionForbidden {
                arm,
                action: fallback,
            });
        }
        for rule in &rules {
            if !arm.allows(rule.action) {
                return Err(BooleanPolicyError::ActionForbidden {
                    arm,
                    action: rule.action,
                });
            }
        }
        Ok(Self {
            arm,
            rules,
            fallback,
        })
    }

    #[must_use]
    pub const fn arm(&self) -> PolicyArm {
        self.arm
    }

    #[must_use]
    pub fn rules(&self) -> &[BooleanActionRule] {
        &self.rules
    }

    #[must_use]
    pub const fn fallback(&self) -> InferenceAction {
        self.fallback
    }

    /// Evaluate the candidate and return exact reference decision cost.
    pub fn decide(&self, predicates: &[bool]) -> Result<BooleanDecision, BooleanPolicyError> {
        for rule in &self.rules {
            rule.expression.validate_predicates(predicates.len())?;
        }

        let mut predicate_reads = 0u64;
        let mut logical_ops = 0u64;

        for (rule_index, rule) in self.rules.iter().enumerate() {
            let complexity = rule.expression.complexity();
            predicate_reads = predicate_reads
                .checked_add(complexity.predicate_reads)
                .ok_or(BooleanPolicyError::ComplexityOverflow)?;
            logical_ops = logical_ops
                .checked_add(complexity.logical_ops)
                .ok_or(BooleanPolicyError::ComplexityOverflow)?;

            if rule.expression.evaluate(predicates)? {
                return Ok(BooleanDecision {
                    action: rule.action,
                    matched_rule: Some(rule_index),
                    evaluated_rules: rule_index + 1,
                    predicate_reads,
                    logical_ops,
                });
            }
        }

        Ok(BooleanDecision {
            action: self.fallback,
            matched_rule: None,
            evaluated_rules: self.rules.len(),
            predicate_reads,
            logical_ops,
        })
    }

    /// Structural cost of evaluating every rule once, independent of outcomes.
    pub fn worst_case_complexity(&self) -> Result<BooleanComplexity, BooleanPolicyError> {
        let mut predicate_reads = 0u64;
        let mut logical_ops = 0u64;
        let mut depth = 0u32;
        for rule in &self.rules {
            let complexity = rule.expression.complexity();
            predicate_reads = predicate_reads
                .checked_add(complexity.predicate_reads)
                .ok_or(BooleanPolicyError::ComplexityOverflow)?;
            logical_ops = logical_ops
                .checked_add(complexity.logical_ops)
                .ok_or(BooleanPolicyError::ComplexityOverflow)?;
            depth = depth.max(complexity.depth);
        }
        Ok(BooleanComplexity {
            predicate_reads,
            logical_ops,
            depth,
        })
    }
}

/// Deterministic result of evaluating one Boolean policy candidate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BooleanDecision {
    action: InferenceAction,
    matched_rule: Option<usize>,
    evaluated_rules: usize,
    predicate_reads: u64,
    logical_ops: u64,
}

impl BooleanDecision {
    #[must_use]
    pub const fn action(self) -> InferenceAction {
        self.action
    }

    #[must_use]
    pub const fn matched_rule(self) -> Option<usize> {
        self.matched_rule
    }

    #[must_use]
    pub const fn evaluated_rules(self) -> usize {
        self.evaluated_rules
    }

    #[must_use]
    pub const fn predicate_reads(self) -> u64 {
        self.predicate_reads
    }

    #[must_use]
    pub const fn logical_ops(self) -> u64 {
        self.logical_ops
    }
}

/// Declared predicate arity for the hand-written C2 reference Boolean shape.
///
/// Index meanings are documentation labels for representation calibration only.
/// They do **not** freeze `permitted_observation_vector` or any other TDI-9.1
/// field, and they do not authorize TDI-9.2.
pub const REFERENCE_C2_PREDICATE_COUNT: usize = 5;

/// Predicate indices used by [`reference_c2_stop_expression`].
///
/// These names mirror the hand-written C2 comments in
/// `adaptive_policies::base_should_stop`. They are representation labels for
/// TDI-9.3.0 calibration, not an experimental observation-vector pin.
pub mod reference_c2_predicates {
    pub const ENOUGH_STEPS: usize = 0;
    pub const TERMINAL: usize = 1;
    pub const RESIDUAL_SMALL: usize = 2;
    pub const DELTA_SMALL: usize = 3;
    pub const MARGIN_LARGE: usize = 4;
}

/// Exact hand-written C2 STOP Boolean (documentation formula).
///
/// `STOP = enough_steps AND (terminal OR (residual_small AND delta_small AND margin_large))`
///
/// This is the independent oracle used by TDI-9.3.0 exhaustive truth-table
/// calibration. It does not read trajectory observations or freeze thresholds.
#[must_use]
pub fn reference_c2_hand_stop(predicates: &[bool; REFERENCE_C2_PREDICATE_COUNT]) -> bool {
    use reference_c2_predicates::{
        DELTA_SMALL, ENOUGH_STEPS, MARGIN_LARGE, RESIDUAL_SMALL, TERMINAL,
    };
    let enough_steps = predicates[ENOUGH_STEPS];
    let terminal = predicates[TERMINAL];
    let residual_small = predicates[RESIDUAL_SMALL];
    let delta_small = predicates[DELTA_SMALL];
    let margin_large = predicates[MARGIN_LARGE];
    enough_steps & (terminal | (residual_small & delta_small & margin_large))
}

/// Compact Boolean IR encoding of the hand-written C2 STOP rule.
#[must_use]
pub fn reference_c2_stop_expression() -> BooleanExpr {
    use reference_c2_predicates::{
        DELTA_SMALL, ENOUGH_STEPS, MARGIN_LARGE, RESIDUAL_SMALL, TERMINAL,
    };
    // enough_steps AND (terminal OR (residual_small AND delta_small AND margin_large))
    BooleanExpr::And(
        Box::new(BooleanExpr::Predicate(ENOUGH_STEPS)),
        Box::new(BooleanExpr::Or(
            Box::new(BooleanExpr::Predicate(TERMINAL)),
            Box::new(BooleanExpr::And(
                Box::new(BooleanExpr::And(
                    Box::new(BooleanExpr::Predicate(RESIDUAL_SMALL)),
                    Box::new(BooleanExpr::Predicate(DELTA_SMALL)),
                )),
                Box::new(BooleanExpr::Predicate(MARGIN_LARGE)),
            )),
        )),
    )
}

/// C2 BooleanPolicy that STOPs exactly when [`reference_c2_stop_expression`] is true.
///
/// Fallback is CONTINUE. This is a representation-calibration fixture only: it
/// does not integrate into the C2 reference evaluator and does not pin TDI-9.1.
pub fn reference_c2_stop_policy() -> Result<BooleanPolicy, BooleanPolicyError> {
    BooleanPolicy::new(
        PolicyArm::C2AdaptiveStopping,
        vec![BooleanActionRule::new(
            reference_c2_stop_expression(),
            InferenceAction::Stop,
        )],
        InferenceAction::Continue,
    )
}

/// Declared predicate arity for the hand-written C3 reference Boolean shape.
///
/// Index meanings are documentation labels for representation calibration only.
/// They do **not** freeze `permitted_observation_vector`, thresholds, cadence,
/// or any other TDI-9.1 field, and they do not authorize TDI-9.2.
///
/// `BASE_STOP` is the already C2-calibrated adaptive-stop Boolean treated as one
/// abstract predicate here so TDI-9.3.0 can calibrate C3's ordered multi-action
/// dispatch without re-deriving the C2 STOP composition.
pub const REFERENCE_C3_PREDICATE_COUNT: usize = 9;

/// Predicate indices used by [`reference_c3_policy`].
///
/// These names mirror the documented C3 reference state machine in
/// `docs/TDI-9.1-REFERENCE-POLICIES.md` and the non-final predicates computed by
/// `C3RecoveryPolicy::decide`. They are representation labels for TDI-9.3.0
/// calibration, not an experimental observation-vector pin.
pub mod reference_c3_predicates {
    pub const BASE_STOP: usize = 0;
    pub const VERIFY_BEFORE_STOP: usize = 1;
    pub const CADENCE_DUE: usize = 2;
    pub const CHECKPOINT_AVAILABLE: usize = 3;
    pub const REMAINING_WORK: usize = 4;
    pub const VERIFIER_VIOLATED: usize = 5;
    pub const VERIFIER_SATISFIED: usize = 6;
    pub const VERIFIER_INDETERMINATE: usize = 7;
    pub const VERIFIER_ABSENT: usize = 8;
}

/// Well-formed C3 verifier encoding: exactly one of the four verifier-state
/// predicates is true. Inconsistent encodings are outside the reference
/// observation carrier and are excluded from exhaustive action calibration.
#[must_use]
pub fn reference_c3_verifier_encoding_well_formed(
    predicates: &[bool; REFERENCE_C3_PREDICATE_COUNT],
) -> bool {
    use reference_c3_predicates::{
        VERIFIER_ABSENT, VERIFIER_INDETERMINATE, VERIFIER_SATISFIED, VERIFIER_VIOLATED,
    };
    let flags = [
        predicates[VERIFIER_VIOLATED],
        predicates[VERIFIER_SATISFIED],
        predicates[VERIFIER_INDETERMINATE],
        predicates[VERIFIER_ABSENT],
    ];
    flags.iter().filter(|flag| **flag).count() == 1
}

/// Exact hand-written C3 action oracle (documentation state machine).
///
/// Returns `None` for the typed fail-closed unrecoverable verifier violation
/// (`Violated` with no checkpoint and no remaining work). That rejection is
/// outside the Boolean action vocabulary (`CONTINUE`/`VERIFY`/`BACKTRACK`/
/// `STOP`) and is deliberately not encoded as a BooleanPolicy action.
///
/// Predicate meanings are representation labels only; this oracle does not read
/// trajectory observations or freeze thresholds.
#[must_use]
pub fn reference_c3_hand_action(
    predicates: &[bool; REFERENCE_C3_PREDICATE_COUNT],
) -> Option<InferenceAction> {
    use reference_c3_predicates::{
        BASE_STOP, CADENCE_DUE, CHECKPOINT_AVAILABLE, REMAINING_WORK, VERIFIER_ABSENT,
        VERIFIER_INDETERMINATE, VERIFIER_SATISFIED, VERIFIER_VIOLATED, VERIFY_BEFORE_STOP,
    };
    let base_stop = predicates[BASE_STOP];
    let verify_before_stop = predicates[VERIFY_BEFORE_STOP];
    let cadence_due = predicates[CADENCE_DUE];
    let checkpoint_available = predicates[CHECKPOINT_AVAILABLE];
    let remaining_work = predicates[REMAINING_WORK];
    let violated = predicates[VERIFIER_VIOLATED];
    let satisfied = predicates[VERIFIER_SATISFIED];
    let indeterminate = predicates[VERIFIER_INDETERMINATE];
    let absent = predicates[VERIFIER_ABSENT];

    if violated && checkpoint_available {
        Some(InferenceAction::Backtrack)
    } else if violated && remaining_work {
        Some(InferenceAction::Continue)
    } else if violated {
        None
    } else if satisfied {
        Some(InferenceAction::Stop)
    } else if indeterminate {
        Some(InferenceAction::Continue)
    } else if absent && base_stop && verify_before_stop {
        Some(InferenceAction::Verify)
    } else if absent && base_stop {
        Some(InferenceAction::Stop)
    } else if absent && cadence_due {
        Some(InferenceAction::Verify)
    } else if absent {
        Some(InferenceAction::Continue)
    } else {
        // Inconsistent verifier encoding: no unique state bit.
        None
    }
}

/// Ordered Boolean IR encoding of the hand-written C3 multi-action state machine.
///
/// Rule order is part of the policy identity (first-match). The unrecoverable
/// `Violated` rejection remains outside this action IR: those rows are excluded
/// from action-equivalence calibration via [`reference_c3_hand_action`] returning
/// `None`.
pub fn reference_c3_policy() -> Result<BooleanPolicy, BooleanPolicyError> {
    use reference_c3_predicates::{
        BASE_STOP, CADENCE_DUE, CHECKPOINT_AVAILABLE, REMAINING_WORK, VERIFIER_ABSENT,
        VERIFIER_INDETERMINATE, VERIFIER_SATISFIED, VERIFIER_VIOLATED, VERIFY_BEFORE_STOP,
    };
    BooleanPolicy::new(
        PolicyArm::C3VerificationRecovery,
        vec![
            BooleanActionRule::new(
                BooleanExpr::And(
                    Box::new(BooleanExpr::Predicate(VERIFIER_VIOLATED)),
                    Box::new(BooleanExpr::Predicate(CHECKPOINT_AVAILABLE)),
                ),
                InferenceAction::Backtrack,
            ),
            BooleanActionRule::new(
                BooleanExpr::And(
                    Box::new(BooleanExpr::Predicate(VERIFIER_VIOLATED)),
                    Box::new(BooleanExpr::Predicate(REMAINING_WORK)),
                ),
                InferenceAction::Continue,
            ),
            BooleanActionRule::new(
                BooleanExpr::Predicate(VERIFIER_SATISFIED),
                InferenceAction::Stop,
            ),
            BooleanActionRule::new(
                BooleanExpr::Predicate(VERIFIER_INDETERMINATE),
                InferenceAction::Continue,
            ),
            BooleanActionRule::new(
                BooleanExpr::And(
                    Box::new(BooleanExpr::And(
                        Box::new(BooleanExpr::Predicate(VERIFIER_ABSENT)),
                        Box::new(BooleanExpr::Predicate(BASE_STOP)),
                    )),
                    Box::new(BooleanExpr::Predicate(VERIFY_BEFORE_STOP)),
                ),
                InferenceAction::Verify,
            ),
            BooleanActionRule::new(
                BooleanExpr::And(
                    Box::new(BooleanExpr::Predicate(VERIFIER_ABSENT)),
                    Box::new(BooleanExpr::Predicate(BASE_STOP)),
                ),
                InferenceAction::Stop,
            ),
            BooleanActionRule::new(
                BooleanExpr::And(
                    Box::new(BooleanExpr::Predicate(VERIFIER_ABSENT)),
                    Box::new(BooleanExpr::Predicate(CADENCE_DUE)),
                ),
                InferenceAction::Verify,
            ),
        ],
        InferenceAction::Continue,
    )
}

/// Fail-closed errors for the TDI-9.3 experimental Boolean-policy IR.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BooleanPolicyError {
    PredicateOutOfRange {
        index: usize,
        predicate_count: usize,
    },
    NonAdaptiveArm {
        arm: PolicyArm,
    },
    ActionForbidden {
        arm: PolicyArm,
        action: InferenceAction,
    },
    ComplexityOverflow,
}

impl fmt::Display for BooleanPolicyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for BooleanPolicyError {}

#[cfg(test)]
mod tests {
    use super::{
        BooleanActionRule, BooleanExpr, BooleanPolicy, BooleanPolicyError,
        REFERENCE_C2_PREDICATE_COUNT, REFERENCE_C3_PREDICATE_COUNT, reference_c2_hand_stop,
        reference_c2_stop_expression, reference_c2_stop_policy, reference_c3_hand_action,
        reference_c3_policy, reference_c3_verifier_encoding_well_formed,
    };
    use crate::experimental::adaptive_inference::{InferenceAction, PolicyArm};

    fn every_predicate_vector() -> impl Iterator<Item = [bool; REFERENCE_C2_PREDICATE_COUNT]> {
        (0..(1usize << REFERENCE_C2_PREDICATE_COUNT)).map(|mask| {
            let mut predicates = [false; REFERENCE_C2_PREDICATE_COUNT];
            for (index, slot) in predicates.iter_mut().enumerate() {
                *slot = ((mask >> index) & 1) == 1;
            }
            predicates
        })
    }

    #[test]
    fn reference_c2_expression_matches_hand_formula_on_exhaustive_table() {
        let expression = reference_c2_stop_expression();
        let mut rows = 0usize;
        for predicates in every_predicate_vector() {
            rows += 1;
            let ir = expression
                .evaluate(&predicates)
                .expect("reference C2 expression uses only declared indices");
            assert_eq!(
                ir,
                reference_c2_hand_stop(&predicates),
                "truth-table mismatch for predicates={predicates:?}"
            );
        }
        assert_eq!(rows, 1usize << REFERENCE_C2_PREDICATE_COUNT);
    }

    #[test]
    fn reference_c2_policy_decides_stop_continue_on_exhaustive_table() {
        let policy = reference_c2_stop_policy().expect("valid reference C2 policy");
        let mut stop_rows = 0usize;
        let mut continue_rows = 0usize;
        for predicates in every_predicate_vector() {
            let decision = policy
                .decide(&predicates)
                .expect("reference C2 policy evaluates on declared arity");
            let expect_stop = reference_c2_hand_stop(&predicates);
            if expect_stop {
                assert_eq!(decision.action(), InferenceAction::Stop);
                assert_eq!(decision.matched_rule(), Some(0));
                stop_rows += 1;
            } else {
                assert_eq!(decision.action(), InferenceAction::Continue);
                assert_eq!(decision.matched_rule(), None);
                continue_rows += 1;
            }
        }
        assert_eq!(
            stop_rows + continue_rows,
            1usize << REFERENCE_C2_PREDICATE_COUNT
        );
        // Sanity: both outcomes exist on the finite table.
        assert!(stop_rows > 0 && continue_rows > 0);
    }

    #[test]
    fn ordered_rules_select_first_matching_action() {
        let policy = BooleanPolicy::new(
            PolicyArm::C3VerificationRecovery,
            vec![
                BooleanActionRule::new(BooleanExpr::Predicate(0), InferenceAction::Backtrack),
                BooleanActionRule::new(BooleanExpr::Predicate(1), InferenceAction::Verify),
            ],
            InferenceAction::Continue,
        )
        .expect("C3 actions allowed");

        let decision = policy.decide(&[true, true]).expect("valid predicates");
        assert_eq!(decision.action(), InferenceAction::Backtrack);
        assert_eq!(decision.matched_rule(), Some(0));
        assert_eq!(decision.evaluated_rules(), 1);
    }

    #[test]
    fn rejects_boolean_policy_for_non_adaptive_arms() {
        for arm in [PolicyArm::C0FixedCompute, PolicyArm::C1StaticPreallocation] {
            assert!(matches!(
                BooleanPolicy::new(
                    arm,
                    vec![BooleanActionRule::new(
                        BooleanExpr::Predicate(0),
                        InferenceAction::Stop,
                    )],
                    InferenceAction::Continue,
                ),
                Err(BooleanPolicyError::NonAdaptiveArm { arm: rejected }) if rejected == arm
            ));
        }
    }

    #[test]
    fn c2_rejects_verification_and_backtracking_actions() {
        assert!(matches!(
            BooleanPolicy::new(
                PolicyArm::C2AdaptiveStopping,
                vec![BooleanActionRule::new(
                    BooleanExpr::Predicate(0),
                    InferenceAction::Verify,
                )],
                InferenceAction::Continue,
            ),
            Err(BooleanPolicyError::ActionForbidden {
                arm: PolicyArm::C2AdaptiveStopping,
                action: InferenceAction::Verify,
            })
        ));
    }

    #[test]
    fn missing_predicate_fails_closed() {
        let policy = BooleanPolicy::new(
            PolicyArm::C2AdaptiveStopping,
            vec![BooleanActionRule::new(
                BooleanExpr::Predicate(2),
                InferenceAction::Stop,
            )],
            InferenceAction::Continue,
        )
        .expect("valid C2 policy");

        assert!(matches!(
            policy.decide(&[true, false]),
            Err(BooleanPolicyError::PredicateOutOfRange {
                index: 2,
                predicate_count: 2,
            })
        ));
    }

    #[test]
    fn validates_all_predicate_references_before_short_circuiting() {
        let policy = BooleanPolicy::new(
            PolicyArm::C2AdaptiveStopping,
            vec![
                BooleanActionRule::new(BooleanExpr::Predicate(0), InferenceAction::Stop),
                BooleanActionRule::new(BooleanExpr::Predicate(2), InferenceAction::Continue),
            ],
            InferenceAction::Continue,
        )
        .expect("valid C2 policy");

        assert!(matches!(
            policy.decide(&[true]),
            Err(BooleanPolicyError::PredicateOutOfRange {
                index: 2,
                predicate_count: 1,
            })
        ));
    }

    #[test]
    fn complexity_is_exact_for_reference_expression() {
        let complexity = reference_c2_stop_expression().complexity();
        assert_eq!(complexity.predicate_reads(), 5);
        assert_eq!(complexity.logical_ops(), 4);
        assert_eq!(complexity.depth(), 4);
    }

    #[test]
    fn reference_c2_policy_worst_case_complexity_is_deterministic() {
        let policy = reference_c2_stop_policy().expect("valid reference C2 policy");
        let complexity = policy
            .worst_case_complexity()
            .expect("reference expression complexity is finite");
        assert_eq!(complexity.predicate_reads(), 5);
        assert_eq!(complexity.logical_ops(), 4);
        assert_eq!(complexity.depth(), 4);
    }

    fn every_c3_predicate_vector() -> impl Iterator<Item = [bool; REFERENCE_C3_PREDICATE_COUNT]> {
        (0..(1usize << REFERENCE_C3_PREDICATE_COUNT)).map(|mask| {
            let mut predicates = [false; REFERENCE_C3_PREDICATE_COUNT];
            for (index, slot) in predicates.iter_mut().enumerate() {
                *slot = ((mask >> index) & 1) == 1;
            }
            predicates
        })
    }

    #[test]
    fn reference_c3_policy_matches_hand_on_well_formed_action_rows() {
        let policy = reference_c3_policy().expect("valid reference C3 policy");
        let mut compared = 0usize;
        let mut unrecoverable = 0usize;
        let mut malformed = 0usize;
        let mut seen = [0usize; 4]; // Continue, Verify, Backtrack, Stop counts
        for predicates in every_c3_predicate_vector() {
            if !reference_c3_verifier_encoding_well_formed(&predicates) {
                malformed += 1;
                continue;
            }
            match reference_c3_hand_action(&predicates) {
                None => {
                    unrecoverable += 1;
                }
                Some(expect) => {
                    let decision = policy
                        .decide(&predicates)
                        .expect("well-formed C3 action rows evaluate");
                    assert_eq!(
                        decision.action(),
                        expect,
                        "C3 action mismatch for predicates={predicates:?}"
                    );
                    compared += 1;
                    match expect {
                        InferenceAction::Continue => seen[0] += 1,
                        InferenceAction::Verify => seen[1] += 1,
                        InferenceAction::Backtrack => seen[2] += 1,
                        InferenceAction::Stop => seen[3] += 1,
                    }
                }
            }
        }
        assert_eq!(
            compared + unrecoverable + malformed,
            1usize << REFERENCE_C3_PREDICATE_COUNT
        );
        // Every action in the C3 vocabulary appears on the well-formed table.
        assert!(seen.iter().all(|&count| count > 0), "seen={seen:?}");
        // Typed fail-closed unrecoverable rows exist and stay outside action IR.
        assert!(unrecoverable > 0);
        assert!(malformed > 0);
    }

    #[test]
    fn reference_c3_unrecoverable_violation_is_outside_action_vocabulary() {
        use super::reference_c3_predicates::{
            CHECKPOINT_AVAILABLE, REMAINING_WORK, VERIFIER_ABSENT, VERIFIER_INDETERMINATE,
            VERIFIER_SATISFIED, VERIFIER_VIOLATED,
        };
        let mut predicates = [false; REFERENCE_C3_PREDICATE_COUNT];
        predicates[VERIFIER_VIOLATED] = true;
        predicates[CHECKPOINT_AVAILABLE] = false;
        predicates[REMAINING_WORK] = false;
        predicates[VERIFIER_SATISFIED] = false;
        predicates[VERIFIER_INDETERMINATE] = false;
        predicates[VERIFIER_ABSENT] = false;
        assert!(reference_c3_verifier_encoding_well_formed(&predicates));
        assert_eq!(reference_c3_hand_action(&predicates), None);
    }

    #[test]
    fn reference_c3_policy_worst_case_complexity_is_deterministic() {
        let policy = reference_c3_policy().expect("valid reference C3 policy");
        let complexity = policy
            .worst_case_complexity()
            .expect("reference C3 complexity is finite");
        // Seven ordered rules: (2+1)+(2+1)+(1+0)+(1+0)+(3+2)+(2+1)+(2+1)
        assert_eq!(complexity.predicate_reads(), 13);
        assert_eq!(complexity.logical_ops(), 6);
        assert_eq!(complexity.depth(), 2);
        assert_eq!(policy.rules().len(), 7);
        assert_eq!(policy.fallback(), InferenceAction::Continue);
        assert_eq!(policy.arm(), PolicyArm::C3VerificationRecovery);
    }
}
