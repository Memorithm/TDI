use std::cmp::Reverse;

use super::super::types::{Error, Schedule};
use super::bank::domain;

pub(crate) fn addresses(n: u8, schedule: Schedule, query_load: usize) -> Result<Vec<usize>, Error> {
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
