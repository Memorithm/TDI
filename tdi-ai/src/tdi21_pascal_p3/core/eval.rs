use super::super::types::{Error, GATES};
use super::bank::Bank;
use super::measure::checked_add;

pub(crate) fn evaluate_bank(
    bank: &Bank,
    assignment: usize,
    term_tests: &mut u64,
) -> Result<u8, Error> {
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
