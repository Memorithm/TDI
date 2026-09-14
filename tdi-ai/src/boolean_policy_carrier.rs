//! Non-final validated input carrier for the existing abstract C3 schema.
//!
//! This is not a trajectory-observation freeze or a production adapter. It closes
//! the gap between filtering invalid rows in a calibration and rejecting them
//! before a caller dispatches a Boolean policy. No TDI-9.2 authority is created.

use core::fmt;

use super::boolean_policy_synthesis::{
    BooleanDecision, BooleanPolicy, BooleanPolicyError, REFERENCE_C3_PREDICATE_COUNT,
    SynthesisSearchEnvelope, reference_c3_predicates, reference_c3_verifier_encoding_well_formed,
};

/// Version of the abstract predicate order, not a TDI-9.1 observation-vector pin.
pub const C3_CARRIER_SCHEMA: &str = "tdi9.3.reference-c3-carrier.v1";

/// An action-bearing row of the calibrated nine-predicate C3 carrier.
///
/// Construction validates exact arity, exactly one verifier-state flag, and the
/// absence of an unrecoverable violation. The stored array cannot be mutated by
/// a caller after validation. Missing state is rejected, never filled with false.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedC3PredicateRow {
    predicates: [bool; REFERENCE_C3_PREDICATE_COUNT],
}

/// Carrier failures remain distinct from candidate-policy admission failures.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum C3CarrierError {
    PredicateArity { expected: usize, actual: usize },
    InvalidVerifierEncoding { active_states: usize },
    UnrecoverableViolation,
    Policy(BooleanPolicyError),
}

impl ValidatedC3PredicateRow {
    /// Validate a row without consulting any candidate policy or target label.
    ///
    /// # Errors
    ///
    /// Rejects missing/extra predicates, zero or multiple verifier-state flags,
    /// and violated states with neither checkpoint nor remaining work.
    pub fn new(raw: &[bool]) -> Result<Self, C3CarrierError> {
        let predicates: [bool; REFERENCE_C3_PREDICATE_COUNT] =
            raw.try_into().map_err(|_| C3CarrierError::PredicateArity {
                expected: REFERENCE_C3_PREDICATE_COUNT,
                actual: raw.len(),
            })?;
        use reference_c3_predicates::{
            CHECKPOINT_AVAILABLE, REMAINING_WORK, VERIFIER_ABSENT, VERIFIER_INDETERMINATE,
            VERIFIER_SATISFIED, VERIFIER_VIOLATED,
        };
        if !reference_c3_verifier_encoding_well_formed(&predicates) {
            let active_states = [
                VERIFIER_VIOLATED,
                VERIFIER_SATISFIED,
                VERIFIER_INDETERMINATE,
                VERIFIER_ABSENT,
            ]
            .iter()
            .filter(|&&index| predicates[index])
            .count();
            return Err(C3CarrierError::InvalidVerifierEncoding { active_states });
        }
        if predicates[VERIFIER_VIOLATED]
            && !predicates[CHECKPOINT_AVAILABLE]
            && !predicates[REMAINING_WORK]
        {
            return Err(C3CarrierError::UnrecoverableViolation);
        }
        Ok(Self { predicates })
    }

    #[must_use]
    pub const fn as_array(&self) -> &[bool; REFERENCE_C3_PREDICATE_COUNT] {
        &self.predicates
    }

    /// Admit a candidate to the existing C3 envelope before evaluating its rules.
    ///
    /// The returned counters are the original rule-evaluator counters only.
    /// Carrier validation and envelope admission are additional work; this API
    /// does not pretend those costs are included or measured in hardware time.
    ///
    /// # Errors
    ///
    /// Rejects candidates outside the reference C3 envelope, including C2 arms,
    /// forbidden connectives, invalid predicate indices and excessive complexity.
    pub fn decide(&self, policy: &BooleanPolicy) -> Result<BooleanDecision, C3CarrierError> {
        SynthesisSearchEnvelope::reference_c3()
            .admits_policy(policy)
            .map_err(C3CarrierError::Policy)?;
        policy
            .decide(&self.predicates)
            .map_err(C3CarrierError::Policy)
    }
}

/// Validate raw carrier bits, then admit and dispatch a candidate C3 policy.
///
/// Input-carrier errors take precedence over candidate-policy errors. No action
/// is produced for an invalid row, even by an unconditional fallback policy.
///
/// # Errors
///
/// Propagates all carrier validation, envelope admission and evaluator errors.
pub fn decide_checked_c3(
    policy: &BooleanPolicy,
    raw: &[bool],
) -> Result<BooleanDecision, C3CarrierError> {
    ValidatedC3PredicateRow::new(raw)?.decide(policy)
}

impl fmt::Display for C3CarrierError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for C3CarrierError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::experimental::adaptive_inference::{InferenceAction, PolicyArm};
    use crate::experimental::boolean_policy_synthesis::{
        BooleanActionRule, BooleanExpr, reference_c2_stop_policy, reference_c3_hand_action,
        reference_c3_policy,
    };

    #[test]
    fn every_abstract_row_is_dispatched_or_explicitly_rejected() {
        let reference = reference_c3_policy().unwrap();
        let mut counts = [0usize; 3];
        for address in 0..512usize {
            let row = core::array::from_fn::<_, 9, _>(|bit| address & (1 << bit) != 0);
            match ValidatedC3PredicateRow::new(&row) {
                Ok(validated) => {
                    counts[0] += 1;
                    let decision = validated.decide(&reference).unwrap();
                    assert_eq!(Some(decision.action()), reference_c3_hand_action(&row));
                    assert_eq!(decision, reference.decide(&row).unwrap());
                    assert_eq!(validated.as_array(), &row);
                }
                Err(C3CarrierError::UnrecoverableViolation) => {
                    counts[1] += 1;
                    assert!(reference_c3_verifier_encoding_well_formed(&row));
                    assert_eq!(reference_c3_hand_action(&row), None);
                    assert!(decide_checked_c3(&reference, &row).is_err());
                }
                Err(C3CarrierError::InvalidVerifierEncoding { .. }) => {
                    counts[2] += 1;
                    assert!(!reference_c3_verifier_encoding_well_formed(&row));
                    assert!(decide_checked_c3(&reference, &row).is_err());
                }
                Err(error) => panic!("unexpected carrier error: {error}"),
            }
        }
        assert_eq!(counts, [120, 8, 384]);
    }

    #[test]
    fn unconditional_policy_cannot_turn_rejections_into_actions() {
        let policy = BooleanPolicy::new(
            PolicyArm::C3VerificationRecovery,
            Vec::new(),
            InferenceAction::Stop,
        )
        .unwrap();
        let mut row = [false; 9];
        assert_eq!(
            decide_checked_c3(&policy, &row),
            Err(C3CarrierError::InvalidVerifierEncoding { active_states: 0 })
        );
        row[reference_c3_predicates::VERIFIER_VIOLATED] = true;
        assert_eq!(
            decide_checked_c3(&policy, &row),
            Err(C3CarrierError::UnrecoverableViolation)
        );
        row[reference_c3_predicates::VERIFIER_SATISFIED] = true;
        assert_eq!(
            decide_checked_c3(&policy, &row),
            Err(C3CarrierError::InvalidVerifierEncoding { active_states: 2 })
        );
    }

    #[test]
    fn both_missing_and_extra_predicates_are_rejected() {
        for count in [0, 8, 10] {
            assert_eq!(
                ValidatedC3PredicateRow::new(&vec![false; count]),
                Err(C3CarrierError::PredicateArity {
                    expected: 9,
                    actual: count,
                })
            );
        }
    }

    #[test]
    fn validated_carrier_does_not_bypass_policy_arm_or_index_checks() {
        let mut row = [false; 9];
        row[reference_c3_predicates::VERIFIER_ABSENT] = true;
        let validated = ValidatedC3PredicateRow::new(&row).unwrap();
        assert!(
            validated
                .decide(&reference_c2_stop_policy().unwrap())
                .is_err()
        );
        let invalid = BooleanPolicy::new(
            PolicyArm::C3VerificationRecovery,
            vec![BooleanActionRule::new(
                BooleanExpr::Predicate(9),
                InferenceAction::Stop,
            )],
            InferenceAction::Continue,
        )
        .unwrap();
        assert!(matches!(
            validated.decide(&invalid),
            Err(C3CarrierError::Policy(_))
        ));
    }

    #[test]
    fn input_validation_precedes_policy_evaluation_and_owns_its_snapshot() {
        let c2 = reference_c2_stop_policy().unwrap();
        assert_eq!(
            decide_checked_c3(&c2, &[false; 9]),
            Err(C3CarrierError::InvalidVerifierEncoding { active_states: 0 })
        );
        let mut raw = [false; 9];
        raw[reference_c3_predicates::VERIFIER_SATISFIED] = true;
        let valid = ValidatedC3PredicateRow::new(&raw).unwrap();
        raw.fill(false);
        assert!(ValidatedC3PredicateRow::new(&raw).is_err());
        assert_eq!(
            valid
                .decide(&reference_c3_policy().unwrap())
                .unwrap()
                .action(),
            InferenceAction::Stop
        );
    }
}
