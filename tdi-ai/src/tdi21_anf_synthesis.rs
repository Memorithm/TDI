//! Development-only B4 algebraic composition for TDI-21.
//!
//! This module adds an explicit GF(2) / algebraic-normal-form (Zhegalkin)
//! composition layer to the existing B3 causal memory without introducing an
//! attention primitive. The first B4 control intentionally preserves B3 task
//! semantics: it synthesizes the existing two-bit write-admission predicate as
//! an ANF program, evaluates that program on each write marker, then delegates
//! storage and retrieval to the bounded two-way B3 memory.
//!
//! This is a substitution/isolation experiment, not learned routing, a model
//! benchmark, a TDI-21.3 freeze, or evidence that ANF improves quality.

use super::tdi21::{
    AnfTerm, ArchitectureArm, ResourceCounters, counted_evaluate_anf,
};
use super::tdi21_stream::{
    BooleanStream, Event, MemoryFootprint, MemoryMode, StepOutput, StreamConfig, StreamCounters,
    StreamError,
};

/// Versioned development semantics for exact truth-table -> ANF synthesis.
pub const ANF_SYNTHESIS_SEMANTICS: &str = "tdi21-anf-mobius-v1";
/// Bounded to keep synthesis exhaustive and development costs explicit.
pub const MAX_ANF_VARIABLES: u8 = 12;
/// Maximum number of non-constant monomials admitted in one synthesized program.
pub const MAX_ANF_TERMS: usize = 4095;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnfSynthesisError {
    ZeroVariables,
    TooManyVariables,
    TruthTableLengthMismatch { expected: usize, actual: usize },
    TooManyTerms,
    AllocationFailed,
    AssignmentOutOfRange,
}

/// Canonical ANF over variables x0..x(n-1). Terms are stored in ascending
/// monomial-mask order, making synthesis deterministic and byte-stable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnfProgram {
    variable_count: u8,
    constant: bool,
    terms: Vec<AnfTerm>,
}

impl AnfProgram {
    #[must_use]
    pub const fn variable_count(&self) -> u8 {
        self.variable_count
    }

    #[must_use]
    pub const fn constant(&self) -> bool {
        self.constant
    }

    #[must_use]
    pub fn terms(&self) -> &[AnfTerm] {
        &self.terms
    }

    /// Evaluate while rejecting assignments that contain undeclared variables.
    pub fn evaluate(&self, assignment: u64) -> Result<bool, AnfSynthesisError> {
        if self.variable_count < 64 && assignment >> self.variable_count != 0 {
            return Err(AnfSynthesisError::AssignmentOutOfRange);
        }
        Ok(super::tdi21::evaluate_anf(
            self.constant,
            &self.terms,
            assignment,
        ))
    }

    /// Count ANF monomial evaluations separately from B3 routing/memory work.
    pub fn evaluate_counted(
        &self,
        assignment: u64,
        counters: &mut ResourceCounters,
    ) -> Result<bool, AnfSynthesisError> {
        if self.variable_count < 64 && assignment >> self.variable_count != 0 {
            return Err(AnfSynthesisError::AssignmentOutOfRange);
        }
        Ok(counted_evaluate_anf(
            self.constant,
            &self.terms,
            assignment,
            counters,
        ))
    }

    /// Semantic representation bits only: variable-count byte, constant bit,
    /// and one 64-bit monomial mask per non-constant term. This is not process
    /// memory, allocator overhead or a hardware memory measurement.
    #[must_use]
    pub fn semantic_bits(&self) -> usize {
        9usize
            .checked_add(self.terms.len().checked_mul(64).expect("ANF term bit overflow"))
            .expect("ANF semantic bit overflow")
    }
}

/// Exact Möbius transform over GF(2). `truth_table[a]` is f(a), where the bit
/// pattern of `a` is the Boolean assignment. No stochastic search is involved.
pub fn synthesize_anf(
    variable_count: u8,
    truth_table: &[bool],
) -> Result<AnfProgram, AnfSynthesisError> {
    if variable_count == 0 {
        return Err(AnfSynthesisError::ZeroVariables);
    }
    if variable_count > MAX_ANF_VARIABLES {
        return Err(AnfSynthesisError::TooManyVariables);
    }
    let expected = 1usize << variable_count;
    if truth_table.len() != expected {
        return Err(AnfSynthesisError::TruthTableLengthMismatch {
            expected,
            actual: truth_table.len(),
        });
    }

    let mut coefficients = Vec::new();
    coefficients
        .try_reserve_exact(expected)
        .map_err(|_| AnfSynthesisError::AllocationFailed)?;
    coefficients.extend_from_slice(truth_table);

    for bit in 0..variable_count {
        let selector = 1usize << bit;
        for mask in 0..expected {
            if mask & selector != 0 {
                coefficients[mask] ^= coefficients[mask ^ selector];
            }
        }
    }

    let term_count = coefficients.iter().skip(1).filter(|&&value| value).count();
    if term_count > MAX_ANF_TERMS {
        return Err(AnfSynthesisError::TooManyTerms);
    }
    let mut terms = Vec::new();
    terms
        .try_reserve_exact(term_count)
        .map_err(|_| AnfSynthesisError::AllocationFailed)?;
    for (mask, &enabled) in coefficients.iter().enumerate().skip(1) {
        if enabled {
            terms.push(AnfTerm {
                variables: mask as u64,
            });
        }
    }

    Ok(AnfProgram {
        variable_count,
        constant: coefficients[0],
        terms,
    })
}

/// Existing B2/B3 write admission: marker bit0 AND NOT marker bit1.
/// Its canonical Zhegalkin form is x0 XOR (x0*x1).
pub fn marker_acceptance_program() -> AnfProgram {
    synthesize_anf(2, &[false, true, false, false])
        .expect("fixed two-variable marker truth table must synthesize")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlgebraicStreamError {
    RequiresTwoWayB3,
    Anf(AnfSynthesisError),
    Stream(StreamError),
}

impl From<StreamError> for AlgebraicStreamError {
    fn from(value: StreamError) -> Self {
        Self::Stream(value)
    }
}

impl From<AnfSynthesisError> for AlgebraicStreamError {
    fn from(value: AnfSynthesisError) -> Self {
        Self::Anf(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AlgebraicFootprint {
    pub b3: MemoryFootprint,
    pub anf_program_semantic_bits: usize,
    pub wrapper_inline_bytes: usize,
}

/// B4 development control: exact ANF composition in front of B3 write routing.
/// Read addressing and B3 memory semantics remain unchanged on purpose, so the
/// incremental contribution and cost of ANF composition are isolated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AlgebraicBooleanStream {
    inner: BooleanStream,
    admission: AnfProgram,
    anf_work: ResourceCounters,
}

impl AlgebraicBooleanStream {
    pub fn new(config: StreamConfig) -> Result<Self, AlgebraicStreamError> {
        if config.mode != MemoryMode::TwoWay {
            return Err(AlgebraicStreamError::RequiresTwoWayB3);
        }
        Ok(Self {
            inner: BooleanStream::new(config)?,
            admission: marker_acceptance_program(),
            anf_work: ResourceCounters::default(),
        })
    }

    #[must_use]
    pub const fn arm(&self) -> ArchitectureArm {
        ArchitectureArm::B4BooleanAnf
    }

    #[must_use]
    pub fn config(&self) -> StreamConfig {
        self.inner.config()
    }

    #[must_use]
    pub fn counters(&self) -> StreamCounters {
        self.inner.counters()
    }

    #[must_use]
    pub fn anf_work(&self) -> ResourceCounters {
        self.anf_work
    }

    #[must_use]
    pub fn admission_program(&self) -> &AnfProgram {
        &self.admission
    }

    #[must_use]
    pub fn footprint(&self) -> AlgebraicFootprint {
        AlgebraicFootprint {
            b3: self.inner.footprint(),
            anf_program_semantic_bits: self.admission.semantic_bits(),
            wrapper_inline_bytes: std::mem::size_of::<Self>(),
        }
    }

    pub fn reset(&mut self) {
        self.inner.reset();
        self.anf_work = ResourceCounters::default();
    }

    /// Valid marker values are algebraically classified before entering B3.
    /// Invalid marker values are passed unchanged so B3 rejects them atomically
    /// and no ANF work is spuriously charged for a rejected API call.
    pub fn step(&mut self, event: Event) -> Result<StepOutput, AlgebraicStreamError> {
        let mapped = match event {
            Event::Write {
                key,
                payload,
                marker,
            } if marker.bits() <= 3 => {
                let active = self
                    .admission
                    .evaluate_counted(marker.bits(), &mut self.anf_work)?;
                Event::Write {
                    key,
                    payload,
                    marker: super::tdi21::BooleanState::from_bits(if active { 1 } else { 3 }),
                }
            }
            other => other,
        };
        Ok(self.inner.step(mapped)?)
    }
}
