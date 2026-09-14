//! Exact non-final local sensitivity on the abstract C3 carrier.
//!
//! This graph is not a solver trajectory, a model robustness result, or a
//! TDI-9.1 observation freeze. A constant wrong policy can have zero sensitivity.

use core::fmt;

use super::adaptive_inference::InferenceAction;
use super::boolean_policy_carrier::{C3CarrierError, ValidatedC3PredicateRow};
use super::boolean_policy_synthesis::{
    BooleanPolicy, BooleanPolicyError, SynthesisSearchEnvelope, reference_c3_hand_action,
    reference_c3_predicates,
};

pub const C3_SENSITIVITY_SCHEMA: &str = "tdi9.3.c3-local-sensitivity.v1";
pub const C3_SENSITIVITY_AXES: [&str; 6] = [
    "base_stop", "verify_before_stop", "cadence_due", "checkpoint_available",
    "remaining_work", "verifier_state_substitution",
];

const BOOL_INDICES: [usize; 5] = [
    reference_c3_predicates::BASE_STOP,
    reference_c3_predicates::VERIFY_BEFORE_STOP,
    reference_c3_predicates::CADENCE_DUE,
    reference_c3_predicates::CHECKPOINT_AVAILABLE,
    reference_c3_predicates::REMAINING_WORK,
];
const VERIFIER_INDICES: [usize; 4] = [
    reference_c3_predicates::VERIFIER_VIOLATED,
    reference_c3_predicates::VERIFIER_SATISFIED,
    reference_c3_predicates::VERIFIER_INDETERMINATE,
    reference_c3_predicates::VERIFIER_ABSENT,
];

/// Counts are directed perturbations, not probabilities or wall-clock costs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct C3AxisSensitivity {
    pub admitted: u64,
    pub action_changes: u64,
    pub rejected: u64,
}

/// Complete bounded accounting; rejected neighbours are not unchanged actions.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct C3SensitivityReport {
    pub valid_anchors: u64,
    pub invalid_anchors: u64,
    pub unrecoverable_anchors: u64,
    pub reference_action_mismatches: u64,
    pub axes: [C3AxisSensitivity; 6],
    /// Source action x destination action: CONTINUE, VERIFY, BACKTRACK, STOP.
    pub transitions: [[u64; 4]; 4],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum C3SensitivityError {
    Policy(BooleanPolicyError),
    Carrier(C3CarrierError),
    MissingReferenceAction,
}

fn action_index(action: InferenceAction) -> usize {
    match action {
        InferenceAction::Continue => 0,
        InferenceAction::Verify => 1,
        InferenceAction::Backtrack => 2,
        InferenceAction::Stop => 3,
    }
}

fn record_edge(
    report: &mut C3SensitivityReport,
    actions: &[Option<InferenceAction>; 512],
    source: InferenceAction,
    destination: usize,
    axis: usize,
) {
    // All indices and counts are bounded by the closed nine-bit carrier.
    if let Some(target) = actions[destination] {
        report.axes[axis].admitted += 1;
        report.axes[axis].action_changes += u64::from(source != target);
        report.transitions[action_index(source)][action_index(target)] += 1;
    } else {
        report.axes[axis].rejected += 1;
    }
}

/// Audit every valid C3 row and every declared local change exactly once.
///
/// The first five axes flip one Boolean signal. The sixth substitutes the
/// current one-hot verifier state by each of the three other categorical states.
/// Substitution changes TWO encoding bits; it is not single-bit Hamming noise.
/// Every destination is subject to the same validated carrier as the source.
///
/// The report includes reference-action disagreement to prevent low sensitivity
/// being mistaken for quality. It never changes or selects a policy.
///
/// # Errors
///
/// Rejects policies outside the reference C3 envelope, propagates unexpected
/// carrier/evaluation failures, and rejects an inconsistent reference oracle.
pub fn c3_policy_sensitivity(
    policy: &BooleanPolicy,
) -> Result<C3SensitivityReport, C3SensitivityError> {
    SynthesisSearchEnvelope::reference_c3()
        .admits_policy(policy)
        .map_err(C3SensitivityError::Policy)?;
    let mut report = C3SensitivityReport::default();
    let mut actions = [None; 512];
    for (address, action) in actions.iter_mut().enumerate() {
        let raw: [bool; 9] = core::array::from_fn(|bit| address & (1usize << bit) != 0);
        match ValidatedC3PredicateRow::new(&raw) {
            Ok(row) => {
                report.valid_anchors += 1;
                let actual = row.decide(policy).map_err(C3SensitivityError::Carrier)?.action();
                let expected = reference_c3_hand_action(&raw)
                    .ok_or(C3SensitivityError::MissingReferenceAction)?;
                report.reference_action_mismatches += u64::from(actual != expected);
                *action = Some(actual);
            }
            Err(C3CarrierError::InvalidVerifierEncoding { .. }) => report.invalid_anchors += 1,
            Err(C3CarrierError::UnrecoverableViolation) => report.unrecoverable_anchors += 1,
            Err(error) => return Err(C3SensitivityError::Carrier(error)),
        }
    }
    for (address, &action) in actions.iter().enumerate() {
        let Some(source) = action else { continue };
        for (axis, &bit) in BOOL_INDICES.iter().enumerate() {
            record_edge(&mut report, &actions, source, address ^ (1usize << bit), axis);
        }
        let old_state = VERIFIER_INDICES.iter().copied()
            .find(|&bit| address & (1usize << bit) != 0)
            .ok_or(C3SensitivityError::MissingReferenceAction)?;
        for &new_state in &VERIFIER_INDICES {
            if new_state != old_state {
                let destination = address ^ (1usize << old_state) ^ (1usize << new_state);
                record_edge(&mut report, &actions, source, destination, 5);
            }
        }
    }
    Ok(report)
}

impl fmt::Display for C3SensitivityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for C3SensitivityError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::experimental::adaptive_inference::PolicyArm;
    use crate::experimental::boolean_policy_synthesis::{
        reference_c2_stop_policy, reference_c3_policy,
    };

    #[test]
    fn reference_partitions_all_nodes_and_all_declared_directed_changes() {
        let report = c3_policy_sensitivity(&reference_c3_policy().unwrap()).unwrap();
        assert_eq!(report.valid_anchors, 120);
        assert_eq!(report.invalid_anchors, 384);
        assert_eq!(report.unrecoverable_anchors, 8);
        assert_eq!(report.reference_action_mismatches, 0);
        let expected = [(120, 24, 0), (120, 16, 0), (120, 16, 0),
            (112, 16, 8), (112, 0, 8), (336, 284, 24)];
        for (axis, &(admitted, changed, rejected)) in report.axes.iter().zip(&expected) {
            assert_eq!((axis.admitted, axis.action_changes, axis.rejected),
                (admitted, changed, rejected));
        }
        assert_eq!(report.axes.iter().map(|axis| axis.admitted).sum::<u64>(), 920);
        assert_eq!(report.axes.iter().map(|axis| axis.rejected).sum::<u64>(), 40);
        assert_eq!(report.axes.iter().map(|axis| axis.action_changes).sum::<u64>(), 356);
    }

    #[test]
    fn reference_action_transitions_retain_direction_and_reverse_edges() {
        let report = c3_policy_sensitivity(&reference_c3_policy().unwrap()).unwrap();
        assert_eq!(report.transitions, [
            [244, 32, 28, 62], [32, 56, 8, 28], [28, 8, 64, 20], [62, 28, 20, 200],
        ]);
        for row in 0..4 {
            for column in 0..4 {
                assert_eq!(report.transitions[row][column], report.transitions[column][row]);
            }
        }
        assert_eq!(report.transitions.iter().flatten().sum::<u64>(), 920);
    }

    #[test]
    fn zero_sensitivity_does_not_imply_correctness_or_absorb_rejected_inputs() {
        let constant = BooleanPolicy::new(
            PolicyArm::C3VerificationRecovery, Vec::new(), InferenceAction::Continue,
        ).unwrap();
        let report = c3_policy_sensitivity(&constant).unwrap();
        assert!(report.axes.iter().all(|axis| axis.action_changes == 0));
        assert_eq!(report.reference_action_mismatches, 72);
        assert_eq!(report.transitions[0][0], 920);
        assert_eq!(report.axes.iter().map(|axis| axis.rejected).sum::<u64>(), 40);
    }

    #[test]
    fn audit_is_deterministic_nonmutating_and_rejects_wrong_policy_arm() {
        let policy = reference_c3_policy().unwrap();
        let before = policy.clone();
        assert_eq!(c3_policy_sensitivity(&policy), c3_policy_sensitivity(&policy));
        assert_eq!(policy, before);
        assert!(matches!(
            c3_policy_sensitivity(&reference_c2_stop_policy().unwrap()),
            Err(C3SensitivityError::Policy(_))
        ));
    }
}
