//! Bounded non-final TDI-9.1 reference evaluator integration.
//!
//! This layer composes already-qualified task generation, policy semantics and
//! deterministic execution. It deliberately selects no primary schedule,
//! adaptive threshold, difficulty, resource envelope, population, seed domain or
//! final-entropy value. Evaluator-only metadata is consumed only after the live
//! trajectory has stopped.

use core::fmt;

use crate::adaptive_execution::{
    CheckpointTraffic, ExecutionAccounting, ReferenceExecution, ReferenceExecutionError,
    SolverCandidate, StoppedCandidate, evaluate_stopped,
};
use crate::adaptive_inference::{InferenceAction, PolicyArm, ResourceEnvelope, ResourceUsage};
use crate::adaptive_policies::{
    C0FixedPolicy, C1Plan, C1StaticPolicy, C2AdaptivePolicy, C3RecoveryPolicy, PolicyDecision,
    ReferencePolicyError,
};
use crate::adaptive_task_generators::{AdaptiveTaskFamily, DifficultyStratum, GeneratedTask};

/// One already-constructed bounded reference policy.
///
/// Constructors and concrete values remain external so this integration layer
/// cannot silently freeze schedules or adaptive thresholds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ReferencePolicy {
    C0(C0FixedPolicy),
    C1(C1StaticPolicy),
    C2(C2AdaptivePolicy),
    C3(C3RecoveryPolicy),
}

impl ReferencePolicy {
    #[must_use]
    pub const fn arm(self) -> PolicyArm {
        match self {
            Self::C0(_) => PolicyArm::C0FixedCompute,
            Self::C1(_) => PolicyArm::C1StaticPreallocation,
            Self::C2(_) => PolicyArm::C2AdaptiveStopping,
            Self::C3(_) => PolicyArm::C3VerificationRecovery,
        }
    }
}

impl From<C0FixedPolicy> for ReferencePolicy {
    fn from(value: C0FixedPolicy) -> Self {
        Self::C0(value)
    }
}

impl From<C1StaticPolicy> for ReferencePolicy {
    fn from(value: C1StaticPolicy) -> Self {
        Self::C1(value)
    }
}

impl From<C2AdaptivePolicy> for ReferencePolicy {
    fn from(value: C2AdaptivePolicy) -> Self {
        Self::C2(value)
    }
}

impl From<C3RecoveryPolicy> for ReferencePolicy {
    fn from(value: C3RecoveryPolicy) -> Self {
        Self::C3(value)
    }
}

/// Immutable evaluator-side record emitted only after an explicit STOP.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReferenceEvaluationRecord {
    arm: PolicyArm,
    family: AdaptiveTaskFamily,
    stratum: DifficultyStratum,
    seed: u64,
    success: bool,
    candidate: SolverCandidate,
    stop_step: u64,
    runtime_decisions: u64,
    accounting: ExecutionAccounting,
}

impl ReferenceEvaluationRecord {
    #[must_use]
    pub const fn arm(self) -> PolicyArm {
        self.arm
    }

    #[must_use]
    pub const fn family(self) -> AdaptiveTaskFamily {
        self.family
    }

    #[must_use]
    pub const fn stratum(self) -> DifficultyStratum {
        self.stratum
    }

    #[must_use]
    pub const fn seed(self) -> u64 {
        self.seed
    }

    #[must_use]
    pub const fn success(self) -> bool {
        self.success
    }

    #[must_use]
    pub const fn candidate(self) -> SolverCandidate {
        self.candidate
    }

    #[must_use]
    pub const fn stop_step(self) -> u64 {
        self.stop_step
    }

    /// Number of runtime policy decisions, including the final STOP decision.
    /// C1 pre-inference planning is charged in resource accounting but is not a
    /// runtime decision.
    #[must_use]
    pub const fn runtime_decisions(self) -> u64 {
        self.runtime_decisions
    }

    #[must_use]
    pub const fn accounting(self) -> ExecutionAccounting {
        self.accounting
    }

    #[must_use]
    pub const fn usage(self) -> ResourceUsage {
        self.accounting.usage()
    }

    #[must_use]
    pub const fn checkpoint_traffic(self) -> CheckpointTraffic {
        self.accounting.checkpoint_traffic()
    }
}

/// Execute one complete non-final reference trajectory.
///
/// `runtime_decision_limit` is a caller-supplied technical safety bound. It is
/// not a task horizon or scientific stopping rule. Reaching it is a typed
/// technical rejection rather than an evaluated prediction.
pub fn evaluate_generated_task(
    generated: GeneratedTask,
    policy: ReferencePolicy,
    envelope: ResourceEnvelope,
    runtime_decision_limit: u64,
) -> Result<ReferenceEvaluationRecord, ReferenceEvaluatorError> {
    if runtime_decision_limit == 0 {
        return Err(ReferenceEvaluatorError::ZeroDecisionLimit);
    }

    let arm = policy.arm();
    let (policy_task, evaluator) = generated.into_parts();
    let family = policy_task.family();
    if evaluator.family() != family {
        return Err(ReferenceEvaluatorError::GeneratorFamilyMismatch);
    }

    let mut execution = ReferenceExecution::new(arm, policy_task, envelope)?;
    let c1_plan = prepare_c1_plan(policy, family, &mut execution)?;

    for decision_index in 0..runtime_decision_limit {
        let decision = choose_decision(policy, c1_plan, &execution)?;
        if decision.arm() != arm {
            return Err(ReferenceEvaluatorError::PolicyArmDrift {
                expected: arm,
                actual: decision.arm(),
            });
        }

        let charge = decision.charge();
        execution.charge_policy_decision(charge.operations(), charge.memory_bits())?;
        if let Some(stopped) = apply_action(&mut execution, decision)? {
            let success = evaluate_stopped(stopped, evaluator)?;
            let runtime_decisions = decision_index
                .checked_add(1)
                .ok_or(ReferenceEvaluatorError::DecisionCountOverflow)?;
            return Ok(ReferenceEvaluationRecord {
                arm,
                family,
                stratum: evaluator.stratum(),
                seed: evaluator.seed(),
                success,
                candidate: stopped.candidate(),
                stop_step: stopped.step_index(),
                runtime_decisions,
                accounting: stopped.accounting(),
            });
        }
    }

    Err(ReferenceEvaluatorError::DecisionLimitExceeded {
        limit: runtime_decision_limit,
    })
}

fn prepare_c1_plan(
    policy: ReferencePolicy,
    family: AdaptiveTaskFamily,
    execution: &mut ReferenceExecution,
) -> Result<Option<C1Plan>, ReferenceEvaluatorError> {
    let ReferencePolicy::C1(policy) = policy else {
        return Ok(None);
    };
    let charge = policy.planning_charge();
    execution.charge_policy_decision(charge.operations(), charge.memory_bits())?;
    Ok(Some(policy.plan(family)))
}

fn choose_decision(
    policy: ReferencePolicy,
    c1_plan: Option<C1Plan>,
    execution: &ReferenceExecution,
) -> Result<PolicyDecision, ReferenceEvaluatorError> {
    Ok(match policy {
        ReferencePolicy::C0(policy) => policy.decide(execution.step_index())?,
        ReferencePolicy::C1(_) => c1_plan
            .ok_or(ReferenceEvaluatorError::MissingC1Plan)?
            .decide(execution.step_index())?,
        ReferencePolicy::C2(policy) => policy.decide(execution.observation()?)?,
        ReferencePolicy::C3(policy) => policy.decide(execution.observation()?)?,
    })
}

fn apply_action(
    execution: &mut ReferenceExecution,
    decision: PolicyDecision,
) -> Result<Option<StoppedCandidate>, ReferenceEvaluatorError> {
    match decision.action() {
        InferenceAction::Continue => {
            execution.continue_step()?;
            Ok(None)
        }
        InferenceAction::Verify => {
            execution.verify()?;
            Ok(None)
        }
        InferenceAction::Backtrack => {
            execution.backtrack()?;
            Ok(None)
        }
        InferenceAction::Stop => Ok(Some(execution.stop()?)),
    }
}

/// Typed fail-closed integration error.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ReferenceEvaluatorError {
    Execution(ReferenceExecutionError),
    Policy(ReferencePolicyError),
    ZeroDecisionLimit,
    DecisionLimitExceeded {
        limit: u64,
    },
    DecisionCountOverflow,
    MissingC1Plan,
    GeneratorFamilyMismatch,
    PolicyArmDrift {
        expected: PolicyArm,
        actual: PolicyArm,
    },
}

impl From<ReferenceExecutionError> for ReferenceEvaluatorError {
    fn from(value: ReferenceExecutionError) -> Self {
        Self::Execution(value)
    }
}

impl From<ReferencePolicyError> for ReferenceEvaluatorError {
    fn from(value: ReferencePolicyError) -> Self {
        Self::Policy(value)
    }
}

impl fmt::Display for ReferenceEvaluatorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for ReferenceEvaluatorError {}
