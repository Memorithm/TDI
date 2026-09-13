//! Non-final single-rule baseline generation for TDI-9.3.2 C3 search.
//!
//! These candidates are deliberately simple controls: one declared Boolean
//! predicate triggers one non-default C3 action and `CONTINUE` is the fallback.
//! Predicate meanings remain external and non-pinning.

use core::fmt;

use super::adaptive_inference::{InferenceAction, PolicyArm};
use super::boolean_policy_search::BooleanPolicySearchCandidate;
use super::boolean_policy_synthesis::{
    BooleanActionRule, BooleanExpr, BooleanPolicy, BooleanPolicyError, SynthesisSearchEnvelope,
};

/// Fail-closed errors for bounded C3 baseline generation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum C3BaselineError {
    C3EnvelopeRequired { actual: PolicyArm },
    Policy(BooleanPolicyError),
}

/// Generate deterministic single-predicate C3 action baselines.
///
/// For every predicate, exactly three policies are emitted in this order:
/// `STOP`, `VERIFY`, then `BACKTRACK`. `CONTINUE` is intentionally the fallback
/// rather than a redundant trigger action. With the calibrated nine-predicate
/// C3 envelope this yields exactly 27 candidates.
///
/// This baseline population is representation/search infrastructure only. It
/// does not choose TDI-9.1 observations, thresholds, cadence, verifier inputs,
/// or recovery semantics, and it does not authorize TDI-9.2.
///
/// # Errors
///
/// Rejects a non-C3 envelope and propagates policy/envelope validation errors.
pub fn generate_c3_single_rule_baselines(
    envelope: &SynthesisSearchEnvelope,
) -> Result<Vec<BooleanPolicySearchCandidate>, C3BaselineError> {
    if envelope.arm() != PolicyArm::C3VerificationRecovery {
        return Err(C3BaselineError::C3EnvelopeRequired {
            actual: envelope.arm(),
        });
    }

    let actions = [
        ("stop", InferenceAction::Stop),
        ("verify", InferenceAction::Verify),
        ("backtrack", InferenceAction::Backtrack),
    ];
    let mut candidates = Vec::with_capacity(envelope.predicate_count() * actions.len());

    for predicate in 0..envelope.predicate_count() {
        for (action_name, action) in actions {
            let policy = BooleanPolicy::new(
                PolicyArm::C3VerificationRecovery,
                vec![BooleanActionRule::new(
                    BooleanExpr::Predicate(predicate),
                    action,
                )],
                InferenceAction::Continue,
            )
            .map_err(C3BaselineError::Policy)?;
            envelope
                .admits_policy(&policy)
                .map_err(C3BaselineError::Policy)?;
            candidates.push(BooleanPolicySearchCandidate {
                candidate_id: format!("tdi9.3-c3-{action_name}-p{predicate}"),
                policy,
            });
        }
    }

    Ok(candidates)
}

impl fmt::Display for C3BaselineError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for C3BaselineError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::experimental::boolean_policy_search::{
        BooleanPolicySearchCase, evaluate_policy_candidates,
    };

    #[test]
    fn calibrated_c3_envelope_yields_exactly_27_deterministic_controls() {
        let envelope = SynthesisSearchEnvelope::reference_c3();
        let first = generate_c3_single_rule_baselines(&envelope).unwrap();
        let second = generate_c3_single_rule_baselines(&envelope).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.len(), 27);
        assert_eq!(first[0].candidate_id, "tdi9.3-c3-stop-p0");
        assert_eq!(first[1].candidate_id, "tdi9.3-c3-verify-p0");
        assert_eq!(first[2].candidate_id, "tdi9.3-c3-backtrack-p0");
        assert_eq!(first.last().unwrap().candidate_id, "tdi9.3-c3-backtrack-p8");
    }

    #[test]
    fn generator_refuses_c2_envelope() {
        assert_eq!(
            generate_c3_single_rule_baselines(&SynthesisSearchEnvelope::reference_c2()),
            Err(C3BaselineError::C3EnvelopeRequired {
                actual: PolicyArm::C2AdaptiveStopping,
            })
        );
    }

    #[test]
    fn generated_candidates_use_continue_fallback_and_shared_evaluator() {
        let envelope = SynthesisSearchEnvelope::reference_c3();
        let candidates = generate_c3_single_rule_baselines(&envelope).unwrap();
        let cases = [
            BooleanPolicySearchCase {
                predicates: vec![true, false, false, false, false, false, false, false, false],
                expected_action: InferenceAction::Stop,
            },
            BooleanPolicySearchCase {
                predicates: vec![false; 9],
                expected_action: InferenceAction::Continue,
            },
        ];
        let evidence = evaluate_policy_candidates(&candidates, &cases, &envelope).unwrap();
        assert_eq!(evidence[0].candidate_id, "tdi9.3-c3-stop-p0");
        assert_eq!(evidence[0].action_mismatches, 0);
        assert_eq!(evidence[0].worst_case_complexity.predicate_reads(), 1);
        assert_eq!(evidence[0].worst_case_complexity.logical_ops(), 0);
        assert_eq!(evidence[0].worst_case_complexity.depth(), 0);
    }
}
