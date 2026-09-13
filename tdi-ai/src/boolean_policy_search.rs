//! Non-final bounded candidate generation and screening for TDI-9.3.1.
//!
//! This module operates only on already-declared Boolean predicate vectors. It
//! does not choose TDI-9.1 observables, derive thresholds, access final material,
//! or authorize TDI-9.2. Its purpose is to make the first TDI-9.3.1 search step
//! executable once a leakage-safe predicate table is supplied by an authorized
//! Development/Validation experiment.

use std::collections::BTreeSet;
use std::fmt;

use super::adaptive_inference::{InferenceAction, PolicyArm};
use super::boolean_policy_synthesis::{
    BooleanActionRule, BooleanComplexity, BooleanExpr, BooleanPolicy, BooleanPolicyError,
    SynthesisSearchEnvelope,
};

/// Deterministic search candidate with experiment-owned identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BooleanPolicySearchCandidate {
    pub candidate_id: String,
    pub policy: BooleanPolicy,
}

/// One leakage-safe labelled predicate row supplied to the search layer.
///
/// The expected action is a Development/Validation search target owned by the
/// calling experiment. This type deliberately contains no raw trajectory,
/// evaluator target, future state, seed identity, or final/holdout material.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BooleanPolicySearchCase {
    pub predicates: Vec<bool>,
    pub expected_action: InferenceAction,
}

/// Exact non-scalar search evidence for one candidate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BooleanPolicySearchEvidence {
    pub candidate_id: String,
    pub action_mismatches: u64,
    pub decision_predicate_reads: u64,
    pub decision_logical_ops: u64,
    pub worst_case_complexity: BooleanComplexity,
}

/// Fail-closed errors for the non-final TDI-9.3.1 search layer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BooleanPolicySearchError {
    C2EnvelopeRequired {
        actual: PolicyArm,
    },
    EmptyCandidateSet,
    EmptyCaseSet,
    EmptyCandidateId {
        index: usize,
    },
    DuplicateCandidateId {
        candidate_id: String,
    },
    PredicateArityMismatch {
        case_index: usize,
        expected: usize,
        actual: usize,
    },
    ExpectedActionForbidden {
        case_index: usize,
        arm: PolicyArm,
        action: InferenceAction,
    },
    CounterOverflow,
    Policy(BooleanPolicyError),
}

/// Generate the preregistered simple C2 baseline grammars required by TDI-9.3.1.
///
/// The returned population contains, in deterministic order:
///
/// 1. every single-predicate STOP rule;
/// 2. every two-distinct-predicate conjunction STOP rule;
/// 3. every two-distinct-predicate disjunction STOP rule.
///
/// Every candidate uses `CONTINUE` fallback and is re-admitted through the
/// supplied [`SynthesisSearchEnvelope`]. This is a bounded baseline generator,
/// not a claim that the grammar is sufficient or optimal.
///
/// # Errors
///
/// Rejects non-C2 envelopes and propagates fail-closed Boolean policy/envelope
/// validation errors.
pub fn generate_c2_baseline_candidates(
    envelope: &SynthesisSearchEnvelope,
) -> Result<Vec<BooleanPolicySearchCandidate>, BooleanPolicySearchError> {
    if envelope.arm() != PolicyArm::C2AdaptiveStopping {
        return Err(BooleanPolicySearchError::C2EnvelopeRequired {
            actual: envelope.arm(),
        });
    }

    let mut candidates = Vec::new();
    let predicate_count = envelope.predicate_count();

    for index in 0..predicate_count {
        candidates.push(make_c2_candidate(
            format!("tdi9.3-c2-single-p{index}"),
            BooleanExpr::Predicate(index),
            envelope,
        )?);
    }

    for left in 0..predicate_count {
        for right in (left + 1)..predicate_count {
            candidates.push(make_c2_candidate(
                format!("tdi9.3-c2-and-p{left}-p{right}"),
                BooleanExpr::And(
                    Box::new(BooleanExpr::Predicate(left)),
                    Box::new(BooleanExpr::Predicate(right)),
                ),
                envelope,
            )?);
        }
    }

    for left in 0..predicate_count {
        for right in (left + 1)..predicate_count {
            candidates.push(make_c2_candidate(
                format!("tdi9.3-c2-or-p{left}-p{right}"),
                BooleanExpr::Or(
                    Box::new(BooleanExpr::Predicate(left)),
                    Box::new(BooleanExpr::Predicate(right)),
                ),
                envelope,
            )?);
        }
    }

    Ok(candidates)
}

fn make_c2_candidate(
    candidate_id: String,
    expression: BooleanExpr,
    envelope: &SynthesisSearchEnvelope,
) -> Result<BooleanPolicySearchCandidate, BooleanPolicySearchError> {
    let policy = BooleanPolicy::new(
        PolicyArm::C2AdaptiveStopping,
        vec![BooleanActionRule::new(expression, InferenceAction::Stop)],
        InferenceAction::Continue,
    )
    .map_err(BooleanPolicySearchError::Policy)?;
    envelope
        .admits_policy(&policy)
        .map_err(BooleanPolicySearchError::Policy)?;
    Ok(BooleanPolicySearchCandidate {
        candidate_id,
        policy,
    })
}

/// Evaluate a finite Boolean policy population on one supplied search table.
///
/// Candidate order is preserved. No scalar score is formed: action errors,
/// actually incurred decision reads/ops, and worst-case structural complexity
/// remain separate evidence components.
///
/// # Errors
///
/// Rejects empty or duplicate candidate identities, empty case sets, predicate
/// arity drift, expected actions forbidden by the envelope arm, arithmetic
/// overflow, or any candidate that fails policy/envelope validation.
pub fn evaluate_policy_candidates(
    candidates: &[BooleanPolicySearchCandidate],
    cases: &[BooleanPolicySearchCase],
    envelope: &SynthesisSearchEnvelope,
) -> Result<Vec<BooleanPolicySearchEvidence>, BooleanPolicySearchError> {
    if candidates.is_empty() {
        return Err(BooleanPolicySearchError::EmptyCandidateSet);
    }
    if cases.is_empty() {
        return Err(BooleanPolicySearchError::EmptyCaseSet);
    }

    let mut ids = BTreeSet::new();
    for (index, candidate) in candidates.iter().enumerate() {
        if candidate.candidate_id.is_empty() {
            return Err(BooleanPolicySearchError::EmptyCandidateId { index });
        }
        if !ids.insert(candidate.candidate_id.as_str()) {
            return Err(BooleanPolicySearchError::DuplicateCandidateId {
                candidate_id: candidate.candidate_id.clone(),
            });
        }
        envelope
            .admits_policy(&candidate.policy)
            .map_err(BooleanPolicySearchError::Policy)?;
    }

    for (case_index, case) in cases.iter().enumerate() {
        if case.predicates.len() != envelope.predicate_count() {
            return Err(BooleanPolicySearchError::PredicateArityMismatch {
                case_index,
                expected: envelope.predicate_count(),
                actual: case.predicates.len(),
            });
        }
        if !envelope.arm().allows(case.expected_action) {
            return Err(BooleanPolicySearchError::ExpectedActionForbidden {
                case_index,
                arm: envelope.arm(),
                action: case.expected_action,
            });
        }
    }

    let mut evidence = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        let mut action_mismatches = 0u64;
        let mut decision_predicate_reads = 0u64;
        let mut decision_logical_ops = 0u64;

        for case in cases {
            let decision = candidate
                .policy
                .decide(&case.predicates)
                .map_err(BooleanPolicySearchError::Policy)?;
            if decision.action() != case.expected_action {
                action_mismatches = action_mismatches
                    .checked_add(1)
                    .ok_or(BooleanPolicySearchError::CounterOverflow)?;
            }
            decision_predicate_reads = decision_predicate_reads
                .checked_add(decision.predicate_reads())
                .ok_or(BooleanPolicySearchError::CounterOverflow)?;
            decision_logical_ops = decision_logical_ops
                .checked_add(decision.logical_ops())
                .ok_or(BooleanPolicySearchError::CounterOverflow)?;
        }

        let worst_case_complexity = candidate
            .policy
            .worst_case_complexity()
            .map_err(BooleanPolicySearchError::Policy)?;
        evidence.push(BooleanPolicySearchEvidence {
            candidate_id: candidate.candidate_id.clone(),
            action_mismatches,
            decision_predicate_reads,
            decision_logical_ops,
            worst_case_complexity,
        });
    }

    Ok(evidence)
}

/// Return non-dominated candidate indices while preserving original order.
///
/// All components are minimized. Equal vectors remain provenance-distinct and
/// therefore both survive the frontier.
///
/// # Errors
///
/// Rejects empty evidence, empty ids, or duplicate candidate ids.
pub fn policy_search_pareto_indices(
    evidence: &[BooleanPolicySearchEvidence],
) -> Result<Vec<usize>, BooleanPolicySearchError> {
    if evidence.is_empty() {
        return Err(BooleanPolicySearchError::EmptyCandidateSet);
    }
    let mut ids = BTreeSet::new();
    for (index, row) in evidence.iter().enumerate() {
        if row.candidate_id.is_empty() {
            return Err(BooleanPolicySearchError::EmptyCandidateId { index });
        }
        if !ids.insert(row.candidate_id.as_str()) {
            return Err(BooleanPolicySearchError::DuplicateCandidateId {
                candidate_id: row.candidate_id.clone(),
            });
        }
    }

    Ok((0..evidence.len())
        .filter(|&candidate_index| {
            !(0..evidence.len()).any(|other_index| {
                other_index != candidate_index
                    && dominates(&evidence[other_index], &evidence[candidate_index])
            })
        })
        .collect())
}

fn dominates(left: &BooleanPolicySearchEvidence, right: &BooleanPolicySearchEvidence) -> bool {
    let left_worst = left.worst_case_complexity;
    let right_worst = right.worst_case_complexity;
    let no_worse = left.action_mismatches <= right.action_mismatches
        && left.decision_predicate_reads <= right.decision_predicate_reads
        && left.decision_logical_ops <= right.decision_logical_ops
        && left_worst.predicate_reads() <= right_worst.predicate_reads()
        && left_worst.logical_ops() <= right_worst.logical_ops()
        && left_worst.depth() <= right_worst.depth();
    let strictly_better = left.action_mismatches < right.action_mismatches
        || left.decision_predicate_reads < right.decision_predicate_reads
        || left.decision_logical_ops < right.decision_logical_ops
        || left_worst.predicate_reads() < right_worst.predicate_reads()
        || left_worst.logical_ops() < right_worst.logical_ops()
        || left_worst.depth() < right_worst.depth();
    no_worse && strictly_better
}

impl fmt::Display for BooleanPolicySearchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for BooleanPolicySearchError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn every_five_predicate_case() -> Vec<BooleanPolicySearchCase> {
        (0..32usize)
            .map(|mask| {
                let predicates = (0..5)
                    .map(|index| ((mask >> index) & 1) == 1)
                    .collect::<Vec<_>>();
                BooleanPolicySearchCase {
                    expected_action: if predicates[0] {
                        InferenceAction::Stop
                    } else {
                        InferenceAction::Continue
                    },
                    predicates,
                }
            })
            .collect()
    }

    #[test]
    fn c2_baseline_generator_is_deterministic_and_complete_for_declared_grammar() {
        let envelope = SynthesisSearchEnvelope::reference_c2();
        let first = generate_c2_baseline_candidates(&envelope).unwrap();
        let second = generate_c2_baseline_candidates(&envelope).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.len(), 25);
        assert_eq!(first[0].candidate_id, "tdi9.3-c2-single-p0");
        assert_eq!(first[4].candidate_id, "tdi9.3-c2-single-p4");
        assert_eq!(first[5].candidate_id, "tdi9.3-c2-and-p0-p1");
        assert_eq!(first.last().unwrap().candidate_id, "tdi9.3-c2-or-p3-p4");
    }

    #[test]
    fn c2_generator_refuses_c3_envelope() {
        assert_eq!(
            generate_c2_baseline_candidates(&SynthesisSearchEnvelope::reference_c3()),
            Err(BooleanPolicySearchError::C2EnvelopeRequired {
                actual: PolicyArm::C3VerificationRecovery,
            })
        );
    }

    #[test]
    fn search_evidence_finds_exact_single_predicate_rule_without_scalar_score() {
        let envelope = SynthesisSearchEnvelope::reference_c2();
        let candidates = generate_c2_baseline_candidates(&envelope).unwrap();
        let evidence =
            evaluate_policy_candidates(&candidates, &every_five_predicate_case(), &envelope)
                .unwrap();

        assert_eq!(evidence[0].candidate_id, "tdi9.3-c2-single-p0");
        assert_eq!(evidence[0].action_mismatches, 0);
        assert_eq!(evidence[0].worst_case_complexity.predicate_reads(), 1);
        assert_eq!(evidence[0].worst_case_complexity.logical_ops(), 0);
        assert_eq!(evidence[0].worst_case_complexity.depth(), 0);
        assert_eq!(policy_search_pareto_indices(&evidence).unwrap(), vec![0]);
    }

    #[test]
    fn search_rejects_predicate_arity_drift() {
        let envelope = SynthesisSearchEnvelope::reference_c2();
        let candidates = generate_c2_baseline_candidates(&envelope).unwrap();
        let cases = vec![BooleanPolicySearchCase {
            predicates: vec![false; 4],
            expected_action: InferenceAction::Continue,
        }];
        assert_eq!(
            evaluate_policy_candidates(&candidates, &cases, &envelope),
            Err(BooleanPolicySearchError::PredicateArityMismatch {
                case_index: 0,
                expected: 5,
                actual: 4,
            })
        );
    }

    #[test]
    fn search_rejects_duplicate_candidate_identity() {
        let envelope = SynthesisSearchEnvelope::reference_c2();
        let mut candidates = generate_c2_baseline_candidates(&envelope).unwrap();
        candidates[1].candidate_id = candidates[0].candidate_id.clone();
        assert!(matches!(
            evaluate_policy_candidates(&candidates, &every_five_predicate_case(), &envelope),
            Err(BooleanPolicySearchError::DuplicateCandidateId { .. })
        ));
    }
}
