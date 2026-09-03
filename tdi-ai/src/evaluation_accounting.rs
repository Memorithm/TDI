//! Exact operation-accounting and provenance foundation for bounded TDI-8.1.
//!
//! This module deliberately separates reference-mechanism work from
//! evaluator/protocol work. It provides exact checked counters and a canonical
//! machine-readable provenance record, but it does not assign A0/A1/A2/A3
//! formulas, convert counts into runtime/FLOP/s/energy claims, freeze a final
//! rejection taxonomy, or create any TDI-8.2 surface.

use core::fmt;

use crate::task_generators::TaskFamily;
use crate::{MemoryAccounting, MemoryAccountingError, ReferenceArm};

/// Exact operation quantity independent of host pointer width.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OperationCount(u128);

impl OperationCount {
    /// Exact zero-operation quantity.
    pub const ZERO: Self = Self(0);

    /// Construct an exact operation count.
    #[must_use]
    pub const fn new(count: u128) -> Self {
        Self(count)
    }

    /// Raw exact count.
    #[must_use]
    pub const fn get(self) -> u128 {
        self.0
    }
}

impl From<u64> for OperationCount {
    fn from(value: u64) -> Self {
        Self(u128::from(value))
    }
}

/// Accounting scope kept separate in all TDI-8.1 records.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OperationScope {
    /// Operations belonging to A0/A1/A2/A3 reference semantics.
    ReferenceMechanism,
    /// Encoding, key derivation and evaluator bookkeeping outside arm semantics.
    EvaluatorProtocol,
}

impl OperationScope {
    /// Stable machine-readable label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::ReferenceMechanism => "reference_mechanism",
            Self::EvaluatorProtocol => "evaluator_protocol",
        }
    }
}

/// Named exact counter axis.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum OperationComponent {
    Binary64Additions,
    Binary64Subtractions,
    Binary64Multiplications,
    Binary64Divisions,
    Binary64Comparisons,
    NonlinearClampApplications,
    AddressProjections,
    AssociativeReads,
    AssociativeWrites,
    HistoryItemScores,
    VsaBindCoordinates,
    VsaBundleCoordinates,
    VsaUnbindCoordinates,
    VsaSimilarityCoordinates,
    EncodedBinary64Coordinates,
    DecodedBinary64Coordinates,
    LogicalKeyDerivations,
    BookkeepingEvents,
    ProvenanceFieldsEmitted,
}

impl OperationComponent {
    /// Stable machine-readable component label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Binary64Additions => "binary64_additions",
            Self::Binary64Subtractions => "binary64_subtractions",
            Self::Binary64Multiplications => "binary64_multiplications",
            Self::Binary64Divisions => "binary64_divisions",
            Self::Binary64Comparisons => "binary64_comparisons",
            Self::NonlinearClampApplications => "nonlinear_clamp_applications",
            Self::AddressProjections => "address_projections",
            Self::AssociativeReads => "associative_reads",
            Self::AssociativeWrites => "associative_writes",
            Self::HistoryItemScores => "history_item_scores",
            Self::VsaBindCoordinates => "vsa_bind_coordinates",
            Self::VsaBundleCoordinates => "vsa_bundle_coordinates",
            Self::VsaUnbindCoordinates => "vsa_unbind_coordinates",
            Self::VsaSimilarityCoordinates => "vsa_similarity_coordinates",
            Self::EncodedBinary64Coordinates => "encoded_binary64_coordinates",
            Self::DecodedBinary64Coordinates => "decoded_binary64_coordinates",
            Self::LogicalKeyDerivations => "logical_key_derivations",
            Self::BookkeepingEvents => "bookkeeping_events",
            Self::ProvenanceFieldsEmitted => "provenance_fields_emitted",
        }
    }
}

/// Fail-closed operation-accounting errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperationAccountingError {
    /// Component-wise aggregation overflowed exact `u128` accounting.
    Overflow {
        scope: OperationScope,
        component: OperationComponent,
    },
}

impl fmt::Display for OperationAccountingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Overflow { scope, component } => write!(
                formatter,
                "TDI-8 operation accounting overflow in {}:{}",
                scope.label(),
                component.label()
            ),
        }
    }
}

impl std::error::Error for OperationAccountingError {}

fn checked_component_add(
    left: OperationCount,
    right: OperationCount,
    scope: OperationScope,
    component: OperationComponent,
) -> Result<OperationCount, OperationAccountingError> {
    left.get()
        .checked_add(right.get())
        .map(OperationCount::new)
        .ok_or(OperationAccountingError::Overflow { scope, component })
}

/// Primitive binary64 arithmetic/comparison counts inside reference semantics.
///
/// These axes are exact source-semantic counters. They are not weighted, and no
/// method converts them to runtime, FLOP/s, energy or a synthetic scalar cost.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReferenceArithmeticCounts {
    additions: OperationCount,
    subtractions: OperationCount,
    multiplications: OperationCount,
    divisions: OperationCount,
    comparisons: OperationCount,
    nonlinear_clamps: OperationCount,
}

impl ReferenceArithmeticCounts {
    #[must_use]
    pub const fn new(
        additions: u128,
        subtractions: u128,
        multiplications: u128,
        divisions: u128,
        comparisons: u128,
        nonlinear_clamps: u128,
    ) -> Self {
        Self {
            additions: OperationCount::new(additions),
            subtractions: OperationCount::new(subtractions),
            multiplications: OperationCount::new(multiplications),
            divisions: OperationCount::new(divisions),
            comparisons: OperationCount::new(comparisons),
            nonlinear_clamps: OperationCount::new(nonlinear_clamps),
        }
    }

    #[must_use]
    pub const fn additions(self) -> OperationCount {
        self.additions
    }

    #[must_use]
    pub const fn subtractions(self) -> OperationCount {
        self.subtractions
    }

    #[must_use]
    pub const fn multiplications(self) -> OperationCount {
        self.multiplications
    }

    #[must_use]
    pub const fn divisions(self) -> OperationCount {
        self.divisions
    }

    #[must_use]
    pub const fn comparisons(self) -> OperationCount {
        self.comparisons
    }

    #[must_use]
    pub const fn nonlinear_clamps(self) -> OperationCount {
        self.nonlinear_clamps
    }

    pub fn checked_add(self, other: Self) -> Result<Self, OperationAccountingError> {
        let scope = OperationScope::ReferenceMechanism;
        Ok(Self {
            additions: checked_component_add(
                self.additions,
                other.additions,
                scope,
                OperationComponent::Binary64Additions,
            )?,
            subtractions: checked_component_add(
                self.subtractions,
                other.subtractions,
                scope,
                OperationComponent::Binary64Subtractions,
            )?,
            multiplications: checked_component_add(
                self.multiplications,
                other.multiplications,
                scope,
                OperationComponent::Binary64Multiplications,
            )?,
            divisions: checked_component_add(
                self.divisions,
                other.divisions,
                scope,
                OperationComponent::Binary64Divisions,
            )?,
            comparisons: checked_component_add(
                self.comparisons,
                other.comparisons,
                scope,
                OperationComponent::Binary64Comparisons,
            )?,
            nonlinear_clamps: checked_component_add(
                self.nonlinear_clamps,
                other.nonlinear_clamps,
                scope,
                OperationComponent::NonlinearClampApplications,
            )?,
        })
    }
}

/// Mechanism-semantic event counters kept distinct from primitive arithmetic.
///
/// These counters intentionally overlap conceptually with arithmetic execution
/// (for example one VSA coordinate operation may contain multiplication). They
/// therefore must never be summed with arithmetic axes into a pseudo-total.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReferenceMechanismEventCounts {
    address_projections: OperationCount,
    associative_reads: OperationCount,
    associative_writes: OperationCount,
    history_item_scores: OperationCount,
    vsa_bind_coordinates: OperationCount,
    vsa_bundle_coordinates: OperationCount,
    vsa_unbind_coordinates: OperationCount,
    vsa_similarity_coordinates: OperationCount,
}

impl ReferenceMechanismEventCounts {
    #[must_use]
    pub const fn new(
        address_projections: u128,
        associative_reads: u128,
        associative_writes: u128,
        history_item_scores: u128,
        vsa_bind_coordinates: u128,
        vsa_bundle_coordinates: u128,
        vsa_unbind_coordinates: u128,
        vsa_similarity_coordinates: u128,
    ) -> Self {
        Self {
            address_projections: OperationCount::new(address_projections),
            associative_reads: OperationCount::new(associative_reads),
            associative_writes: OperationCount::new(associative_writes),
            history_item_scores: OperationCount::new(history_item_scores),
            vsa_bind_coordinates: OperationCount::new(vsa_bind_coordinates),
            vsa_bundle_coordinates: OperationCount::new(vsa_bundle_coordinates),
            vsa_unbind_coordinates: OperationCount::new(vsa_unbind_coordinates),
            vsa_similarity_coordinates: OperationCount::new(vsa_similarity_coordinates),
        }
    }

    #[must_use]
    pub const fn address_projections(self) -> OperationCount {
        self.address_projections
    }

    #[must_use]
    pub const fn associative_reads(self) -> OperationCount {
        self.associative_reads
    }

    #[must_use]
    pub const fn associative_writes(self) -> OperationCount {
        self.associative_writes
    }

    #[must_use]
    pub const fn history_item_scores(self) -> OperationCount {
        self.history_item_scores
    }

    #[must_use]
    pub const fn vsa_bind_coordinates(self) -> OperationCount {
        self.vsa_bind_coordinates
    }

    #[must_use]
    pub const fn vsa_bundle_coordinates(self) -> OperationCount {
        self.vsa_bundle_coordinates
    }

    #[must_use]
    pub const fn vsa_unbind_coordinates(self) -> OperationCount {
        self.vsa_unbind_coordinates
    }

    #[must_use]
    pub const fn vsa_similarity_coordinates(self) -> OperationCount {
        self.vsa_similarity_coordinates
    }

    pub fn checked_add(self, other: Self) -> Result<Self, OperationAccountingError> {
        let scope = OperationScope::ReferenceMechanism;
        Ok(Self {
            address_projections: checked_component_add(
                self.address_projections,
                other.address_projections,
                scope,
                OperationComponent::AddressProjections,
            )?,
            associative_reads: checked_component_add(
                self.associative_reads,
                other.associative_reads,
                scope,
                OperationComponent::AssociativeReads,
            )?,
            associative_writes: checked_component_add(
                self.associative_writes,
                other.associative_writes,
                scope,
                OperationComponent::AssociativeWrites,
            )?,
            history_item_scores: checked_component_add(
                self.history_item_scores,
                other.history_item_scores,
                scope,
                OperationComponent::HistoryItemScores,
            )?,
            vsa_bind_coordinates: checked_component_add(
                self.vsa_bind_coordinates,
                other.vsa_bind_coordinates,
                scope,
                OperationComponent::VsaBindCoordinates,
            )?,
            vsa_bundle_coordinates: checked_component_add(
                self.vsa_bundle_coordinates,
                other.vsa_bundle_coordinates,
                scope,
                OperationComponent::VsaBundleCoordinates,
            )?,
            vsa_unbind_coordinates: checked_component_add(
                self.vsa_unbind_coordinates,
                other.vsa_unbind_coordinates,
                scope,
                OperationComponent::VsaUnbindCoordinates,
            )?,
            vsa_similarity_coordinates: checked_component_add(
                self.vsa_similarity_coordinates,
                other.vsa_similarity_coordinates,
                scope,
                OperationComponent::VsaSimilarityCoordinates,
            )?,
        })
    }
}

/// Exact reference-mechanism operation record.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReferenceOperationAccounting {
    arithmetic: ReferenceArithmeticCounts,
    mechanism_events: ReferenceMechanismEventCounts,
}

impl ReferenceOperationAccounting {
    #[must_use]
    pub const fn new(
        arithmetic: ReferenceArithmeticCounts,
        mechanism_events: ReferenceMechanismEventCounts,
    ) -> Self {
        Self {
            arithmetic,
            mechanism_events,
        }
    }

    #[must_use]
    pub const fn arithmetic(self) -> ReferenceArithmeticCounts {
        self.arithmetic
    }

    #[must_use]
    pub const fn mechanism_events(self) -> ReferenceMechanismEventCounts {
        self.mechanism_events
    }

    pub fn checked_add(self, other: Self) -> Result<Self, OperationAccountingError> {
        Ok(Self {
            arithmetic: self.arithmetic.checked_add(other.arithmetic)?,
            mechanism_events: self.mechanism_events.checked_add(other.mechanism_events)?,
        })
    }
}

/// Exact evaluator/protocol counters that are never charged to arm semantics.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EvaluatorProtocolCounts {
    encoded_binary64_coordinates: OperationCount,
    decoded_binary64_coordinates: OperationCount,
    logical_key_derivations: OperationCount,
    bookkeeping_events: OperationCount,
    provenance_fields_emitted: OperationCount,
}

impl EvaluatorProtocolCounts {
    #[must_use]
    pub const fn new(
        encoded_binary64_coordinates: u128,
        decoded_binary64_coordinates: u128,
        logical_key_derivations: u128,
        bookkeeping_events: u128,
        provenance_fields_emitted: u128,
    ) -> Self {
        Self {
            encoded_binary64_coordinates: OperationCount::new(encoded_binary64_coordinates),
            decoded_binary64_coordinates: OperationCount::new(decoded_binary64_coordinates),
            logical_key_derivations: OperationCount::new(logical_key_derivations),
            bookkeeping_events: OperationCount::new(bookkeeping_events),
            provenance_fields_emitted: OperationCount::new(provenance_fields_emitted),
        }
    }

    #[must_use]
    pub const fn encoded_binary64_coordinates(self) -> OperationCount {
        self.encoded_binary64_coordinates
    }

    #[must_use]
    pub const fn decoded_binary64_coordinates(self) -> OperationCount {
        self.decoded_binary64_coordinates
    }

    #[must_use]
    pub const fn logical_key_derivations(self) -> OperationCount {
        self.logical_key_derivations
    }

    #[must_use]
    pub const fn bookkeeping_events(self) -> OperationCount {
        self.bookkeeping_events
    }

    #[must_use]
    pub const fn provenance_fields_emitted(self) -> OperationCount {
        self.provenance_fields_emitted
    }

    pub fn checked_add(self, other: Self) -> Result<Self, OperationAccountingError> {
        let scope = OperationScope::EvaluatorProtocol;
        Ok(Self {
            encoded_binary64_coordinates: checked_component_add(
                self.encoded_binary64_coordinates,
                other.encoded_binary64_coordinates,
                scope,
                OperationComponent::EncodedBinary64Coordinates,
            )?,
            decoded_binary64_coordinates: checked_component_add(
                self.decoded_binary64_coordinates,
                other.decoded_binary64_coordinates,
                scope,
                OperationComponent::DecodedBinary64Coordinates,
            )?,
            logical_key_derivations: checked_component_add(
                self.logical_key_derivations,
                other.logical_key_derivations,
                scope,
                OperationComponent::LogicalKeyDerivations,
            )?,
            bookkeeping_events: checked_component_add(
                self.bookkeeping_events,
                other.bookkeeping_events,
                scope,
                OperationComponent::BookkeepingEvents,
            )?,
            provenance_fields_emitted: checked_component_add(
                self.provenance_fields_emitted,
                other.provenance_fields_emitted,
                scope,
                OperationComponent::ProvenanceFieldsEmitted,
            )?,
        })
    }
}

/// Two-scope operation accounting. No cross-scope scalar total is defined.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EvaluationOperationAccounting {
    reference: ReferenceOperationAccounting,
    evaluator: EvaluatorProtocolCounts,
}

impl EvaluationOperationAccounting {
    #[must_use]
    pub const fn new(
        reference: ReferenceOperationAccounting,
        evaluator: EvaluatorProtocolCounts,
    ) -> Self {
        Self {
            reference,
            evaluator,
        }
    }

    #[must_use]
    pub const fn reference(self) -> ReferenceOperationAccounting {
        self.reference
    }

    #[must_use]
    pub const fn evaluator(self) -> EvaluatorProtocolCounts {
        self.evaluator
    }

    pub fn checked_add(self, other: Self) -> Result<Self, OperationAccountingError> {
        Ok(Self {
            reference: self.reference.checked_add(other.reference)?,
            evaluator: self.evaluator.checked_add(other.evaluator)?,
        })
    }
}

/// Validation errors for one machine-readable bounded evaluation record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvaluationProvenanceError {
    ZeroEventCount,
    ZeroQueryCount,
    QueryCountExceedsEvents { event_count: u64, query_count: u64 },
    MemoryAccounting(MemoryAccountingError),
}

impl fmt::Display for EvaluationProvenanceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroEventCount => formatter.write_str("TDI-8 provenance requires processed events"),
            Self::ZeroQueryCount => formatter.write_str("TDI-8 provenance requires at least one query"),
            Self::QueryCountExceedsEvents {
                event_count,
                query_count,
            } => write!(
                formatter,
                "TDI-8 provenance query count {query_count} exceeds event count {event_count}"
            ),
            Self::MemoryAccounting(error) => write!(formatter, "memory accounting: {error}"),
        }
    }
}

impl std::error::Error for EvaluationProvenanceError {}

impl From<MemoryAccountingError> for EvaluationProvenanceError {
    fn from(error: MemoryAccountingError) -> Self {
        Self::MemoryAccounting(error)
    }
}

/// Bounded, deterministic provenance envelope for one generator/arm execution.
///
/// Operation values are totals across `event_count` processed task events. A
/// per-item value is therefore represented exactly as the rational
/// `component_count / event_count`; no floating-point normalization is performed
/// by this foundation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EvaluationProvenanceRecord {
    arm: ReferenceArm,
    family: TaskFamily,
    generator_seed: u64,
    event_count: u64,
    query_count: u64,
    memory: MemoryAccounting,
    operations: EvaluationOperationAccounting,
}

impl EvaluationProvenanceRecord {
    /// Construct a validated record without inventing any experimental defaults.
    pub fn new(
        arm: ReferenceArm,
        family: TaskFamily,
        generator_seed: u64,
        event_count: u64,
        query_count: u64,
        memory: MemoryAccounting,
        operations: EvaluationOperationAccounting,
    ) -> Result<Self, EvaluationProvenanceError> {
        if event_count == 0 {
            return Err(EvaluationProvenanceError::ZeroEventCount);
        }
        if query_count == 0 {
            return Err(EvaluationProvenanceError::ZeroQueryCount);
        }
        if query_count > event_count {
            return Err(EvaluationProvenanceError::QueryCountExceedsEvents {
                event_count,
                query_count,
            });
        }
        memory.validate_for_arm(arm)?;
        Ok(Self {
            arm,
            family,
            generator_seed,
            event_count,
            query_count,
            memory,
            operations,
        })
    }

    #[must_use]
    pub const fn arm(self) -> ReferenceArm {
        self.arm
    }

    #[must_use]
    pub const fn family(self) -> TaskFamily {
        self.family
    }

    #[must_use]
    pub const fn generator_seed(self) -> u64 {
        self.generator_seed
    }

    #[must_use]
    pub const fn event_count(self) -> u64 {
        self.event_count
    }

    #[must_use]
    pub const fn query_count(self) -> u64 {
        self.query_count
    }

    #[must_use]
    pub const fn memory(self) -> MemoryAccounting {
        self.memory
    }

    #[must_use]
    pub const fn operations(self) -> EvaluationOperationAccounting {
        self.operations
    }

    /// Stable parseable record with a fixed schema and field order.
    #[must_use]
    pub fn canonical_record(self) -> String {
        let arithmetic = self.operations.reference().arithmetic();
        let mechanism = self.operations.reference().mechanism_events();
        let evaluator = self.operations.evaluator();
        format!(
            concat!(
                "tdi8-evaluation-provenance-v1;arm={};family={};generator_seed={};",
                "event_count={};query_count={};",
                "memory.recurrent_state_bits={};memory.associative_payload_bits={};",
                "memory.associative_metadata_bits={};memory.vsa_workspace_bits={};",
                "memory.temporary_working_bits={};memory.cumulative_history_bits={};",
                "memory.static_parameter_bits={};",
                "reference.binary64_additions={};reference.binary64_subtractions={};",
                "reference.binary64_multiplications={};reference.binary64_divisions={};",
                "reference.binary64_comparisons={};reference.nonlinear_clamp_applications={};",
                "reference.address_projections={};reference.associative_reads={};",
                "reference.associative_writes={};reference.history_item_scores={};",
                "reference.vsa_bind_coordinates={};reference.vsa_bundle_coordinates={};",
                "reference.vsa_unbind_coordinates={};reference.vsa_similarity_coordinates={};",
                "evaluator.encoded_binary64_coordinates={};evaluator.decoded_binary64_coordinates={};",
                "evaluator.logical_key_derivations={};evaluator.bookkeeping_events={};",
                "evaluator.provenance_fields_emitted={}"
            ),
            self.arm.label(),
            task_family_label(self.family),
            self.generator_seed,
            self.event_count,
            self.query_count,
            self.memory.recurrent_state().get(),
            self.memory.associative_payload().get(),
            self.memory.associative_metadata().get(),
            self.memory.vsa_workspace().get(),
            self.memory.temporary_working().get(),
            self.memory.cumulative_history().get(),
            self.memory.static_parameters().get(),
            arithmetic.additions().get(),
            arithmetic.subtractions().get(),
            arithmetic.multiplications().get(),
            arithmetic.divisions().get(),
            arithmetic.comparisons().get(),
            arithmetic.nonlinear_clamps().get(),
            mechanism.address_projections().get(),
            mechanism.associative_reads().get(),
            mechanism.associative_writes().get(),
            mechanism.history_item_scores().get(),
            mechanism.vsa_bind_coordinates().get(),
            mechanism.vsa_bundle_coordinates().get(),
            mechanism.vsa_unbind_coordinates().get(),
            mechanism.vsa_similarity_coordinates().get(),
            evaluator.encoded_binary64_coordinates().get(),
            evaluator.decoded_binary64_coordinates().get(),
            evaluator.logical_key_derivations().get(),
            evaluator.bookkeeping_events().get(),
            evaluator.provenance_fields_emitted().get(),
        )
    }
}

fn task_family_label(family: TaskFamily) -> &'static str {
    match family {
        TaskFamily::AssociativeRecall => "T1",
        TaskFamily::DelayedCopy => "T2",
        TaskFamily::InterferenceRecall => "T3",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        EvaluationOperationAccounting, EvaluationProvenanceError, EvaluationProvenanceRecord,
        EvaluatorProtocolCounts, OperationAccountingError, OperationComponent, OperationScope,
        ReferenceArithmeticCounts, ReferenceMechanismEventCounts, ReferenceOperationAccounting,
    };
    use crate::task_generators::TaskFamily;
    use crate::{MemoryAccounting, ReferenceArm, StorageBits};

    fn operations() -> EvaluationOperationAccounting {
        EvaluationOperationAccounting::new(
            ReferenceOperationAccounting::new(
                ReferenceArithmeticCounts::new(1, 2, 3, 4, 5, 6),
                ReferenceMechanismEventCounts::new(7, 8, 9, 10, 11, 12, 13, 14),
            ),
            EvaluatorProtocolCounts::new(15, 16, 17, 18, 19),
        )
    }

    fn a1_memory() -> MemoryAccounting {
        MemoryAccounting::zero()
            .with_recurrent_state(StorageBits::new(128))
            .with_temporary_working(StorageBits::new(64))
            .with_static_parameters(StorageBits::new(256))
    }

    #[test]
    fn reference_and_evaluator_scopes_remain_separate() {
        let accounting = operations();
        assert_eq!(
            accounting.reference().arithmetic().additions().get(),
            1
        );
        assert_eq!(
            accounting.reference().mechanism_events().associative_reads().get(),
            8
        );
        assert_eq!(accounting.evaluator().encoded_binary64_coordinates().get(), 15);
    }

    #[test]
    fn componentwise_aggregation_is_exact_and_fail_closed_on_overflow() {
        let left = ReferenceArithmeticCounts::new(u128::MAX, 0, 0, 0, 0, 0);
        let right = ReferenceArithmeticCounts::new(1, 0, 0, 0, 0, 0);
        assert_eq!(
            left.checked_add(right),
            Err(OperationAccountingError::Overflow {
                scope: OperationScope::ReferenceMechanism,
                component: OperationComponent::Binary64Additions,
            })
        );

        let doubled = operations().checked_add(operations()).expect("exact aggregation");
        assert_eq!(doubled.reference().arithmetic().multiplications().get(), 6);
        assert_eq!(doubled.evaluator().bookkeeping_events().get(), 36);
    }

    #[test]
    fn provenance_rejects_invalid_task_counts_and_arm_memory() {
        assert_eq!(
            EvaluationProvenanceRecord::new(
                ReferenceArm::A1,
                TaskFamily::AssociativeRecall,
                7,
                0,
                1,
                a1_memory(),
                operations(),
            ),
            Err(EvaluationProvenanceError::ZeroEventCount)
        );
        assert_eq!(
            EvaluationProvenanceRecord::new(
                ReferenceArm::A1,
                TaskFamily::AssociativeRecall,
                7,
                2,
                0,
                a1_memory(),
                operations(),
            ),
            Err(EvaluationProvenanceError::ZeroQueryCount)
        );
        assert_eq!(
            EvaluationProvenanceRecord::new(
                ReferenceArm::A1,
                TaskFamily::AssociativeRecall,
                7,
                2,
                3,
                a1_memory(),
                operations(),
            ),
            Err(EvaluationProvenanceError::QueryCountExceedsEvents {
                event_count: 2,
                query_count: 3,
            })
        );

        let invalid_a1 = MemoryAccounting::zero()
            .with_recurrent_state(StorageBits::new(128))
            .with_associative_payload(StorageBits::new(64));
        assert!(matches!(
            EvaluationProvenanceRecord::new(
                ReferenceArm::A1,
                TaskFamily::AssociativeRecall,
                7,
                2,
                1,
                invalid_a1,
                operations(),
            ),
            Err(EvaluationProvenanceError::MemoryAccounting(_))
        ));
    }

    #[test]
    fn canonical_provenance_is_stable_machine_readable_and_scope_explicit() {
        let record = EvaluationProvenanceRecord::new(
            ReferenceArm::A1,
            TaskFamily::DelayedCopy,
            42,
            10,
            3,
            a1_memory(),
            operations(),
        )
        .expect("valid provenance");
        let canonical = record.canonical_record();
        assert_eq!(canonical, record.canonical_record());
        assert!(canonical.starts_with("tdi8-evaluation-provenance-v1;arm=A1;family=T2;"));
        assert!(canonical.contains("generator_seed=42;event_count=10;query_count=3;"));
        assert!(canonical.contains("memory.recurrent_state_bits=128;"));
        assert!(canonical.contains("reference.binary64_additions=1;"));
        assert!(canonical.contains("reference.associative_reads=8;"));
        assert!(canonical.contains("evaluator.encoded_binary64_coordinates=15;"));
        assert!(canonical.ends_with("evaluator.provenance_fields_emitted=19"));
    }

    #[test]
    fn per_item_reporting_remains_exact_rational_data() {
        let record = EvaluationProvenanceRecord::new(
            ReferenceArm::A1,
            TaskFamily::DelayedCopy,
            9,
            10,
            2,
            a1_memory(),
            operations(),
        )
        .expect("record");
        assert_eq!(record.event_count(), 10);
        assert_eq!(
            record.operations().reference().arithmetic().multiplications().get(),
            3
        );
    }
}
