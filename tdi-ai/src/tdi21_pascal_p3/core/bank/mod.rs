mod shape;
mod terms;

use super::super::types::{Error, GATES};

pub(crate) use shape::domain;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Bank {
    pub(crate) n: u8,
    pub(crate) k: usize,
    pub(crate) density: usize,
    pub(crate) constants: u8,
    pub(crate) terms: Vec<Vec<usize>>,
}

pub(crate) fn generate(n: u8, requested_density: usize) -> Result<Bank, Error> {
    let k = shape::domain(n)?;
    shape::density(requested_density, k)?;

    let mut terms = Vec::with_capacity(GATES);
    let mut constants = 0u8;
    for gate in 0..GATES {
        if gate % 2 == 1 {
            constants |= 1u8 << gate;
        }
        terms.push(terms::lane(gate, requested_density, k)?);
    }

    Ok(Bank {
        n,
        k,
        density: requested_density,
        constants,
        terms,
    })
}
