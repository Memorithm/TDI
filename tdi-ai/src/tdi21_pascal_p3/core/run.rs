use super::super::types::{Cell, Error, GATES, Schedule, Split};
use super::bank::{domain, generate};
use super::eval::evaluate_bank;
use super::materialize::{generic, pascal};
use super::measure::{checked_add, checked_sum, checksum_update, signed_delta};
use super::query::addresses;

fn split_allows(split: Split, n: u8) -> bool {
    match split {
        Split::Development => matches!(n, 9 | 11),
        Split::Validation => matches!(n, 13 | 14),
    }
}

pub fn run_cell(
    split: Split,
    n: u8,
    density: usize,
    schedule: Schedule,
    query_load: usize,
    reuse: usize,
) -> Result<Cell, Error> {
    let k = domain(n)?;
    if !split_allows(split, n) {
        return Err(Error::SplitWidthMismatch);
    }
    if !matches!(density, 4 | 32 | 256) {
        return Err(Error::InvalidDensity);
    }
    if !matches!(reuse, 1 | 8) {
        return Err(Error::InvalidReuse);
    }
    if !(query_load == 16 || query_load == 64 || query_load == k) {
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
