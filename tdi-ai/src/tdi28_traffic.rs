//! TDI-28 Stage-0 declared memory/communication traffic model.
//!
//! A transparent algebraic *reference transfer profile* maps verified logical
//! work counters into declared bytes and message counts. This is NOT a hardware
//! measurement: no DRAM controller, cache coherency, router, NIC or power
//! sensor is sampled. The caller freezes scenario coefficients *before* replay;
//! scoring has no post-hoc weights argument. Real-world attribution requires
//! RemoteOps-controlled independent measurement and calibration.
//!
//! The model intentionally prices cache validation and deferral signaling,
//! so skipping a Boolean gate need not be cheaper under all scenarios.

use super::tdi28_energy_algebra::{AccountingError, Work};
use super::tdi28_four_arm::{
    FourArmReport, FourError, FourProxyScores, MatchedCostModel, MatchedEvent, replay_four,
};

/// One declared, non-calibrated transfer shape for a particular test scenario.
/// Each field specifies reference logical bytes or messages per indicated
/// counter event. Zero disables that hypothetical transfer, for local compute.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TransferProfile {
    /// Nonzero identity of this synthetic profile; not a calibration ID.
    pub revision: u64,
    pub bytes_per_value_read: u64,
    pub bytes_per_revision_read: u64,
    pub bytes_per_cache_probe: u64,
    pub bytes_per_cache_validation: u64,
    pub bytes_per_cache_write: u64,
    pub bytes_per_output_transition: u64,
    pub bytes_per_deferred_queue_write: u64,
    pub bytes_per_dispatch_read: u64,
    pub bytes_per_evaluation_message: u64,
    pub bytes_per_validation_message: u64,
    pub bytes_per_deferral_message: u64,
    pub bytes_per_dispatch_message: u64,
    pub messages_per_evaluation: u64,
    pub messages_per_validation: u64,
    pub messages_per_deferral: u64,
    pub messages_per_dispatch: u64,
}

/// Declared integer cost units per byte/message. These units are not joules.
/// The physical system's cost can be nonlinear and time-dependent.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TransferPrices {
    pub per_read_byte: u64,
    pub per_write_byte: u64,
    pub per_communication_byte: u64,
    pub per_message: u64,
}

/// Frozen-at-admission synthetic scenario for all four reference arms.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrafficContract {
    pub profile: TransferProfile,
    pub prices: TransferPrices,
}

/// Compositional declared logical transfer accounting.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Traffic {
    pub memory_read_bytes: u128,
    pub memory_write_bytes: u128,
    pub communication_bytes: u128,
    pub messages: u128,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrafficError {
    InvalidProfileRevision,
    Overflow,
    SummaryMismatch,
    UnrepresentableUnitWeight,
    Accounting(AccountingError),
    Four(FourError),
}

impl From<AccountingError> for TrafficError {
    fn from(error: AccountingError) -> Self {
        Self::Accounting(error)
    }
}

impl From<FourError> for TrafficError {
    fn from(error: FourError) -> Self {
        Self::Four(error)
    }
}

impl Traffic {
    /// Addition is associative when all intermediate counters fit in u128;
    /// this reference property is not a statement about physical energy.
    pub fn checked_add(self, other: Self) -> Result<Self, TrafficError> {
        Ok(Self {
            memory_read_bytes: self
                .memory_read_bytes
                .checked_add(other.memory_read_bytes)
                .ok_or(TrafficError::Overflow)?,
            memory_write_bytes: self
                .memory_write_bytes
                .checked_add(other.memory_write_bytes)
                .ok_or(TrafficError::Overflow)?,
            communication_bytes: self
                .communication_bytes
                .checked_add(other.communication_bytes)
                .ok_or(TrafficError::Overflow)?,
            messages: self
                .messages
                .checked_add(other.messages)
                .ok_or(TrafficError::Overflow)?,
        })
    }

    /// Integer *proxy* score, with checked arithmetic and no hidden weights.
    pub fn price(self, prices: TransferPrices) -> Result<u128, TrafficError> {
        let terms = [
            (self.memory_read_bytes, prices.per_read_byte),
            (self.memory_write_bytes, prices.per_write_byte),
            (self.communication_bytes, prices.per_communication_byte),
            (self.messages, prices.per_message),
        ];
        terms.into_iter().try_fold(0u128, |total, (count, unit)| {
            total
                .checked_add(
                    count
                        .checked_mul(u128::from(unit))
                        .ok_or(TrafficError::Overflow)?,
                )
                .ok_or(TrafficError::Overflow)
        })
    }
}

fn logical_bytes(parts: &[(u64, u64)]) -> Result<u128, TrafficError> {
    parts.iter().try_fold(0u128, |acc, (count, unit)| {
        acc.checked_add(u128::from(*count) * u128::from(*unit))
            .ok_or(TrafficError::Overflow)
    })
}

/// Map *observed reference counters* to declared transfer quantities.
/// For instance `gate_evaluations` counts output-store events, while
/// `cache_validation_comparisons` accounts optional extra inspections.
/// This does not measure actual cache lines, packets or DRAM transactions.
pub fn declared_traffic(work: Work, profile: TransferProfile) -> Result<Traffic, TrafficError> {
    if profile.revision == 0 {
        return Err(TrafficError::InvalidProfileRevision);
    }

    Ok(Traffic {
        memory_read_bytes: logical_bytes(&[
            (work.input_value_reads, profile.bytes_per_value_read),
            (work.revision_reads, profile.bytes_per_revision_read),
            (work.cache_probes, profile.bytes_per_cache_probe),
            (
                work.cache_validation_comparisons,
                profile.bytes_per_cache_validation,
            ),
            (work.event_decisions, profile.bytes_per_dispatch_read),
        ])?,
        memory_write_bytes: logical_bytes(&[
            (work.gate_evaluations, profile.bytes_per_cache_write),
            (work.output_transitions, profile.bytes_per_output_transition),
            (work.deferred_events, profile.bytes_per_deferred_queue_write),
        ])?,
        communication_bytes: logical_bytes(&[
            (work.gate_evaluations, profile.bytes_per_evaluation_message),
            (
                work.cache_validation_comparisons,
                profile.bytes_per_validation_message,
            ),
            (work.deferred_events, profile.bytes_per_deferral_message),
            (work.event_decisions, profile.bytes_per_dispatch_message),
        ])?,
        messages: logical_bytes(&[
            (work.gate_evaluations, profile.messages_per_evaluation),
            (
                work.cache_validation_comparisons,
                profile.messages_per_validation,
            ),
            (work.deferred_events, profile.messages_per_deferral),
            (work.event_decisions, profile.messages_per_dispatch),
        ])?,
    })
}

/// Whether the prospective C3 budget prices transfers before deciding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrafficBudgetMode {
    /// Legacy Stage-0 reference: price traffic only after policy execution.
    WorkOnly,
    /// Include the declared memory/communication cost in C3 admission.
    WorkPlusTraffic,
}

/// Convert a frozen linear transfer model into per-operation Work coefficients.
/// Each reference counter is evaluated as a unit impulse, so their weighted
/// sum is algebraically identical to Work+Traffic scoring for any nonnegative
/// counter vector, provided all checked arithmetic succeeds. No outcomes
/// or future events enter this compilation.
fn traffic_aware_cost_model(
    base: MatchedCostModel,
    contract: TrafficContract,
) -> Result<MatchedCostModel, TrafficError> {
    let units = [
        Work { gate_evaluations: 1, ..Work::default() },
        Work { input_value_reads: 1, ..Work::default() },
        Work { revision_reads: 1, ..Work::default() },
        Work { cache_probes: 1, ..Work::default() },
        Work { cache_validation_comparisons: 1, ..Work::default() },
        Work { output_transitions: 1, ..Work::default() },
        Work { event_decisions: 1, ..Work::default() },
        Work { deferred_events: 1, ..Work::default() },
        Work { idle_ticks: 1, ..Work::default() },
    ];
    let mut merged = [0u64; 9];
    for (i, unit) in units.into_iter().enumerate() {
        let transfer_cost = declared_traffic(unit, contract.profile)?.price(contract.prices)?;
        let logical_cost = unit.proxy_score(base.weights)?;
        let total = logical_cost
            .checked_add(transfer_cost)
            .ok_or(TrafficError::Overflow)?;
        merged[i] =
            u64::try_from(total).map_err(|_| TrafficError::UnrepresentableUnitWeight)?;
    }
    Ok(MatchedCostModel {
        model_revision: base.model_revision,
        weights: Work {
            gate_evaluations: merged[0],
            input_value_reads: merged[1],
            revision_reads: merged[2],
            cache_probes: merged[3],
            cache_validation_comparisons: merged[4],
            output_transitions: merged[5],
            event_decisions: merged[6],
            deferred_events: merged[7],
            idle_ticks: merged[8],
        },
    })
}

/// Sealed replay result. Neither the transfer shape nor the price table can
/// be changed after the event replay through this API.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrafficReport {
    reference: FourArmReport,
    /// Original work-only coefficients, held separately to prevent
    /// charging transfer costs twice when C3 uses compiled weights.
    base_work_model: MatchedCostModel,
    budget_mode: TrafficBudgetMode,
    contract: TrafficContract,
    totals: [Traffic; 4],
    event_traffic: Vec<[Traffic; 4]>,
}

impl TrafficReport {
    #[must_use]
    pub fn reference(&self) -> &FourArmReport {
        &self.reference
    }

    #[must_use]
    pub const fn budget_mode(&self) -> TrafficBudgetMode {
        self.budget_mode
    }

    #[must_use]
    pub const fn base_work_model(&self) -> MatchedCostModel {
        self.base_work_model
    }

    #[must_use]
    pub const fn contract(&self) -> TrafficContract {
        self.contract
    }

    #[must_use]
    pub const fn totals(&self) -> [Traffic; 4] {
        self.totals
    }

    #[must_use]
    pub fn event_traffic(&self) -> &[[Traffic; 4]] {
        &self.event_traffic
    }

    /// Return the combined Work+Traffic proxy score only if C0/C1/C2/C3
    /// delivered exactly matched per-event outputs with no missing optional
    /// results or pending obligations. Report raw transfers otherwise.
    pub fn strict_total_proxy_scores(&self) -> Result<Option<FourProxyScores>, TrafficError> {
        if !self.reference.strict_quality_parity() {
            return Ok(None);
        }
        // The reference model may already contain compiled traffic costs;
        // always use the original Work weights to avoid double charging.
        let prices = self.contract.prices;
        let work = [
            self.reference.c0_work,
            self.reference.c1_work,
            self.reference.c2_work,
            self.reference.c3_work,
        ];
        let mut costs = [0u128; 4];
        for index in 0..4 {
            costs[index] = work[index].proxy_score(self.base_work_model.weights)?;
        }
        let mut results = [0u128; 4];
        for index in 0..4 {
            results[index] = costs[index]
                .checked_add(self.totals[index].price(prices)?)
                .ok_or(TrafficError::Overflow)?;
        }
        Ok(Some(FourProxyScores {
            c0: results[0],
            c1: results[1],
            c2: results[2],
            c3: results[3],
        }))
    }
}

/// Bind synthetic memory and communication assumptions *before* replaying
/// all four arms. No policy sees future events or completed reference values.
/// This legacy entry point retains the original Work-only C3 selection.
/// The explicit traffic-governed variant below compiles transfer costs
/// *before* invoking the C3 planner, without retraining or future leakage.
pub fn replay_four_with_traffic(
    events: &[MatchedEvent],
    horizon: u64,
    work_model: MatchedCostModel,
    contract: TrafficContract,
) -> Result<TrafficReport, TrafficError> {
    replay_traffic_mode(events, horizon, work_model, contract, TrafficBudgetMode::WorkOnly)
}

/// Compare all arms under identical *combined* per-event budgets. C3
/// prospectively uses the declared memory/communication costs to select
/// EVALUATE, REUSE or DEFER; C0/C1/C2 retain their original fixed policies.
/// A C3 mandatory task that exceeds its combined ceiling is rejected, not
/// converted to an apparent saving. No hardware energy is measured.
pub fn replay_four_traffic_governed(
    events: &[MatchedEvent],
    horizon: u64,
    work_model: MatchedCostModel,
    contract: TrafficContract,
) -> Result<TrafficReport, TrafficError> {
    replay_traffic_mode(
        events,
        horizon,
        work_model,
        contract,
        TrafficBudgetMode::WorkPlusTraffic,
    )
}

fn replay_traffic_mode(
    events: &[MatchedEvent],
    horizon: u64,
    work_model: MatchedCostModel,
    contract: TrafficContract,
    budget_mode: TrafficBudgetMode,
) -> Result<TrafficReport, TrafficError> {
    if contract.profile.revision == 0 {
        return Err(TrafficError::InvalidProfileRevision);
    }
    let selection_model = match budget_mode {
        TrafficBudgetMode::WorkOnly => work_model,
        TrafficBudgetMode::WorkPlusTraffic => traffic_aware_cost_model(work_model, contract)?,
    };
    let reference = replay_four(events, horizon, selection_model)?;
    let works = [
        reference.c0_work,
        reference.c1_work,
        reference.c2_work,
        reference.c3_work,
    ];
    let mut totals = [Traffic::default(); 4];
    for index in 0..4 {
        totals[index] = declared_traffic(works[index], contract.profile)?;
    }

    let mut event_traffic = Vec::with_capacity(reference.events.len());
    let mut composed = [Traffic::default(); 4];
    for event in &reference.events {
        let mut vector = [Traffic::default(); 4];
        for index in 0..4 {
            vector[index] = declared_traffic(event.event_work[index], contract.profile)?;
            composed[index] = composed[index].checked_add(vector[index])?;
        }
        event_traffic.push(vector);
    }

    // Every traffic-bearing reference counter is event-local; idle ticks
    // have a separate Work coefficient, no traffic profile entry.
    if totals != composed {
        return Err(TrafficError::SummaryMismatch);
    }
    Ok(TrafficReport {
        reference,
        base_work_model: work_model,
        budget_mode,
        contract,
        totals,
        event_traffic,
    })
}

#[cfg(test)]
mod tests {
    use super::super::tdi28_energy_algebra::{Demand, Gate, InputRevisions};
    use super::*;

    fn profile() -> TransferProfile {
        TransferProfile {
            revision: 1,
            bytes_per_value_read: 32,
            bytes_per_revision_read: 8,
            bytes_per_cache_probe: 8,
            bytes_per_cache_validation: 8,
            bytes_per_cache_write: 1,
            bytes_per_output_transition: 1,
            bytes_per_deferred_queue_write: 16,
            bytes_per_dispatch_read: 2,
            bytes_per_evaluation_message: 4,
            bytes_per_validation_message: 0,
            bytes_per_deferral_message: 2,
            bytes_per_dispatch_message: 0,
            messages_per_evaluation: 1,
            messages_per_validation: 0,
            messages_per_deferral: 1,
            messages_per_dispatch: 0,
        }
    }

    fn contract() -> TrafficContract {
        TrafficContract {
            profile: profile(),
            prices: TransferPrices {
                per_read_byte: 1,
                per_write_byte: 1,
                per_communication_byte: 1,
                per_message: 1,
            },
        }
    }

    fn model() -> MatchedCostModel {
        MatchedCostModel {
            model_revision: 1,
            weights: Work {
                gate_evaluations: 10,
                input_value_reads: 2,
                revision_reads: 1,
                cache_probes: 1,
                cache_validation_comparisons: 2,
                output_transitions: 1,
                event_decisions: 1,
                deferred_events: 1,
                idle_ticks: 1,
            },
        }
    }

    fn event(tick: u64, version: u64, left: bool, optional: bool) -> MatchedEvent {
        MatchedEvent {
            event: super::super::tdi28_trace::TraceEvent {
                tick,
                gate: Gate::Xor,
                revisions: InputRevisions {
                    left: version,
                    right: 1,
                },
                left,
                right: false,
                demand: if optional {
                    Demand::Optional
                } else {
                    Demand::Required
                },
            },
            optional_deadline: if optional { Some(tick + 3) } else { None },
            event_ceiling: 5000,
        }
    }

    #[test]
    fn same_required_events_keep_identical_c0_c1_transfer_traffic() {
        let inputs = [
            event(0, 1, false, false),
            event(2, 1, false, false),
            event(4, 2, true, false),
        ];
        let report = replay_four_with_traffic(&inputs, 5, model(), contract()).unwrap();
        assert!(report.reference().strict_quality_parity());
        assert_eq!(report.totals()[0], report.totals()[1]);
        assert!(report.totals()[2].memory_read_bytes < report.totals()[0].memory_read_bytes);
        assert_eq!(report.event_traffic().len(), inputs.len());
        let scored = report.strict_total_proxy_scores().unwrap().unwrap();
        assert_eq!(scored.c0, scored.c1);
        assert!(scored.c2 < scored.c0);
        assert!(scored.c3 < scored.c0);
    }

    #[test]
    fn communication_pricing_can_reverse_logic_only_rankings() {
        let inputs = [event(0, 1, false, false), event(1, 1, false, false)];
        let mut transfer = contract();
        transfer.profile.bytes_per_validation_message = 5000;
        transfer.profile.messages_per_validation = 1;
        transfer.prices.per_communication_byte = 10;
        let report = replay_four_with_traffic(&inputs, 1, model(), transfer).unwrap();
        let work = report.reference().strict_proxy_scores().unwrap().unwrap();
        assert!(work.c2 < work.c0);
        let combined = report.strict_total_proxy_scores().unwrap().unwrap();
        assert!(combined.c2 > combined.c0);
        assert!(combined.c3 > combined.c0);
    }

    #[test]
    fn traffic_priced_c3_recomputes_instead_of_excessive_remote_cache_check() {
        let mut inputs = [event(0, 1, false, false), event(1, 1, false, false)];
        for item in &mut inputs {
            item.event_ceiling = 100_000;
        }
        let mut remote = contract();
        remote.profile.bytes_per_validation_message = 5_000;
        remote.profile.messages_per_validation = 1;
        remote.prices.per_communication_byte = 10;
        let legacy = replay_four_with_traffic(&inputs, 1, model(), remote).unwrap();
        let governed = replay_four_traffic_governed(&inputs, 1, model(), remote).unwrap();
        assert_eq!(legacy.budget_mode(), TrafficBudgetMode::WorkOnly);
        assert_eq!(governed.budget_mode(), TrafficBudgetMode::WorkPlusTraffic);
        assert!(legacy.reference().strict_quality_parity());
        assert!(governed.reference().strict_quality_parity());
        assert_eq!(legacy.reference().events[1].c3_decision, super::super::tdi28_energy_algebra::Decision::Reuse);
        assert_eq!(governed.reference().events[1].c3_decision, super::super::tdi28_energy_algebra::Decision::Evaluate);
        let before = legacy.strict_total_proxy_scores().unwrap().unwrap();
        let after = governed.strict_total_proxy_scores().unwrap().unwrap();
        assert_eq!(after.c0, before.c0);
        assert_eq!(after.c1, before.c1);
        assert_eq!(after.c2, before.c2);
        assert!(after.c3 < before.c3);
        assert!(after.c3 < after.c2);
    }

    #[test]
    fn compiled_traffic_coefficients_equal_explicit_addition() {
        let frozen = contract();
        let compiled = traffic_aware_cost_model(model(), frozen).unwrap();
        let observed = Work {
            gate_evaluations: 3,
            input_value_reads: 5,
            revision_reads: 9,
            cache_probes: 2,
            cache_validation_comparisons: 2,
            output_transitions: 1,
            event_decisions: 7,
            deferred_events: 4,
            idle_ticks: 15,
        };
        let explicit = observed
            .proxy_score(model().weights)
            .unwrap()
            .checked_add(declared_traffic(observed, frozen.profile).unwrap().price(frozen.prices).unwrap())
            .unwrap();
        assert_eq!(observed.proxy_score(compiled.weights).unwrap(), explicit);
        assert_eq!(compiled.model_revision, model().model_revision);
    }

    #[test]
    fn required_work_fails_when_combined_memory_cost_exceeds_ceiling() {
        let mut input = event(0, 1, false, false);
        input.event_ceiling = 20;
        // Work-only C3 fits (17 units), combined memory/communication
        // overhead does not. There is no silent required-output deferral.
        assert!(replay_four_with_traffic(&[input], 0, model(), contract()).is_ok());
        assert!(matches!(
            replay_four_traffic_governed(&[input], 0, model(), contract()),
            Err(TrafficError::Four(FourError::C3(
                super::super::tdi28_c3_policy::C3Error::BudgetInsufficient { .. }
            )))
        ));
    }

    #[test]
    fn invalid_traffic_price_fails_closed_before_any_replay() {
        let mut bad = contract();
        bad.profile.bytes_per_evaluation_message = u64::MAX;
        bad.prices.per_communication_byte = u64::MAX;
        assert_eq!(
            replay_four_traffic_governed(&[], 0, model(), bad),
            Err(TrafficError::UnrepresentableUnitWeight)
        );
        bad.profile.revision = 0;
        assert_eq!(
            replay_four_traffic_governed(&[], 0, model(), bad),
            Err(TrafficError::InvalidProfileRevision)
        );
    }

    #[test]
    fn local_no_network_scenario_has_zero_declared_communication() {
        let mut local = profile();
        local.bytes_per_evaluation_message = 0;
        local.bytes_per_validation_message = 0;
        local.bytes_per_deferral_message = 0;
        local.bytes_per_dispatch_message = 0;
        local.messages_per_evaluation = 0;
        local.messages_per_validation = 0;
        local.messages_per_deferral = 0;
        local.messages_per_dispatch = 0;
        let t = declared_traffic(
            Work {
                gate_evaluations: 2,
                cache_validation_comparisons: 3,
                deferred_events: 1,
                ..Work::default()
            },
            local,
        )
        .unwrap();
        assert_eq!(t.communication_bytes, 0);
        assert_eq!(t.messages, 0);
    }

    #[test]
    fn uncovered_optional_observation_blocks_aggregate_cost_claim() {
        let inputs = [event(0, 1, false, true), event(2, 1, false, false)];
        // Work ceiling low enough to defer first, high enough for second;
        // event-level budgets are encoded by the shared matched events.
        let mut inputs = inputs;
        inputs[0].event_ceiling = 8;
        let report = replay_four_with_traffic(&inputs, 4, model(), contract()).unwrap();
        assert!(!report.reference().strict_quality_parity());
        assert_eq!(report.strict_total_proxy_scores().unwrap(), None);
        assert!(report.totals()[0].memory_read_bytes > 0);
    }

    #[test]
    fn transfer_accumulation_associates_and_rejects_overflow() {
        let a = Traffic {
            memory_read_bytes: 1,
            memory_write_bytes: 2,
            communication_bytes: 3,
            messages: 4,
        };
        let b = Traffic {
            memory_read_bytes: 5,
            memory_write_bytes: 6,
            communication_bytes: 7,
            messages: 8,
        };
        let c = Traffic {
            memory_read_bytes: 9,
            memory_write_bytes: 10,
            communication_bytes: 11,
            messages: 12,
        };
        assert_eq!(
            a.checked_add(b).unwrap().checked_add(c),
            a.checked_add(b.checked_add(c).unwrap())
        );
        assert_eq!(
            a.checked_add(Traffic {
                memory_read_bytes: u128::MAX,
                ..Traffic::default()
            }),
            Err(TrafficError::Overflow)
        );
    }

    #[test]
    fn zero_revision_and_unrepresentable_prices_fail_closed() {
        let mut invalid = profile();
        invalid.revision = 0;
        assert_eq!(
            declared_traffic(Work::default(), invalid),
            Err(TrafficError::InvalidProfileRevision)
        );
        let huge = Traffic {
            memory_read_bytes: u128::MAX,
            ..Traffic::default()
        };
        assert_eq!(
            huge.price(TransferPrices {
                per_read_byte: 2,
                ..TransferPrices::default()
            }),
            Err(TrafficError::Overflow)
        );
        let huge_work = Work {
            input_value_reads: u64::MAX,
            cache_probes: u64::MAX,
            ..Work::default()
        };
        let overloaded = TransferProfile {
            revision: 1,
            bytes_per_value_read: u64::MAX,
            bytes_per_cache_probe: u64::MAX,
            ..TransferProfile::default()
        };
        assert_eq!(
            declared_traffic(huge_work, overloaded),
            Err(TrafficError::Overflow)
        );
    }

    #[test]
    fn network_and_memory_are_independent_dimensions() {
        let work = Work {
            gate_evaluations: 1,
            cache_validation_comparisons: 1,
            event_decisions: 1,
            ..Work::default()
        };
        let t = declared_traffic(work, profile()).unwrap();
        assert_eq!(t.memory_write_bytes, 1);
        assert_eq!(t.communication_bytes, 4);
        assert_eq!(t.messages, 1);
    }
}
