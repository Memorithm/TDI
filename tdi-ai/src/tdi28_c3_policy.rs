//! TDI-28 C3: prospective cost-guided Boolean execution on bounded events.
//!
//! Development-only reference semantics. Proxy coefficients are caller
//! hypotheses, NOT calibrated joules. A work budget is per admitted event,
//! not a wall-clock, power, memory bandwidth, or global scheduler limit.
//! The planner sees only gate/revisions, cache metadata, current tick,
//! deadline, and declared costs. It never sees input values or future events.
//! Explicit deadlines require independent wakeups and a fail-closed ledger.

use super::tdi28_energy_algebra::{
    AccountingError, CachedOutput, Decision, Gate, InputRevisions, Step, Work,
};

/// The current event's output obligation. An optional event may be deferred
/// only *strictly before* its already declared absolute logical deadline.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Obligation {
    RequiredNow,
    Optional { deadline_tick: u64 },
}

/// Versioned, synthetic cost/instantaneous budget contract.
///
/// The model revision identifies a caller-frozen synthetic coefficient set;
/// a nonzero revision is not proof that the weights are calibrated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProxyBudget {
    pub model_revision: u64,
    pub weights: Work,
    pub per_event_ceiling: u128,
}

/// C3 rejection reasons. In particular, required work is never silently
/// dropped when its declared upper bound exceeds a budget.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum C3Error {
    UnknownCostModel,
    UnpricedCompute,
    AlreadyPastDeadline { now: u64, deadline: u64 },
    BudgetInsufficient { upper_bound: u128, ceiling: u128 },
    CostOverflow,
}

impl From<AccountingError> for C3Error {
    fn from(_: AccountingError) -> Self {
        Self::CostOverflow
    }
}

/// A sealed, cost-auditable plan. Fields remain private so callers cannot
/// bypass the decision gate by fabricating a plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct C3Plan {
    gate: Gate,
    revisions: InputRevisions,
    cache: Option<CachedOutput>,
    decision: Decision,
    validated_cache: bool,
    upper_bound: u128,
    budget: ProxyBudget,
}

impl C3Plan {
    #[must_use]
    pub const fn decision(self) -> Decision {
        self.decision
    }

    /// Upper bound under the declared proxy weights, including one possible
    /// output transition for an evaluation with previous cached state.
    #[must_use]
    pub const fn proxy_upper_bound(self) -> u128 {
        self.upper_bound
    }

    #[must_use]
    pub const fn cost_model_revision(self) -> u64 {
        self.budget.model_revision
    }
}

/// Executed event result with traceable proxy cost, not energy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct C3Execution {
    pub step: Step,
    pub proxy_cost: u128,
    pub proxy_upper_bound: u128,
    pub model_revision: u64,
}

fn bookkeeping(has_cache: bool, validation: bool) -> Work {
    Work {
        revision_reads: 2,
        cache_probes: u64::from(has_cache),
        cache_validation_comparisons: u64::from(validation),
        event_decisions: 1,
        ..Work::default()
    }
}

/// Select a C3 action before loading any input value.
///
/// The planner first compares the *prospective* cost of always evaluating
/// with checking and reusing a present cache. It only validates a cache if
/// that path is strictly cheaper on the declared model. Stale checks are
/// still charged to an evaluation. If the chosen action exceeds the per-event
/// ceiling, only not-yet-due optional work may be deferred. No output is
/// claimed for deferred work; the caller must register and later discharge
/// the corresponding obligation through an explicit deadline ledger.
///
/// A cache revision must truthfully reflect every input dependency. The
/// planner cannot validate this assumption from revision stamps alone.
pub fn plan_c3(
    gate: Gate,
    revisions: InputRevisions,
    cache: Option<CachedOutput>,
    now: u64,
    obligation: Obligation,
    budget: ProxyBudget,
) -> Result<C3Plan, C3Error> {
    if budget.model_revision == 0 {
        return Err(C3Error::UnknownCostModel);
    }
    if budget.weights.gate_evaluations == 0 && budget.weights.input_value_reads == 0 {
        return Err(C3Error::UnpricedCompute);
    }
    if let Obligation::Optional { deadline_tick } = obligation {
        if deadline_tick < now {
            return Err(C3Error::AlreadyPastDeadline {
                now,
                deadline: deadline_tick,
            });
        }
    }

    let has_cache = cache.is_some();
    let mut eval_work = bookkeeping(has_cache, false);
    eval_work.gate_evaluations = 1;
    eval_work.input_value_reads = 2;
    // Conservative: output might change relative to any prior cached value.
    eval_work.output_transitions = u64::from(has_cache);
    let eval_upper = eval_work.proxy_score(budget.weights)?;

    let mut reuse_work = bookkeeping(has_cache, true);
    // Only consider checking a cache when its *prospective* total is cheaper.
    let inspect = has_cache && reuse_work.proxy_score(budget.weights)? < eval_upper;
    let fresh = inspect
        && cache.is_some_and(|saved| saved.gate == gate && saved.revisions == revisions);

    let selected = if fresh {
        reuse_work
    } else {
        if inspect {
            eval_work.cache_validation_comparisons = 1;
        }
        eval_work
    };
    let mut decision = if fresh {
        Decision::Reuse
    } else {
        Decision::Evaluate
    };
    let mut upper_bound = selected.proxy_score(budget.weights)?;

    if upper_bound > budget.per_event_ceiling {
        if matches!(
            obligation,
            Obligation::Optional { deadline_tick } if deadline_tick > now
        ) {
            // Inspected-cache overhead is not rolled back by a decision
            // to defer. A later wakeup/decision also has its own cost.
            reuse_work = bookkeeping(has_cache, inspect);
            reuse_work.deferred_events = 1;
            upper_bound = reuse_work.proxy_score(budget.weights)?;
            decision = Decision::Defer;
        }
        if upper_bound > budget.per_event_ceiling {
            return Err(C3Error::BudgetInsufficient {
                upper_bound,
                ceiling: budget.per_event_ceiling,
            });
        }
    }

    Ok(C3Plan {
        gate,
        revisions,
        cache,
        decision,
        validated_cache: inspect,
        upper_bound,
        budget,
    })
}

/// Execute the sealed decision, loading input values **only** for EVALUATE.
///
/// The evaluator must supply values stable for the revisions declared at
/// planning. This function does not retroactively validate values seen by
/// a previous cache entry. The billed reference-work vector includes
/// validation even on a stale-cache evaluation or subsequent deferral.
pub fn execute_c3<F>(plan: C3Plan, read_values: F) -> Result<C3Execution, C3Error>
where
    F: FnOnce() -> (bool, bool),
{
    let mut work = bookkeeping(plan.cache.is_some(), plan.validated_cache);
    let step = match plan.decision {
        Decision::Evaluate => {
            let (left, right) = read_values();
            let output = plan.gate.apply(left, right);
            work.gate_evaluations = 1;
            work.input_value_reads = 2;
            work.output_transitions =
                u64::from(plan.cache.is_some_and(|saved| saved.value != output));
            Step {
                decision: Decision::Evaluate,
                output: Some(output),
                cache: Some(CachedOutput {
                    revisions: plan.revisions,
                    gate: plan.gate,
                    value: output,
                }),
                work,
            }
        }
        Decision::Reuse => Step {
            decision: Decision::Reuse,
            output: plan.cache.map(|saved| saved.value),
            cache: plan.cache,
            work,
        },
        Decision::Defer => {
            work.deferred_events = 1;
            Step {
                decision: Decision::Defer,
                output: None,
                cache: plan.cache,
                work,
            }
        }
    };
    let proxy_cost = step.work.proxy_score(plan.budget.weights)?;
    // All weights are nonnegative, and conservative transitions are included,
    // so any larger actual cost signals an internal accounting defect.
    if proxy_cost > plan.upper_bound {
        return Err(C3Error::CostOverflow);
    }
    Ok(C3Execution {
        step,
        proxy_cost,
        proxy_upper_bound: plan.upper_bound,
        model_revision: plan.budget.model_revision,
    })
}

/// Live obligation identity. Identical revisions are insufficient if the
/// Boolean operator changed in the meantime.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PendingIdentity {
    pub gate: Gate,
    pub revisions: InputRevisions,
}

/// One outstanding optional result and the absolute due time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PendingDeadline {
    pub request_id: u64,
    pub identity: PendingIdentity,
    pub deadline_tick: u64,
}

/// Bounded deadline tracker; no hidden auto-cancel and no timer or remote
/// scheduler side effects. Each future wakeup must be explicitly delivered.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeadlineLedger {
    pending: Vec<PendingDeadline>,
    resolved: u64,
    cancelled: u64,
    last_tick: Option<u64>,
}

/// A missed due time is an error, not an energy saving.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeadlineError {
    NonIncreasingTime,
    DuplicateRequest(u64),
    MissingRequest(u64),
    CapacityExceeded,
    Missed { request_id: u64, due: u64, now: u64 },
}

impl DeadlineLedger {
    pub const MAX_PENDING: usize = 4096;

    /// Move logical time forward. A due event is admissible at its deadline,
    /// but cannot be skipped over without resolving the pending obligation.
    pub fn advance(&mut self, tick: u64) -> Result<(), DeadlineError> {
        if self.last_tick.is_some_and(|last| tick < last) {
            return Err(DeadlineError::NonIncreasingTime);
        }
        if let Some(missed) = self
            .pending
            .iter()
            .find(|pending| pending.deadline_tick < tick)
        {
            return Err(DeadlineError::Missed {
                request_id: missed.request_id,
                due: missed.deadline_tick,
                now: tick,
            });
        }
        self.last_tick = Some(tick);
        Ok(())
    }

    /// Record only a genuinely deferred request, at a predeclared deadline.
    pub fn defer(&mut self, pending: PendingDeadline) -> Result<(), DeadlineError> {
        if self.pending.iter().any(|old| old.request_id == pending.request_id) {
            return Err(DeadlineError::DuplicateRequest(pending.request_id));
        }
        if self.pending.len() >= Self::MAX_PENDING {
            return Err(DeadlineError::CapacityExceeded);
        }
        if let Some(now) = self.last_tick {
            if pending.deadline_tick <= now {
                return Err(DeadlineError::Missed {
                    request_id: pending.request_id,
                    due: pending.deadline_tick,
                    now,
                });
            }
        }
        self.pending.push(pending);
        Ok(())
    }

    /// Discharge a deferred result only when the produced output has the
    /// exact operator and revision identity originally requested.
    /// A different current output cannot silently count as resolution.
    pub fn resolve(
        &mut self,
        request_id: u64,
        identity: PendingIdentity,
    ) -> Result<bool, DeadlineError> {
        let index = self
            .pending
            .iter()
            .position(|item| item.request_id == request_id)
            .ok_or(DeadlineError::MissingRequest(request_id))?;
        if self.pending[index].identity != identity {
            return Ok(false);
        }
        self.pending.remove(index);
        self.resolved += 1;
        Ok(true)
    }

    /// Explicit revocation is recorded separately from successful completion.
    pub fn cancel(&mut self, request_id: u64) -> Result<(), DeadlineError> {
        let index = self
            .pending
            .iter()
            .position(|item| item.request_id == request_id)
            .ok_or(DeadlineError::MissingRequest(request_id))?;
        self.pending.remove(index);
        self.cancelled += 1;
        Ok(())
    }

    #[must_use]
    pub fn pending(&self) -> &[PendingDeadline] {
        &self.pending
    }

    #[must_use]
    pub const fn resolved(&self) -> u64 {
        self.resolved
    }

    #[must_use]
    pub const fn cancelled(&self) -> u64 {
        self.cancelled
    }

    /// A completed trace is not declared settled while deferred obligations
    /// remain, even if their deadlines are in the future.
    #[must_use]
    pub fn settled(&self) -> bool {
        self.pending.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    fn revisions(n: u64) -> InputRevisions {
        InputRevisions { left: n, right: n }
    }

    fn cache(n: u64, value: bool) -> Option<CachedOutput> {
        Some(CachedOutput {
            gate: Gate::Xor,
            revisions: revisions(n),
            value,
        })
    }

    fn costs() -> ProxyBudget {
        ProxyBudget {
            model_revision: 1,
            weights: Work {
                gate_evaluations: 10,
                input_value_reads: 2,
                revision_reads: 1,
                cache_probes: 1,
                cache_validation_comparisons: 2,
                event_decisions: 1,
                deferred_events: 1,
                output_transitions: 1,
                ..Work::default()
            },
            per_event_ceiling: 50,
        }
    }

    #[test]
    fn required_uncached_gate_evaluates_at_matched_cost() {
        let plan = plan_c3(
            Gate::Xor,
            revisions(1),
            None,
            1,
            Obligation::RequiredNow,
            costs(),
        )
        .unwrap();
        assert_eq!(plan.decision(), Decision::Evaluate);
        let run = execute_c3(plan, || (true, false)).unwrap();
        assert_eq!(run.step.output, Some(true));
        assert_eq!(run.step.work.gate_evaluations, 1);
        assert_eq!(run.model_revision, 1);
        assert!(run.proxy_cost <= run.proxy_upper_bound);
    }

    #[test]
    fn fresh_reuse_never_calls_value_reader() {
        let plan = plan_c3(
            Gate::Xor,
            revisions(1),
            cache(1, true),
            1,
            Obligation::RequiredNow,
            costs(),
        )
        .unwrap();
        assert_eq!(plan.decision(), Decision::Reuse);
        let reads = Cell::new(0);
        let run = execute_c3(plan, || {
            reads.set(reads.get() + 1);
            (true, false)
        })
        .unwrap();
        assert_eq!(reads.get(), 0);
        assert_eq!(run.step.output, Some(true));
        assert_eq!(run.step.work.cache_validation_comparisons, 1);
    }

    #[test]
    fn sufficiently_expensive_cache_validation_prefers_evaluation() {
        let mut budget = costs();
        budget.weights.cache_validation_comparisons = 100;
        budget.per_event_ceiling = 200;
        let plan = plan_c3(
            Gate::Xor,
            revisions(1),
            cache(1, true),
            1,
            Obligation::RequiredNow,
            budget,
        )
        .unwrap();
        assert_eq!(plan.decision(), Decision::Evaluate);
        let run = execute_c3(plan, || (true, false)).unwrap();
        assert_eq!(run.step.work.cache_validation_comparisons, 0);
        assert_eq!(run.step.output, Some(true));
    }

    #[test]
    fn stale_cache_inspection_is_billed_even_if_evaluation_follows() {
        let plan = plan_c3(
            Gate::Xor,
            revisions(2),
            cache(1, false),
            2,
            Obligation::RequiredNow,
            costs(),
        )
        .unwrap();
        assert_eq!(plan.decision(), Decision::Evaluate);
        let run = execute_c3(plan, || (true, false)).unwrap();
        assert_eq!(run.step.work.cache_validation_comparisons, 1);
        assert_eq!(run.step.work.output_transitions, 1);
        assert!(run.proxy_cost <= run.proxy_upper_bound);
    }

    #[test]
    fn optional_can_defer_but_due_required_work_cannot_be_suppressed() {
        let mut budget = costs();
        budget.per_event_ceiling = 8;
        let optional = plan_c3(
            Gate::And,
            revisions(1),
            None,
            3,
            Obligation::Optional { deadline_tick: 5 },
            budget,
        )
        .unwrap();
        assert_eq!(optional.decision(), Decision::Defer);
        let reads = Cell::new(0);
        let result = execute_c3(optional, || {
            reads.set(reads.get() + 1);
            (true, true)
        })
        .unwrap();
        assert_eq!(result.step.output, None);
        assert_eq!(reads.get(), 0);
        assert_eq!(
            plan_c3(
                Gate::And,
                revisions(1),
                None,
                5,
                Obligation::Optional { deadline_tick: 5 },
                budget,
            ),
            Err(C3Error::BudgetInsufficient {
                upper_bound: 17,
                ceiling: 8,
            })
        );
    }

    #[test]
    fn later_wakeup_can_resolve_before_deadline() {
        let mut budget = costs();
        budget.per_event_ceiling = 8;
        let deferred = plan_c3(
            Gate::Xor,
            revisions(4),
            None,
            2,
            Obligation::Optional { deadline_tick: 6 },
            budget,
        )
        .unwrap();
        assert_eq!(deferred.decision(), Decision::Defer);
        let id = PendingIdentity {
            gate: Gate::Xor,
            revisions: revisions(4),
        };
        let mut ledger = DeadlineLedger::default();
        ledger.advance(2).unwrap();
        ledger
            .defer(PendingDeadline {
                request_id: 1,
                identity: id,
                deadline_tick: 6,
            })
            .unwrap();
        ledger.advance(4).unwrap();
        budget.per_event_ceiling = 50;
        let executed = plan_c3(
            Gate::Xor,
            revisions(4),
            None,
            4,
            Obligation::RequiredNow,
            budget,
        )
        .unwrap();
        assert_eq!(execute_c3(executed, || (true, false)).unwrap().step.output, Some(true));
        assert!(ledger.resolve(1, id).unwrap());
        assert!(ledger.settled());
        assert_eq!(ledger.resolved(), 1);
        ledger.advance(7).unwrap();
    }

    #[test]
    fn wrong_identity_and_missed_deadline_fail_closed() {
        let mut ledger = DeadlineLedger::default();
        ledger.advance(1).unwrap();
        ledger.defer(PendingDeadline {
            request_id: 7,
            identity: PendingIdentity {
                gate: Gate::Or,
                revisions: revisions(1),
            },
            deadline_tick: 3,
        }).unwrap();
        assert!(!ledger.resolve(7, PendingIdentity {
            gate: Gate::And,
            revisions: revisions(1),
        }).unwrap());
        assert!(!ledger.settled());
        assert_eq!(
            ledger.advance(4),
            Err(DeadlineError::Missed {
                request_id: 7,
                due: 3,
                now: 4,
            })
        );
        ledger.advance(3).unwrap();
        ledger.cancel(7).unwrap();
        assert_eq!(ledger.cancelled(), 1);
        assert!(ledger.settled());
    }

    #[test]
    fn cost_model_deadline_and_boundaries_are_rejected() {
        let mut no_model = costs();
        no_model.model_revision = 0;
        assert_eq!(
            plan_c3(Gate::Xor, revisions(1), None, 1, Obligation::RequiredNow, no_model),
            Err(C3Error::UnknownCostModel)
        );
        let mut no_price = costs();
        no_price.weights.gate_evaluations = 0;
        no_price.weights.input_value_reads = 0;
        assert_eq!(
            plan_c3(Gate::Xor, revisions(1), None, 1, Obligation::RequiredNow, no_price),
            Err(C3Error::UnpricedCompute)
        );
        assert_eq!(
            plan_c3(
                Gate::Xor,
                revisions(1),
                None,
                9,
                Obligation::Optional { deadline_tick: 8 },
                costs()
            ),
            Err(C3Error::AlreadyPastDeadline { now: 9, deadline: 8 })
        );
    }

    #[test]
    fn deadline_ledger_rejects_duplicate_and_time_reversal() {
        let mut ledger = DeadlineLedger::default();
        ledger.advance(2).unwrap();
        let entry = PendingDeadline {
            request_id: 1,
            identity: PendingIdentity {
                gate: Gate::And,
                revisions: revisions(3),
            },
            deadline_tick: 4,
        };
        ledger.defer(entry).unwrap();
        assert_eq!(ledger.defer(entry), Err(DeadlineError::DuplicateRequest(1)));
        assert_eq!(ledger.advance(1), Err(DeadlineError::NonIncreasingTime));
        assert!(!ledger.settled());
    }
}
