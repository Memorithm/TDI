//! Exact non-final calibration for the TDI-9.3.1 bounded C2 search substrate.
//!
//! This module uses only the already-qualified hand-written C2 Boolean formula
//! as an independent representation oracle. It does not consume trajectories,
//! choose TDI-9.1 observations or thresholds, access Development/Validation
//! datasets, or authorize TDI-9.2.

use super::adaptive_inference::InferenceAction;
use super::boolean_policy_search::{
    BooleanPolicySearchCandidate, BooleanPolicySearchCase, BooleanPolicySearchError,
    BooleanPolicySearchEvidence, evaluate_policy_candidates, generate_c2_baseline_candidates,
    policy_search_pareto_indices,
};
use super::boolean_policy_synthesis::{
    REFERENCE_C2_PREDICATE_COUNT, SynthesisSearchEnvelope, reference_c2_hand_stop,
    reference_c2_stop_policy,
};

/// Exhaustive `2^5` representation-calibration table for the documented C2
/// hand Boolean.
///
/// Predicate index meanings remain the non-pinning TDI-9.3 representation
/// labels declared in `boolean_policy_synthesis`. This function does not derive
/// predicates from trajectory observations.
#[must_use]
pub fn reference_c2_search_cases() -> Vec<BooleanPolicySearchCase> {
    let row_count = 1usize << REFERENCE_C2_PREDICATE_COUNT;
    (0..row_count)
        .map(|mask| {
            let mut fixed = [false; REFERENCE_C2_PREDICATE_COUNT];
            for (index, value) in fixed.iter_mut().enumerate() {
                *value = ((mask >> index) & 1) == 1;
            }
            BooleanPolicySearchCase {
                predicates: fixed.to_vec(),
                expected_action: if reference_c2_hand_stop(&fixed) {
                    InferenceAction::Stop
                } else {
                    InferenceAction::Continue
                },
            }
        })
        .collect()
}

/// Evaluate the preregistered simple TDI-9.3.1 C2 grammar against the exact
/// hand-formula representation oracle.
///
/// This is a calibration of the search machinery and grammar expressivity, not
/// adaptive-inference evidence. In particular, a mismatch count here says only
/// that a Boolean expression differs from the documented C2 formula on the
/// abstract predicate truth table.
///
/// # Errors
///
/// Propagates fail-closed search/envelope/evaluation errors.
pub fn evaluate_reference_c2_simple_baselines(
) -> Result<Vec<BooleanPolicySearchEvidence>, BooleanPolicySearchError> {
    let envelope = SynthesisSearchEnvelope::reference_c2();
    let candidates = generate_c2_baseline_candidates(&envelope)?;
    evaluate_policy_candidates(&candidates, &reference_c2_search_cases(), &envelope)
}

/// Evaluate the calibrated full hand-written C2 Boolean through the same search
/// evidence path used by bounded candidates.
///
/// # Errors
///
/// Propagates policy construction or search evaluation errors.
pub fn evaluate_reference_c2_full_policy(
) -> Result<BooleanPolicySearchEvidence, BooleanPolicySearchError> {
    let envelope = SynthesisSearchEnvelope::reference_c2();
    let policy = reference_c2_stop_policy().map_err(BooleanPolicySearchError::Policy)?;
    let candidates = [BooleanPolicySearchCandidate {
        candidate_id: "tdi9.3-c2-reference-full".to_owned(),
        policy,
    }];
    let mut evidence = evaluate_policy_candidates(&candidates, &reference_c2_search_cases(), &envelope)?;
    evidence.pop().ok_or(BooleanPolicySearchError::EmptyCandidateSet)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calibration_table_is_complete_and_independently_labelled() {
        let cases = reference_c2_search_cases();
        assert_eq!(cases.len(), 32);
        assert!(
            cases
                .iter()
                .all(|case| case.predicates.len() == REFERENCE_C2_PREDICATE_COUNT)
        );
        assert_eq!(
            cases
                .iter()
                .filter(|case| case.expected_action == InferenceAction::Stop)
                .count(),
            9
        );
    }

    #[test]
    fn simple_grammar_has_exact_one_row_best_approximation() {
        let evidence = evaluate_reference_c2_simple_baselines().unwrap();
        assert_eq!(evidence.len(), 25);

        let minimum = evidence
            .iter()
            .map(|row| row.action_mismatches)
            .min()
            .unwrap();
        assert_eq!(minimum, 1);

        let best = evidence
            .iter()
            .filter(|row| row.action_mismatches == minimum)
            .map(|row| row.candidate_id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(best, vec!["tdi9.3-c2-and-p0-p1"]);
    }

    #[test]
    fn simple_grammar_pareto_front_preserves_accuracy_cost_tradeoff() {
        let evidence = evaluate_reference_c2_simple_baselines().unwrap();
        let frontier = policy_search_pareto_indices(&evidence).unwrap();
        let ids = frontier
            .iter()
            .map(|&index| evidence[index].candidate_id.as_str())
            .collect::<Vec<_>>();
        assert_eq!(ids, vec!["tdi9.3-c2-single-p0", "tdi9.3-c2-and-p0-p1"]);
    }

    #[test]
    fn full_reference_policy_is_exact_under_same_evidence_path() {
        let evidence = evaluate_reference_c2_full_policy().unwrap();
        assert_eq!(evidence.action_mismatches, 0);
        assert_eq!(evidence.worst_case_complexity.predicate_reads(), 5);
        assert_eq!(evidence.worst_case_complexity.logical_ops(), 4);
        assert_eq!(evidence.worst_case_complexity.depth(), 4);
    }
}
