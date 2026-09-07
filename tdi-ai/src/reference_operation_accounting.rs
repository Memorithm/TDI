//! Exact semantic operation accounting for bounded TDI-8.1 reference arms.
//!
//! Counts in this module are deterministic reference-semantic work units. They
//! are deliberately not CPU instructions, FLOPs, latency, energy, bandwidth,
//! GPU occupancy, or hardware-performance claims. Validation, allocation and
//! host-container bookkeeping are outside this architecture-semantic count.

use core::fmt;

use crate::assr_h_reference::A3VsaReadRoute;
use crate::assr_reference::{A2ReadStatus, A2StepReport, RecurrentLayout};
use crate::full_history_reference::FullHistoryLayout;

/// Fail-closed errors while deriving exact TDI-8.1 semantic operation counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferenceOperationAccountingError {
    /// A checked integer addition or multiplication exceeded `u128`.
    Overflow,
    /// A0 read accounting requires a successful, therefore non-empty, history.
    ZeroHistoryItemsForRead,
}

impl fmt::Display for ReferenceOperationAccountingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Overflow => formatter.write_str("TDI-8.1 semantic operation accounting overflow"),
            Self::ZeroHistoryItemsForRead => {
                formatter.write_str("A0 read operation accounting requires non-empty history")
            }
        }
    }
}

impl std::error::Error for ReferenceOperationAccountingError {}

/// Component-wise deterministic semantic work for one reference action.
///
/// Each field counts one explicitly declared scalar/reference event rather than
/// attempting to infer compiler, ISA, or accelerator instruction counts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReferenceOperationAccounting {
    recurrent_mac_terms: u128,
    activation_terms: u128,
    associative_address_projections: u128,
    associative_payload_fusions: u128,
    associative_payload_stores: u128,
    vsa_bind_terms: u128,
    vsa_bundle_terms: u128,
    vsa_unbind_terms: u128,
    vsa_input_fusions: u128,
    history_distance_terms: u128,
    history_selection_comparisons: u128,
    history_scalar_stores: u128,
}

impl ReferenceOperationAccounting {
    /// Zero semantic work.
    #[must_use]
    pub const fn zero() -> Self {
        Self {
            recurrent_mac_terms: 0,
            activation_terms: 0,
            associative_address_projections: 0,
            associative_payload_fusions: 0,
            associative_payload_stores: 0,
            vsa_bind_terms: 0,
            vsa_bundle_terms: 0,
            vsa_unbind_terms: 0,
            vsa_input_fusions: 0,
            history_distance_terms: 0,
            history_selection_comparisons: 0,
            history_scalar_stores: 0,
        }
    }

    /// Exact A0 append work: one semantic store for each key/value coordinate.
    pub fn a0_append(layout: FullHistoryLayout) -> Result<Self, ReferenceOperationAccountingError> {
        let stores = checked_add(
            u128::from(layout.key_width()),
            u128::from(layout.value_width()),
        )?;
        Ok(Self {
            history_scalar_stores: stores,
            ..Self::zero()
        })
    }

    /// Exact A0 successful hard-content read work.
    ///
    /// The merged reference evaluates one squared-distance coordinate term for
    /// every `(history item, key coordinate)` pair and one `<=` selection
    /// comparison per history item. Output allocation/copy bookkeeping is not
    /// counted as architecture-semantic compute.
    pub fn a0_read(
        layout: FullHistoryLayout,
        history_items: u64,
    ) -> Result<Self, ReferenceOperationAccountingError> {
        if history_items == 0 {
            return Err(ReferenceOperationAccountingError::ZeroHistoryItemsForRead);
        }
        let distance_terms =
            checked_mul(u128::from(history_items), u128::from(layout.key_width()))?;
        Ok(Self {
            history_distance_terms: distance_terms,
            history_selection_comparisons: u128::from(history_items),
            ..Self::zero()
        })
    }

    /// Exact A1 recurrent step work from the merged fixed-order recurrence.
    pub fn a1_step(layout: RecurrentLayout) -> Result<Self, ReferenceOperationAccountingError> {
        let state_width = u128::from(layout.state_width());
        let row_terms = checked_add(state_width, u128::from(layout.input_width()))?;
        let recurrent_mac_terms = checked_mul(state_width, row_terms)?;
        Ok(Self {
            recurrent_mac_terms,
            activation_terms: state_width,
            ..Self::zero()
        })
    }

    /// Exact A2 semantic work from the observed merged `A2StepReport`.
    ///
    /// A read always performs one address projection. A hit performs one
    /// coordinate-wise associative fusion plus one additional hard-tanh per
    /// recurrent-state coordinate. An admitted write performs a second address
    /// projection and stores one payload scalar per state coordinate.
    pub fn a2_step(
        layout: RecurrentLayout,
        report: A2StepReport,
    ) -> Result<Self, ReferenceOperationAccountingError> {
        let mut accounting = Self::a1_step(layout)?;
        accounting.associative_address_projections = 1;
        let state_width = u128::from(layout.state_width());

        if matches!(report.read(), A2ReadStatus::Hit { .. }) {
            accounting.associative_payload_fusions = state_width;
            accounting.activation_terms = checked_add(accounting.activation_terms, state_width)?;
        }
        if report.write().is_some() {
            accounting.associative_address_projections =
                checked_add(accounting.associative_address_projections, 1)?;
            accounting.associative_payload_stores = state_width;
        }
        Ok(accounting)
    }

    /// Exact A3 routed-step work for the merged read/fuse/A2 transition.
    ///
    /// `Skip` adds no VSA work. `Key` performs one unbind scalar term and one
    /// VSA-to-input fusion per recurrent input coordinate before unchanged A2.
    pub fn a3_routed_step(
        layout: RecurrentLayout,
        route: A3VsaReadRoute,
        report: A2StepReport,
    ) -> Result<Self, ReferenceOperationAccountingError> {
        let mut accounting = Self::a2_step(layout, report)?;
        if matches!(route, A3VsaReadRoute::Key(_)) {
            let width = u128::from(layout.input_width());
            accounting.vsa_unbind_terms = width;
            accounting.vsa_input_fusions = width;
        }
        Ok(accounting)
    }

    /// Exact A3 atomic skip-and-store work used by the qualified A3 adapter.
    ///
    /// VSA preparation performs one bipolar bind and one bundle addition per
    /// VSA/input coordinate before unchanged A2 executes. The prepared VSA
    /// commit contains no numeric operation and contributes zero semantic units.
    pub fn a3_skip_and_store_step(
        layout: RecurrentLayout,
        report: A2StepReport,
    ) -> Result<Self, ReferenceOperationAccountingError> {
        let mut accounting = Self::a2_step(layout, report)?;
        let width = u128::from(layout.input_width());
        accounting.vsa_bind_terms = width;
        accounting.vsa_bundle_terms = width;
        Ok(accounting)
    }

    /// Checked component-wise accumulation across task events.
    pub fn checked_add(self, other: Self) -> Result<Self, ReferenceOperationAccountingError> {
        Ok(Self {
            recurrent_mac_terms: checked_add(self.recurrent_mac_terms, other.recurrent_mac_terms)?,
            activation_terms: checked_add(self.activation_terms, other.activation_terms)?,
            associative_address_projections: checked_add(
                self.associative_address_projections,
                other.associative_address_projections,
            )?,
            associative_payload_fusions: checked_add(
                self.associative_payload_fusions,
                other.associative_payload_fusions,
            )?,
            associative_payload_stores: checked_add(
                self.associative_payload_stores,
                other.associative_payload_stores,
            )?,
            vsa_bind_terms: checked_add(self.vsa_bind_terms, other.vsa_bind_terms)?,
            vsa_bundle_terms: checked_add(self.vsa_bundle_terms, other.vsa_bundle_terms)?,
            vsa_unbind_terms: checked_add(self.vsa_unbind_terms, other.vsa_unbind_terms)?,
            vsa_input_fusions: checked_add(self.vsa_input_fusions, other.vsa_input_fusions)?,
            history_distance_terms: checked_add(
                self.history_distance_terms,
                other.history_distance_terms,
            )?,
            history_selection_comparisons: checked_add(
                self.history_selection_comparisons,
                other.history_selection_comparisons,
            )?,
            history_scalar_stores: checked_add(
                self.history_scalar_stores,
                other.history_scalar_stores,
            )?,
        })
    }

    /// Checked total across all declared semantic-operation classes.
    pub fn total(self) -> Result<u128, ReferenceOperationAccountingError> {
        let mut total = 0u128;
        for component in [
            self.recurrent_mac_terms,
            self.activation_terms,
            self.associative_address_projections,
            self.associative_payload_fusions,
            self.associative_payload_stores,
            self.vsa_bind_terms,
            self.vsa_bundle_terms,
            self.vsa_unbind_terms,
            self.vsa_input_fusions,
            self.history_distance_terms,
            self.history_selection_comparisons,
            self.history_scalar_stores,
        ] {
            total = checked_add(total, component)?;
        }
        Ok(total)
    }

    #[must_use]
    pub const fn recurrent_mac_terms(self) -> u128 {
        self.recurrent_mac_terms
    }
    #[must_use]
    pub const fn activation_terms(self) -> u128 {
        self.activation_terms
    }
    #[must_use]
    pub const fn associative_address_projections(self) -> u128 {
        self.associative_address_projections
    }
    #[must_use]
    pub const fn associative_payload_fusions(self) -> u128 {
        self.associative_payload_fusions
    }
    #[must_use]
    pub const fn associative_payload_stores(self) -> u128 {
        self.associative_payload_stores
    }
    #[must_use]
    pub const fn vsa_bind_terms(self) -> u128 {
        self.vsa_bind_terms
    }
    #[must_use]
    pub const fn vsa_bundle_terms(self) -> u128 {
        self.vsa_bundle_terms
    }
    #[must_use]
    pub const fn vsa_unbind_terms(self) -> u128 {
        self.vsa_unbind_terms
    }
    #[must_use]
    pub const fn vsa_input_fusions(self) -> u128 {
        self.vsa_input_fusions
    }
    #[must_use]
    pub const fn history_distance_terms(self) -> u128 {
        self.history_distance_terms
    }
    #[must_use]
    pub const fn history_selection_comparisons(self) -> u128 {
        self.history_selection_comparisons
    }
    #[must_use]
    pub const fn history_scalar_stores(self) -> u128 {
        self.history_scalar_stores
    }
}

fn checked_add(left: u128, right: u128) -> Result<u128, ReferenceOperationAccountingError> {
    left.checked_add(right)
        .ok_or(ReferenceOperationAccountingError::Overflow)
}

fn checked_mul(left: u128, right: u128) -> Result<u128, ReferenceOperationAccountingError> {
    left.checked_mul(right)
        .ok_or(ReferenceOperationAccountingError::Overflow)
}

#[cfg(test)]
mod tests {
    use super::{ReferenceOperationAccounting, ReferenceOperationAccountingError};
    use crate::associative_memory::AssociativeMemoryLayout;
    use crate::assr_h_reference::A3VsaReadRoute;
    use crate::assr_reference::{A2Reference, RecurrentLayout, RecurrentParameters};
    use crate::full_history_reference::FullHistoryLayout;

    fn recurrent_parameters() -> RecurrentParameters {
        let layout = RecurrentLayout::new(2, 2).expect("layout");
        RecurrentParameters::new(layout, vec![1.0, 0.0, 0.0, 1.0], vec![0.0; 4], vec![0.0; 2])
            .expect("parameters")
    }

    fn hit_and_write_report() -> (RecurrentLayout, crate::assr_reference::A2StepReport) {
        let parameters = recurrent_parameters();
        let layout = parameters.layout();
        let memory_layout = AssociativeMemoryLayout::new(8, 2).expect("memory layout");
        let mut a2 = A2Reference::new(parameters, memory_layout, 7, 1.0).expect("A2");
        a2.step(&[0.25, -0.25], 99, Some(17)).expect("store");
        let report = a2.step(&[0.0, 0.0], 17, Some(17)).expect("hit/update");
        (layout, report)
    }

    #[test]
    fn a1_formula_matches_fixed_order_recurrent_loops() {
        let layout = RecurrentLayout::new(3, 2).expect("layout");
        let count = ReferenceOperationAccounting::a1_step(layout).expect("count");
        assert_eq!(count.recurrent_mac_terms(), 10);
        assert_eq!(count.activation_terms(), 2);
        assert_eq!(count.total(), Ok(12));
    }

    #[test]
    fn a2_hit_and_write_adds_only_observed_work() {
        let (layout, report) = hit_and_write_report();
        let count = ReferenceOperationAccounting::a2_step(layout, report).expect("count");
        assert_eq!(count.recurrent_mac_terms(), 8);
        assert_eq!(count.activation_terms(), 4);
        assert_eq!(count.associative_address_projections(), 2);
        assert_eq!(count.associative_payload_fusions(), 2);
        assert_eq!(count.associative_payload_stores(), 2);
        assert_eq!(count.total(), Ok(18));
    }

    #[test]
    fn a3_keyed_read_and_atomic_store_have_distinct_vsa_work() {
        let (layout, report) = hit_and_write_report();
        let keyed =
            ReferenceOperationAccounting::a3_routed_step(layout, A3VsaReadRoute::Key(17), report)
                .expect("keyed count");
        assert_eq!(keyed.vsa_unbind_terms(), 2);
        assert_eq!(keyed.vsa_input_fusions(), 2);
        assert_eq!(keyed.vsa_bind_terms(), 0);
        assert_eq!(keyed.vsa_bundle_terms(), 0);

        let stored = ReferenceOperationAccounting::a3_skip_and_store_step(layout, report)
            .expect("store count");
        assert_eq!(stored.vsa_unbind_terms(), 0);
        assert_eq!(stored.vsa_input_fusions(), 0);
        assert_eq!(stored.vsa_bind_terms(), 2);
        assert_eq!(stored.vsa_bundle_terms(), 2);
    }

    #[test]
    fn a0_counts_scale_exactly_with_history_and_width() {
        let layout = FullHistoryLayout::new(3, 2).expect("layout");
        let append = ReferenceOperationAccounting::a0_append(layout).expect("append");
        assert_eq!(append.history_scalar_stores(), 5);
        assert_eq!(append.total(), Ok(5));

        let read = ReferenceOperationAccounting::a0_read(layout, 4).expect("read");
        assert_eq!(read.history_distance_terms(), 12);
        assert_eq!(read.history_selection_comparisons(), 4);
        assert_eq!(read.total(), Ok(16));
        assert_eq!(
            ReferenceOperationAccounting::a0_read(layout, 0),
            Err(ReferenceOperationAccountingError::ZeroHistoryItemsForRead)
        );
    }

    #[test]
    fn aggregation_fails_closed_on_u128_overflow() {
        let maxed = ReferenceOperationAccounting {
            history_scalar_stores: u128::MAX,
            ..ReferenceOperationAccounting::zero()
        };
        let one = ReferenceOperationAccounting {
            history_scalar_stores: 1,
            ..ReferenceOperationAccounting::zero()
        };
        assert_eq!(
            maxed.checked_add(one),
            Err(ReferenceOperationAccountingError::Overflow)
        );
        assert_eq!(maxed.total(), Ok(u128::MAX));
    }
}
