//! TDI-28 Stage-0 four-arm, matched-event Development reference.
//!
//! C0 and C1 deliberately execute the identical always-evaluate reference;
//! C1 additionally exposes the same logical work vector. C2 is unconditional
//! versioned reuse/defer. C3 is prospective synthetic-cost selection with
//! explicit deadlines and budget admission. All receive the same events,
//! declared proxy coefficients, and per-event ceilings. These *logical*
//! counters exclude offline preflight and ledger book-keeping, so they are
//! not hardware energy, memory traffic, or end-to-end runtime measurements.

use super::tdi28_c3_policy::{
    C3Error, DeadlineError, DeadlineLedger, Obligation, PendingDeadline, PendingIdentity,
    ProxyBudget, execute_c3, plan_c3,
};
use super::tdi28_energy_algebra::{AccountingError, CachedOutput, Decision, Demand, Work};
use super::tdi28_trace::{DebtLedger, TraceError, TraceEvent, replay_pair};

/// A single matched event with a predeclared, arm-independent cost ceiling.
///
/// Optional observations require an explicit absolute logical deadline.
/// A required output may not carry an optional deadline.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MatchedEvent {
    pub event: TraceEvent,
    pub optional_deadline: Option<u64>,
    pub event_ceiling: u128,
}

/// Predeclared synthetic coefficients shared by all four arms.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MatchedCostModel {
    pub model_revision: u64,
    pub weights: Work,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FourError {
    Trace(TraceError),
    C3(C3Error),
    Deadline(DeadlineError),
    Accounting(AccountingError),
    MissingOptionalDeadline { tick: u64 },
    UnexpectedRequiredDeadline { tick: u64 },
    WrongC3Output { tick: u64 },
}

impl From<TraceError> for FourError {
    fn from(error: TraceError) -> Self {
        Self::Trace(error)
    }
}

impl From<C3Error> for FourError {
    fn from(error: C3Error) -> Self {
        Self::C3(error)
    }
}

impl From<DeadlineError> for FourError {
    fn from(error: DeadlineError) -> Self {
        Self::Deadline(error)
    }
}

impl From<AccountingError> for FourError {
    fn from(error: AccountingError) -> Self {
        Self::Accounting(error)
    }
}

/// One event's prospective quality/cost/accounting evidence in every arm.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FourEventRecord {
    pub tick: u64,
    pub demand: Demand,
    pub c0_output: bool,
    pub c2_output: Option<bool>,
    pub c3_output: Option<bool>,
    pub c2_decision: Decision,
    pub c3_decision: Decision,
    /// Costs here exclude idle ticks, which are reported in aggregate.
    pub event_proxy_costs: [u128; 4],
    pub event_ceiling: u128,
}

/// Scores are made available only when every arm delivered the same
/// observable output at **every** event and no obligation is outstanding.
/// This is a work-proxy comparison, not evidence of real energy savings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FourProxyScores {
    pub c0: u128,
    pub c1: u128,
    pub c2: u128,
    pub c3: u128,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FourArmReport {
    /// Fixed once for the entire matched replay; the report's scoring method
    /// never accepts replacement coefficients after outcomes are observed.
    cost_model: MatchedCostModel,
    pub events: Vec<FourEventRecord>,
    pub c0_work: Work,
    pub c1_work: Work,
    pub c2_work: Work,
    pub c3_work: Work,
    /// Number of observed per-event ceiling exceedances for each arm.
    /// C0/C1/C2 have no budget-admission behavior: these are diagnostics.
    /// C3 rejects an unadmissible event and cannot report a violation.
    pub budget_exceedances: [u64; 4],
    /// Counts from the original C2 trace reference; they expose deferred
    /// optional observations even if later superseded or resolved.
    pub c2_debt: DebtLedger,
    /// C3 pending requests require explicit matching wakeup evidence.
    pub c3_deadlines: DeadlineLedger,
}

impl FourArmReport {
    #[must_use]
    pub const fn cost_model(&self) -> MatchedCostModel {
        self.cost_model
    }

    /// Strict equal-coverage, equal-output, no-debt evidence.
    #[must_use]
    pub fn strict_quality_parity(&self) -> bool {
        self.c2_debt.outstanding_requests == 0
            && self.c2_debt.deferred_requests == 0
            && self.c3_deadlines.settled()
            && self.c3_deadlines.cancelled() == 0
            && self.events.iter().all(|item| {
                item.c2_output == Some(item.c0_output) && item.c3_output == Some(item.c0_output)
            })
    }

    /// Returns `None` rather than mislabeling deferred, cancelled or
    /// uncovered optional work as equal-utility cost savings.
    pub fn strict_proxy_scores(&self) -> Result<Option<FourProxyScores>, AccountingError> {
        if !self.strict_quality_parity() {
            return Ok(None);
        }
        let weights = self.cost_model.weights;
        Ok(Some(FourProxyScores {
            c0: self.c0_work.proxy_score(weights)?,
            c1: self.c1_work.proxy_score(weights)?,
            c2: self.c2_work.proxy_score(weights)?,
            c3: self.c3_work.proxy_score(weights)?,
        }))
    }
}

/// Replays matched events. Every policy observes only current gate, versions,
/// demand/deadline, cache identity and the predeclared cost model. Offline
/// reference values enter only the independent reference and lazy evaluation
/// callbacks; no future-value, output or alternative-arm leakage to C3.
///
/// A returned report means all mandatory outputs were correct. Pending C3
/// deadlines beyond the trace's final tick may be allowed only when the
/// supplied horizon has not already missed them; they remain visible as debt.
/// Unadmitted mandatory C3 work fails closed with a typed error.
pub fn replay_four(
    events: &[MatchedEvent],
    horizon: u64,
    model: MatchedCostModel,
) -> Result<FourArmReport, FourError> {
    let baseline_events: Vec<_> = events.iter().map(|item| item.event).collect();
    // Reuse the existing offline revision-consistency/ordering oracle.
    let pair = replay_pair(&baseline_events, horizon)?;

    for item in events {
        match (item.event.demand, item.optional_deadline) {
            (Demand::Required, Some(_)) => {
                return Err(FourError::UnexpectedRequiredDeadline {
                    tick: item.event.tick,
                });
            }
            (Demand::Optional, None) => {
                return Err(FourError::MissingOptionalDeadline {
                    tick: item.event.tick,
                });
            }
            _ => {}
        }
    }

    let mut records = Vec::with_capacity(events.len());
    let mut ledger = DeadlineLedger::default();
    let mut cache: Option<CachedOutput> = None;
    let mut c3_work = Work::default();
    let mut budget_exceedances = [0u64; 4];
    let mut last_tick = None;

    for (index, item) in events.iter().enumerate() {
        let event = item.event;
        ledger.advance(event.tick)?;

        let gap = match last_tick {
            Some(previous) => event.tick - previous - 1,
            None => event.tick,
        };
        c3_work = c3_work.checked_add(Work {
            idle_ticks: gap,
            ..Work::default()
        })?;
        last_tick = Some(event.tick);

        let obligation = match event.demand {
            Demand::Required => Obligation::RequiredNow,
            Demand::Optional => Obligation::Optional {
                deadline_tick: item.optional_deadline.expect("validated"),
            },
        };
        let budget = ProxyBudget {
            model_revision: model.model_revision,
            weights: model.weights,
            per_event_ceiling: item.event_ceiling,
        };
        let plan = plan_c3(
            event.gate,
            event.revisions,
            cache,
            event.tick,
            obligation,
            budget,
        )?;
        let execution = execute_c3(plan, || (event.left, event.right))?;
        let step = execution.step();

        // Resolve every older deferred request backed by this *current*
        // C3 result. A stale/different revision never resolves another
        // obligation and pending deadlines are not silently cancelled.
        if step.output.is_some() {
            let current_identity = execution.identity();
            let to_resolve: Vec<_> = ledger
                .pending()
                .iter()
                .filter(|pending| pending.identity == current_identity)
                .map(|pending| pending.request_id)
                .collect();
            for request_id in to_resolve {
                let resolved = ledger.resolve(request_id, &execution)?;
                debug_assert!(resolved, "matching C3 execution resolves its debt");
            }
        } else {
            let deadline_tick = item.optional_deadline.expect("deferred optional");
            ledger.defer(
                PendingDeadline {
                    request_id: u64::try_from(index + 1).expect("bounded event count"),
                    identity: PendingIdentity {
                        gate: event.gate,
                        revisions: event.revisions,
                    },
                    deadline_tick,
                },
                &execution,
            )?;
        }

        let baseline = pair.records[index];
        if step
            .output
            .is_some_and(|value| value != baseline.baseline_output)
            || (event.demand == Demand::Required && step.output != Some(baseline.baseline_output))
        {
            return Err(FourError::WrongC3Output { tick: event.tick });
        }

        // C0 and C1 deliberately have the *same* execution and counters.
        // C1 only exposes the vector; there is no fabricated runtime gain.
        let c0 = baseline.baseline_work.proxy_score(model.weights)?;
        let c1 = c0;
        let c2 = baseline.governed_work.proxy_score(model.weights)?;
        let c3 = execution.proxy_cost();
        let costs = [c0, c1, c2, c3];
        for (arm, cost) in costs.into_iter().enumerate() {
            if cost > item.event_ceiling {
                budget_exceedances[arm] = budget_exceedances[arm]
                    .checked_add(1)
                    .ok_or(AccountingError::Overflow)?;
            }
        }
        c3_work = c3_work.checked_add(step.work)?;
        cache = step.cache;
        records.push(FourEventRecord {
            tick: event.tick,
            demand: event.demand,
            c0_output: baseline.baseline_output,
            c2_output: baseline.governed_output,
            c3_output: step.output,
            c2_decision: baseline.decision,
            c3_decision: step.decision,
            event_proxy_costs: costs,
            event_ceiling: item.event_ceiling,
        });
    }

    ledger.advance(horizon)?;
    let last = last_tick.expect("non-empty pair validated");
    c3_work = c3_work.checked_add(Work {
        idle_ticks: horizon - last,
        ..Work::default()
    })?;
    Ok(FourArmReport {
        cost_model: model,
        events: records,
        c0_work: pair.baseline_work,
        c1_work: pair.baseline_work,
        c2_work: pair.governed_work,
        c3_work,
        budget_exceedances,
        c2_debt: pair.debt,
        c3_deadlines: ledger,
    })
}

#[cfg(test)]
mod tests {
    use super::super::tdi28_energy_algebra::{Gate, InputRevisions};
    use super::*;

    fn model(cache_validation: u64) -> MatchedCostModel {
        MatchedCostModel {
            model_revision: 1,
            weights: Work {
                gate_evaluations: 10,
                input_value_reads: 2,
                revision_reads: 1,
                cache_probes: 1,
                cache_validation_comparisons: cache_validation,
                output_transitions: 1,
                event_decisions: 1,
                deferred_events: 1,
                idle_ticks: 1,
            },
        }
    }

    fn event(
        tick: u64,
        revision: u64,
        left: bool,
        demand: Demand,
        deadline: Option<u64>,
        ceiling: u128,
    ) -> MatchedEvent {
        MatchedEvent {
            event: TraceEvent {
                tick,
                gate: Gate::Xor,
                revisions: InputRevisions {
                    left: revision,
                    right: 1,
                },
                left,
                right: false,
                demand,
            },
            optional_deadline: deadline,
            event_ceiling: ceiling,
        }
    }

    #[test]
    fn required_unchanged_inputs_give_strict_parity_and_lower_proxy_work() {
        let events = [
            event(0, 1, false, Demand::Required, None, 50),
            event(2, 1, false, Demand::Required, None, 50),
            event(5, 2, true, Demand::Required, None, 50),
            event(7, 2, true, Demand::Required, None, 50),
        ];
        let report = replay_four(&events, 10, model(2)).unwrap();
        assert!(report.strict_quality_parity());
        let scores = report
            .strict_proxy_scores()
            .unwrap()
            .unwrap();
        assert_eq!(scores.c0, scores.c1);
        assert!(scores.c2 < scores.c0);
        assert!(scores.c3 < scores.c0);
        assert_eq!(report.c3_work.gate_evaluations, 2);
        assert_eq!(report.c3_work.idle_ticks, report.c0_work.idle_ticks);
        assert_eq!(report.budget_exceedances, [0, 0, 0, 0]);
    }

    #[test]
    fn expensive_cache_validation_makes_c3_recompute_while_c2_reuses() {
        let events = [
            event(0, 1, false, Demand::Required, None, 250),
            event(1, 1, false, Demand::Required, None, 250),
        ];
        let report = replay_four(&events, 1, model(100)).unwrap();
        assert!(report.strict_quality_parity());
        assert_eq!(report.events[1].c2_decision, Decision::Reuse);
        assert_eq!(report.events[1].c3_decision, Decision::Evaluate);
        let scores = report
            .strict_proxy_scores()
            .unwrap()
            .unwrap();
        assert!(scores.c3 < scores.c2);
    }

    #[test]
    fn optional_debt_can_be_resolved_but_quality_is_not_falsely_equal() {
        let events = [
            event(0, 1, false, Demand::Optional, Some(4), 8),
            event(2, 1, false, Demand::Required, None, 50),
        ];
        let report = replay_four(&events, 4, model(2)).unwrap();
        assert_eq!(report.events[0].c3_decision, Decision::Defer);
        assert_eq!(report.events[0].c2_decision, Decision::Defer);
        assert_eq!(report.c3_deadlines.resolved(), 1);
        assert!(report.c3_deadlines.settled());
        assert!(!report.strict_quality_parity());
        assert_eq!(report.strict_proxy_scores().unwrap(), None);
    }

    #[test]
    fn overdue_optional_debt_is_rejected_even_if_no_more_events_arrive() {
        let events = [event(0, 1, false, Demand::Optional, Some(3), 8)];
        assert_eq!(
            replay_four(&events, 4, model(2)),
            Err(FourError::Deadline(DeadlineError::Missed {
                request_id: 1,
                due: 3,
                now: 4
            }))
        );
        let unsettled = replay_four(&events, 3, model(2)).unwrap();
        assert!(!unsettled.c3_deadlines.settled());
        assert_eq!(
            unsettled.strict_proxy_scores().unwrap(),
            None
        );
    }

    #[test]
    fn required_event_over_budget_fails_closed() {
        let events = [event(0, 1, false, Demand::Required, None, 8)];
        assert!(matches!(
            replay_four(&events, 0, model(2)),
            Err(FourError::C3(C3Error::BudgetInsufficient { .. }))
        ));
    }

    #[test]
    fn invalid_deadline_shapes_and_conflicting_revisions_are_rejected() {
        let no_deadline = [event(0, 1, false, Demand::Optional, None, 30)];
        assert_eq!(
            replay_four(&no_deadline, 0, model(2)),
            Err(FourError::MissingOptionalDeadline { tick: 0 })
        );
        let bad_required = [event(0, 1, false, Demand::Required, Some(3), 30)];
        assert_eq!(
            replay_four(&bad_required, 3, model(2)),
            Err(FourError::UnexpectedRequiredDeadline { tick: 0 })
        );
        let conflicting = [
            event(0, 1, false, Demand::Required, None, 50),
            event(1, 1, true, Demand::Required, None, 50),
        ];
        assert!(matches!(
            replay_four(&conflicting, 1, model(2)),
            Err(FourError::Trace(TraceError::ConflictingRevision { .. }))
        ));
    }

    #[test]
    fn matched_ceiling_exceedances_are_exposed_without_rewriting_baselines() {
        let events = [event(0, 1, false, Demand::Required, None, 18)];
        let report = replay_four(&events, 0, model(2)).unwrap();
        assert_eq!(report.budget_exceedances, [0, 0, 0, 0]);
        assert!(report.strict_quality_parity());
        let cheaper_budget = [event(0, 1, false, Demand::Optional, Some(3), 8)];
        let diagnostic = replay_four(&cheaper_budget, 3, model(2)).unwrap();
        assert_eq!(diagnostic.budget_exceedances, [1, 1, 0, 0]);
        assert_eq!(diagnostic.c3_deadlines.pending().len(), 1);
    }
}
