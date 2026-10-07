//! Prospective TDI-21 Pascal P5 realized-cost benchmark support.
//!
//! P5 is Development/Validation only. It freezes a host/toolchain-specific timing
//! procedure over the already-frozen P4 semantic geometry. No protected/final
//! population is represented here.

use std::cmp::Reverse;
use std::hint::black_box;
use std::time::Instant;

use crate::tdi21_pascal_p4::{self, LoadRegion, Schedule, Split};

pub const SEMANTICS: &str = "tdi21-pascal-p5-realized-cost-v1";
pub const WARMUP_ROUNDS: usize = 1;
pub const MEASURED_ROUNDS: usize = 6;
const GATES: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arm {
    Pascal,
    Direct,
    Generic,
}

impl Arm {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Pascal => "pascal",
            Self::Direct => "direct",
            Self::Generic => "generic",
        }
    }

    const fn index(self) -> usize {
        match self {
            Self::Pascal => 0,
            Self::Direct => 1,
            Self::Generic => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    P4(tdi21_pascal_p4::Error),
    DuplicateGeneratedMask,
    SemanticMismatch,
    TimingDeltaOverflow,
}

impl From<tdi21_pascal_p4::Error> for Error {
    fn from(value: tdi21_pascal_p4::Error) -> Self {
        Self::P4(value)
    }
}

#[derive(Clone, Debug)]
struct Bank {
    n: u8,
    k: usize,
    density: usize,
    constants: u8,
    terms: Vec<Vec<usize>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimedCell {
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
    pub pairwise_token_comparisons: u64,
    pub p4_work: [u64; 3],
    pub p4_deltas: [i128; 2],
    pub raw_ns: [[u128; MEASURED_ROUNDS]; 3],
    pub median_ns: [u128; 3],
    pub delta_ns: [i128; 2],
}

impl TimedCell {
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
        fields.push(self.pairwise_token_comparisons.to_string());
        fields.extend(self.p4_work.map(|x| x.to_string()));
        fields.extend(self.p4_deltas.map(|x| x.to_string()));
        for arm in 0..3 {
            fields.extend(self.raw_ns[arm].map(|x| x.to_string()));
        }
        fields.extend(self.median_ns.map(|x| x.to_string()));
        fields.extend(self.delta_ns.map(|x| x.to_string()));
        fields.push(SEMANTICS.to_owned());
        fields.join("\t")
    }
}

pub fn canonical_header() -> &'static str {
    "split\tn\tk\tdensity\tschedule\tregion\ttarget_load\tquery_load\treuse\tchecksum_pascal\tchecksum_direct\tchecksum_generic\tmismatch_pascal\tmismatch_generic\tpairwise_token_comparisons\tw_pascal\tw_direct\tw_generic\tp4_delta_direct\tp4_delta_generic\tpascal_ns_r0\tpascal_ns_r1\tpascal_ns_r2\tpascal_ns_r3\tpascal_ns_r4\tpascal_ns_r5\tdirect_ns_r0\tdirect_ns_r1\tdirect_ns_r2\tdirect_ns_r3\tdirect_ns_r4\tdirect_ns_r5\tgeneric_ns_r0\tgeneric_ns_r1\tgeneric_ns_r2\tgeneric_ns_r3\tgeneric_ns_r4\tgeneric_ns_r5\tmedian_ns_pascal\tmedian_ns_direct\tmedian_ns_generic\tdelta_ns_direct\tdelta_ns_generic\tsemantics"
}

pub const fn timing_order(round: usize) -> [Arm; 3] {
    match round % 3 {
        0 => [Arm::Pascal, Arm::Direct, Arm::Generic],
        1 => [Arm::Direct, Arm::Generic, Arm::Pascal],
        _ => [Arm::Generic, Arm::Pascal, Arm::Direct],
    }
}

pub fn median_ns_six(mut samples: [u128; MEASURED_ROUNDS]) -> u128 {
    samples.sort_unstable();
    (samples[2] + samples[3]) / 2
}

pub const fn declared_cell_count(split: Split) -> usize {
    match split {
        Split::Development => 108,
        Split::Validation => 54,
    }
}

pub const fn declared_widths(split: Split) -> &'static [u8] {
    match split {
        Split::Development => &[10, 12],
        Split::Validation => &[15],
    }
}

pub fn canonical_plan_header() -> &'static str {
    "split\tn\tk\tdensity\tschedule\tregion\ttarget_load\tquery_load\treuse\tsemantics"
}

pub fn canonical_plan_records(split: Split) -> Result<Vec<String>, Error> {
    let mut records = Vec::with_capacity(declared_cell_count(split));
    each_declared_cell(split, |n, density, schedule, region, reuse| {
        let k = domain(n)?;
        let target_load = tdi21_pascal_p4::rounded_load(n, density, region)?;
        let query_load = match reuse {
            1 => target_load,
            8 => (target_load + 4) / 8,
            _ => return Err(Error::P4(tdi21_pascal_p4::Error::InvalidReuse)),
        };
        records.push(
            [
                split.name().to_owned(),
                n.to_string(),
                k.to_string(),
                density.to_string(),
                schedule.name().to_owned(),
                region.name().to_owned(),
                target_load.to_string(),
                query_load.to_string(),
                reuse.to_string(),
                SEMANTICS.to_owned(),
            ]
            .join("\t"),
        );
        Ok(())
    })?;
    Ok(records)
}

fn domain(n: u8) -> Result<usize, Error> {
    match n {
        10 | 12 | 15 => Ok(1usize << n),
        _ => Err(Error::P4(tdi21_pascal_p4::Error::UnsupportedWidth)),
    }
}

fn generate(n: u8, density: usize) -> Result<Bank, Error> {
    let k = domain(n)?;
    if !matches!(density, 4 | 16 | 64) || density >= k {
        return Err(Error::P4(tdi21_pascal_p4::Error::InvalidDensity));
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

fn addresses(n: u8, schedule: Schedule, query_load: usize) -> Result<Vec<usize>, Error> {
    let k = domain(n)?;
    if query_load == 0 || query_load > k {
        return Err(Error::P4(tdi21_pascal_p4::Error::InvalidQueryLoad));
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

fn evaluate_bank(bank: &Bank, assignment: usize) -> u8 {
    let mut packed = 0u8;
    for gate in 0..GATES {
        let mut value = ((bank.constants >> gate) & 1) != 0;
        for &term in &bank.terms[gate] {
            if assignment & term == term {
                value = !value;
            }
        }
        if value {
            packed |= 1u8 << gate;
        }
    }
    packed
}

fn pascal_table(bank: &Bank) -> Vec<u8> {
    let mut table = vec![0u8; bank.k];
    table[0] = bank.constants;
    for gate in 0..GATES {
        let lane = 1u8 << gate;
        for &mask in &bank.terms[gate] {
            table[mask] ^= lane;
        }
    }
    for bit in 0..bank.n {
        let selector = 1usize << bit;
        for mask in 0..bank.k {
            if mask & selector != 0 {
                table[mask] ^= table[mask ^ selector];
            }
        }
    }
    table
}

fn generic_table(bank: &Bank) -> Vec<u8> {
    (0..bank.k)
        .map(|assignment| evaluate_bank(bank, assignment))
        .collect()
}

fn checksum_update(hash: u64, address: usize, value: u8) -> u64 {
    let mixed = hash ^ (address as u64).rotate_left(17) ^ u64::from(value);
    mixed.wrapping_mul(0x1000_0000_01b3)
}

fn table_checksum(table: &[u8], query_addresses: &[usize], reuse: usize) -> u64 {
    let mut checksum = 0xcbf2_9ce4_8422_2325;
    for _ in 0..reuse {
        for &address in query_addresses {
            checksum = checksum_update(checksum, address, table[address]);
        }
    }
    checksum
}

fn direct_checksum(bank: &Bank, query_addresses: &[usize], reuse: usize) -> u64 {
    let mut checksum = 0xcbf2_9ce4_8422_2325;
    for _ in 0..reuse {
        for &address in query_addresses {
            checksum = checksum_update(checksum, address, evaluate_bank(bank, address));
        }
    }
    checksum
}

fn run_arm(arm: Arm, bank: &Bank, query_addresses: &[usize], reuse: usize) -> u64 {
    match arm {
        Arm::Pascal => {
            let table = pascal_table(bank);
            table_checksum(&table, query_addresses, reuse)
        }
        Arm::Direct => direct_checksum(bank, query_addresses, reuse),
        Arm::Generic => {
            let table = generic_table(bank);
            table_checksum(&table, query_addresses, reuse)
        }
    }
}

fn measure_arm(arm: Arm, bank: &Bank, query_addresses: &[usize], reuse: usize) -> (u64, u128) {
    let start = Instant::now();
    let checksum = run_arm(
        arm,
        black_box(bank),
        black_box(query_addresses),
        black_box(reuse),
    );
    let elapsed = start.elapsed().as_nanos();
    (black_box(checksum), elapsed)
}

fn signed_ns(control: u128, pascal: u128) -> Result<i128, Error> {
    let control = i128::try_from(control).map_err(|_| Error::TimingDeltaOverflow)?;
    let pascal = i128::try_from(pascal).map_err(|_| Error::TimingDeltaOverflow)?;
    Ok(control - pascal)
}

pub fn verify_semantic_cell(
    split: Split,
    n: u8,
    density: usize,
    schedule: Schedule,
    region: LoadRegion,
    reuse: usize,
) -> Result<(), Error> {
    let p4 = tdi21_pascal_p4::run_cell(split, n, density, schedule, region, reuse)?;
    if p4.mismatches != [0, 0] || p4.pairwise_token_comparisons != 0 {
        return Err(Error::SemanticMismatch);
    }
    let bank = generate(n, density)?;
    if bank.density != density {
        return Err(Error::SemanticMismatch);
    }
    let query_addresses = addresses(n, schedule, p4.query_load)?;
    for arm in [Arm::Pascal, Arm::Direct, Arm::Generic] {
        let checksum = run_arm(arm, &bank, &query_addresses, reuse);
        if checksum != p4.checksums[arm.index()] {
            return Err(Error::SemanticMismatch);
        }
    }
    Ok(())
}

pub fn run_timed_cell(
    split: Split,
    n: u8,
    density: usize,
    schedule: Schedule,
    region: LoadRegion,
    reuse: usize,
) -> Result<TimedCell, Error> {
    let p4 = tdi21_pascal_p4::run_cell(split, n, density, schedule, region, reuse)?;
    if p4.mismatches != [0, 0] || p4.pairwise_token_comparisons != 0 {
        return Err(Error::SemanticMismatch);
    }
    let bank = generate(n, density)?;
    let query_addresses = addresses(n, schedule, p4.query_load)?;

    for arm in [Arm::Pascal, Arm::Direct, Arm::Generic] {
        let checksum = black_box(run_arm(arm, &bank, &query_addresses, reuse));
        if checksum != p4.checksums[arm.index()] {
            return Err(Error::SemanticMismatch);
        }
    }

    let mut raw_ns = [[0u128; MEASURED_ROUNDS]; 3];
    for (round, order) in (0..MEASURED_ROUNDS).map(timing_order).enumerate() {
        for arm in order {
            let (checksum, elapsed) = measure_arm(arm, &bank, &query_addresses, reuse);
            if checksum != p4.checksums[arm.index()] {
                return Err(Error::SemanticMismatch);
            }
            raw_ns[arm.index()][round] = elapsed;
        }
    }

    let median_ns = [
        median_ns_six(raw_ns[0]),
        median_ns_six(raw_ns[1]),
        median_ns_six(raw_ns[2]),
    ];
    let delta_ns = [
        signed_ns(median_ns[1], median_ns[0])?,
        signed_ns(median_ns[2], median_ns[0])?,
    ];

    Ok(TimedCell {
        split,
        n,
        k: p4.k,
        density,
        schedule,
        region,
        target_load: p4.target_load,
        query_load: p4.query_load,
        reuse,
        checksums: p4.checksums,
        mismatches: p4.mismatches,
        pairwise_token_comparisons: p4.pairwise_token_comparisons,
        p4_work: p4.work,
        p4_deltas: p4.deltas,
        raw_ns,
        median_ns,
        delta_ns,
    })
}

fn each_declared_cell<F>(split: Split, mut visit: F) -> Result<(), Error>
where
    F: FnMut(u8, usize, Schedule, LoadRegion, usize) -> Result<(), Error>,
{
    for &n in declared_widths(split) {
        for density in [4usize, 16, 64] {
            for schedule in [
                Schedule::Affine,
                Schedule::LowWeightFirst,
                Schedule::HighWeightFirst,
            ] {
                for region in [LoadRegion::Low, LoadRegion::Cross, LoadRegion::High] {
                    for reuse in [1usize, 8] {
                        visit(n, density, schedule, region, reuse)?;
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn verify_declared_semantics(split: Split) -> Result<usize, Error> {
    let mut count = 0usize;
    each_declared_cell(split, |n, density, schedule, region, reuse| {
        verify_semantic_cell(split, n, density, schedule, region, reuse)?;
        count += 1;
        Ok(())
    })?;
    Ok(count)
}

pub fn run_timing_matrix(split: Split) -> Result<Vec<TimedCell>, Error> {
    let mut cells = Vec::with_capacity(declared_cell_count(split));
    each_declared_cell(split, |n, density, schedule, region, reuse| {
        cells.push(run_timed_cell(split, n, density, schedule, region, reuse)?);
        Ok(())
    })?;
    debug_assert_eq!(cells.len(), declared_cell_count(split));
    Ok(cells)
}
