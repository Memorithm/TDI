//! Exact non-final Pascal P4 crossover harness.
//!
//! P4 is Development/Validation only. It preserves matched Pascal/direct/generic
//! arms and the frozen P3 work accounting without any protected/final path.

use std::cmp::Reverse;

pub const SEMANTICS: &str = "tdi21-pascal-p4-v1";
const GATES: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Split {
    Development,
    Validation,
}

impl Split {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Validation => "validation",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Schedule {
    Affine,
    LowWeightFirst,
    HighWeightFirst,
}

impl Schedule {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Affine => "affine",
            Self::LowWeightFirst => "low_weight_first",
            Self::HighWeightFirst => "high_weight_first",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadRegion {
    Low,
    Cross,
    High,
}

impl LoadRegion {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Cross => "cross",
            Self::High => "high",
        }
    }

    fn ratio(self) -> (u128, u128) {
        match self {
            Self::Low => (1, 2),
            Self::Cross => (1, 1),
            Self::High => (2, 1),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    UnsupportedWidth,
    InvalidDensity,
    InvalidQueryLoad,
    InvalidReuse,
    SplitWidthMismatch,
    DuplicateGeneratedMask,
    CounterOverflow,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cell {
    pub split: Split,
    pub n: u8,
    pub k: usize,
    pub density: usize,
    pub schedule: Schedule,
    pub region: LoadRegion,
    pub target_load: usize,
    pub query_load: usize,
    pub reuse: usize,
    pub checksums: [u64; 3],
    pub mismatches: [u64; 2],
    pub counts: [u64; 4],
    pub representation: [usize; 3],
    pub pairwise_token_comparisons: u64,
    pub work: [u64; 3],
    pub deltas: [i128; 2],
}

impl Cell {
    pub fn canonical_record(&self) -> String {
        let mut fields = vec![
            self.split.name().to_owned(),
            self.n.to_string(),
            self.k.to_string(),
            self.density.to_string(),
            self.schedule.name().to_owned(),
            self.region.name().to_owned(),
            self.target_load.to_string(),
            self.query_load.to_string(),
            self.reuse.to_string(),
        ];
        fields.extend(self.checksums.map(|x| x.to_string()));
        fields.extend(self.mismatches.map(|x| x.to_string()));
        fields.extend(self.counts.map(|x| x.to_string()));
        fields.extend(self.representation.map(|x| x.to_string()));
        fields.push(self.pairwise_token_comparisons.to_string());
        fields.extend(self.work.map(|x| x.to_string()));
        fields.extend(self.deltas.map(|x| x.to_string()));
        fields.push(SEMANTICS.to_owned());
        fields.join("\t")
    }
}

pub fn canonical_header() -> &'static str {
    "split\tn\tk\tdensity\tschedule\tregion\ttarget_load\tquery_load\treuse\tchecksum_pascal\tchecksum_direct\tchecksum_generic\tmismatch_pascal\tmismatch_generic\tpascal_zeta_xors\tdirect_term_tests\tgeneric_term_tests\tquery_lookups\tcoefficient_bytes\tmaterialized_table_bytes\tanf_semantic_bits\tpairwise_token_comparisons\tw_pascal\tw_direct\tw_generic\tdelta_direct\tdelta_generic\tsemantics"
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Bank {
    n: u8,
    k: usize,
    density: usize,
    constants: u8,
    terms: Vec<Vec<usize>>,
}

fn domain(n: u8) -> Result<usize, Error> {
    if matches!(n, 10 | 12 | 15) {
        Ok(1usize << n)
    } else {
        Err(Error::UnsupportedWidth)
    }
}

fn generate(n: u8, density: usize) -> Result<Bank, Error> {
    let k = domain(n)?;
    if !matches!(density, 4 | 16 | 64) || density >= k {
        return Err(Error::InvalidDensity);
    }

    let mut terms = Vec::with_capacity(GATES);
    let mut constants = 0u8;
    for gate in 0..GATES {
        if gate % 2 == 1 {
            constants |= 1u8 << gate;
        }
        let mut seen = vec![false; k];
        let mut lane = Vec::with_capacity(density);
        for index in 0..density {
            let mask = 1 + ((257 * gate + 71 * index) % (k - 1));
            if seen[mask] {
                return Err(Error::DuplicateGeneratedMask);
            }
            seen[mask] = true;
            lane.push(mask);
        }
        terms.push(lane);
    }

    Ok(Bank {
        n,
        k,
        density,
        constants,
        terms,
    })
}

fn checked_add(target: &mut u64, amount: u64) -> Result<(), Error> {
    *target = target.checked_add(amount).ok_or(Error::CounterOverflow)?;
    Ok(())
}

fn checked_sum(left: u64, right: u64) -> Result<u64, Error> {
    left.checked_add(right).ok_or(Error::CounterOverflow)
}

fn signed_delta(control: u64, pascal: u64) -> i128 {
    i128::from(control) - i128::from(pascal)
}

fn checksum_update(hash: u64, address: usize, value: u8) -> u64 {
    let mixed = hash ^ (address as u64).rotate_left(17) ^ u64::from(value);
    mixed.wrapping_mul(0x1000_0000_01b3)
}

fn evaluate_bank(bank: &Bank, assignment: usize, term_tests: &mut u64) -> Result<u8, Error> {
    let mut packed = 0u8;
    for gate in 0..GATES {
        let mut value = ((bank.constants >> gate) & 1) != 0;
        for &term in &bank.terms[gate] {
            checked_add(term_tests, 1)?;
            if assignment & term == term {
                value = !value;
            }
        }
        if value {
            packed |= 1u8 << gate;
        }
    }
    Ok(packed)
}

fn pascal(bank: &Bank) -> Result<(Vec<u8>, u64), Error> {
    let mut table = vec![0u8; bank.k];
    table[0] = bank.constants;
    for gate in 0..GATES {
        let lane = 1u8 << gate;
        for &mask in &bank.terms[gate] {
            table[mask] ^= lane;
        }
    }

    let mut xor_count = 0u64;
    for bit in 0..bank.n {
        let selector = 1usize << bit;
        for mask in 0..bank.k {
            if mask & selector != 0 {
                table[mask] ^= table[mask ^ selector];
                checked_add(&mut xor_count, 1)?;
            }
        }
    }
    Ok((table, xor_count))
}

fn generic(bank: &Bank) -> Result<(Vec<u8>, u64), Error> {
    let mut table = Vec::with_capacity(bank.k);
    let mut term_tests = 0u64;
    for assignment in 0..bank.k {
        table.push(evaluate_bank(bank, assignment, &mut term_tests)?);
    }
    Ok((table, term_tests))
}

fn addresses(n: u8, schedule: Schedule, query_load: usize) -> Result<Vec<usize>, Error> {
    let k = domain(n)?;
    if query_load == 0 || query_load > k {
        return Err(Error::InvalidQueryLoad);
    }

    let mut addresses: Vec<usize> = match schedule {
        Schedule::Affine => (0..k)
            .map(|index| (17usize + 37usize * index) % k)
            .collect(),
        Schedule::LowWeightFirst | Schedule::HighWeightFirst => (0..k).collect(),
    };

    match schedule {
        Schedule::Affine => {}
        Schedule::LowWeightFirst => {
            addresses.sort_by_key(|&address| (address.count_ones(), address));
        }
        Schedule::HighWeightFirst => {
            addresses.sort_by_key(|&address| (Reverse(address.count_ones()), address));
        }
    }
    addresses.truncate(query_load);
    Ok(addresses)
}

pub fn rounded_load(n: u8, density: usize, region: LoadRegion) -> Result<usize, Error> {
    let k = domain(n)?;
    if !matches!(density, 4 | 16 | 64) {
        return Err(Error::InvalidDensity);
    }
    let (numerator, denominator) = region.ratio();
    let top = u128::from(n) * (k as u128) * numerator;
    let bottom = 2u128 * (8u128 * density as u128 - 1) * denominator;
    usize::try_from((top + bottom / 2) / bottom)
        .map_err(|_| Error::CounterOverflow)
        .map(|load| load.max(1))
}

fn split_allows(split: Split, n: u8) -> bool {
    match split {
        Split::Development => matches!(n, 10 | 12),
        Split::Validation => n == 15,
    }
}

pub fn run_cell(
    split: Split,
    n: u8,
    density: usize,
    schedule: Schedule,
    region: LoadRegion,
    reuse: usize,
) -> Result<Cell, Error> {
    let k = domain(n)?;
    if !split_allows(split, n) {
        return Err(Error::SplitWidthMismatch);
    }
    if !matches!(reuse, 1 | 8) {
        return Err(Error::InvalidReuse);
    }

    let target_load = rounded_load(n, density, region)?;
    let query_load = match reuse {
        1 => target_load,
        8 => (target_load + 4) / 8,
        _ => return Err(Error::InvalidReuse),
    };
    if query_load == 0 || query_load > k {
        return Err(Error::InvalidQueryLoad);
    }

    let bank = generate(n, density)?;
    let query_addresses = addresses(n, schedule, query_load)?;
    let (pascal_table, pascal_zeta_xors) = pascal(&bank)?;
    let (generic_table, generic_term_tests) = generic(&bank)?;

    let mut direct_term_tests = 0u64;
    let mut checksums = [0xcbf2_9ce4_8422_2325; 3];
    let mut mismatches = [0u64; 2];

    for _ in 0..reuse {
        for &address in &query_addresses {
            let direct = evaluate_bank(&bank, address, &mut direct_term_tests)?;
            let pascal_value = pascal_table[address];
            let generic_value = generic_table[address];

            if pascal_value != direct {
                checked_add(&mut mismatches[0], 1)?;
            }
            if generic_value != direct {
                checked_add(&mut mismatches[1], 1)?;
            }

            checksums[0] = checksum_update(checksums[0], address, pascal_value);
            checksums[1] = checksum_update(checksums[1], address, direct);
            checksums[2] = checksum_update(checksums[2], address, generic_value);
        }
    }

    let query_lookups = u64::try_from(query_addresses.len())
        .ok()
        .and_then(|count| count.checked_mul(reuse as u64))
        .ok_or(Error::CounterOverflow)?;

    let w_pascal = checked_sum(pascal_zeta_xors, query_lookups)?;
    let w_direct = direct_term_tests;
    let w_generic = checked_sum(generic_term_tests, query_lookups)?;

    let anf_semantic_bits = GATES
        .checked_mul(9)
        .and_then(|base| {
            GATES
                .checked_mul(bank.density)
                .and_then(|terms| terms.checked_mul(64))
                .and_then(|bits| base.checked_add(bits))
        })
        .ok_or(Error::CounterOverflow)?;

    Ok(Cell {
        split,
        n,
        k,
        density,
        schedule,
        region,
        target_load,
        query_load,
        reuse,
        checksums,
        mismatches,
        counts: [
            pascal_zeta_xors,
            direct_term_tests,
            generic_term_tests,
            query_lookups,
        ],
        representation: [bank.k, bank.k, anf_semantic_bits],
        pairwise_token_comparisons: 0,
        work: [w_pascal, w_direct, w_generic],
        deltas: [
            signed_delta(w_direct, w_pascal),
            signed_delta(w_generic, w_pascal),
        ],
    })
}

fn run_matrix(split: Split, widths: &[u8]) -> Result<Vec<Cell>, Error> {
    let mut cells = Vec::new();
    for &n in widths {
        for density in [4usize, 16, 64] {
            for schedule in [
                Schedule::Affine,
                Schedule::LowWeightFirst,
                Schedule::HighWeightFirst,
            ] {
                for region in [LoadRegion::Low, LoadRegion::Cross, LoadRegion::High] {
                    for reuse in [1usize, 8] {
                        cells.push(run_cell(split, n, density, schedule, region, reuse)?);
                    }
                }
            }
        }
    }
    Ok(cells)
}

pub fn run_development_matrix() -> Result<Vec<Cell>, Error> {
    let cells = run_matrix(Split::Development, &[10, 12])?;
    debug_assert_eq!(cells.len(), 108);
    Ok(cells)
}

pub fn run_validation_matrix() -> Result<Vec<Cell>, Error> {
    let cells = run_matrix(Split::Validation, &[15])?;
    debug_assert_eq!(cells.len(), 54);
    Ok(cells)
}
