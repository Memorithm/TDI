//! Deterministic TDI-9.1 reference execution for the non-final P1/P2/P3 tasks.
//!
//! This module sits between the already-frozen action/accounting contracts and
//! future C0/C1/C2/C3 policy implementations. It owns solver transitions,
//! independent verifier semantics, one bounded P3 checkpoint, replay accounting
//! and post-STOP evaluation. It does not choose actions or inspect evaluator
//! targets before an arm has stopped.

use core::fmt;

use crate::adaptive_inference::{
    AdaptiveInferenceError, ComputeComponent, InferenceAction, PolicyArm, PolicyObservation,
    ResourceEnvelope, ResourceMeter, ResourceUsage, VerifierSignal, validate_action,
};
use crate::adaptive_task_generators::{
    AdaptiveTaskFamily, BinaryTarget, EvaluatorRecord, EvaluatorTarget, ForkBranch, ForkEvent,
    PolicyTask,
};

const P1_SOLVER_OPS: u64 = 4;
const P3_CHOICE_OPS: u64 = 2;
const P3_EVIDENCE_OPS: u64 = 6;
const P3_ELIMINATE_OPS: u64 = 3;
const P1_VERIFIER_BASE_OPS: u64 = 3;
const P1_VERIFIER_EVENT_OPS: u64 = 2;
const P2_SOLVER_BASE_OPS: u64 = 5;
const P2_SOLVER_BIT_OPS: u64 = 2;
const P2_VERIFIER_ROW_BASE_OPS: u64 = 4;
const P2_VERIFIER_BIT_OPS: u64 = 2;
const P3_VERIFIER_BASE_OPS: u64 = 4;
const P3_VERIFIER_EVENT_OPS: u64 = 2;

// Canonical packed logical P3 checkpoint:
// cursor:u64 + left:i64 + right:i64 + eliminated:u8 + committed:u8 + forbidden:u8.
pub const P3_CHECKPOINT_BYTES: u64 = 27;

// The memory contract is a deterministic logical reference model, not a claim
// about compiler stack layout. Immutable task input is common to all arms and
// excluded from arm working-memory accounting.
const RESOURCE_METER_SHADOW_BITS: u64 = 768;
const EXECUTION_METADATA_BITS: u64 = 456;
const TRANSACTION_METADATA_SHADOW_BITS: u64 = 456;
const ACTION_SCRATCH_BITS: u64 = 256;

/// Current deterministic solver candidate. No evaluator label is embedded.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SolverCandidate {
    P1(BinaryTarget),
    P2 { bits: u64, width: u8 },
    P3(ForkBranch),
}

impl SolverCandidate {
    #[must_use]
    pub const fn family(self) -> AdaptiveTaskFamily {
        match self {
            Self::P1(_) => AdaptiveTaskFamily::StagedEvidenceAccumulation,
            Self::P2 { .. } => AdaptiveTaskFamily::VerificationSensitiveInference,
            Self::P3(_) => AdaptiveTaskFamily::RecoverableDeceptiveFork,
        }
    }
}

/// Checkpoint copy traffic required by the frozen TDI-9 accounting contract.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CheckpointTraffic {
    store_bytes: u64,
    restore_bytes: u64,
}

impl CheckpointTraffic {
    #[must_use]
    pub const fn store_bytes(self) -> u64 {
        self.store_bytes
    }

    #[must_use]
    pub const fn restore_bytes(self) -> u64 {
        self.restore_bytes
    }

    pub fn total_bytes(self) -> Result<u64, ReferenceExecutionError> {
        self.store_bytes
            .checked_add(self.restore_bytes)
            .ok_or(ReferenceExecutionError::CheckpointTrafficOverflow)
    }

    fn add_store(&mut self, bytes: u64) -> Result<(), ReferenceExecutionError> {
        self.store_bytes = self
            .store_bytes
            .checked_add(bytes)
            .ok_or(ReferenceExecutionError::CheckpointTrafficOverflow)?;
        Ok(())
    }

    fn add_restore(&mut self, bytes: u64) -> Result<(), ReferenceExecutionError> {
        self.restore_bytes = self
            .restore_bytes
            .checked_add(bytes)
            .ok_or(ReferenceExecutionError::CheckpointTrafficOverflow)?;
        Ok(())
    }
}

/// Complete execution accounting snapshot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExecutionAccounting {
    usage: ResourceUsage,
    checkpoint_traffic: CheckpointTraffic,
}

impl ExecutionAccounting {
    #[must_use]
    pub const fn usage(self) -> ResourceUsage {
        self.usage
    }

    #[must_use]
    pub const fn checkpoint_traffic(self) -> CheckpointTraffic {
        self.checkpoint_traffic
    }
}

/// Candidate emitted only by an explicit STOP action.
///
/// Post-hoc evaluator target access requires this type rather than a live
/// [`ReferenceExecution`], making the STOP boundary explicit in the API.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StoppedCandidate {
    candidate: SolverCandidate,
    step_index: u64,
    accounting: ExecutionAccounting,
}

impl StoppedCandidate {
    #[must_use]
    pub const fn candidate(self) -> SolverCandidate {
        self.candidate
    }

    #[must_use]
    pub const fn step_index(self) -> u64 {
        self.step_index
    }

    #[must_use]
    pub const fn accounting(self) -> ExecutionAccounting {
        self.accounting
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SolverState {
    P1 {
        cursor: u64,
        sum: i64,
    },
    P2 {
        cursor: u64,
        bits: u64,
        width: u8,
    },
    P3 {
        cursor: u64,
        left_score: i64,
        right_score: i64,
        eliminated_mask: u8,
        committed: Option<ForkBranch>,
        forbidden: Option<ForkBranch>,
    },
}

impl SolverState {
    #[must_use]
    const fn cursor(self) -> u64 {
        match self {
            Self::P1 { cursor, .. } | Self::P2 { cursor, .. } | Self::P3 { cursor, .. } => cursor,
        }
    }

    #[must_use]
    const fn logical_bits(self) -> u64 {
        match self {
            Self::P1 { .. } => 130,
            Self::P2 { .. } => 138,
            Self::P3 { .. } => 202,
        }
    }

    fn with_forbidden_branch(self, branch: ForkBranch) -> Result<Self, ReferenceExecutionError> {
        match self {
            Self::P3 {
                cursor,
                left_score,
                right_score,
                eliminated_mask,
                committed,
                ..
            } => Ok(Self::P3 {
                cursor,
                left_score,
                right_score,
                eliminated_mask,
                committed,
                forbidden: Some(branch),
            }),
            _ => Err(ReferenceExecutionError::BacktrackUnsupportedForTask),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Checkpoint {
    state: SolverState,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct SolverSummary {
    state_delta: f64,
    residual: f64,
    score_margin: f64,
}

impl SolverSummary {
    const ZERO: Self = Self {
        state_delta: 0.0,
        residual: 0.0,
        score_margin: 0.0,
    };
}

/// Deterministic action executor for one policy-visible task instance.
pub struct ReferenceExecution {
    arm: PolicyArm,
    task: PolicyTask,
    solver: SolverState,
    meter: ResourceMeter,
    checkpoint_traffic: CheckpointTraffic,
    checkpoint: Option<Checkpoint>,
    action_count: u64,
    high_water_step: u64,
    replay_until: Option<u64>,
    last_verifier_signal: Option<VerifierSignal>,
    last_rejected_branch: Option<ForkBranch>,
    last_summary: SolverSummary,
    policy_memory_bits: u64,
    stopped: bool,
}

impl ReferenceExecution {
    /// Build a live execution from policy-visible task input only.
    pub fn new(
        arm: PolicyArm,
        task: PolicyTask,
        envelope: ResourceEnvelope,
    ) -> Result<Self, ReferenceExecutionError> {
        let solver = initial_solver(&task)?;
        let mut meter = ResourceMeter::new(envelope);
        let residual = task_len_u64(&task)? as f64;
        let last_summary = SolverSummary {
            residual,
            ..SolverSummary::ZERO
        };
        set_memory_state(&mut meter, solver, 0, None, 0)?;
        Ok(Self {
            arm,
            task,
            solver,
            meter,
            checkpoint_traffic: CheckpointTraffic::default(),
            checkpoint: None,
            action_count: 0,
            high_water_step: 0,
            replay_until: None,
            last_verifier_signal: None,
            last_rejected_branch: None,
            last_summary,
            policy_memory_bits: 0,
            stopped: false,
        })
    }

    #[must_use]
    pub const fn arm(&self) -> PolicyArm {
        self.arm
    }

    #[must_use]
    pub fn task(&self) -> &PolicyTask {
        &self.task
    }

    #[must_use]
    pub const fn step_index(&self) -> u64 {
        self.solver.cursor()
    }

    pub fn current_candidate(&self) -> Result<SolverCandidate, ReferenceExecutionError> {
        solver_candidate(self.solver)
    }

    #[must_use]
    pub const fn accounting(&self) -> ExecutionAccounting {
        ExecutionAccounting {
            usage: self.meter.usage(),
            checkpoint_traffic: self.checkpoint_traffic,
        }
    }

    #[must_use]
    pub const fn checkpoint_available(&self) -> bool {
        self.checkpoint.is_some()
    }

    /// Current/past-only observation for the selected arm.
    pub fn observation(&self) -> Result<PolicyObservation, ReferenceExecutionError> {
        let used = self.meter.usage().total_compute_ops()?;
        let remaining_compute_ops = self
            .meter
            .envelope()
            .max_compute_ops()
            .checked_sub(used)
            .ok_or(ReferenceExecutionError::AccountingInvariant)?;
        let verifier_signal = if self.arm == PolicyArm::C3VerificationRecovery {
            self.last_verifier_signal
        } else {
            None
        };
        let available_checkpoints = if self.arm == PolicyArm::C3VerificationRecovery
            && self.checkpoint.is_some()
        {
            1
        } else {
            0
        };
        Ok(PolicyObservation::new(
            self.solver.cursor(),
            remaining_compute_ops,
            self.last_summary.state_delta,
            self.last_summary.residual,
            self.last_summary.score_margin,
            self.action_count,
            verifier_signal,
            available_checkpoints,
        )?
        .validate_for_arm(self.arm)?)
    }

    /// Charge a future policy implementation without embedding policy logic in
    /// this execution layer. The operation and policy-memory updates are atomic.
    pub fn charge_policy_decision(
        &mut self,
        operations: u64,
        policy_memory_bits: u64,
    ) -> Result<(), ReferenceExecutionError> {
        self.ensure_running()?;
        let mut meter = self.meter;
        meter.charge_compute(ComputeComponent::PolicyDecision, operations)?;
        let temporary = transaction_temporary_bits(self.solver, 0)?;
        set_memory_state(
            &mut meter,
            self.solver,
            policy_memory_bits,
            self.checkpoint,
            temporary,
        )?;
        self.meter = meter;
        self.policy_memory_bits = policy_memory_bits;
        Ok(())
    }

    /// Execute one deterministic solver transition. On C3/P3, the first
    /// ChoicePoint stores exactly one checkpoint before the transition.
    pub fn continue_step(&mut self) -> Result<SolverCandidate, ReferenceExecutionError> {
        self.ensure_running()?;
        validate_action(self.arm, InferenceAction::Continue)?;
        if self.solver.cursor() >= task_len_u64(&self.task)? {
            return Err(ReferenceExecutionError::SolverExhausted);
        }

        let mut meter = self.meter;
        let mut traffic = self.checkpoint_traffic;
        let mut checkpoint = self.checkpoint;
        let mut checkpoint_copy_bits = 0u64;

        if self.arm == PolicyArm::C3VerificationRecovery
            && checkpoint.is_none()
            && matches!(next_p3_event(&self.task, self.solver)?, Some(ForkEvent::ChoicePoint))
        {
            let bits = P3_CHECKPOINT_BYTES
                .checked_mul(8)
                .ok_or(ReferenceExecutionError::ArithmeticOverflow)?;
            meter.charge_compute(ComputeComponent::Checkpoint, P3_CHECKPOINT_BYTES)?;
            traffic.add_store(P3_CHECKPOINT_BYTES)?;
            checkpoint = Some(Checkpoint { state: self.solver });
            checkpoint_copy_bits = bits;
        }

        let replaying = self
            .replay_until
            .is_some_and(|limit| self.solver.cursor() < limit);
        let (next_solver, summary, transition_ops) = solver_transition(self.solver, &self.task)?;
        meter.charge_compute(
            if replaying {
                ComputeComponent::Replay
            } else {
                ComputeComponent::Solver
            },
            transition_ops,
        )?;

        let next_action_count = self
            .action_count
            .checked_add(1)
            .ok_or(ReferenceExecutionError::ArithmeticOverflow)?;
        let next_high_water = self.high_water_step.max(next_solver.cursor());
        let next_replay_until = self
            .replay_until
            .filter(|limit| next_solver.cursor() < *limit);
        let temporary = transaction_temporary_bits(next_solver, checkpoint_copy_bits)?;
        set_memory_state(
            &mut meter,
            next_solver,
            self.policy_memory_bits,
            checkpoint,
            temporary,
        )?;

        self.meter = meter;
        self.checkpoint_traffic = traffic;
        self.checkpoint = checkpoint;
        self.solver = next_solver;
        self.last_summary = summary;
        self.action_count = next_action_count;
        self.high_water_step = next_high_water;
        self.replay_until = next_replay_until;
        self.last_verifier_signal = None;
        self.last_rejected_branch = None;
        solver_candidate(self.solver)
    }

    /// Invoke the frozen independent verifier. It never reads evaluator target
    /// metadata. P1 and P3 inspect observed evidence only; P2 may check the full
    /// public constraint system, which is the explicit costed verification path.
    pub fn verify(&mut self) -> Result<VerifierSignal, ReferenceExecutionError> {
        self.ensure_running()?;
        validate_action(self.arm, InferenceAction::Verify)?;
        let candidate = solver_candidate(self.solver)?;
        let (signal, operations) = independent_verify(&self.task, self.solver, candidate)?;

        let mut meter = self.meter;
        meter.charge_compute(ComputeComponent::Verifier, operations)?;
        let temporary = transaction_temporary_bits(self.solver, ACTION_SCRATCH_BITS)?;
        set_memory_state(
            &mut meter,
            self.solver,
            self.policy_memory_bits,
            self.checkpoint,
            temporary,
        )?;
        let next_action_count = self
            .action_count
            .checked_add(1)
            .ok_or(ReferenceExecutionError::ArithmeticOverflow)?;

        self.meter = meter;
        self.action_count = next_action_count;
        self.last_verifier_signal = Some(signal);
        self.last_rejected_branch = match (signal, candidate) {
            (VerifierSignal::Violated, SolverCandidate::P3(branch)) => Some(branch),
            _ => None,
        };
        Ok(signal)
    }

    /// Restore the single eligible P3 checkpoint after a verifier-confirmed
    /// violation. The rejected live branch is retained only as local recovery
    /// state, so replay cannot recommit to the same refuted binary branch.
    pub fn backtrack(&mut self) -> Result<SolverCandidate, ReferenceExecutionError> {
        self.ensure_running()?;
        validate_action(self.arm, InferenceAction::Backtrack)?;
        if self.last_verifier_signal != Some(VerifierSignal::Violated) {
            return Err(ReferenceExecutionError::BacktrackRequiresViolation);
        }
        let checkpoint = self
            .checkpoint
            .ok_or(ReferenceExecutionError::CheckpointUnavailable)?;
        let rejected = self
            .last_rejected_branch
            .ok_or(ReferenceExecutionError::BacktrackUnsupportedForTask)?;

        let mut restored = checkpoint.state.with_forbidden_branch(rejected)?;
        if restored.cursor() >= self.solver.cursor() {
            return Err(ReferenceExecutionError::CheckpointNotEarlier);
        }
        let previous_cursor = self.solver.cursor();
        let mut meter = self.meter;
        let mut traffic = self.checkpoint_traffic;
        meter.charge_compute(ComputeComponent::Checkpoint, P3_CHECKPOINT_BYTES)?;
        traffic.add_restore(P3_CHECKPOINT_BYTES)?;
        let copy_bits = P3_CHECKPOINT_BYTES
            .checked_mul(8)
            .ok_or(ReferenceExecutionError::ArithmeticOverflow)?;
        let temporary = transaction_temporary_bits(restored, copy_bits)?;
        set_memory_state(
            &mut meter,
            restored,
            self.policy_memory_bits,
            self.checkpoint,
            temporary,
        )?;
        let next_action_count = self
            .action_count
            .checked_add(1)
            .ok_or(ReferenceExecutionError::ArithmeticOverflow)?;
        let remaining = task_len_u64(&self.task)?
            .checked_sub(restored.cursor())
            .ok_or(ReferenceExecutionError::AccountingInvariant)?;
        let depth = previous_cursor
            .checked_sub(restored.cursor())
            .ok_or(ReferenceExecutionError::AccountingInvariant)?;
        let score_margin = solver_score_margin(restored)?;

        self.meter = meter;
        self.checkpoint_traffic = traffic;
        self.solver = restored;
        self.action_count = next_action_count;
        self.replay_until = Some(previous_cursor);
        self.last_verifier_signal = None;
        self.last_rejected_branch = None;
        self.last_summary = SolverSummary {
            state_delta: depth as f64,
            residual: remaining as f64,
            score_margin,
        };
        restored = self.solver;
        solver_candidate(restored)
    }

    /// STOP emits the live solver candidate and closes the trajectory. No
    /// evaluator target is needed here.
    pub fn stop(&mut self) -> Result<StoppedCandidate, ReferenceExecutionError> {
        self.ensure_running()?;
        validate_action(self.arm, InferenceAction::Stop)?;
        let candidate = solver_candidate(self.solver)?;
        let next_action_count = self
            .action_count
            .checked_add(1)
            .ok_or(ReferenceExecutionError::ArithmeticOverflow)?;
        self.action_count = next_action_count;
        self.stopped = true;
        Ok(StoppedCandidate {
            candidate,
            step_index: self.solver.cursor(),
            accounting: self.accounting(),
        })
    }

    fn ensure_running(&self) -> Result<(), ReferenceExecutionError> {
        if self.stopped {
            Err(ReferenceExecutionError::AlreadyStopped)
        } else {
            Ok(())
        }
    }
}

/// Score only a stopped candidate against evaluator-owned metadata.
pub fn evaluate_stopped(
    stopped: StoppedCandidate,
    evaluator: EvaluatorRecord,
) -> Result<bool, ReferenceExecutionError> {
    if stopped.candidate.family() != evaluator.family() {
        return Err(ReferenceExecutionError::EvaluatorFamilyMismatch);
    }
    match (stopped.candidate, evaluator.target()) {
        (SolverCandidate::P1(candidate), EvaluatorTarget::P1(target)) => Ok(candidate == target),
        (
            SolverCandidate::P2 {
                bits: candidate,
                width: candidate_width,
            },
            EvaluatorTarget::P2 {
                bits: target,
                width: target_width,
            },
        ) if candidate_width == target_width => Ok(candidate == target),
        (SolverCandidate::P3(candidate), EvaluatorTarget::P3(target)) => Ok(candidate == target),
        _ => Err(ReferenceExecutionError::EvaluatorFamilyMismatch),
    }
}

fn initial_solver(task: &PolicyTask) -> Result<SolverState, ReferenceExecutionError> {
    match task {
        PolicyTask::P1(_) => Ok(SolverState::P1 { cursor: 0, sum: 0 }),
        PolicyTask::P2(task) => Ok(SolverState::P2 {
            cursor: 0,
            bits: 0,
            width: task.width(),
        }),
        PolicyTask::P3(_) => Ok(SolverState::P3 {
            cursor: 0,
            left_score: 0,
            right_score: 0,
            eliminated_mask: 0,
            committed: None,
            forbidden: None,
        }),
    }
}

fn solver_transition(
    state: SolverState,
    task: &PolicyTask,
) -> Result<(SolverState, SolverSummary, u64), ReferenceExecutionError> {
    match (state, task) {
        (SolverState::P1 { cursor, sum }, PolicyTask::P1(task)) => {
            let index = usize::try_from(cursor).map_err(|_| ReferenceExecutionError::TaskTooLarge)?;
            let value = *task
                .evidence()
                .get(index)
                .ok_or(ReferenceExecutionError::SolverExhausted)?;
            if !matches!(value, -1 | 1) {
                return Err(ReferenceExecutionError::TaskContractViolation);
            }
            let next_sum = sum
                .checked_add(i64::from(value))
                .ok_or(ReferenceExecutionError::ArithmeticOverflow)?;
            let next_cursor = cursor
                .checked_add(1)
                .ok_or(ReferenceExecutionError::ArithmeticOverflow)?;
            let remaining = task_len_u64(task_as_policy(task))?
                .checked_sub(next_cursor)
                .ok_or(ReferenceExecutionError::AccountingInvariant)?;
            Ok((
                SolverState::P1 {
                    cursor: next_cursor,
                    sum: next_sum,
                },
                SolverSummary {
                    state_delta: f64::from(value.unsigned_abs()),
                    residual: remaining as f64,
                    score_margin: next_sum as f64,
                },
                P1_SOLVER_OPS,
            ))
        }
        (
            SolverState::P2 {
                cursor,
                bits,
                width,
            },
            PolicyTask::P2(task),
        ) => {
            if width != task.width() {
                return Err(ReferenceExecutionError::TaskContractViolation);
            }
            let index = usize::try_from(cursor).map_err(|_| ReferenceExecutionError::TaskTooLarge)?;
            let constraint = *task
                .constraints()
                .get(index)
                .ok_or(ReferenceExecutionError::SolverExhausted)?;
            let bit = u8::try_from(cursor).map_err(|_| ReferenceExecutionError::TaskTooLarge)?;
            if bit >= width {
                return Err(ReferenceExecutionError::SolverExhausted);
            }
            let pivot = 1u64 << bit;
            if constraint.mask() & pivot == 0 {
                return Err(ReferenceExecutionError::TaskContractViolation);
            }
            let lower_mask = if bit == 0 { 0 } else { pivot - 1 };
            let lower_parity = (bits & constraint.mask() & lower_mask).count_ones() % 2 == 1;
            let pivot_value = lower_parity ^ constraint.parity();
            let next_bits = if pivot_value {
                bits | pivot
            } else {
                bits & !pivot
            };
            let next_cursor = cursor
                .checked_add(1)
                .ok_or(ReferenceExecutionError::ArithmeticOverflow)?;
            let remaining = u64::from(width)
                .checked_sub(next_cursor)
                .ok_or(ReferenceExecutionError::AccountingInvariant)?;
            let operations = checked_linear_ops(
                P2_SOLVER_BASE_OPS,
                P2_SOLVER_BIT_OPS,
                u64::from(bit) + 1,
            )?;
            Ok((
                SolverState::P2 {
                    cursor: next_cursor,
                    bits: next_bits,
                    width,
                },
                SolverSummary {
                    state_delta: if next_bits == bits { 0.0 } else { 1.0 },
                    residual: remaining as f64,
                    score_margin: next_cursor as f64,
                },
                operations,
            ))
        }
        (
            SolverState::P3 {
                cursor,
                mut left_score,
                mut right_score,
                mut eliminated_mask,
                mut committed,
                forbidden,
            },
            PolicyTask::P3(task),
        ) => {
            let index = usize::try_from(cursor).map_err(|_| ReferenceExecutionError::TaskTooLarge)?;
            let event = *task
                .events()
                .get(index)
                .ok_or(ReferenceExecutionError::SolverExhausted)?;
            let (state_delta, operations) = match event {
                ForkEvent::ChoicePoint => (0.0, P3_CHOICE_OPS),
                ForkEvent::Evidence {
                    left_delta,
                    right_delta,
                } => {
                    if left_delta < 0 || right_delta < 0 {
                        return Err(ReferenceExecutionError::TaskContractViolation);
                    }
                    left_score = left_score
                        .checked_add(i64::from(left_delta))
                        .ok_or(ReferenceExecutionError::ArithmeticOverflow)?;
                    right_score = right_score
                        .checked_add(i64::from(right_delta))
                        .ok_or(ReferenceExecutionError::ArithmeticOverflow)?;
                    if committed.is_none() {
                        committed = Some(preferred_branch(
                            left_score,
                            right_score,
                            eliminated_mask,
                            forbidden,
                        )?);
                    }
                    (
                        i64::from(left_delta)
                            .checked_add(i64::from(right_delta))
                            .ok_or(ReferenceExecutionError::ArithmeticOverflow)? as f64,
                        P3_EVIDENCE_OPS,
                    )
                }
                ForkEvent::EliminateBranch { branch } => {
                    eliminated_mask |= branch_bit(branch);
                    if eliminated_mask == 0b11 {
                        return Err(ReferenceExecutionError::TaskContractViolation);
                    }
                    (1.0, P3_ELIMINATE_OPS)
                }
            };
            let next_cursor = cursor
                .checked_add(1)
                .ok_or(ReferenceExecutionError::ArithmeticOverflow)?;
            let remaining = u64::try_from(task.events().len())
                .map_err(|_| ReferenceExecutionError::TaskTooLarge)?
                .checked_sub(next_cursor)
                .ok_or(ReferenceExecutionError::AccountingInvariant)?;
            let next = SolverState::P3 {
                cursor: next_cursor,
                left_score,
                right_score,
                eliminated_mask,
                committed,
                forbidden,
            };
            Ok((
                next,
                SolverSummary {
                    state_delta,
                    residual: remaining as f64,
                    score_margin: (left_score - right_score) as f64,
                },
                operations,
            ))
        }
        _ => Err(ReferenceExecutionError::TaskStateMismatch),
    }
}

fn independent_verify(
    task: &PolicyTask,
    state: SolverState,
    candidate: SolverCandidate,
) -> Result<(VerifierSignal, u64), ReferenceExecutionError> {
    match (task, state, candidate) {
        (PolicyTask::P1(task), SolverState::P1 { cursor, .. }, SolverCandidate::P1(candidate)) => {
            let observed = usize::try_from(cursor).map_err(|_| ReferenceExecutionError::TaskTooLarge)?;
            let event_count = task.evidence().len();
            let operations = checked_linear_ops(
                P1_VERIFIER_BASE_OPS,
                P1_VERIFIER_EVENT_OPS,
                u64::try_from(observed).map_err(|_| ReferenceExecutionError::TaskTooLarge)?,
            )?;
            if observed < event_count {
                return Ok((VerifierSignal::Indeterminate, operations));
            }
            let mut sum = 0i64;
            for value in task.evidence() {
                if !matches!(*value, -1 | 1) {
                    return Err(ReferenceExecutionError::TaskContractViolation);
                }
                sum = sum
                    .checked_add(i64::from(*value))
                    .ok_or(ReferenceExecutionError::ArithmeticOverflow)?;
            }
            if sum == 0 {
                return Err(ReferenceExecutionError::TaskContractViolation);
            }
            let expected = if sum > 0 {
                BinaryTarget::Positive
            } else {
                BinaryTarget::Negative
            };
            Ok((
                if candidate == expected {
                    VerifierSignal::Satisfied
                } else {
                    VerifierSignal::Violated
                },
                operations,
            ))
        }
        (
            PolicyTask::P2(task),
            SolverState::P2 { width, .. },
            SolverCandidate::P2 {
                bits,
                width: candidate_width,
            },
        ) => {
            if width != task.width() || candidate_width != width {
                return Err(ReferenceExecutionError::TaskStateMismatch);
            }
            let mut all_satisfied = true;
            for constraint in task.constraints() {
                let parity = (bits & constraint.mask()).count_ones() % 2 == 1;
                all_satisfied &= parity == constraint.parity();
            }
            let row_ops = checked_linear_ops(
                P2_VERIFIER_ROW_BASE_OPS,
                P2_VERIFIER_BIT_OPS,
                u64::from(width),
            )?;
            let operations = row_ops
                .checked_mul(
                    u64::try_from(task.constraints().len())
                        .map_err(|_| ReferenceExecutionError::TaskTooLarge)?,
                )
                .ok_or(ReferenceExecutionError::ArithmeticOverflow)?;
            Ok((
                if all_satisfied {
                    VerifierSignal::Satisfied
                } else {
                    VerifierSignal::Violated
                },
                operations,
            ))
        }
        (
            PolicyTask::P3(task),
            SolverState::P3 { cursor, .. },
            SolverCandidate::P3(candidate),
        ) => {
            let observed = usize::try_from(cursor).map_err(|_| ReferenceExecutionError::TaskTooLarge)?;
            let mut saw_elimination = false;
            let mut candidate_eliminated = false;
            for event in task.events().iter().take(observed) {
                if let ForkEvent::EliminateBranch { branch } = event {
                    saw_elimination = true;
                    candidate_eliminated |= *branch == candidate;
                }
            }
            let operations = checked_linear_ops(
                P3_VERIFIER_BASE_OPS,
                P3_VERIFIER_EVENT_OPS,
                u64::try_from(observed).map_err(|_| ReferenceExecutionError::TaskTooLarge)?,
            )?;
            Ok((
                if candidate_eliminated {
                    VerifierSignal::Violated
                } else if saw_elimination {
                    VerifierSignal::Satisfied
                } else {
                    VerifierSignal::Indeterminate
                },
                operations,
            ))
        }
        _ => Err(ReferenceExecutionError::TaskStateMismatch),
    }
}

fn solver_candidate(state: SolverState) -> Result<SolverCandidate, ReferenceExecutionError> {
    match state {
        SolverState::P1 { sum, .. } => Ok(SolverCandidate::P1(if sum < 0 {
            BinaryTarget::Negative
        } else {
            BinaryTarget::Positive
        })),
        SolverState::P2 { bits, width, .. } => Ok(SolverCandidate::P2 { bits, width }),
        SolverState::P3 {
            left_score,
            right_score,
            eliminated_mask,
            committed,
            forbidden,
            ..
        } => Ok(SolverCandidate::P3(match committed {
            Some(branch) => branch,
            None => preferred_branch(left_score, right_score, eliminated_mask, forbidden)?,
        })),
    }
}

fn solver_score_margin(state: SolverState) -> Result<f64, ReferenceExecutionError> {
    match state {
        SolverState::P1 { sum, .. } => Ok(sum as f64),
        SolverState::P2 { cursor, .. } => Ok(cursor as f64),
        SolverState::P3 {
            left_score,
            right_score,
            ..
        } => left_score
            .checked_sub(right_score)
            .map(|value| value as f64)
            .ok_or(ReferenceExecutionError::ArithmeticOverflow),
    }
}

fn preferred_branch(
    left_score: i64,
    right_score: i64,
    eliminated_mask: u8,
    forbidden: Option<ForkBranch>,
) -> Result<ForkBranch, ReferenceExecutionError> {
    if eliminated_mask & 0b11 == 0b11 {
        return Err(ReferenceExecutionError::TaskContractViolation);
    }
    if forbidden == Some(ForkBranch::Left) || eliminated_mask & branch_bit(ForkBranch::Left) != 0 {
        return Ok(ForkBranch::Right);
    }
    if forbidden == Some(ForkBranch::Right) || eliminated_mask & branch_bit(ForkBranch::Right) != 0 {
        return Ok(ForkBranch::Left);
    }
    Ok(if left_score >= right_score {
        ForkBranch::Left
    } else {
        ForkBranch::Right
    })
}

const fn branch_bit(branch: ForkBranch) -> u8 {
    match branch {
        ForkBranch::Left => 0b01,
        ForkBranch::Right => 0b10,
    }
}

fn next_p3_event(
    task: &PolicyTask,
    state: SolverState,
) -> Result<Option<ForkEvent>, ReferenceExecutionError> {
    match (task, state) {
        (PolicyTask::P3(task), SolverState::P3 { cursor, .. }) => {
            let index = usize::try_from(cursor).map_err(|_| ReferenceExecutionError::TaskTooLarge)?;
            Ok(task.events().get(index).copied())
        }
        (_, SolverState::P3 { .. }) | (PolicyTask::P3(_), _) => {
            Err(ReferenceExecutionError::TaskStateMismatch)
        }
        _ => Ok(None),
    }
}

fn task_len_u64(task: &PolicyTask) -> Result<u64, ReferenceExecutionError> {
    u64::try_from(task.event_count()).map_err(|_| ReferenceExecutionError::TaskTooLarge)
}

// Helper used only in the P1 transition arm where a concrete P1 task borrow is
// already available. Reconstructing a PolicyTask would allocate, so this tiny
// wrapper is deliberately avoided; callers should use the concrete length.
fn task_as_policy(_task: &crate::adaptive_task_generators::P1PolicyTask) -> &PolicyTask {
    unreachable!("internal helper must not be called")
}

fn set_memory_state(
    meter: &mut ResourceMeter,
    solver: SolverState,
    policy_memory_bits: u64,
    checkpoint: Option<Checkpoint>,
    temporary_bits: u64,
) -> Result<(), ReferenceExecutionError> {
    let persistent = solver
        .logical_bits()
        .checked_add(EXECUTION_METADATA_BITS)
        .ok_or(ReferenceExecutionError::ArithmeticOverflow)?;
    let checkpoint_bits = if checkpoint.is_some() {
        P3_CHECKPOINT_BYTES
            .checked_mul(8)
            .ok_or(ReferenceExecutionError::ArithmeticOverflow)?
    } else {
        0
    };
    meter.set_memory(
        persistent,
        policy_memory_bits,
        checkpoint_bits,
        temporary_bits,
    )?;
    Ok(())
}

fn transaction_temporary_bits(
    solver: SolverState,
    extra_bits: u64,
) -> Result<u64, ReferenceExecutionError> {
    [
        solver.logical_bits(),
        RESOURCE_METER_SHADOW_BITS,
        TRANSACTION_METADATA_SHADOW_BITS,
        ACTION_SCRATCH_BITS,
        extra_bits,
    ]
    .into_iter()
    .try_fold(0u64, |total, value| {
        total
            .checked_add(value)
            .ok_or(ReferenceExecutionError::ArithmeticOverflow)
    })
}

fn checked_linear_ops(
    base: u64,
    per_item: u64,
    count: u64,
) -> Result<u64, ReferenceExecutionError> {
    per_item
        .checked_mul(count)
        .and_then(|value| base.checked_add(value))
        .ok_or(ReferenceExecutionError::ArithmeticOverflow)
}

/// Typed fail-closed execution failure.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ReferenceExecutionError {
    AdaptiveInference(AdaptiveInferenceError),
    AlreadyStopped,
    SolverExhausted,
    TaskTooLarge,
    TaskStateMismatch,
    TaskContractViolation,
    ArithmeticOverflow,
    CheckpointTrafficOverflow,
    AccountingInvariant,
    CheckpointUnavailable,
    CheckpointNotEarlier,
    BacktrackRequiresViolation,
    BacktrackUnsupportedForTask,
    EvaluatorFamilyMismatch,
}

impl From<AdaptiveInferenceError> for ReferenceExecutionError {
    fn from(value: AdaptiveInferenceError) -> Self {
        Self::AdaptiveInference(value)
    }
}

impl fmt::Display for ReferenceExecutionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for ReferenceExecutionError {}
