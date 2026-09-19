//! Development-only bounded search for the TDI-21 relational address control.
//!
//! Each output address bit is selected independently from the tiny Boolean
//! family {0, 1, x_i, NOT x_i}. Validation cannot participate in fitting.
//! This isolates feature-coverage/generalization effects before richer ANF search.

use std::collections::BTreeMap;

use super::tdi21_relational_tasks::{
    DevelopmentRelationalSet, RELATIONAL_V1_ENTITY_BITS, RELATIONAL_V1_RELATION_BITS,
    RelationalEpisode, ValidationRelationalSet,
};

pub const RELATIONAL_ADDRESS_SEARCH_SEMANTICS: &str = "tdi21-relational-address-search-v1";
pub const RELATIONAL_ADDRESS_BITS: u8 = RELATIONAL_V1_ENTITY_BITS + RELATIONAL_V1_RELATION_BITS;
pub const MAX_ADDRESS_SAMPLES: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnaryAddressRule {
    Constant(bool),
    Literal { variable: u8, inverted: bool },
}

impl UnaryAddressRule {
    #[must_use]
    pub const fn evaluate(self, input: u8) -> bool {
        match self {
            Self::Constant(value) => value,
            Self::Literal { variable, inverted } => {
                let value = ((input >> variable) & 1) != 0;
                value ^ inverted
            }
        }
    }

    #[must_use]
    const fn complexity(self) -> u8 {
        match self {
            Self::Constant(_) => 0,
            Self::Literal { .. } => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AddressSample {
    pub input: u8,
    pub expected: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AddressSet {
    samples: Vec<AddressSample>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DevelopmentAddressSet(AddressSet);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidationAddressSet(AddressSet);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AddressSearchWork {
    pub candidate_rules_evaluated: u64,
    pub sample_bit_evaluations: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AddressSearchResult {
    rules: [UnaryAddressRule; RELATIONAL_ADDRESS_BITS as usize],
    development_samples: usize,
    development_bit_mismatches: u64,
    work: AddressSearchWork,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AddressValidationEvidence {
    pub samples: u64,
    pub address_mismatches: u64,
    pub bit_mismatches: u64,
    /// Additional distinct inputs mapped onto an already-seen predicted key.
    /// This is encoder aliasing, not a B3 bucket collision.
    pub prediction_aliases: u64,
    pub rule_evaluations: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AddressSearchError {
    EmptySamples,
    TooManySamples,
    DuplicateInput { input: u8 },
    InputOutOfRange { input: u8 },
    ExpectedOutOfRange { expected: u8 },
    IdentifierOutOfRange,
    CounterOverflow,
    AllocationFailed,
}

fn address_limit() -> u16 {
    1u16 << RELATIONAL_ADDRESS_BITS
}

fn build_set(samples: &[AddressSample]) -> Result<AddressSet, AddressSearchError> {
    if samples.is_empty() {
        return Err(AddressSearchError::EmptySamples);
    }
    if samples.len() > MAX_ADDRESS_SAMPLES {
        return Err(AddressSearchError::TooManySamples);
    }
    let limit = address_limit();
    let mut seen = BTreeMap::new();
    let mut owned = Vec::new();
    owned
        .try_reserve_exact(samples.len())
        .map_err(|_| AddressSearchError::AllocationFailed)?;
    for sample in samples.iter().copied() {
        if u16::from(sample.input) >= limit {
            return Err(AddressSearchError::InputOutOfRange { input: sample.input });
        }
        if u16::from(sample.expected) >= limit {
            return Err(AddressSearchError::ExpectedOutOfRange {
                expected: sample.expected,
            });
        }
        if seen.insert(sample.input, ()).is_some() {
            return Err(AddressSearchError::DuplicateInput { input: sample.input });
        }
        owned.push(sample);
    }
    Ok(AddressSet { samples: owned })
}

impl DevelopmentAddressSet {
    pub fn new(samples: &[AddressSample]) -> Result<Self, AddressSearchError> {
        build_set(samples).map(Self)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.samples.len()
    }
}

impl ValidationAddressSet {
    pub fn new(samples: &[AddressSample]) -> Result<Self, AddressSearchError> {
        build_set(samples).map(Self)
    }
}

fn pack(relation: u64, subject: u64) -> Result<u8, AddressSearchError> {
    let entity_limit = 1u64 << RELATIONAL_V1_ENTITY_BITS;
    let relation_limit = 1u64 << RELATIONAL_V1_RELATION_BITS;
    if subject >= entity_limit || relation >= relation_limit {
        return Err(AddressSearchError::IdentifierOutOfRange);
    }
    Ok(((relation << RELATIONAL_V1_ENTITY_BITS) | subject) as u8)
}

fn task_samples(episodes: &[RelationalEpisode]) -> Result<Vec<AddressSample>, AddressSearchError> {
    let mut unique = BTreeMap::new();
    for episode in episodes {
        for fact in &episode.facts {
            let assignment = pack(fact.relation, fact.subject)?;
            unique.insert(
                assignment,
                AddressSample {
                    input: assignment,
                    expected: assignment,
                },
            );
        }
    }
    let mut samples = Vec::new();
    samples
        .try_reserve_exact(unique.len())
        .map_err(|_| AddressSearchError::AllocationFailed)?;
    samples.extend(unique.into_values());
    Ok(samples)
}

pub fn development_from_relational_tasks(
    tasks: &DevelopmentRelationalSet,
) -> Result<DevelopmentAddressSet, AddressSearchError> {
    DevelopmentAddressSet::new(&task_samples(tasks.episodes())?)
}

pub fn validation_from_relational_tasks(
    tasks: &ValidationRelationalSet,
) -> Result<ValidationAddressSet, AddressSearchError> {
    ValidationAddressSet::new(&task_samples(tasks.episodes())?)
}

fn rules() -> [UnaryAddressRule; 18] {
    let mut out = [UnaryAddressRule::Constant(false); 18];
    out[0] = UnaryAddressRule::Constant(false);
    out[1] = UnaryAddressRule::Constant(true);
    let mut index = 2;
    let mut variable = 0;
    while variable < RELATIONAL_ADDRESS_BITS {
        out[index] = UnaryAddressRule::Literal {
            variable,
            inverted: false,
        };
        out[index + 1] = UnaryAddressRule::Literal {
            variable,
            inverted: true,
        };
        index += 2;
        variable += 1;
    }
    out
}

fn checked_add(target: &mut u64, amount: u64) -> Result<(), AddressSearchError> {
    *target = target
        .checked_add(amount)
        .ok_or(AddressSearchError::CounterOverflow)?;
    Ok(())
}

fn bit_mismatches(rule: UnaryAddressRule, bit: u8, samples: &[AddressSample]) -> u64 {
    samples
        .iter()
        .filter(|sample| {
            rule.evaluate(sample.input) != (((sample.expected >> bit) & 1) != 0)
        })
        .count() as u64
}

fn better_rule(
    mismatches: u64,
    rule: UnaryAddressRule,
    best_mismatches: u64,
    best_rule: UnaryAddressRule,
) -> bool {
    (mismatches, rule.complexity(), rule) < (best_mismatches, best_rule.complexity(), best_rule)
}

pub fn fit_relational_address(
    development: &DevelopmentAddressSet,
) -> Result<AddressSearchResult, AddressSearchError> {
    let candidates = rules();
    let mut selected = [UnaryAddressRule::Constant(false); RELATIONAL_ADDRESS_BITS as usize];
    let mut total_mismatches = 0u64;
    let mut work = AddressSearchWork::default();

    for bit in 0..RELATIONAL_ADDRESS_BITS {
        let mut best: Option<(u64, UnaryAddressRule)> = None;
        for rule in candidates {
            checked_add(&mut work.candidate_rules_evaluated, 1)?;
            checked_add(
                &mut work.sample_bit_evaluations,
                development.0.samples.len() as u64,
            )?;
            let mismatches = bit_mismatches(rule, bit, &development.0.samples);
            match best {
                None => best = Some((mismatches, rule)),
                Some((best_mismatches, best_rule))
                    if better_rule(mismatches, rule, best_mismatches, best_rule) =>
                {
                    best = Some((mismatches, rule));
                }
                _ => {}
            }
        }
        let (mismatches, rule) = best.expect("fixed non-empty rule family");
        checked_add(&mut total_mismatches, mismatches)?;
        selected[bit as usize] = rule;
    }

    Ok(AddressSearchResult {
        rules: selected,
        development_samples: development.0.samples.len(),
        development_bit_mismatches: total_mismatches,
        work,
    })
}

impl AddressSearchResult {
    #[must_use]
    pub fn rules(&self) -> &[UnaryAddressRule; RELATIONAL_ADDRESS_BITS as usize] {
        &self.rules
    }

    #[must_use]
    pub const fn development_samples(&self) -> usize {
        self.development_samples
    }

    #[must_use]
    pub const fn development_bit_mismatches(&self) -> u64 {
        self.development_bit_mismatches
    }

    #[must_use]
    pub const fn work(&self) -> AddressSearchWork {
        self.work
    }

    #[must_use]
    pub fn predict(&self, input: u8) -> u8 {
        let mut output = 0u8;
        for (bit, rule) in self.rules.iter().copied().enumerate() {
            if rule.evaluate(input) {
                output |= 1u8 << bit;
            }
        }
        output
    }
}

pub fn evaluate_address_validation(
    result: &AddressSearchResult,
    validation: &ValidationAddressSet,
) -> Result<AddressValidationEvidence, AddressSearchError> {
    let mut evidence = AddressValidationEvidence::default();
    let mut predicted_inputs = BTreeMap::new();
    for sample in &validation.0.samples {
        checked_add(&mut evidence.samples, 1)?;
        checked_add(
            &mut evidence.rule_evaluations,
            u64::from(RELATIONAL_ADDRESS_BITS),
        )?;
        let predicted = result.predict(sample.input);
        if let Some(previous_input) = predicted_inputs.insert(predicted, sample.input)
            && previous_input != sample.input
        {
            checked_add(&mut evidence.prediction_aliases, 1)?;
        }
        if predicted != sample.expected {
            checked_add(&mut evidence.address_mismatches, 1)?;
        }
        checked_add(
            &mut evidence.bit_mismatches,
            (predicted ^ sample.expected).count_ones() as u64,
        )?;
    }
    Ok(evidence)
}
