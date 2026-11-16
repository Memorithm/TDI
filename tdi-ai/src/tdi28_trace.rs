//! TDI-28 bounded, deterministic Development-only event-trace qualification.
//!
//! This is an evaluator-owned trace harness, not a trained model, physical
//! energy meter, confirmatory experiment, or V888 topology evaluator.
//! Input values are available to the offline consistency oracle, but the
//! online decision path receives only current gate, revisions, demand, and
//! cache; evaluation values pass through a lazy callback.
//! Reference-work counts deliberately exclude evaluator preflight overhead.

use std::collections::BTreeMap;

use super::tdi28_energy_algebra::{
    AccountingError, Arm, CachedOutput, Decision, Demand, Gate, InputRevisions, Work, evaluate,
};

/// Hard bound on test fixture size; no unbounded stream or hardware work.
pub const MAX_TRACE_EVENTS: usize = 4096;

/// One event with fully declared evaluator-owned values and obligations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TraceEvent {
    pub tick: u64,
    pub gate: Gate,
    pub revisions: InputRevisions,
    pub left: bool,
    pub right: bool,
    pub demand: Demand,
}

/// Which dependency violated the unique-version data contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputSide {
    Left,
    Right,
}

/// Rejection types for invalid traces and accounting failures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TraceError {
    Empty,
    TooManyEvents,
    NonIncreasingTick { previous: u64, current: u64 },
    HorizonBeforeLast { horizon: u64, last: u64 },
    ConflictingRevision { side: InputSide, revision: u64 },
    IncorrectOutput { tick: u64 },
    Overflow,
}

impl From<AccountingError> for TraceError {
    fn from(_: AccountingError) -> Self {
        Self::Overflow
    }
}

/// Per-event evidence: all non-required outputs remain explicitly optional.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EventRecord {
    pub tick: u64,
    pub demanded: Demand,
    pub decision: Decision,
    pub baseline_output: bool,
    pub governed_output: Option<bool>,
    pub baseline_work: Work,
    pub governed_work: Work,
}

/// Pending optional requests are not called energy savings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DebtLedger {
    pub deferred_requests: u64,
    pub resolved_requests: u64,
    pub superseded_requests: u64,
    pub outstanding_requests: u64,
}

/// Complete deterministic trace report. No joule values are inferred.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraceReport {
    pub records: Vec<EventRecord>,
    pub baseline_work: Work,
    pub governed_work: Work,
    pub debt: DebtLedger,
}

impl TraceReport {
    /// A cost comparison is admissible only once every deferred request has
    /// been resolved or explicitly superseded by a newer optional state.
    /// Supersession is reported separately, never hidden in the work vector.
    #[must_use]
    pub const fn is_settled(&self) -> bool {
        self.debt.outstanding_requests == 0
    }

    /// Return weighted *proxy* scores only for a fully settled trace.
    ///
    /// This is not a power reading or an energy-savings claim.
    pub fn settled_proxy_scores(
        &self,
        weights: Work,
    ) -> Result<Option<(u128, u128)>, AccountingError> {
        if !self.is_settled() {
            return Ok(None);
        }
        Ok(Some((
            self.baseline_work.proxy_score(weights)?,
            self.governed_work.proxy_score(weights)?,
        )))
    }
}

#[derive(Clone, Copy)]
struct Pending {
    gate: Gate,
    revisions: InputRevisions,
    count: u64,
}

fn checked_increment(value: &mut u64, by: u64) -> Result<(), TraceError> {
    *value = value.checked_add(by).ok_or(TraceError::Overflow)?;
    Ok(())
}

fn checked_revision(
    seen: &mut BTreeMap<u64, bool>,
    revision: u64,
    value: bool,
    side: InputSide,
) -> Result<(), TraceError> {
    match seen.get(&revision) {
        Some(previous) if *previous != value => {
            Err(TraceError::ConflictingRevision { side, revision })
        }
        Some(_) => Ok(()),
        None => {
            seen.insert(revision, value);
            Ok(())
        }
    }
}

/// Validate all evaluator-owned declared input identities independently of
/// either candidate. This offline oracle is **not** an online policy feature.
/// A reused version with a changed value rejects the entire trace.
fn preflight(events: &[TraceEvent], horizon: u64) -> Result<(), TraceError> {
    if events.is_empty() {
        return Err(TraceError::Empty);
    }
    if events.len() > MAX_TRACE_EVENTS {
        return Err(TraceError::TooManyEvents);
    }
    let mut left = BTreeMap::new();
    let mut right = BTreeMap::new();
    let mut previous = None;
    for event in events {
        if let Some(prior) = previous {
            if event.tick <= prior {
                return Err(TraceError::NonIncreasingTick {
                    previous: prior,
                    current: event.tick,
                });
            }
        }
        previous = Some(event.tick);
        checked_revision(
            &mut left,
            event.revisions.left,
            event.left,
            InputSide::Left,
        )?;
        checked_revision(
            &mut right,
            event.revisions.right,
            event.right,
            InputSide::Right,
        )?;
    }
    let last = previous.expect("non-empty trace");
    if horizon < last {
        return Err(TraceError::HorizonBeforeLast { horizon, last });
    }
    Ok(())
}

/// Replay exactly the same trace through always-evaluate and governed arms.
///
/// - The online policy is never passed future events or reference outcomes.
/// - Required outputs must match exactly at each event.
/// - A deferred optional request remains visible as debt until a matching
///   computation resolves it or a different gate/dependency supersedes it.
/// - Idle time is charged identically across arms, including the tail to
///   `horizon`, but the work vector is not calibrated physical energy.
/// - All counters use checked arithmetic; no silent saturation.
pub fn replay_pair(events: &[TraceEvent], horizon: u64) -> Result<TraceReport, TraceError> {
    preflight(events, horizon)?;

    let mut records = Vec::with_capacity(events.len());
    let mut baseline_cache: Option<CachedOutput> = None;
    let mut governed_cache: Option<CachedOutput> = None;
    let mut baseline_work = Work::default();
    let mut governed_work = Work::default();
    let mut last_tick = None;
    let mut pending: Option<Pending> = None;
    let mut deferred_requests = 0u64;
    let mut resolved_requests = 0u64;
    let mut superseded_requests = 0u64;

    for event in events {
        let idle = match last_tick {
            None => event.tick,
            Some(previous) => event.tick - previous - 1,
        };
        let idle_work = Work {
            idle_ticks: idle,
            ..Work::default()
        };
        baseline_work = baseline_work.checked_add(idle_work)?;
        governed_work = governed_work.checked_add(idle_work)?;
        last_tick = Some(event.tick);

        if let Some(debt) = pending {
            if debt.gate != event.gate || debt.revisions != event.revisions {
                checked_increment(&mut superseded_requests, debt.count)?;
                pending = None;
            }
        }

        let baseline = evaluate(
            Arm::AttributedAlwaysEvaluate,
            event.gate,
            event.revisions,
            event.demand,
            baseline_cache,
            || (event.left, event.right),
        );
        let governed = evaluate(
            Arm::EnergyGoverned,
            event.gate,
            event.revisions,
            event.demand,
            governed_cache,
            || (event.left, event.right),
        );
        let reference = baseline.output.expect("reference always evaluates");
        if event.demand == Demand::Required && governed.output != Some(reference) {
            return Err(TraceError::IncorrectOutput { tick: event.tick });
        }
        if governed.output.is_some_and(|output| output != reference) {
            return Err(TraceError::IncorrectOutput { tick: event.tick });
        }

        match governed.decision {
            Decision::Defer => {
                checked_increment(&mut deferred_requests, 1)?;
                match &mut pending {
                    Some(debt) => checked_increment(&mut debt.count, 1)?,
                    None => {
                        pending = Some(Pending {
                            gate: event.gate,
                            revisions: event.revisions,
                            count: 1,
                        });
                    }
                }
            }
            Decision::Evaluate | Decision::Reuse => {
                if let Some(debt) = pending.take() {
                    checked_increment(&mut resolved_requests, debt.count)?;
                }
            }
        }

        baseline_work = baseline_work.checked_add(baseline.work)?;
        governed_work = governed_work.checked_add(governed.work)?;
        baseline_cache = baseline.cache;
        governed_cache = governed.cache;
        records.push(EventRecord {
            tick: event.tick,
            demanded: event.demand,
            decision: governed.decision,
            baseline_output: reference,
            governed_output: governed.output,
            baseline_work: baseline.work,
            governed_work: governed.work,
        });
    }

    let last = last_tick.expect("non-empty trace");
    let tail_idle = horizon - last;
    let idle_work = Work {
        idle_ticks: tail_idle,
        ..Work::default()
    };
    baseline_work = baseline_work.checked_add(idle_work)?;
    governed_work = governed_work.checked_add(idle_work)?;

    Ok(TraceReport {
        records,
        baseline_work,
        governed_work,
        debt: DebtLedger {
            deferred_requests,
            resolved_requests,
            superseded_requests,
            outstanding_requests: pending.map_or(0, |debt| debt.count),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(tick: u64, version: u64, left: bool, demand: Demand) -> TraceEvent {
        TraceEvent {
            tick,
            gate: Gate::Xor,
            revisions: InputRevisions {
                left: version,
                right: 1,
            },
            left,
            right: false,
            demand,
        }
    }

    #[test]
    fn repeated_required_events_preserve_outputs_and_skip_evaluations() {
        let events = [
            event(0, 1, false, Demand::Required),
            event(2, 1, false, Demand::Required),
            event(5, 2, true, Demand::Required),
            event(7, 2, true, Demand::Required),
        ];
        let report = replay_pair(&events, 10).unwrap();
        assert!(report.is_settled());
        assert_eq!(report.baseline_work.gate_evaluations, 4);
        assert_eq!(report.governed_work.gate_evaluations, 2);
        assert_eq!(report.baseline_work.idle_ticks, 7);
        assert_eq!(report.governed_work.idle_ticks, 7);
        assert_eq!(report.records[1].decision, Decision::Reuse);
        assert_eq!(report.records[3].governed_output, Some(true));
    }

    #[test]
    fn deferred_optional_is_later_resolved_at_required_boundary() {
        let events = [
            event(0, 1, false, Demand::Optional),
            event(2, 1, false, Demand::Optional),
            event(4, 1, false, Demand::Required),
        ];
        let report = replay_pair(&events, 4).unwrap();
        assert_eq!(report.debt.deferred_requests, 2);
        assert_eq!(report.debt.resolved_requests, 2);
        assert_eq!(report.debt.superseded_requests, 0);
        assert_eq!(report.debt.outstanding_requests, 0);
        assert_eq!(report.records[0].governed_output, None);
        assert_eq!(report.records[2].governed_output, Some(false));
        assert_eq!(report.governed_work.gate_evaluations, 1);
        assert!(report.settled_proxy_scores(Work::default()).unwrap().is_some());
    }

    #[test]
    fn obsolete_optional_request_is_reported_as_superseded() {
        let events = [
            event(0, 1, false, Demand::Optional),
            event(1, 2, true, Demand::Required),
        ];
        let report = replay_pair(&events, 1).unwrap();
        assert!(report.is_settled());
        assert_eq!(report.debt.superseded_requests, 1);
        assert_eq!(report.debt.resolved_requests, 0);
        assert_eq!(report.records[0].governed_output, None);
        assert_eq!(report.records[1].governed_output, Some(true));
    }

    #[test]
    fn outstanding_deferred_work_blocks_cost_verdict() {
        let report = replay_pair(&[event(3, 1, true, Demand::Optional)], 7).unwrap();
        assert_eq!(report.debt.outstanding_requests, 1);
        assert_eq!(report.baseline_work.idle_ticks, 7);
        assert_eq!(report.governed_work.idle_ticks, 7);
        assert_eq!(report.settled_proxy_scores(Work::default()).unwrap(), None);
    }

    #[test]
    fn conflicting_revision_is_rejected_by_offline_oracle() {
        let events = [
            event(0, 9, false, Demand::Required),
            event(1, 9, true, Demand::Required),
        ];
        assert_eq!(
            replay_pair(&events, 1),
            Err(TraceError::ConflictingRevision {
                side: InputSide::Left,
                revision: 9,
            })
        );
    }

    #[test]
    fn nonincreasing_ticks_and_short_horizon_are_rejected() {
        let one = event(2, 1, false, Demand::Required);
        assert_eq!(replay_pair(&[], 0), Err(TraceError::Empty));
        assert_eq!(
            replay_pair(&[one, one], 2),
            Err(TraceError::NonIncreasingTick {
                previous: 2,
                current: 2,
            })
        );
        assert_eq!(
            replay_pair(&[one], 1),
            Err(TraceError::HorizonBeforeLast {
                horizon: 1,
                last: 2,
            })
        );
    }

    #[test]
    fn adversarial_weights_can_make_governed_reference_costlier() {
        let events = [
            event(0, 1, false, Demand::Required),
            event(1, 1, false, Demand::Required),
        ];
        let report = replay_pair(&events, 1).unwrap();
        let heavy_cache = Work {
            cache_validation_comparisons: 100,
            gate_evaluations: 1,
            ..Work::default()
        };
        let scores = report.settled_proxy_scores(heavy_cache).unwrap().unwrap();
        // The governed arm checks cache validity. The always-evaluate arm
        // does not; an expensive check can outweigh one skipped gate.
        assert_eq!(scores, (2, 101));
    }

    #[test]
    fn too_large_trace_rejects_before_reference_execution() {
        let too_many = vec![event(0, 1, false, Demand::Required); MAX_TRACE_EVENTS + 1];
        assert_eq!(
            replay_pair(&too_many, 0),
            Err(TraceError::TooManyEvents)
        );
    }
}
