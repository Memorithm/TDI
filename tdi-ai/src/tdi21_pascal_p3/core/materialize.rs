use super::super::types::{Error, GATES};
use super::bank::Bank;
use super::eval::evaluate_bank;
use super::measure::checked_add;

pub(crate) fn pascal(bank: &Bank) -> Result<(Vec<u8>, u64), Error> {
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

pub(crate) fn generic(bank: &Bank) -> Result<(Vec<u8>, u64), Error> {
    let mut table = Vec::with_capacity(bank.k);
    let mut term_tests = 0u64;
    for assignment in 0..bank.k {
        table.push(evaluate_bank(bank, assignment, &mut term_tests)?);
    }
    Ok((table, term_tests))
}
