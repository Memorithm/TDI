//! Exact, non-final C3 search calibration with explicit excluded-row accounting.
//!
//! This example does not observe trajectories or select observation thresholds.
//! Invalid verifier encodings and unrecoverable verifier violations are counted
//! separately and never passed to the action-only candidate evaluator. This is
//! a harness boundary, not a claim that the raw policy IR rejects those rows.

use std::error::Error;

use tdi_ai::experimental::adaptive_inference::InferenceAction;
use tdi_ai::experimental::boolean_policy_search::{
    BooleanPolicySearchCandidate, BooleanPolicySearchCase, BooleanPolicySearchEvidence,
    evaluate_policy_candidates, policy_search_pareto_indices,
};
use tdi_ai::experimental::boolean_policy_search_c3::generate_c3_single_rule_baselines;
use tdi_ai::experimental::boolean_policy_synthesis::{
    REFERENCE_C3_PREDICATE_COUNT, SynthesisSearchEnvelope, reference_c3_hand_action,
    reference_c3_policy, reference_c3_verifier_encoding_well_formed,
};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

const FULL_POLICY_ID: &str = "tdi9.3-c3-reference-full";
const ACTIONS: [InferenceAction; 4] = [
    InferenceAction::Continue,
    InferenceAction::Verify,
    InferenceAction::Backtrack,
    InferenceAction::Stop,
];

#[derive(Debug, PartialEq, Eq)]
struct Carrier {
    cases: Vec<BooleanPolicySearchCase>,
    action_rows: Vec<usize>,
    rejection_rows: Vec<usize>,
    invalid_encoding_rows: Vec<usize>,
}

fn predicates_at(row: usize) -> [bool; REFERENCE_C3_PREDICATE_COUNT] {
    std::array::from_fn(|bit| row & (1usize << bit) != 0)
}

fn reference_carrier() -> Carrier {
    let mut carrier = Carrier {
        cases: Vec::new(),
        action_rows: Vec::new(),
        rejection_rows: Vec::new(),
        invalid_encoding_rows: Vec::new(),
    };
    for row in 0..(1usize << REFERENCE_C3_PREDICATE_COUNT) {
        let predicates = predicates_at(row);
        if !reference_c3_verifier_encoding_well_formed(&predicates) {
            carrier.invalid_encoding_rows.push(row);
        } else if let Some(expected_action) = reference_c3_hand_action(&predicates) {
            carrier.action_rows.push(row);
            carrier.cases.push(BooleanPolicySearchCase {
                predicates: predicates.to_vec(),
                expected_action,
            });
        } else {
            carrier.rejection_rows.push(row);
        }
    }
    carrier
}

fn candidate_population() -> Result<Vec<BooleanPolicySearchCandidate>> {
    let mut candidates =
        generate_c3_single_rule_baselines(&SynthesisSearchEnvelope::reference_c3())?;
    candidates.push(BooleanPolicySearchCandidate {
        candidate_id: FULL_POLICY_ID.to_owned(),
        policy: reference_c3_policy()?,
    });
    Ok(candidates)
}

fn action_index(action: InferenceAction) -> Result<usize> {
    ACTIONS
        .iter()
        .position(|known| *known == action)
        .ok_or_else(|| "action outside the declared C3 matrix vocabulary".into())
}

// Matrix rows are EXPECTED actions; columns are ACTUAL actions. All 16 cells
// are retained, including zeros. Counts never silently discard a rare action.
fn confusion_matrix(
    candidate: &BooleanPolicySearchCandidate,
    cases: &[BooleanPolicySearchCase],
) -> Result<[[u64; 4]; 4]> {
    let mut matrix = [[0u64; 4]; 4];
    for case in cases {
        let actual = candidate.policy.decide(&case.predicates)?.action();
        let cell = &mut matrix[action_index(case.expected_action)?][action_index(actual)?];
        *cell = cell.checked_add(1).ok_or("confusion matrix overflow")?;
    }
    Ok(matrix)
}

fn calibrate(
    carrier: &Carrier,
    candidates: &[BooleanPolicySearchCandidate],
) -> Result<Vec<BooleanPolicySearchEvidence>> {
    let evidence = evaluate_policy_candidates(
        candidates,
        &carrier.cases,
        &SynthesisSearchEnvelope::reference_c3(),
    )?;
    for (candidate, row) in candidates.iter().zip(&evidence) {
        let matrix = confusion_matrix(candidate, &carrier.cases)?;
        let total: u64 = matrix.iter().flatten().sum();
        let correct: u64 = (0..4).map(|index| matrix[index][index]).sum();
        if total != u64::try_from(carrier.cases.len())? || total - correct != row.action_mismatches
        {
            return Err("action matrix and shared search evidence disagree".into());
        }
    }
    Ok(evidence)
}

fn main() -> Result<()> {
    if std::env::args_os().len() != 1 {
        return Err("this fixed representation calibration accepts no arguments".into());
    }
    let carrier = reference_carrier();
    let candidates = candidate_population()?;
    let evidence = calibrate(&carrier, &candidates)?;
    let frontier = policy_search_pareto_indices(&evidence)?;
    println!("# schema=tdi9.3.c3-calibration.v1; evidence=EXACT_REPRESENTATION_ONLY");
    println!("# No trajectories, observation freeze, final material or execution authorization.");
    println!("# Invalid/rejected rows are excluded by the harness, not rejected by the raw IR.");
    println!(
        "# action_rows={}; unrecoverable_rejections={}; invalid_verifier_encodings={}",
        carrier.action_rows.len(),
        carrier.rejection_rows.len(),
        carrier.invalid_encoding_rows.len(),
    );
    println!("# matrix order=CONTINUE,VERIFY,BACKTRACK,STOP; rows=expected; columns=actual");
    println!("candidate\tmismatches\treads\tlogical_ops\tworst_reads\tworst_ops\tdepth\tpareto");
    for (index, row) in evidence.iter().enumerate() {
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            row.candidate_id,
            row.action_mismatches,
            row.decision_predicate_reads,
            row.decision_logical_ops,
            row.worst_case_complexity.predicate_reads(),
            row.worst_case_complexity.logical_ops(),
            row.worst_case_complexity.depth(),
            frontier.contains(&index),
        );
        println!(
            "# matrix {} {:?}",
            row.candidate_id,
            confusion_matrix(&candidates[index], &carrier.cases)?,
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn carrier_partitions_all_512_rows_without_silent_exclusions() {
        let carrier = reference_carrier();
        assert_eq!(carrier.action_rows.len(), 120);
        assert_eq!(carrier.cases.len(), 120);
        assert_eq!(carrier.rejection_rows.len(), 8);
        assert_eq!(carrier.invalid_encoding_rows.len(), 384);
        let mut occurrences = [0u8; 512];
        for &row in carrier
            .action_rows
            .iter()
            .chain(&carrier.rejection_rows)
            .chain(&carrier.invalid_encoding_rows)
        {
            occurrences[row] += 1;
        }
        assert!(occurrences.iter().all(|count| *count == 1));
        let counts: Vec<usize> = ACTIONS
            .iter()
            .map(|action| {
                carrier
                    .cases
                    .iter()
                    .filter(|case| case.expected_action == *action)
                    .count()
            })
            .collect();
        assert_eq!(counts, vec![48, 16, 16, 40]);
    }

    #[test]
    fn every_rejection_is_a_well_formed_unrecoverable_violation() {
        let carrier = reference_carrier();
        for row in carrier.rejection_rows {
            let p = predicates_at(row);
            assert!(reference_c3_verifier_encoding_well_formed(&p));
            // Independent operational criterion: violated, no checkpoint, no work.
            assert!(p[5] && !p[3] && !p[4]);
            assert_eq!(reference_c3_hand_action(&p), None);
        }
        for row in carrier.invalid_encoding_rows {
            let p = predicates_at(row);
            assert_ne!(p[5..9].iter().filter(|value| **value).count(), 1);
        }
    }

    #[test]
    fn full_reference_is_exact_and_reports_actual_and_structural_cost_separately() {
        let carrier = reference_carrier();
        let candidates = candidate_population().unwrap();
        let evidence = calibrate(&carrier, &candidates).unwrap();
        let full = evidence.last().unwrap();
        assert_eq!(full.candidate_id, FULL_POLICY_ID);
        assert_eq!(full.action_mismatches, 0);
        assert_eq!(full.decision_predicate_reads, 784);
        assert_eq!(full.decision_logical_ops, 328);
        assert_eq!(full.worst_case_complexity.predicate_reads(), 13);
        assert_eq!(full.worst_case_complexity.logical_ops(), 6);
        assert_eq!(full.worst_case_complexity.depth(), 2);
    }

    #[test]
    fn simple_baselines_expose_missing_verify_and_backtrack_actions() {
        let carrier = reference_carrier();
        let candidates = candidate_population().unwrap();
        let evidence = calibrate(&carrier, &candidates).unwrap();
        assert_eq!(evidence.len(), 28);
        let best_error = evidence[..27]
            .iter()
            .map(|row| row.action_mismatches)
            .min()
            .unwrap();
        assert_eq!(best_error, 40);
        let best: Vec<usize> = (0..27)
            .filter(|&index| evidence[index].action_mismatches == best_error)
            .collect();
        assert_eq!(best, vec![18]);
        assert_eq!(evidence[18].candidate_id, "tdi9.3-c3-stop-p6");
        assert_eq!(
            confusion_matrix(&candidates[18], &carrier.cases).unwrap(),
            [[48, 0, 0, 0], [16, 0, 0, 0], [16, 0, 0, 0], [8, 0, 0, 32]],
        );
        assert_eq!(
            policy_search_pareto_indices(&evidence).unwrap(),
            vec![18, 27]
        );
    }

    #[test]
    fn calibration_is_deterministic_and_uses_no_dataset_or_runtime_state() {
        let first = reference_carrier();
        let second = reference_carrier();
        assert_eq!(first, second);
        let candidates = candidate_population().unwrap();
        assert_eq!(
            calibrate(&first, &candidates).unwrap(),
            calibrate(&second, &candidates).unwrap(),
        );
    }
}
