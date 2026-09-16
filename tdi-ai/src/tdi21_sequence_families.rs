//! Typed deterministic Development/Validation sequence families for TDI-21 B4.
//!
//! Families contain explicit causal evaluator cases only. Counterfactual outcome
//! labels are never authored here: they are derived by `tdi21_sequence_materializer`.
//! Development and Validation use distinct Rust types and disjoint payload
//! namespaces while intentionally allowing the same candidate-visible predicate
//! states to recur across splits.

use super::tdi21::{BooleanState, MemoryRead};
use super::tdi21_distributional_objective::{
    DevelopmentAdmissionSet, DistributionalObjectiveError, ValidationAdmissionSet,
};
use super::tdi21_event_predicates::WRITE_PREDICATE_WIDTH;
use super::tdi21_predicate_identifiability::{AdmissionAuditCase, FutureRecallProbe};
use super::tdi21_sequence_materializer::{
    MaterializationSummary, SequenceMaterializationError, materialize_admission_cases,
};
use super::tdi21_stream::{Event, StreamConfig};

pub const SEQUENCE_FAMILY_SEMANTICS: &str = "tdi21-b4-v1-sequence-families-v1";
pub const FAMILY_CASE_COUNT: usize = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SequenceSplit {
    Development,
    Validation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FamilyScenario {
    PreserveFirstOldFact,
    RetainNewFact,
    PreserveSecondOldFact,
    ExactUpdate,
    PreserveUnrelatedDuringUpdate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FamilyCaseId {
    pub split: SequenceSplit,
    pub scenario: FamilyScenario,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FamilyCase {
    id: FamilyCaseId,
    audit: AdmissionAuditCase,
}

impl FamilyCase {
    #[must_use]
    pub const fn id(&self) -> FamilyCaseId {
        self.id
    }

    #[must_use]
    pub const fn audit(&self) -> &AdmissionAuditCase {
        &self.audit
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SequenceFamily {
    split: SequenceSplit,
    cases: Vec<FamilyCase>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DevelopmentSequenceFamily(SequenceFamily);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidationSequenceFamily(SequenceFamily);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaterializedDevelopmentFamily {
    ids: Vec<FamilyCaseId>,
    dataset: DevelopmentAdmissionSet,
    summary: MaterializationSummary,
}

impl MaterializedDevelopmentFamily {
    #[must_use]
    pub fn ids(&self) -> &[FamilyCaseId] {
        &self.ids
    }

    #[must_use]
    pub const fn dataset(&self) -> &DevelopmentAdmissionSet {
        &self.dataset
    }

    #[must_use]
    pub const fn summary(&self) -> MaterializationSummary {
        self.summary
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaterializedValidationFamily {
    ids: Vec<FamilyCaseId>,
    dataset: ValidationAdmissionSet,
    summary: MaterializationSummary,
}

impl MaterializedValidationFamily {
    #[must_use]
    pub fn ids(&self) -> &[FamilyCaseId] {
        &self.ids
    }

    #[must_use]
    pub const fn dataset(&self) -> &ValidationAdmissionSet {
        &self.dataset
    }

    #[must_use]
    pub const fn summary(&self) -> MaterializationSummary {
        self.summary
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SequenceFamilyError {
    SplitMismatch {
        family: SequenceSplit,
        case: SequenceSplit,
        scenario: FamilyScenario,
    },
    DuplicateId {
        split: SequenceSplit,
        scenario: FamilyScenario,
    },
    DuplicateCaseWithinSplit {
        split: SequenceSplit,
    },
    SharedCaseAcrossSplits,
    Materialization(SequenceMaterializationError),
    Dataset(DistributionalObjectiveError),
    AllocationFailed,
}

impl From<SequenceMaterializationError> for SequenceFamilyError {
    fn from(value: SequenceMaterializationError) -> Self {
        Self::Materialization(value)
    }
}

impl From<DistributionalObjectiveError> for SequenceFamilyError {
    fn from(value: DistributionalObjectiveError) -> Self {
        Self::Dataset(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PayloadTemplate {
    first_old: u64,
    second_old: u64,
    new_fact: u64,
    update: u64,
}

fn template(split: SequenceSplit) -> PayloadTemplate {
    match split {
        SequenceSplit::Development => PayloadTemplate {
            first_old: 11,
            second_old: 55,
            new_fact: 99,
            update: 77,
        },
        SequenceSplit::Validation => PayloadTemplate {
            first_old: 21,
            second_old: 65,
            new_fact: 109,
            update: 87,
        },
    }
}

fn write(key: u64, payload: u64) -> Event {
    Event::Write {
        key,
        payload: BooleanState::from_bits(payload),
        marker: BooleanState::from_bits(1),
    }
}

fn hit(payload: u64) -> MemoryRead {
    MemoryRead::Hit(BooleanState::from_bits(payload))
}

fn audit_case(
    payloads: PayloadTemplate,
    decision: Event,
    probe_key: u64,
    expected: u64,
) -> AdmissionAuditCase {
    AdmissionAuditCase {
        prefix: vec![write(1, payloads.first_old), write(5, payloads.second_old)],
        decision,
        probe: FutureRecallProbe {
            key: probe_key,
            expected: hit(expected),
        },
    }
}

fn family_case(
    split: SequenceSplit,
    scenario: FamilyScenario,
    audit: AdmissionAuditCase,
) -> FamilyCase {
    FamilyCase {
        id: FamilyCaseId { split, scenario },
        audit,
    }
}

fn generate(split: SequenceSplit) -> SequenceFamily {
    let p = template(split);
    let cases = vec![
        family_case(
            split,
            FamilyScenario::PreserveFirstOldFact,
            audit_case(p, write(9, p.new_fact), 1, p.first_old),
        ),
        family_case(
            split,
            FamilyScenario::RetainNewFact,
            audit_case(p, write(9, p.new_fact), 9, p.new_fact),
        ),
        family_case(
            split,
            FamilyScenario::PreserveSecondOldFact,
            audit_case(p, write(9, p.new_fact), 5, p.second_old),
        ),
        family_case(
            split,
            FamilyScenario::ExactUpdate,
            audit_case(p, write(5, p.update), 5, p.update),
        ),
        family_case(
            split,
            FamilyScenario::PreserveUnrelatedDuringUpdate,
            audit_case(p, write(5, p.update), 1, p.first_old),
        ),
    ];
    SequenceFamily { split, cases }
}

fn validate_family(family: &SequenceFamily) -> Result<(), SequenceFamilyError> {
    for (i, left) in family.cases.iter().enumerate() {
        if left.id.split != family.split {
            return Err(SequenceFamilyError::SplitMismatch {
                family: family.split,
                case: left.id.split,
                scenario: left.id.scenario,
            });
        }
        for right in &family.cases[..i] {
            if left.id == right.id {
                return Err(SequenceFamilyError::DuplicateId {
                    split: family.split,
                    scenario: left.id.scenario,
                });
            }
            if left.audit == right.audit {
                return Err(SequenceFamilyError::DuplicateCaseWithinSplit {
                    split: family.split,
                });
            }
        }
    }
    Ok(())
}

pub fn validate_split_disjointness(
    development: &DevelopmentSequenceFamily,
    validation: &ValidationSequenceFamily,
) -> Result<(), SequenceFamilyError> {
    validate_family(&development.0)?;
    validate_family(&validation.0)?;
    for development_case in &development.0.cases {
        for validation_case in &validation.0.cases {
            if development_case.audit == validation_case.audit {
                return Err(SequenceFamilyError::SharedCaseAcrossSplits);
            }
        }
    }
    Ok(())
}

impl DevelopmentSequenceFamily {
    #[must_use]
    pub fn v1() -> Self {
        Self(generate(SequenceSplit::Development))
    }

    #[must_use]
    pub fn cases(&self) -> &[FamilyCase] {
        &self.0.cases
    }
}

impl ValidationSequenceFamily {
    #[must_use]
    pub fn v1() -> Self {
        Self(generate(SequenceSplit::Validation))
    }

    #[must_use]
    pub fn cases(&self) -> &[FamilyCase] {
        &self.0.cases
    }
}

fn audits(cases: &[FamilyCase]) -> Result<Vec<AdmissionAuditCase>, SequenceFamilyError> {
    let mut out = Vec::new();
    out.try_reserve_exact(cases.len())
        .map_err(|_| SequenceFamilyError::AllocationFailed)?;
    out.extend(cases.iter().map(|case| case.audit.clone()));
    Ok(out)
}

fn ids(cases: &[FamilyCase]) -> Result<Vec<FamilyCaseId>, SequenceFamilyError> {
    let mut out = Vec::new();
    out.try_reserve_exact(cases.len())
        .map_err(|_| SequenceFamilyError::AllocationFailed)?;
    out.extend(cases.iter().map(FamilyCase::id));
    Ok(out)
}

pub fn materialize_development_family(
    config: StreamConfig,
    family: &DevelopmentSequenceFamily,
) -> Result<MaterializedDevelopmentFamily, SequenceFamilyError> {
    validate_family(&family.0)?;
    let materialized = materialize_admission_cases(config, &audits(&family.0.cases)?)?;
    let dataset = DevelopmentAdmissionSet::new(WRITE_PREDICATE_WIDTH, materialized.samples())?;
    Ok(MaterializedDevelopmentFamily {
        ids: ids(&family.0.cases)?,
        dataset,
        summary: materialized.summary(),
    })
}

pub fn materialize_validation_family(
    config: StreamConfig,
    family: &ValidationSequenceFamily,
) -> Result<MaterializedValidationFamily, SequenceFamilyError> {
    validate_family(&family.0)?;
    let materialized = materialize_admission_cases(config, &audits(&family.0.cases)?)?;
    let dataset = ValidationAdmissionSet::new(WRITE_PREDICATE_WIDTH, materialized.samples())?;
    Ok(MaterializedValidationFamily {
        ids: ids(&family.0.cases)?,
        dataset,
        summary: materialized.summary(),
    })
}
