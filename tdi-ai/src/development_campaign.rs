//! Bounded in-memory development campaigns. Every attempted seed is retained,
//! including generator and technical failures; no scientific aggregate verdict.
use super::{
    adaptive_evaluator::ReferencePolicy,
    adaptive_inference::ResourceEnvelope,
    adaptive_rejections::{ReferenceEvaluationOutcome, evaluate_generated_task_recorded},
    adaptive_task_generators::GeneratedTask,
};
use crate::provenance::{DevelopmentDomain, ExperimentManifest};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq)]
pub struct PolicySpec {
    pub id: String,
    pub policy: ReferencePolicy,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CampaignPlan {
    manifest: ExperimentManifest,
    domain: DevelopmentDomain,
    indices: Vec<u64>,
    policies: Vec<PolicySpec>,
    envelope: ResourceEnvelope,
    decision_limit: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CampaignError {
    EmptyPlan,
    InvalidIndex,
    DuplicateIndex,
    InvalidPolicyId,
    TrialLimit,
    ZeroDecisionLimit,
    Capacity,
    PlanMismatch,
}
impl CampaignPlan {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        manifest: ExperimentManifest,
        domain: DevelopmentDomain,
        indices: Vec<u64>,
        policies: Vec<PolicySpec>,
        envelope: ResourceEnvelope,
        decision_limit: u64,
        max_trials: usize,
    ) -> Result<Self, CampaignError> {
        if indices.is_empty() || policies.is_empty() {
            return Err(CampaignError::EmptyPlan);
        }
        if decision_limit == 0 {
            return Err(CampaignError::ZeroDecisionLimit);
        }
        if indices
            .len()
            .checked_mul(policies.len())
            .is_none_or(|n| n > max_trials)
        {
            return Err(CampaignError::TrialLimit);
        }
        let mut seen = BTreeSet::new();
        for &index in &indices {
            if domain.seed(index).is_none() {
                return Err(CampaignError::InvalidIndex);
            }
            if !seen.insert(index) {
                return Err(CampaignError::DuplicateIndex);
            }
        }
        let mut ids = BTreeSet::new();
        for policy in &policies {
            if policy.id.trim().is_empty() || !ids.insert(policy.id.clone()) {
                return Err(CampaignError::InvalidPolicyId);
            }
        }
        Ok(Self {
            manifest,
            domain,
            indices,
            policies,
            envelope,
            decision_limit,
        })
    }
    /// Binds the actual policy parameters, execution envelope, ordered indices
    /// and domain in addition to caller-supplied source/generator identities.
    #[must_use]
    pub fn canonical_record(&self) -> String {
        let mut out = self.manifest.canonical_record();
        out.push_str(&format!(
            "\ncampaign/v1;domain={:?};compute={};memory={};decisions={};indices={:?}\n",
            self.domain,
            self.envelope.max_compute_ops(),
            self.envelope.max_memory_bits(),
            self.decision_limit,
            self.indices
        ));
        for p in &self.policies {
            let config = p.policy.canonical_record();
            out.push_str(&format!(
                "{}:{}{}:{}",
                p.id.len(),
                p.id,
                config.len(),
                config
            ));
        }
        out
    }
    pub fn start<E>(&self) -> Result<CampaignReport<E>, CampaignError> {
        let mut trials = Vec::new();
        trials
            .try_reserve_exact(self.indices.len())
            .map_err(|_| CampaignError::Capacity)?;
        Ok(CampaignReport {
            plan: self.clone(),
            trials,
        })
    }
    /// Resume an owned, unforgeable prefix in place. Cancellation is checked
    /// between complete paired trials. Allocation/plan errors preserve `report`.
    /// Generator identity is a caller contract; arbitrary closures are not attested.
    pub fn run<E>(
        &self,
        report: &mut CampaignReport<E>,
        mut generate: impl FnMut(u64) -> Result<GeneratedTask, E>,
        mut cancelled: impl FnMut() -> bool,
    ) -> Result<(), CampaignError> {
        if report.plan != *self {
            return Err(CampaignError::PlanMismatch);
        }
        for &index in &self.indices[report.trials.len()..] {
            if cancelled() {
                break;
            }
            let seed = self.domain.seed(index).ok_or(CampaignError::InvalidIndex)?;
            // Reserve before invoking the generator so an allocation failure
            // neither consumes a trial nor loses previously recorded evidence.
            let mut outcomes = Vec::new();
            outcomes
                .try_reserve_exact(self.policies.len())
                .map_err(|_| CampaignError::Capacity)?;
            let outcome = match generate(seed) {
                Err(error) => TrialOutcome::GeneratorRejected(error),
                Ok(task) if task.evaluator().seed() != seed => TrialOutcome::SeedMismatch {
                    actual: task.evaluator().seed(),
                },
                Ok(task) => {
                    for policy in &self.policies {
                        outcomes.push(evaluate_generated_task_recorded(
                            task.clone(),
                            policy.policy,
                            self.envelope,
                            self.decision_limit,
                        ));
                    }
                    TrialOutcome::Evaluated(outcomes)
                }
            };
            report.trials.push(TrialRecord {
                index,
                seed,
                outcome,
            });
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum TrialOutcome<E> {
    Evaluated(Vec<ReferenceEvaluationOutcome>),
    GeneratorRejected(E),
    SeedMismatch { actual: u64 },
}
#[derive(Clone, Debug, PartialEq)]
pub struct TrialRecord<E> {
    pub index: u64,
    pub seed: u64,
    pub outcome: TrialOutcome<E>,
}
/// Read-only results prevent callers from altering the prefix accepted by resume.
/// Persistence across processes is deliberately not implied by this in-memory API.
#[derive(Clone, Debug, PartialEq)]
pub struct CampaignReport<E> {
    plan: CampaignPlan,
    trials: Vec<TrialRecord<E>>,
}
impl<E> CampaignReport<E> {
    #[must_use]
    pub fn trials(&self) -> &[TrialRecord<E>] {
        &self.trials
    }
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.trials.len() == self.plan.indices.len()
    }
    #[must_use]
    pub fn plan(&self) -> &CampaignPlan {
        &self.plan
    }
}
