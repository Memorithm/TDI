//! Development-only bounded sparse-ANF search for TDI-21 B4.
//!
//! Search accepts a typed Development set only. Validation has a distinct type
//! and can be evaluated only after a program has been selected. This keeps the
//! first search surface structurally incapable of reading Validation labels.
//! No final/holdout population, attention primitive, model training, or claim of
//! generalization lives in this module.

use std::collections::BTreeSet;

use super::tdi21_anf_synthesis::{AnfProgram, AnfSynthesisError, synthesize_anf};

pub const ANF_SEARCH_SEMANTICS: &str = "tdi21-sparse-anf-search-v1";
pub const MAX_SEARCH_VARIABLES: u8 = 6;
pub const MAX_SEARCH_DEGREE: u8 = 3;
pub const MAX_SEARCH_TERMS: usize = 4;
pub const MAX_SEARCH_CANDIDATES: u64 = 65_536;
pub const MAX_SEARCH_CASES: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LabeledAssignment {
    pub assignment: u64,
    pub expected: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CaseSet {
    variable_count: u8,
    cases: Vec<LabeledAssignment>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DevelopmentSet(CaseSet);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidationSet(CaseSet);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SearchEnvelope {
    pub variable_count: u8,
    pub max_degree: u8,
    pub max_terms: usize,
    pub max_candidates: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SearchWork {
    pub candidates_evaluated: u64,
    pub case_evaluations: u64,
    pub monomial_evaluations: u64,
}

/// Selected candidate together with the exact variable arity under which it was
/// fitted. Arity is part of candidate identity and cannot be supplied anew by a
/// caller during conversion or Validation evaluation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchResult {
    variable_count: u8,
    constant: bool,
    monomials: Vec<u64>,
    development_mismatches: u64,
    work: SearchWork,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ValidationEvidence {
    pub cases: u64,
    pub mismatches: u64,
    pub monomial_evaluations: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnfSearchError {
    ZeroVariables,
    TooManyVariables,
    EmptyCaseSet,
    TooManyCases,
    AssignmentOutOfRange { index: usize, assignment: u64 },
    DuplicateAssignment { assignment: u64 },
    VariableCountMismatch { expected: u8, actual: u8 },
    ZeroDegree,
    DegreeExceedsVariableCount,
    DegreeExceedsSearchLimit,
    TooManyTerms,
    ZeroCandidateBudget,
    CandidateBudgetExceedsLimit,
    CandidateSpaceExceedsBudget { required: u64, budget: u64 },
    CounterOverflow,
    AllocationFailed,
    Synthesis(AnfSynthesisError),
}

fn build_case_set(
    variable_count: u8,
    cases: &[LabeledAssignment],
) -> Result<CaseSet, AnfSearchError> {
    if variable_count == 0 {
        return Err(AnfSearchError::ZeroVariables);
    }
    if variable_count > MAX_SEARCH_VARIABLES {
        return Err(AnfSearchError::TooManyVariables);
    }
    if cases.is_empty() {
        return Err(AnfSearchError::EmptyCaseSet);
    }
    if cases.len() > MAX_SEARCH_CASES {
        return Err(AnfSearchError::TooManyCases);
    }
    let limit = 1u64 << variable_count;
    let mut assignments = BTreeSet::new();
    for (index, case) in cases.iter().copied().enumerate() {
        if case.assignment >= limit {
            return Err(AnfSearchError::AssignmentOutOfRange {
                index,
                assignment: case.assignment,
            });
        }
        if !assignments.insert(case.assignment) {
            return Err(AnfSearchError::DuplicateAssignment {
                assignment: case.assignment,
            });
        }
    }
    let mut owned = Vec::new();
    owned
        .try_reserve_exact(cases.len())
        .map_err(|_| AnfSearchError::AllocationFailed)?;
    owned.extend_from_slice(cases);
    Ok(CaseSet {
        variable_count,
        cases: owned,
    })
}

impl DevelopmentSet {
    pub fn new(variable_count: u8, cases: &[LabeledAssignment]) -> Result<Self, AnfSearchError> {
        build_case_set(variable_count, cases).map(Self)
    }

    #[must_use]
    pub const fn variable_count(&self) -> u8 {
        self.0.variable_count
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.cases.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.cases.is_empty()
    }
}

impl ValidationSet {
    pub fn new(variable_count: u8, cases: &[LabeledAssignment]) -> Result<Self, AnfSearchError> {
        build_case_set(variable_count, cases).map(Self)
    }

    #[must_use]
    pub const fn variable_count(&self) -> u8 {
        self.0.variable_count
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.cases.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.cases.is_empty()
    }
}

impl SearchEnvelope {
    pub fn validate(self) -> Result<(), AnfSearchError> {
        if self.variable_count == 0 {
            return Err(AnfSearchError::ZeroVariables);
        }
        if self.variable_count > MAX_SEARCH_VARIABLES {
            return Err(AnfSearchError::TooManyVariables);
        }
        if self.max_degree == 0 {
            return Err(AnfSearchError::ZeroDegree);
        }
        if self.max_degree > self.variable_count {
            return Err(AnfSearchError::DegreeExceedsVariableCount);
        }
        if self.max_degree > MAX_SEARCH_DEGREE {
            return Err(AnfSearchError::DegreeExceedsSearchLimit);
        }
        if self.max_terms > MAX_SEARCH_TERMS {
            return Err(AnfSearchError::TooManyTerms);
        }
        if self.max_candidates == 0 {
            return Err(AnfSearchError::ZeroCandidateBudget);
        }
        if self.max_candidates > MAX_SEARCH_CANDIDATES {
            return Err(AnfSearchError::CandidateBudgetExceedsLimit);
        }
        Ok(())
    }
}

impl SearchResult {
    #[must_use]
    pub const fn variable_count(&self) -> u8 {
        self.variable_count
    }

    #[must_use]
    pub const fn constant(&self) -> bool {
        self.constant
    }

    #[must_use]
    pub fn monomials(&self) -> &[u64] {
        &self.monomials
    }

    #[must_use]
    pub const fn development_mismatches(&self) -> u64 {
        self.development_mismatches
    }

    #[must_use]
    pub const fn work(&self) -> SearchWork {
        self.work
    }

    #[must_use]
    pub fn max_degree(&self) -> u32 {
        self.monomials
            .iter()
            .map(|mask| mask.count_ones())
            .max()
            .unwrap_or(0)
    }

    /// Canonicalize the selected sparse program through the exact ANF
    /// synthesizer at the same arity used during fitting.
    pub fn to_anf_program(&self) -> Result<AnfProgram, AnfSearchError> {
        let rows = 1usize << self.variable_count;
        let mut table = Vec::new();
        table
            .try_reserve_exact(rows)
            .map_err(|_| AnfSearchError::AllocationFailed)?;
        for assignment in 0..rows as u64 {
            table.push(evaluate_sparse(self.constant, &self.monomials, assignment));
        }
        synthesize_anf(self.variable_count, &table).map_err(AnfSearchError::Synthesis)
    }
}

fn checked_add(counter: &mut u64, amount: u64) -> Result<(), AnfSearchError> {
    *counter = counter
        .checked_add(amount)
        .ok_or(AnfSearchError::CounterOverflow)?;
    Ok(())
}

fn evaluate_sparse(constant: bool, monomials: &[u64], assignment: u64) -> bool {
    let mut value = constant;
    for &mask in monomials {
        value ^= assignment & mask == mask;
    }
    value
}

fn monomial_universe(envelope: SearchEnvelope) -> Result<Vec<u64>, AnfSearchError> {
    envelope.validate()?;
    let limit = 1u64 << envelope.variable_count;
    let mut masks = Vec::new();
    for mask in 1..limit {
        if mask.count_ones() <= u32::from(envelope.max_degree) {
            masks.push(mask);
        }
    }
    Ok(masks)
}

fn binomial(n: usize, k: usize) -> Result<u64, AnfSearchError> {
    if k > n {
        return Ok(0);
    }
    let k = k.min(n - k);
    let mut value = 1u64;
    for i in 0..k {
        value = value
            .checked_mul((n - i) as u64)
            .ok_or(AnfSearchError::CounterOverflow)?;
        value /= (i + 1) as u64;
    }
    Ok(value)
}

fn candidate_space_size(monomials: usize, max_terms: usize) -> Result<u64, AnfSearchError> {
    let mut combinations = 0u64;
    for terms in 0..=max_terms.min(monomials) {
        checked_add(&mut combinations, binomial(monomials, terms)?)?;
    }
    combinations
        .checked_mul(2)
        .ok_or(AnfSearchError::CounterOverflow)
}

fn enumerate_combinations(
    universe: &[u64],
    start: usize,
    remaining: usize,
    current: &mut Vec<u64>,
    out: &mut Vec<Vec<u64>>,
) -> Result<(), AnfSearchError> {
    if remaining == 0 {
        out.try_reserve(1)
            .map_err(|_| AnfSearchError::AllocationFailed)?;
        out.push(current.clone());
        return Ok(());
    }
    if universe.len().saturating_sub(start) < remaining {
        return Ok(());
    }
    let final_start = universe.len() - remaining;
    for index in start..=final_start {
        current.push(universe[index]);
        enumerate_combinations(universe, index + 1, remaining - 1, current, out)?;
        current.pop();
    }
    Ok(())
}

fn better_candidate(
    mismatches: u64,
    constant: bool,
    monomials: &[u64],
    best_mismatches: u64,
    best_constant: bool,
    best_monomials: &[u64],
) -> bool {
    let degree = monomials
        .iter()
        .map(|mask| mask.count_ones())
        .max()
        .unwrap_or(0);
    let best_degree = best_monomials
        .iter()
        .map(|mask| mask.count_ones())
        .max()
        .unwrap_or(0);
    (mismatches, monomials.len(), degree, constant, monomials)
        < (
            best_mismatches,
            best_monomials.len(),
            best_degree,
            best_constant,
            best_monomials,
        )
}

/// Exhaustive bounded sparse-ANF search over Development labels only.
///
/// Selection is deterministic and lexicographic: fewer Development mismatches,
/// then fewer monomials, lower maximum degree, constant=false before true, then
/// canonical monomial-mask order. Validation labels cannot be supplied here.
pub fn fit_sparse_anf(
    development: &DevelopmentSet,
    envelope: SearchEnvelope,
) -> Result<SearchResult, AnfSearchError> {
    envelope.validate()?;
    if development.variable_count() != envelope.variable_count {
        return Err(AnfSearchError::VariableCountMismatch {
            expected: envelope.variable_count,
            actual: development.variable_count(),
        });
    }
    let universe = monomial_universe(envelope)?;
    let required = candidate_space_size(universe.len(), envelope.max_terms)?;
    if required > envelope.max_candidates {
        return Err(AnfSearchError::CandidateSpaceExceedsBudget {
            required,
            budget: envelope.max_candidates,
        });
    }

    let mut combinations = Vec::new();
    combinations
        .try_reserve_exact((required / 2) as usize)
        .map_err(|_| AnfSearchError::AllocationFailed)?;
    for term_count in 0..=envelope.max_terms.min(universe.len()) {
        enumerate_combinations(&universe, 0, term_count, &mut Vec::new(), &mut combinations)?;
    }

    let mut work = SearchWork::default();
    let mut best: Option<(u64, bool, Vec<u64>)> = None;
    for constant in [false, true] {
        for monomials in &combinations {
            checked_add(&mut work.candidates_evaluated, 1)?;
            let mut mismatches = 0u64;
            for case in &development.0.cases {
                checked_add(&mut work.case_evaluations, 1)?;
                checked_add(&mut work.monomial_evaluations, monomials.len() as u64)?;
                if evaluate_sparse(constant, monomials, case.assignment) != case.expected {
                    checked_add(&mut mismatches, 1)?;
                }
            }
            match &best {
                None => best = Some((mismatches, constant, monomials.clone())),
                Some((best_mismatches, best_constant, best_monomials))
                    if better_candidate(
                        mismatches,
                        constant,
                        monomials,
                        *best_mismatches,
                        *best_constant,
                        best_monomials,
                    ) =>
                {
                    best = Some((mismatches, constant, monomials.clone()));
                }
                _ => {}
            }
        }
    }
    let (development_mismatches, constant, monomials) =
        best.expect("validated search space always contains constant candidates");
    Ok(SearchResult {
        variable_count: envelope.variable_count,
        constant,
        monomials,
        development_mismatches,
        work,
    })
}

/// Evaluate an already-selected program on Validation labels. The fitted arity
/// is part of the result, so callers cannot reinterpret a selected program under
/// a different variable count. This function cannot alter selection.
pub fn evaluate_validation(
    result: &SearchResult,
    validation: &ValidationSet,
) -> Result<ValidationEvidence, AnfSearchError> {
    if validation.variable_count() != result.variable_count {
        return Err(AnfSearchError::VariableCountMismatch {
            expected: result.variable_count,
            actual: validation.variable_count(),
        });
    }
    let mut evidence = ValidationEvidence {
        cases: 0,
        mismatches: 0,
        monomial_evaluations: 0,
    };
    for case in &validation.0.cases {
        checked_add(&mut evidence.cases, 1)?;
        checked_add(
            &mut evidence.monomial_evaluations,
            result.monomials.len() as u64,
        )?;
        if evaluate_sparse(result.constant, &result.monomials, case.assignment) != case.expected {
            checked_add(&mut evidence.mismatches, 1)?;
        }
    }
    Ok(evidence)
}
