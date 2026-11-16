//! TDI-28 Stage-0: two experimental energy-aware Boolean-calculus approaches.
//!
//! This is a deterministic logical-work oracle, NOT a physical energy model.
//! Work counters become energy estimates only with an independently calibrated,
//! hardware-specific weight table. Inputs are provided lazily so reuse/deferral
//! can be verified not to request input values at the reference-API boundary.
//! Revision stamps are assumed truthful and collision-free under the caller's
//! dependency-tracking contract.

/// Boolean gate evaluated in a deliberately small, deterministic oracle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gate {
    And,
    Or,
    Xor,
}

impl Gate {
    #[must_use]
    pub const fn apply(self, left: bool, right: bool) -> bool {
        match self {
            Self::And => left && right,
            Self::Or => left || right,
            Self::Xor => left ^ right,
        }
    }
}

/// Exact identities of both input versions, established before selecting a path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InputRevisions {
    pub left: u64,
    pub right: u64,
}

/// A caller-maintained output with its complete input dependency identity.
///
/// Stamps must change whenever any dependency changes; a false/stale stamp
/// violates the cache contract and is not detectable without reading inputs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CachedOutput {
    pub revisions: InputRevisions,
    pub gate: Gate,
    pub value: bool,
}

/// Arm A always evaluates but attributes logical work to the operation.
/// Arm B can reuse a proven-fresh cache or defer optional observations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arm {
    AttributedAlwaysEvaluate,
    EnergyGoverned,
}

/// Whether a new output is required at this exact event boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Demand {
    Required,
    Optional,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    Evaluate,
    Reuse,
    Defer,
}

/// Separate, exact reference event counts, not joules/instructions/latency.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Work {
    pub gate_evaluations: u64,
    pub input_value_reads: u64,
    pub revision_reads: u64,
    pub cache_probes: u64,
    pub output_transitions: u64,
    pub event_decisions: u64,
    pub deferred_events: u64,
    pub idle_ticks: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccountingError {
    Overflow,
}

impl Work {
    /// Checked composition of independent event-work records.
    pub fn checked_add(self, other: Self) -> Result<Self, AccountingError> {
        Ok(Self {
            gate_evaluations: self.gate_evaluations.checked_add(other.gate_evaluations).ok_or(AccountingError::Overflow)?,
            input_value_reads: self.input_value_reads.checked_add(other.input_value_reads).ok_or(AccountingError::Overflow)?,
            revision_reads: self.revision_reads.checked_add(other.revision_reads).ok_or(AccountingError::Overflow)?,
            cache_probes: self.cache_probes.checked_add(other.cache_probes).ok_or(AccountingError::Overflow)?,
            output_transitions: self.output_transitions.checked_add(other.output_transitions).ok_or(AccountingError::Overflow)?,
            event_decisions: self.event_decisions.checked_add(other.event_decisions).ok_or(AccountingError::Overflow)?,
            deferred_events: self.deferred_events.checked_add(other.deferred_events).ok_or(AccountingError::Overflow)?,
            idle_ticks: self.idle_ticks.checked_add(other.idle_ticks).ok_or(AccountingError::Overflow)?,
        })
    }

    /// Weighted *proxy* units using explicit, caller-supplied coefficients.
    /// Zero or arbitrary coefficients are not calibrated physical energy.
    pub fn proxy_score(self, weights: Work) -> Result<u128, AccountingError> {
        let pairs = [
            (self.gate_evaluations, weights.gate_evaluations),
            (self.input_value_reads, weights.input_value_reads),
            (self.revision_reads, weights.revision_reads),
            (self.cache_probes, weights.cache_probes),
            (self.output_transitions, weights.output_transitions),
            (self.event_decisions, weights.event_decisions),
            (self.deferred_events, weights.deferred_events),
            (self.idle_ticks, weights.idle_ticks),
        ];
        pairs
            .into_iter()
            .try_fold(0u128, |total, (n, weight)| {
                total.checked_add(u128::from(n) * u128::from(weight))
            })
            .ok_or(AccountingError::Overflow)
    }
}

/// The *observed* decision and its optional output.
///
/// Deferred work is not counted as completed and the prior cached state is
/// returned only as a reusable object, never as a valid current output.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Step {
    pub decision: Decision,
    pub output: Option<bool>,
    pub cache: Option<CachedOutput>,
    pub work: Work,
}

/// Compare the two approaches on identical gate, dependency stamps and demand.
///
/// The supplied input reader MUST be side-effect-free and stable for the
/// declared revisions; it is not invoked for REUSE or DEFER. This is an exact
/// reference-semantic claim, not proof that a real compiler/hardware read zero
/// bytes. `cache` is never passed to the input reader.
pub fn evaluate<F>(
    arm: Arm,
    gate: Gate,
    revisions: InputRevisions,
    demand: Demand,
    cache: Option<CachedOutput>,
    read_values: F,
) -> Step
where
    F: FnOnce() -> (bool, bool),
{
    let mut work = Work {
        revision_reads: 2,
        cache_probes: u64::from(cache.is_some()),
        event_decisions: 1,
        ..Work::default()
    };

    let fresh = cache.filter(|saved| saved.revisions == revisions && saved.gate == gate);
    let decision = match arm {
        Arm::AttributedAlwaysEvaluate => Decision::Evaluate,
        Arm::EnergyGoverned if fresh.is_some() => Decision::Reuse,
        Arm::EnergyGoverned if demand == Demand::Optional => Decision::Defer,
        Arm::EnergyGoverned => Decision::Evaluate,
    };

    match decision {
        Decision::Evaluate => {
            let (left, right) = read_values();
            let output = gate.apply(left, right);
            work.input_value_reads = 2;
            work.gate_evaluations = 1;
            work.output_transitions = u64::from(cache.is_some_and(|saved| saved.value != output));
            Step {
                decision,
                output: Some(output),
                cache: Some(CachedOutput {
                    revisions,
                    gate,
                    value: output,
                }),
                work,
            }
        }
        Decision::Reuse => Step {
            decision,
            output: fresh.map(|saved| saved.value),
            cache: fresh,
            work,
        },
        Decision::Defer => {
            work.deferred_events = 1;
            Step {
                decision,
                output: None,
                cache,
                work,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    fn stamp(version: u64) -> InputRevisions {
        InputRevisions {
            left: version,
            right: version,
        }
    }

    #[test]
    fn fresh_reuse_does_not_request_input_values() {
        let prior = CachedOutput {
            revisions: stamp(1),
            gate: Gate::And,
            value: false,
        };
        let reads = Cell::new(0);
        let step = evaluate(
            Arm::EnergyGoverned,
            Gate::And,
            stamp(1),
            Demand::Required,
            Some(prior),
            || {
                reads.set(reads.get() + 1);
                (false, true)
            },
        );
        assert_eq!(step.decision, Decision::Reuse);
        assert_eq!(step.output, Some(false));
        assert_eq!(step.work.input_value_reads, 0);
        assert_eq!(step.work.gate_evaluations, 0);
        assert_eq!(reads.get(), 0);
    }

    #[test]
    fn stale_dependency_forces_required_recomputation() {
        let prior = CachedOutput {
            revisions: stamp(1),
            gate: Gate::Xor,
            value: false,
        };
        let step = evaluate(
            Arm::EnergyGoverned,
            Gate::Xor,
            stamp(2),
            Demand::Required,
            Some(prior),
            || (true, false),
        );
        assert_eq!(step.decision, Decision::Evaluate);
        assert_eq!(step.output, Some(true));
        assert_eq!(step.work.output_transitions, 1);
        assert_eq!(step.work.input_value_reads, 2);
        assert_eq!(step.cache.unwrap().revisions, stamp(2));
    }

    #[test]
    fn different_gate_invalidates_even_equal_input_versions() {
        let step = evaluate(
            Arm::EnergyGoverned,
            Gate::Or,
            stamp(7),
            Demand::Required,
            Some(CachedOutput {
                revisions: stamp(7),
                gate: Gate::And,
                value: false,
            }),
            || (true, false),
        );
        assert_eq!(step.decision, Decision::Evaluate);
        assert_eq!(step.output, Some(true));
    }

    #[test]
    fn optional_stale_event_is_deferred_without_silent_output() {
        let reads = Cell::new(0);
        let step = evaluate(
            Arm::EnergyGoverned,
            Gate::Or,
            stamp(3),
            Demand::Optional,
            None,
            || {
                reads.set(reads.get() + 1);
                (true, false)
            },
        );
        assert_eq!(step.decision, Decision::Defer);
        assert_eq!(step.output, None);
        assert_eq!(step.work.deferred_events, 1);
        assert_eq!(reads.get(), 0);
    }

    #[test]
    fn always_evaluate_preserves_control_even_when_optional_and_cached() {
        let step = evaluate(
            Arm::AttributedAlwaysEvaluate,
            Gate::And,
            stamp(2),
            Demand::Optional,
            Some(CachedOutput {
                revisions: stamp(2),
                gate: Gate::And,
                value: true,
            }),
            || (true, false),
        );
        assert_eq!(step.decision, Decision::Evaluate);
        assert_eq!(step.output, Some(false));
        assert_eq!(step.work.output_transitions, 1);
    }

    #[test]
    fn required_stream_has_identical_outputs_with_truthful_revision_tracking() {
        let inputs = [
            (stamp(1), false, false),
            (stamp(1), false, false),
            (stamp(2), true, false),
            (stamp(2), true, false),
            (stamp(3), true, true),
        ];
        let mut baseline_cache = None;
        let mut governed_cache = None;
        let mut baseline_work = Work::default();
        let mut governed_work = Work::default();
        for (revisions, left, right) in inputs {
            let baseline = evaluate(
                Arm::AttributedAlwaysEvaluate,
                Gate::Xor,
                revisions,
                Demand::Required,
                baseline_cache,
                || (left, right),
            );
            let governed = evaluate(
                Arm::EnergyGoverned,
                Gate::Xor,
                revisions,
                Demand::Required,
                governed_cache,
                || (left, right),
            );
            assert_eq!(baseline.output, governed.output);
            baseline_cache = baseline.cache;
            governed_cache = governed.cache;
            baseline_work = baseline_work.checked_add(baseline.work).unwrap();
            governed_work = governed_work.checked_add(governed.work).unwrap();
        }
        assert_eq!(baseline_work.gate_evaluations, 5);
        assert_eq!(governed_work.gate_evaluations, 3);
        assert_eq!(baseline_work.input_value_reads, 10);
        assert_eq!(governed_work.input_value_reads, 6);
    }

    #[test]
    fn weighting_can_reverse_the_preferred_arm() {
        let evaluated = Work {
            gate_evaluations: 1,
            input_value_reads: 2,
            event_decisions: 1,
            revision_reads: 2,
            ..Work::default()
        };
        let reused = Work {
            event_decisions: 1,
            revision_reads: 2,
            cache_probes: 1,
            ..Work::default()
        };
        let expensive_compute = Work {
            gate_evaluations: 20,
            input_value_reads: 10,
            cache_probes: 1,
            ..Work::default()
        };
        let expensive_cache = Work {
            gate_evaluations: 1,
            input_value_reads: 1,
            cache_probes: 100,
            ..Work::default()
        };
        assert!(reused.proxy_score(expensive_compute).unwrap() < evaluated.proxy_score(expensive_compute).unwrap());
        assert!(evaluated.proxy_score(expensive_cache).unwrap() < reused.proxy_score(expensive_cache).unwrap());
    }

    #[test]
    fn counters_reject_overflow_and_record_idle_cost_separately() {
        let maxed = Work {
            gate_evaluations: u64::MAX,
            ..Work::default()
        };
        let one = Work {
            gate_evaluations: 1,
            ..Work::default()
        };
        assert_eq!(maxed.checked_add(one), Err(AccountingError::Overflow));
        let idle = Work {
            idle_ticks: 20,
            ..Work::default()
        };
        let weights = Work {
            idle_ticks: 3,
            ..Work::default()
        };
        assert_eq!(idle.proxy_score(weights), Ok(60));
    }
}
