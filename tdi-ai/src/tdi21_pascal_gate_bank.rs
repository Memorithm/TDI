//! TDI-21 Pascal/ANF gate-bank P1 Development/Validation harness.
//!
//! This module implements the preregistered non-final execution comparison in
//! docs/TDI-21.0-PASCAL-GATE-BANK-P1-PREREGISTRATION.md. It is an exact,
//! dependency-free research harness. It does not authorize confirmation or
//! downstream model/runtime promotion.

use std::cmp::Reverse;

pub const PASCAL_P1_SEMANTICS: &str = "tdi21-pascal-gate-bank-p1-v1";
pub const PASCAL_P1_GATE_COUNT: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PascalP1Split {
    Development,
    Validation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PascalP1Schedule {
    Affine,
    LowWeightFirst,
    HighWeightFirst,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PascalP1Decision {
    RejectImplementationOrProtocol,
    RetainDevelopmentOnlyNoGeneralization,
    RetainNonfinalPascalGateBankUtility,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PascalP1Error {
    UnsupportedWidth,
    InvalidDensity,
    InvalidQueryLoad,
    SplitWidthMismatch,
    DuplicateGeneratedMask,
    CounterOverflow,
    AllocationFailed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PascalP1GateBank {
    variable_count: u8,
    domain_size: usize,
    density: usize,
    constants: u8,
    terms: Vec<Vec<usize>>,
}

impl PascalP1GateBank {
    #[must_use]
    pub const fn variable_count(&self) -> u8 {
        self.variable_count
    }

    #[must_use]
    pub const fn domain_size(&self) -> usize {
        self.domain_size
    }

    #[must_use]
    pub const fn density(&self) -> usize {
        self.density
    }

    #[must_use]
    pub const fn constants(&self) -> u8 {
        self.constants
    }

    #[must_use]
    pub fn terms(&self, gate: usize) -> &[usize] {
        &self.terms[gate]
    }

    #[must_use]
    pub fn anf_semantic_bits(&self) -> usize {
        PASCAL_P1_GATE_COUNT * 9 + PASCAL_P1_GATE_COUNT * self.density * 64
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PascalP1CellEvidence {
    pub split: PascalP1Split,
    pub variable_count: u8,
    pub domain_size: usize,
    pub density: usize,
    pub schedule: PascalP1Schedule,
    pub query_load: usize,
    pub reuse: usize,
    pub pascal_checksum: u64,
    pub direct_checksum: u64,
    pub generic_checksum: u64,
    pub pascal_mismatches: u64,
    pub generic_mismatches: u64,
    pub pascal_zeta_xors: u64,
    pub direct_term_tests: u64,
    pub generic_term_tests: u64,
    pub query_lookups: u64,
    pub coefficient_bytes: usize,
    pub materialized_table_bytes: usize,
    pub anf_semantic_bits: usize,
    pub pairwise_token_comparisons: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PascalP1Gates {
    pub g1_exactness: bool,
    pub g2_representation_accounting: bool,
    pub g3_tdi21_prohibition: bool,
    pub g4_development_structural_utility: bool,
    pub g5_validation_generalization: bool,
    pub g6_sparse_boundary_retained: bool,
}

fn frozen_width(variable_count: u8) -> bool {
    matches!(variable_count, 6 | 8 | 10 | 12)
}

fn split_allows(split: PascalP1Split, variable_count: u8) -> bool {
    match split {
        PascalP1Split::Development => matches!(variable_count, 6 | 8),
        PascalP1Split::Validation => matches!(variable_count, 10 | 12),
    }
}

pub fn generate_pascal_p1_bank(
    variable_count: u8,
    density: usize,
) -> Result<PascalP1GateBank, PascalP1Error> {
    if !frozen_width(variable_count) {
        return Err(PascalP1Error::UnsupportedWidth);
    }
    let domain_size = 1usize << variable_count;
    if density == 0 || density >= domain_size {
        return Err(PascalP1Error::InvalidDensity);
    }

    let mut terms = Vec::new();
    terms
        .try_reserve_exact(PASCAL_P1_GATE_COUNT)
        .map_err(|_| PascalP1Error::AllocationFailed)?;
    let mut constants = 0u8;

    for gate in 0..PASCAL_P1_GATE_COUNT {
        if gate % 2 == 1 {
            constants |= 1u8 << gate;
        }
        let mut seen = vec![false; domain_size];
        let mut gate_terms = Vec::new();
        gate_terms
            .try_reserve_exact(density)
            .map_err(|_| PascalP1Error::AllocationFailed)?;
        for term in 0..density {
            let mask = 1 + ((257usize * gate + 73usize * term) % (domain_size - 1));
            if seen[mask] {
                return Err(PascalP1Error::DuplicateGeneratedMask);
            }
            seen[mask] = true;
            gate_terms.push(mask);
        }
        terms.push(gate_terms);
    }

    Ok(PascalP1GateBank {
        variable_count,
        domain_size,
        density,
        constants,
        terms,
    })
}

fn checked_add(target: &mut u64, amount: u64) -> Result<(), PascalP1Error> {
    *target = target
        .checked_add(amount)
        .ok_or(PascalP1Error::CounterOverflow)?;
    Ok(())
}

fn evaluate_gate(
    bank: &PascalP1GateBank,
    gate: usize,
    assignment: usize,
    term_tests: &mut u64,
) -> Result<bool, PascalP1Error> {
    let mut value = ((bank.constants >> gate) & 1) != 0;
    for &term in &bank.terms[gate] {
        checked_add(term_tests, 1)?;
        if assignment & term == term {
            value = !value;
        }
    }
    Ok(value)
}

fn evaluate_bank(
    bank: &PascalP1GateBank,
    assignment: usize,
    term_tests: &mut u64,
) -> Result<u8, PascalP1Error> {
    let mut value = 0u8;
    for gate in 0..PASCAL_P1_GATE_COUNT {
        if evaluate_gate(bank, gate, assignment, term_tests)? {
            value |= 1u8 << gate;
        }
    }
    Ok(value)
}

pub fn pascal_p1_materialize(bank: &PascalP1GateBank) -> Result<(Vec<u8>, u64), PascalP1Error> {
    let mut table = vec![0u8; bank.domain_size];
    table[0] = bank.constants;
    for gate in 0..PASCAL_P1_GATE_COUNT {
        let lane = 1u8 << gate;
        for &mask in &bank.terms[gate] {
            table[mask] ^= lane;
        }
    }

    let mut xor_count = 0u64;
    for bit in 0..bank.variable_count {
        let selector = 1usize << bit;
        for mask in 0..bank.domain_size {
            if mask & selector != 0 {
                let lower = table[mask ^ selector];
                table[mask] ^= lower;
                checked_add(&mut xor_count, 1)?;
            }
        }
    }
    Ok((table, xor_count))
}

pub fn generic_p1_materialize(bank: &PascalP1GateBank) -> Result<(Vec<u8>, u64), PascalP1Error> {
    let mut table = Vec::new();
    table
        .try_reserve_exact(bank.domain_size)
        .map_err(|_| PascalP1Error::AllocationFailed)?;
    let mut term_tests = 0u64;
    for assignment in 0..bank.domain_size {
        table.push(evaluate_bank(bank, assignment, &mut term_tests)?);
    }
    Ok((table, term_tests))
}

pub fn pascal_p1_query_addresses(
    variable_count: u8,
    schedule: PascalP1Schedule,
    query_load: usize,
) -> Result<Vec<usize>, PascalP1Error> {
    if !frozen_width(variable_count) {
        return Err(PascalP1Error::UnsupportedWidth);
    }
    let domain_size = 1usize << variable_count;
    if query_load == 0 || query_load > domain_size {
        return Err(PascalP1Error::InvalidQueryLoad);
    }

    let mut addresses: Vec<usize> = match schedule {
        PascalP1Schedule::Affine => (0..domain_size)
            .map(|index| (17usize + 37usize * index) % domain_size)
            .collect(),
        PascalP1Schedule::LowWeightFirst => (0..domain_size).collect(),
        PascalP1Schedule::HighWeightFirst => (0..domain_size).collect(),
    };
    match schedule {
        PascalP1Schedule::Affine => {}
        PascalP1Schedule::LowWeightFirst => {
            addresses.sort_by_key(|&address| (address.count_ones(), address));
        }
        PascalP1Schedule::HighWeightFirst => {
            addresses.sort_by_key(|&address| (Reverse(address.count_ones()), address));
        }
    }
    addresses.truncate(query_load);
    Ok(addresses)
}

fn checksum_update(hash: u64, address: usize, value: u8) -> u64 {
    let mixed = hash ^ (address as u64).rotate_left(17) ^ u64::from(value);
    mixed.wrapping_mul(0x1000_0000_01b3)
}

pub fn run_pascal_p1_cell(
    split: PascalP1Split,
    variable_count: u8,
    density: usize,
    schedule: PascalP1Schedule,
    query_load: usize,
    reuse: usize,
) -> Result<PascalP1CellEvidence, PascalP1Error> {
    if !split_allows(split, variable_count) {
        return Err(PascalP1Error::SplitWidthMismatch);
    }
    if reuse == 0 {
        return Err(PascalP1Error::InvalidQueryLoad);
    }

    let bank = generate_pascal_p1_bank(variable_count, density)?;
    let addresses = pascal_p1_query_addresses(variable_count, schedule, query_load)?;
    let (pascal_table, pascal_zeta_xors) = pascal_p1_materialize(&bank)?;
    let (generic_table, generic_term_tests) = generic_p1_materialize(&bank)?;

    let mut direct_term_tests = 0u64;
    let mut pascal_checksum = 0xcbf2_9ce4_8422_2325;
    let mut direct_checksum = 0xcbf2_9ce4_8422_2325;
    let mut generic_checksum = 0xcbf2_9ce4_8422_2325;
    let mut pascal_mismatches = 0u64;
    let mut generic_mismatches = 0u64;

    for _ in 0..reuse {
        for &address in &addresses {
            let direct = evaluate_bank(&bank, address, &mut direct_term_tests)?;
            let pascal = pascal_table[address];
            let generic = generic_table[address];
            if pascal != direct {
                checked_add(&mut pascal_mismatches, 1)?;
            }
            if generic != direct {
                checked_add(&mut generic_mismatches, 1)?;
            }
            pascal_checksum = checksum_update(pascal_checksum, address, pascal);
            direct_checksum = checksum_update(direct_checksum, address, direct);
            generic_checksum = checksum_update(generic_checksum, address, generic);
        }
    }

    let query_lookups = u64::try_from(addresses.len())
        .ok()
        .and_then(|count| count.checked_mul(reuse as u64))
        .ok_or(PascalP1Error::CounterOverflow)?;

    Ok(PascalP1CellEvidence {
        split,
        variable_count,
        domain_size: bank.domain_size,
        density,
        schedule,
        query_load,
        reuse,
        pascal_checksum,
        direct_checksum,
        generic_checksum,
        pascal_mismatches,
        generic_mismatches,
        pascal_zeta_xors,
        direct_term_tests,
        generic_term_tests,
        query_lookups,
        coefficient_bytes: bank.domain_size,
        materialized_table_bytes: bank.domain_size,
        anf_semantic_bits: bank.anf_semantic_bits(),
        pairwise_token_comparisons: 0,
    })
}

fn frozen_densities(domain_size: usize) -> Vec<usize> {
    let mut values = vec![
        4usize,
        32usize.min(domain_size - 1),
        256usize.min(domain_size - 1),
    ];
    values.sort_unstable();
    values.dedup();
    values
}

fn frozen_query_loads(domain_size: usize) -> Vec<usize> {
    let mut values = vec![
        16usize.min(domain_size),
        64usize.min(domain_size),
        domain_size,
    ];
    values.sort_unstable();
    values.dedup();
    values
}

pub fn run_pascal_p1_matrix() -> Result<Vec<PascalP1CellEvidence>, PascalP1Error> {
    let mut evidence = Vec::new();
    for (split, widths) in [
        (PascalP1Split::Development, [6u8, 8u8]),
        (PascalP1Split::Validation, [10u8, 12u8]),
    ] {
        for variable_count in widths {
            let domain_size = 1usize << variable_count;
            for density in frozen_densities(domain_size) {
                for schedule in [
                    PascalP1Schedule::Affine,
                    PascalP1Schedule::LowWeightFirst,
                    PascalP1Schedule::HighWeightFirst,
                ] {
                    for query_load in frozen_query_loads(domain_size) {
                        for reuse in [1usize, 8usize] {
                            evidence.push(run_pascal_p1_cell(
                                split,
                                variable_count,
                                density,
                                schedule,
                                query_load,
                                reuse,
                            )?);
                        }
                    }
                }
            }
        }
    }
    Ok(evidence)
}

pub fn evaluate_pascal_p1_gates(cells: &[PascalP1CellEvidence]) -> PascalP1Gates {
    let g1_exactness = !cells.is_empty()
        && cells.iter().all(|cell| {
            cell.pascal_mismatches == 0
                && cell.generic_mismatches == 0
                && cell.pascal_checksum == cell.direct_checksum
                && cell.generic_checksum == cell.direct_checksum
        });
    let g2_representation_accounting = !cells.is_empty()
        && cells.iter().all(|cell| {
            cell.pascal_zeta_xors > 0
                && cell.direct_term_tests > 0
                && cell.generic_term_tests > 0
                && cell.query_lookups > 0
                && cell.coefficient_bytes == cell.domain_size
                && cell.materialized_table_bytes == cell.domain_size
                && cell.anf_semantic_bits > 0
        });
    let g3_tdi21_prohibition = !cells.is_empty()
        && cells
            .iter()
            .all(|cell| cell.pairwise_token_comparisons == 0);

    let development_targets: Vec<_> = cells
        .iter()
        .filter(|cell| {
            cell.split == PascalP1Split::Development
                && cell.density >= 32
                && cell.query_load == cell.domain_size
        })
        .collect();
    let g4_development_structural_utility = !development_targets.is_empty()
        && development_targets.iter().all(|cell| {
            cell.pascal_zeta_xors < cell.direct_term_tests
                && cell.pascal_zeta_xors < cell.generic_term_tests
        });

    let validation_targets: Vec<_> = cells
        .iter()
        .filter(|cell| {
            cell.split == PascalP1Split::Validation
                && cell.density >= 32
                && cell.query_load == cell.domain_size
        })
        .collect();
    let g5_validation_generalization = !validation_targets.is_empty()
        && validation_targets.iter().all(|cell| {
            cell.pascal_zeta_xors < cell.direct_term_tests
                && cell.pascal_zeta_xors < cell.generic_term_tests
        });

    let g6_sparse_boundary_retained = cells
        .iter()
        .any(|cell| cell.direct_term_tests < cell.pascal_zeta_xors);

    PascalP1Gates {
        g1_exactness,
        g2_representation_accounting,
        g3_tdi21_prohibition,
        g4_development_structural_utility,
        g5_validation_generalization,
        g6_sparse_boundary_retained,
    }
}

#[must_use]
pub fn pascal_p1_decision(gates: PascalP1Gates) -> PascalP1Decision {
    if !(gates.g1_exactness && gates.g2_representation_accounting && gates.g3_tdi21_prohibition) {
        PascalP1Decision::RejectImplementationOrProtocol
    } else if gates.g4_development_structural_utility && gates.g5_validation_generalization {
        PascalP1Decision::RetainNonfinalPascalGateBankUtility
    } else {
        PascalP1Decision::RetainDevelopmentOnlyNoGeneralization
    }
}